//! Modular residual. Spec seed §4.5.
//!
//! `δ = ((y − ŷ + 128) mod 256) − 128`, symbol `= δ mod 256`, and
//! `y = (ŷ + δ) mod 256`. Modulo is mathematical, via [`i16::rem_euclid`].

/// `δ = ((y − prediction + 128) mod 256) − 128`, in `−128..=127`.
pub fn signed_residual(y: u8, prediction: u8) -> i16 {
    let widened = i16::from(y) - i16::from(prediction) + 128;
    widened.rem_euclid(256) - 128
}

/// Coded symbol, `δ mod 256`. Encoder entry point for a predicted byte.
pub fn residual_symbol(y: u8, prediction: u8) -> u8 {
    signed_residual(y, prediction).rem_euclid(256) as u8
}

/// `y = (prediction + δ) mod 256`, with `δ` recovered from `symbol`.
///
/// Decoder entry point. `symbol < 128` is `δ`; otherwise `δ = symbol − 256`.
pub fn reconstruct(prediction: u8, symbol: u8) -> u8 {
    let delta = if symbol < 128 {
        i16::from(symbol)
    } else {
        i16::from(symbol) - 256
    };
    (i16::from(prediction) + delta).rem_euclid(256) as u8
}

/// Direct entropy coding. The symbol is the original byte.
///
/// Does not call [`signed_residual`], [`residual_symbol`], or [`reconstruct`].
/// Story 1.8's Direct `encode_block` path is what must call this.
pub fn direct_symbol(y: u8) -> u8 {
    y
}
