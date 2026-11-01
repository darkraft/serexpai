---
baseline_commit: f1b1ed1dfb03c9feb32a8703964de70377a369f9
---

# Story 1.1: Round-trip a file through whole-file raw

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want a file to survive a whole-file raw bitstream,
so that the worst case is lossless and size-bounded before predictors exist.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or committed fixture must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence is enough for story 1.2 to start (elicitation amendment 1, 2026-11-01); it does not close AC02 or AC07.

### Group A — Workspace and contract

1. **S1.1-AC01.** A clean checkout builds the four `sxp-` crates with the pinned toolchain, edition, and committed lockfile. The dependency pins below are present as exact `=` requirements. A resolution failure stops the task. Do not substitute a newer crate or a different Rust.
2. **S1.1-AC02.** `sxp-kernel` forbids unsafe code, has zero third-party normal dependencies, and exposes typed errors with no shell context strings and no global mutable state. `ci.sh` runs formatting, clippy with warnings denied, tests, and the kernel dependency check, and it does so only through `cargo` and `rustup`. The acceptance record names both targets: `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.
3. **S1.1-AC03.** G0 is the existing bitstream spec seed. Do not write a second normative document and do not start `spec/bitstream.md` (that is story 2.1). Before the framing implementation, commit the seven-byte empty stream and the CRC-32/ISO-HDLC check fixture for the ASCII bytes `123456789`.

### Group B — Round-trip and bounds

4. **S1.1-AC04.** Empty, short, random, one already-compressed fixture, and the LEB128 width boundaries round-trip or are checked as header fixtures. Under the two-byte fixed header, raw size is exactly `N + 6 + leb128_len(N)`, and at most `N + 11` for `N < 2^35`. Width boundaries above a few kilobytes are writer/parser fixtures. Do not allocate those inputs.
5. **S1.1-AC05.** Declared lengths at or above `2^35`, non-minimal LEB128, overflowing LEB128, truncated LEB128, bad magic, versions other than 1 (including 0 and 15), reserved bits, modes 2 and 3, truncated payload, truncated trailer, CRC mismatch, and trailing bytes each return the typed error named below. Each rule has the committed fixture named below. Mode 0 with `N = 0` is a format error. Mode 0 with `N > 0` is not a format error.
6. **S1.1-AC06.** Stream-to-sink tests show decoder-owned heap does not grow with `N` and stays at most 64 KiB under the accounting boundary below. The recorded fuzz profile and the malformed fixtures produce no panic and no out-of-bounds access.
7. **S1.1-AC07.** The only writer is whole-file raw. Blocked-mode writing does not exist until story 1.8. Exact-byte fixtures pass on both supported targets.

## Tasks / Subtasks

- [x] **Workspace** (AC: 1, 2)
  - [x] Add the workspace, four crates, `rust-toolchain.toml`, `Cargo.lock`, `/target/` in `.gitignore`, and `.gitkeep` files only in `tools/refdec/`, `tools/fetch/`, and `corpora/`.
  - [x] Pin every third-party dependency with `=`. Generate the lockfile with `rustup run 1.99.1 cargo generate-lockfile` and commit it. Every later `cargo` invocation in `ci.sh` is `rustup run 1.99.1 cargo … --locked`.
  - [x] `ci.sh` uses `rustup` and `cargo` only. `rustup show active-toolchain` must contain `1.99.1`. Clippy is `cargo clippy --workspace --all-targets --locked -- -D warnings`. Tests are `cargo test --locked`.
  - [x] `cargo tree -p sxp-kernel -e normal --target all --prefix none --locked` prints exactly one line. The same command with `-e build` prints exactly one line. `sxp-kernel` has no `build.rs`.
  - [x] Optional target argument: `./ci.sh aarch64-unknown-linux-gnu` runs `rustup target add` for that triple and passes `--target` to cargo. qemu-user, if needed, is a cargo runner in `.cargo/config.toml`.
- [x] **Fixtures before framing code** (AC: 3)
  - [x] Confirm `bitstream-spec-seed.md` still has SHA-256 `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`. If it does not, stop.
  - [x] Commit the fixtures and `spec/vectors/index.toml` in a commit that contains no framing implementation. Record that commit hash in the Dev Agent Record before adding `crc32.rs`, `leb128.rs`, `header.rs`, or `raw.rs`.
- [x] **Kernel framing** (AC: 4, 5, 7)
  - [x] Implement CRC, unsigned LEB128, header, and streaming whole-file raw under `crates/sxp-kernel/src/`.
  - [x] `encode_whole_file_raw_header` and `write_whole_file_raw` always emit version 1, mode 1. Neither takes a mode argument.
  - [x] Commit one fixture per reject rule, listed in `spec/vectors/index.toml`. Keep the mode-0 `N > 0` literal in the test source only.
- [x] **Bounds, heap, fuzz** (AC: 4, 6)
  - [x] Stream round-trips through `N = 2^21` and `N = 2^21 + 1` without storing the payload. Header-only checks at `2^28` and `N = 2^35 - 1` (legal, overhead 11). Writer rejects `N >= 2^35`.
  - [x] Counting-allocator test in `tests/heap_bound.rs`. Proptest profile and the 56 named bit-flip results below.
- [x] **Acceptance record** (AC: 2, 7)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands and results in the Dev Agent Record. Leave the story open if either target did not run.

### Review Findings

- [x] [Review][Patch] Lengths 127 and 128, and the 2 MiB streams, are full-byte round-trips that do not store the payload [crates/sxp-kernel/tests/round_trip.rs]
- [x] [Review][Patch] `write_whole_file_raw` and `read_whole_file_raw` take `impl Read` and `impl Write` [crates/sxp-kernel/src/raw.rs]
- [x] [Review][Patch] CI toolchain check rejects a version that merely contains `1.99.1` [ci.sh:6]
- [x] [Review][Patch] Empty `cargo tree` output still prints the single-line diagnostic [ci.sh]
- [x] [Review][Patch] `ci.sh` rejects more than one argument [ci.sh]
- [x] [Review][Patch] Vector parser clears optional fields and rejects duplicate keys [crates/sxp-kernel/tests/support/mod.rs]
- [x] [Review][Patch] Fixture binaries are marked binary in `.gitattributes`
- [x] [Review][Patch] Gzip fixture test matches the documented hex [crates/sxp-kernel/tests/round_trip.rs]
- [x] [Review][Patch] Overflow fixtures fail if a sixth length byte is read [crates/sxp-kernel/tests/round_trip.rs]
- [x] [Review][Patch] Truncation prefixes assert the named error, in one test [crates/sxp-kernel/tests/proptest_raw.rs]
- [x] [Review][Patch] File list and the `cargo fmt` exception are recorded in the Dev Agent Record

## Dev Notes

### What this story builds

Greenfield. There is no Rust tree yet. This story creates the Cargo workspace and a kernel that writes and reads one mode: whole-file raw (magic, version/mode byte, minimal LEB128 length, verbatim bytes, CRC-32).

Later stories own the contents of these paths. Create the directories with a `.gitkeep` and no other file:

- `tools/refdec/` (story 2.2). `ci.sh` does not invoke Python. Amendment 6: a missing Python decoder does not block stories 1.1–1.13.
- `tools/fetch/` and `corpora/` (story 1.12). Do not create `corpora/manifest.toml`.

Do not create these files at all:

- `spec/bitstream.md`, `spec/sine_lut.*` (stories 2.1 and the evaluator stories). A placeholder does not close a gate.
- `compress` / `decompress` CLI behaviour, temp-file rename, and stderr sentences (story 1.11). The `sxp-cli` binary compiles and does nothing else.
- Predictors, residual math, range coder, probability model, Block headers (stories 1.2–1.8).
- `LICENSE` files and a CLA (story 2.3). Set `license = "MIT OR Apache-2.0"` on the crates only.
- Product version and `docs/CHANGELOG.md`. Crate version is `0.0.7`, matching the repository. Do not bump it.
- README claims that a CLI exists. Leave the README.

AC01's directories are the four crate directories, `spec/vectors/`, and the three `.gitkeep` directories above.

### Toolchain — cargo and rustup only

Project rule: every Rust command goes through `cargo` or `rustup`.

- `rust-toolchain.toml` sets `channel = "1.99.1"`, components `rustfmt` and `clippy`, and targets `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. Edition 2024 belongs in `[workspace.package]`, not in that file.
- `ci.sh` runs every Cargo command as `rustup run 1.99.1 cargo … --locked`. That keeps a distro `cargo` earlier on `PATH` from rewriting the lockfile. `rustup show active-toolchain` must contain `1.99.1`. Do not call `rustc --version`.
- Allowed: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`, `cargo tree`, `cargo metadata`, `cargo fetch`, `cargo generate-lockfile`, `rustup show`, `rustup toolchain install`, `rustup component add`, `rustup target add`, `rustup run`.
- Forbidden: `rustc`, `rustfmt`, `clippy-driver`, a binary under `~/.rustup/toolchains/`, `RUSTC` / `CARGO_BUILD_RUSTC`, a toolchain tarball, apt, or snap. If rustup is absent, stop. Do not fall back to a system compiler.
- Cross tests: `rustup target add <triple>` then `rustup run 1.99.1 cargo test --locked --target <triple>`. A qemu runner belongs in `.cargo/config.toml` as `runner = "qemu-aarch64"` (or `qemu-x86_64` on the other host).

Rust 1.99.1 was released 2026-10-15 and supports edition 2024. Confirmed crate versions, all exact, all not yanked:

| Crate | Requirement | Where |
| --- | --- | --- |
| rayon | `=1.12.0` | `sxp-encoder` normal dependency |
| clap | `=4.6.7` | `sxp-cli` normal dependency, default features |
| serde | `=1.0.229` | `sxp-bench` |
| serde_json | `=1.0.154` | `sxp-bench` |
| toml | `=1.1.8` | `sxp-bench`. The published version string is `1.1.8+spec-1.1.0`. A lock entry with that build metadata is the same pin, not drift. |
| sha2 | `=0.11.1` | `sxp-bench` |
| proptest | `=1.12.0`, `default-features = false`, `features = ["std"]` | `sxp-kernel` dev-dependency only |

If `cargo fetch` cannot resolve one of those, stop and report it. Do not widen the requirement and do not add `[patch]`. A virtual workspace does not infer the resolver from the edition. Set `resolver = "3"` explicitly. Set workspace `rust-version = "1.99.1"`, `edition = "2024"`, `license = "MIT OR Apache-2.0"`, `version = "0.0.7"`, and `publish = false`. Each crate inherits those with `workspace = true` fields. Generate `Cargo.lock` with `rustup run 1.99.1 cargo generate-lockfile`.

Crate edges, declared now even where the story does not call them yet:

- `sxp-encoder` depends on `sxp-kernel` and `rayon`. It does not serialize a field.
- `sxp-bench` depends on `sxp-encoder` and `sxp-kernel`, plus the bench pins above.
- `sxp-cli` depends on `sxp-bench`, `sxp-encoder`, `sxp-kernel`, and `clap`.

`read_whole_file_raw` calls the kernel only. Do not route decode through the encoder. Do not import an unused dependency just to silence a lint, and do not enable `unused_crate_dependencies`.

`sxp-kernel` normal dependencies: none. No `build.rs`. Dev-dependency proptest is allowed. `cargo tree -e normal` excludes it.

### Kernel shape

```text
crates/sxp-kernel/src/lib.rs       # forbid(unsafe_code), re-exports
crates/sxp-kernel/src/error.rs
crates/sxp-kernel/src/crc32.rs
crates/sxp-kernel/src/leb128.rs
crates/sxp-kernel/src/header.rs
crates/sxp-kernel/src/raw.rs
crates/sxp-kernel/tests/           # all tests, including proptest and the allocator
```

Lints live only as inner attributes in `lib.rs`. Do not set them in `[lints]` in `Cargo.toml`. Package lints also cover `tests/`, and `forbid` cannot be overridden there. The heap test needs `unsafe impl GlobalAlloc`.

```rust
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable
)]
```

`expect`, `todo`, `unimplemented`, and `unreachable` panic and are not covered by `unwrap_used`. No `#[allow]` of those lints in `src/`. No `#[cfg(test)]` module in `src/`. Integration tests may use `assert!`. The only `unsafe` in the story is the test-only allocator in `tests/heap_bound.rs`.

`clippy::indexing_slicing` also denies `buf[..n]` and a table lookup `TABLE[i]`. Read bytes with `get` or `split_at_checked`, and turn a missing byte into the typed truncation error. Implement CRC-32 by the bitwise algorithm. A `const` lookup table is allowed only when every read goes through `get` and a logic error on a missing entry; an immutable table is not global mutable state.

LEB128 shifts and remaining-length arithmetic use `checked_shl` and `checked_sub`. Overflow returns `Leb128Overflow`. Debug overflow panics fail AC06.

No `thiserror`, `anyhow`, or other error crate. `Error` is a plain enum deriving `Copy`, `Clone`, `Debug`, `PartialEq`, and `Eq`. `Display` is the variant name and nothing more (`BadMagic`, not a sentence). I/O failures are `Error::Io(std::io::ErrorKind)`, not a wrapped `std::io::Error`. Library functions return `Result`.

No `static mut`, no lazy global, no process-wide codec state.

Public surface the tests and later stories use:

- `crc32_iso_hdlc` incremental update over a caller-owned buffer, plus a helper that folds a `&[u8]`.
- `encode_leb128` / `decode_leb128` for unsigned values. Encoded form fits in `[u8; 8]` plus a length; do not pull in `arrayvec`.
- `encode_whole_file_raw_header(n: u64) -> Result<HeaderBytes, Error>`. This function always writes version 1 and mode 1. It has no mode argument. Tests that need mode 0 use the literal fixture bytes.
- `decode_header(&[u8]) -> Result<(Header, usize), Error>` where `Header` carries version, mode, and `N`.
- `write_whole_file_raw(len: u64, payload: impl Read, out: impl Write) -> Result<(), Error>`.
- `read_whole_file_raw(input: impl Read, out: impl Write) -> Result<(), Error>`.

Do not use `read_exact` or `std::io::copy`. Fill with a loop. `ErrorKind::Interrupted` retries. A read of 0 bytes is the truncation error for the section in progress (`TruncatedHeader`, `TruncatedLeb128`, `TruncatedPayload`, or `TruncatedTrailer`), not `Io(UnexpectedEof)`.

`len` is authoritative. The writer reads exactly `len` bytes, then one more probe byte. End of input on that probe is success. One extra byte is `ExcessPayload`. A short reader is `TruncatedPayload`. `N >= 2^35` is `LengthOutOfRange` and writes nothing. Any later error may leave bytes already written on the sink. Story 1.11's temp file and rename owns rollback. The decoder copies payload to the sink before it checks the CRC, so `CrcMismatch` may leave the payload on the sink. Header errors and `BlockedModeUnsupported` write nothing.

### Wire format (locked from the spec seed)

Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` §1–§2, and spine AD-11. Little-endian integers. Inside a byte, fields fill from the least significant bit.

Whole-file raw layout:

| Field | Bytes | Value |
| --- | --- | --- |
| magic | 1 | `0x8D` |
| flags | 1 | bits 0–3 version, bits 4–5 mode, bits 6–7 reserved |
| N | 1–5 | unsigned LEB128, minimal, `N < 2^35` |
| payload | N | verbatim input |
| trailer | 4 | `LE32(CRC-32/ISO-HDLC of the payload)` |

Version 1, mode 1 (whole-file raw), reserved 0 packs as flags byte `0x11`.

Empty input, the AC03 fixture `spec/vectors/whole-file-raw-empty.bin`:

```text
8D 11 00 00 00 00 00
```

That is magic, flags `0x11`, LEB128(0) = `00`, and CRC of no bytes `00 00 00 00`. Seven bytes. Size formula: `0 + 6 + 1 = 7`.

CRC-32/ISO-HDLC: reflected polynomial `0xEDB88320`, init `0xFFFFFFFF`, xorout `0xFFFFFFFF`. Check value of ASCII `123456789` is `0xCBF43926`. Fixture `spec/vectors/crc32-123456789.bin` is the four little-endian bytes `26 39 F4 CB`. The test feeds the nine ASCII bytes through the kernel CRC and compares the file. Empty CRC is `0x00000000`.

Size is `2 + leb128_len(N) + N + 4`, which is `N + 6 + leb128_len(N)`. Five-byte lengths reach `N + 11`. `N = 2^35 - 1` is legal and is the tight bound. `N >= 2^35` is rejected and is never written.

Unsigned LEB128: 7 data bits, continuation in bit 7. The decoder never reads a sixth length byte. If the fifth byte has bit 7 set, return `Leb128Overflow` without reading further. Then, a multi-byte encoding whose last byte is `0x00` is `NonMinimalLeb128` (seed §2). Then, a continuation bit with no following byte is `TruncatedLeb128`. Five bytes hold 35 bits, so the largest legal encoding is `FF FF FF FF 7F` (`2^35 - 1`). `80 80 80 80 80 00` hits the range rule, not the non-minimal rule, because its fifth byte is `0x80`.

| N | Width | Encoding |
| --- | ---: | --- |
| 0 | 1 | `00` |
| 127 | 1 | `7F` |
| 128 | 2 | `80 01` |
| 16383 | 2 | `FF 7F` |
| 16384 | 3 | `80 80 01` |
| 2^21 − 1 | 3 | `FF FF 7F` |
| 2^21 | 4 | `80 80 80 01` |
| 2^28 − 1 | 4 | `FF FF FF 7F` |
| 2^28 | 5 | `80 80 80 80 01` |
| 2^35 − 1 | 5 | `FF FF FF FF 7F` |
| 2^35 | rejected on byte 5 | `80 80 80 80 80` and the sixth byte is never read |

Full byte round-trips use a `Read` that does not store the payload: `N` in `{0, 1, 127, 128}`, the committed payloads below, and `N = 2^21` and `N = 2^21 + 1`. Widths at `2^28` and `2^35 - 1` are header encode/decode only. Do not stream those lengths. qemu is too slow, and the test must not allocate them.

Check order on read, first match wins:

1. No bytes at all, or magic seen and the flags byte missing → `TruncatedHeader`.
2. First byte is not `0x8D` → `BadMagic`.
3. Version ≠ 1 → `BadVersion`. Covers 0, 15, and 2..=14.
4. Reserved bits 6–7 nonzero → `ReservedBits`.
5. Mode 2 or 3 → `ReservedMode`.
6. LEB128: fifth byte has bit 7 set → `Leb128Overflow`. Else a multi-byte encoding ending in `0x00` → `NonMinimalLeb128`. Else a dangling continuation → `TruncatedLeb128`.
7. Mode 0 and `N = 0` → `BlockedEmpty`. This stays a format error after story 1.8.
8. Mode 0 and `N > 0` → header parse succeeds. `read_whole_file_raw` returns `BlockedModeUnsupported` and writes nothing. Story 1.8 replaces this arm. Mode 0 is not a reserved mode. This case is a test literal, not a file under `spec/vectors/`.
9. Fewer than `N` payload bytes → `TruncatedPayload`.
10. Fewer than 4 trailer bytes → `TruncatedTrailer`.
11. CRC mismatch → `CrcMismatch`.
12. Any byte after the trailer → `TrailingBytes`.

Every single-bit flip of `whole-file-raw-empty.bin` has one expected error. Assert the variant, not only `Err`.

| Flip | Result |
| --- | --- |
| Byte 0, any of 8 bits | `BadMagic` |
| Byte 1 bit 0 (`0x10`) | `BadVersion` (version 0) |
| Byte 1 bit 1 (`0x13`) | `BadVersion` (version 3) |
| Byte 1 bit 2 (`0x15`) | `BadVersion` (version 5) |
| Byte 1 bit 3 (`0x19`) | `BadVersion` (version 9) |
| Byte 1 bit 4 (`0x01`) | `BlockedEmpty` |
| Byte 1 bit 5 (`0x31`) | `ReservedMode` (mode 3) |
| Byte 1 bit 6 (`0x51`) | `ReservedBits` |
| Byte 1 bit 7 (`0x91`) | `ReservedBits` |
| Byte 2 `0x01`, `0x02`, `0x04` | `TruncatedTrailer` (`N` is 1, 2, or 4) |
| Byte 2 `0x08`, `0x10`, `0x20`, `0x40` | `TruncatedPayload` (`N` is 8, 16, 32, or 64) |
| Byte 2 `0x80` | `NonMinimalLeb128` |
| Bytes 3–6, any of 32 bits | `CrcMismatch` |

Streaming decode copies through a stack buffer of 8192 bytes and updates the CRC in place. Do not call `read_to_end` or `Vec::with_capacity(n)`. Stack memory is not heap. The implementation allocates no decoder-owned heap.

### Committed fixtures

Load fixtures with `include_bytes!` from paths under the crate, so the tests do not depend on the process current directory under qemu. `spec/vectors/index.toml` maps every file to a provisional rule id and the expected error variant. Story 2.1 may adopt these ids. Rule ids in this story:

| Rule id | File | Expected |
| --- | --- | --- |
| `raw-empty` | `whole-file-raw-empty.bin` | round-trip, decoded empty |
| `raw-123456789` | `whole-file-raw-123456789.bin` | round-trip, decoded ASCII `123456789` |
| `crc-123456789` | `crc32-123456789.bin` | CRC bytes of that ASCII input |
| `raw-a5` | `payload-a5.bin` | round-trip of payload `A5` |
| `raw-gzip` | `payload-gzip.bin` | round-trip of the gzip fixture |
| `reject-empty` | `reject/empty.bin` | `TruncatedHeader` |
| `reject-truncated-header` | `reject/truncated-header.bin` | `TruncatedHeader` |
| `reject-bad-magic` | `reject/bad-magic.bin` | `BadMagic` |
| `reject-version-0` | `reject/version-0.bin` | `BadVersion` |
| `reject-version-2` | `reject/version-2.bin` | `BadVersion` |
| `reject-version-15` | `reject/version-15.bin` | `BadVersion` |
| `reject-reserved-bits` | `reject/reserved-bits.bin` | `ReservedBits` |
| `reject-mode-2` | `reject/mode-2.bin` | `ReservedMode` |
| `reject-mode-3` | `reject/mode-3.bin` | `ReservedMode` |
| `reject-blocked-empty` | `reject/blocked-empty.bin` | `BlockedEmpty` |
| `reject-leb-nonminimal` | `reject/leb-nonminimal.bin` | `NonMinimalLeb128` |
| `reject-leb-overflow` | `reject/leb-overflow.bin` | `Leb128Overflow` |
| `reject-leb-overflow-cont` | `reject/leb-overflow-cont.bin` | `Leb128Overflow` |
| `reject-leb-range-before-nonminimal` | `reject/leb-range-before-nonminimal.bin` | `Leb128Overflow` |
| `reject-leb-truncated` | `reject/leb-truncated.bin` | `TruncatedLeb128` |
| `reject-truncated-payload` | `reject/truncated-payload.bin` | `TruncatedPayload` |
| `reject-truncated-trailer` | `reject/truncated-trailer.bin` | `TruncatedTrailer` |
| `reject-crc-mismatch` | `reject/crc-mismatch.bin` | `CrcMismatch` |
| `reject-trailing` | `reject/trailing.bin` | `TrailingBytes` |

Exact bytes:

| File | Bytes |
| --- | --- |
| `whole-file-raw-empty.bin` | `8D 11 00 00 00 00 00` |
| `whole-file-raw-123456789.bin` | `8D 11 09 31 32 33 34 35 36 37 38 39 26 39 F4 CB` |
| `crc32-123456789.bin` | `26 39 F4 CB` |
| `payload-a5.bin` | `A5` |
| `reject/empty.bin` | empty file |
| `reject/truncated-header.bin` | `8D` |
| `reject/bad-magic.bin` | `00 11 00 00 00 00 00` |
| `reject/version-0.bin` | `8D 10 00 00 00 00 00` |
| `reject/version-15.bin` | `8D 1F 00 00 00 00 00` |
| `reject/version-2.bin` | `8D 12 00 00 00 00 00` |
| `reject/reserved-bits.bin` | `8D 51 00 00 00 00 00` |
| `reject/mode-2.bin` | `8D 21 00 00 00 00 00` |
| `reject/mode-3.bin` | `8D 31 00 00 00 00 00` |
| `reject/blocked-empty.bin` | `8D 01 00 00 00 00 00` |
| `reject/leb-nonminimal.bin` | `8D 11 80 00 00 00 00 00` |
| `reject/leb-overflow.bin` | `8D 11 80 80 80 80 80 01` |
| `reject/leb-overflow-cont.bin` | `8D 11 FF FF FF FF FF` |
| `reject/leb-range-before-nonminimal.bin` | `8D 11 80 80 80 80 80 00` |
| `reject/leb-truncated.bin` | `8D 11 80` |
| `reject/truncated-payload.bin` | `8D 11 01` |
| `reject/truncated-trailer.bin` | `8D 11 01 A5 EA B8` |
| `reject/crc-mismatch.bin` | `8D 11 01 A5 EB B8 BE 74` |
| `reject/trailing.bin` | `8D 11 00 00 00 00 00 00` |

`payload-gzip.bin` is `gzip -9n` of the ASCII line `serexpai-story-1.1-already-compressed\n` (58 bytes). SHA-256 `bb97b5ff0d6edca7aad354076341f1f54a18faf886f29992d58b8c5d1dfd18e8`. Hex:

```text
1f8b08000000000002032b4e2d4aad2848ccd42d2ec92faad435d433d44dcc29
4a4d4ca9d44dcecf2d284a2d2e4e4de1020050f1f47f26000000
```

`N = 58` encodes as `3A`. The raw stream length is `58 + 6 + 1 = 65`. Record that provenance next to the file in `index.toml`.

Known good `N = 1` payload `A5`: `8D 11 01 A5 EA B8 BE 74`. CRC of `{0xA5}` is `0x74BEB8EA`. The mismatch fixture flips the first CRC byte. The `123456789` stream is 16 bytes: flags `8D 11`, LEB128 `09`, nine ASCII bytes, CRC `26 39 F4 CB`.

`BlockedModeUnsupported` is not a published vector. The test literal is mode 0, `N = 1`, payload `00`, CRC of `{0x00}` = `0xD202EF8D`: `8D 01 01 00 8D EF 02 D2`. The sink stays empty. Story 1.8 will accept this shape as a real blocked stream, so it must not sit in `spec/vectors/`.

Also run `BadVersion` for every version in `0..=15` except 1, through `decode_header`, so versions 3–14 are the same rule as the version-2 fixture.

### Heap accounting and fuzz profile

Accounting boundary, written in the doc comment of `tests/heap_bound.rs`: count the peak decoder-owned heap during `read_whole_file_raw`. Count working buffers and tables the decoder allocates. Exclude caller-owned input, the sink, and bytes allocated while building the fixture. Do not satisfy the cap by allocating the file in the caller and calling that outside the decoder.

That file is its own integration-test binary and contains one `#[test]`, so the global counter is not shared with other tests. Track the peak, not the total. Build the input only after the CRC of `N` zero bytes is known: a stack header, `std::io::repeat(0).take(N)`, and the four CRC bytes. The sink is `std::io::sink()`. Compare `N = 0` and `N = 1_048_576`. Both peaks are `≤ 65536` and the 1 MiB peak is not larger. Start the counter after the CRC exists.

Record this profile in a comment at the top of the proptest file, and use it:

```text
fuzz-profile: proptest =1.12.0, default-features = false, features = ["std"],
cases = 256, rng_seed = RngSeed::Fixed(0x5111_0001),
failure_persistence = FileFailurePersistence::Off
```

If `RngSeed::Fixed` is not in proptest 1.12.0, stop and report the actual API. Do not substitute an unrecorded seed. Generators:

- Round-trip payloads of length `0..=4096`. Decode(encode(payload)) equals the payload, and the encoded length equals `N + 6 + leb128_len(N)`.
- Every truncation prefix of the empty stream and of the `A5` stream.
- Random suffixes after the prefix `8D 11`.
- The 56 single-bit flips in the table above, each with its named variant.

A failing generated case is reduced and added under `spec/vectors/reject/` with a new index row. Do not commit `.proptest-regressions`. Do not add `cargo-fuzz` or libfuzzer. Finite cases are not a proof over every byte string; the comment says so.

### Project Structure Notes

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
ci.sh
.gitignore                 # add /target/
crates/sxp-kernel/
crates/sxp-encoder/
crates/sxp-bench/
crates/sxp-cli/
spec/vectors/              # fixtures and index.toml
tools/refdec/.gitkeep
tools/fetch/.gitkeep
corpora/.gitkeep
```

Clippy denies apply to `sxp-kernel` only. The other crates still build with `-D warnings`.

No `project-context.md` exists in this repository. The normative inputs are the epic breakdown, the architecture spine, and the bitstream spec seed.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.1, elicitation amendments 1, 3, 5, 6]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` §1–§2]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-1, AD-8, AD-11, AD-12, Stack, Structural Seed]
- [Source: `.cursor/rules/cargo-rustup.mdc`]
- G0 file hash: SHA-256 `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e` of `bitstream-spec-seed.md` as of this story. The seed is gitignored on the code branch, so the acceptance record keeps this hash rather than a path alone.

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created
- Plan review applied: fifth-byte LEB128 rule, fixed reject order, raw-only header encoder, lints in `lib.rs` only, `--locked` via `rustup run`, gzip fixture, vector index, `.gitkeep` directories, G0 hash.
- Fixture commit, recorded before framing modules existed: `5a58bbb011ebe761bb246cc4557587b056b21d9c` ("Add the pinned workspace and whole-file raw fixtures."). G0 SHA-256 rechecked and still `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`.
- Whole-file raw lives in `sxp-kernel`: bitwise CRC-32/ISO-HDLC, unsigned LEB128, header, and streaming read/write. The header encoder has no mode argument. `ci.sh` places `--locked` and `--target` before clippy's `--`, so rustc does not see `--locked`.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-01: fmt, clippy `-D warnings`, the workspace tests, and both kernel tree checks.
- `./ci.sh aarch64-unknown-linux-gnu` passed on 2026-11-01 under `qemu-aarch64`, including the 2 MiB stream, the proptest profile (256 cases, fixed seed), and the heap bound. `.cargo/config.toml` sets the linker to `aarch64-linux-gnu-gcc` and the runner to `qemu-aarch64`. This host had neither on `PATH`; the run used a local cross gcc and a static qemu-user ahead of `PATH`.
- `cargo fmt` does not accept `--locked` (`unexpected argument '--locked'`, exit 2). Clippy, test, and `cargo tree` pass `--locked`. Code review applied: streamed round-trips for 0, 1, 127, 128, 2^21, and 2^21+1 without storing the payload; public I/O takes `impl Read` and `impl Write`; CI version and argument checks; binary fixture attributes; gzip hex pin; unread sixth length byte; named truncation errors.

### File List

- `.cargo/config.toml`
- `.gitattributes`
- `.gitignore`
- `Cargo.lock`
- `Cargo.toml`
- `ci.sh`
- `corpora/.gitkeep`
- `crates/sxp-bench/Cargo.toml`
- `crates/sxp-bench/src/lib.rs`
- `crates/sxp-cli/Cargo.toml`
- `crates/sxp-cli/src/main.rs`
- `crates/sxp-encoder/Cargo.toml`
- `crates/sxp-encoder/src/lib.rs`
- `crates/sxp-kernel/Cargo.toml`
- `crates/sxp-kernel/src/lib.rs`
- `crates/sxp-kernel/src/crc32.rs`
- `crates/sxp-kernel/src/error.rs`
- `crates/sxp-kernel/src/header.rs`
- `crates/sxp-kernel/src/leb128.rs`
- `crates/sxp-kernel/src/raw.rs`
- `crates/sxp-kernel/tests/heap_bound.rs`
- `crates/sxp-kernel/tests/proptest_raw.rs`
- `crates/sxp-kernel/tests/round_trip.rs`
- `crates/sxp-kernel/tests/support/mod.rs`
- `rust-toolchain.toml`
- `spec/vectors/index.toml`
- `spec/vectors/*.bin`
- `spec/vectors/reject/*.bin`
- `tools/fetch/.gitkeep`
- `tools/refdec/.gitkeep`

### Change Log

- 2026-11-01: Whole-file raw framing, tests, CI target argument, and aarch64 qemu runner. Fixture commit `5a58bbb011ebe761bb246cc4557587b056b21d9c` predates the framing modules.
- 2026-11-01: Addressed code review findings — 11 patches applied.

## Story completion status

done. Both target CI runs passed, and the second code review found nothing left to fix.
