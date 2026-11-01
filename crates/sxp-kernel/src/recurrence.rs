//! Recurrence evaluator. Spec seed §4.3.
//!
//! Order 1 reads `a`, `b`, and `s0`. Order 2 also reads `c` and `s1`.
//! Position is the iteration count `n`, not `u = n << 5`.

use crate::Error;

const MOD_LIMIT: u32 = 1 << 24;
const COEFF_LIMIT: i32 = 1 << 24;

/// Validated recurrence Formula.
///
/// Coefficients are `a`, `b`, then `c`. State is `s0`, then `s1`.
/// Order 1 leaves `c` and `s1` at 0 and does not read them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recurrence {
    order: u8,
    m_mod: u32,
    coefficients: [i32; 3],
    state: [u32; 2],
}

impl Recurrence {
    /// Build a recurrence Formula from its order, modulus, coefficients, and state.
    ///
    /// A negative order does not fit in `u8`. Order `0` and order `3` fit and
    /// are rejected. A negative modulus does not fit in `u32`. A modulus above
    /// `u32::MAX` does not fit in `u32`. `0` and `1` fit and are rejected. `2`
    /// and `2^24` are legal. `2^24 + 1` and `u32::MAX` fit and are rejected. A
    /// negative state does not fit in `u32`. `s_i = m_mod - 1` is legal.
    /// `s_i >= m_mod` is representable in `u32` and is rejected.
    /// `±(2^24 - 1)` are legal. `±2^24` fit in `i32` and are rejected.
    /// `i32::MIN` and `i32::MAX` are rejected because they are outside
    /// `|c| < 2^24`, not because of a special `i32` rule. Stored `cw <= 25`
    /// is the story 1.10 wire cap; this
    /// constructor does not take `cw`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when the order, modulus, coefficient count,
    /// state count, any coefficient, or any state is outside the domain.
    /// Out-of-range values are rejected, not clamped. A longer slice is not
    /// accepted with its tail ignored.
    pub fn new(order: u8, m_mod: u32, coefficients: &[i32], state: &[u32]) -> Result<Self, Error> {
        if !(1..=2).contains(&order) || !(2..=MOD_LIMIT).contains(&m_mod) {
            return Err(Error::InvalidParameter);
        }
        let coefficient_count = usize::from(order) + 1;
        let state_count = usize::from(order);
        if coefficients.len() != coefficient_count || state.len() != state_count {
            return Err(Error::InvalidParameter);
        }
        let mut stored = [0i32; 3];
        for index in 0..coefficient_count {
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
        let mut stored_state = [0u32; 2];
        for index in 0..state_count {
            let value = state.get(index).copied().ok_or(Error::InvalidParameter)?;
            if value >= m_mod {
                return Err(Error::InvalidParameter);
            }
            let slot = stored_state.get_mut(index).ok_or(Error::InvalidParameter)?;
            *slot = value;
        }
        Ok(Self {
            order,
            m_mod,
            coefficients: stored,
            state: stored_state,
        })
    }

    /// `x[n]`, in `0..m_mod`, after [`i64::rem_euclid`] by the modulus.
    ///
    /// `predict` reduces this with `rem_euclid(value, 256)`.
    /// A negative position does not fit in `u16`. There is no Block-length
    /// parameter. Each call restarts at `s0` and `s1`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `n > 2047`.
    pub fn value(&self, n: u16) -> Result<i64, Error> {
        if n > 2047 {
            return Err(Error::InvalidParameter);
        }
        let modulus = i64::from(self.m_mod);
        match self.order {
            1 => {
                let a = self.coefficient(0)?;
                let b = self.coefficient(1)?;
                let mut x = self.initial(0)?;
                for _ in 0..n {
                    x = (a * x + b).rem_euclid(modulus);
                }
                Ok(x)
            }
            2 => {
                if n == 0 {
                    return self.initial(0);
                }
                let a = self.coefficient(0)?;
                let b = self.coefficient(1)?;
                let c = self.coefficient(2)?;
                let mut prev = self.initial(0)?;
                let mut curr = self.initial(1)?;
                for _ in 1..n {
                    let next = (a * curr + b * prev + c).rem_euclid(modulus);
                    prev = curr;
                    curr = next;
                }
                Ok(curr)
            }
            _ => Err(Error::InvalidParameter),
        }
    }

    fn coefficient(&self, index: usize) -> Result<i64, Error> {
        self.coefficients
            .get(index)
            .copied()
            .map(i64::from)
            .ok_or(Error::InvalidParameter)
    }

    fn initial(&self, index: usize) -> Result<i64, Error> {
        self.state
            .get(index)
            .copied()
            .map(i64::from)
            .ok_or(Error::InvalidParameter)
    }

    /// Prediction byte, `rem_euclid(value, 256)`.
    ///
    /// A negative position does not fit in `u16`. There is no Block-length
    /// parameter. Each call restarts at `s0` and `s1`.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `n > 2047`.
    pub fn predict(&self, n: u16) -> Result<u8, Error> {
        Ok(self.value(n)?.rem_euclid(256) as u8)
    }
}
