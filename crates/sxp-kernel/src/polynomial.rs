//! Polynomial evaluator. Spec seed §4.1.
//!
//! Coefficients are lowest power first. Position uses `u = n << 5` for every
//! Block length.

use crate::Error;

/// Validated polynomial Formula.
///
/// Coefficients are lowest power first: index 0 is the constant term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Polynomial {
    degree: u8,
    q: u8,
    coefficients: [i32; 16],
}

impl Polynomial {
    /// Build a polynomial from its mathematical degree and Q.
    ///
    /// `+2^31` does not fit in `i32` and cannot be passed. A negative degree
    /// does not fit in `u8`. `i32::MAX` (`2^31 - 1`) is legal. `i32::MIN`
    /// (`-2^31`) is representable and is rejected. `q` is the mathematical Q
    /// in `8..=24`, not the stored `Q - 8` field.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when the degree, Q, coefficient count, or
    /// any coefficient is outside the domain. Out-of-range values are rejected,
    /// not clamped.
    pub fn new(degree: u8, q: u8, coefficients: &[i32]) -> Result<Self, Error> {
        if degree > 15 || !(8..=24).contains(&q) {
            return Err(Error::InvalidParameter);
        }
        let count = usize::from(degree) + 1;
        if coefficients.len() != count {
            return Err(Error::InvalidParameter);
        }
        let mut stored = [0i32; 16];
        for index in 0..count {
            let coefficient = coefficients
                .get(index)
                .copied()
                .ok_or(Error::InvalidParameter)?;
            if coefficient == i32::MIN {
                return Err(Error::InvalidParameter);
            }
            let slot = stored.get_mut(index).ok_or(Error::InvalidParameter)?;
            *slot = coefficient;
        }
        Ok(Self {
            degree,
            q,
            coefficients: stored,
        })
    }

    /// Rounded Horner value, before [`i64::rem_euclid`].
    ///
    /// A negative position does not fit in `u16`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `n > 2047`.
    pub fn value(&self, n: u16) -> Result<i64, Error> {
        if n > 2047 {
            return Err(Error::InvalidParameter);
        }
        let u = i64::from(n) << 5;
        let mut k = self.degree;
        let mut acc = self.coefficient(k)?;
        while k > 0 {
            k -= 1;
            acc = ((acc * u) >> 16) + self.coefficient(k)?;
        }
        let half = 1i64 << u32::from(self.q - 1);
        Ok((acc + half) >> u32::from(self.q))
    }

    fn coefficient(&self, index: u8) -> Result<i64, Error> {
        self.coefficients
            .get(usize::from(index))
            .copied()
            .map(i64::from)
            .ok_or(Error::InvalidParameter)
    }

    /// Prediction byte, `rem_euclid(value, 256)`.
    ///
    /// A negative position does not fit in `u16`. There is no Block-length
    /// parameter; `u` is always `n << 5`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `n > 2047`.
    pub fn predict(&self, n: u16) -> Result<u8, Error> {
        Ok(self.value(n)?.rem_euclid(256) as u8)
    }

    pub(crate) fn degree(&self) -> u8 {
        self.degree
    }

    pub(crate) fn q(&self) -> u8 {
        self.q
    }

    /// Lowest power first, `degree + 1` coefficients. The unused tail is omitted.
    pub(crate) fn live_coefficients(&self) -> Result<&[i32], Error> {
        let count = usize::from(self.degree).saturating_add(1);
        self.coefficients
            .get(..count)
            .ok_or(Error::InvalidParameter)
    }
}
