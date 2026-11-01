//! Trigonometric evaluator. Spec seed §4.2.
//!
//! Coefficients are `c0`, then `a_k`, `b_k` for `k = 1..=order`. Phase is
//! `n * omega` modulo 2^32, not `u = n << 5`.

use crate::Error;

const SINE_LUT: &[u8] = include_bytes!("../../../spec/sine_lut.bin");
const COEFF_LIMIT: i32 = 1 << 28;

/// Validated trigonometric Formula.
///
/// Coefficients are `c0`, then `a_1`, `b_1`, … in that order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trigonometric {
    order: u8,
    q: u8,
    omega: u32,
    coefficients: [i32; 33],
}

impl Trigonometric {
    /// Build a trigonometric Formula from its mathematical order, Q, and omega.
    ///
    /// A negative order does not fit in `u8`. Order `0` fits and is rejected.
    /// `omega >= 2^32` does not fit in `u32`; `0` and `u32::MAX` are legal.
    /// `±(2^28 - 1)` are legal. `±2^28` fit in `i32` and are rejected.
    /// `i32::MIN` and `i32::MAX` are rejected because they are outside
    /// `|c| < 2^28`. `q` is the mathematical Q in `8..=24`, not the stored
    /// `Q - 8` field. Stored `cw <= 29` is the story 1.10 wire cap; this
    /// constructor does not take `cw`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when the order, Q, coefficient count, or
    /// any coefficient is outside the domain. Out-of-range values are rejected,
    /// not clamped.
    pub fn new(order: u8, q: u8, omega: u32, coefficients: &[i32]) -> Result<Self, Error> {
        if !(1..=16).contains(&order) || !(8..=24).contains(&q) {
            return Err(Error::InvalidParameter);
        }
        let count = 2 * usize::from(order) + 1;
        if coefficients.len() != count {
            return Err(Error::InvalidParameter);
        }
        let mut stored = [0i32; 33];
        for index in 0..count {
            let coefficient = coefficients
                .get(index)
                .copied()
                .ok_or(Error::InvalidParameter)?;
            if coefficient <= -COEFF_LIMIT || coefficient >= COEFF_LIMIT {
                return Err(Error::InvalidParameter);
            }
            let slot = stored.get_mut(index).ok_or(Error::InvalidParameter)?;
            *slot = coefficient;
        }
        Ok(Self {
            order,
            q,
            omega,
            coefficients: stored,
        })
    }

    /// Rounded value `(S + 2^(Q + 14)) >> (Q + 15)`, before [`i64::rem_euclid`].
    ///
    /// Halfway cases, including negative ones, round toward `+∞`.
    /// A negative position does not fit in `u16`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `n > 2047`.
    pub fn value(&self, n: u16) -> Result<i64, Error> {
        if n > 2047 {
            return Err(Error::InvalidParameter);
        }
        let theta = (u64::from(n) * u64::from(self.omega)) as u32;
        let mut sum = self.coefficient(0)? << 15;
        let mut k = 1u8;
        while k <= self.order {
            let p = (u64::from(k) * u64::from(theta)) as u32;
            let p_cos = (u64::from(p) + (1u64 << 30)) as u32;
            let a_index = usize::from(k) * 2 - 1;
            let b_index = usize::from(k) * 2;
            sum += self.coefficient(a_index)? * sin_lut(p)?
                + self.coefficient(b_index)? * sin_lut(p_cos)?;
            k += 1;
        }
        let shift = u32::from(self.q) + 15;
        let half = 1i64 << (shift - 1);
        Ok((sum + half) >> shift)
    }

    fn coefficient(&self, index: usize) -> Result<i64, Error> {
        self.coefficients
            .get(index)
            .copied()
            .map(i64::from)
            .ok_or(Error::InvalidParameter)
    }

    /// Prediction byte, `rem_euclid(value, 256)`.
    ///
    /// A negative position does not fit in `u16`. There is no Block-length
    /// parameter. Phase is `(n * omega) mod 2^32`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `n > 2047`.
    pub fn predict(&self, n: u16) -> Result<u8, Error> {
        Ok(self.value(n)?.rem_euclid(256) as u8)
    }
}

/// Accept only the committed 2050-byte sine table.
///
/// # Errors
///
/// [`Error::InvalidParameter`] when `bytes` differ in length or content.
pub fn check_sine_lut(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() != SINE_LUT.len() || bytes != SINE_LUT {
        return Err(Error::InvalidParameter);
    }
    Ok(())
}

fn sin_lut(p: u32) -> Result<i64, Error> {
    let index = usize::try_from(p >> 22).map_err(|_| Error::InvalidParameter)?;
    let fraction = i64::from((p >> 6) & 0xFFFF);
    let v0 = i64::from(entry(index)?);
    let v1 = i64::from(entry(index + 1)?);
    Ok(v0 + (((v1 - v0) * fraction + (1i64 << 15)) >> 16))
}

fn entry(index: usize) -> Result<i16, Error> {
    let start = index.checked_mul(2).ok_or(Error::InvalidParameter)?;
    let end = start.checked_add(2).ok_or(Error::InvalidParameter)?;
    let bytes = SINE_LUT.get(start..end).ok_or(Error::InvalidParameter)?;
    let array: [u8; 2] = bytes.try_into().map_err(|_| Error::InvalidParameter)?;
    Ok(i16::from_le_bytes(array))
}
