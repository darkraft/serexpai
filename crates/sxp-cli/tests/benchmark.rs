//! `benchmark` writes an exploratory report and leaves a failed run's report in place.

use std::path::Path;

use sxp_cli::{MemFs, run_with};

fn run(fs: &mut MemFs, args: &[&str]) -> sxp_cli::Output {
    run_with(args.iter().copied(), fs)
}

#[test]
fn usage_and_missing_directory_fail_before_a_report() {
    let mut fs = MemFs::new();
    let usage = run(&mut fs, &["sxp-cli", "benchmark"]);
    assert_eq!(usage.code, 1);
    assert!(usage.stderr.contains("sxp-cli: benchmark: Usage\n"));
    let missing = run(
        &mut fs,
        &["sxp-cli", "benchmark", "out", "manifest.toml", "gate.json"],
    );
    assert_eq!(missing.code, 1);
    assert!(missing.stderr.contains("NotFound"));
    assert!(!fs.contains(Path::new("out/report.json")));
}

#[test]
fn a_failed_run_leaves_the_previous_report() {
    let mut fs = MemFs::new();
    fs.insert_dir(Path::new("out"));
    fs.insert_file(Path::new("out/report.json"), b"previous".to_vec(), 8);
    fs.insert_file(
        Path::new("manifest.toml"),
        manifest_with_bad_hash().into(),
        1,
    );
    fs.insert_file(Path::new("gate.json"), gate().into(), 1);
    let output = run(
        &mut fs,
        &["sxp-cli", "benchmark", "out", "manifest.toml", "gate.json"],
    );
    assert_eq!(output.code, 1);
    assert!(output.stderr.contains("sxp-cli: benchmark: HashMismatch\n"));
    assert_eq!(
        fs.bytes(Path::new("out/report.json")).as_deref(),
        Some(b"previous".as_slice())
    );
}

#[test]
fn a_directory_is_not_a_manifest() {
    let mut fs = MemFs::new();
    fs.insert_dir(Path::new("out"));
    fs.insert_dir(Path::new("manifest.toml"));
    fs.insert_file(Path::new("gate.json"), gate().into(), 1);
    let output = run(
        &mut fs,
        &["sxp-cli", "benchmark", "out", "manifest.toml", "gate.json"],
    );
    assert_eq!(output.code, 1);
    assert!(output.stderr.contains("IsADirectory"));
    assert!(!fs.contains(Path::new("out/report.json")));
}

#[test]
fn benchmark_writes_matching_stdout_and_report() {
    let mut fs = MemFs::new();
    fs.insert_dir(Path::new("out"));
    fs.insert_file(Path::new("manifest.toml"), manifest().into(), 1);
    fs.insert_file(Path::new("gate.json"), gate().into(), 1);
    let output = run(
        &mut fs,
        &["sxp-cli", "benchmark", "out", "manifest.toml", "gate.json"],
    );
    assert_eq!(output.code, 0, "{}", output.stderr);
    assert!(output.stderr.is_empty());
    let report = fs.bytes(Path::new("out/report.json")).expect("report");
    assert_eq!(output.stdout.as_bytes(), report.as_slice());
    let text = String::from_utf8(report).expect("utf8");
    assert!(text.contains("\"status\":\"exploratory/unreportable\""));
    assert!(text.contains("\"verdict\":null"));
    assert!(fs.contains(Path::new("out/annex.json")));
    assert!(fs.contains(Path::new("out/synthetic-polynomial.sxpc")));
    assert!(fs.contains(Path::new("out/synthetic-trigonometric.sxpc")));
    assert!(fs.contains(Path::new("out/synthetic-recurrence.sxpc")));
    let timing =
        String::from_utf8(fs.bytes(Path::new("out/timing.log")).expect("timing")).expect("utf8");
    assert!(timing.starts_with("host="));
    assert!(timing.contains("\nscope=benchmark\n"));
    assert!(timing.contains("\ncompleted=true\n"));
    assert!(timing.contains("\nbudget_seconds=86400\n"));
}

fn gate() -> &'static str {
    r#"{"corpus_gate_ready":false,"scope":"corpus","unresolved":[{"field":"k.status","reason":"proposed"}]}"#
}

fn manifest() -> String {
    synthetic_manifest("7364c598b868aae6247ed6554b1fc9e2e251220f237c913f0ac2720a5b2b5e92")
}

fn manifest_with_bad_hash() -> String {
    synthetic_manifest("0000000000000000000000000000000000000000000000000000000000000000")
}

fn synthetic_manifest(polynomial_hash: &str) -> String {
    format!(
        "\
[[input]]
id = \"synthetic-polynomial\"
class = \"synthetic\"
role = \"synthetic\"
counts_toward_sm1 = false
transform = \"kernel-predict\"
derived_sha256 = \"{polynomial_hash}\"

[[input]]
id = \"synthetic-trigonometric\"
class = \"synthetic\"
role = \"synthetic\"
counts_toward_sm1 = false
transform = \"kernel-predict\"
derived_sha256 = \"1a2fa720c8ec350f1519e98632eae12add852e2d73c4ce931fd1d91129e93fcb\"

[[input]]
id = \"synthetic-recurrence\"
class = \"synthetic\"
role = \"synthetic\"
counts_toward_sm1 = false
transform = \"kernel-predict\"
derived_sha256 = \"90b1408a5ea7b47f68e2ed160763d5f1d196a4533e74ad67a357005466a1c291\"
"
    )
}

#[test]
fn a_file_is_not_an_output_directory() {
    let mut fs = MemFs::new();
    fs.insert_file(Path::new("out"), b"x".to_vec(), 1);
    fs.insert_file(Path::new("manifest.toml"), b"[]".to_vec(), 2);
    fs.insert_file(Path::new("gate.json"), b"{}".to_vec(), 2);
    let output = run(
        &mut fs,
        &["sxp-cli", "benchmark", "out", "manifest.toml", "gate.json"],
    );
    assert_eq!(output.code, 1);
    assert!(output.stderr.contains("NotADirectory"));
    assert_eq!(fs.bytes(Path::new("out")).as_deref(), Some(b"x".as_slice()));
}
