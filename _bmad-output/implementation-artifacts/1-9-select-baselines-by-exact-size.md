---
baseline_commit: 08af68a16f64769888cb3a8db8fc2553231b46e5
---

# Story 1.9: Select baselines by exact size

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want exact-size selection and a deterministic partition over baseline Blocks,
so that the blocked file is the minimum of the published key, and Whole-file raw still wins a tie.

## Acceptance Criteria

1. **S1.9-AC01.** D-03 (approved 2026-11-01) stands: the final partial span uses the smallest nominal in {256, 1024, 2048} that is `>=` the bytes left. `allowed_nominals` matches the tables below, including every fixed-length row. Identical full and final transitions are one entry. A fixed arm may emit a nominal that is not in its set. That is required. Do not reject it.

2. **S1.9-AC02.** Every transition cost is `encode_block(...).len()`: section, padding, parameters, payload, and flush. The reducer minimises `(block bytes, block count, first length code, candidate id)` under the suffix-DP rule below. Ids are Raw 0, Direct 1, Previous-byte 2, Constant `c` = `3 + c`. The full set evaluates all 256 constants. Per span, the winner is the minimum of `(bytes, id)`.

3. **S1.9-AC03.** A test-local exhaustive search, which does not call `reduce`, matches `select_baselines` on the listed lengths. On every chosen span, no other supplied candidate has a smaller `encode_block` length. An equal length has an id greater than or equal to the winner.

4. **S1.9-AC04.** The pinned fixtures below cover Raw versus coded, Direct versus Previous-byte, constants, a block-count tie, a length-code tie, and Whole-file raw, including the empty input. Whole-file raw wins when `block_bytes >= N`. The empty input is the 7-byte Whole-file raw stream. `N = 0` does not call `write_blocked_file`.

5. **S1.9-AC05.** Fixed-length arms round-trip through `read_blocked_file`. `threads` 0, 1, and 2 yield identical keys and identical file bytes. `./ci.sh` passes on `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`.

6. **S1.9-AC06.** `SearchMode::Reference` calls the cost function once per candidate. `SearchMode::HeaderPrune` skips a later candidate whose section length is already `>=` the incumbent exact size, and the counting fixture below shows those skips. The two modes, and any `threads` value, select the same key and the same bytes. Do not add an entropy screen, a shared model sequence, or a coder-size lower bound.

## Tasks / Subtasks

- [x] Nominals, ids, and encoder errors (AC: 1, 2)
  - [x] `allowed_nominals` matches both tables, in ascending nominal order
  - [x] `candidate_id` is Raw 0, Direct 1, Previous-byte 2, Constant `c` = `3 + c`
  - [x] `crates/sxp-encoder/src/lib.rs` forbids unsafe code and denies the kernel lint list
  - [x] `N >= LENGTH_LIMIT` returns `Error::Kernel(LengthOutOfRange)` before any search. Test `check_length(LENGTH_LIMIT)` without allocating
- [x] Span choice (AC: 2, 6)
  - [x] `choose_candidate` sorts by id and calls `encode_block` through the supplied function
  - [x] A cost error, including `BlockTooLarge`, returns immediately. It is not a skip
  - [x] Header-prune counting fixture: incumbent 2 skips every Constant and still calls Raw, Direct, and Previous-byte
- [x] Suffix reducer (AC: 2, 4)
  - [x] `reduce` reads a `SpanTable` and does not encode
  - [x] N = 512 block-count fixture and N = 1280 length-code fixture match the keys below
- [x] Select and write (AC: 2, 4, 5)
  - [x] `select_baselines` fills the table, then reduces on the calling thread
  - [x] `threads` 0 and 1 stay on the calling thread. `threads >= 2` uses a local `ThreadPoolBuilder` of that size, never the global pool
  - [x] `encode_baselines` writes only through `write_whole_file_raw` or `write_blocked_file`
  - [x] Predicted file length equals the written length on every fixture
- [x] Pinned fixtures and fixed arms (AC: 1, 4, 5)
  - [x] Empty, N = 6, N = 7, N = 8, and both constant-only fixtures
  - [x] `Fixed(1024)` at 256 bytes emits nominal 256. At 257 bytes it emits nominal 1024. `Fixed(2048)` at 300 bytes emits nominal 1024
  - [x] Each emitted file round-trips. `output.len() <= N + 11`, except the empty file, which is 7
- [x] Independent search and both targets (AC: 3, 5, 6)
  - [x] Brute force on lengths 1, 7, 8, 255, 256, 257, 512, and 600
  - [x] Reference, HeaderPrune, and `threads` 2 agree on those lengths
  - [x] `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`

### Review Findings

- [x] [Review][Patch] Multi-block fixed arms never reach the writer [crates/sxp-encoder/tests/select.rs:378]

## Dev Notes

### What this story builds

`sxp-encoder` learns to choose baseline Blocks. The kernel already frames a Block and a file. This story only chooses.

Out of scope, do not implement: Formula kinds, a Codebook, candidate ids at or above 259, field writers in the encoder (story 1.10); the compress CLI, temp-file rename, destination rollback (story 1.11); benchmark arms, sidecars, work counts (story 1.13); a shared model probability sequence across lengths; an entropy screen; a coder-size lower bound; rANS; edits to `bitstream-spec-seed.md`, `block.rs`, or `spec/vectors/index.toml`. Rayon 1.12.0 is already a dependency. Do not add a crate. Do not bump the format version.

`read_whole_file_raw` and `write_whole_file_raw` stay as they are. `encode_block` stays as it is. The encoder slices the input and passes those slices to the kernel. It never packs a header byte.

### API

```rust
pub enum LengthPolicy {
    All,
    Fixed(NominalLength),
}

pub enum SearchMode {
    Reference,
    HeaderPrune,
}

pub struct PartitionKey {
    pub block_bytes: u64,
    pub block_count: u32,
    pub steps: Vec<(u8, u16)>, // (length code, candidate id), file order
}

pub struct Choice {
    pub nominal: NominalLength,
    pub kind: BlockKind,
    pub id: u16,
    pub block_len: u64,
}

pub struct Selection {
    pub key: PartitionKey,
    pub choices: Vec<Choice>,
    pub whole_file_raw: bool,
    pub file_len: u64,
}

pub enum Error {
    Kernel(sxp_kernel::Error),
    NoCandidate,
    ThreadPool,
    MissingSpan,
}

pub fn candidate_id(kind: BlockKind) -> u16
pub fn all_baselines() -> Vec<BlockKind>
pub fn allowed_nominals(remaining: u64, policy: LengthPolicy) -> Vec<NominalLength>
pub fn check_length(n: u64) -> Result<(), Error>

#[doc(hidden)]
pub fn choose_candidate(
    candidates: &[BlockKind],
    mode: SearchMode,
    cost: impl Fn(BlockKind) -> Result<u64, sxp_kernel::Error>,
) -> Result<(BlockKind, u16, u64), Error>

#[doc(hidden)]
pub fn reduce(n: u64, policy: LengthPolicy, spans: &SpanTable) -> Result<PartitionKey, Error>

pub fn select_baselines(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Result<Selection, Error>

pub fn encode_baselines(
    input: &[u8],
    candidates: &[BlockKind],
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Result<Vec<u8>, Error>
```

`SpanTable` is a `BTreeMap<(u64, u8), (u16, u64)>`: `(offset, length code) → (candidate id, block bytes)`.

`Error`'s `Display` is the variant name. `Kernel` forwards the kernel `Display`, which is already the kernel variant name. Do not add a source chain or a context string.

`threads == 0` means one calling thread. It does not mean rayon's default. `ThreadPoolBuilder::build` failure is `Error::ThreadPool`. Do not `unwrap` it.

### Allowed nominals

`remaining == 0` returns an empty vec. Full transitions are the nominals in the arm's set whose byte size is `<= remaining`. The final transition exists only when `remaining <= max(set)`, and its nominal is the smallest of {256, 1024, 2048} that is `>= remaining`, even when that nominal is not in the set. Drop the duplicate when both rules produce the same nominal. Order the result by ascending byte size.

Set `All` = {256, 1024, 2048}. `Fixed(L)` has set `{L}`.

| remaining | All |
| --- | --- |
| 1, 255, 256 | 256 |
| 257, 1023, 1024 | 256, 1024 |
| 1025, 2047, 2048 | 256, 1024, 2048 |
| 2049 | 256, 1024, 2048, all full |

| remaining | Fixed(256) | Fixed(1024) | Fixed(2048) |
| --- | --- | --- | --- |
| 1 | 256 | 256 | 256 |
| 256 | 256 | 256 | 256 |
| 257 | 256 | 1024 | 1024 |
| 1024 | 256 | 1024 | 1024 |
| 1025 | 256 | 1024 | 2048 |
| 2048 | 256 | 1024 | 2048 |
| 2049 | 256 | 1024 | 2048 |
| 4096 | 256 | 1024 | 2048 |

`Fixed(1024)` at remaining 256 emits nominal 256. `is_canonical(N1024, 256)` is false, so the kernel would reject a "pure" 1024 nominal. Emit 256. At remaining 257 the same arm emits 1024, which is not in `{1024}` as a full block (`1024 > 257`) but is the canonical final. One nominal means the partition shape is fixed. Only the candidate per span varies.

Block starts are multiples of 256. A transition's actual length is `actual_len`. The next offset is `offset + actual`. When `actual` is not a multiple of 256, that next offset is `N`.

Only reachable offsets are states. An offset is reachable when a chain of allowed transitions from 0 lands on it. `Fixed(1024)` does not have a state at offset 256 when every full step is 1024. `reduce` follows those transitions. It does not open a state at every multiple of 256, and it does not demand a span that no transition can reach.

### Suffix DP

Fill every allowed span first. The span winner is `min (bytes, id)` and does not depend on any other span. Then reduce from the end on one thread.

The terminal state at offset `N` has `block_bytes = 0` and `block_count = 0`. At offset `o`, each allowed nominal has one table entry. Its candidate key is `(edge bytes + suffix bytes, 1 + suffix count, this length code, this id)`. Smaller bytes win, then smaller count, then smaller length code, then smaller id. Keep the first state if all four are equal.

Distinct nominals have distinct length codes, and each nominal already has one id. The first step therefore decides the lexicographic order. Do not walk the suffix sequence to compare keys. Do still add the suffix bytes and the suffix count. Reconstruct `steps` by walking parent pointers from offset 0, in file order.

A missing allowed span is `Error::MissingSpan`. Production must fill every allowed span, so a test that omits one is the only caller that should see it.

`block_bytes` is the sum of the chosen encoded Blocks. It does not include the file header or the CRC. Whole-file raw wins when `N == 0` or `block_bytes >= N`. That comparison is exact: both modes use `encode_leb128(N)` and a one-byte flags field (`0x01` blocked, `0x11` whole-file raw). No `N` makes those widths differ. Predicted length is `encode_whole_file_raw_header(N).len + payload + 4`, where `payload` is `N` for whole-file raw and `block_bytes` otherwise. Use the kernel header function for the width. Do not write those header bytes into a blocked file, and do not reimplement LEB128.

`N = 0` returns immediately: empty key, empty choices, `whole_file_raw`, `file_len` 7. It ignores the candidate list, including an empty one. `N > 0` with an empty list after id dedup is `Error::NoCandidate`, before the search.

`N >= LENGTH_LIMIT` is `Error::Kernel(LengthOutOfRange)` before the search. `check_length` is that predicate.

### Header prune

Section length is 1 for Raw, Direct, and Previous-byte, and 2 for Constant. Evaluate in ascending id. Always call the first. Skip a later candidate when `section_len >= incumbent exact size`. A lower id has already won an equal size. This is the only enabled shortcut.

The skip is real only when the incumbent is at most 2. The only baseline Block that small is Raw of one byte (`00` plus the byte). Direct and Previous-byte have a 1-byte section, so they are still encoded. All 256 constants are skipped. Coded Blocks are at least 5 bytes, so the prune never removes a candidate that could beat Raw on a one-byte span.

Counting fixture, single-threaded `choose_candidate`, not a file. Costs: Raw → 2, every other kind → 50. `Reference` calls the function 259 times. `HeaderPrune` calls it 3 times (Raw, Direct, Previous-byte) and returns Raw. Both return id 0 and cost 2.

Second fixture, so a cheap incumbent does not skip everything: Raw → 5, Direct → 4, every other kind → 9. `HeaderPrune` calls all 259. Winner is Direct, id 1, cost 4.

Do not share model state across lengths. Do not skip on an unproven coder bound. Story 1.10 may add Formula section lengths. It must not renumber these ids.

### Pinned fixtures

Measured from this kernel on 2026-11-03. All-zero inputs. Nominal on these short inputs is 256. Do not "correct" the constant sizes. They are what `encode_block` returns.

| Input | Candidates | Span winner | Block bytes | File |
| --- | --- | --- | --- | --- |
| empty | ignored | none | 0 | whole-file raw `8d 11 00 00 00 00 00` |
| 6 zeros | all baselines | Raw, id 0. Direct and Previous-byte are also 7 | 7 | whole-file raw, because 7 > 6 |
| 7 zeros | all baselines | Direct, id 1. Previous-byte is also 7 | 7 | whole-file raw `8d 11 07 00 00 00 00 00 00 00 7e df 6c 9d` |
| 8 zeros | all baselines | Direct, id 1. Previous-byte is also 7 | 7 | blocked `8d 01 08 04 00 00 00 00 00 00 69 df 22 65` |
| 1 zero | constants only | Constant(1), id 4. Constant(0) is 7 bytes; the other 255 constants are 6 | 6 | whole-file raw, because 6 > 1. The key still records Constant(1) |
| 9 zeros | constants only | Constant(0), id 3. Every constant is 8 bytes | 8 | blocked `8d 01 09 0c 00 00 00 00 00 00 00 ae 14 09 e6` |

The 6-zero file is compared with `write_whole_file_raw`, not with a pasted CRC. The key is one Raw step `(0, 0)`.

Injected costs use `reduce` directly. Default cost is 100 unless named. One candidate, id 0.

N = 512, block-count tie. Allowed spans are only `(0, 1024)`, `(0, 256)`, and `(256, 256)`. Costs: `(0, 1024) = 10`, each 256-span `= 5`. The one-block path and the two-block path both total 10. The one-block path wins on count. Key: `block_bytes = 10`, `block_count = 1`, `steps = [(1, 0)]`. There is no 2048 span at remaining 512. Do not give every block cost 10. That makes the totals 10 and 20, and the count key never decides.

N = 1280, length-code tie. Costs of 6 at `(0, 256)`, `(256, 1024)`, `(0, 1024)`, and `(1024, 256)`. Cost 100 at every other allowed span, including `(0, 2048)`, `(512, 1024)`, and `(768, 1024)`. Both two-block paths total 12. `(256 then 1024)` starts with length code 0 and beats `(1024 then 256)`, which starts with length code 1. Key: `block_bytes = 12`, `block_count = 2`, `steps = [(0, 0), (1, 0)]`. A single 2048-nominal block is allowed here. Its cost stays 100 so it does not win on count.

`encode_baselines` output length equals `Selection.file_len` on every fixture, both modes.

### Fixed-arm round-trips

Use all baselines and `SearchMode::Reference`.

- 256 zeros, `Fixed(1024)`: one Block, nominal 256, `read_blocked_file` returns the input.
- 257 zeros, `Fixed(1024)`: one Block, nominal 1024.
- 300 zeros, `Fixed(2048)`: one Block, nominal 1024.

Do not pin which candidate wins those three. Pin the nominal and the round-trip.

### Independent search

In `crates/sxp-encoder/tests/select.rs`, write a recursive search that calls `allowed_nominals` and `encode_block` and does not call `reduce` or `select_baselines`. Minimise the same four-part key. Compare it with `select_baselines(..., Reference, 1)` on these lengths, each filled by an xorshift with seed `0x5111_0009`: 1, 7, 8, 255, 256, 257, 512, 600. Stop there. Do not add a property over large `N`. Debug builds encode every candidate.

On each chosen span of those files, every other baseline is at least as large, and an equal size has a higher or equal id.

`HeaderPrune` and `threads = 2` return the same key and the same `encode_baselines` bytes as `Reference` with `threads = 1` on those eight lengths.

### Project Structure Notes

- `crates/sxp-encoder/src/lib.rs` re-exports the public items and owns the lint block.
- `crates/sxp-encoder/src/error.rs` and `crates/sxp-encoder/src/select.rs`.
- Tests: `crates/sxp-encoder/tests/select.rs` only. No new kernel test. No `spec/vectors` row.
- `src/` does not call `assert`, `unwrap`, `expect`, or `panic`. Integration tests may `assert`.
- Index with `get`. `sxp-kernel` stays a path dependency with an empty `[dependencies]` table of its own.

### Previous story intelligence

Story 1.8 put framing in the kernel and left the encoder as a stub. `encode_block` returns `BlockTooLarge` instead of a `Vec` when the buffer would pass `MAX_ENCODED_BLOCK`. Propagate that. Do not catch it and keep searching.

`Error::Display` in the kernel is the variant name. Match that. `write_blocked_file(0, …)` is `BlockedEmpty` and writes nothing. The `N = 0` path must not call it.

Story 1.8's review required tests for the states the happy path skips: padding, reserved kind, and a non-canonical length that loses to an earlier check. This story's skipped states are the header-prune constants, the lost block-count path, and the lost length-code path. Each has a fixture above.

Both-platform `./ci.sh` is the gate. Story 1.8 ran aarch64 under `qemu-aarch64` with `/tmp/serexpai-cross/bin` on `PATH`. Seed for generated bytes is `0x5111_0009`.

### Git intelligence

`HEAD` at story creation is `08af68a`, `feat: round-trip one baseline block (v0.0.14)`. Recent titles:

- `08af68a` feat: round-trip one baseline block (v0.0.14)
- `c05a8b0` feat: adapt an order-0 probability model (v0.0.13)
- `d06ecd6` feat: code decisions with the binary range coder (v0.0.12)
- `aa2fb8f` feat: evaluate a recurrence formula (v0.0.11)
- `277f5a1` feat: evaluate a trigonometric formula (v0.0.10)

The selector is new. The kernel is a callee. Capture `baseline_commit` when implementation starts and do not overwrite it.

### Latest technical information

Stay on toolchain `1.99.1` and rayon `1.12.0`. Do not `rustup update`. Do not take a newer rayon. `ThreadPoolBuilder::num_threads` rejects 0 by treating it as the default pool size, which is why this story maps 0 to "one calling thread" before rayon sees it. No new web API is involved.

### Project context reference

No `project-context.md` exists. The encoder lint block, AD-8, AD-9, and AD-10 are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.9, S1.9-AC01–AC06]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-2, AD-8, AD-9, AD-10, AD-13]
- [Source: `docs/decision_logs/decision_log_2026_W44.md` — D-03, 2026-11-01]
- [Source: `_bmad-output/implementation-artifacts/1-8-round-trip-one-baseline-block.md` — `encode_block`, canonical nominals, file writers]
- [Source: `crates/sxp-kernel/src/block.rs` — `is_canonical`, `actual_len`, `encode_block`, `write_blocked_file`]
- [Source: `crates/sxp-kernel/src/header.rs` — flags `0x01` and `0x11`, shared LEB128 width]
- [Source: `crates/sxp-kernel/src/raw.rs` — `write_whole_file_raw`]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- `sort_by_key` needs a closure because `NominalLength::bytes` takes `self`.
- The header-prune counter uses `Cell` because `choose_candidate` takes `Fn`, so the same function can run on a rayon pool.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- Added baseline exact-size selection in `sxp-encoder`. Costs are `encode_block` lengths. Files are written only by `write_whole_file_raw` and `write_blocked_file`.
- The suffix DP minimises block bytes, then block count, then length code, then candidate id. Whole-file raw wins when `block_bytes >= N`, including the empty input.
- Header prune skips constants once a one-byte Raw Block is the incumbent. Reference mode, header prune, and thread counts 0, 1, and 2 select the same bytes.
- Fixed arms emit the canonical final nominal, including a nominal outside the fixed set. An independent brute force matches the selector on the eight listed lengths.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-03. `git rev-parse HEAD` was `08af68a16f64769888cb3a8db8fc2553231b46e5`.
- `./ci.sh aarch64-unknown-linux-gnu` passed on the same date and the same `HEAD`, under `qemu-aarch64` with `/tmp/serexpai-cross/bin` on `PATH`.
- Code review applied 1 patch. Fourteen findings were dismissed. Nothing was deferred. After that patch, `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu` passed again at the same uncommitted `HEAD` `08af68a16f64769888cb3a8db8fc2553231b46e5`.

### File List

- crates/sxp-encoder/src/lib.rs
- crates/sxp-encoder/src/error.rs
- crates/sxp-encoder/src/select.rs
- crates/sxp-encoder/tests/select.rs
- _bmad-output/implementation-artifacts/1-9-select-baselines-by-exact-size.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-11-03: Story context created. Reachable offsets only. Header prune is measured, not assumed.
- 2026-11-03: Implemented baseline exact-size selection, pinned fixtures, and both-platform CI.
- 2026-11-03: Addressed code review findings — 1 patch applied, 14 dismissed, none deferred.

## Story completion status

done. The review patch round-trips multi-block fixed arms through the kernel writer. Both target CI runs passed again. D-07 remains open. Formula selection is story 1.10.
