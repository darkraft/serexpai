---
baseline_commit: c05a8b00ca6e892e115c451890ab65fa950d111f
---

# Story 1.8: Round-trip one baseline block

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want Raw, Direct, Previous-byte, and Constant Blocks to share one framing and coding interface,
so that a blocked file can round-trip one baseline Block, and later Formula kinds can use the same function without a second framer.

## Acceptance Criteria

1. **S1.8-AC01.** D-02 (approved 2026-11-01) is the layout: a 2-bit length code and a 3-bit kind, LSB first, then kind-specific fields, then checked zero padding to the next byte, then a byte-aligned payload. Commit the header, field, and padding bytes below before `encode_block` exists. Do not edit `bitstream-spec-seed.md`. The seed still tags the kind numbers **[PROPOSED]**; this story uses that table anyway, because it is the only published assignment and story 1.10 already names wire kinds 4, 5, and 6. Record that as an accepted risk. Do not retag the seed.

2. **S1.8-AC02.** One `encode_block` handles Raw, Direct, Previous-byte, and Constant. Raw copies bytes and does not call the model, the coder, or the residual functions. Direct calls `direct_symbol` and then the strawman model and a fresh range coder. Previous-byte predicts 0, then the previous reconstructed byte. Constant predicts its signalled byte at every index. Each Block builds a new `Order0Model` from `ModelConfig::strawman()` and a new `RangeEncoder`. `BlockKind` has those four variants only.

3. **S1.8-AC03.** Nominal lengths are 256, 1024, and 2048. Actual length is `min(nominal, remaining)`. A nominal length is canonical when it is `<= remaining`, and, if `remaining <= 2048`, also when it is the single smallest nominal `>= remaining`. `decode_block` takes the Block bytes and `remaining`. It does not take a previous Block. `encode_block` requires `bytes.len() == actual`.

4. **S1.8-AC04.** Exact vectors cover every baseline kind at every canonical length code, the partial finals, and actual lengths 1, 255, 256, 257, 1023, 1024, 1025, 2047, and 2048 where that length code is canonical. A multi-Block file consumes each payload exactly, and the next header or the CRC trailer is still there.

5. **S1.8-AC05.** Reject reserved length code 3, reserved kind 7, wire kinds 4, 5, and 6, a non-canonical nominal, non-zero padding, a truncated header or raw payload, a truncated coder payload, blocked mode with `N = 0`, a bad file CRC, and trailing bytes. The coded payload length is `RangeDecoder::bytes_consumed`, not a stored field. Check order is pinned below.

6. **S1.8-AC06.** Replacing the bytes of an earlier Block does not change this Block's predictions, model state, encoded payload, or how many bytes this Block consumes. Both-platform `./ci.sh` passes. A blocked file whose `N` is 1 MiB of 256-byte Raw Blocks stays within 64 KiB of decoder-owned heap, and that peak is not higher than the peak for a one-Block file.

## Tasks / Subtasks

- [x] Commit header and file fixtures before the encoder (AC: 1)
  - [x] Add `spec/vectors/block-baseline-headers.hex` with the lines in Dev Notes, and `spec/vectors/block-raw-n1.bin` plus `spec/vectors/block-raw-n2.bin`
  - [x] Do not add rows to `spec/vectors/index.toml`
- [x] Add the block errors and `crates/sxp-kernel/src/block.rs` (AC: 1, 2, 5)
  - [x] Pack and unpack the section. Constant `c` straddles the two section bytes
  - [x] `encode_block` / `decode_block` with the pinned check order
  - [x] Export the new types from `lib.rs`. Extend `Error::name`
- [x] Pin the coded payloads and the canonical matrix (AC: 2, 3, 4)
  - [x] Direct `0xA5`, Previous-byte `01 00`, Constant `c = 1` over `01 00`, and one Raw byte
  - [x] Every kind × every canonical cell in the length table, including partial finals
- [x] Rejection tests (AC: 5)
  - [x] `0x1F` is `ReservedLength`. Kind 4 with a non-canonical length code is `UnsupportedKind`
  - [x] Constant padding `0c 20`. Truncated header, truncated raw, truncated coder
  - [x] `write_blocked_file(0, …)` and a mode-0 `N = 0` header are `BlockedEmpty` and write nothing
- [x] File reader and writer (AC: 4, 5, 6)
  - [x] `write_blocked_file` validates every plan, then writes. A structural plan error leaves the sink empty
  - [x] `read_blocked_file` streams one window. Mode 1 returns `UnexpectedMode`
  - [x] Two full 256-byte Raw Blocks, and a 256-byte Block followed by a 1-byte final Block
  - [x] Mutating Block 1 does not change Block 2's encoded bytes
- [x] Heap and both targets (AC: 6)
  - [x] `tests/block_heap.rs` with its own counting allocator. `N = 256` and `N = 1 MiB`
  - [x] Adversarial 2048-byte coded Block fits in `MAX_ENCODED_BLOCK`
  - [x] `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`

### Review Findings

- [x] [Review][Patch] The blocked reader returns writer-only `BlockLengthMismatch` [crates/sxp-kernel/src/block.rs:307]
- [x] [Review][Patch] `block-raw-n2.bin` is written and never read back [crates/sxp-kernel/tests/block.rs:302]
- [x] [Review][Patch] Rejection tests never send wire kinds 5 and 6, or padding bits 6 and 7 [crates/sxp-kernel/tests/block.rs:205]
- [x] [Review][Patch] `BLOCK_WINDOW` comment does not derive 36864 [crates/sxp-kernel/src/block.rs:28]

## Dev Notes

### What this story builds

The kernel learns to frame one baseline Block and to read and write a mode-0 file made of those Blocks. The caller supplies every plan. This story does not choose a kind or a length.

Out of scope, do not implement: Candidate search, cost comparison, an `exact_size` selector, DP partitioning (story 1.9); Formula field parsers or writers, `BlockKind` variants other than the four below, and any call from block code into `polynomial`, `trigonometric`, or `recurrence` (story 1.10); CLI, temp-file rename, destination rollback (story 1.11); rANS; a Python reference decoder; `spec/bitstream.md`; editing the spec seed; changing the strawman `p`, `CAP`, or RECIP; new crates; new `index.toml` rows.

`read_whole_file_raw` and `write_whole_file_raw` stay as they are. Mode 0 through `read_whole_file_raw` is still `BlockedModeUnsupported`, and the existing test `blocked_mode_with_payload_writes_nothing` must keep passing. `encode_whole_file_raw_header` keeps its signature. Do not give it a mode argument.

### Wire layout

LSB first inside each byte. A field that crosses a byte continues in the next byte's low bits. Padding bits are zero.

Length code in bits 0–1: `0 = 256`, `1 = 1024`, `2 = 2048`, `3 = reserved`. Kind in bits 2–4: `0` Raw, `1` Direct, `2` Previous-byte, `3` Constant, `4` Polynomial, `5` Trigonometric, `6` Recurrence, `7` reserved.

Raw, Direct, and Previous-byte have no extra field. The section is one byte: `length_code | (kind << 2)`. Bits 5–7 are padding. Constant adds `c` as the next 8 bits, so the section is two bytes and bits 13–15 (bits 5–7 of byte 1) are padding.

```
byte0 = lc | (kind << 2) | ((c & 0x07) << 5)     # c only for Constant
byte1 = (c >> 3) & 0x1F                          # Constant only
```

Pinned headers, also the body of `spec/vectors/block-baseline-headers.hex`, one `name hex` pair per line, lowercase hex, no `0x`:

```
raw-256 00
raw-1024 01
raw-2048 02
direct-256 04
direct-1024 05
direct-2048 06
previous-256 08
previous-1024 09
previous-2048 0a
constant-0 0c00
constant-1 2c00
constant-8 0c01
constant-ff ec1f
reserved-length 03
reserved-kind 1c
length-and-kind-reserved 1f
padding-raw 20
padding-constant 0c20
kind-polynomial 10
kind-polynomial-noncanonical-length 11
```

`0x11` is kind 4, length code 1. `0x1c` is kind 7, length code 0. `0x1f` is length code 3 and kind 7. `0x20` is Raw with padding bit 5 set. `0c 20` is Constant `c = 0` with padding bit 5 of byte 1 set.

### Canonical length

`actual = min(nominal, remaining)`. Allowed nominals: every nominal `<= remaining`, plus, when `1 <= remaining <= 2048`, the smallest nominal that is `>= remaining`. `remaining == 0` allows nothing.

| remaining | 256 | 1024 | 2048 |
| --- | --- | --- | --- |
| 1 | actual 1 | reject | reject |
| 255 | 255 | reject | reject |
| 256 | 256 | reject | reject |
| 257 | 256 | 257 | reject |
| 1023 | 256 | 1023 | reject |
| 1024 | 256 | 1024 | reject |
| 1025 | 256 | 1024 | 1025 |
| 2047 | 256 | 1024 | 2047 |
| 2048 | 256 | 1024 | 2048 |
| 2049 and above | 256 | 1024 | 2048 (full, not a partial) |

There is no partial Block larger than 2048. Two Blocks cannot cover `N = 2`: the first Block's actual length would already be 2. The shortest two-Block file is `N = 257`: a full 256, then a final 1. A two-full-Block file is `N = 512`.

### Check order

First match wins. Do not look at a later rule after one fails.

1. No first section byte → `TruncatedBlock`.
2. Length code 3 → `ReservedLength`. This wins over kind, so `0x1F` is `ReservedLength`.
3. Kind 7 → `ReservedKind`.
4. Kind 4, 5, or 6 → `UnsupportedKind`. This wins over canonicality, so `0x11` with `remaining = 1` is `UnsupportedKind`. Do not read Formula fields.
5. Constant and the second section byte is absent → `TruncatedBlock`.
6. Any padding bit set → `NonZeroPadding`.
7. Nominal not canonical for `remaining` → `NonCanonicalLength`.
8. Raw payload short → `TruncatedBlock`.
9. Coder errors pass through: `TruncatedCoder`, `SaturatedRangeCode`, `NonzeroRangeCode`.

`BlockLengthMismatch` is writer-only. The decoder cannot produce it: actual length never exceeds `remaining`. Use it when a plan's slice length is not `actual`, or the plans' actual lengths do not sum to `N`. `BlockTooLarge` is the writer guard below. `UnexpectedMode` is mode 1 passed to `read_blocked_file`. `BlockedEmpty` stays the mode-0 `N = 0` error. `Display` is the variant name only.

### Coding path

```
Raw:     section || bytes
Direct:  symbol = direct_symbol(y)          # do not call residual_symbol
Prev:    yhat = 0; symbol = residual_symbol(y, yhat); yhat = y
Const:   symbol = residual_symbol(y, c)
then for the three coded kinds:
  model = Order0Model::new(ModelConfig::strawman()?)
  encoder = RangeEncoder::new()
  model.encode_symbol(&mut encoder, symbol) for each byte
  payload = encoder.finish()
```

Decode uses `RangeDecoder::new` on the bytes after the section. Those bytes may include the next header and the CRC. `finish` allows leftover input. `consumed = section_len + decoder.bytes_consumed()`. Reconstruct Previous-byte and Constant with `reconstruct`. Direct's symbol is the byte. A new model per Block is the reset. Do not reuse coder state.

`residual(y, 0) == y`, so the first Previous-byte symbol matches Direct. Later symbols diverge once `yhat != 0`. The `01 00` vector is that case.

Pinned payloads, from the story 1.7 coder and the strawman update:

| Case | Bytes |
| --- | --- |
| Direct, one byte `0xA5`, length 256, remaining 1 | `04 a4 ff f8 00` |
| Previous-byte, source `01 00`, length 256, remaining 2 | symbols `01 ff`, block `08 01 ff 78 00 00 00` |
| Constant `c = 1`, source `01 00`, length 256, remaining 2 | symbols `00 ff`, block `2c 00 00 ff 78 04 00 00` |
| Raw, one byte `0x8D`, length 256, remaining 1 | `00 8d` |

File fixtures, mode 0, version 1, CRC-32/ISO-HDLC of the decoded bytes, little-endian trailer:

- `spec/vectors/block-raw-n1.bin`: `8d 01 01 00 8d 10 10 0b 41`. `N = 1`, one Raw Block, payload `0x8D`.
- `spec/vectors/block-raw-n2.bin`: `8d 01 02 00 11 22 0b 70 60 c7`. `N = 2`, one Raw Block, payload `11 22`. Not two Blocks.

Build the two-Block cases in the test. Do not paste a 256-byte hex fixture. First Block header `00`, then 256 payload bytes, then the next header. After `decode_block`, the first unconsumed byte is that next header, or the first CRC byte when the Block is last.

### Window bound

Post-decision `range` is at least 4096 when the pre-decision `range` is at least `2^24` and `freq0` is in `1..=4095`:

- bit 0: new range is `(range >> 12) * freq0 >= range >> 12 >= 4096`
- bit 1 at `freq0 = 4095`: new range is `(range & 4095) + (range >> 12) >= 4096`

Renormalisation shifts while `range < 2^24`. `4096 << 8 = 2^20`, and one more shift reaches `2^28`. At most two shifts per decision. A carry flush emits bytes that earlier shifts deferred, so total emitted bytes are at most the number of `shift_low` calls. Decisions: `2048 * 8 = 16384`. Termination: 5 more shifts. The encoder drops the first emitted byte. Wire payload `<= 16384 * 2 + 5 - 1 = 32772`. Section `<= 2`. Encoded Block `<= 32774`.

```
MAX_ENCODED_BLOCK = 32774
BLOCK_WINDOW = 36864
```

`BLOCK_WINDOW` leaves room for a 4-byte trailer after the largest Block. Put the proof next to the constants.

`write_blocked_file` encodes each Block into a buffer first. If that buffer's length exceeds `MAX_ENCODED_BLOCK`, return `BlockTooLarge` and do not write that Block. An earlier Block may already be in the sink. The adversarial test below must fit, so this error is the guard, not an expected outcome of a legal baseline Block.

`read_blocked_file` keeps one `Vec` window of capacity `BLOCK_WINDOW`. Fill it, call `decode_block` on the window, write the decoded bytes, then move the unconsumed tail to the front. On `TruncatedBlock` or `TruncatedCoder`, read more if the window is below capacity and the input is not exhausted, then retry. A full window or a real EOF returns that error. Do not allocate with `N`. The caller's sink is not decoder-owned heap.

### File API

```rust
pub enum NominalLength { N256, N1024, N2048 }
pub enum BlockKind { Raw, Direct, PreviousByte, Constant(u8) }

pub struct BlockView {
    pub kind: BlockKind,
    pub nominal: NominalLength,
    pub bytes: Vec<u8>,
    pub consumed: usize,
}

pub fn encode_block(kind: BlockKind, nominal: NominalLength, remaining: u64, bytes: &[u8]) -> Result<Vec<u8>, Error>
pub fn decode_block(input: &[u8], remaining: u64) -> Result<BlockView, Error>

pub struct BlockPlan<'a> {
    pub kind: BlockKind,
    pub nominal: NominalLength,
    pub bytes: &'a [u8],
}

pub fn write_blocked_file(n: u64, plans: &[BlockPlan<'_>], mut output: impl Write) -> Result<(), Error>
pub fn read_blocked_file(mut input: impl Read, mut output: impl Write) -> Result<(), Error>
```

`write_blocked_file` rejects `n >= LENGTH_LIMIT` with `LengthOutOfRange` and an empty sink. It rejects `n == 0` with `BlockedEmpty` and an empty sink. It checks every plan against the running `remaining` before the first `write`. Any `BlockLengthMismatch` or `NonCanonicalLength` leaves the sink empty. Then it writes the mode-0 header (`0x8D`, flags `0x01`, minimal LEB128 `N`), each encoded Block, and `LE32(crc)` of the source bytes in order. Header flags `0x01` are version 1, mode 0, reserved bits clear. Do not call `encode_whole_file_raw_header`.

`read_blocked_file` uses `decode_header`. Mode 1 returns `UnexpectedMode` and writes nothing. Mode 0 with `N = 0` is already `BlockedEmpty`. CRC mismatch and a short trailer may already have written decoded bytes. That matches `read_whole_file_raw`. Do not buffer the file to hide them. Trailing bytes after the CRC are `TrailingBytes`. A CRC mismatch is `CrcMismatch`.

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable` on `src/`. Tests may use `assert`. Index with `get`.

Keep every signature story 1.7 listed, plus `Order0Model`, `ModelConfig`, `encode_symbol`, and `decode_symbol`. `sxp-kernel` still has an empty `[dependencies]` table. No `build.rs`. `sha2` and `proptest` stay dev-dependencies.

New `Error` variants: `ReservedLength`, `ReservedKind`, `UnsupportedKind`, `NonCanonicalLength`, `NonZeroPadding`, `TruncatedBlock`, `BlockLengthMismatch`, `BlockTooLarge`, `UnexpectedMode`. Add them to `name()`.

### Project Structure Notes

- Block framer: `crates/sxp-kernel/src/block.rs`. It may call `header` (`decode_header` only), `leb128`, `crc32`, `residual`, `model`, and `range_coder`.
- `range_coder` must not call `model` or `block`. `model` must not call `block`.
- Tests: `crates/sxp-kernel/tests/block.rs` and `crates/sxp-kernel/tests/block_heap.rs`.
- Fixtures: `spec/vectors/block-baseline-headers.hex`, `spec/vectors/block-raw-n1.bin`, `spec/vectors/block-raw-n2.bin`.
- Do not put framing in `sxp-encoder`.

### Previous story intelligence

Story 1.7 put the model in its own module and left `range_coder.rs` untouched. Hashes are one-line SHA-256 files, not `index.toml` rows. `tests/support/mod.rs` still asserts 24 index rows; new Block fixtures stay out of that index. `Error::Display` is the variant name. Integration tests own `proptest`. Story 1.7 used seed `0x5111_0007`. This story uses `0x5111_0008`.

`encode` and `decode` still take a caller-supplied `freq0`. Do not add a model argument to them. Direct must call `direct_symbol`. The residual test at `tests/residual.rs` already says the Direct path is the caller.

Story 1.7's review wanted rustdoc on the function that does the work, and tests that reach the state the happy path skips. Put the check order and the window proof on `decode_block` and `encode_block`. Test padding in the Constant's second byte, `0x1F`, and kind 4 with a non-canonical length.

### Git intelligence

`HEAD` is `c05a8b0`, `feat: adapt an order-0 probability model (v0.0.13)`. Recent titles:

- `c05a8b0` feat: adapt an order-0 probability model (v0.0.13)
- `d06ecd6` feat: code decisions with the binary range coder (v0.0.12)
- `aa2fb8f` feat: evaluate a recurrence formula (v0.0.11)
- `277f5a1` feat: evaluate a trigonometric formula (v0.0.10)
- `09f0f51` feat: evaluate a polynomial formula (v0.0.9)

The framer is new. The coder and the model are callees. Whole-file raw stays the mode-1 path.

### Latest technical information

Stay on the pinned toolchain `1.99.1` (`rust-toolchain.toml`). Do not `rustup update`. Do not add a crate. Bit packing is shifts and masks, the same approach as `header.rs`. `proptest 1.12.0` and `sha2 0.11.1` are already pinned. No new web API is involved.

### Project context reference

No `project-context.md` exists in this tree. The kernel lint block in `lib.rs`, AD-8, and AD-11 are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.8, S1.8-AC01–AC06]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §1, §2, §3, §4.4, §4.5, §6]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-2, AD-8, AD-9, AD-11, AD-12]
- [Source: `docs/decision_logs/decision_log_2026_W44.md` — D-02, 2026-11-01]
- [Source: `_bmad-output/implementation-artifacts/1-7-adapt-an-order-0-probability-model.md` — model API, proptest seed, both-platform gate]
- [Source: `crates/sxp-kernel/src/raw.rs` — mode 0 still unsupported on the whole-file reader]
- [Source: `crates/sxp-kernel/src/residual.rs` — `direct_symbol`, `residual_symbol`, `reconstruct`]
- [Source: `crates/sxp-kernel/src/range_coder.rs` — `finish` allows leftover input; `bytes_consumed`]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- `N = 512` and `N = 258` use a two-byte LEB128, so file-level tests take the header length from `decode_header` instead of assuming three bytes.
- The second Block sits in front of the CRC trailer, not at the end of the file.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added `encode_block` and `decode_block` for Raw, Direct, Previous-byte, and Constant. Constant `c` straddles the section. Each coded Block uses a new strawman model and a new range coder. Direct calls `direct_symbol`.
- Added `write_blocked_file` and `read_blocked_file`. Mode 0 with `N = 0` stays `BlockedEmpty`. Mode 1 on the blocked reader is `UnexpectedMode`. `read_whole_file_raw` still returns `BlockedModeUnsupported`.
- Pinned header, payload, and file fixtures. The canonical matrix, check order, cross-block isolation, one-byte streaming, and the 2048-byte adversarial block are tested.
- Decoder window is 36864 bytes. `MAX_ENCODED_BLOCK` is 32774. The 1 MiB Raw-block heap peak does not exceed the one-Block peak and stays under 64 KiB.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-03. `git rev-parse HEAD` was `c05a8b00ca6e892e115c451890ab65fa950d111f`.
- `./ci.sh aarch64-unknown-linux-gnu` passed on the same date and the same `HEAD`, under `qemu-aarch64` with `/tmp/serexpai-cross/bin` on `PATH`.
- Code review applied 4 patches. Eight findings were dismissed. Nothing was deferred. After those patches, `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu` passed again at the same uncommitted `HEAD` `c05a8b00ca6e892e115c451890ab65fa950d111f`.

### File List

- crates/sxp-kernel/src/block.rs
- crates/sxp-kernel/src/error.rs
- crates/sxp-kernel/src/lib.rs
- crates/sxp-kernel/tests/block.rs
- crates/sxp-kernel/tests/block_heap.rs
- spec/vectors/block-baseline-headers.hex
- spec/vectors/block-raw-n1.bin
- spec/vectors/block-raw-n2.bin
- _bmad-output/implementation-artifacts/1-8-round-trip-one-baseline-block.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-11-03: Story context created. Kind numbers stay the seed's proposed table, used as published. The two-byte file is one Raw Block.
- 2026-11-03: Implemented baseline Block framing, the mode-0 file reader and writer, pinned fixtures, and both-platform CI.
- 2026-11-03: Addressed code review findings — 4 patches applied, 8 dismissed, none deferred.

## Story completion status

done. Both target CI runs passed after the review patches. The reader no longer returns `BlockLengthMismatch`. Kinds 5 and 6 and padding bits 6 and 7 are rejected in tests. D-07 remains open. Kind numbers remain the seed's proposed table.
