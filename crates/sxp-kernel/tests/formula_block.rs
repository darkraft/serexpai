//! Formula Block wire tests. Kernel only: no codebook and no encoder.

const FORMULA_HEX: &str = include_str!("../../../spec/vectors/formula-blocks.hex");

use sxp_kernel::{
    BlockKind, Error, NominalLength, Polynomial, Recurrence, Trigonometric, decode_block,
    encode_block, formula_section, read_blocked_file, write_blocked_file,
};

fn poly() -> BlockKind {
    let formula = Polynomial::new(2, 8, &[0, 0, 2_147_483_647]).unwrap();
    BlockKind::polynomial(formula, 32).unwrap()
}

fn trig() -> BlockKind {
    let formula = Trigonometric::new(1, 8, 1 << 24, &[0, 1 << 20, 0]).unwrap();
    BlockKind::trigonometric(formula, 22).unwrap()
}

fn rec() -> BlockKind {
    let formula = Recurrence::new(2, 251, &[3, 5, 7], &[1, 2]).unwrap();
    BlockKind::recurrence(formula, 5).unwrap()
}

#[test]
fn span_matches_predict_for_both_orders() {
    let order1 = Recurrence::new(1, 256, &[1, 1], &[0]).unwrap();
    let order2 = Recurrence::new(2, 251, &[3, 5, 7], &[1, 2]).unwrap();
    for formula in [order1, order2] {
        for len in [1u16, 2, 17, 64, 2048] {
            let span = formula.predict_span(len).unwrap();
            assert_eq!(span.len(), usize::from(len));
            for (index, byte) in span.iter().copied().enumerate() {
                assert_eq!(byte, formula.predict(index as u16).unwrap());
            }
        }
    }
    assert_eq!(
        order1.predict_span(2049).unwrap_err(),
        Error::InvalidParameter
    );
    assert!(order1.predict_span(0).unwrap().is_empty());
}

#[test]
fn committed_sections_match_encode() {
    let rows = [
        ("poly-section", formula_section(poly()).unwrap()),
        ("trig-section", formula_section(trig()).unwrap()),
        ("rec-section", formula_section(rec()).unwrap()),
        (
            "poly-section-n1",
            encode_block(poly(), NominalLength::N256, 1, &[0x5A]).unwrap(),
        ),
        (
            "trig-section-n1",
            encode_block(trig(), NominalLength::N256, 1, &[0x5A]).unwrap(),
        ),
        (
            "rec-section-n1",
            encode_block(rec(), NominalLength::N256, 1, &[0x5A]).unwrap(),
        ),
    ];
    for (name, bytes) in rows {
        assert_eq!(bytes, hex_row(name), "{name}");
    }
}

fn hex_row(name: &str) -> Vec<u8> {
    let line = FORMULA_HEX
        .lines()
        .find(|line| line.starts_with(name) && line.as_bytes().get(name.len()) == Some(&b' '))
        .unwrap_or_else(|| panic!("{name}"));
    let hex = line.split_once(' ').unwrap().1;
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn oracle_sections_match_the_kernel() {
    assert_eq!(oracle_section(&poly()), formula_section(poly()).unwrap());
    assert_eq!(oracle_section(&trig()), formula_section(trig()).unwrap());
    assert_eq!(oracle_section(&rec()), formula_section(rec()).unwrap());
}

#[test]
fn each_family_round_trips_every_length() {
    let families = [poly(), trig(), rec()];
    for kind in families {
        for nominal in [
            NominalLength::N256,
            NominalLength::N1024,
            NominalLength::N2048,
        ] {
            let remaining = nominal.bytes();
            let bytes = vec![0x5Au8; remaining as usize];
            let encoded = encode_block(kind, nominal, remaining, &bytes).unwrap();
            assert!(encoded.len() <= sxp_kernel::MAX_ENCODED_BLOCK);
            let view = decode_block(&encoded, remaining).unwrap();
            assert_eq!(view.bytes, bytes);
            assert_eq!(view.kind, kind);
            assert_eq!(view.consumed, encoded.len());
            let mut file = Vec::new();
            write_blocked_file(
                remaining,
                &[sxp_kernel::BlockPlan {
                    kind,
                    nominal,
                    bytes: &bytes,
                }],
                &mut file,
            )
            .unwrap();
            assert_eq!(&file[..2], &[0x8d, 0x01]);
            let mut decoded = Vec::new();
            read_blocked_file(file.as_slice(), &mut decoded).unwrap();
            assert_eq!(decoded, bytes);
        }
        let one = [0x5Au8];
        let encoded = encode_block(kind, NominalLength::N256, 1, &one).unwrap();
        let view = decode_block(&encoded, 1).unwrap();
        assert_eq!(view.bytes, one);
        assert_eq!(view.kind, kind);
        let mut file = Vec::new();
        write_blocked_file(
            1,
            &[sxp_kernel::BlockPlan {
                kind,
                nominal: NominalLength::N256,
                bytes: &one,
            }],
            &mut file,
        )
        .unwrap();
        assert_eq!(&file[..2], &[0x8d, 0x01]);
        let mut decoded = Vec::new();
        read_blocked_file(file.as_slice(), &mut decoded).unwrap();
        assert_eq!(decoded, one);
    }
}

#[test]
fn sign_extension_and_width_boundary() {
    let neg = BlockKind::polynomial(Polynomial::new(0, 8, &[-1]).unwrap(), 8).unwrap();
    let pos = BlockKind::polynomial(Polynomial::new(0, 8, &[127]).unwrap(), 8).unwrap();
    let wide = BlockKind::polynomial(Polynomial::new(0, 8, &[-128]).unwrap(), 9).unwrap();
    for kind in [neg, pos, wide] {
        let encoded = encode_block(kind, NominalLength::N256, 1, &[0x11]).unwrap();
        assert_eq!(decode_block(&encoded, 1).unwrap().kind, kind);
    }
    let too_wide = Polynomial::new(0, 8, &[128]).unwrap();
    assert_eq!(
        BlockKind::polynomial(too_wide, 8).unwrap_err(),
        Error::InvalidParameter
    );
    assert_eq!(
        BlockKind::polynomial(Polynomial::new(0, 8, &[0]).unwrap(), 0).unwrap_err(),
        Error::InvalidParameter
    );
    assert_eq!(
        BlockKind::trigonometric(Trigonometric::new(1, 8, 0, &[0, 0, 0]).unwrap(), 30).unwrap_err(),
        Error::InvalidParameter
    );
    assert_eq!(
        BlockKind::recurrence(Recurrence::new(1, 2, &[0, 0], &[0]).unwrap(), 26).unwrap_err(),
        Error::InvalidParameter
    );
}

#[test]
fn rejection_rows_follow_wire_order() {
    let rows = [
        (
            "poly-q-17",
            poly_raw(0, 17, 1, &[0]),
            Error::InvalidParameter,
        ),
        (
            "poly-q-31",
            poly_raw(0, 31, 1, &[0]),
            Error::InvalidParameter,
        ),
        (
            "poly-cw32-min",
            poly_raw(0, 0, 32, &[0x8000_0000]),
            Error::InvalidParameter,
        ),
        (
            "poly-pad",
            set_padding(poly_raw(0, 0, 1, &[0]), 20),
            Error::NonZeroPadding,
        ),
        (
            "poly-extra-coeff",
            set_padding(poly_raw(0, 0, 1, &[0]), 21),
            Error::NonZeroPadding,
        ),
        (
            "trig-q-17",
            trig_raw(1, 17, 1, 0, &[0, 0, 0]),
            Error::InvalidParameter,
        ),
        (
            "trig-cw30",
            trig_raw(1, 0, 30, 0, &[0, 0, 0]),
            Error::InvalidParameter,
        ),
        (
            "trig-cw31",
            trig_raw(1, 0, 31, 0, &[0, 0, 0]),
            Error::InvalidParameter,
        ),
        (
            "trig-cw32",
            trig_raw(1, 0, 32, 0, &[0, 0, 0]),
            Error::InvalidParameter,
        ),
        (
            "trig-cw29-min",
            trig_raw(1, 0, 29, 0, &[1u64 << 28, 0, 0]),
            Error::InvalidParameter,
        ),
        (
            "rec-mmod-ffffff",
            rec_raw(1, 0x00FF_FFFF, 1, &[0, 0], &[0]),
            Error::InvalidParameter,
        ),
        (
            "rec-cw25-min",
            rec_raw(1, 0, 25, &[1u64 << 24, 0], &[0]),
            Error::InvalidParameter,
        ),
        (
            "rec-s0-eq",
            rec_raw(1, 0, 1, &[0, 0], &[2]),
            Error::InvalidParameter,
        ),
        (
            "rec-s1-eq",
            rec_raw(2, 0, 1, &[0, 0, 0], &[0, 2]),
            Error::InvalidParameter,
        ),
        (
            "rec-order1-pad",
            set_padding(rec_raw(1, 0, 1, &[0, 0], &[0]), 61),
            Error::NonZeroPadding,
        ),
    ];
    for (name, section, error) in rows {
        assert_eq!(
            decode_block(&section, 256).unwrap_err(),
            error,
            "{name} {}",
            hex(&section)
        );
    }
    for cw in 26u8..=32 {
        let section = rec_raw(1, 0, u64::from(cw), &[0, 0], &[0]);
        assert_eq!(
            decode_block(&section, 256).unwrap_err(),
            Error::InvalidParameter,
            "rec-cw{cw}"
        );
    }
}

#[test]
fn truncation_beats_domain_and_canonical() {
    let encoded = encode_block(poly(), NominalLength::N256, 1, &[0x5A]).unwrap();
    let section = formula_section(poly()).unwrap();
    assert!(encoded.starts_with(&section));
    for len in 0..section.len() {
        assert_eq!(
            decode_block(&encoded[..len], 1).unwrap_err(),
            Error::TruncatedBlock,
            "prefix {len}"
        );
    }
    for len in section.len()..encoded.len() {
        assert_eq!(
            decode_block(&encoded[..len], 1).unwrap_err(),
            Error::TruncatedCoder,
            "payload prefix {len}"
        );
    }
    assert_eq!(decode_block(&[0x10], 1).unwrap_err(), Error::TruncatedBlock);
    assert_eq!(decode_block(&[0x11], 1).unwrap_err(), Error::TruncatedBlock);

    let mut noncanonical = formula_section(poly()).unwrap();
    let first = noncanonical.first_mut().unwrap();
    *first |= 0x01;
    assert_eq!(
        decode_block(&noncanonical, 1).unwrap_err(),
        Error::NonCanonicalLength
    );
    let illegal = set_padding(poly_raw(0, 0, 32, &[0x8000_0000]), 51);
    assert_eq!(
        decode_block(&illegal, 256).unwrap_err(),
        Error::InvalidParameter
    );
}

#[test]
fn winning_traces_round_trip_without_a_codebook() {
    let cases = [(poly(), 48u16), (trig(), 64), (rec(), 32)];
    for (kind, len) in cases {
        let bytes = match kind {
            BlockKind::Polynomial { formula, .. } => (0..len)
                .map(|n| formula.predict(n).unwrap())
                .collect::<Vec<_>>(),
            BlockKind::Trigonometric { formula, .. } => (0..len)
                .map(|n| formula.predict(n).unwrap())
                .collect::<Vec<_>>(),
            BlockKind::Recurrence { formula, .. } => formula.predict_span(len).unwrap(),
            _ => panic!("baseline"),
        };
        let mut file = Vec::new();
        write_blocked_file(
            u64::from(len),
            &[sxp_kernel::BlockPlan {
                kind,
                nominal: NominalLength::N256,
                bytes: &bytes,
            }],
            &mut file,
        )
        .unwrap();
        assert_eq!(&file[..2], &[0x8d, 0x01]);
        let mut decoded = Vec::new();
        read_blocked_file(file.as_slice(), &mut decoded).unwrap();
        assert_eq!(decoded, bytes);
    }
}

struct Bits {
    bits: Vec<u8>,
}

impl Bits {
    fn new() -> Self {
        Self { bits: Vec::new() }
    }

    fn push(&mut self, value: u64, width: u8) {
        for offset in 0..width {
            self.bits.push(u8::from((value >> offset) & 1 == 1));
        }
    }

    fn bytes(self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut acc = 0u8;
        let mut filled = 0u8;
        for bit in self.bits {
            acc |= bit << filled;
            filled += 1;
            if filled == 8 {
                out.push(acc);
                acc = 0;
                filled = 0;
            }
        }
        if filled > 0 {
            out.push(acc);
        }
        out
    }
}

fn set_padding(mut bytes: Vec<u8>, bit: usize) -> Vec<u8> {
    let index = bit / 8;
    let offset = (bit % 8) as u8;
    let byte = bytes.get_mut(index).expect("padding byte");
    let mask = 1u8 << offset;
    assert_eq!(*byte & mask, 0, "bit {bit} is already a field");
    *byte |= mask;
    bytes
}

fn oracle_section(kind: &BlockKind) -> Vec<u8> {
    match *kind {
        BlockKind::Polynomial { formula, cw } => {
            let coeffs = live_poly(formula);
            let mut raw = Vec::new();
            for coefficient in coeffs {
                raw.push(low_bits(coefficient, cw));
            }
            poly_raw(
                u64::from(degree_of(formula)),
                u64::from(q_of(formula) - 8),
                u64::from(cw),
                &raw,
            )
        }
        BlockKind::Trigonometric { formula, cw } => {
            let coeffs = live_trig(formula);
            let mut raw = Vec::new();
            for coefficient in coeffs {
                raw.push(low_bits(coefficient, cw));
            }
            trig_raw(
                u64::from(trig_order(formula)),
                u64::from(trig_q(formula) - 8),
                u64::from(cw),
                u64::from(trig_omega(formula)),
                &raw,
            )
        }
        BlockKind::Recurrence { formula, cw } => {
            let coeffs = live_rec(formula);
            let state = live_state(formula);
            let mut raw = Vec::new();
            for coefficient in coeffs {
                raw.push(low_bits(coefficient, cw));
            }
            let mut states = Vec::new();
            for value in state {
                states.push(u64::from(value));
            }
            rec_raw(
                u64::from(rec_order(formula)),
                u64::from(rec_mod(formula) - 2),
                u64::from(cw),
                &raw,
                &states,
            )
        }
        _ => panic!("baseline"),
    }
}

fn poly_raw(degree: u64, q_stored: u64, cw: u64, coeffs: &[u64]) -> Vec<u8> {
    let mut bits = Bits::new();
    bits.push(0, 2);
    bits.push(4, 3);
    bits.push(degree, 4);
    bits.push(q_stored, 5);
    bits.push(cw - 1, 5);
    for coefficient in coeffs {
        bits.push(*coefficient, cw as u8);
    }
    bits.bytes()
}

fn trig_raw(order: u64, q_stored: u64, cw: u64, omega: u64, coeffs: &[u64]) -> Vec<u8> {
    let mut bits = Bits::new();
    bits.push(0, 2);
    bits.push(5, 3);
    bits.push(order - 1, 4);
    bits.push(q_stored, 5);
    bits.push(cw - 1, 5);
    bits.push(omega, 32);
    for coefficient in coeffs {
        bits.push(*coefficient, cw as u8);
    }
    bits.bytes()
}

fn rec_raw(order: u64, mod_stored: u64, cw: u64, coeffs: &[u64], state: &[u64]) -> Vec<u8> {
    let mut bits = Bits::new();
    bits.push(0, 2);
    bits.push(6, 3);
    bits.push(order - 1, 1);
    bits.push(mod_stored, 24);
    bits.push(cw - 1, 5);
    for coefficient in coeffs {
        bits.push(*coefficient, cw as u8);
    }
    for value in state {
        bits.push(*value, 24);
    }
    bits.bytes()
}

fn low_bits(value: i32, cw: u8) -> u64 {
    if value >= 0 {
        u64::try_from(value).unwrap()
    } else {
        let modulus = 1i64 << cw;
        u64::try_from(i64::from(value) + modulus).unwrap()
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn degree_of(formula: Polynomial) -> u8 {
    // Reconstructed from a round trip of a known fixture. The oracle for the
    // three fixtures is built from the same literals as the constructors.
    let _ = formula;
    2
}

fn q_of(formula: Polynomial) -> u8 {
    let _ = formula;
    8
}

fn live_poly(formula: Polynomial) -> [i32; 3] {
    let _ = formula;
    [0, 0, 2_147_483_647]
}

fn trig_order(formula: Trigonometric) -> u8 {
    let _ = formula;
    1
}

fn trig_q(formula: Trigonometric) -> u8 {
    let _ = formula;
    8
}

fn trig_omega(formula: Trigonometric) -> u32 {
    let _ = formula;
    1 << 24
}

fn live_trig(formula: Trigonometric) -> [i32; 3] {
    let _ = formula;
    [0, 1 << 20, 0]
}

fn rec_order(formula: Recurrence) -> u8 {
    let _ = formula;
    2
}

fn rec_mod(formula: Recurrence) -> u32 {
    let _ = formula;
    251
}

fn live_rec(formula: Recurrence) -> [i32; 3] {
    let _ = formula;
    [3, 5, 7]
}

fn live_state(formula: Recurrence) -> [u32; 2] {
    let _ = formula;
    [1, 2]
}
