---
baseline_commit: aa2fb8fdbee623bceb1123db7dd8010d87a7c579
---

# Story 1.6: Code decisions with the binary range coder

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want a specified binary range coder with exact byte counts and independently decodable decisions,
so that a later Block can ask the kernel for the exact coded length and recover every bit without a model update or a coder id.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or the committed hash must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence is enough to start the next story. It does not close AC06.

1. **S1.6-AC01.** G0 already specifies the coder in `bitstream-spec-seed.md` §6 and §6.1. Do not edit the seed, and do not add a second normative document. Encoder state starts `low = 0` (`u64`), `range = 0xFFFF_FFFF` (`u32`), `cache = 0` (`u8`), `cache_size = 1` (`u64`). The same module exposes encode, decode, and exact size. No coder id is written or read.
2. **S1.6-AC02.** `freq0` is in `1..=4095` and the total is the constant 4096. `freq0` of `0` or `4096..=65535` is `Error::InvalidParameter`, and so is a bit outside `0..=1`. The check happens before any state change. For every fixed `(bit, freq0)` schedule below, decoding returns those bits and `exact_size` equals the emitted length, including carry bytes and the four-byte termination.
3. **S1.6-AC03.** `finish` calls `shift_low` five times. The byte sink drops only the first emitted byte, which is `0x00`, and keeps every later byte from that same call. The termination overhead on top of the omitted byte is four bytes when the body itself emits nothing. Pinned bytes: bit 0 / `freq0` 2048 → `00 00 00 00`; bit 1 / `freq0` 2048 → `7F FF F8 00`; bit 0 / 1 → `00 00 00 00 00`; bit 1 / 1 → `00 0F FF FF`; bit 0 / 4095 → `00 00 00 00`; bit 1 / 4095 → `FF EF F0 01 00`. The carry schedule is pinned in the tasks.
4. **S1.6-AC04.** The 2048-symbol fixture is 2048 bytes `i & 0xFF`, each expanded MSB first into eight decisions, with `freq0 = 1 + (decision_index % 4095)`. That is 16384 decisions. Commit the encoded length and SHA-256. Property tests cover other schedules.
5. **S1.6-AC05.** Decoder initialization reads four bytes, most significant byte first. Fewer than four bytes is `Error::TruncatedCoder`, including a three-byte `FF FF FF`. Four bytes `FF FF FF FF` is `Error::SaturatedRangeCode`. After the last decision, `code != 0` is `Error::NonzeroRangeCode`. Every proper prefix of a pinned fixture, decoded with that fixture's own `freq0` list, returns `Error::TruncatedCoder`. A concatenated buffer is sliced with `bytes_consumed`; the decoder does not read the next frame.
6. **S1.6-AC06.** The per-decision order is: take `freq0` from the caller, decide the bit, update coder state, refill while `range < 2^24`. There is no fifth step and no probability update. No rANS, no deferred symbol, no pending-symbol flag, and no coder state carried into a second `new`. Both platforms match the committed hash.

## Tasks / Subtasks

- [x] **Encoder** (AC: 1, 2, 3)
  - [x] Add `crates/sxp-kernel/src/range_coder.rs`. `pub struct RangeEncoder` with private fields `low: u64`, `range: u32`, `cache: u8`, `cache_size: u64`, `out: Vec<u8>`, `dropped_leading: bool`. Derive `Debug` only.
  - [x] `pub fn new() -> Self`. `low = 0`, `range = 0xFFFF_FFFF`, `cache = 0`, `cache_size = 1`, `out` empty, `dropped_leading = false`. These four numbers are the only legal start. A second block is a new encoder.
  - [x] `pub fn encode(&mut self, bit: u8, freq0: u16) -> Result<(), Error>`. Reject, before any write, when `bit > 1` or `freq0 == 0` or `freq0 > 4095`, with `Error::InvalidParameter`. A rejected call leaves `state()` unchanged.
  - [x] Bound: `let product = u64::from(self.range >> 12) * u64::from(freq0);` then `u32::try_from(product)`. The product is at most `0xF_FFFF * 4095 = 0xFFEF_F001`, so the conversion succeeds. Do not use `wrapping_mul`. Bit 0 sets `range = bound`. Bit 1 does `low += u64::from(bound)` and `range -= bound`. `low` stays below `2^33`, so the carry is 0 or 1.
  - [x] Renormalize with `while self.range < 0x0100_0000`, never `if`. Body: `self.range <<= 8` (legal because `range < 2^24`, so the shift stays in `u32`), then `shift_low`. Freq0 1 can make `range` as small as `2^12` and the next decision can shift twice. The loop condition is the only stop.
  - [x] `shift_low`, in this order. If `low < 0xFF00_0000 || low >= (1 << 32)`: `carry = low >> 32`; emit `(cache as u64 + carry) as u8` via the sink (the sum is 0 or 1, and the low 8 bits are the byte); emit `(0xFF + carry) as u8` exactly `cache_size - 1` times; `cache = ((low >> 24) & 0xFF) as u8`; `cache_size = 0`. Then, on both paths, `cache_size += 1` and `low = (low & 0x00FF_FFFF) << 8`. Cache is bits 24..31 of `low`, not the carry and not `(low as u8)`.
  - [x] The sink drops only the first byte it is asked to emit. Later bytes from that same `shift_low` are stored. Implement it as a `dropped_leading` flag, not `Vec::remove(0)`, not `out[1..]`, and not by skipping a whole `shift_low` call. Bit 1 / freq0 4095 is the guard: the stored stream is `FF EF F0 01 00`, which starts with `FF`.
  - [x] `pub fn finish(self) -> Vec<u8>`. Call `shift_low` five times and return `out`. `finish` does not return `Result`. It does not check the dropped byte.
  - [x] `pub fn state(&self) -> EncoderState` where `EncoderState` is `Copy` and holds `low`, `range`, `cache`, `cache_size`. Rustdoc: trace observation, not a wire field.
  - [x] Rustdoc on `encode`: `freq0` `0` and `4096` fit in `u16` and are rejected; a value above `u16::MAX` does not fit. Bit `2` fits in `u8` and is rejected. Total is 4096 and is not an argument. `bound` fits in `u32`. `range << 8` runs only while `range < 2^24`. `low` stays below `2^33`.
  - [x] No `unsafe`. No new dependency. No float. No `wrapping_*` on `low`, `range`, or `bound`. No model field, no `p`, no rANS function, no Block length.
- [x] **Decoder** (AC: 1, 5, 6)
  - [x] `pub struct RangeDecoder<'a>` with private `code: u32`, `range: u32`, `input: &'a [u8]`, `pos: usize`. Derive `Debug` only.
  - [x] `pub fn new(input: &'a [u8]) -> Result<Self, Error>`. If `input.len() < 4`, return `Error::TruncatedCoder` and do not look at the bytes. Otherwise assemble `code` with `get` and shifts: `b0` is the most significant byte. If `code == u32::MAX`, return `Error::SaturatedRangeCode`. `range = 0xFFFF_FFFF`, `pos = 4`.
  - [x] `pub fn decode(&mut self, freq0: u16) -> Result<u8, Error>`. The same `freq0` rejection as the encoder, before any write, leaving `state()` unchanged. `bound` is the same product. If `code < bound`: bit `0`, `range = bound`. Else: bit `1`, `code -= bound`, `range -= bound`. Then the same `while range < 0x0100_0000` loop, in this call, not deferred to the next `decode`: `range <<= 8`, read one byte, `code = (code << 8) | u32::from(byte)`. A missing byte is `Error::TruncatedCoder`. After that error the decoder is unusable; do not define rollback. `code < range < 2^24` at the shift, so `code << 8` fits in `u32`.
  - [x] `pub fn finish(&self) -> Result<(), Error>`. If `code != 0`, return `Error::NonzeroRangeCode`. Otherwise `Ok(())`. Do not read input. Do not require `pos == input.len()`.
  - [x] `bytes_consumed` is `pos`. After a successful `finish`, `pos` equals the encoded length of that schedule: 4 plus the number of refill bytes. `pub fn state(&self) -> DecoderState` holds `code`, `range`, `bytes_consumed`.
  - [x] Refill is eager. Do not copy an LZMA decoder that renormalizes at the start of the next symbol. A lazy refill can still round-trip and still fail `bytes_consumed`.
- [x] **Schedule wrappers** (AC: 1, 2)
  - [x] `pub fn encode_schedule(decisions: &[(u8, u16)]) -> Result<Vec<u8>, Error>` builds one `RangeEncoder`, encodes each pair, and returns `finish`.
  - [x] `pub fn exact_size(decisions: &[(u8, u16)]) -> Result<usize, Error>` is `encode_schedule(decisions).map(|bytes| bytes.len())`, or the same encoder writing into a counter. It is not `4 + shift_count` and it is not a log2 estimate.
  - [x] `pub fn decode_schedule(input: &[u8], freq0s: &[u16]) -> Result<Vec<u8>, Error>` calls `new`, then `decode` for each `freq0`, then `finish`. If `bytes_consumed() < input.len()` after a successful `finish`, return the existing `Error::TrailingBytes`. Concatenated frames are not this function's job.
  - [x] All three call the types above. Do not write a second encoder for `exact_size`.
- [x] **Errors** (AC: 2, 5)
  - [x] Add `TruncatedCoder`, `SaturatedRangeCode`, and `NonzeroRangeCode` to `Error`, and an arm for each in `name()`. `Display` stays the variant name. Do not add `thiserror`. Do not reuse `TruncatedPayload` for a short coder buffer.
  - [x] In `lib.rs` add `mod range_coder;` and re-export `RangeEncoder`, `RangeDecoder`, `EncoderState`, `DecoderState`, `encode_schedule`, `decode_schedule`, and `exact_size`. The module-doc sentence may name the binary range coder. The G0 hash stays `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`. Do not edit the lint block.
  - [x] Do not edit `tests/support/mod.rs` or `spec/vectors/index.toml`. The support loader asserts 24 rows.
- [x] **Pinned streams** (AC: 2, 3)
  - [x] Add `crates/sxp-kernel/tests/range_coder.rs`. Do not `mod support`.
  - [x] For each row, `encode_schedule` equals the bytes, `exact_size` equals that length, and `decode_schedule` equals the bits. `bytes_consumed()` after `RangeDecoder::finish` equals that length. Pairs are `(bit, freq0)`.

    | case | decisions | bytes |
    |---|---|---|
    | empty | none | `00 00 00 00` |
    | bit 0 / 2048 | `(0, 2048)` | `00 00 00 00` |
    | bit 1 / 2048 | `(1, 2048)` | `7F FF F8 00` |
    | bit 0 / 1 | `(0, 1)` | `00 00 00 00 00` |
    | bit 1 / 1 | `(1, 1)` | `00 0F FF FF` |
    | bit 0 / 4095 | `(0, 4095)` | `00 00 00 00` |
    | bit 1 / 4095 | `(1, 4095)` | `FF EF F0 01 00` |
    | two shifts | `(0, 1), (0, 1)` | `00 00 00 00 00 00 00` |
    | carry | `(0, 1), (1, 2048), (0, 1), (1, 2047)` | `00 08 00 37 F8 00 00` |

  - [x] Bit 0 / 1 consumes 5 bytes. After that single `decode(1)`, `range == 0x0FFF_FF00`, `code == 0`, `bytes_consumed() == 5`. An implementation that refills on the next call reports 4 here and is wrong.
  - [x] Two shifts: the second decision shifts twice. The stored stream is 7 bytes. `bytes_consumed()` is 7. Decoding with one `freq0` of 1 against this buffer is not the assertion; pass both frequencies.
  - [x] Bit 1 / 4095 starts with `FF`. That byte is the pending byte from the same `shift_low` that emitted the dropped `0x00`. Dropping the whole call yields a stream that does not start with `FF`.
  - [x] Carry: `decode_schedule` returns bits `0, 1, 0, 1` and consumes 7. On flush, `low` is `0x1_37F8_0000`, `cache` is `0x07`, and `cache_size` is 2, so the sink emits `0x08` and then `0x00`. The `0x00` is `(0xFF + 1) & 0xFF`, the pending `0xFF` plus a carry. Assert the bytes, not a carry counter.
- [x] **Trace** (AC: 6)
  - [x] Schedule `(1, 1), (0, 4095), (1, 2048)` encodes to `7F FF FF FF`. After each `encode`, before `finish`, encoder `range` is `0xFFF0_0000`, then `0xFFE0_0100`, then `0x7FF0_0100`. Decoder `code` / `range` / `bytes_consumed` after each `decode` are `(0x7FF0_0000, 0xFFF0_0000, 4)`, `(0x7FF0_0000, 0xFFE0_0100, 4)`, `(0, 0x7FF0_0100, 4)`. Encoder `range` equals decoder `range` at each row. `finish` succeeds. There is no model argument on `encode` or `decode`.
  - [x] Build a second encoder after the first `finish`. Its bit-1 / 2048 stream is `7F FF F8 00`. The first encoder's state is not an input.
- [x] **Rejection** (AC: 2, 5)
  - [x] Encoder: `freq0` `0`, `4096`, and `u16::MAX`; bit `2` and bit `255`. Each returns `Error::InvalidParameter`. `state()` is unchanged. `finish` after a rejected `encode` on a fresh encoder is the empty stream `00 00 00 00`. `encode(1, 2048)` after a rejected call on a fresh encoder is `7F FF F8 00`.
  - [x] Decoder on `00 00 00 00`: `decode(0)` and `decode(4096)` return `Error::InvalidParameter` and leave `state()` unchanged. A following `decode(2048)` returns `0`, and `finish` is `Ok`.
  - [x] `new` of `FF FF FF` is `TruncatedCoder`. `new` of `FF FF FF FF` is `SaturatedRangeCode`. `decode_schedule` of those four bytes with an empty `freq0` list is `SaturatedRangeCode`, not `Ok`.
  - [x] `decode_schedule(&[0x00, 0x00, 0x00, 0x01], &[2048])` is `NonzeroRangeCode`.
  - [x] `decode_schedule(&[0x00, 0x00, 0x00, 0x00, 0x00], &[2048])` is `TrailingBytes`.
  - [x] For every pinned stream, every proper prefix passed to `decode_schedule` with that stream's `freq0` list returns `TruncatedCoder`.
- [x] **Concatenation** (AC: 5)
  - [x] Concatenate bit 1 / 2048 (`7F FF F8 00`) and bit 0 / 1 (`00 00 00 00 00`). `RangeDecoder` on the buffer, one `decode(2048)`, then `finish`, returns bit `1` and `bytes_consumed() == 4`. A second `RangeDecoder` on the remainder returns bit `0`, consumes 5, and `finish` is `Ok`. `decode_schedule` of the whole buffer with `&[2048]` is `TrailingBytes`.
  - [x] The same two frames in the other order: first consume 5, then 4.
- [x] **2048-symbol hash** (AC: 4)
  - [x] Commit `spec/vectors/range-coder-2048.sha256` containing `fd9dbd2f270428e94800a34b5b35ca26a2fc6aaeb53f07257a31347a0d9168bf` and one trailing newline. Load it with `include_str!(...).trim()`. Do not add a `[[vector]]` row.
  - [x] Build the schedule in the test: for `i` in `0..2048`, `symbol = (i as u8)`; for `k` in `0..8`, bit `(symbol >> (7 - k)) & 1`, and `freq0 = 1 + ((decision_index as u16) % 4095)` with `decision_index` starting at 0. 16384 decisions. `encode_schedule` has length 3321. Byte 18 is `0x13`. Byte 100 is `0xCE`. Byte 1000 is `0x78`. The last eight bytes are `5E 3E FF FF CE 03 4A A1`. SHA-256 of the whole stream matches the file. `decode_schedule` returns the same bits and consumes 3321.
  - [x] Hash with `sha2 = "=0.11.1"`. Hex with `write!(s, "{b:02x}")`. The script in Dev Notes is the authority if the file and the test disagree. Do not regenerate the file from a broken encoder to make the test pass.
- [x] **Property tests** (AC: 2, 4)
  - [x] `proptest = "=1.12.0"`, `default-features = false`. Config: `cases: 256`, `rng_seed: RngSeed::Fixed(0x5111_0006)`, `failure_persistence: Some(Box::new(FileFailurePersistence::Off))`, remaining fields from `Config::default()`. Do not change seeds `0x5111_0001` through `0x5111_0005`. No regex strategies.
  - [x] Draw `0..=48` decisions, each bit in `0..=1` and `freq0` in `1..=4095`. `decode_schedule` of `encode_schedule` returns those bits and consumes the whole buffer. `exact_size` equals the buffer length. `bytes_consumed()` after `finish` equals that length.
- [x] **Both platforms** (AC: 6)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands, `git rev-parse HEAD`, and that the 2048-symbol hash test passed. Existing recurrence, trigonometric, polynomial, residual, and whole-file raw tests must still pass.
  - [x] `aarch64-unknown-linux-gnu` links with `aarch64-linux-gnu-gcc` and runs under `qemu-aarch64` (`.cargo/config.toml`). If those are not on `PATH`, put `/tmp/serexpai-cross/bin` ahead of `PATH`. Do not edit `.cargo/config.toml`.
  - [x] Leave this subtask unchecked if either target did not run. Do not set the story Status to `done`. `dev-story` sets `review`. Code review sets `done` only when both runs are recorded.
  - [x] Do not edit `bitstream-spec-seed.md`, `CHANGELOG.md`, the workspace version, `polynomial.rs`, `trigonometric.rs`, `recurrence.rs`, `residual.rs`, `raw.rs`, or story 1.1–1.5 tests. Do not commit unless the user asks.

### Review Findings

- [x] [Review][Patch] Truncated refill leaves the decoder usable [crates/sxp-kernel/src/range_coder.rs:201]
- [x] [Review][Patch] Required rustdoc sentences are on the struct, not on encode [crates/sxp-kernel/src/range_coder.rs:62]

## Dev Notes

### What this story builds

One binary range coder in `sxp-kernel`. The caller supplies each bit and each `freq0`. The coder returns the bytes, the exact byte length, and the bits. It does not adapt a probability, frame a Block, or hash a Formula.

Out of scope: the order-0 model and `freq0 = max(p >> 4, 1)` (story 1.7); `encode_block` (story 1.8); rANS (seed Appendix A); a Python reference decoder (amendment 6, G4); `spec/bitstream.md` (story 2.1). The coder does not know how many decisions a Block has. The caller passes the `freq0` list. That list is what ends the frame.

### Procedure

Governing text is seed §6.1, which elaborates AD-7. The stream is a byte string, most significant byte first. It is not a little-endian integer.

```
encode(bit, freq0):
    bound = (range >> 12) * freq0
    if bit == 0: range = bound
    else:        low = low + bound; range = range - bound
    while range < 2^24:
        range = (range << 8) mod 2^32
        shift_low()

shift_low():
    if low < 0xFF000000 or low >= 2^32:
        carry = low >> 32
        emit((cache + carry) & 0xFF)
        repeat cache_size - 1 times: emit((0xFF + carry) & 0xFF)
        cache = (low >> 24) & 0xFF
        cache_size = 0
    cache_size = cache_size + 1
    low = (low & 0x00FFFFFF) << 8

flush: shift_low() five times
```

The sink's first `emit` is discarded. Every later `emit` is stored, including the second byte of the call that produced the discarded byte. After `shift_low`, `low < 2^32`. `(low as u32) << 8` is the same value; `(low << 8)` without the 24-bit mask is not, once `low >= 2^32`.

Decoder, eager refill:

```
code = first four bytes, MSB first
range = 0xFFFFFFFF
reject code == 0xFFFFFFFF
for each freq0:
    bound = (range >> 12) * freq0
    if code < bound: bit = 0; range = bound
    else:            bit = 1; code = code - bound; range = range - bound
    while range < 2^24:
        range = (range << 8) mod 2^32
        code  = ((code << 8) | next_byte()) mod 2^32
reject unless code == 0
```

`bytes_consumed` after `finish` is 4 plus the number of `next_byte` calls. The omitted encoder byte is not part of `input`.

### 2048-symbol oracle

Run from the repo root only if the Rust digest disagrees with the pin. Stdout is the digest. Do not copy this script into the kernel.

```python
import hashlib

def encode(decisions):
    low, rng, cache, cache_size, out = 0, 0xFFFFFFFF, 0, 1, []
    def shift_low():
        nonlocal low, cache, cache_size
        if low < 0xFF000000 or low >= 2**32:
            carry = low >> 32
            out.append((cache + carry) & 0xFF)
            for _ in range(cache_size - 1):
                out.append((0xFF + carry) & 0xFF)
            cache = (low >> 24) & 0xFF
            cache_size = 0
        cache_size += 1
        low = (low & 0x00FFFFFF) << 8
    for bit, freq0 in decisions:
        bound = (rng >> 12) * freq0
        if bit == 0:
            rng = bound
        else:
            low += bound
            rng -= bound
        while rng < 2**24:
            rng = (rng << 8) & 0xFFFFFFFF
            shift_low()
    for _ in range(5):
        shift_low()
    assert out[0] == 0
    return bytes(out[1:])

decisions = []
for i in range(2048):
    symbol = i & 0xFF
    for k in range(8):
        decisions.append(((symbol >> (7 - k)) & 1, 1 + (len(decisions) % 4095)))
raw = encode(decisions)
assert len(raw) == 3321
assert raw[18] == 0x13 and raw[100] == 0xCE and raw[1000] == 0x78
assert raw[-8:].hex() == "5e3effffce034aa1"
print(hashlib.sha256(raw).hexdigest())
```

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable` on `src/`. Tests may use `assert`. Read input with `get`.

Public surface that must keep its signatures: `read_whole_file_raw`, `write_whole_file_raw`, `encode_whole_file_raw_header`, `decode_header`, `Header`, `HeaderBytes`, `encode_leb128`, `decode_leb128`, `Leb128`, `LENGTH_LIMIT`, `Crc32`, `crc32_iso_hdlc`, `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, `Polynomial`, `Trigonometric`, `check_sine_lut`, `Recurrence`, and every existing `Error` variant plus `Error::name`.

`sxp-kernel` has an empty `[dependencies]` table. `ci.sh` fails unless `cargo tree -e normal` and `-e build` for `sxp-kernel` are each one line, and unless `crates/sxp-kernel/build.rs` is absent. `sha2` and `proptest` are already dev-dependencies.

`Error::Display` is the variant name only.

### Project Structure Notes

- Coder: `crates/sxp-kernel/src/range_coder.rs`.
- Errors: `crates/sxp-kernel/src/error.rs`, three variants and three `name()` arms.
- Export: `crates/sxp-kernel/src/lib.rs`, one `mod` and the `pub use` list above.
- Tests: `crates/sxp-kernel/tests/range_coder.rs`.
- Hash: `spec/vectors/range-coder-2048.sha256`.
- Do not put the coder in `sxp-encoder`. Do not fold it into `raw.rs`.

### Previous story intelligence

Story 1.5 put the recurrence evaluator in its own module, exported it with `pub use`, and tested it from `tests/` without `mod support`. The hash artifact is a one-line SHA-256 file, not an `index.toml` row. `Error::InvalidParameter` is the domain rejection. Integration tests own `proptest` and `sha2`. The next free proptest seed is `0x5111_0006`.

Story 1.5's review patches were doc precision and tests that missed a representable neighbour. Reject `freq0` 0 and 4096, and bit 2, on the value that fits in the parameter type. Prefer the pinned bytes over a round-trip that would also pass for a different coder. A fixture that can pass with the leading `FF` of bit 1 / 4095 dropped will be rejected.

The evaluators are not inputs to this module. Do not call `residual_symbol`, `Polynomial`, `Trigonometric`, or `Recurrence`.

### Git intelligence

`HEAD` is `aa2fb8fdbee623bceb1123db7dd8010d87a7c579`, `feat: evaluate a recurrence formula (v0.0.11)`. Stories 1.1–1.5 are in that history. There is no coder module to extend. Recent titles:

- `aa2fb8f` feat: evaluate a recurrence formula (v0.0.11)
- `277f5a1` feat: evaluate a trigonometric formula (v0.0.10)
- `09f0f51` feat: evaluate a polynomial formula (v0.0.9)
- `fb5a3fd` feat: round-trip whole-file raw and reconstruct predictions (v0.0.8)
- `5a58bbb` Add the pinned workspace and whole-file raw fixtures.

### Latest technical information

Stay on the pinned toolchain `1.99.1` (`rust-toolchain.toml`). Do not `rustup update`. `u32` multiply of `(range >> 12)` by `freq0` overflows once `freq0` is unvalidated (`u16::MAX`); validate first, then multiply in `u64`. `u32 << 8` shifts bits off the top of the register; that shift is exact only while the value is `< 2^24`, which is the loop condition. `u64` addition of `bound` is what keeps the carry. Do not add a crate. `proptest 1.12.0` and `sha2 0.11.1` are already pinned.

### Project context reference

No `project-context.md` exists in this tree. The kernel lint block in `lib.rs` and AD-7 in the architecture spine are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.6, FR-8, acceptance evidence, elicitation amendments 1, 3, and 6]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §6, §6.1]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-7]
- [Source: `_bmad-output/implementation-artifacts/1-5-evaluate-a-recurrence-formula.md` — module export, hash artifact, proptest profile, both-platform gate]
- [Source: `crates/sxp-kernel/src/lib.rs` — lint block and G0 hash]
- [Source: `crates/sxp-kernel/src/error.rs` — `Error::name`]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created
- `RangeEncoder` / `RangeDecoder` in `sxp-kernel` implement seed §6.1. `low` is `u64`, refill is a `while` inside the decision, and the byte sink drops only the first emitted byte. `exact_size` is the length of `encode_schedule`. `freq0` outside `1..=4095` and a bit above 1 are `Error::InvalidParameter` and leave `state()` unchanged. New errors are `TruncatedCoder`, `SaturatedRangeCode`, and `NonzeroRangeCode`. `decode_schedule` returns `TrailingBytes` when input remains. `RangeEncoder` implements `Default` because `clippy::new_without_default` is denied.
- Pinned streams, the carry schedule `00 08 00 37 F8 00 00`, the two-shift 7-byte stream, and the 2048-symbol SHA-256 `fd9dbd2f270428e94800a34b5b35ca26a2fc6aaeb53f07257a31347a0d9168bf` (3321 bytes) pass. `symbol_fixture_matches_the_committed_hash` passed on both targets.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-01, including the range-coder hash test and the existing polynomial, trigonometric, recurrence, residual, and whole-file raw tests.
- `./ci.sh aarch64-unknown-linux-gnu` passed on 2026-11-01 under `qemu-aarch64`, including the same hash test. Linker and runner came from `/tmp/serexpai-cross/bin` ahead of `PATH`. `HEAD` is still `aa2fb8fdbee623bceb1123db7dd8010d87a7c579`. This story is uncommitted.
- Code review patches applied: a truncated refill leaves `range < 2^24`, so a later `decode` and `finish` return `TruncatedCoder`, and the `encode` rustdoc states the bound, shift, and `low` limits. Both CI targets were rerun after those patches and passed, including `symbol_fixture_matches_the_committed_hash`.

### File List

- `crates/sxp-kernel/src/range_coder.rs`
- `crates/sxp-kernel/src/error.rs`
- `crates/sxp-kernel/src/lib.rs`
- `crates/sxp-kernel/tests/range_coder.rs`
- `spec/vectors/range-coder-2048.sha256`

### Change Log

- 2026-11-01: Binary range coder, exact size, pinned streams, carry fixture, and both-platform 2048-symbol hash.
- 2026-11-01: Addressed code review findings — 2 patches applied.

## Story completion status

done. Both target CI runs passed after the review patches, including the range-coder hash test and the existing evaluator and whole-file raw tests. The probability model and Block framing stay with later stories.
