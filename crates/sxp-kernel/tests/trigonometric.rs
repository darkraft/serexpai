// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0004),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every coefficient tuple.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sha2::{Digest, Sha256};
use sxp_kernel::{Error, Trigonometric, check_sine_lut};

const SINE_LUT_SHA256: &str = include_str!("../../../spec/sine_lut.sha256");
const TRIGONOMETRIC_SHA256: &str =
    include_str!("../../../spec/vectors/trigonometric-order16-q8.sha256");
const SINE_LUT_BYTES: &[u8] = include_bytes!("../../../spec/sine_lut.bin");

/// `|c · LUT| < 2^43` and `|c0 << 15| < 2^43`. Seed §4.2 says each term
/// `< 2^28 · 2^15 = 2^43`. The strict bound holds because `|c| <= 2^28 - 1`
/// and `|LUT| <= 32767`, so `|c · LUT| <= (2^28 - 1) · 32767 = 8795824553985`
/// and `|c0 << 15| <= (2^28 - 1) · 32768 = 8796092989440`, both
/// `< 2^43 = 8796093022208`.
const TERM_LIMIT: i128 = 1i128 << 43;
/// `|S| < 2^49`. Seed §4.2: at most 33 terms, each `< 2^43`.
/// `|S| <= 8796092989440 + 32 · 8795824553985 = 290262478716960 < 2^49`.
const SUM_LIMIT: i128 = 1i128 << 49;

const COEFF_MAX: i32 = (1 << 28) - 1;

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0004),
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

fn load_lut() -> [i16; 1025] {
    assert_eq!(SINE_LUT_BYTES.len(), 2050);
    let mut lut = [0i16; 1025];
    for (index, slot) in lut.iter_mut().enumerate() {
        let start = index * 2;
        let pair = SINE_LUT_BYTES.get(start..start + 2).unwrap();
        let array: [u8; 2] = pair.try_into().unwrap();
        *slot = i16::from_le_bytes(array);
    }
    lut
}

fn sin_lut(lut: &[i16; 1025], p: u32) -> i128 {
    let index = usize::from((p >> 22) as u16);
    let fraction = i128::from((p >> 6) & 0xFFFF);
    let v0 = i128::from(*lut.get(index).unwrap());
    let v1 = i128::from(*lut.get(index + 1).unwrap());
    v0 + (((v1 - v0) * fraction + (1i128 << 15)) >> 16)
}

/// Independent `i128` evaluation. `S` is the pre-round accumulator.
fn oracle(coefficients: &[i32], q: u8, omega: u32, n: u16) -> (i128, i128) {
    let lut = load_lut();
    let theta = (u128::from(n) * u128::from(omega)) as u32;
    let mut sum = i128::from(*coefficients.first().unwrap()) << 15;
    assert!(sum.abs() < TERM_LIMIT);
    let order = u8::try_from(coefficients.len() / 2).unwrap();
    for k in 1..=order {
        let p = (u128::from(k) * u128::from(theta)) as u32;
        let p_cos = (u128::from(p) + (1u128 << 30)) as u32;
        let a = i128::from(*coefficients.get(usize::from(2 * k - 1)).unwrap());
        let b = i128::from(*coefficients.get(usize::from(2 * k)).unwrap());
        let sine = sin_lut(&lut, p);
        let cosine = sin_lut(&lut, p_cos);
        let sine_term = a * sine;
        let cosine_term = b * cosine;
        assert!(sine_term.abs() < TERM_LIMIT);
        assert!(cosine_term.abs() < TERM_LIMIT);
        sum += sine_term + cosine_term;
        assert!(sum.abs() < SUM_LIMIT);
    }
    let value = (sum + (1i128 << (q + 14))) >> (q + 15);
    (sum, value)
}

struct Expected<'a> {
    order: u8,
    q: u8,
    omega: u32,
    n: u16,
    coefficients: &'a [i32],
    sum: i128,
    value: i64,
    yhat: u8,
}

fn assert_expected(case: &Expected<'_>) {
    let (oracle_sum, oracle_value) = oracle(case.coefficients, case.q, case.omega, case.n);
    assert_eq!(oracle_sum, case.sum);
    assert_eq!(oracle_value, i128::from(case.value));
    let trig = Trigonometric::new(case.order, case.q, case.omega, case.coefficients).unwrap();
    assert_eq!(trig.value(case.n).unwrap(), case.value);
    assert_eq!(trig.predict(case.n).unwrap(), case.yhat);
}

fn alternating() -> [i32; 33] {
    let mut coefficients = [0i32; 33];
    for (index, slot) in coefficients.iter_mut().enumerate() {
        *slot = if index % 2 == 0 {
            COEFF_MAX
        } else {
            -COEFF_MAX
        };
    }
    coefficients
}

#[test]
fn constant_rounding_and_domain_endpoints() {
    let rows: &[(i32, u8, i128, i64, u8)] = &[
        (-129, 8, -4_227_072, -1, 255),
        (-128, 8, -4_194_304, 0, 0),
        (-127, 8, -4_161_536, 0, 0),
        (127, 8, 4_161_536, 0, 0),
        (128, 8, 4_194_304, 1, 1),
        (129, 8, 4_227_072, 1, 1),
        (COEFF_MAX, 8, 8_796_092_989_440, 1_048_576, 0),
        (-COEFF_MAX, 8, -8_796_092_989_440, -1_048_576, 0),
        (8_388_607, 24, 274_877_874_176, 0, 0),
        (8_388_608, 24, 274_877_906_944, 1, 1),
        (8_388_609, 24, 274_877_939_712, 1, 1),
        (-8_388_609, 24, -274_877_939_712, -1, 255),
        (-8_388_608, 24, -274_877_906_944, 0, 0),
        (-8_388_607, 24, -274_877_874_176, 0, 0),
        (COEFF_MAX, 24, 8_796_092_989_440, 16, 16),
        (-COEFF_MAX, 24, -8_796_092_989_440, -16, 240),
    ];
    for (c0, q, sum, value, yhat) in rows {
        let coefficients = [*c0, 0, 0];
        assert_expected(&Expected {
            order: 1,
            q: *q,
            omega: 0,
            n: 0,
            coefficients: &coefficients,
            sum: *sum,
            value: *value,
            yhat: *yhat,
        });
    }
}

#[test]
fn harmonics_quadrants_interpolation_and_phase() {
    let rows = [
        Expected {
            order: 1,
            q: 8,
            omega: 0,
            n: 0,
            coefficients: &[0, 129, 0],
            sum: 0,
            value: 0,
            yhat: 0,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 0,
            n: 0,
            coefficients: &[0, 0, 129],
            sum: 4_226_943,
            value: 1,
            yhat: 1,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 1_073_741_824,
            n: 1,
            coefficients: &[0, 129, 0],
            sum: 4_226_943,
            value: 1,
            yhat: 1,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 2_147_483_648,
            n: 1,
            coefficients: &[0, 129, 0],
            sum: 0,
            value: 0,
            yhat: 0,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 3_221_225_472,
            n: 1,
            coefficients: &[0, 129, 0],
            sum: -4_226_943,
            value: -1,
            yhat: 255,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 1_073_741_824,
            n: 1,
            coefficients: &[0, 0, 129],
            sum: 0,
            value: 0,
            yhat: 0,
        },
        Expected {
            order: 2,
            q: 8,
            omega: 2_147_483_648,
            n: 1,
            coefficients: &[0, 0, 0, 0, 129],
            sum: 4_226_943,
            value: 1,
            yhat: 1,
        },
        Expected {
            order: 2,
            q: 8,
            omega: 2_147_483_648,
            n: 1,
            coefficients: &[0, 0, 0, 129, 0],
            sum: 0,
            value: 0,
            yhat: 0,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 2_097_088,
            n: 1,
            coefficients: &[0, 4_194_304, 0],
            sum: 419_430_400,
            value: 50,
            yhat: 50,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 2_097_152,
            n: 1,
            coefficients: &[0, 4_194_304, 0],
            sum: 423_624_704,
            value: 51,
            yhat: 51,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 2_097_216,
            n: 1,
            coefficients: &[0, 4_194_304, 0],
            sum: 423_624_704,
            value: 51,
            yhat: 51,
        },
        Expected {
            order: 1,
            q: 8,
            omega: u32::MAX,
            n: 1,
            coefficients: &[0, 8_388_608, 0],
            sum: 0,
            value: 0,
            yhat: 0,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 3_221_225_472,
            n: 2047,
            coefficients: &[0, 129, 0],
            sum: 4_226_943,
            value: 1,
            yhat: 1,
        },
        Expected {
            order: 1,
            q: 8,
            omega: 3_221_225_472,
            n: 0,
            coefficients: &[0, 129, 0],
            sum: 0,
            value: 0,
            yhat: 0,
        },
    ];
    for case in &rows {
        assert_expected(case);
    }

    let quadrant = Trigonometric::new(1, 8, 3_221_225_472, &[0, 129, 0]).unwrap();
    assert_eq!(quadrant.predict(2047).unwrap(), 1);
    assert_eq!(quadrant.predict(0).unwrap(), 0);
    let wrong_theta = ((u64::from(2047u32) << 5) * 3_221_225_472u64) as u32;
    assert_eq!(wrong_theta, 0);
}

#[test]
fn order16_both_q_limits() {
    let coefficients = alternating();
    let rows: &[(u8, u16, i128, i64, u8)] = &[
        (8, 0, 149_529_285_853_200, 17_825_280, 0),
        (8, 1, 116_637_084_245_685, 13_904_224, 96),
        (8, 2047, 174_557_133_935_580, 20_808_832, 128),
        (24, 0, 149_529_285_853_200, 272, 16),
        (24, 1, 116_637_084_245_685, 212, 212),
        (24, 2047, 174_557_133_935_580, 318, 62),
    ];
    for (q, n, sum, value, yhat) in rows {
        assert_expected(&Expected {
            order: 16,
            q: *q,
            omega: 16_777_217,
            n: *n,
            coefficients: &coefficients,
            sum: *sum,
            value: *value,
            yhat: *yhat,
        });
    }
}

#[test]
fn width_bound_on_order16() {
    let omega = u32::MAX;
    let vectors = [
        vec![COEFF_MAX; 33],
        vec![-COEFF_MAX; 33],
        alternating().to_vec(),
    ];
    for coefficients in vectors {
        let (_sum, value) = oracle(&coefficients, 8, omega, 2047);
        let trig = Trigonometric::new(16, 8, omega, &coefficients).unwrap();
        assert_eq!(trig.value(2047).unwrap(), i64::try_from(value).unwrap());
    }
}

#[test]
fn table_integrity_and_prediction_hash() {
    let lut = load_lut();
    assert_eq!(lut[0], lut[1024]);
    assert_eq!(
        [
            lut[0], lut[1], lut[256], lut[512], lut[768], lut[1023], lut[1024]
        ],
        [0, 201, 32767, 0, -32767, -201, 0]
    );
    assert!(lut.iter().all(|entry| i32::from(*entry).abs() <= 32767));
    let max_step = lut.windows(2).map(|pair| (pair[1] - pair[0]).abs()).max();
    assert_eq!(max_step, Some(201));
    assert_eq!(sha256_hex(SINE_LUT_BYTES), SINE_LUT_SHA256.trim());
    assert_eq!(check_sine_lut(SINE_LUT_BYTES), Ok(()));

    let mut flipped = SINE_LUT_BYTES.to_vec();
    flipped[4] ^= 0x01;
    assert_eq!(check_sine_lut(&flipped), Err(Error::InvalidParameter));
    assert_ne!(sha256_hex(&flipped), SINE_LUT_SHA256.trim());
    assert_eq!(
        check_sine_lut(&SINE_LUT_BYTES[..2049]),
        Err(Error::InvalidParameter)
    );
    let mut longer = SINE_LUT_BYTES.to_vec();
    longer.push(0);
    assert_eq!(check_sine_lut(&longer), Err(Error::InvalidParameter));

    let coefficients = alternating();
    let trig = Trigonometric::new(16, 8, 16_777_217, &coefficients).unwrap();
    let mut bytes = [0u8; 2048];
    for n in 0..2048u16 {
        bytes[usize::from(n)] = trig.predict(n).unwrap();
    }
    assert_eq!(sha256_hex(&bytes), TRIGONOMETRIC_SHA256.trim());
    assert_eq!(
        bytes[..16],
        [
            0x00, 0x60, 0x40, 0x40, 0x20, 0x80, 0x80, 0x00, 0x00, 0xa0, 0x00, 0x60, 0xe0, 0xc0,
            0x00, 0x40
        ]
    );
    assert_eq!(bytes[255], 160);
    assert_eq!(bytes[2047], 128);
}

#[test]
fn invalid_neighbours_are_rejected() {
    assert_eq!(
        Trigonometric::new(0, 8, 0, &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(17, 8, 0, &[0; 35]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 7, 0, &[0, 0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 25, 0, &[0, 0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 8, 0, &[1 << 28, 0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 8, 0, &[-(1 << 28), 0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 8, 0, &[i32::MIN, 0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 8, 0, &[i32::MAX, 0, 0]),
        Err(Error::InvalidParameter)
    );

    let mut middle = vec![0i32; 33];
    middle[15] = 1 << 28;
    assert_eq!(
        Trigonometric::new(16, 8, 0, &middle),
        Err(Error::InvalidParameter)
    );
    middle[15] = 0;
    middle[32] = -(1 << 28);
    assert_eq!(
        Trigonometric::new(16, 8, 0, &middle),
        Err(Error::InvalidParameter)
    );
    middle[32] = 0;
    middle[15] = -(1 << 28);
    assert_eq!(
        Trigonometric::new(16, 8, 0, &middle),
        Err(Error::InvalidParameter)
    );
    middle[15] = 0;
    middle[32] = 1 << 28;
    assert_eq!(
        Trigonometric::new(16, 8, 0, &middle),
        Err(Error::InvalidParameter)
    );

    assert_eq!(
        Trigonometric::new(1, 8, 0, &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(1, 8, 0, &[0; 5]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(16, 8, 0, &[0; 32]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Trigonometric::new(16, 8, 0, &[0; 34]),
        Err(Error::InvalidParameter)
    );

    let legal = Trigonometric::new(1, 8, 0, &[COEFF_MAX, 0, 0]).unwrap();
    assert_eq!(legal.value(2048), Err(Error::InvalidParameter));
    assert_eq!(legal.predict(2048), Err(Error::InvalidParameter));
    assert_eq!(legal.value(0).unwrap(), 1_048_576);
    assert_eq!(Trigonometric::new(1, 8, 0, &[0, 0, 0]).map(|_| ()), Ok(()));
    assert_eq!(
        Trigonometric::new(1, 8, u32::MAX, &[0, 0, 0]).map(|_| ()),
        Ok(())
    );
}

#[derive(Clone, Debug)]
enum Illegal {
    Order(u8),
    Q(u8),
    Coeff { order: u8, index: u8, value: i32 },
    Position(u16),
}

fn illegal_case() -> impl Strategy<Value = Illegal> {
    prop_oneof![
        prop_oneof![Just(0u8), 17u8..=255].prop_map(Illegal::Order),
        prop_oneof![0u8..=7, 25u8..=255].prop_map(Illegal::Q),
        (1u8..=16).prop_flat_map(|order| {
            let last = order * 2;
            (0u8..=last, 0u8..4).prop_map(move |(index, which)| {
                let value = match which {
                    0 => 1 << 28,
                    1 => -(1 << 28),
                    2 => i32::MAX,
                    _ => i32::MIN,
                };
                Illegal::Coeff {
                    order,
                    index,
                    value,
                }
            })
        }),
        (2048u16..=u16::MAX).prop_map(Illegal::Position),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn legal_inputs_match_i128_oracle(
        order in 1u8..=16,
        q in 8u8..=24,
        n in 0u16..=2047,
        omega in any::<u32>(),
        coefficients in proptest::collection::vec(-COEFF_MAX..=COEFF_MAX, 33),
    ) {
        let count = 2 * usize::from(order) + 1;
        let coefficients = &coefficients[..count];
        let (_sum, value) = oracle(coefficients, q, omega, n);
        let trig = Trigonometric::new(order, q, omega, coefficients).unwrap();
        let got = trig.value(n).unwrap();
        prop_assert_eq!(got, i64::try_from(value).unwrap());
        prop_assert_eq!(trig.predict(n).unwrap(), got.rem_euclid(256) as u8);
    }

    #[test]
    fn illegal_neighbour_is_invalid_parameter(case in illegal_case()) {
        let result = match case {
            Illegal::Order(order) => {
                let count = if order == 0 { 1 } else { 35 };
                Trigonometric::new(order, 8, 0, &vec![0; count]).map(|_| ())
            }
            Illegal::Q(q) => Trigonometric::new(1, q, 0, &[0, 0, 0]).map(|_| ()),
            Illegal::Coeff { order, index, value } => {
                let mut coefficients = vec![0i32; 2 * usize::from(order) + 1];
                let slot = coefficients.get_mut(usize::from(index)).unwrap();
                *slot = value;
                Trigonometric::new(order, 8, 0, &coefficients).map(|_| ())
            }
            Illegal::Position(n) => {
                let trig = Trigonometric::new(1, 8, 0, &[0, 0, 0]).unwrap();
                trig.value(n).and(trig.predict(n)).map(|_| ())
            }
        };
        prop_assert_eq!(result, Err(Error::InvalidParameter));
    }
}
