//! Formula Block sections. Spec seed §3.2.
//!
//! Kinds 0–3 stay in `block`. Bits are LSB-first. A stored `cw` that is outside
//! the family cap still determines the section length; the domain check comes
//! after the truncation check.

use crate::block::BlockKind;
use crate::error::Error;
use crate::polynomial::Polynomial;
use crate::recurrence::Recurrence;
use crate::trigonometric::Trigonometric;

/// Padded section length. Baselines are 1 byte, Constant is 2.
///
/// # Errors
///
/// [`Error::InvalidParameter`] when a formula `cw` or coefficient does not fit.
pub fn section_len(kind: BlockKind) -> Result<u64, Error> {
    match kind {
        BlockKind::Constant(_) => Ok(2),
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte => Ok(1),
        BlockKind::Polynomial { .. }
        | BlockKind::Trigonometric { .. }
        | BlockKind::Recurrence { .. } => {
            let bytes = write_section(kind, 0)?;
            u64::try_from(bytes.len()).map_err(|_| Error::InvalidParameter)
        }
    }
}

/// Formula section at length code 0, with zero padding and no payload.
///
/// # Errors
///
/// [`Error::InvalidParameter`] for a baseline kind, or a formula that does not fit.
pub fn formula_section(kind: BlockKind) -> Result<Vec<u8>, Error> {
    match kind {
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte | BlockKind::Constant(_) => {
            Err(Error::InvalidParameter)
        }
        BlockKind::Polynomial { .. }
        | BlockKind::Trigonometric { .. }
        | BlockKind::Recurrence { .. } => write_section(kind, 0),
    }
}

pub(crate) fn append_section(
    out: &mut Vec<u8>,
    length_code: u8,
    kind: BlockKind,
) -> Result<(), Error> {
    let bytes = write_section(kind, length_code)?;
    out.extend_from_slice(&bytes);
    Ok(())
}

pub(crate) fn predictions(kind: BlockKind, actual: usize) -> Result<Vec<u8>, Error> {
    let len = u16::try_from(actual).map_err(|_| Error::InvalidParameter)?;
    if len > 2048 {
        return Err(Error::InvalidParameter);
    }
    match kind {
        BlockKind::Polynomial { formula, .. } => {
            let mut out = Vec::with_capacity(actual);
            for n in 0..len {
                out.push(formula.predict(n)?);
            }
            Ok(out)
        }
        BlockKind::Trigonometric { formula, .. } => {
            let mut out = Vec::with_capacity(actual);
            for n in 0..len {
                out.push(formula.predict(n)?);
            }
            Ok(out)
        }
        BlockKind::Recurrence { formula, .. } => formula.predict_span(len),
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte | BlockKind::Constant(_) => {
            Err(Error::InvalidParameter)
        }
    }
}

/// Parse a formula section from the front of `input`.
///
/// The caller has already rejected a missing byte, length code 3, and kind 7.
/// This returns truncation before a domain error, and a domain error before padding.
pub(crate) fn parse_formula(input: &[u8]) -> Result<(usize, BlockKind), Error> {
    let length_code = take(input, 0, 2)?;
    let _ = length_code;
    let kind = take(input, 2, 3)?;
    match kind {
        4 => parse_polynomial(input),
        5 => parse_trigonometric(input),
        6 => parse_recurrence(input),
        _ => Err(Error::InvalidParameter),
    }
}

pub(crate) fn fits_signed(value: i32, cw: u8) -> bool {
    match cw {
        32 => true,
        1..=31 => {
            let shift = u32::from(cw - 1);
            let max = (1i64 << shift) - 1;
            let min = -max - 1;
            let sample = i64::from(value);
            sample >= min && sample <= max
        }
        _ => false,
    }
}

fn write_section(kind: BlockKind, length_code: u8) -> Result<Vec<u8>, Error> {
    if length_code > 3 {
        return Err(Error::InvalidParameter);
    }
    let mut sink = Sink::new();
    sink.push(u64::from(length_code), 2)?;
    match kind {
        BlockKind::Polynomial { formula, cw } => {
            check_cw(cw, 32)?;
            let coeffs = formula.live_coefficients()?;
            sink.push(4, 3)?;
            sink.push(u64::from(formula.degree()), 4)?;
            sink.push(u64::from(formula.q() - 8), 5)?;
            sink.push(u64::from(cw - 1), 5)?;
            for coefficient in coeffs {
                sink.push(low_bits(*coefficient, cw)?, u32::from(cw))?;
            }
        }
        BlockKind::Trigonometric { formula, cw } => {
            check_cw(cw, 29)?;
            let coeffs = formula.live_coefficients()?;
            sink.push(5, 3)?;
            sink.push(u64::from(formula.order() - 1), 4)?;
            sink.push(u64::from(formula.q() - 8), 5)?;
            sink.push(u64::from(cw - 1), 5)?;
            sink.push(u64::from(formula.omega()), 32)?;
            for coefficient in coeffs {
                sink.push(low_bits(*coefficient, cw)?, u32::from(cw))?;
            }
        }
        BlockKind::Recurrence { formula, cw } => {
            check_cw(cw, 25)?;
            let coeffs = formula.live_coefficients()?;
            let state = formula.live_state()?;
            let stored_mod = u64::from(formula.modulus()) - 2;
            sink.push(6, 3)?;
            sink.push(u64::from(formula.order() - 1), 1)?;
            sink.push(stored_mod, 24)?;
            sink.push(u64::from(cw - 1), 5)?;
            for coefficient in coeffs {
                sink.push(low_bits(*coefficient, cw)?, u32::from(cw))?;
            }
            for value in state {
                sink.push(u64::from(*value), 24)?;
            }
        }
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte | BlockKind::Constant(_) => {
            return Err(Error::InvalidParameter);
        }
    }
    Ok(sink.finish())
}

fn parse_polynomial(input: &[u8]) -> Result<(usize, BlockKind), Error> {
    let degree = take(input, 5, 4)?;
    let q_stored = take(input, 9, 5)?;
    let cw_stored = take(input, 14, 5)?;
    let cw = cw_stored + 1;
    let cw_u8 = u8::try_from(cw).map_err(|_| Error::InvalidParameter)?;
    let count = degree + 1;
    let bits = 5 + 4 + 5 + 5 + count * cw;
    let section = bytes_for(bits)?;
    if input.len() < section {
        return Err(Error::TruncatedBlock);
    }
    if q_stored > 16 {
        return Err(Error::InvalidParameter);
    }
    let mut coeffs = Vec::with_capacity(usize_of(count)?);
    let mut cursor = 19usize;
    for _ in 0..count {
        let raw = take(input, cursor, u8_of(cw)?)?;
        coeffs.push(sign_extend(raw, cw_u8)?);
        cursor = cursor.saturating_add(usize_of(cw)?);
    }
    let degree_u8 = u8::try_from(degree).map_err(|_| Error::InvalidParameter)?;
    let q = u8::try_from(q_stored + 8).map_err(|_| Error::InvalidParameter)?;
    let formula = Polynomial::new(degree_u8, q, &coeffs)?;
    let kind = BlockKind::polynomial(formula, cw_u8)?;
    check_padding(input, cursor, section)?;
    Ok((section, kind))
}

fn parse_trigonometric(input: &[u8]) -> Result<(usize, BlockKind), Error> {
    let order_stored = take(input, 5, 4)?;
    let q_stored = take(input, 9, 5)?;
    let cw_stored = take(input, 14, 5)?;
    let order = order_stored + 1;
    let cw = cw_stored + 1;
    let cw_u8 = u8::try_from(cw).map_err(|_| Error::InvalidParameter)?;
    let count = 2 * order + 1;
    let bits = 5 + 4 + 5 + 5 + 32 + count * cw;
    let section = bytes_for(bits)?;
    if input.len() < section {
        return Err(Error::TruncatedBlock);
    }
    if q_stored > 16 {
        return Err(Error::InvalidParameter);
    }
    if cw > 29 {
        return Err(Error::InvalidParameter);
    }
    let omega = take(input, 19, 32)?;
    let mut coeffs = Vec::with_capacity(usize_of(count)?);
    let mut cursor = 51usize;
    for _ in 0..count {
        let raw = take(input, cursor, cw_u8)?;
        coeffs.push(sign_extend(raw, cw_u8)?);
        cursor = cursor.saturating_add(usize_of(cw)?);
    }
    let order_u8 = u8::try_from(order).map_err(|_| Error::InvalidParameter)?;
    let q = u8::try_from(q_stored + 8).map_err(|_| Error::InvalidParameter)?;
    let omega_u32 = u32::try_from(omega).map_err(|_| Error::InvalidParameter)?;
    let formula = Trigonometric::new(order_u8, q, omega_u32, &coeffs)?;
    let kind = BlockKind::trigonometric(formula, cw_u8)?;
    check_padding(input, cursor, section)?;
    Ok((section, kind))
}

fn parse_recurrence(input: &[u8]) -> Result<(usize, BlockKind), Error> {
    let order_stored = take(input, 5, 1)?;
    let mod_stored = take(input, 6, 24)?;
    let cw_stored = take(input, 30, 5)?;
    let order = order_stored + 1;
    let cw = cw_stored + 1;
    let cw_u8 = u8::try_from(cw).map_err(|_| Error::InvalidParameter)?;
    let coeff_count = order + 1;
    let state_count = order;
    let bits = 5 + 1 + 24 + 5 + coeff_count * cw + state_count * 24;
    let section = bytes_for(bits)?;
    if input.len() < section {
        return Err(Error::TruncatedBlock);
    }
    if mod_stored > (1u64 << 24) - 2 {
        return Err(Error::InvalidParameter);
    }
    if cw > 25 {
        return Err(Error::InvalidParameter);
    }
    let mut coeffs = Vec::with_capacity(usize_of(coeff_count)?);
    let mut cursor = 35usize;
    for _ in 0..coeff_count {
        let raw = take(input, cursor, cw_u8)?;
        coeffs.push(sign_extend(raw, cw_u8)?);
        cursor = cursor.saturating_add(usize_of(cw)?);
    }
    let mut state = Vec::with_capacity(usize_of(state_count)?);
    for _ in 0..state_count {
        let raw = take(input, cursor, 24)?;
        let value = u32::try_from(raw).map_err(|_| Error::InvalidParameter)?;
        state.push(value);
        cursor = cursor.saturating_add(24);
    }
    let order_u8 = u8::try_from(order).map_err(|_| Error::InvalidParameter)?;
    let modulus = u32::try_from(mod_stored + 2).map_err(|_| Error::InvalidParameter)?;
    let formula = Recurrence::new(order_u8, modulus, &coeffs, &state)?;
    let kind = BlockKind::recurrence(formula, cw_u8)?;
    check_padding(input, cursor, section)?;
    Ok((section, kind))
}

fn check_cw(cw: u8, max: u8) -> Result<(), Error> {
    if (1..=max).contains(&cw) {
        Ok(())
    } else {
        Err(Error::InvalidParameter)
    }
}

fn check_padding(input: &[u8], start: usize, section: usize) -> Result<(), Error> {
    let end = section.saturating_mul(8);
    for bit in start..end {
        if take(input, bit, 1)? != 0 {
            return Err(Error::NonZeroPadding);
        }
    }
    Ok(())
}

fn bytes_for(bits: u64) -> Result<usize, Error> {
    let rounded = bits.saturating_add(7) / 8;
    usize::try_from(rounded).map_err(|_| Error::InvalidParameter)
}

fn usize_of(value: u64) -> Result<usize, Error> {
    usize::try_from(value).map_err(|_| Error::InvalidParameter)
}

fn u8_of(value: u64) -> Result<u8, Error> {
    u8::try_from(value).map_err(|_| Error::InvalidParameter)
}

fn sign_extend(raw: u64, cw: u8) -> Result<i32, Error> {
    if !(1..=32).contains(&cw) {
        return Err(Error::InvalidParameter);
    }
    let width = u32::from(cw);
    let mask = (1u64 << width) - 1;
    let low = raw & mask;
    if cw == 32 {
        let bits = u32::try_from(low).map_err(|_| Error::InvalidParameter)?;
        return Ok(i32::from_le_bytes(bits.to_le_bytes()));
    }
    let sign = 1u64 << (width - 1);
    let signed = if low & sign == 0 {
        i64::try_from(low).map_err(|_| Error::InvalidParameter)?
    } else {
        let modulus = 1i64 << width;
        let positive = i64::try_from(low).map_err(|_| Error::InvalidParameter)?;
        positive - modulus
    };
    i32::try_from(signed).map_err(|_| Error::InvalidParameter)
}

fn low_bits(value: i32, cw: u8) -> Result<u64, Error> {
    if !fits_signed(value, cw) {
        return Err(Error::InvalidParameter);
    }
    let width = u32::from(cw);
    if value >= 0 {
        return u64::try_from(value).map_err(|_| Error::InvalidParameter);
    }
    let modulus = 1i64 << width;
    let wrapped = i64::from(value) + modulus;
    u64::try_from(wrapped).map_err(|_| Error::InvalidParameter)
}

fn take(input: &[u8], start: usize, width: u8) -> Result<u64, Error> {
    if width == 0 || width > 32 {
        return Err(Error::InvalidParameter);
    }
    let mut value = 0u64;
    for offset in 0..usize::from(width) {
        let index = start.saturating_add(offset);
        let byte_index = index / 8;
        let bit = u32::try_from(index % 8).map_err(|_| Error::InvalidParameter)?;
        let byte = input
            .get(byte_index)
            .copied()
            .ok_or(Error::TruncatedBlock)?;
        let one = u64::from((byte >> bit) & 1);
        value |= one << u32::try_from(offset).map_err(|_| Error::InvalidParameter)?;
    }
    Ok(value)
}

struct Sink {
    bytes: Vec<u8>,
    acc: u8,
    filled: u8,
}

impl Sink {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            acc: 0,
            filled: 0,
        }
    }

    fn push(&mut self, mut value: u64, mut width: u32) -> Result<(), Error> {
        if width == 0 || width > 32 {
            return Err(Error::InvalidParameter);
        }
        let mask_all = (1u64 << width) - 1;
        if value > mask_all {
            return Err(Error::InvalidParameter);
        }
        while width > 0 {
            let room = u32::from(8 - self.filled);
            let take_bits = room.min(width);
            let mask = (1u64 << take_bits) - 1;
            let shifted = (value & mask) << u32::from(self.filled);
            let bits = u8::try_from(shifted).map_err(|_| Error::InvalidParameter)?;
            self.acc |= bits;
            value >>= take_bits;
            width -= take_bits;
            self.filled += u8::try_from(take_bits).map_err(|_| Error::InvalidParameter)?;
            if self.filled == 8 {
                self.bytes.push(self.acc);
                self.acc = 0;
                self.filled = 0;
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Vec<u8> {
        if self.filled > 0 {
            self.bytes.push(self.acc);
        }
        self.bytes
    }
}
