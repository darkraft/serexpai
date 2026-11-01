use crate::error::Error;
use crate::leb128::{LENGTH_LIMIT, decode_leb128, encode_leb128};

/// Parsed file header. `mode` 0 is blocked and `mode` 1 is whole-file raw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: u8,
    pub mode: u8,
    pub n: u64,
}

/// Encoded whole-file raw header. Only the first `len` bytes are significant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderBytes {
    pub len: u8,
    pub bytes: [u8; 7],
}

/// Version 1, mode 1. This function has no mode argument.
pub fn encode_whole_file_raw_header(n: u64) -> Result<HeaderBytes, Error> {
    if n >= LENGTH_LIMIT {
        return Err(Error::LengthOutOfRange);
    }
    let leb = encode_leb128(n)?;
    let mut bytes = [0u8; 7];
    *bytes.get_mut(0).ok_or(Error::Leb128Overflow)? = 0x8D;
    *bytes.get_mut(1).ok_or(Error::Leb128Overflow)? = 0x11;
    let leb_len = usize::from(leb.len);
    for offset in 0..leb_len {
        let src = leb
            .bytes
            .get(offset)
            .copied()
            .ok_or(Error::Leb128Overflow)?;
        let dest_at = 2usize.checked_add(offset).ok_or(Error::Leb128Overflow)?;
        let dest = bytes.get_mut(dest_at).ok_or(Error::Leb128Overflow)?;
        *dest = src;
    }
    let len = u8::try_from(2usize.checked_add(leb_len).ok_or(Error::Leb128Overflow)?)
        .map_err(|_| Error::Leb128Overflow)?;
    Ok(HeaderBytes { len, bytes })
}

/// Check order: truncation of the fixed bytes, magic, version, reserved bits,
/// reserved mode, LEB128, then blocked-empty. Mode 0 with `N > 0` parses successfully.
pub fn decode_header(input: &[u8]) -> Result<(Header, usize), Error> {
    let magic = match input.first() {
        None => return Err(Error::TruncatedHeader),
        Some(byte) => *byte,
    };
    if magic != 0x8D {
        return Err(Error::BadMagic);
    }
    let flags = match input.get(1) {
        None => return Err(Error::TruncatedHeader),
        Some(byte) => *byte,
    };
    let version = flags & 0x0f;
    let mode = (flags >> 4) & 0x03;
    let reserved = flags >> 6;
    if version != 1 {
        return Err(Error::BadVersion);
    }
    if reserved != 0 {
        return Err(Error::ReservedBits);
    }
    if mode >= 2 {
        return Err(Error::ReservedMode);
    }
    let rest = input.get(2..).ok_or(Error::TruncatedHeader)?;
    let (n, leb_len) = decode_leb128(rest)?;
    if mode == 0 && n == 0 {
        return Err(Error::BlockedEmpty);
    }
    let total = 2usize.checked_add(leb_len).ok_or(Error::Leb128Overflow)?;
    Ok((Header { version, mode, n }, total))
}
