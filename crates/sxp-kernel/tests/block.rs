// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 64, rng_seed = RngSeed::Fixed(0x5111_0008),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every Block.

use std::io::{self, Cursor};

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sxp_kernel::{
    BLOCK_WINDOW, BlockKind, BlockPlan, Error, LENGTH_LIMIT, MAX_ENCODED_BLOCK, ModelConfig,
    NominalLength, Order0Model, actual_len, decode_block, decode_header, encode_block,
    is_canonical, read_blocked_file, read_whole_file_raw, write_blocked_file, write_whole_file_raw,
};

const HEADERS: &str = include_str!("../../../spec/vectors/block-baseline-headers.hex");
const RAW_N1: &[u8] = include_bytes!("../../../spec/vectors/block-raw-n1.bin");
const RAW_N2: &[u8] = include_bytes!("../../../spec/vectors/block-raw-n2.bin");

fn config() -> Config {
    Config {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0008),
        ..Config::default()
    }
}

fn parse_hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
        .collect()
}

fn header_rows() -> Vec<(&'static str, Vec<u8>)> {
    HEADERS
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, hex) = line.split_once(' ').expect("row");
            (name, parse_hex(hex))
        })
        .collect()
}

fn row(name: &str) -> Vec<u8> {
    header_rows()
        .into_iter()
        .find(|(got, _)| *got == name)
        .map(|(_, bytes)| bytes)
        .expect(name)
}

fn pattern(len: usize, seed: u8) -> Vec<u8> {
    (0..len).map(|i| seed.wrapping_add(i as u8)).collect()
}

fn kind_of(code: u8, constant: u8) -> BlockKind {
    match code {
        0 => BlockKind::Raw,
        1 => BlockKind::Direct,
        2 => BlockKind::PreviousByte,
        _ => BlockKind::Constant(constant),
    }
}

fn nominal_of(code: u8) -> NominalLength {
    match code {
        0 => NominalLength::N256,
        1 => NominalLength::N1024,
        _ => NominalLength::N2048,
    }
}

#[test]
fn pinned_headers_and_coded_payloads() {
    let one = [0xA5u8];
    let direct = encode_block(BlockKind::Direct, NominalLength::N256, 1, &one).unwrap();
    assert_eq!(direct, parse_hex("04a4fff800"));

    let prev_src = [0x01u8, 0x00];
    let prev = encode_block(BlockKind::PreviousByte, NominalLength::N256, 2, &prev_src).unwrap();
    assert_eq!(prev, parse_hex("0801ff78000000"));

    let constant = encode_block(BlockKind::Constant(1), NominalLength::N256, 2, &prev_src).unwrap();
    assert_eq!(constant, parse_hex("2c0000ff78040000"));

    let raw = encode_block(BlockKind::Raw, NominalLength::N256, 1, &[0x8D]).unwrap();
    assert_eq!(raw, parse_hex("008d"));

    assert_eq!(row("constant-ff"), parse_hex("ec1f"));
    let ff = encode_block(BlockKind::Constant(0xFF), NominalLength::N256, 1, &[0x11]).unwrap();
    assert_eq!(&ff[..2], row("constant-ff").as_slice());
    assert_eq!(row("constant-0"), parse_hex("0c00"));
    assert_eq!(row("constant-1"), parse_hex("2c00"));
    assert_eq!(row("constant-8"), parse_hex("0c01"));
    assert_eq!(row("padding-raw"), parse_hex("20"));
    assert_eq!(row("padding-constant"), parse_hex("0c20"));
    assert_eq!(row("length-and-kind-reserved"), parse_hex("1f"));
    assert_eq!(row("kind-polynomial-noncanonical-length"), parse_hex("11"));
}

#[test]
fn header_file_matches_each_length_code() {
    let cases = [
        ("raw-256", BlockKind::Raw, NominalLength::N256, 256u64),
        ("raw-1024", BlockKind::Raw, NominalLength::N1024, 1024),
        ("raw-2048", BlockKind::Raw, NominalLength::N2048, 2048),
        ("direct-256", BlockKind::Direct, NominalLength::N256, 256),
        ("direct-1024", BlockKind::Direct, NominalLength::N1024, 1024),
        ("direct-2048", BlockKind::Direct, NominalLength::N2048, 2048),
        (
            "previous-256",
            BlockKind::PreviousByte,
            NominalLength::N256,
            256,
        ),
        (
            "previous-1024",
            BlockKind::PreviousByte,
            NominalLength::N1024,
            1024,
        ),
        (
            "previous-2048",
            BlockKind::PreviousByte,
            NominalLength::N2048,
            2048,
        ),
    ];
    for (name, kind, nominal, remaining) in cases {
        let bytes = vec![0u8; actual_len(nominal, remaining) as usize];
        let encoded = encode_block(kind, nominal, remaining, &bytes).unwrap();
        let expect = row(name);
        assert_eq!(&encoded[..expect.len()], expect.as_slice(), "{name}");
    }
}

#[test]
fn canonical_matrix_round_trips() {
    let rows = [
        (1u64, [Some(1u64), None, None]),
        (255, [Some(255), None, None]),
        (256, [Some(256), None, None]),
        (257, [Some(256), Some(257), None]),
        (1023, [Some(256), Some(1023), None]),
        (1024, [Some(256), Some(1024), None]),
        (1025, [Some(256), Some(1024), Some(1025)]),
        (2047, [Some(256), Some(1024), Some(2047)]),
        (2048, [Some(256), Some(1024), Some(2048)]),
        (2049, [Some(256), Some(1024), Some(2048)]),
    ];
    for (remaining, cells) in rows {
        for (code, cell) in cells.iter().enumerate() {
            let nominal = nominal_of(code as u8);
            match cell {
                None => {
                    assert!(!is_canonical(nominal, remaining), "{remaining} {code}");
                    let err = encode_block(BlockKind::Raw, nominal, remaining, &[0]).unwrap_err();
                    assert_eq!(err, Error::NonCanonicalLength);
                }
                Some(actual) => {
                    assert!(is_canonical(nominal, remaining));
                    assert_eq!(actual_len(nominal, remaining), *actual);
                    for kind_code in 0..4u8 {
                        let kind = kind_of(kind_code, 0x5A);
                        let bytes = pattern(*actual as usize, kind_code.wrapping_add(3));
                        let encoded = encode_block(kind, nominal, remaining, &bytes).unwrap();
                        assert!(encoded.len() <= MAX_ENCODED_BLOCK);
                        let view = decode_block(&encoded, remaining).unwrap();
                        assert_eq!(view.bytes, bytes, "kind {kind_code} rem {remaining}");
                        assert_eq!(view.consumed, encoded.len());
                        assert_eq!(view.kind, kind);
                        assert_eq!(view.nominal, nominal);
                        let mut extra = encoded.clone();
                        extra.push(0x5A);
                        let again = decode_block(&extra, remaining).unwrap();
                        assert_eq!(again.consumed, encoded.len());
                        assert_eq!(again.bytes, bytes);
                    }
                }
            }
        }
    }
}

#[test]
fn rejections_follow_the_check_order() {
    assert_eq!(decode_block(&[], 1).unwrap_err(), Error::TruncatedBlock);
    assert_eq!(
        decode_block(&row("reserved-length"), 256).unwrap_err(),
        Error::ReservedLength
    );
    assert_eq!(
        decode_block(&row("length-and-kind-reserved"), 1).unwrap_err(),
        Error::ReservedLength
    );
    assert_eq!(
        decode_block(&row("reserved-kind"), 256).unwrap_err(),
        Error::ReservedKind
    );
    assert_eq!(
        decode_block(&row("kind-polynomial"), 1).unwrap_err(),
        Error::UnsupportedKind
    );
    assert_eq!(
        decode_block(&row("kind-polynomial-noncanonical-length"), 1).unwrap_err(),
        Error::UnsupportedKind
    );
    assert_eq!(
        decode_block(&[0x14], 1).unwrap_err(),
        Error::UnsupportedKind
    );
    assert_eq!(
        decode_block(&[0x18], 1).unwrap_err(),
        Error::UnsupportedKind
    );
    assert_eq!(decode_block(&[0x40], 1).unwrap_err(), Error::NonZeroPadding);
    assert_eq!(decode_block(&[0x80], 1).unwrap_err(), Error::NonZeroPadding);
    assert_eq!(
        decode_block(&[0x0C, 0x40], 256).unwrap_err(),
        Error::NonZeroPadding
    );
    assert_eq!(
        decode_block(&[0x0C, 0x80], 256).unwrap_err(),
        Error::NonZeroPadding
    );
    assert_eq!(decode_block(&[0x0C], 1).unwrap_err(), Error::TruncatedBlock);
    assert_eq!(
        decode_block(&row("padding-raw"), 1).unwrap_err(),
        Error::NonZeroPadding
    );
    assert_eq!(
        decode_block(&row("padding-constant"), 256).unwrap_err(),
        Error::NonZeroPadding
    );
    assert_eq!(
        decode_block(&[0x01], 1).unwrap_err(),
        Error::NonCanonicalLength
    );
    assert_eq!(decode_block(&[0x00], 1).unwrap_err(), Error::TruncatedBlock);
    let direct = encode_block(BlockKind::Direct, NominalLength::N256, 1, &[0xA5]).unwrap();
    assert_eq!(
        decode_block(&direct[..direct.len() - 1], 1).unwrap_err(),
        Error::TruncatedCoder
    );
    let mut saturated = direct.clone();
    for slot in saturated.iter_mut().skip(1).take(4) {
        *slot = 0xFF;
    }
    assert_eq!(
        decode_block(&saturated, 1).unwrap_err(),
        Error::SaturatedRangeCode
    );
    let mut flipped = direct.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 0x01;
    assert_eq!(
        decode_block(&flipped, 1).unwrap_err(),
        Error::NonzeroRangeCode
    );
}

#[test]
fn writer_plan_errors_leave_the_sink_empty() {
    let mut sink = Vec::new();
    let err = write_blocked_file(0, &[], &mut sink).unwrap_err();
    assert_eq!(err, Error::BlockedEmpty);
    assert!(sink.is_empty());

    let mut sink = Vec::new();
    let err = write_blocked_file(LENGTH_LIMIT, &[], &mut sink).unwrap_err();
    assert_eq!(err, Error::LengthOutOfRange);
    assert!(sink.is_empty());

    let mut sink = Vec::new();
    let plan = BlockPlan {
        kind: BlockKind::Raw,
        nominal: NominalLength::N1024,
        bytes: &[0x11, 0x22],
    };
    let err = write_blocked_file(2, &[plan], &mut sink).unwrap_err();
    assert_eq!(err, Error::NonCanonicalLength);
    assert!(sink.is_empty());

    let mut sink = Vec::new();
    let plan = BlockPlan {
        kind: BlockKind::Raw,
        nominal: NominalLength::N256,
        bytes: &[0x11],
    };
    let err = write_blocked_file(2, &[plan], &mut sink).unwrap_err();
    assert_eq!(err, Error::BlockLengthMismatch);
    assert!(sink.is_empty());
}

#[test]
fn file_fixtures_and_mode_split() {
    let mut out = Vec::new();
    let plan = BlockPlan {
        kind: BlockKind::Raw,
        nominal: NominalLength::N256,
        bytes: &[0x8D],
    };
    write_blocked_file(1, &[plan], &mut out).unwrap();
    assert_eq!(out, RAW_N1);
    let mut decoded = Vec::new();
    read_blocked_file(&mut Cursor::new(RAW_N1), &mut decoded).unwrap();
    assert_eq!(decoded, [0x8D]);

    let mut out = Vec::new();
    let plan = BlockPlan {
        kind: BlockKind::Raw,
        nominal: NominalLength::N256,
        bytes: &[0x11, 0x22],
    };
    write_blocked_file(2, &[plan], &mut out).unwrap();
    assert_eq!(out, RAW_N2);
    let mut decoded = Vec::new();
    read_blocked_file(&mut Cursor::new(RAW_N2), &mut decoded).unwrap();
    assert_eq!(decoded, [0x11, 0x22]);

    let mut sink = Vec::new();
    let err = read_whole_file_raw(&mut Cursor::new(RAW_N1), &mut sink).unwrap_err();
    assert_eq!(err, Error::BlockedModeUnsupported);
    assert!(sink.is_empty());

    let mut raw = Vec::new();
    write_whole_file_raw(0, &mut io::empty(), &mut raw).unwrap();
    let mut sink = Vec::new();
    let err = read_blocked_file(&mut Cursor::new(raw), &mut sink).unwrap_err();
    assert_eq!(err, Error::UnexpectedMode);
    assert!(sink.is_empty());

    let mut empty = Vec::new();
    let err = read_blocked_file(&mut Cursor::new([0x8D, 0x01, 0x00]), &mut empty).unwrap_err();
    assert_eq!(err, Error::BlockedEmpty);
    assert!(empty.is_empty());
}

#[test]
fn file_crc_trailer_and_trailing_bytes() {
    let mut short = RAW_N1.to_vec();
    short.pop();
    let mut sink = Vec::new();
    let err = read_blocked_file(&mut Cursor::new(short), &mut sink).unwrap_err();
    assert_eq!(err, Error::TruncatedTrailer);

    let mut bad = RAW_N1.to_vec();
    let last = bad.len() - 1;
    bad[last] ^= 0x01;
    let mut sink = Vec::new();
    let err = read_blocked_file(&mut Cursor::new(bad), &mut sink).unwrap_err();
    assert_eq!(err, Error::CrcMismatch);

    let mut extra = RAW_N1.to_vec();
    extra.push(0x00);
    let mut sink = Vec::new();
    let err = read_blocked_file(&mut Cursor::new(extra), &mut sink).unwrap_err();
    assert_eq!(err, Error::TrailingBytes);
    assert_eq!(sink, [0x8D]);
}

#[test]
fn multi_block_streams_keep_the_next_header() {
    let first = vec![0x11u8; 256];
    let second = vec![0x22u8; 256];
    let plans = [
        BlockPlan {
            kind: BlockKind::Raw,
            nominal: NominalLength::N256,
            bytes: &first,
        },
        BlockPlan {
            kind: BlockKind::Direct,
            nominal: NominalLength::N256,
            bytes: &second,
        },
    ];
    let mut file = Vec::new();
    write_blocked_file(512, &plans, &mut file).unwrap();
    let mut decoded = Vec::new();
    read_blocked_file(&mut Cursor::new(&file), &mut decoded).unwrap();
    let mut expect = first.clone();
    expect.extend_from_slice(&second);
    assert_eq!(decoded, expect);

    let encoded_first = encode_block(BlockKind::Raw, NominalLength::N256, 512, &first).unwrap();
    let (_, header_len) = decode_header(&file).unwrap();
    let view = decode_block(&file[header_len..], 512).unwrap();
    assert_eq!(view.consumed, encoded_first.len());
    assert_eq!(view.bytes, first);
    assert_eq!(file[header_len + view.consumed], 0x04);

    let mut head = vec![0x11u8; 256];
    let tail = [0x22u8];
    let plans = [
        BlockPlan {
            kind: BlockKind::Raw,
            nominal: NominalLength::N256,
            bytes: &head,
        },
        BlockPlan {
            kind: BlockKind::PreviousByte,
            nominal: NominalLength::N256,
            bytes: &tail,
        },
    ];
    let mut file = Vec::new();
    write_blocked_file(257, &plans, &mut file).unwrap();
    let mut decoded = Vec::new();
    read_blocked_file(&mut Cursor::new(&file), &mut decoded).unwrap();
    head.push(0x22);
    assert_eq!(decoded, head);

    let wide = encode_leb128_header_round_trip();
    assert!(wide.len() > 3);
}

fn encode_leb128_header_round_trip() -> Vec<u8> {
    let bytes = vec![0x7Fu8; 128];
    let plan = BlockPlan {
        kind: BlockKind::Constant(0x7F),
        nominal: NominalLength::N256,
        bytes: &bytes,
    };
    let mut file = Vec::new();
    write_blocked_file(128, &[plan], &mut file).unwrap();
    let mut decoded = Vec::new();
    read_blocked_file(&mut Cursor::new(&file), &mut decoded).unwrap();
    assert_eq!(decoded, bytes);
    assert_eq!(file[2], 0x80);
    file
}

#[test]
fn an_earlier_block_does_not_change_the_next_payload() {
    let tail = [0x01u8, 0x00];
    let first_a = vec![0x11u8; 256];
    let mut first_b = vec![0x22u8; 256];
    let encoded_tail =
        encode_block(BlockKind::PreviousByte, NominalLength::N256, 2, &tail).unwrap();
    let plans_a = [
        BlockPlan {
            kind: BlockKind::Direct,
            nominal: NominalLength::N256,
            bytes: &first_a,
        },
        BlockPlan {
            kind: BlockKind::PreviousByte,
            nominal: NominalLength::N256,
            bytes: &tail,
        },
    ];
    first_b[0] = 0x99;
    let plans_b = [
        BlockPlan {
            kind: BlockKind::Direct,
            nominal: NominalLength::N256,
            bytes: &first_b,
        },
        BlockPlan {
            kind: BlockKind::PreviousByte,
            nominal: NominalLength::N256,
            bytes: &tail,
        },
    ];
    let mut file_a = Vec::new();
    let mut file_b = Vec::new();
    write_blocked_file(258, &plans_a, &mut file_a).unwrap();
    write_blocked_file(258, &plans_b, &mut file_b).unwrap();
    assert_ne!(file_a, file_b);
    assert_eq!(
        &file_a[file_a.len() - 4 - encoded_tail.len()..file_a.len() - 4],
        encoded_tail.as_slice()
    );
    assert_eq!(
        &file_b[file_b.len() - 4 - encoded_tail.len()..file_b.len() - 4],
        encoded_tail.as_slice()
    );
    let (_, header_len) = decode_header(&file_a).unwrap();
    let view_a = decode_block(&file_a[header_len..], 258).unwrap();
    let view_b = decode_block(&file_b[header_len..], 258).unwrap();
    assert_ne!(view_a.bytes, view_b.bytes);
    let next_a = decode_block(&file_a[header_len + view_a.consumed..], 2).unwrap();
    let next_b = decode_block(&file_b[header_len + view_b.consumed..], 2).unwrap();
    assert_eq!(next_a.bytes, tail);
    assert_eq!(next_b.bytes, tail);
    assert_eq!(next_a.consumed, next_b.consumed);
    assert_eq!(next_a.consumed, encoded_tail.len());
}

struct OneByte<R> {
    inner: R,
}

impl<R: io::Read> io::Read for OneByte<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut byte = [0u8; 1];
        let n = self.inner.read(&mut byte)?;
        if n == 1 {
            buf[0] = byte[0];
        }
        Ok(n)
    }
}

#[test]
fn coded_file_round_trips_one_byte_at_a_time() {
    let first = pattern(256, 0xA5);
    let second = [0x01u8, 0x00];
    let plans = [
        BlockPlan {
            kind: BlockKind::Direct,
            nominal: NominalLength::N256,
            bytes: &first,
        },
        BlockPlan {
            kind: BlockKind::Constant(0x3C),
            nominal: NominalLength::N256,
            bytes: &second,
        },
    ];
    let mut file = Vec::new();
    write_blocked_file(258, &plans, &mut file).unwrap();
    let mut decoded = Vec::new();
    read_blocked_file(
        &mut OneByte {
            inner: Cursor::new(&file),
        },
        &mut decoded,
    )
    .unwrap();
    let mut expect = first;
    expect.extend_from_slice(&second);
    assert_eq!(decoded, expect);
}

#[test]
fn adversarial_coded_block_fits_the_window() {
    let bytes = less_likely(2048);
    let encoded = encode_block(BlockKind::Direct, NominalLength::N2048, 2048, &bytes).unwrap();
    assert!(encoded.len() <= MAX_ENCODED_BLOCK, "{}", encoded.len());
    assert!(encoded.len() + 4 <= BLOCK_WINDOW);
    let view = decode_block(&encoded, 2048).unwrap();
    assert_eq!(view.bytes, bytes);
}

fn less_likely(n: usize) -> Vec<u8> {
    let mut model = Order0Model::new(ModelConfig::strawman().unwrap());
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let mut symbol = 0u8;
        let mut node = 1u16;
        for depth in 0..8 {
            let (freq0, freq1) = model.freq(node).unwrap();
            let bit = u8::from(freq0 > freq1);
            model.update(node, bit).unwrap();
            symbol = (symbol << 1) | bit;
            if depth + 1 < 8 {
                node = node.saturating_mul(2).saturating_add(u16::from(bit));
            }
        }
        out.push(symbol);
    }
    out
}

proptest! {
    #![proptest_config(config())]
    #[test]
    fn random_canonical_blocks_round_trip(
        kind_code in 0u8..4,
        nominal_code in 0u8..3,
        remaining in 1u64..=3000u64,
        seed in any::<u8>(),
        constant in any::<u8>(),
    ) {
        let nominal = nominal_of(nominal_code);
        let kind = kind_of(kind_code, constant);
        if !is_canonical(nominal, remaining) {
            let err = encode_block(kind, nominal, remaining, &[]).unwrap_err();
            prop_assert_eq!(err, Error::NonCanonicalLength);
            return Ok(());
        }
        let actual = actual_len(nominal, remaining) as usize;
        let bytes = pattern(actual, seed);
        let encoded = encode_block(kind, nominal, remaining, &bytes).unwrap();
        let view = decode_block(&encoded, remaining).unwrap();
        prop_assert_eq!(&view.bytes, &bytes);
        prop_assert_eq!(view.consumed, encoded.len());
    }
}
