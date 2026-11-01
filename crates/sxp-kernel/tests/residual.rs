// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0002),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every byte string.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sha2::{Digest, Sha256};
use sxp_kernel::{direct_symbol, reconstruct, residual_symbol, signed_residual};

const RESIDUAL_SHA256: &str = include_str!("../../../spec/vectors/residual-symbols.sha256");

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0002),
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

fn assert_pair(y: u8, prediction: u8) {
    let symbol = residual_symbol(y, prediction);
    assert_eq!(symbol, y.wrapping_sub(prediction));
    let delta = signed_residual(y, prediction);
    assert!((-128..=127).contains(&delta));
    assert_eq!(delta, i16::from(symbol.cast_signed()));
    assert_eq!(reconstruct(prediction, symbol), y);
}

#[test]
fn exhaustive_pairs_match_wrapping_oracle_and_hash() {
    let mut symbols = Vec::with_capacity(65_536);
    for y in 0..=255u8 {
        for prediction in 0..=255u8 {
            assert_pair(y, prediction);
            symbols.push(residual_symbol(y, prediction));
        }
    }
    assert_eq!(symbols.len(), 65_536);
    assert_eq!(sha256_hex(&symbols), RESIDUAL_SHA256.trim());
}

#[test]
fn named_residual_fixtures() {
    let rows = [
        (0u8, 128u8, -128i16, 128u8),
        (128, 0, -128, 128),
        (0, 255, 1, 1),
        (255, 0, -1, 255),
        (127, 0, 127, 127),
        (0, 129, 127, 127),
        (5, 5, 0, 0),
    ];
    for (y, prediction, delta, symbol) in rows {
        assert_eq!(signed_residual(y, prediction), delta);
        assert_eq!(residual_symbol(y, prediction), symbol);
        assert_eq!(reconstruct(prediction, symbol), y);
    }
    assert_eq!(reconstruct(0, 128), 128);
}

#[test]
fn direct_symbol_is_the_byte() {
    for y in 0..=255u8 {
        assert_eq!(direct_symbol(y), y);
    }
    assert_ne!(residual_symbol(0, 1), direct_symbol(0));
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn random_pairs_reconstruct(y in any::<u8>(), prediction in any::<u8>()) {
        let symbol = residual_symbol(y, prediction);
        prop_assert_eq!(symbol, y.wrapping_sub(prediction));
        let delta = signed_residual(y, prediction);
        prop_assert!((-128..=127).contains(&delta));
        prop_assert_eq!(delta, i16::from(symbol.cast_signed()));
        prop_assert_eq!(reconstruct(prediction, symbol), y);
    }

    #[test]
    fn random_blocks_reconstruct(
        len in prop_oneof![Just(1usize), Just(256usize), Just(1024usize), Just(2048usize)],
        payload in proptest::collection::vec(any::<u8>(), 2048),
        predictions in proptest::collection::vec(any::<u8>(), 2048),
    ) {
        for i in 0..len {
            let y = payload[i];
            let prediction = predictions[i];
            prop_assert_eq!(reconstruct(prediction, residual_symbol(y, prediction)), y);
        }
    }

    #[test]
    fn causal_table_predictor_reconstructs(
        payload in proptest::collection::vec(any::<u8>(), 1..=64usize),
        table in proptest::collection::vec(any::<u8>(), 256..=256usize),
    ) {
        let mut decoded = Vec::with_capacity(payload.len());
        let mut previous: Option<u8> = None;
        for &y in &payload {
            let prediction = match previous {
                None => 0,
                Some(prior) => table[usize::from(prior)],
            };
            let symbol = residual_symbol(y, prediction);
            let got = reconstruct(prediction, symbol);
            decoded.push(got);
            previous = Some(got);
        }
        prop_assert_eq!(decoded, payload);
    }
}
