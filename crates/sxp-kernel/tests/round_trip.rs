mod support;

use std::io::{self, Read, Write};

use sxp_kernel::{
    Error, LENGTH_LIMIT, crc32_iso_hdlc, decode_header, encode_whole_file_raw_header,
    read_whole_file_raw, write_whole_file_raw,
};

use support::{decode_hex, load, named_error, vectors};

#[test]
fn index_vectors_match_their_rules() {
    let rows = vectors();
    assert!(rows.len() >= 24);
    for row in rows {
        let bytes = load(row.path);
        match row.kind {
            "reject" => {
                let mut input = bytes;
                let mut sink = Vec::new();
                let err = read_whole_file_raw(&mut input, &mut sink).expect_err(row.id);
                assert_eq!(err, named_error(row.error.expect("error")));
                assert_eq!(err.to_string(), err.name());
            }
            "stream" => {
                let mut input = bytes;
                let mut sink = Vec::new();
                read_whole_file_raw(&mut input, &mut sink).unwrap();
                let decoded = decode_hex(row.decoded.expect("decoded"));
                assert_eq!(sink, decoded);
                let mut encoded = Vec::new();
                write_whole_file_raw(decoded.len() as u64, &mut decoded.as_slice(), &mut encoded)
                    .unwrap();
                assert_eq!(encoded, bytes);
                assert_eq!(
                    encoded.len() as u64,
                    decoded.len() as u64 + 6 + leb_len(decoded.len() as u64)
                );
            }
            "payload" => {
                let mut encoded = Vec::new();
                write_whole_file_raw(bytes.len() as u64, &mut &*bytes, &mut encoded).unwrap();
                let mut sink = Vec::new();
                read_whole_file_raw(&mut encoded.as_slice(), &mut sink).unwrap();
                assert_eq!(sink, bytes);
                assert_eq!(
                    encoded.len() as u64,
                    bytes.len() as u64 + 6 + leb_len(bytes.len() as u64)
                );
            }
            "crc" => {
                let input = decode_hex(row.input.expect("input"));
                assert_eq!(crc32_iso_hdlc(&input).to_le_bytes().as_slice(), bytes);
                assert_eq!(crc32_iso_hdlc(b""), 0);
            }
            other => panic!("kind {other}"),
        }
    }
}

#[test]
fn gzip_fixture_is_the_pinned_blob() {
    let bytes = load("payload-gzip.bin");
    let expected = decode_hex(
        "hex:1f8b08000000000002032b4e2d4aad2848ccd42d2ec92faad435d433d44dcc294a4d4ca9d44dcecf2d284a2d2e4e4de1020050f1f47f26000000",
    );
    assert_eq!(bytes, expected.as_slice());
    assert_eq!(bytes.len(), 58);
}

#[test]
fn every_other_version_is_rejected() {
    for version in 0..=15u8 {
        if version == 1 {
            continue;
        }
        let flags = version | 0x10;
        let err = decode_header(&[0x8D, flags]).expect_err("version");
        assert_eq!(err, Error::BadVersion);
    }
}

#[test]
fn empty_stream_bit_flips_have_named_errors() {
    let base = *include_bytes!("../../../spec/vectors/whole-file-raw-empty.bin");
    assert_eq!(base.len(), 7);
    for byte in 0..7 {
        for bit in 0..8u8 {
            let mut flipped = base;
            flipped[byte] ^= 1 << bit;
            let mut sink = Vec::new();
            let err = read_whole_file_raw(&mut flipped.as_slice(), &mut sink).expect_err("flip");
            assert_eq!(err, expect_flip(byte, bit), "byte {byte} bit {bit}");
        }
    }
}

fn expect_flip(byte: usize, bit: u8) -> Error {
    match (byte, bit) {
        (0, _) => Error::BadMagic,
        (1, 0..=3) => Error::BadVersion,
        (1, 4) => Error::BlockedEmpty,
        (1, 5) => Error::ReservedMode,
        (1, 6..=7) => Error::ReservedBits,
        (2, 0..=2) => Error::TruncatedTrailer,
        (2, 3..=6) => Error::TruncatedPayload,
        (2, 7) => Error::NonMinimalLeb128,
        (3..=6, _) => Error::CrcMismatch,
        _ => panic!("flip outside the empty stream"),
    }
}

#[test]
fn overflow_does_not_read_a_sixth_length_byte() {
    for path in [
        "reject/leb-overflow.bin",
        "reject/leb-range-before-nonminimal.bin",
    ] {
        let bytes = load(path);
        let mut input = Budget {
            data: bytes,
            pos: 0,
            max: 7,
            past: false,
        };
        let mut sink = Vec::new();
        let err = read_whole_file_raw(&mut input, &mut sink).expect_err(path);
        assert_eq!(err, Error::Leb128Overflow);
        assert!(!input.past, "{path} read past the fifth length byte");
    }
}

#[test]
fn blocked_mode_with_payload_writes_nothing() {
    let bytes = [0x8D, 0x01, 0x01, 0x00, 0x8D, 0xEF, 0x02, 0xD2];
    let mut sink = Vec::new();
    let err = read_whole_file_raw(&mut bytes.as_slice(), &mut sink).expect_err("blocked");
    assert_eq!(err, Error::BlockedModeUnsupported);
    assert!(sink.is_empty());
}

#[test]
fn writer_rejects_short_long_and_oversized_lengths() {
    let mut out = Vec::new();
    let err = write_whole_file_raw(LENGTH_LIMIT, &mut io::empty(), &mut out).expect_err("limit");
    assert_eq!(err, Error::LengthOutOfRange);
    assert!(out.is_empty());

    let mut out = Vec::new();
    let err = write_whole_file_raw(2, &mut &[1u8][..], &mut out).expect_err("short");
    assert_eq!(err, Error::TruncatedPayload);

    let mut out = Vec::new();
    let err = write_whole_file_raw(1, &mut &[1u8, 2][..], &mut out).expect_err("long");
    assert_eq!(err, Error::ExcessPayload);
}

#[test]
fn width_boundaries_and_streamed_two_megabytes() {
    let samples = [
        0u64,
        127,
        128,
        16_383,
        16_384,
        (1 << 21) - 1,
        1 << 21,
        (1 << 28) - 1,
        1 << 28,
        LENGTH_LIMIT - 1,
    ];
    for n in samples {
        let header = encode_whole_file_raw_header(n).unwrap();
        let len = usize::from(header.len);
        let bytes = &header.bytes[..len];
        let (decoded, used) = decode_header(bytes).unwrap();
        assert_eq!(used, len);
        assert_eq!(decoded.n, n);
        assert_eq!(decoded.mode, 1);
        assert_eq!(decoded.version, 1);
        assert_eq!(len as u64, 2 + leb_len(n));
        if n == LENGTH_LIMIT - 1 {
            assert_eq!(2 + leb_len(n) + 4, 11);
        }
    }
    for n in [0u64, 1, 127, 128, 1 << 21, (1 << 21) + 1] {
        let header = encode_whole_file_raw_header(n).unwrap();
        let crc = crc_of_zeros(n);
        let mut written = FrameCheck {
            header: header.bytes,
            header_len: usize::from(header.len),
            n,
            crc: crc.to_le_bytes(),
            at: 0,
        };
        write_whole_file_raw(n, Zeros { left: n }, &mut written).unwrap();
        assert_eq!(written.at, u64::from(header.len) + n + 4);

        let mut generated = Generated {
            header: header.bytes,
            header_len: usize::from(header.len),
            header_pos: 0,
            zeros_left: n,
            crc: crc.to_le_bytes(),
            crc_pos: 0,
        };
        let mut sink = ZeroSink { n: 0 };
        read_whole_file_raw(&mut generated, &mut sink).unwrap();
        assert_eq!(sink.n, n);
    }
}

fn leb_len(n: u64) -> u64 {
    let header = encode_whole_file_raw_header(n).unwrap();
    u64::from(header.len) - 2
}

struct Zeros {
    left: u64,
}

impl Read for Zeros {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.left == 0 || buf.is_empty() {
            return Ok(0);
        }
        let room = u64::try_from(buf.len()).unwrap_or(self.left);
        let n = usize::try_from(self.left.min(room)).unwrap_or(buf.len());
        for slot in buf.iter_mut().take(n) {
            *slot = 0;
        }
        self.left -= u64::try_from(n).unwrap_or(0);
        Ok(n)
    }
}

struct Budget<'a> {
    data: &'a [u8],
    pos: usize,
    max: usize,
    past: bool,
}

impl Read for Budget<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.max || buf.is_empty() {
            self.past = self.pos >= self.max;
            return Ok(0);
        }
        let n = buf
            .len()
            .min(self.max - self.pos)
            .min(self.data.len() - self.pos);
        buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

fn crc_of_zeros(n: u64) -> u32 {
    let mut crc = sxp_kernel::Crc32::new();
    let zeros = [0u8; 8192];
    let mut left = n;
    while left > 0 {
        let chunk = usize::try_from(left.min(8192)).unwrap_or(8192);
        crc.update(&zeros[..chunk]);
        left -= u64::try_from(chunk).unwrap_or(0);
    }
    crc.finish()
}

struct FrameCheck {
    header: [u8; 7],
    header_len: usize,
    n: u64,
    crc: [u8; 4],
    at: u64,
}

impl Write for FrameCheck {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &byte in buf {
            let header_len = u64::try_from(self.header_len).unwrap_or(0);
            if self.at < header_len {
                let index = usize::try_from(self.at).unwrap_or(0);
                assert_eq!(byte, self.header[index]);
            } else if self.at < header_len + self.n {
                assert_eq!(byte, 0);
            } else {
                let index = usize::try_from(self.at - header_len - self.n).unwrap_or(4);
                assert_eq!(byte, self.crc[index]);
            }
            self.at += 1;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct Generated {
    header: [u8; 7],
    header_len: usize,
    header_pos: usize,
    zeros_left: u64,
    crc: [u8; 4],
    crc_pos: usize,
}

impl Read for Generated {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.header_pos < self.header_len {
            buf[0] = self.header[self.header_pos];
            self.header_pos += 1;
            return Ok(1);
        }
        if self.zeros_left > 0 {
            let room = u64::try_from(buf.len()).unwrap_or(self.zeros_left);
            let n = usize::try_from(self.zeros_left.min(room)).unwrap_or(buf.len());
            for slot in buf.iter_mut().take(n) {
                *slot = 0;
            }
            self.zeros_left -= u64::try_from(n).unwrap_or(0);
            return Ok(n);
        }
        if self.crc_pos < 4 {
            buf[0] = self.crc[self.crc_pos];
            self.crc_pos += 1;
            return Ok(1);
        }
        Ok(0)
    }
}

struct ZeroSink {
    n: u64,
}

impl Write for ZeroSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for &byte in buf {
            assert_eq!(byte, 0);
        }
        self.n += u64::try_from(buf.len()).unwrap_or(0);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
