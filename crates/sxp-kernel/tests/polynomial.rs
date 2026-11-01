// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0003),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every coefficient tuple.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sha2::{Digest, Sha256};
use sxp_kernel::{Error, Polynomial};

const POLYNOMIAL_SHA256: &str =
    include_str!("../../../spec/vectors/polynomial-degree15-q24.sha256");

/// `|acc| < 2^35`. Seed §4.1 says `|acc| <= (degree + 1)·2^31 < 2^35`.
/// That strict bound holds because `|c| <= 2^31 - 1`, so
/// `|acc| <= (degree + 1)·(2^31 - 1) = 2^35 - 16` at degree 15.
const ACC_LIMIT: i128 = 1i128 << 35;
/// `|acc·u| < 2^51`, the other half of the seed §4.1 bound.
const PRODUCT_LIMIT: i128 = 1i128 << 51;

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0003),
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

/// Independent `i128` Horner. `acc` is the pre-round accumulator.
fn oracle(coefficients: &[i32], q: u8, n: u16) -> (i128, i128) {
    let degree = coefficients.len() - 1;
    let u = i128::from(n) << 5;
    let mut acc = i128::from(*coefficients.get(degree).unwrap());
    assert!(acc.abs() < ACC_LIMIT);
    let mut k = degree;
    while k > 0 {
        k -= 1;
        let product = acc * u;
        assert!(product.abs() < PRODUCT_LIMIT);
        acc = (product >> 16) + i128::from(*coefficients.get(k).unwrap());
        assert!(acc.abs() < ACC_LIMIT);
    }
    let value = (acc + (1i128 << (q - 1))) >> q;
    (acc, value)
}

fn assert_case(q: u8, n: u16, coefficients: &[i32], acc: i128, value: i64, yhat: u8) {
    let degree = u8::try_from(coefficients.len() - 1).unwrap();
    let (oracle_acc, oracle_value) = oracle(coefficients, q, n);
    assert_eq!(oracle_acc, acc);
    assert_eq!(oracle_value, i128::from(value));
    let poly = Polynomial::new(degree, q, coefficients).unwrap();
    assert_eq!(poly.value(n).unwrap(), value);
    assert_eq!(poly.predict(n).unwrap(), yhat);
}

fn alternating() -> [i32; 16] {
    let mut coeffs = [0i32; 16];
    for (k, slot) in coeffs.iter_mut().enumerate() {
        *slot = if k % 2 == 0 { i32::MAX } else { i32::MIN + 1 };
    }
    coeffs
}

#[test]
fn degree0_rounding_and_domain_endpoints() {
    let rows: &[(&[i32], u8, i128, i64, u8)] = &[
        (&[-128], 8, -128, 0, 0),
        (&[127], 8, 127, 0, 0),
        (&[128], 8, 128, 1, 1),
        (&[129], 8, 129, 1, 1),
        (&[-129], 8, -129, -1, 255),
        (&[-127], 8, -127, 0, 0),
        (&[2_147_483_647], 8, 2_147_483_647, 8_388_608, 0),
        (&[-2_147_483_647], 8, -2_147_483_647, -8_388_608, 0),
        (&[8_388_607], 24, 8_388_607, 0, 0),
        (&[8_388_608], 24, 8_388_608, 1, 1),
        (&[8_388_609], 24, 8_388_609, 1, 1),
        (&[-8_388_609], 24, -8_388_609, -1, 255),
        (&[-8_388_608], 24, -8_388_608, 0, 0),
        (&[-8_388_607], 24, -8_388_607, 0, 0),
        (&[2_147_483_647], 24, 2_147_483_647, 128, 128),
        (&[-2_147_483_647], 24, -2_147_483_647, -128, 128),
    ];
    for (coefficients, q, acc, value, yhat) in rows {
        assert_case(*q, 0, coefficients, *acc, *value, *yhat);
    }
}

#[test]
fn shift_scale_is_not_optional() {
    assert_case(8, 2047, &[0, 1], 0, 0, 0);
    assert_case(8, 2047, &[0, 129], 128, 1, 1);
}

#[test]
fn degree15_boundaries_and_coefficient_order() {
    let forward = alternating();
    assert_case(8, 0, &forward, 2_147_483_647, 8_388_608, 0);
    assert_case(8, 2047, &forward, 8_359_991, 32_656, 144);
    assert_case(24, 0, &forward, 2_147_483_647, 128, 128);
    assert_case(24, 2047, &forward, 8_359_991, 0, 0);

    let mut reversed = forward;
    reversed.reverse();
    assert_case(8, 2047, &reversed, -8_360_004, -32_656, 112);
}

#[test]
fn width_bound_at_n_2047() {
    // Next to the oracle asserts: `|acc| <= (degree + 1)·2^31 < 2^35` and
    // `|acc·u| < 2^51`. `|c| <= 2^31 - 1` tightens the first to
    // `|acc| <= (degree + 1)·(2^31 - 1) = 2^35 - 16` at degree 15.
    let alternating = alternating();
    let vectors: [&[i32]; 3] = [&[i32::MAX; 16], &[i32::MIN + 1; 16], &alternating];
    for coefficients in vectors {
        let (_acc, value) = oracle(coefficients, 24, 2047);
        let poly = Polynomial::new(15, 24, coefficients).unwrap();
        assert_eq!(poly.value(2047).unwrap(), i64::try_from(value).unwrap());
    }
}

#[test]
fn prediction_hash_and_block_length_prefix() {
    let coefficients = alternating();
    let poly = Polynomial::new(15, 24, &coefficients).unwrap();
    let mut bytes = [0u8; 2048];
    for n in 0..2048u16 {
        bytes[usize::from(n)] = poly.predict(n).unwrap();
    }
    assert_eq!(sha256_hex(&bytes), POLYNOMIAL_SHA256.trim());
    for n in 0..=8 {
        assert_eq!(poly.predict(n).unwrap(), 128);
    }
    for n in 9..=15 {
        assert_eq!(poly.predict(n).unwrap(), 127);
    }
    assert_eq!(poly.predict(255).unwrap(), 114);

    let mut acc = i128::from(coefficients[15]);
    let u = 2i128 << 8;
    for k in (0..15).rev() {
        acc = ((acc * u) >> 16) + i128::from(coefficients[k]);
    }
    let scaled = ((acc + (1i128 << 23)) >> 24).rem_euclid(256) as u8;
    assert_eq!(poly.predict(2).unwrap(), 128);
    assert_eq!(scaled, 127);
}

#[test]
fn invalid_neighbours_are_rejected() {
    assert_eq!(
        Polynomial::new(16, 8, &[0; 17]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(Polynomial::new(0, 7, &[0]), Err(Error::InvalidParameter));
    assert_eq!(Polynomial::new(0, 25, &[0]), Err(Error::InvalidParameter));
    assert_eq!(
        Polynomial::new(0, 8, &[i32::MIN]),
        Err(Error::InvalidParameter)
    );
    let mut leading_min = [0i32; 16];
    leading_min[15] = i32::MIN;
    assert_eq!(
        Polynomial::new(15, 8, &leading_min),
        Err(Error::InvalidParameter)
    );
    let mut middle_min = [1i32, 1, 1];
    middle_min[1] = i32::MIN;
    assert_eq!(
        Polynomial::new(2, 8, &middle_min),
        Err(Error::InvalidParameter)
    );
    assert_eq!(Polynomial::new(0, 8, &[]), Err(Error::InvalidParameter));
    assert_eq!(Polynomial::new(0, 8, &[0, 0]), Err(Error::InvalidParameter));
    assert_eq!(
        Polynomial::new(15, 8, &[0; 15]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Polynomial::new(15, 8, &[0; 17]),
        Err(Error::InvalidParameter)
    );
    let poly = Polynomial::new(0, 8, &[-128]).unwrap();
    assert_eq!(poly.value(2048), Err(Error::InvalidParameter));
    assert_eq!(poly.predict(2048), Err(Error::InvalidParameter));
    assert_eq!(poly.value(0).unwrap(), 0);
}

#[derive(Clone, Debug)]
enum Illegal {
    Degree(u8),
    Q(u8),
    MinCoeff { degree: u8, index: u8 },
    Position(u16),
}

fn illegal_case() -> impl Strategy<Value = Illegal> {
    prop_oneof![
        (16u8..=255).prop_map(Illegal::Degree),
        prop_oneof![0u8..=7u8, 25u8..=255u8].prop_map(Illegal::Q),
        (0u8..=15).prop_flat_map(|degree| {
            (0u8..=degree).prop_map(move |index| Illegal::MinCoeff { degree, index })
        }),
        (2048u16..=u16::MAX).prop_map(Illegal::Position),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn legal_inputs_match_i128_oracle(
        degree in 0u8..=15,
        q in 8u8..=24,
        n in 0u16..=2047,
        coefficients in proptest::collection::vec(i32::MIN + 1..=i32::MAX, 16),
    ) {
        let count = usize::from(degree) + 1;
        let coefficients = &coefficients[..count];
        let (_acc, value) = oracle(coefficients, q, n);
        let poly = Polynomial::new(degree, q, coefficients).unwrap();
        let got = poly.value(n).unwrap();
        prop_assert_eq!(got, i64::try_from(value).unwrap());
        prop_assert_eq!(poly.predict(n).unwrap(), got.rem_euclid(256) as u8);
    }

    #[test]
    fn illegal_neighbour_is_invalid_parameter(case in illegal_case()) {
        let result = match case {
            Illegal::Degree(degree) => Polynomial::new(degree, 8, &[0; 17]).map(|_| ()),
            Illegal::Q(q) => Polynomial::new(0, q, &[0]).map(|_| ()),
            Illegal::MinCoeff { degree, index } => {
                let mut coefficients = vec![0i32; usize::from(degree) + 1];
                let slot = coefficients.get_mut(usize::from(index)).unwrap();
                *slot = i32::MIN;
                Polynomial::new(degree, 8, &coefficients).map(|_| ())
            }
            Illegal::Position(n) => {
                let poly = Polynomial::new(0, 8, &[0]).unwrap();
                poly.value(n).and(poly.predict(n)).map(|_| ())
            }
        };
        prop_assert_eq!(result, Err(Error::InvalidParameter));
    }
}
