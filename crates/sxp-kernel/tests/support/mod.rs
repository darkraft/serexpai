#![allow(dead_code)]

use sxp_kernel::Error;

pub struct Vector {
    pub id: &'static str,
    pub path: &'static str,
    pub kind: &'static str,
    pub error: Option<&'static str>,
    pub decoded: Option<&'static str>,
    pub input: Option<&'static str>,
}

#[derive(Default)]
struct Pending {
    id: Option<&'static str>,
    path: Option<&'static str>,
    kind: Option<&'static str>,
    error: Option<&'static str>,
    decoded: Option<&'static str>,
    input: Option<&'static str>,
}

pub fn vectors() -> Vec<Vector> {
    let text = include_str!("../../../../spec/vectors/index.toml");
    let mut out = Vec::new();
    let mut pending = Pending::default();
    let mut started = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[vector]]" {
            flush(&mut out, &mut pending, started);
            started = true;
            continue;
        }
        if !started {
            panic!("key before [[vector]]");
        }
        let Some((key, raw_value)) = line.split_once('=') else {
            panic!("expected key = value");
        };
        let key = key.trim();
        let value = quoted(raw_value.trim());
        match key {
            "id" => assign(&mut pending.id, value, "id"),
            "path" => assign(&mut pending.path, value, "path"),
            "kind" => assign(&mut pending.kind, value, "kind"),
            "error" => assign(&mut pending.error, value, "error"),
            "decoded" => assign(&mut pending.decoded, value, "decoded"),
            "input" => assign(&mut pending.input, value, "input"),
            "note" => {}
            other => panic!("unknown index key {other}"),
        }
    }
    flush(&mut out, &mut pending, started);
    assert_eq!(out.len(), 24, "index.toml vector count");
    let mut ids = Vec::new();
    for row in &out {
        assert!(!ids.contains(&row.id), "duplicate id {}", row.id);
        ids.push(row.id);
    }
    out
}

fn flush(out: &mut Vec<Vector>, pending: &mut Pending, started: bool) {
    if !started {
        return;
    }
    let record = std::mem::take(pending);
    out.push(Vector {
        id: nonempty(record.id, "id"),
        path: nonempty(record.path, "path"),
        kind: nonempty(record.kind, "kind"),
        error: record.error,
        decoded: record.decoded,
        input: record.input,
    });
}

fn quoted(raw: &'static str) -> &'static str {
    let Some(inner) = raw.strip_prefix('"') else {
        panic!("value must be quoted");
    };
    let Some(inner) = inner.strip_suffix('"') else {
        panic!("value must be quoted");
    };
    if inner.contains('"') || inner.contains('\\') {
        panic!("unsupported quote syntax");
    }
    inner
}

fn nonempty<'a>(slot: Option<&'a str>, key: &str) -> &'a str {
    match slot {
        Some(value) if !value.trim().is_empty() => value,
        _ => panic!("missing {key}"),
    }
}

fn assign(slot: &mut Option<&'static str>, value: &'static str, key: &str) {
    if slot.is_some() {
        panic!("duplicate {key}");
    }
    *slot = Some(value);
}

pub fn load(path: &str) -> &'static [u8] {
    match path {
        "whole-file-raw-empty.bin" => {
            include_bytes!("../../../../spec/vectors/whole-file-raw-empty.bin")
        }
        "whole-file-raw-123456789.bin" => {
            include_bytes!("../../../../spec/vectors/whole-file-raw-123456789.bin")
        }
        "crc32-123456789.bin" => include_bytes!("../../../../spec/vectors/crc32-123456789.bin"),
        "payload-a5.bin" => include_bytes!("../../../../spec/vectors/payload-a5.bin"),
        "payload-gzip.bin" => include_bytes!("../../../../spec/vectors/payload-gzip.bin"),
        "reject/empty.bin" => include_bytes!("../../../../spec/vectors/reject/empty.bin"),
        "reject/truncated-header.bin" => {
            include_bytes!("../../../../spec/vectors/reject/truncated-header.bin")
        }
        "reject/bad-magic.bin" => include_bytes!("../../../../spec/vectors/reject/bad-magic.bin"),
        "reject/version-0.bin" => include_bytes!("../../../../spec/vectors/reject/version-0.bin"),
        "reject/version-2.bin" => include_bytes!("../../../../spec/vectors/reject/version-2.bin"),
        "reject/version-15.bin" => include_bytes!("../../../../spec/vectors/reject/version-15.bin"),
        "reject/reserved-bits.bin" => {
            include_bytes!("../../../../spec/vectors/reject/reserved-bits.bin")
        }
        "reject/mode-2.bin" => include_bytes!("../../../../spec/vectors/reject/mode-2.bin"),
        "reject/mode-3.bin" => include_bytes!("../../../../spec/vectors/reject/mode-3.bin"),
        "reject/blocked-empty.bin" => {
            include_bytes!("../../../../spec/vectors/reject/blocked-empty.bin")
        }
        "reject/leb-nonminimal.bin" => {
            include_bytes!("../../../../spec/vectors/reject/leb-nonminimal.bin")
        }
        "reject/leb-overflow.bin" => {
            include_bytes!("../../../../spec/vectors/reject/leb-overflow.bin")
        }
        "reject/leb-overflow-cont.bin" => {
            include_bytes!("../../../../spec/vectors/reject/leb-overflow-cont.bin")
        }
        "reject/leb-range-before-nonminimal.bin" => {
            include_bytes!("../../../../spec/vectors/reject/leb-range-before-nonminimal.bin")
        }
        "reject/leb-truncated.bin" => {
            include_bytes!("../../../../spec/vectors/reject/leb-truncated.bin")
        }
        "reject/truncated-payload.bin" => {
            include_bytes!("../../../../spec/vectors/reject/truncated-payload.bin")
        }
        "reject/truncated-trailer.bin" => {
            include_bytes!("../../../../spec/vectors/reject/truncated-trailer.bin")
        }
        "reject/crc-mismatch.bin" => {
            include_bytes!("../../../../spec/vectors/reject/crc-mismatch.bin")
        }
        "reject/trailing.bin" => include_bytes!("../../../../spec/vectors/reject/trailing.bin"),
        other => panic!("no embedded fixture for {other}"),
    }
}

pub fn decode_hex(text: &str) -> Vec<u8> {
    let rest = text.strip_prefix("hex:").expect("hex prefix");
    assert_eq!(rest.len() % 2, 0);
    let raw = rest.as_bytes();
    let mut out = Vec::with_capacity(rest.len() / 2);
    let mut i = 0;
    while i < raw.len() {
        let hi = hex_nibble(raw[i]);
        let lo = hex_nibble(raw[i + 1]);
        out.push((hi << 4) | lo);
        i += 2;
    }
    out
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("bad hex"),
    }
}

pub fn named_error(name: &str) -> Error {
    match name {
        "TruncatedHeader" => Error::TruncatedHeader,
        "BadMagic" => Error::BadMagic,
        "BadVersion" => Error::BadVersion,
        "ReservedBits" => Error::ReservedBits,
        "ReservedMode" => Error::ReservedMode,
        "Leb128Overflow" => Error::Leb128Overflow,
        "NonMinimalLeb128" => Error::NonMinimalLeb128,
        "TruncatedLeb128" => Error::TruncatedLeb128,
        "BlockedEmpty" => Error::BlockedEmpty,
        "TruncatedPayload" => Error::TruncatedPayload,
        "TruncatedTrailer" => Error::TruncatedTrailer,
        "CrcMismatch" => Error::CrcMismatch,
        "TrailingBytes" => Error::TrailingBytes,
        other => panic!("unknown error {other}"),
    }
}
