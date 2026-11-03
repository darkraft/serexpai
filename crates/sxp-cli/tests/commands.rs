use std::ffi::OsString;
use std::path::{Path, PathBuf};

use sxp_cli::{Fail, Fs, MemFs, Output, StdFs, line, normalize, parent_of, run_with};
use sxp_encoder::{LengthPolicy, SearchMode, all_baselines, encode_codebook, generate_codebook};
use sxp_kernel::LENGTH_LIMIT;

fn argv(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

fn run(fs: &mut dyn Fs, args: &[&str]) -> Output {
    run_with(argv(args), fs)
}

fn assert_token(out: &Output, command: Option<&str>, token: &str) {
    assert_eq!(out.code, 1, "stdout={} stderr={}", out.stdout, out.stderr);
    assert!(out.stdout.is_empty());
    assert_eq!(out.stderr, line(command, token));
}

fn lcg(seed: u64, n: usize) -> Vec<u8> {
    let mut state = seed;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        state = state.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
        out.push(u8::try_from(state >> 56).unwrap_or(0));
    }
    out
}

fn seeded(len: usize) -> Vec<u8> {
    let mut bytes = lcg(0x11_11, len.max(1));
    bytes.truncate(len);
    bytes
}

#[test]
fn help_exits_0_and_usage_exits_1() {
    let mut fs = MemFs::new();
    let help = run(&mut fs, &["sxp-cli", "--help"]);
    assert_eq!(help.code, 0);
    assert!(help.stderr.is_empty());
    assert!(help.stdout.contains("compress"));
    assert!(help.stdout.contains("decompress"));

    let command_help = run(&mut fs, &["sxp-cli", "compress", "--help"]);
    assert_eq!(command_help.code, 0);
    assert!(command_help.stderr.is_empty());
    assert!(command_help.stdout.contains("compress"));

    let bare = run(&mut fs, &["sxp-cli"]);
    assert_token(&bare, None, "Usage");

    let missing = run(&mut fs, &["sxp-cli", "compress", "src"]);
    assert_token(&missing, Some("compress"), "Usage");

    let extra = run(&mut fs, &["sxp-cli", "decompress", "src", "dst", "more"]);
    assert_token(&extra, Some("decompress"), "Usage");
}

#[test]
fn matrix_round_trips_and_matches_every_thread() {
    let random = lcg(0x11_11, 64);
    let mut rows = vec![
        Vec::new(),
        seeded(1),
        seeded(255),
        seeded(256),
        seeded(257),
        seeded(1024),
        seeded(1025),
        seeded(2048),
        random.clone(),
    ];
    let encoded = {
        let mut fs = MemFs::new();
        fs.insert_file(Path::new("src"), random.clone(), random.len() as u64);
        let out = run(&mut fs, &["sxp-cli", "compress", "src", "dst"]);
        assert_eq!(out.code, 0, "{}", out.stderr);
        fs.bytes(Path::new("dst")).unwrap()
    };
    rows.push(encoded);

    let codebook = generate_codebook("fixture-v1").unwrap();
    let baselines = all_baselines();
    for input in rows {
        let mut fs = MemFs::new();
        let len = u64::try_from(input.len()).unwrap();
        fs.insert_file(Path::new("src"), input.clone(), len);
        let compressed = run(&mut fs, &["sxp-cli", "compress", "src", "dst"]);
        assert_eq!(compressed.code, 0, "{}", compressed.stderr);
        assert!(compressed.stdout.is_empty());
        assert!(compressed.stderr.is_empty());
        let bytes = fs.bytes(Path::new("dst")).unwrap();
        assert!(
            bytes.len() as u64 <= len + 11,
            "len {len} got {}",
            bytes.len()
        );

        let mut back = MemFs::new();
        back.insert_file(Path::new("src"), bytes.clone(), bytes.len() as u64);
        let restored = run(&mut back, &["sxp-cli", "decompress", "src", "out"]);
        assert_eq!(restored.code, 0, "{}", restored.stderr);
        assert_eq!(back.bytes(Path::new("out")).unwrap(), input);

        for threads in [0usize, 1, 2] {
            let got = encode_codebook(
                &input,
                &baselines,
                &codebook,
                LengthPolicy::All,
                SearchMode::HeaderPrune,
                threads,
            )
            .unwrap();
            assert_eq!(got, bytes, "threads {threads} len {len}");
        }
        let reference = encode_codebook(
            &input,
            &baselines,
            &codebook,
            LengthPolicy::All,
            SearchMode::Reference,
            0,
        )
        .unwrap();
        assert_eq!(reference, bytes);
    }
}

#[test]
fn malformed_inputs_leave_the_destination_unchanged() {
    let cases = [
        (Vec::new(), "TruncatedHeader"),
        (vec![0x00], "BadMagic"),
        (
            vec![0x8D, 0x11, 0x00, 0x00, 0x00, 0x00, 0x01],
            "CrcMismatch",
        ),
        (
            vec![0x8D, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            "TrailingBytes",
        ),
        (
            include_bytes!("../../../spec/vectors/reject/leb-overflow.bin").to_vec(),
            "Leb128Overflow",
        ),
    ];
    for (bytes, token) in cases {
        let mut fs = MemFs::new();
        let len = u64::try_from(bytes.len()).unwrap_or(0);
        fs.insert_file(Path::new("src"), bytes, len);
        fs.insert_file(Path::new("dst"), b"OLD".to_vec(), 3);
        fs.insert_file(Path::new(".sxp-tmp-keep"), b"keep".to_vec(), 4);
        let out = run(&mut fs, &["sxp-cli", "decompress", "src", "dst"]);
        assert_token(&out, Some("decompress"), token);
        assert_eq!(fs.bytes(Path::new("dst")).unwrap(), b"OLD");
        assert_eq!(fs.bytes(Path::new(".sxp-tmp-keep")).unwrap(), b"keep");
        assert!(fs.temp_names().is_empty());
    }

    let mut missing = MemFs::new();
    missing.insert_file(Path::new("src"), vec![0x00], 1);
    let out = run(&mut missing, &["sxp-cli", "decompress", "src", "dst"]);
    assert_token(&out, Some("decompress"), "BadMagic");
    assert!(!missing.contains(Path::new("dst")));
}

#[test]
fn length_limit_is_declared_n_on_decompress_and_metadata_on_compress() {
    let mut fs = MemFs::new();
    fs.insert_file(Path::new("src"), Vec::new(), LENGTH_LIMIT);
    let out = run(&mut fs, &["sxp-cli", "compress", "src", "dst"]);
    assert_token(&out, Some("compress"), "LengthOutOfRange");
    assert_eq!(fs.opens(), 0);
    assert!(!fs.contains(Path::new("dst")));

    let empty = vec![0x8D, 0x11, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut dec = MemFs::new();
    let len = empty.len() as u64;
    dec.insert_file(Path::new("src"), empty, LENGTH_LIMIT);
    let out = run(&mut dec, &["sxp-cli", "decompress", "src", "dst"]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(dec.bytes(Path::new("dst")).unwrap().is_empty());
    assert!(len < LENGTH_LIMIT);
}

#[test]
fn failures_remove_only_the_temp_file() {
    let rows = [
        ("compress", Fail::OpenRead(1), false),
        ("compress", Fail::OpenRead(2), true),
        ("compress", Fail::Write, true),
        ("compress", Fail::Flush, true),
        ("compress", Fail::Sync, true),
        ("compress", Fail::Rename, true),
        ("decompress", Fail::OpenRead(1), false),
        ("decompress", Fail::ReadBody, true),
        ("decompress", Fail::Write, true),
        ("decompress", Fail::Flush, true),
        ("decompress", Fail::Sync, true),
        ("decompress", Fail::Rename, true),
    ];
    let mut setup = MemFs::new();
    setup.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    let setup_out = run(&mut setup, &["sxp-cli", "compress", "src", "dst"]);
    assert_eq!(setup_out.code, 0, "{}", setup_out.stderr);
    let bitstream = setup.bytes(Path::new("dst")).unwrap();
    assert!(!bitstream.is_empty());

    for (command, fail, created) in rows {
        let mut fs = MemFs::new();
        let src = if command == "compress" {
            b"abc".to_vec()
        } else {
            bitstream.clone()
        };
        fs.insert_file(Path::new("src"), src.clone(), src.len() as u64);
        fs.insert_file(Path::new("dst"), b"OLD".to_vec(), 3);
        fs.insert_file(Path::new(".sxp-tmp-keep"), b"keep".to_vec(), 4);
        fs.fail(fail);
        let out = run(&mut fs, &["sxp-cli", command, "src", "dst"]);
        assert_eq!(out.code, 1, "{command} {fail:?} {}", out.stderr);
        assert!(out.stdout.is_empty());
        assert_eq!(
            fs.bytes(Path::new("dst")).as_deref(),
            Some(b"OLD".as_slice())
        );
        assert_eq!(fs.bytes(Path::new("src")).as_deref(), Some(src.as_slice()));
        assert_eq!(fs.bytes(Path::new(".sxp-tmp-keep")).unwrap(), b"keep");
        assert!(fs.temp_names().is_empty(), "{command} {fail:?}");
        assert_eq!(fs.creates() > 0, created, "{command} {fail:?}");
    }
}

#[test]
fn alias_and_shape_refusals_create_nothing() {
    let mut same = MemFs::new();
    same.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    let out = run(&mut same, &["sxp-cli", "compress", "src", "./src"]);
    assert_token(&out, Some("compress"), "SameFile");
    assert_eq!(same.bytes(Path::new("src")).unwrap(), b"abc");

    let mut dir = MemFs::new();
    dir.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    dir.insert_dir(Path::new("dst"));
    let out = run(&mut dir, &["sxp-cli", "compress", "src", "dst"]);
    assert_token(&out, Some("compress"), "IsADirectory");

    let mut slash = MemFs::new();
    slash.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    let out = run(&mut slash, &["sxp-cli", "compress", "src", "dst/"]);
    assert_token(&out, Some("compress"), "Usage");
    assert_eq!(slash.creates(), 0);
    assert!(!slash.contains(Path::new("dst")));

    let mut dot = MemFs::new();
    dot.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    let out = run(&mut dot, &["sxp-cli", "compress", "src", "dst/."]);
    assert_token(&out, Some("compress"), "Usage");
    assert_eq!(dot.creates(), 0);

    let mut via = MemFs::new();
    via.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    via.insert_dir(Path::new("real"));
    via.insert_symlink(Path::new("link"), Path::new("real"));
    let out = run(&mut via, &["sxp-cli", "compress", "src", "link/out"]);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(via.bytes(Path::new("link/out")).is_some());

    let mut parent = MemFs::new();
    parent.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    let out = run(&mut parent, &["sxp-cli", "compress", "src", "missing/out"]);
    assert_token(&out, Some("compress"), "NotFound");

    let mut linked = MemFs::new();
    linked.insert_dir(Path::new("sub"));
    linked.insert_symlink(Path::new("mid"), Path::new("sub"));
    linked.insert_symlink(Path::new("src"), Path::new("mid"));
    let out = run(&mut linked, &["sxp-cli", "decompress", "src", "dst"]);
    assert_token(&out, Some("decompress"), "IsADirectory");

    let mut denied = MemFs::new();
    denied.insert_file(Path::new("src"), b"abc".to_vec(), 3);
    denied.fail(Fail::Stat);
    let out = run(&mut denied, &["sxp-cli", "compress", "src", "dst"]);
    assert_token(&out, Some("compress"), "Io");
    assert_eq!(denied.creates(), 0);
    assert_eq!(denied.opens(), 0);

    assert_eq!(parent_of(Path::new("sym/../out")), Path::new("sym/.."));
    assert_eq!(normalize(Path::new("../../file")), Path::new("../../file"));
}

#[test]
fn recorded_prefixed_bytes_do_not_install_a_destination() {
    for seed in 0..32u64 {
        let mut bytes = vec![0x8D, 0x11];
        bytes.extend(lcg(seed.wrapping_mul(17).wrapping_add(3), 8));
        let mut fs = MemFs::new();
        fs.insert_file(Path::new("src"), bytes.clone(), bytes.len() as u64);
        let out = run(&mut fs, &["sxp-cli", "decompress", "src", "dst"]);
        assert_eq!(out.code, 1, "seed {seed} {}", out.stderr);
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
        assert!(!fs.contains(Path::new("dst")), "seed {seed}");
    }
}

#[test]
fn std_fs_round_trip_and_hard_link_refusal() {
    let root = std::env::temp_dir().join(format!("sxp-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let src = root.join("src");
    let dst = root.join("dst");
    let back = root.join("back");
    let link = root.join("link");
    std::fs::write(&src, b"round").unwrap();
    let mut fs = StdFs;
    let compressed = run_with(os_paths(&["sxp-cli", "compress"], &[&src, &dst]), &mut fs);
    assert_eq!(compressed.code, 0, "{}", compressed.stderr);
    let restored = run_with(
        os_paths(&["sxp-cli", "decompress"], &[&dst, &back]),
        &mut fs,
    );
    assert_eq!(restored.code, 0, "{}", restored.stderr);
    assert_eq!(std::fs::read(&back).unwrap(), b"round");

    let directory = run_with(os_paths(&["sxp-cli", "compress"], &[&root, &dst]), &mut fs);
    assert_token(&directory, Some("compress"), "IsADirectory");

    std::fs::hard_link(&src, &link).unwrap();
    let refused = run_with(os_paths(&["sxp-cli", "compress"], &[&src, &link]), &mut fs);
    assert_token(&refused, Some("compress"), "SameFile");
    assert_eq!(std::fs::read(&src).unwrap(), b"round");
    let _ = std::fs::remove_dir_all(&root);
}

fn os_paths(head: &[&str], paths: &[&PathBuf]) -> Vec<OsString> {
    let mut args = argv(head);
    for path in paths {
        args.push(path.as_os_str().to_os_string());
    }
    args
}
