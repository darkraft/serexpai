// fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
// cases = 256, rng_seed = RngSeed::Fixed(0x5111_0005),
// failure_persistence = FileFailurePersistence::Off
// Finite cases are not a proof over every coefficient tuple.

use std::fmt::Write as _;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngSeed};
use sha2::{Digest, Sha256};
use sxp_kernel::{Error, Recurrence};

const RECURRENCE_SHA256: &str = include_str!("../../../spec/vectors/recurrence-order2-m24.sha256");

/// `|a·x| <= (2^24 - 1)^2`. Seed §4.3 says `|a·x| < 2^48`.
/// That strict bound holds because `|a| <= 2^24 - 1` and `x <= 2^24 - 1`,
/// so `|a·x| <= 281474943156225 < 2^48 = 281474976710656`.
const AX_LIMIT: i128 = 281_474_943_156_225;
/// `|a·x + b·x' + c| <= 2·(2^24 - 1)^2 + (2^24 - 1)`.
/// `|a·x + b·x'| <= 562949886312450 < 2^49`, and the full sum is
/// `562949903089665 < 2^49 = 562949953421312`.
const LINEAR_LIMIT: i128 = 562_949_903_089_665;
const COEFF_MAX: i32 = (1 << 24) - 1;
const MOD_MAX: u32 = 1 << 24;

fn config() -> Config {
    Config {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        rng_seed: RngSeed::Fixed(0x5111_0005),
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

fn assert_product(product: i128) {
    assert!(product.abs() <= AX_LIMIT);
    assert!(product.abs() < (1i128 << 48));
}

fn assert_linear(linear: i128) {
    assert!(linear.abs() <= LINEAR_LIMIT);
    assert!(linear.abs() < (1i128 << 49));
}

/// Independent `i128` recurrence. `linear` is the pre-remainder combination
/// of the step that produced `x[n]`, absent when that step does not exist.
fn oracle(coefficients: &[i32], state: &[u32], m_mod: u32, n: u16) -> (Option<i128>, i128) {
    let modulus = i128::from(m_mod);
    let a = i128::from(*coefficients.first().unwrap());
    let b = i128::from(*coefficients.get(1).unwrap());
    if state.len() == 1 {
        let mut x = i128::from(*state.first().unwrap());
        let mut linear = None;
        for _ in 0..n {
            let product = a * x;
            assert_product(product);
            let step = product + b;
            assert_linear(step);
            linear = Some(step);
            x = step.rem_euclid(modulus);
        }
        return (linear, x);
    }
    let c = i128::from(*coefficients.get(2).unwrap());
    let s0 = i128::from(*state.first().unwrap());
    if n == 0 {
        return (None, s0);
    }
    let mut prev = s0;
    let mut curr = i128::from(*state.get(1).unwrap());
    let mut linear = None;
    for _ in 1..n {
        let ax = a * curr;
        let bx = b * prev;
        assert_product(ax);
        assert_product(bx);
        let step = ax + bx + c;
        assert_linear(step);
        linear = Some(step);
        prev = curr;
        curr = step.rem_euclid(modulus);
    }
    (linear, curr)
}

struct Case {
    m_mod: u32,
    coefficients: &'static [i32],
    state: &'static [u32],
    n: u16,
    linear: Option<i128>,
    value: i64,
    yhat: u8,
}

fn case(
    m_mod: u32,
    coefficients: &'static [i32],
    state: &'static [u32],
    n: u16,
    linear: Option<i128>,
    value: i64,
    yhat: u8,
) -> Case {
    Case {
        m_mod,
        coefficients,
        state,
        n,
        linear,
        value,
        yhat,
    }
}

fn assert_case(case: &Case) {
    let order = u8::try_from(case.state.len()).unwrap();
    let (oracle_linear, oracle_value) = oracle(case.coefficients, case.state, case.m_mod, case.n);
    assert_eq!(oracle_linear, case.linear);
    assert_eq!(oracle_value, i128::from(case.value));
    let formula = Recurrence::new(order, case.m_mod, case.coefficients, case.state).unwrap();
    assert_eq!(formula.value(case.n).unwrap(), case.value);
    assert_eq!(formula.predict(case.n).unwrap(), case.yhat);
}

#[test]
fn order1_sequences_and_domain_endpoints() {
    let rows = [
        case(2, &[0, 0], &[0], 0, None, 0, 0),
        case(2, &[0, 0], &[0], 2047, Some(0), 0, 0),
        case(2, &[1, 1], &[0], 0, None, 0, 0),
        case(2, &[1, 1], &[0], 1, Some(1), 1, 1),
        case(2, &[1, 1], &[0], 2047, Some(1), 1, 1),
        case(2, &[1, 0], &[1], 0, None, 1, 1),
        case(2, &[1, 0], &[1], 2047, Some(1), 1, 1),
        case(5, &[-1, 0], &[1], 1, Some(-1), 4, 4),
        case(5, &[-1, 0], &[1], 2047, Some(-1), 4, 4),
        case(7, &[2, 3], &[2], 1, Some(7), 0, 0),
        case(5, &[2, 1], &[0], 1, Some(1), 1, 1),
        case(MOD_MAX, &[1, 1], &[0], 256, Some(256), 256, 0),
        case(MOD_MAX, &[1, 1], &[0], 2047, Some(2047), 2047, 255),
        case(
            MOD_MAX,
            &[0, 0],
            &[COEFF_MAX as u32],
            0,
            None,
            16_777_215,
            255,
        ),
        case(MOD_MAX, &[0, 0], &[COEFF_MAX as u32], 1, Some(0), 0, 0),
        case(
            MOD_MAX,
            &[COEFF_MAX, -COEFF_MAX],
            &[COEFF_MAX as u32],
            1,
            Some(281_474_926_379_010),
            2,
            2,
        ),
        case(MOD_MAX, &[-COEFF_MAX, COEFF_MAX], &[1], 1, Some(0), 0, 0),
    ];
    for row in &rows {
        assert_case(row);
    }
}

#[test]
fn order2_sequences_and_domain_endpoints() {
    let rows = [
        case(2, &[1, 1, 0], &[0, 1], 0, None, 0, 0),
        case(2, &[1, 1, 0], &[0, 1], 1, None, 1, 1),
        case(2, &[1, 1, 0], &[0, 1], 2, Some(1), 1, 1),
        case(2, &[1, 1, 0], &[0, 1], 2047, Some(1), 1, 1),
        case(2, &[1, 0, 1], &[1, 1], 0, None, 1, 1),
        case(2, &[1, 0, 1], &[1, 1], 2, Some(2), 0, 0),
        case(7, &[2, 3, 0], &[1, 2], 2, Some(7), 0, 0),
        case(7, &[2, 3, 5], &[1, 2], 2, Some(12), 5, 5),
        case(5, &[0, -1, 0], &[1, 0], 2, Some(-1), 4, 4),
        case(5, &[0, 0, -3], &[0, 0], 2, Some(-3), 2, 2),
        case(
            MOD_MAX,
            &[0, 0, COEFF_MAX],
            &[0, 0],
            2,
            Some(i128::from(COEFF_MAX)),
            i64::from(COEFF_MAX),
            255,
        ),
        case(
            MOD_MAX,
            &[0, 0, -COEFF_MAX],
            &[0, 0],
            2,
            Some(i128::from(-COEFF_MAX)),
            1,
            1,
        ),
        case(
            MOD_MAX,
            &[COEFF_MAX, -COEFF_MAX, 1],
            &[0, COEFF_MAX as u32],
            0,
            None,
            0,
            0,
        ),
        case(
            MOD_MAX,
            &[COEFF_MAX, -COEFF_MAX, 1],
            &[0, COEFF_MAX as u32],
            1,
            None,
            i64::from(COEFF_MAX),
            255,
        ),
        case(
            MOD_MAX,
            &[COEFF_MAX, -COEFF_MAX, 1],
            &[0, COEFF_MAX as u32],
            2,
            Some(281_474_943_156_226),
            2,
            2,
        ),
        case(
            MOD_MAX,
            &[COEFF_MAX, -COEFF_MAX, 1],
            &[0, COEFF_MAX as u32],
            3,
            Some(-281_474_909_601_794),
            16_777_214,
            254,
        ),
    ];
    for row in &rows {
        assert_case(row);
    }
}

#[test]
fn width_bound_holds_at_the_endpoints() {
    let state = [COEFF_MAX as u32, COEFF_MAX as u32];
    let vectors: &[&[i32]] = &[
        &[COEFF_MAX, COEFF_MAX, COEFF_MAX],
        &[-COEFF_MAX, -COEFF_MAX, -COEFF_MAX],
        &[COEFF_MAX, -COEFF_MAX, COEFF_MAX],
    ];
    let mixed_state = [0u32, COEFF_MAX as u32];
    for (index, coefficients) in vectors.iter().enumerate() {
        let state = if index == 2 { &mixed_state } else { &state };
        for n in 0u16..=2047 {
            let (_linear, value) = oracle(coefficients, state, MOD_MAX, n);
            let formula = Recurrence::new(2, MOD_MAX, coefficients, state).unwrap();
            assert_eq!(formula.value(n).unwrap(), i64::try_from(value).unwrap());
        }
    }
}

#[test]
fn prediction_hash_matches_the_committed_digest() {
    let formula = Recurrence::new(
        2,
        MOD_MAX,
        &[COEFF_MAX, -COEFF_MAX, 1],
        &[0, COEFF_MAX as u32],
    )
    .unwrap();
    let mut out = Vec::with_capacity(2048);
    for n in 0u16..=2047 {
        let (_linear, value) = oracle(
            &[COEFF_MAX, -COEFF_MAX, 1],
            &[0, COEFF_MAX as u32],
            MOD_MAX,
            n,
        );
        assert_eq!(formula.value(n).unwrap(), i64::try_from(value).unwrap());
        out.push(formula.predict(n).unwrap());
    }
    let prefix = [
        0x00, 0xff, 0x02, 0xfe, 0x05, 0xfa, 0x0c, 0xef, 0x1e, 0xd2, 0x4d, 0x86, 0xc8, 0xbf, 0x0a,
        0xb6,
    ];
    assert_eq!(out.get(..16).unwrap(), &prefix);
    assert_eq!(*out.first().unwrap(), 0);
    assert_eq!(*out.get(1).unwrap(), 255);
    assert_eq!(*out.get(2).unwrap(), 2);
    assert_eq!(*out.get(255).unwrap(), 150);
    assert_eq!(*out.get(2047).unwrap(), 111);
    assert_eq!(sha256_hex(&out), RECURRENCE_SHA256.trim());
}

#[test]
fn invalid_neighbours_are_rejected() {
    assert_eq!(
        Recurrence::new(0, 2, &[0, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(0, 2, &[0], &[]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(3, 2, &[0, 0, 0, 0], &[0, 0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 0, &[0, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 1, &[0, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, MOD_MAX + 1, &[0, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, u32::MAX, &[0, 0], &[0]),
        Err(Error::InvalidParameter)
    );

    assert_eq!(
        Recurrence::new(1, 2, &[1 << 24, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[-(1 << 24), 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, 1 << 24], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, -(1 << 24)], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[1 << 24, 0, 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 1 << 24, 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 1 << 24], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[-(1 << 24), 0, 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, -(1 << 24), 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, -(1 << 24)], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[i32::MIN, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[i32::MAX, 0], &[0]),
        Err(Error::InvalidParameter)
    );

    assert_eq!(
        Recurrence::new(1, 2, &[0, 0], &[2]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, 0], &[3]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, 0], &[u32::MAX]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, MOD_MAX, &[0, 0], &[MOD_MAX]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 0], &[0, 2]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 0], &[0, u32::MAX]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, MOD_MAX, &[0, 0, 0], &[0, MOD_MAX]),
        Err(Error::InvalidParameter)
    );

    assert_eq!(
        Recurrence::new(1, 2, &[0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, 0, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 0, 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, 0], &[]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(1, 2, &[0, 0], &[0, 0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 0], &[0]),
        Err(Error::InvalidParameter)
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 0], &[0, 0, 0]),
        Err(Error::InvalidParameter)
    );

    let legal = Recurrence::new(1, 2, &[0, 0], &[0]).unwrap();
    assert_eq!(legal.value(2048), Err(Error::InvalidParameter));
    assert_eq!(legal.predict(2048), Err(Error::InvalidParameter));

    assert_eq!(
        Recurrence::new(1, 2, &[COEFF_MAX, -COEFF_MAX], &[0]).map(|_| ()),
        Ok(())
    );
    assert_eq!(
        Recurrence::new(
            2,
            MOD_MAX,
            &[COEFF_MAX, -COEFF_MAX, COEFF_MAX],
            &[0, COEFF_MAX as u32]
        )
        .map(|_| ()),
        Ok(())
    );
    assert_eq!(
        Recurrence::new(1, MOD_MAX, &[0, 0], &[MOD_MAX - 1]).map(|_| ()),
        Ok(())
    );
    assert_eq!(
        Recurrence::new(2, 2, &[0, 0, 0], &[1, 0]).map(|_| ()),
        Ok(())
    );
}

#[test]
fn each_call_restarts_at_the_initial_state() {
    let first = Recurrence::new(1, 5, &[-1, 0], &[1]).unwrap();
    let second = Recurrence::new(2, 7, &[2, 3, 5], &[1, 2]).unwrap();
    assert_eq!(first.predict(2047).unwrap(), 4);
    assert_eq!(second.predict(2).unwrap(), 5);
    assert_eq!(first.predict(0).unwrap(), 1);
    assert_eq!(first.predict(2047).unwrap(), 4);
    assert_eq!(first.predict(1).unwrap(), first.predict(1).unwrap());
}

#[derive(Clone, Debug)]
struct Legal {
    order: u8,
    m_mod: u32,
    n: u16,
    coefficients: Vec<i32>,
    state: Vec<u32>,
}

fn legal_case() -> impl Strategy<Value = Legal> {
    (1u8..=2, 2u32..=MOD_MAX, 0u16..=2047).prop_flat_map(|(order, m_mod, n)| {
        let coefficient_count = usize::from(order) + 1;
        let state_count = usize::from(order);
        (
            Just(order),
            Just(m_mod),
            Just(n),
            proptest::collection::vec(-COEFF_MAX..=COEFF_MAX, coefficient_count),
            proptest::collection::vec(0u32..m_mod, state_count),
        )
            .prop_map(|(order, m_mod, n, coefficients, state)| Legal {
                order,
                m_mod,
                n,
                coefficients,
                state,
            })
    })
}

#[derive(Clone, Debug)]
enum Illegal {
    Order(u8),
    Modulus(u32),
    Coeff { order: u8, index: u8, value: i32 },
    State { order: u8, index: u8, m_mod: u32 },
    Position(u16),
}

fn illegal_case() -> impl Strategy<Value = Illegal> {
    prop_oneof![
        prop_oneof![Just(0u8), 3u8..=255].prop_map(Illegal::Order),
        prop_oneof![Just(0u32), Just(1u32), (MOD_MAX + 1)..=u32::MAX].prop_map(Illegal::Modulus),
        (1u8..=2).prop_flat_map(|order| {
            (Just(order), 0u8..=order, 0u8..4).prop_map(|(order, index, which)| {
                let value = match which {
                    0 => 1 << 24,
                    1 => -(1 << 24),
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
        (1u8..=2, 2u32..=MOD_MAX).prop_flat_map(|(order, m_mod)| {
            (Just(order), Just(m_mod), 0u8..order).prop_map(|(order, m_mod, index)| {
                Illegal::State {
                    order,
                    index,
                    m_mod,
                }
            })
        }),
        (2048u16..=u16::MAX).prop_map(Illegal::Position),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn legal_inputs_match_i128_oracle(case in legal_case()) {
        let Legal {
            order,
            m_mod,
            n,
            coefficients,
            state,
        } = case;
        let (_linear, value) = oracle(&coefficients, &state, m_mod, n);
        let formula = Recurrence::new(order, m_mod, &coefficients, &state).unwrap();
        let got = formula.value(n).unwrap();
        prop_assert_eq!(got, i64::try_from(value).unwrap());
        prop_assert_eq!(formula.predict(n).unwrap(), got.rem_euclid(256) as u8);
    }

    #[test]
    fn illegal_neighbour_is_invalid_parameter(case in illegal_case()) {
        let result = match case {
            Illegal::Order(order) => {
                let (coefficients, state): (&[i32], &[u32]) = if order == 0 {
                    (&[0], &[])
                } else {
                    (&[0, 0], &[0])
                };
                Recurrence::new(order, 2, coefficients, state).map(|_| ())
            }
            Illegal::Modulus(m_mod) => Recurrence::new(1, m_mod, &[0, 0], &[0]).map(|_| ()),
            Illegal::Coeff { order, index, value } => {
                let mut coefficients = vec![0i32; usize::from(order) + 1];
                let slot = coefficients.get_mut(usize::from(index)).unwrap();
                *slot = value;
                let state = vec![0u32; usize::from(order)];
                Recurrence::new(order, 2, &coefficients, &state).map(|_| ())
            }
            Illegal::State { order, index, m_mod } => {
                let coefficients = vec![0i32; usize::from(order) + 1];
                let mut state = vec![0u32; usize::from(order)];
                let slot = state.get_mut(usize::from(index)).unwrap();
                *slot = m_mod;
                Recurrence::new(order, m_mod, &coefficients, &state).map(|_| ())
            }
            Illegal::Position(n) => {
                let formula = Recurrence::new(1, 2, &[0, 0], &[0]).unwrap();
                prop_assert_eq!(formula.value(n), Err(Error::InvalidParameter));
                prop_assert_eq!(formula.predict(n), Err(Error::InvalidParameter));
                return Ok(());
            }
        };
        prop_assert_eq!(result, Err(Error::InvalidParameter));
    }
}
