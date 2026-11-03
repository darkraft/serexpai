//! Integer-only signals. Each parameter tuple is the seed.
//!
//! A signal is two Block-local passes of `n` in `0..2048`. The second half
//! equals the first because every Block restarts.

use sha2::{Digest, Sha256};
use sxp_kernel::{Polynomial, Recurrence, Trigonometric};

const HEX: [u8; 16] = *b"0123456789abcdef";

fn nibble(value: u8) -> Result<char, sxp_kernel::Error> {
    HEX.get(usize::from(value))
        .copied()
        .map(char::from)
        .ok_or(sxp_kernel::Error::InvalidParameter)
}

/// Lowercase SHA-256 of `bytes`.
///
/// # Errors
///
/// [`sxp_kernel::Error::InvalidParameter`] when a nibble is out of range.
pub fn sha256_hex(bytes: &[u8]) -> Result<String, sxp_kernel::Error> {
    let digest = Sha256::digest(bytes);
    let mut out = String::new();
    for byte in digest {
        out.push(nibble(byte >> 4)?);
        out.push(nibble(byte & 0x0f)?);
    }
    Ok(out)
}

fn two_passes(
    predict: impl Fn(u16) -> Result<u8, sxp_kernel::Error>,
) -> Result<Vec<u8>, sxp_kernel::Error> {
    let mut out = Vec::with_capacity(4096);
    for _half in 0..2u8 {
        let mut n = 0u16;
        while n < 2048 {
            out.push(predict(n)?);
            n += 1;
        }
    }
    Ok(out)
}

/// Degree 1, Q 8, coefficients `[0, 65536]`.
///
/// # Errors
///
/// [`sxp_kernel::Error::InvalidParameter`] when the formula or a position is rejected.
pub fn polynomial_signal() -> Result<Vec<u8>, sxp_kernel::Error> {
    let formula = Polynomial::new(1, 8, &[0, 65_536])?;
    two_passes(|n| formula.predict(n))
}

/// Order 1, Q 8, omega `0x0100_0000`, coefficients `[32768, 32000, 0]`.
///
/// # Errors
///
/// [`sxp_kernel::Error::InvalidParameter`] when the formula or a position is rejected.
pub fn trigonometric_signal() -> Result<Vec<u8>, sxp_kernel::Error> {
    let formula = Trigonometric::new(1, 8, 0x0100_0000, &[32_768, 32_000, 0])?;
    two_passes(|n| formula.predict(n))
}

/// Order 2, modulus 256, coefficients `[1, 1, 0]`, state `[0, 1]`.
///
/// Each half is [`Recurrence::predict_span`].
///
/// # Errors
///
/// [`sxp_kernel::Error::InvalidParameter`] when the formula or the span is rejected.
pub fn recurrence_signal() -> Result<Vec<u8>, sxp_kernel::Error> {
    let formula = Recurrence::new(2, 256, &[1, 1, 0], &[0, 1])?;
    let mut out = Vec::with_capacity(4096);
    for _half in 0..2u8 {
        out.extend(formula.predict_span(2048)?);
    }
    Ok(out)
}
