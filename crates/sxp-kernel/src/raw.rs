use std::io::{ErrorKind, Read, Write};

use crate::crc32::Crc32;
use crate::error::Error;
use crate::header::{Header, decode_header, encode_whole_file_raw_header};
use crate::leb128::LENGTH_LIMIT;

const CHUNK: usize = 8192;

fn read_byte<R: Read>(input: &mut R) -> Result<Option<u8>, Error> {
    let mut buf = [0u8; 1];
    loop {
        match input.read(&mut buf) {
            Ok(0) => return Ok(None),
            Ok(_) => {
                let byte = buf.first().copied().ok_or(Error::Io(ErrorKind::Other))?;
                return Ok(Some(byte));
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(Error::Io(err.kind())),
        }
    }
}

fn write_all<W: Write>(output: &mut W, mut bytes: &[u8]) -> Result<(), Error> {
    while !bytes.is_empty() {
        match output.write(bytes) {
            Ok(0) => return Err(Error::Io(ErrorKind::WriteZero)),
            Ok(n) => {
                bytes = bytes.get(n..).ok_or(Error::Leb128Overflow)?;
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(Error::Io(err.kind())),
        }
    }
    Ok(())
}

fn read_header<R: Read>(input: &mut R) -> Result<Header, Error> {
    let (_buf, _len, header) = read_header_prefix(input)?;
    Ok(header)
}

/// Bytes consumed for a complete header, and that header.
///
/// `used != len` is [`Error::Leb128Overflow`]. Only [`Error::TruncatedHeader`]
/// and [`Error::TruncatedLeb128`] pull another byte, and only while fewer than
/// 7 bytes are buffered.
pub(crate) fn read_header_prefix<R: Read>(
    input: &mut R,
) -> Result<([u8; 7], usize, Header), Error> {
    let mut buf = [0u8; 7];
    let mut len = 0usize;
    loop {
        let prefix = buf.get(..len).ok_or(Error::Leb128Overflow)?;
        match decode_header(prefix) {
            Ok((header, used)) if used == len => return Ok((buf, len, header)),
            Ok(_) => return Err(Error::Leb128Overflow),
            Err(Error::TruncatedHeader) | Err(Error::TruncatedLeb128) if len < 7 => {}
            Err(err) => return Err(err),
        }
        match read_byte(input)? {
            None => {
                let prefix = buf.get(..len).ok_or(Error::Leb128Overflow)?;
                return match decode_header(prefix) {
                    Err(err) => Err(err),
                    Ok(_) => Err(Error::TruncatedHeader),
                };
            }
            Some(byte) => {
                let slot = buf.get_mut(len).ok_or(Error::Leb128Overflow)?;
                *slot = byte;
                len = len.checked_add(1).ok_or(Error::Leb128Overflow)?;
            }
        }
    }
}

fn copy_exact<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
    mut remaining: u64,
    crc: &mut Crc32,
) -> Result<(), Error> {
    let mut buf = [0u8; CHUNK];
    while remaining > 0 {
        let want = remaining.min(u64::try_from(CHUNK).map_err(|_| Error::Leb128Overflow)?);
        let want_us = usize::try_from(want).map_err(|_| Error::Leb128Overflow)?;
        let mut filled = 0usize;
        while filled < want_us {
            let dest = buf.get_mut(filled..want_us).ok_or(Error::Leb128Overflow)?;
            match input.read(dest) {
                Ok(0) => return Err(Error::TruncatedPayload),
                Ok(n) => {
                    filled = filled.checked_add(n).ok_or(Error::Leb128Overflow)?;
                }
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(Error::Io(err.kind())),
            }
        }
        let chunk = buf.get(..filled).ok_or(Error::Leb128Overflow)?;
        crc.update(chunk);
        write_all(output, chunk)?;
        let filled_u64 = u64::try_from(filled).map_err(|_| Error::Leb128Overflow)?;
        remaining = remaining
            .checked_sub(filled_u64)
            .ok_or(Error::Leb128Overflow)?;
    }
    Ok(())
}

fn read_exact_into<R: Read>(input: &mut R, dest: &mut [u8], on_eof: Error) -> Result<(), Error> {
    let mut filled = 0usize;
    while filled < dest.len() {
        let slot = dest.get_mut(filled..).ok_or(Error::Leb128Overflow)?;
        match input.read(slot) {
            Ok(0) => return Err(on_eof),
            Ok(n) => {
                filled = filled.checked_add(n).ok_or(Error::Leb128Overflow)?;
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(Error::Io(err.kind())),
        }
    }
    Ok(())
}

/// Write a whole-file raw stream. `len` is authoritative.
///
/// On `Err`, `output` may already hold a prefix. Rollback belongs to story 1.11.
pub fn write_whole_file_raw(
    len: u64,
    mut payload: impl Read,
    mut output: impl Write,
) -> Result<(), Error> {
    if len >= LENGTH_LIMIT {
        return Err(Error::LengthOutOfRange);
    }
    let header = encode_whole_file_raw_header(len)?;
    let header_len = usize::from(header.len);
    let header_bytes = header
        .bytes
        .get(..header_len)
        .ok_or(Error::Leb128Overflow)?;
    write_all(&mut output, header_bytes)?;
    let mut crc = Crc32::new();
    copy_exact(&mut payload, &mut output, len, &mut crc)?;
    if read_byte(&mut payload)?.is_some() {
        return Err(Error::ExcessPayload);
    }
    write_all(&mut output, &crc.finish().to_le_bytes())
}

/// Read a whole-file raw stream.
///
/// Payload is copied to `output` before the CRC is checked. Header failures and
/// [`Error::BlockedModeUnsupported`] write nothing.
pub fn read_whole_file_raw(mut input: impl Read, mut output: impl Write) -> Result<(), Error> {
    let header = read_header(&mut input)?;
    if header.mode == 0 {
        return Err(Error::BlockedModeUnsupported);
    }
    let mut crc = Crc32::new();
    copy_exact(&mut input, &mut output, header.n, &mut crc)?;
    let mut trailer = [0u8; 4];
    read_exact_into(&mut input, &mut trailer, Error::TruncatedTrailer)?;
    if u32::from_le_bytes(trailer) != crc.finish() {
        return Err(Error::CrcMismatch);
    }
    if read_byte(&mut input)?.is_some() {
        return Err(Error::TrailingBytes);
    }
    Ok(())
}
