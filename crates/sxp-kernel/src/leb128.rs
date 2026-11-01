use crate::error::Error;

/// Largest declared length plus one. Values at or above this are rejected.
pub const LENGTH_LIMIT: u64 = 1 << 35;

/// Unsigned LEB128, at most five bytes, for a value below [`LENGTH_LIMIT`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leb128 {
    pub len: u8,
    pub bytes: [u8; 8],
}

pub fn encode_leb128(n: u64) -> Result<Leb128, Error> {
    if n >= LENGTH_LIMIT {
        return Err(Error::LengthOutOfRange);
    }
    let mut value = n;
    let mut bytes = [0u8; 8];
    let mut len: usize = 0;
    loop {
        let low = u8::try_from(value & 0x7f).map_err(|_| Error::Leb128Overflow)?;
        value >>= 7;
        let stored = if value == 0 { low } else { low | 0x80 };
        let slot = bytes.get_mut(len).ok_or(Error::Leb128Overflow)?;
        *slot = stored;
        len = len.checked_add(1).ok_or(Error::Leb128Overflow)?;
        if value == 0 {
            let len = u8::try_from(len).map_err(|_| Error::Leb128Overflow)?;
            return Ok(Leb128 { len, bytes });
        }
    }
}

/// Decode a minimal unsigned LEB128.
///
/// The fifth byte having bit 7 set is [`Error::Leb128Overflow`]. No sixth byte is read.
/// A finished multi-byte encoding whose last byte is `0x00` is [`Error::NonMinimalLeb128`].
/// A dangling continuation before that is [`Error::TruncatedLeb128`].
pub fn decode_leb128(input: &[u8]) -> Result<(u64, usize), Error> {
    let mut n: u64 = 0;
    let mut shift: u32 = 0;
    let mut index: usize = 0;
    loop {
        let byte = input.get(index).copied().ok_or(Error::TruncatedLeb128)?;
        let position = index;
        index = index.checked_add(1).ok_or(Error::Leb128Overflow)?;
        if position == 4 && (byte & 0x80) != 0 {
            return Err(Error::Leb128Overflow);
        }
        let piece = u64::from(byte & 0x7f);
        let shifted = piece.checked_shl(shift).ok_or(Error::Leb128Overflow)?;
        n = n.checked_add(shifted).ok_or(Error::Leb128Overflow)?;
        if byte & 0x80 == 0 {
            if index > 1 && byte == 0 {
                return Err(Error::NonMinimalLeb128);
            }
            if n >= LENGTH_LIMIT {
                return Err(Error::Leb128Overflow);
            }
            return Ok((n, index));
        }
        shift = shift.checked_add(7).ok_or(Error::Leb128Overflow)?;
    }
}
