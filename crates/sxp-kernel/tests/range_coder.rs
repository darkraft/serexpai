// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0006),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every decision schedule.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sha2::{Digest, Sha256};
use sxp_kernel::{Error, RangeDecoder, RangeEncoder, decode_schedule, encode_schedule, exact_size};

const RANGE_CODER_SHA256: &str = include_str!("../../../spec/vectors/range-coder-2048.sha256");

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0006),
        ..Config::default()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        write!(hex, "{byte:02x}").unwrap();
    }
    hex
}

struct Case {
    decisions: &'static [(u8, u16)],
    bytes: &'static [u8],
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            decisions: &[],
            bytes: &[0x00, 0x00, 0x00, 0x00],
        },
        Case {
            decisions: &[(0, 2048)],
            bytes: &[0x00, 0x00, 0x00, 0x00],
        },
        Case {
            decisions: &[(1, 2048)],
            bytes: &[0x7F, 0xFF, 0xF8, 0x00],
        },
        Case {
            decisions: &[(0, 1)],
            bytes: &[0x00, 0x00, 0x00, 0x00, 0x00],
        },
        Case {
            decisions: &[(1, 1)],
            bytes: &[0x00, 0x0F, 0xFF, 0xFF],
        },
        Case {
            decisions: &[(0, 4095)],
            bytes: &[0x00, 0x00, 0x00, 0x00],
        },
        Case {
            decisions: &[(1, 4095)],
            bytes: &[0xFF, 0xEF, 0xF0, 0x01, 0x00],
        },
        Case {
            decisions: &[(0, 1), (0, 1)],
            bytes: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        },
        Case {
            decisions: &[(0, 1), (1, 2048), (0, 1), (1, 2047)],
            bytes: &[0x00, 0x08, 0x00, 0x37, 0xF8, 0x00, 0x00],
        },
    ]
}

fn freqs(decisions: &[(u8, u16)]) -> Vec<u16> {
    decisions.iter().map(|(_, freq0)| *freq0).collect()
}

fn bits(decisions: &[(u8, u16)]) -> Vec<u8> {
    decisions.iter().map(|(bit, _)| *bit).collect()
}

#[test]
fn pinned_streams_match_bytes_and_consumption() {
    for case in cases() {
        let encoded = encode_schedule(case.decisions).unwrap();
        assert_eq!(encoded, case.bytes);
        assert_eq!(exact_size(case.decisions).unwrap(), case.bytes.len());
        assert_eq!(
            decode_schedule(case.bytes, &freqs(case.decisions)).unwrap(),
            bits(case.decisions)
        );
        let mut decoder = RangeDecoder::new(case.bytes).unwrap();
        for &freq0 in &freqs(case.decisions) {
            decoder.decode(freq0).unwrap();
        }
        decoder.finish().unwrap();
        assert_eq!(decoder.bytes_consumed(), case.bytes.len());
    }
}

#[test]
fn bit0_freq1_refills_inside_decode() {
    let bytes = [0x00, 0x00, 0x00, 0x00, 0x00];
    let mut decoder = RangeDecoder::new(&bytes).unwrap();
    assert_eq!(decoder.decode(1).unwrap(), 0);
    let state = decoder.state();
    assert_eq!(state.range, 0x0FFF_FF00);
    assert_eq!(state.code, 0);
    assert_eq!(state.bytes_consumed, 5);
    decoder.finish().unwrap();
}

#[test]
fn bit1_freq4095_keeps_the_pending_byte() {
    let encoded = encode_schedule(&[(1, 4095)]).unwrap();
    assert_eq!(encoded, [0xFF, 0xEF, 0xF0, 0x01, 0x00]);
}

#[test]
fn carry_schedule_bytes_and_bits() {
    let decisions = [(0, 1), (1, 2048), (0, 1), (1, 2047)];
    let encoded = encode_schedule(&decisions).unwrap();
    assert_eq!(encoded, [0x00, 0x08, 0x00, 0x37, 0xF8, 0x00, 0x00]);
    assert_eq!(
        decode_schedule(&encoded, &freqs(&decisions)).unwrap(),
        [0, 1, 0, 1]
    );
}

#[test]
fn trace_ranges_agree_and_a_new_encoder_starts_over() {
    let decisions = [(1, 1), (0, 4095), (1, 2048)];
    let mut encoder = RangeEncoder::new();
    let expected_range = [0xFFF0_0000, 0xFFE0_0100, 0x7FF0_0100];
    for (index, &(bit, freq0)) in decisions.iter().enumerate() {
        encoder.encode(bit, freq0).unwrap();
        assert_eq!(encoder.state().range, expected_range[index]);
    }
    let bytes = encoder.finish();
    assert_eq!(bytes, [0x7F, 0xFF, 0xFF, 0xFF]);

    let mut decoder = RangeDecoder::new(&bytes).unwrap();
    let expected = [
        (0x7FF0_0000, 0xFFF0_0000, 4),
        (0x7FF0_0000, 0xFFE0_0100, 4),
        (0, 0x7FF0_0100, 4),
    ];
    for (index, &(_, freq0)) in decisions.iter().enumerate() {
        decoder.decode(freq0).unwrap();
        let state = decoder.state();
        assert_eq!(
            (state.code, state.range, state.bytes_consumed),
            expected[index]
        );
    }
    decoder.finish().unwrap();

    let fresh = encode_schedule(&[(1, 2048)]).unwrap();
    assert_eq!(fresh, [0x7F, 0xFF, 0xF8, 0x00]);
}

#[test]
fn rejected_encode_does_not_change_state() {
    let mut encoder = RangeEncoder::new();
    let before = encoder.state();
    for &(bit, freq0) in &[(2, 2048), (255, 1), (0, 0), (1, 4096), (0, u16::MAX)] {
        assert_eq!(encoder.encode(bit, freq0), Err(Error::InvalidParameter));
        assert_eq!(encoder.state(), before);
    }
    assert_eq!(encoder.finish(), [0x00, 0x00, 0x00, 0x00]);

    let mut encoder = RangeEncoder::new();
    assert_eq!(encoder.encode(2, 1), Err(Error::InvalidParameter));
    encoder.encode(1, 2048).unwrap();
    assert_eq!(encoder.finish(), [0x7F, 0xFF, 0xF8, 0x00]);
}

#[test]
fn truncated_refill_stays_truncated() {
    let bytes = [0x00, 0x00, 0x00, 0x00];
    let mut decoder = RangeDecoder::new(&bytes).unwrap();
    assert_eq!(decoder.decode(1), Err(Error::TruncatedCoder));
    assert_eq!(decoder.decode(2048), Err(Error::TruncatedCoder));
    assert_eq!(decoder.finish(), Err(Error::TruncatedCoder));
    assert!(decoder.state().range < 0x0100_0000);
}

#[test]
fn rejected_decode_does_not_change_state() {
    let bytes = [0x00, 0x00, 0x00, 0x00];
    let mut decoder = RangeDecoder::new(&bytes).unwrap();
    let before = decoder.state();
    assert_eq!(decoder.decode(0), Err(Error::InvalidParameter));
    assert_eq!(decoder.decode(4096), Err(Error::InvalidParameter));
    assert_eq!(decoder.state(), before);
    assert_eq!(decoder.decode(2048).unwrap(), 0);
    decoder.finish().unwrap();
}

#[test]
fn init_and_finish_errors_are_distinct() {
    assert!(matches!(
        RangeDecoder::new(&[0xFF, 0xFF, 0xFF]),
        Err(Error::TruncatedCoder)
    ));
    assert!(matches!(
        RangeDecoder::new(&[0xFF, 0xFF, 0xFF, 0xFF]),
        Err(Error::SaturatedRangeCode)
    ));
    assert_eq!(
        decode_schedule(&[0xFF, 0xFF, 0xFF, 0xFF], &[]),
        Err(Error::SaturatedRangeCode)
    );
    assert_eq!(
        decode_schedule(&[0x00, 0x00, 0x00, 0x01], &[2048]),
        Err(Error::NonzeroRangeCode)
    );
    assert_eq!(
        decode_schedule(&[0x00, 0x00, 0x00, 0x00, 0x00], &[2048]),
        Err(Error::TrailingBytes)
    );
}

#[test]
fn every_proper_prefix_is_truncated() {
    for case in cases() {
        let freq0s = freqs(case.decisions);
        for length in 0..case.bytes.len() {
            assert_eq!(
                decode_schedule(&case.bytes[..length], &freq0s),
                Err(Error::TruncatedCoder),
                "prefix {length} of {:?}",
                case.bytes
            );
        }
    }
}

#[test]
fn concatenated_frames_stop_at_bytes_consumed() {
    let first = encode_schedule(&[(1, 2048)]).unwrap();
    let second = encode_schedule(&[(0, 1)]).unwrap();
    let mut joined = first.clone();
    joined.extend_from_slice(&second);
    let mut decoder = RangeDecoder::new(&joined).unwrap();
    assert_eq!(decoder.decode(2048).unwrap(), 1);
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 4);
    let rest = &joined[decoder.bytes_consumed()..];
    let mut decoder = RangeDecoder::new(rest).unwrap();
    assert_eq!(decoder.decode(1).unwrap(), 0);
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 5);
    assert_eq!(decode_schedule(&joined, &[2048]), Err(Error::TrailingBytes));

    let mut reversed = second;
    reversed.extend_from_slice(&first);
    let mut decoder = RangeDecoder::new(&reversed).unwrap();
    assert_eq!(decoder.decode(1).unwrap(), 0);
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 5);
    let rest = &reversed[decoder.bytes_consumed()..];
    let mut decoder = RangeDecoder::new(rest).unwrap();
    assert_eq!(decoder.decode(2048).unwrap(), 1);
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 4);
}

fn symbol_schedule() -> Vec<(u8, u16)> {
    let mut decisions = Vec::with_capacity(16_384);
    for i in 0..2048 {
        let symbol = i as u8;
        for k in 0..8 {
            let bit = (symbol >> (7 - k)) & 1;
            let freq0 = 1 + ((decisions.len() as u16) % 4095);
            decisions.push((bit, freq0));
        }
    }
    decisions
}

#[test]
fn symbol_fixture_matches_the_committed_hash() {
    let decisions = symbol_schedule();
    assert_eq!(decisions.len(), 16_384);
    let encoded = encode_schedule(&decisions).unwrap();
    assert_eq!(encoded.len(), 3321);
    assert_eq!(encoded[18], 0x13);
    assert_eq!(encoded[100], 0xCE);
    assert_eq!(encoded[1000], 0x78);
    assert_eq!(
        &encoded[encoded.len() - 8..],
        &[0x5E, 0x3E, 0xFF, 0xFF, 0xCE, 0x03, 0x4A, 0xA1]
    );
    assert_eq!(sha256_hex(&encoded), RANGE_CODER_SHA256.trim());
    let freq0s = freqs(&decisions);
    assert_eq!(
        decode_schedule(&encoded, &freq0s).unwrap(),
        bits(&decisions)
    );
    let mut decoder = RangeDecoder::new(&encoded).unwrap();
    for &freq0 in &freq0s {
        decoder.decode(freq0).unwrap();
    }
    decoder.finish().unwrap();
    assert_eq!(decoder.bytes_consumed(), 3321);
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn random_schedules_round_trip(decisions in prop::collection::vec((0u8..=1, 1u16..=4095), 0..=48)) {
        let encoded = encode_schedule(&decisions).unwrap();
        prop_assert_eq!(exact_size(&decisions).unwrap(), encoded.len());
        let freq0s = freqs(&decisions);
        prop_assert_eq!(decode_schedule(&encoded, &freq0s).unwrap(), bits(&decisions));
        let mut decoder = RangeDecoder::new(&encoded).unwrap();
        for &freq0 in &freq0s {
            decoder.decode(freq0).unwrap();
        }
        decoder.finish().unwrap();
        prop_assert_eq!(decoder.bytes_consumed(), encoded.len());
    }
}
