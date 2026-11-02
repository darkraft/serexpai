//! Binary range coder from bitstream-spec-seed.md §6.1.
//!
//! The caller supplies each bit and `freq0`. This module does not update a
//! probability model, and it does not frame a Block. The total is 4096.

use crate::Error;

/// Trace view of the encoder. Not a wire field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncoderState {
    pub low: u64,
    pub range: u32,
    pub cache: u8,
    pub cache_size: u64,
}

/// Trace view of the decoder. Not a wire field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecoderState {
    pub code: u32,
    pub range: u32,
    pub bytes_consumed: usize,
}

/// Carry-propagating binary range encoder.
///
/// `low` is an unsigned value of at least 33 bits. `range << 8` runs only
/// while `range < 2^24`. After validation, `bound` is at most `0xFFEF_F001`
/// and fits in `u32`. `low` stays below `2^33`.
#[derive(Debug)]
pub struct RangeEncoder {
    low: u64,
    range: u32,
    cache: u8,
    cache_size: u64,
    out: Vec<u8>,
    dropped_leading: bool,
}

impl RangeEncoder {
    /// `low = 0`, `range = 0xFFFFFFFF`, `cache = 0`, `cache_size = 1`.
    pub fn new() -> Self {
        Self {
            low: 0,
            range: 0xFFFF_FFFF,
            cache: 0,
            cache_size: 1,
            out: Vec::new(),
            dropped_leading: false,
        }
    }

    pub fn state(&self) -> EncoderState {
        EncoderState {
            low: self.low,
            range: self.range,
            cache: self.cache,
            cache_size: self.cache_size,
        }
    }

    /// Encode one decision.
    ///
    /// `freq0` of `0` and `4096` fit in `u16` and are rejected. A value above
    /// `u16::MAX` does not fit. Bit `2` fits in `u8` and is rejected. The
    /// total is 4096 and is not an argument. A rejected call does not change
    /// [`Self::state`]. After validation, `bound` is at most `0xFFEF_F001`
    /// and fits in `u32`. `range << 8` runs only while `range < 2^24`. `low`
    /// stays below `2^33`.
    pub fn encode(&mut self, bit: u8, freq0: u16) -> Result<(), Error> {
        if bit > 1 || !(1..=4095).contains(&freq0) {
            return Err(Error::InvalidParameter);
        }
        let product = u64::from(self.range >> 12) * u64::from(freq0);
        let bound = u32::try_from(product).map_err(|_| Error::InvalidParameter)?;
        if bit == 0 {
            self.range = bound;
        } else {
            self.low += u64::from(bound);
            self.range -= bound;
        }
        while self.range < 0x0100_0000 {
            self.range <<= 8;
            self.shift_low();
        }
        Ok(())
    }

    /// Five `shift_low` calls. The first emitted byte has already been dropped.
    pub fn finish(mut self) -> Vec<u8> {
        for _ in 0..5 {
            self.shift_low();
        }
        self.out
    }

    fn shift_low(&mut self) {
        if self.low < 0xFF00_0000 || self.low >= 0x1_0000_0000 {
            let carry = self.low >> 32;
            self.emit((u64::from(self.cache) + carry) & 0xFF);
            let pending = (0xFFu64 + carry) & 0xFF;
            for _ in 1..self.cache_size {
                self.emit(pending);
            }
            self.cache = ((self.low >> 24) & 0xFF) as u8;
            self.cache_size = 0;
        }
        self.cache_size += 1;
        self.low = (self.low & 0x00FF_FFFF) << 8;
    }

    /// Drops only the first byte this sink is asked to store.
    fn emit(&mut self, byte: u64) {
        let byte = (byte & 0xFF) as u8;
        if !self.dropped_leading {
            self.dropped_leading = true;
            return;
        }
        self.out.push(byte);
    }
}

impl Default for RangeEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Carry-propagating binary range decoder.
///
/// Refill runs inside [`Self::decode`], before the function returns. `code << 8`
/// runs only while `code < range < 2^24`.
#[derive(Debug)]
pub struct RangeDecoder<'a> {
    code: u32,
    range: u32,
    input: &'a [u8],
    pos: usize,
}

impl<'a> RangeDecoder<'a> {
    /// Reads four bytes, most significant byte first.
    ///
    /// Fewer than four bytes is [`Error::TruncatedCoder`], including three
    /// `0xFF` bytes. `code == u32::MAX` is [`Error::SaturatedRangeCode`].
    pub fn new(input: &'a [u8]) -> Result<Self, Error> {
        let Some(b0) = input.first().copied() else {
            return Err(Error::TruncatedCoder);
        };
        let Some(b1) = input.get(1).copied() else {
            return Err(Error::TruncatedCoder);
        };
        let Some(b2) = input.get(2).copied() else {
            return Err(Error::TruncatedCoder);
        };
        let Some(b3) = input.get(3).copied() else {
            return Err(Error::TruncatedCoder);
        };
        let code =
            (u32::from(b0) << 24) | (u32::from(b1) << 16) | (u32::from(b2) << 8) | u32::from(b3);
        if code == u32::MAX {
            return Err(Error::SaturatedRangeCode);
        }
        Ok(Self {
            code,
            range: 0xFFFF_FFFF,
            input,
            pos: 4,
        })
    }

    pub fn state(&self) -> DecoderState {
        DecoderState {
            code: self.code,
            range: self.range,
            bytes_consumed: self.pos,
        }
    }

    pub fn bytes_consumed(&self) -> usize {
        self.pos
    }

    /// Decide one bit from `freq0` and refill until `range >= 2^24`.
    ///
    /// A missing refill byte is [`Error::TruncatedCoder`]. `range` is left below
    /// `2^24`, so a later `decode` fails the same way and [`Self::finish`]
    /// returns [`Error::TruncatedCoder`]. `freq0` outside `1..=4095` is
    /// [`Error::InvalidParameter`] and leaves [`Self::state`] unchanged.
    pub fn decode(&mut self, freq0: u16) -> Result<u8, Error> {
        if !(1..=4095).contains(&freq0) {
            return Err(Error::InvalidParameter);
        }
        let product = u64::from(self.range >> 12) * u64::from(freq0);
        let bound = u32::try_from(product).map_err(|_| Error::InvalidParameter)?;
        let bit = if self.code < bound {
            self.range = bound;
            0
        } else {
            self.code -= bound;
            self.range -= bound;
            1
        };
        while self.range < 0x0100_0000 {
            let Some(byte) = self.input.get(self.pos).copied() else {
                return Err(Error::TruncatedCoder);
            };
            self.range <<= 8;
            self.pos += 1;
            self.code = (self.code << 8) | u32::from(byte);
        }
        Ok(bit)
    }

    /// `code` must be 0. Leftover input is not an error here.
    ///
    /// A refill that ran out of bytes leaves `range < 2^24`. `finish` then
    /// returns [`Error::TruncatedCoder`], and so does a later [`Self::decode`].
    pub fn finish(&self) -> Result<(), Error> {
        if self.range < 0x0100_0000 {
            return Err(Error::TruncatedCoder);
        }
        if self.code != 0 {
            return Err(Error::NonzeroRangeCode);
        }
        Ok(())
    }
}

/// Encode a decision list. Each pair is `(bit, freq0)`.
pub fn encode_schedule(decisions: &[(u8, u16)]) -> Result<Vec<u8>, Error> {
    let mut encoder = RangeEncoder::new();
    for &(bit, freq0) in decisions {
        encoder.encode(bit, freq0)?;
    }
    Ok(encoder.finish())
}

/// Byte length of [`encode_schedule`] for the same decisions.
pub fn exact_size(decisions: &[(u8, u16)]) -> Result<usize, Error> {
    encode_schedule(decisions).map(|bytes| bytes.len())
}

/// Decode one frame. Bytes after the frame are [`Error::TrailingBytes`].
pub fn decode_schedule(input: &[u8], freq0s: &[u16]) -> Result<Vec<u8>, Error> {
    let mut decoder = RangeDecoder::new(input)?;
    let mut bits = Vec::new();
    for &freq0 in freq0s {
        bits.push(decoder.decode(freq0)?);
    }
    decoder.finish()?;
    if decoder.bytes_consumed() < input.len() {
        return Err(Error::TrailingBytes);
    }
    Ok(bits)
}
