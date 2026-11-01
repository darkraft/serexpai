---
baseline_commit: 5a58bbb011ebe761bb246cc4557587b056b21d9c
---

# Story 1.2: Reconstruct every byte from its prediction

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want every predicted byte to reconstruct the original exactly,
so that prediction quality affects size, never correctness.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or the committed hash must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence is enough for story 1.3 to start (elicitation amendment 1, 2026-11-01). It does not close AC04.

1. **S1.2-AC01.** Exhaustively test all 65,536 pairs `y`, `ŷ` in `0..255`. Compute `δ = ((y − ŷ + 128) mod 256) − 128`, emit `symbol = δ mod 256`, recover signed `δ` as `symbol` when `symbol < 128` and `symbol − 256` otherwise, and reconstruct `y = (ŷ + δ) mod 256`. Modulo is mathematical (non-negative). Rust `%` is not that operation.
2. **S1.2-AC02.** Explicit fixtures cover both wrap directions and `y = 0`, `ŷ = 128` producing `δ = −128` and symbol `128`. Random Block/predictor property tests have zero mismatches.
3. **S1.2-AC03.** Encoder and decoder use the same kernel residual and reconstruction functions. Direct entropy coding emits `y` and does not invoke residual prediction. Story 1.8 owns the `encode_block` call site. This story ships the functions and a contract test only.
4. **S1.2-AC04.** Publish the exhaustive symbol-sequence hash. Pair order is `y` ascending, then `ŷ` ascending, one unsigned symbol byte per pair. Both platforms match the committed artifact.

## Tasks / Subtasks

- [x] **Repair the vector index parser** (AC: 1)
  - [x] `crates/sxp-kernel/tests/support/mod.rs` does not parse. The closure parameter list contains statements, `match key` uses unbound names, `fn assign` sits inside `vectors()`, and `decoded` / `input` arms are duplicated. Story 1.1 tests have not compiled in this tree. Repair this file before any residual code.
  - [x] Parse `spec/vectors/index.toml` from `include_str!`. On `[[vector]]`, flush the pending record. Split a key line with `split_once('=')`, trim both sides, and require a value wrapped in `"`. Strip one pair of quotes. Panic on a non-blank, non-`#` line with no `=`. Panic on a key line before the first `[[vector]]`. Panic on a duplicate key. Panic if a flush has `error`, `decoded`, or `input` set while `id`, `path`, or `kind` is missing. Ignore `note`. After a successful flush the next record starts empty.
  - [x] Keep `#![allow(dead_code)]`. Keep `load`, `decode_hex`, and `named_error` and their match arms unchanged. Do not edit fixture bytes or existing `[[vector]]` rows.
  - [x] Prefer a `Pending` struct with `#[derive(Default)]`, flushed by `std::mem::take`, and a module-level `assign`. Delete the duplicate match arms. `unreachable_patterns` fails `./ci.sh`.
  - [x] Assert `vectors()` returns 24 records and 24 unique ids, matching the 24 `[[vector]]` blocks and the 24 `load` arms.
  - [x] Run `./ci.sh`. If anything other than this parser fails, stop and report it. Do not "fix" story 1.1 kernel code. One known oddity to leave alone: `raw.rs` maps a slice failure to `Error::Leb128Overflow`.
- [x] **Residual and reconstruction** (AC: 1, 3)
  - [x] Add `crates/sxp-kernel/src/residual.rs`. In `lib.rs` add only `mod residual;` and the `pub use`. Do not edit the module docs' G0 hash, the lint block, or any other `pub use`.
  - [x] `pub fn signed_residual(y: u8, prediction: u8) -> i16`. Widen with `i16::from`. Compute `((y − prediction + 128) mod 256) − 128` using `rem_euclid`. The intermediate `y − prediction + 128` is in `−127..=383` and fits in `i16`. Result is in `−128..=127`.
  - [x] `pub fn residual_symbol(y: u8, prediction: u8) -> u8` is exactly `signed_residual(y, prediction).rem_euclid(256) as u8`. Do not use `u8::try_from` (denied `expect` / `unwrap` would hide a bug). The value is already in `0..=255`.
  - [x] `pub fn reconstruct(prediction: u8, symbol: u8) -> u8`. Recover `δ` as `i16::from(symbol)` when `symbol < 128`, otherwise `i16::from(symbol) - 256`. Return `(i16::from(prediction) + δ).rem_euclid(256) as u8`.
  - [x] These two functions are the pair story 1.8 must call: `residual_symbol` on encode, `reconstruct` on decode. Do not add a second residual implementation in `sxp-encoder`.
  - [x] In `residual.rs` only, do not use `%` or `wrapping_add` / `wrapping_sub` / `wrapping_neg`. Tests may use `u8::wrapping_sub` as an independent oracle.
  - [x] No new `Error` variant. The functions are total on `u8`. No `unsafe`. No allocator use.
- [x] **Direct symbol** (AC: 3)
  - [x] `pub fn direct_symbol(y: u8) -> u8` has the body `y`. It does not call `signed_residual`, `residual_symbol`, or `reconstruct`.
  - [x] Test `direct_symbol(y) == y` for every `y`, and `residual_symbol(0, 1) != direct_symbol(0)`. Story 1.8 proves that Direct's `encode_block` path calls `direct_symbol` and does not call `residual_symbol`. Record that split in the acceptance notes. Do not implement `encode_block`.
- [x] **Exhaustive pairs, fixtures, and the hash** (AC: 1, 2, 4)
  - [x] Add `crates/sxp-kernel/tests/residual.rs`. Do not `mod support`.
  - [x] Loop `for y in 0..=255u8` then `for prediction in 0..=255u8`. For every pair assert `residual_symbol(y, prediction) == y.wrapping_sub(prediction)`, `(-128..=127).contains(&signed_residual(y, prediction))`, and `reconstruct(prediction, residual_symbol(y, prediction)) == y`.
  - [x] Build a 65,536-byte buffer in that loop order, index `256 * u16::from(y) + u16::from(prediction)`. Assert the length before hashing.
  - [x] Commit `spec/vectors/residual-symbols.sha256` containing the lowercase hex digest and one trailing newline. The test loads it with `include_str!(...).trim()` and compares. Do not add this hash to `spec/vectors/index.toml`.
  - [x] The digest is SHA-256 over those 65,536 raw bytes, with no header, no hex text, and no newline. Pinned value: `a8abf656d48d4ef997f294870ea52a827fe67197c243d63a6d805db66fbee1f1`. It was produced by `python3 -c "import hashlib;print(hashlib.sha256(bytes((y-p)%256 for y in range(256) for p in range(256))).hexdigest())"`. Prediction-major order (`p` outer, `y` inner) is the different digest `3a65bb13c9ac39a926c4635022fb6b1969db468b8bbd1c0ba9265c0625fbb0b2`. If the test fails, the code is wrong. Do not regenerate the artifact from the implementation.
  - [x] Hash in the test with dev-dependency `sha2 = "=0.11.1"`, already pinned for `sxp-bench`. sha2 0.11's digest is a `hybrid-array` `Array`. Encode hex with `write!(s, "{b:02x}")` over the bytes, or compare raw digest bytes. Do not `format!("{:x}", digest)`. Do not add a `hex` or `base16ct` crate.
  - [x] `sha2` is a dev-dependency only. Then run `cargo update -p sxp-kernel` once. The lock diff may add `"sha2"` to the `sxp-kernel` package dependency list and nothing else. Do not run `cargo generate-lockfile` or a plain `cargo update`. `cargo tree -p sxp-kernel -e normal --target all --prefix none --locked` stays one line. The same command with `-e build` stays one line.
  - [x] Explicit fixtures, each checked for `δ`, symbol, and reconstructed `y`:

    | y | ŷ | δ | symbol |
    |---|---|---|---|
    | 0 | 128 | −128 | 128 |
    | 128 | 0 | −128 | 128 |
    | 0 | 255 | +1 | 1 |
    | 255 | 0 | −1 | 255 |
    | 127 | 0 | +127 | 127 |
    | 0 | 129 | +127 | 127 |
    | 5 | 5 | 0 | 0 |

    Also `reconstruct(0, 128) == 128`. The `(0, 255)` and `(255, 0)` rows catch a swapped `residual_symbol` argument order. `reconstruct` is commutative in its two `u8` inputs; those rows do not catch a swapped `reconstruct` argument order.
- [x] **Property tests** (AC: 2)
  - [x] In `tests/residual.rs`, use `proptest =1.12.0` with `default-features = false`. Config: `cases: 256`, `rng_seed: RngSeed::Fixed(0x5111_0002)`, `failure_persistence: Some(Box::new(FileFailurePersistence::Off))`, remaining fields from `Config::default()`. Do not change the raw profile seed `0x5111_0001` in `proptest_raw.rs`. Do not use regex strategies.
  - [x] One test draws random `(y, prediction)` pairs and asserts the three checks from the exhaustive task.
  - [x] One test draws a random payload whose length is one of `1`, `256`, `1024`, `2048`, and an independent random prediction byte per position. Reconstruct each byte. Zero mismatches.
  - [x] One causal test: a random 256-byte table, first prediction `0`, later prediction `table[previous_decoded]`. Encode with `residual_symbol`. Decode with `reconstruct`, feeding the reconstructed byte back as the previous byte. The decoded payload equals the original. This is a test double, not the Previous-byte Candidate. Do not add that Candidate.
- [x] **Both platforms** (AC: 4)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands, the revision (`git rev-parse HEAD` plus a note that story 1.1 framing may still be uncommitted), and that the residual hash test passed.
  - [x] Leave this subtask unchecked if either target did not run. Do not set the story Status to `done`. `dev-story` sets `review`. Code review sets `done` only when both runs are recorded.
  - [x] Do not edit `bitstream-spec-seed.md`, `CHANGELOG.md`, or the workspace version. Do not commit unless the user asks. Do not revert uncommitted story 1.1 files (`raw.rs`, `header.rs`, `leb128.rs`, `crc32.rs`, `error.rs`, `tests/`, `ci.sh`, `.cargo/config.toml`, `.gitattributes`).

### Review Findings

- [x] [Review][Patch] Exhaustive and random-pair tests lock signed δ to the symbol byte [crates/sxp-kernel/tests/residual.rs:38]
- [x] [Review][Patch] Blank id, path, or kind is rejected [crates/sxp-kernel/tests/support/mod.rs:96]
- [x] [Review][Patch] A quoted value with an interior quote or backslash is rejected [crates/sxp-kernel/tests/support/mod.rs:91]

## Dev Notes

### What this story builds

Modular residual math in `sxp-kernel`, and nothing downstream of it. Prediction quality is irrelevant here: every `ŷ` reconstructs `y`.

Out of scope: polynomial, trigonometric, and recurrence evaluators (stories 1.3–1.5); the range coder (1.6); the probability model (1.7); `encode_block` and Raw / Direct / Previous-byte / Constant framing (1.8); partition, Codebook, and CLI (1.9–1.11); `spec/bitstream.md` (story 2.1). G0 for this untagged clause is `bitstream-spec-seed.md` §4.5. Do not write a second normative document.

### Arithmetic

Governing rule, AD-3 and spec seed §4.5: `δ = ((y − ŷ + 128) mod 256) − 128`, `y = (ŷ + δ) mod 256`, symbol `= δ mod 256` as `0..255`. Under Direct the symbol is `y`.

`δ mod 256` equals `(y − ŷ) mod 256`. That identity is the test oracle (`wrapping_sub`). It is not a licence to implement `residual.rs` with wrapping helpers. The implementation computes `δ` with `rem_euclid`, then reduces.

Rust `%` truncates toward zero. For `y = 0`, `ŷ = 255`, the widened sum is `−127`. `% 256` yields `−127` and a `δ` of `−255`. `rem_euclid` yields `129`, then `δ = 1`, symbol `1`. The `(0, 255)` fixture fails a `%` implementation.

`reconstruct(prediction, symbol)` argument order is `(ŷ, symbol)`. `residual_symbol(y, prediction)` argument order is `(y, ŷ)`.

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable`. Those lints apply to `src/`, not to `tests/`. Tests already use `expect` and `panic`.

Public surface that must keep its signatures: `read_whole_file_raw`, `write_whole_file_raw`, `encode_whole_file_raw_header`, `decode_header`, `Header`, `HeaderBytes`, `encode_leb128`, `decode_leb128`, `Leb128`, `LENGTH_LIMIT`, `Crc32`, `crc32_iso_hdlc`, and every `Error` variant plus `Error::name`.

`sxp-kernel` has an empty `[dependencies]` table. `ci.sh` fails unless `cargo tree -e normal` and `-e build` for `sxp-kernel` are each one line, and unless `crates/sxp-kernel/build.rs` is absent. A dev-dependency does not appear in those two edges. `sha2` must not become a normal dependency.

Whole-file raw stays a streaming path. Residual functions allocate nothing and are not called from `raw.rs`.

### Working tree

`HEAD` is `5a58bbb`, the fixture commit story 1.1 required before framing existed. The framing modules and tests are uncommitted. `tests/support/mod.rs` does not parse, so the story 1.1 CI note in that story file is not reproducible until the repair task passes `./ci.sh`. Do not change story 1.1's sprint status in this story. Do not revert the framing.

When a commit is requested later, story 1.1 framing plus the parser repair is one commit, and the residual change is a second commit. This story does not make those commits.

### Project Structure Notes

- Residual code: `crates/sxp-kernel/src/residual.rs`.
- Tests: `crates/sxp-kernel/tests/residual.rs`.
- Hash artifact: `spec/vectors/residual-symbols.sha256`.
- Parser repair: `crates/sxp-kernel/tests/support/mod.rs` only.
- Lockfile: `Cargo.lock`, limited to the `sxp-kernel` dependency-list edit described above.
- `sxp-encoder` already depends on the kernel. Do not reverse that, and do not add residual math there.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.2, FR-1, vector-set inventory, elicitation amendment 1]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-3, crate table, AD-1 dev-dependencies]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §4.5, §8 Residual row]
- [Source: `_bmad-output/implementation-artifacts/1-1-round-trip-a-file-through-whole-file-raw.md` — kernel layout, proptest profile, lint block]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created
- Repaired `tests/support/mod.rs` so it parses `spec/vectors/index.toml`. `./ci.sh` then passed the existing story 1.1 suite. `cargo fmt` also reformatted `crc32.rs`, `lib.rs` use-order, and `round_trip.rs`. No story 1.1 kernel behavior was changed.
- `signed_residual`, `residual_symbol`, and `reconstruct` live in `sxp-kernel`. `residual_symbol` is `signed_residual(...).rem_euclid(256) as u8`. `direct_symbol` returns `y` and does not call them. Story 1.8 still owns the Direct `encode_block` call site.
- Exhaustive 65,536-pair test matches `u8::wrapping_sub` and SHA-256 `a8abf656d48d4ef997f294870ea52a827fe67197c243d63a6d805db66fbee1f1` from `spec/vectors/residual-symbols.sha256`. Named fixtures include both wrap directions, `δ = −128`, and the symbol 127/128 boundary.
- `sha2 =0.11.1` is a kernel dev-dependency only. `cargo update -p sxp-kernel` added `sha2` to that package's lock entry and synced the workspace crates from 0.0.7 to 0.0.8, matching `Cargo.toml`. Normal and build trees for `sxp-kernel` stay one line.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-01, including `exhaustive_pairs_match_wrapping_oracle_and_hash`.
- `./ci.sh aarch64-unknown-linux-gnu` passed on 2026-11-01 under `qemu-aarch64`, including the same hash test. Linker and runner came from `/tmp/serexpai-cross/bin` ahead of `PATH`. `HEAD` is still `5a58bbb011ebe761bb246cc4557587b056b21d9c`. Story 1.1 framing and this story are uncommitted.
- Code review patches applied: exact δ check via `symbol.cast_signed()` on every exhaustive and random pair; the vector parser rejects a blank id, path, or kind, and rejects a quote or backslash inside a quoted value. Both CI targets were rerun after those patches and passed.
- `cargo update -p sxp-kernel` also rewrote the lockfile package versions from 0.0.7 to 0.0.8. The workspace `Cargo.toml` already said 0.0.8. That sync is story 1.1's version, not a residual dependency. A later commit should keep the version hunks with the framing commit and the `"sha2"` line with this story.

### File List

- `crates/sxp-kernel/src/residual.rs`
- `crates/sxp-kernel/src/lib.rs`
- `crates/sxp-kernel/tests/residual.rs`
- `crates/sxp-kernel/tests/support/mod.rs`
- `crates/sxp-kernel/Cargo.toml`
- `Cargo.lock`
- `spec/vectors/residual-symbols.sha256`
- `crates/sxp-kernel/src/crc32.rs` (fmt only)
- `crates/sxp-kernel/tests/round_trip.rs` (fmt only)

### Change Log

- 2026-11-01: Kernel residual and reconstruction, direct-symbol contract, exhaustive hash, property tests, vector-index parser repair.
- 2026-11-01: Addressed code review findings — 3 patches applied.

## Story completion status

done. Both target CI runs passed after the review patches, including the residual hash test. Direct `encode_block` evidence stays with story 1.8.
