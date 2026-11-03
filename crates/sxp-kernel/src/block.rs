//! Block framing. Spec seed §3, §4.4, and §4.5.
//!
//! Kind numbers are the seed §3.1 table. That table is still tagged proposed.
//! Story 1.8 uses it as published and does not retag the seed.
//!
//! Wire payload of one coded Block is at most 32772 bytes. Before each
//! decision `range >= 2^24` and `freq0` is in `1..=4095`, so the post-decision
//! range is at least 4096. Renormalisation then shifts at most twice
//! (`4096 << 16 = 2^28`). A carry flush emits bytes those shifts deferred, so
//! emitted bytes are at most the `shift_low` count. A 2048-byte Block has
//! 16384 decisions, plus 5 termination shifts, minus the dropped leading byte:
//! `16384 * 2 + 5 - 1 = 32772` payload bytes. The largest formula section is
//! the order-16 trigonometric section, 1008 bits, 126 bytes. The encoded Block
//! is at most `126 + 32772 = 32898` bytes.

use std::io::{ErrorKind, Read, Write};

use crate::crc32::Crc32;
use crate::error::Error;
use crate::formula_wire;
use crate::header::decode_header;
use crate::leb128::{LENGTH_LIMIT, encode_leb128};
use crate::model::{ModelConfig, Order0Model};
use crate::polynomial::Polynomial;
use crate::range_coder::{RangeDecoder, RangeEncoder};
use crate::recurrence::Recurrence;
use crate::residual::{direct_symbol, reconstruct, residual_symbol};
use crate::trigonometric::Trigonometric;

/// Encoded Block length the writer refuses to emit.
pub const MAX_ENCODED_BLOCK: usize = 32_898;

/// Decoder read-ahead, 36 KiB. [`MAX_ENCODED_BLOCK`] plus a 4-byte trailer is
/// 32902. This constant is that ceiling rounded up to 36 KiB.
pub const BLOCK_WINDOW: usize = 36_864;

/// Nominal Block length. Code 3 is not representable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NominalLength {
    N256,
    N1024,
    N2048,
}

impl NominalLength {
    /// Wire length code, bits 0–1 of the section.
    pub const fn code(self) -> u8 {
        match self {
            Self::N256 => 0,
            Self::N1024 => 1,
            Self::N2048 => 2,
        }
    }

    /// Nominal length in bytes.
    pub const fn bytes(self) -> u64 {
        match self {
            Self::N256 => 256,
            Self::N1024 => 1024,
            Self::N2048 => 2048,
        }
    }

    fn from_wire(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::N256),
            1 => Some(Self::N1024),
            2 => Some(Self::N2048),
            _ => None,
        }
    }
}

/// One Block. Formula variants carry the evaluator and the stored coefficient width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Raw,
    Direct,
    PreviousByte,
    Constant(u8),
    Polynomial { formula: Polynomial, cw: u8 },
    Trigonometric { formula: Trigonometric, cw: u8 },
    Recurrence { formula: Recurrence, cw: u8 },
}

impl BlockKind {
    /// Polynomial kind. `cw` is the stored width, not a production choice.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `cw` is outside 1..=32 or a live
    /// coefficient does not fit in that signed width.
    pub fn polynomial(formula: Polynomial, cw: u8) -> Result<Self, Error> {
        check_width(cw, 32, formula.live_coefficients()?)?;
        Ok(Self::Polynomial { formula, cw })
    }

    /// Trigonometric kind. `cw` is the stored width, at most 29.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `cw` is outside 1..=29 or a live
    /// coefficient does not fit in that signed width.
    pub fn trigonometric(formula: Trigonometric, cw: u8) -> Result<Self, Error> {
        check_width(cw, 29, formula.live_coefficients()?)?;
        Ok(Self::Trigonometric { formula, cw })
    }

    /// Recurrence kind. `cw` is the stored width, at most 25.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidParameter`] when `cw` is outside 1..=25 or a live
    /// coefficient does not fit in that signed width.
    pub fn recurrence(formula: Recurrence, cw: u8) -> Result<Self, Error> {
        check_width(cw, 25, formula.live_coefficients()?)?;
        Ok(Self::Recurrence { formula, cw })
    }

    fn wire(self) -> u8 {
        match self {
            Self::Raw => 0,
            Self::Direct => 1,
            Self::PreviousByte => 2,
            Self::Constant(_) => 3,
            Self::Polynomial { .. } => 4,
            Self::Trigonometric { .. } => 5,
            Self::Recurrence { .. } => 6,
        }
    }
}

fn check_width(cw: u8, max: u8, coefficients: &[i32]) -> Result<(), Error> {
    if !(1..=max).contains(&cw) {
        return Err(Error::InvalidParameter);
    }
    for coefficient in coefficients {
        if !formula_wire::fits_signed(*coefficient, cw) {
            return Err(Error::InvalidParameter);
        }
    }
    Ok(())
}

/// One decoded Block. `consumed` counts section and payload bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockView {
    pub kind: BlockKind,
    pub nominal: NominalLength,
    pub bytes: Vec<u8>,
    pub consumed: usize,
}

/// Caller-supplied Block. This story does not choose the kind or the length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockPlan<'a> {
    pub kind: BlockKind,
    pub nominal: NominalLength,
    pub bytes: &'a [u8],
}

/// `nominal <= remaining`, or, when `remaining <= 2048`, the smallest nominal
/// that is `>= remaining`. `remaining == 0` is never canonical.
pub fn is_canonical(nominal: NominalLength, remaining: u64) -> bool {
    if remaining == 0 {
        return false;
    }
    let n = nominal.bytes();
    if n <= remaining {
        return true;
    }
    if remaining > 2048 {
        return false;
    }
    let smallest = if remaining <= 256 {
        256
    } else if remaining <= 1024 {
        1024
    } else {
        2048
    };
    n == smallest
}

/// `min(nominal, remaining)`.
pub fn actual_len(nominal: NominalLength, remaining: u64) -> u64 {
    nominal.bytes().min(remaining)
}

/// Encode one baseline Block.
///
/// `remaining` is the file bytes still unplaced, including `bytes`.
/// `bytes.len()` must equal [`actual_len`]. A Block longer than
/// [`MAX_ENCODED_BLOCK`] is [`Error::BlockTooLarge`] and is not returned.
/// Raw copies `bytes`. Direct calls [`direct_symbol`]. Previous-byte and
/// Constant call [`residual_symbol`]. Each coded Block uses a new strawman
/// model and a new range encoder.
pub fn encode_block(
    kind: BlockKind,
    nominal: NominalLength,
    remaining: u64,
    bytes: &[u8],
) -> Result<Vec<u8>, Error> {
    if !is_canonical(nominal, remaining) {
        return Err(Error::NonCanonicalLength);
    }
    let actual =
        usize::try_from(actual_len(nominal, remaining)).map_err(|_| Error::BlockLengthMismatch)?;
    if bytes.len() != actual {
        return Err(Error::BlockLengthMismatch);
    }
    let mut out = Vec::new();
    match kind {
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte | BlockKind::Constant(_) => {
            push_section(&mut out, nominal, kind)?;
        }
        BlockKind::Polynomial { .. }
        | BlockKind::Trigonometric { .. }
        | BlockKind::Recurrence { .. } => {
            formula_wire::append_section(&mut out, nominal.code(), kind)?;
        }
    }
    match kind {
        BlockKind::Raw => out.extend_from_slice(bytes),
        BlockKind::Direct => append_coded(&mut out, bytes.iter().copied().map(direct_symbol))?,
        BlockKind::PreviousByte => {
            let mut yhat = 0u8;
            let mut symbols = Vec::with_capacity(bytes.len());
            for &y in bytes {
                symbols.push(residual_symbol(y, yhat));
                yhat = y;
            }
            append_coded(&mut out, symbols.into_iter())?;
        }
        BlockKind::Constant(c) => append_coded(
            &mut out,
            bytes.iter().copied().map(|y| residual_symbol(y, c)),
        )?,
        BlockKind::Polynomial { .. }
        | BlockKind::Trigonometric { .. }
        | BlockKind::Recurrence { .. } => {
            let predictions = formula_wire::predictions(kind, actual)?;
            append_predicted(&mut out, bytes, &predictions)?;
        }
    }
    if out.len() > MAX_ENCODED_BLOCK {
        return Err(Error::BlockTooLarge);
    }
    Ok(out)
}

/// Decode one baseline Block from the front of `input`.
///
/// Bytes after this Block stay unread. `consumed` is the section length plus
/// [`RangeDecoder::bytes_consumed`](crate::RangeDecoder::bytes_consumed) for a
/// coded Block. Check order, first match wins:
///
/// 1. missing first byte → [`Error::TruncatedBlock`]
/// 2. length code 3 → [`Error::ReservedLength`]
/// 3. kind 7 → [`Error::ReservedKind`]
/// 4. formula section short → [`Error::TruncatedBlock`]
/// 5. formula field outside its domain → [`Error::InvalidParameter`]
/// 6. padding bit set → [`Error::NonZeroPadding`]
/// 7. nominal not canonical → [`Error::NonCanonicalLength`]
/// 8. raw payload short, or a baseline section short → [`Error::TruncatedBlock`]
/// 9. coder errors pass through
pub fn decode_block(input: &[u8], remaining: u64) -> Result<BlockView, Error> {
    let first = match input.first().copied() {
        Some(byte) => byte,
        None => return Err(Error::TruncatedBlock),
    };
    let length_code = first & 0x03;
    let kind_code = (first >> 2) & 0x07;
    if length_code == 3 {
        return Err(Error::ReservedLength);
    }
    if kind_code == 7 {
        return Err(Error::ReservedKind);
    }
    let nominal = NominalLength::from_wire(length_code).ok_or(Error::ReservedLength)?;
    if kind_code >= 4 {
        let (section_len, kind) = formula_wire::parse_formula(input)?;
        if !is_canonical(nominal, remaining) {
            return Err(Error::NonCanonicalLength);
        }
        let actual =
            usize::try_from(actual_len(nominal, remaining)).map_err(|_| Error::TruncatedBlock)?;
        let payload = input.get(section_len..).ok_or(Error::TruncatedBlock)?;
        let predictions = formula_wire::predictions(kind, actual)?;
        let (bytes, payload_used) = decode_predicted(payload, &predictions)?;
        let consumed = section_len
            .checked_add(payload_used)
            .ok_or(Error::TruncatedBlock)?;
        return Ok(BlockView {
            kind,
            nominal,
            bytes,
            consumed,
        });
    }
    let (section_len, kind) = parse_baseline(input, first, kind_code)?;
    if !is_canonical(nominal, remaining) {
        return Err(Error::NonCanonicalLength);
    }
    let actual =
        usize::try_from(actual_len(nominal, remaining)).map_err(|_| Error::TruncatedBlock)?;
    let payload = input.get(section_len..).ok_or(Error::TruncatedBlock)?;
    let (bytes, payload_used) = match kind {
        BlockKind::Raw => {
            let raw = payload.get(..actual).ok_or(Error::TruncatedBlock)?;
            (raw.to_vec(), actual)
        }
        BlockKind::Direct => decode_coded(payload, actual, Prediction::Direct)?,
        BlockKind::PreviousByte => decode_coded(payload, actual, Prediction::Previous)?,
        BlockKind::Constant(c) => decode_coded(payload, actual, Prediction::Constant(c))?,
        BlockKind::Polynomial { .. }
        | BlockKind::Trigonometric { .. }
        | BlockKind::Recurrence { .. } => return Err(Error::InvalidParameter),
    };
    let consumed = section_len
        .checked_add(payload_used)
        .ok_or(Error::TruncatedBlock)?;
    Ok(BlockView {
        kind,
        nominal,
        bytes,
        consumed,
    })
}

/// Write a mode-0 file. Structural plan errors leave `output` empty.
///
/// `n == 0` is [`Error::BlockedEmpty`]. `n >= 2^35` is
/// [`Error::LengthOutOfRange`]. Each plan is checked before the header is
/// written. [`Error::BlockTooLarge`] is raised before that Block is written;
/// an earlier Block may already be in `output`.
pub fn write_blocked_file(
    n: u64,
    plans: &[BlockPlan<'_>],
    mut output: impl Write,
) -> Result<(), Error> {
    if n >= LENGTH_LIMIT {
        return Err(Error::LengthOutOfRange);
    }
    if n == 0 {
        return Err(Error::BlockedEmpty);
    }
    let mut remaining = n;
    for plan in plans {
        if !is_canonical(plan.nominal, remaining) {
            return Err(Error::NonCanonicalLength);
        }
        let actual = actual_len(plan.nominal, remaining);
        let actual_us = usize::try_from(actual).map_err(|_| Error::BlockLengthMismatch)?;
        if plan.bytes.len() != actual_us {
            return Err(Error::BlockLengthMismatch);
        }
        remaining = remaining
            .checked_sub(actual)
            .ok_or(Error::BlockLengthMismatch)?;
    }
    if remaining != 0 {
        return Err(Error::BlockLengthMismatch);
    }
    let header = blocked_header(n)?;
    write_all(&mut output, &header)?;
    let mut crc = Crc32::new();
    remaining = n;
    for plan in plans {
        let encoded = encode_block(plan.kind, plan.nominal, remaining, plan.bytes)?;
        write_all(&mut output, &encoded)?;
        crc.update(plan.bytes);
        let actual = actual_len(plan.nominal, remaining);
        remaining = remaining
            .checked_sub(actual)
            .ok_or(Error::BlockLengthMismatch)?;
    }
    write_all(&mut output, &crc.finish().to_le_bytes())
}

/// Read a mode-0 file one Block at a time.
///
/// Mode 1 is [`Error::UnexpectedMode`] and writes nothing. The read-ahead
/// buffer is [`BLOCK_WINDOW`] bytes and does not grow with `N`. A CRC mismatch
/// may already have written decoded bytes, matching [`crate::read_whole_file_raw`].
pub fn read_blocked_file(mut input: impl Read, mut output: impl Write) -> Result<(), Error> {
    let header = read_stream_header(&mut input)?;
    if header.mode == 1 {
        return Err(Error::UnexpectedMode);
    }
    if header.mode != 0 {
        return Err(Error::ReservedMode);
    }
    let mut window = Window::new();
    let mut crc = Crc32::new();
    let mut remaining = header.n;
    while remaining > 0 {
        let view = next_block(&mut window, &mut input, remaining)?;
        let n = u64::try_from(view.bytes.len()).map_err(|_| Error::TruncatedBlock)?;
        crc.update(&view.bytes);
        write_all(&mut output, &view.bytes)?;
        remaining = remaining.checked_sub(n).ok_or(Error::TruncatedBlock)?;
    }
    let mut trailer = [0u8; 4];
    read_following(
        &mut window,
        &mut input,
        &mut trailer,
        Error::TruncatedTrailer,
    )?;
    if u32::from_le_bytes(trailer) != crc.finish() {
        return Err(Error::CrcMismatch);
    }
    if !window.is_empty() {
        return Err(Error::TrailingBytes);
    }
    if read_byte(&mut input)?.is_some() {
        return Err(Error::TrailingBytes);
    }
    Ok(())
}

enum Prediction {
    Direct,
    Previous,
    Constant(u8),
}

fn parse_baseline(input: &[u8], first: u8, kind_code: u8) -> Result<(usize, BlockKind), Error> {
    if kind_code == 3 {
        let second = match input.get(1).copied() {
            Some(byte) => byte,
            None => return Err(Error::TruncatedBlock),
        };
        if second >> 5 != 0 {
            return Err(Error::NonZeroPadding);
        }
        let c = ((first >> 5) & 0x07) | ((second & 0x1f) << 3);
        return Ok((2, BlockKind::Constant(c)));
    }
    if first >> 5 != 0 {
        return Err(Error::NonZeroPadding);
    }
    let kind = match kind_code {
        0 => BlockKind::Raw,
        1 => BlockKind::Direct,
        2 => BlockKind::PreviousByte,
        _ => return Err(Error::UnsupportedKind),
    };
    Ok((1, kind))
}

fn push_section(out: &mut Vec<u8>, nominal: NominalLength, kind: BlockKind) -> Result<(), Error> {
    let kind_code = kind.wire();
    let mut first = nominal.code() | (kind_code << 2);
    match kind {
        BlockKind::Constant(c) => {
            first |= (c & 0x07) << 5;
            out.push(first);
            out.push((c >> 3) & 0x1f);
        }
        BlockKind::Raw | BlockKind::Direct | BlockKind::PreviousByte => out.push(first),
        BlockKind::Polynomial { .. }
        | BlockKind::Trigonometric { .. }
        | BlockKind::Recurrence { .. } => return Err(Error::InvalidParameter),
    }
    Ok(())
}

fn append_predicted(out: &mut Vec<u8>, bytes: &[u8], predictions: &[u8]) -> Result<(), Error> {
    if predictions.len() != bytes.len() {
        return Err(Error::BlockLengthMismatch);
    }
    let mut symbols = Vec::with_capacity(bytes.len());
    for (index, yhat) in predictions.iter().copied().enumerate() {
        let y = bytes
            .get(index)
            .copied()
            .ok_or(Error::BlockLengthMismatch)?;
        symbols.push(residual_symbol(y, yhat));
    }
    append_coded(out, symbols.into_iter())
}

fn decode_predicted(payload: &[u8], predictions: &[u8]) -> Result<(Vec<u8>, usize), Error> {
    let mut decoder = RangeDecoder::new(payload)?;
    let mut model = Order0Model::new(ModelConfig::strawman()?);
    let mut bytes = Vec::with_capacity(predictions.len());
    for yhat in predictions {
        let (symbol, _) = model.decode_symbol(&mut decoder)?;
        bytes.push(reconstruct(*yhat, symbol));
    }
    decoder.finish()?;
    Ok((bytes, decoder.bytes_consumed()))
}

fn append_coded(out: &mut Vec<u8>, symbols: impl Iterator<Item = u8>) -> Result<(), Error> {
    let mut model = Order0Model::new(ModelConfig::strawman()?);
    let mut encoder = RangeEncoder::new();
    for symbol in symbols {
        model.encode_symbol(&mut encoder, symbol)?;
    }
    out.extend_from_slice(&encoder.finish());
    Ok(())
}

fn decode_coded(
    payload: &[u8],
    actual: usize,
    prediction: Prediction,
) -> Result<(Vec<u8>, usize), Error> {
    let mut decoder = RangeDecoder::new(payload)?;
    let mut model = Order0Model::new(ModelConfig::strawman()?);
    let mut bytes = Vec::with_capacity(actual);
    let mut yhat = 0u8;
    for _ in 0..actual {
        let (symbol, _) = model.decode_symbol(&mut decoder)?;
        let y = match prediction {
            Prediction::Direct => symbol,
            Prediction::Previous => {
                let y = reconstruct(yhat, symbol);
                yhat = y;
                y
            }
            Prediction::Constant(c) => reconstruct(c, symbol),
        };
        bytes.push(y);
    }
    decoder.finish()?;
    Ok((bytes, decoder.bytes_consumed()))
}

fn blocked_header(n: u64) -> Result<Vec<u8>, Error> {
    let leb = encode_leb128(n)?;
    let leb_len = usize::from(leb.len);
    let mut out = Vec::with_capacity(leb_len.saturating_add(2));
    out.push(0x8d);
    out.push(0x01);
    let leb_bytes = leb.bytes.get(..leb_len).ok_or(Error::Leb128Overflow)?;
    out.extend_from_slice(leb_bytes);
    Ok(out)
}

struct Window {
    buf: Vec<u8>,
}

impl Window {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(BLOCK_WINDOW),
        }
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    fn available(&self) -> &[u8] {
        self.buf.as_slice()
    }

    fn consume(&mut self, n: usize) -> Result<(), Error> {
        let len = self.buf.len();
        if n > len {
            return Err(Error::TruncatedBlock);
        }
        let rest = len - n;
        if n > 0 && rest > 0 {
            for offset in 0..rest {
                let from = n.checked_add(offset).ok_or(Error::TruncatedBlock)?;
                let byte = self.buf.get(from).copied().ok_or(Error::TruncatedBlock)?;
                let slot = self.buf.get_mut(offset).ok_or(Error::TruncatedBlock)?;
                *slot = byte;
            }
        }
        self.buf.truncate(rest);
        Ok(())
    }

    fn pull(&mut self, input: &mut impl Read) -> Result<usize, Error> {
        if self.buf.len() >= BLOCK_WINDOW {
            return Ok(0);
        }
        let mut tmp = [0u8; 4096];
        let room = BLOCK_WINDOW - self.buf.len();
        let want = room.min(tmp.len());
        let dest = tmp.get_mut(..want).ok_or(Error::TruncatedBlock)?;
        let n = read_some(input, dest)?;
        let got = tmp.get(..n).ok_or(Error::TruncatedBlock)?;
        self.buf.extend_from_slice(got);
        Ok(n)
    }
}

fn next_block(
    window: &mut Window,
    input: &mut impl Read,
    remaining: u64,
) -> Result<BlockView, Error> {
    loop {
        match decode_block(window.available(), remaining) {
            Ok(view) => {
                window.consume(view.consumed)?;
                return Ok(view);
            }
            Err(Error::TruncatedBlock) | Err(Error::TruncatedCoder) => {
                let grew = window.pull(input)?;
                if grew == 0 {
                    return decode_block(window.available(), remaining);
                }
            }
            Err(err) => return Err(err),
        }
    }
}

fn read_following(
    window: &mut Window,
    input: &mut impl Read,
    dest: &mut [u8],
    on_eof: Error,
) -> Result<(), Error> {
    let mut filled = 0usize;
    while filled < dest.len() {
        if window.is_empty() {
            let n = window.pull(input)?;
            if n == 0 {
                return Err(on_eof);
            }
        }
        let need = dest.len() - filled;
        let take = need.min(window.buf.len());
        for offset in 0..take {
            let byte = window
                .buf
                .get(offset)
                .copied()
                .ok_or(Error::TruncatedBlock)?;
            let at = filled.checked_add(offset).ok_or(Error::TruncatedBlock)?;
            let slot = dest.get_mut(at).ok_or(Error::TruncatedBlock)?;
            *slot = byte;
        }
        window.consume(take)?;
        filled = filled.checked_add(take).ok_or(Error::TruncatedBlock)?;
    }
    Ok(())
}

fn read_stream_header(input: &mut impl Read) -> Result<crate::header::Header, Error> {
    let mut buf = [0u8; 7];
    let mut len = 0usize;
    loop {
        let prefix = buf.get(..len).ok_or(Error::Leb128Overflow)?;
        match decode_header(prefix) {
            Ok((header, used)) if used == len => return Ok(header),
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

fn read_byte(input: &mut impl Read) -> Result<Option<u8>, Error> {
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

fn read_some(input: &mut impl Read, dest: &mut [u8]) -> Result<usize, Error> {
    if dest.is_empty() {
        return Ok(0);
    }
    loop {
        match input.read(dest) {
            Ok(n) => return Ok(n),
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(Error::Io(err.kind())),
        }
    }
}

fn write_all(output: &mut impl Write, mut bytes: &[u8]) -> Result<(), Error> {
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
