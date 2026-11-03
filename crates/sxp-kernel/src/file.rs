//! Mode dispatch for a whole file. The body readers own framing and CRC.

use std::io::{Cursor, Read, Write};

use crate::block::read_blocked_file;
use crate::error::Error;
use crate::raw::{read_header_prefix, read_whole_file_raw};

/// Decode a mode-0 or mode-1 file.
///
/// The header is parsed once into a 7-byte stack buffer and replayed into the
/// existing reader. A header error writes nothing. The body is not buffered.
pub fn read_file(mut input: impl Read, output: impl Write) -> Result<(), Error> {
    let (buf, len, header) = read_header_prefix(&mut input)?;
    let prefix = buf.get(..len).ok_or(Error::Leb128Overflow)?;
    let chained = Cursor::new(prefix).chain(input);
    match header.mode {
        0 => read_blocked_file(chained, output),
        1 => read_whole_file_raw(chained, output),
        _ => Err(Error::ReservedMode),
    }
}
