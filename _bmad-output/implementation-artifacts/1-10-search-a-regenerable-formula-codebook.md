---
baseline_commit: e6bdfbc680e3a4db9701de6b1d2b3edee1ddc0f9
---

# Story 1.10: Search a regenerable formula codebook

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want a reproducible Codebook searched exhaustively, with every Formula stored in the Block,
so that the decoder rebuilds each prediction from the bitstream alone.

## Acceptance Criteria

1. **S1.10-AC01.** `generate_codebook("fixture-v1")` emits the three formulas below, in that order, and no others. Family order is polynomial, trigonometric, recurrence. Caps are 2048 per family and 4096 total. Duplicate canonical sections keep the first entry and drop the later one. Ids are assigned after that drop, with no gaps, starting at 259. The content hash is SHA-256 of the canonical blob below. Two calls return the same bytes and the same hash. `./ci.sh` on both supported targets agrees.

2. **S1.10-AC02.** `generate_codebook("")` and any other unknown name return `AbsentRecipe`. `generate_codebook("production")` and `generate_codebook("benchmark")` return `UnresolvedProduction` and emit nothing. `assemble_codebook` rejects those same names and labels, an empty name, a missing family, a cap breach, a non-formula kind, and a formula whose `cw` or coefficients fail the checks below. The fixture label is `fixture`. It is not `benchmark` and not `production`. No generate or assemble function takes input bytes. Fixture `cw` values are fixture literals. There is no production width and no `D-07` constant.

3. **S1.10-AC03.** The kernel writes and parses every formula field. The encoder does not pack bits. Polynomial stores degree, `Q`, `cw`, and degree+1 coefficients, lowest power first. Trigonometric stores order, `Q`, `cw`, omega, and `2·order+1` coefficients (`c0`, then `a_k`, `b_k`). Recurrence stores order, modulus, `cw`, coefficients, and initial states. Order 1 has `a`, `b`, and `s0` only. Stored trigonometric `cw` is 1..=29. Stored recurrence `cw` is 1..=25. Stored polynomial `cw` is 1..=32, and a stored `−2^31` is rejected. `encode_block` / `decode_block` are the only Block codec. Format version stays 1.

4. **S1.10-AC04.** The rejection and valid rows below exist as committed vectors. They cover sign extension, each width boundary, the implied coefficient count, each parameter-domain boundary, reserved kind and length, illegal stored fields, a non-zero padding bit, and every truncation prefix of one short formula Block. Each fixture family round-trips at nominal 256, 1024, and 2048, and at actual length 1. `read_blocked_file` restores those bytes. The kernel test does not link `sxp-encoder` and does not open a Codebook.

5. **S1.10-AC05.** Reference mode costs every baseline and every fixture formula on every allowed span. Formula ids are 259, 260, and 261. A same-span equal cost keeps the smaller id, so a baseline beats a formula, and wire kinds 4, 5, and 6 are not consulted. On the three traces below, the matching formula wins against all baselines and the other two formulas.

6. **S1.10-AC06.** The only prune is the existing header prune, using the kernel section length. `CostCell::Skipped` is the sidecar sentinel `0xFFFF`. A real Block is at most 32898 bytes, so it cannot collide with that sentinel. Reference mode has no skipped cell. A cost error aborts and is not stored as a cell. An independent oracle matches the selector on candidate, partition, block bytes, and file bytes. Every skipped cell's Reference cost is at least the incumbent cost at the skip.

7. **S1.10-AC07.** `threads` 0, 1, and 2, and both search modes, return the same key and the same file bytes on the traces below. The file's version nibble stays 1. The decoder never receives the recipe name, the label, the hash, or encoder state.

## Tasks / Subtasks

- [x] Kernel formula wire (AC: 3, 4)
  - [x] `BlockKind` gains the three formula variants, each holding the evaluator and `cw`
  - [x] `pub(crate)` accessors return only the live coefficient and state prefix
  - [x] `section_len` and `formula_section` live in the kernel. Baseline sections stay 1 and 2 bytes
  - [x] Kinds 0–3 keep the existing header writer. Kinds 4–6 use the new packer and do not call the baseline padding check
  - [x] `Recurrence::predict_span` is one forward pass. Encode and decode use it
  - [x] `MAX_ENCODED_BLOCK` becomes 32898. `BLOCK_WINDOW` stays 36864
  - [x] Short kind 4/5/6 inputs return `TruncatedBlock`. Update the story 1.8 assertions
- [x] Fixture codebook (AC: 1, 2)
  - [x] `generate_codebook` and `assemble_codebook` with the rejection table below
  - [x] Dedup, caps, family check, then ids `259 + index`
  - [x] SHA-256 of the canonical blob. `sha2 = "=0.11.1"` on `sxp-encoder` only. Commit `Cargo.lock`
  - [x] Pin `spec/vectors/codebook-fixture-v1.sha256`
- [x] Search (AC: 5, 6, 7)
  - [x] `SearchCandidate { id, kind }` is what `choose_candidate`, the span table, and `Choice` carry
  - [x] `candidate_id` returns `Option<u16>`: `Some` for baselines at the old numbers, `None` for a formula
  - [x] `select_baselines` and `encode_baselines` reject a formula kind with `NoCandidate`
  - [x] `select_codebook` / `encode_codebook` append the codebook after the baselines
  - [x] Writing uses `Choice.kind`. Ids `>= 259` are not passed to `kind_from_id`
  - [x] `span_cells` returns `CostCell`. Header prune records `Skipped`
- [x] Vectors, oracle, both targets (AC: 4, 5, 6, 7)
  - [x] Independent section packer in the kernel tests matches `formula_section` on the three fixture formulas
  - [x] Commit valid and rejection rows. Truncate one short Block at every prefix
  - [x] Each family wins on its trace. Injected ties prefer the smaller id
  - [x] Oracle and `threads` 0, 1, 2 agree. `./ci.sh` passes on both targets

### Review Findings

- [x] [Review][Patch] Domain checks run before padding [crates/sxp-kernel/src/formula_wire.rs]
- [x] [Review][Patch] Full nominals and the three winning traces round-trip through read_blocked_file [crates/sxp-kernel/tests/formula_block.rs]
- [x] [Review][Patch] The one-block oracle compares complete file bytes [crates/sxp-encoder/tests/codebook.rs]
- [x] [Review][Patch] A complete non-canonical formula section is NonCanonicalLength [crates/sxp-kernel/tests/formula_block.rs]
- [x] [Review][Patch] select_baselines documents that N == 0 ignores a formula kind [crates/sxp-encoder/src/select.rs]
- [x] [Review][Patch] select_codebook rejects a baseline kind stored in the codebook [crates/sxp-encoder/src/select.rs]
- [x] [Review][Patch] Two candidates with the same id and different kinds are NoCandidate [crates/sxp-encoder/src/select.rs]

## Dev Notes

### What this story builds

The kernel learns to frame a formula Block. The encoder learns to regenerate a labelled fixture Codebook and to search it with the story 1.9 DP. The decoder reads parameters from the Block. It does not read a recipe.

Out of scope, do not implement: the compress CLI, temp-file rename, destination rollback (story 1.11); a sidecar file, benchmark arms, work counts, a report (story 1.13); tuning on corpus bytes; a production recipe; a production `cw`; an entropy screen; a shared model sequence; a coder-size lower bound; rANS; a format-version bump; edits to `bitstream-spec-seed.md`. Do not add a crate other than `sha2 = "=0.11.1"` on `sxp-encoder`. Do not add `sha2` to the kernel's normal dependencies. Rayon 1.12.0 stays.

`read_whole_file_raw` and `write_whole_file_raw` stay as they are. Baseline header bytes stay as they are. Story 1.9 ids 0..=258 stay as they are.

### Fixture formulas

These `cw` values are fixture literals. They are not a D-07 choice. D-07 stays open. D-09 stays open: `production` and `benchmark` do not generate.

| Id | Family | Parameters |
| --- | --- | --- |
| 259 | polynomial | degree 2, Q 8, cw 32, coefficients `[0, 0, 2_147_483_647]` (`i32::MAX`) |
| 260 | trigonometric | order 1, Q 8, omega `1 << 24`, cw 22, coefficients `[0, 1 << 20, 0]` |
| 261 | recurrence | order 2, `m_mod` 251, cw 5, coefficients `[3, 5, 7]`, state `[1, 2]` |

`1 << 20` does not fit in signed cw 21 (`2^20 - 1` is the maximum). cw 22 is the fixture width. `i32::MAX` needs cw 32.

Measured on this kernel on 2026-11-03, before formula framing existed. Costs are baseline `encode_block` lengths. The formula is expected to win because a zero-residual payload is the Direct-on-zeros payload (7 bytes at these lengths) and the section is about 13–15 bytes, under the baseline best:

| Trace | Length | How built | Baseline best | Winner |
| --- | --- | --- | --- | --- |
| polynomial | 48 | `predict(0..48)` | Direct 48 | id 259 |
| trigonometric | 64 | `predict(0..64)` | Raw 65 | id 260 |
| recurrence | 32 | `predict(0..32)` | Raw 33 | id 261 |

Search each trace with `all_baselines()` plus the whole fixture. Assert the winning id. Do not pin the file bytes in this table. Do assert the round-trip.

A degree-0 zero polynomial, and the degree-1 ramp `coefficients [0, 1 << 19]`, lose to Previous-byte or Direct. Do not use them as the fixture. Previous-byte on a 256-byte ramp is 9 bytes, which is smaller than any formula section plus a coder flush.

### Codebook API

```rust
pub struct Codebook {
    pub name: String,
    pub label: String,
    pub version: u16,
    pub formulas: Vec<BlockKind>,
    pub content_sha256: [u8; 32],
}

pub fn generate_codebook(name: &str) -> Result<Codebook, Error>
pub fn assemble_codebook(
    name: &str,
    label: &str,
    version: u16,
    formulas: Vec<BlockKind>,
) -> Result<Codebook, Error>
pub fn check_family_caps(polynomial: usize, trigonometric: usize, recurrence: usize) -> Result<(), Error>
```

`generate_codebook` accepts only `"fixture-v1"`. That calls `assemble_codebook("fixture-v1", "fixture", 1, <the three kinds>)`.

| Call | Error |
| --- | --- |
| `generate_codebook("")`, `generate_codebook("no-such")` | `AbsentRecipe` |
| `generate_codebook("production")`, `generate_codebook("benchmark")` | `UnresolvedProduction` |
| `assemble_codebook` name or label `production` or `benchmark` | `UnresolvedProduction` |
| `assemble_codebook` empty name | `AbsentRecipe` |
| a baseline kind in `formulas` | `Kernel(InvalidParameter)` |
| `cw` or a coefficient that fails the fit check | `Kernel(InvalidParameter)` |

`check_family_caps` order: a family count above 2048, or a total above 4096, is `CapExceeded`. Otherwise a zero count is `MissingFamily`. `(2049, 0, 0)` is `CapExceeded`. `(1, 1, 0)` is `MissingFamily`. `(2048, 2048, 0)` is `MissingFamily`. `(2048, 2048, 2048)` is `CapExceeded`. `(2048, 2048, 1)` is `CapExceeded` because the total is 4097. `(2048, 2047, 1)` is `Ok`. The fixture calls this after dedup.

Dedup key: `formula_section`, which is the kernel section at length code 0. Keep the first. Ids are `259 + position` after the drop. A duplicate polynomial in front of the trig and recurrence formulas yields three entries, ids 259, 260, 261, not 259, 261, 262. The same mathematical polynomial at cw 8 and cw 9 is two entries, because `cw` is inside the section.

`assemble_codebook("fixture-alt", "fixture", 1, same three)` is allowed and hashes differently from `"fixture-v1"`, because the name is inside the blob. `generate_codebook("fixture-alt")` is still `AbsentRecipe`.

No function in this module takes `&[u8]` input, a path, or a tuning corpus. Do not add one.

### Canonical blob and hash

SHA-256 over this byte string, nothing else:

- ASCII `sxp-codebook-v1` (15 bytes, no trailing NUL)
- schema version `u16` LE = 1
- name: `u16` LE byte length, then the UTF-8 name
- label: `u16` LE byte length, then the UTF-8 label
- recipe version `u16` LE
- count `u32` LE
- each formula in id order: family `u8` (0 polynomial, 1 trigonometric, 2 recurrence), `u16` LE byte length, then `formula_section` bytes

`content_sha256` is that digest. Commit it as one hex line in `spec/vectors/codebook-fixture-v1.sha256`. The test recomputes the blob with `sha2` and compares both the call result and the file. `Cargo.lock` changes because `sxp-encoder` gains `sha2 = "=0.11.1"`. CI uses `--locked`.

### Wire layout

LSB first inside a byte. A field that crosses a byte continues in the next byte's low bits. Signed fields are two's complement of `cw` bits, sign-extended to `i32` on read. `cw == 32` is the raw `i32` bits. Build the mask in `u64`. `1u32 << 32` is a compile error or a panic, and this crate denies panic.

Padding bits to the next byte are zero. A set padding bit is `NonZeroPadding`.

`formula_section(kind)` packs a formula at length code 0 and returns the padded section with no coder payload. `section_len` is that length for a formula, `Ok(2)` for Constant, and `Ok(1)` for Raw, Direct, and Previous-byte. A baseline passed to `formula_section` is `InvalidParameter`.

Do not route kinds 0–3 through the new packer. The pinned headers in `spec/vectors/block-baseline-headers.hex` must stay byte-identical. Kinds 4–6 must not use `parse_baseline`. That function treats bits 5–7 of byte 0 as padding. Those bits are the first formula field.

Structural length is known once degree or order, and the stored `cw`, have been read. Use the stored `cw` even when it is outside the family cap. The coefficient count is `degree + 1`, `2·order + 1`, or `order + 1`. It is not a separate field. Order 1 recurrence does not read `c` or `s1`.

Decode check order, first match wins:

1. No first byte → `TruncatedBlock`.
2. Length code 3 → `ReservedLength`.
3. Kind 7 → `ReservedKind`.
4. For kinds 4–6: if the structural fields or the computed section are short → `TruncatedBlock`.
5. Domain checks in wire order → `InvalidParameter`.
6. Padding bit set → `NonZeroPadding`.
7. Nominal not canonical → `NonCanonicalLength`.
8. Raw payload short → `TruncatedBlock`.
9. Coder errors pass through, including `TruncatedCoder`.

`next_block` refills on `TruncatedBlock` and `TruncatedCoder` only. Returning `InvalidParameter` for a `cw` whose section is not yet buffered would stop that refill. Truncation wins until the computed section is present.

Domain checks, in wire order. Stop at the first failure:

- Polynomial: `Q − 8` in 17..=31; then a coefficient outside `[−2^31 + 1, 2^31 − 1]` or outside signed `cw`. Every stored polynomial `cw` in 1..=32 is in range. The `cw` 32 pattern `0x8000_0000` is the domain failure.
- Trigonometric: `Q − 8` in 17..=31; then stored `cw` in 30..=32; then a coefficient with absolute value `>= 2^28`. omega itself is any `u32`.
- Recurrence: stored `m_mod − 2 == 0x00FF_FFFF`; then stored `cw` in 26..=32; then a coefficient with absolute value `>= 2^24` or outside signed `cw`; then `s0` or `s1 >= m_mod`.

Encode builds the evaluator with `Polynomial::new`, `Trigonometric::new`, or `Recurrence::new`, then checks `cw` and the signed fit. Both failures are `InvalidParameter`. Decode builds the same evaluators, so the domain limits stay in those constructors. Add `pub(crate)` accessors for the live prefix only. Unused tail slots of `[i32; 16]`, `[i32; 33]`, `[i32; 3]`, and `[u32; 2]` must not reach the wire. Do not make those fields public. Do not give the encoder a public getter.

`BlockKind::polynomial(formula, cw)`, and the trigonometric and recurrence equivalents, are the constructors the codebook calls. `cw` outside the family cap, or a live coefficient that does not fit in signed `cw`, is `InvalidParameter`.

### Section size and the Block cap

Maximum sections, in bytes after padding:

| Family | Bits | Bytes |
| --- | --- | --- |
| polynomial, degree 15, cw 32 | 531 | 67 |
| trigonometric, order 16, cw 29 | 1008 | 126 |
| recurrence, order 2, cw 25 | 158 | 20 |

Coder payload bound stays 32772. `MAX_ENCODED_BLOCK` is `126 + 32772 = 32898`. The comment in `block.rs` that says the section is at most 2 bytes, and the comment that says 32778, are stale. Replace them. `BLOCK_WINDOW` stays 36864: `32898 + 4 = 32902` is under it. A Block past 32898 is still `BlockTooLarge`, and the search returns that error. It is not a skip.

### Prediction

`n` is the Block-local index, `0 .. actual − 1`. Each Block starts again at the formula's own parameters. Recurrence starts again at `s0` and `s1`.

`Recurrence::predict` restarts from `s0` on every call. A 2048-byte Block must not call it once per position. Add:

```rust
impl Recurrence {
    pub fn predict_span(&self, len: u16) -> Result<Vec<u8>, Error>
}
```

`len == 0` is an empty `Ok`. `len > 2048` is `InvalidParameter`. `out[i] == predict(i)` for every `i < len`. Encode and decode of a recurrence Block call `predict_span(actual)` once. Polynomial and trigonometric encode may call `predict` per position. Test both orders at lengths 1, 2, 17, 64, and 2048.

### Search API

`candidate_id` becomes `Option<u16>`. Baseline arms stay Raw 0, Direct 1, Previous-byte 2, Constant `c` = `3 + c`. Formula arms return `None`. Update the story 1.9 asserts to `Some(0)` through `Some(258)`. Add `None` for one fixture formula. Do not renumber baselines.

```rust
pub struct SearchCandidate {
    pub id: u16,
    pub kind: BlockKind,
}

pub enum CostCell {
    Evaluated(u64),
    Skipped,
}

pub fn select_codebook(
    input: &[u8],
    baselines: &[BlockKind],
    codebook: &Codebook,
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Result<Selection, Error>

pub fn encode_codebook(
    input: &[u8],
    baselines: &[BlockKind],
    codebook: &Codebook,
    policy: LengthPolicy,
    mode: SearchMode,
    threads: usize,
) -> Result<Vec<u8>, Error>

#[doc(hidden)]
pub fn span_cells(
    bytes: &[u8],
    nominal: NominalLength,
    remaining: u64,
    candidates: &[SearchCandidate],
    mode: SearchMode,
) -> Result<Vec<(u16, CostCell)>, Error>
```

`SpanTable` becomes `BTreeMap<(u64, u8), SpanCost>` with `SpanCost { id, bytes, kind }`. `reduce` reads `id` and `bytes` only. `choices_from` copies `kind` from the table into `Choice`. For an id `<= 258`, the stored kind must equal `kind_from_id`. A mismatch is `NoCandidate`. An id `>= 259` must not call `kind_from_id`. `encode_codebook` writes `Choice.kind`.

`select_baselines` and `encode_baselines` still take `&[BlockKind]`. A formula kind in that slice is `NoCandidate` before any search. They build `SearchCandidate` with `candidate_id(...).expect` replaced by an explicit `ok_or(NoCandidate)`. Do not use `expect`.

`choose_candidate` takes `&[SearchCandidate]`. Sort by `id`. Dedup by `id`. Header prune calls `sxp_kernel::section_len`. The first candidate is always costed. A later candidate whose section length is `>=` the incumbent exact size is `Skipped` and is not costed. `section_len` or `encode_block` returning `Err` aborts the span. `Skipped` is not an error.

Story 1.13 will store `Skipped` as the `u16` sentinel `0xFFFF` and an evaluated cost as that cost. This story does not write a sidecar file. `span_cells` is the contract. Reference mode returns `Evaluated` for every candidate. Legal costs are `<= 32898`.

Update every `SpanTable` literal in `crates/sxp-encoder/tests/select.rs`. Injected rows that use id 0 store `BlockKind::Raw`. The reducer still ignores `kind`. One new test stores a formula kind and checks `choices_from` returns that kind, so a `kind_from_id` fallback cannot hide.

### Ties and skips

Injected `choose_candidate` costs, not real files:

- Raw id 0 cost 5, and a degree-0 Q-8 cw-1 polynomial as id 259 cost 5. That section is 3 bytes, so header prune still evaluates it. Winner id 0. The cell is `Evaluated`, not `Skipped`.
- Raw id 0 cost 5, formula id 259 cost 4. Winner id 259.
- Formula id 260 cost 4, formula id 259 cost 4. Winner id 259. Wire kind is not the key. Put the higher id first in the input slice and assert the sort still picks 259.

Real skip, 32 zero bytes, all baselines plus the fixture, `HeaderPrune`: the winning baseline's exact size is 8 (`Direct` on 32 zeros). Each fixture section is longer than 8, so each formula cell is `Skipped`. Reference mode evaluates them and selects the same baseline id. For every `Skipped` cell, the Reference cost of that candidate is `>=` the incumbent at the skip. The incumbent is the minimum evaluated cost among lower ids.

`HeaderPrune` and `Reference` on the three winning traces select the same id and the same `encode_codebook` bytes. The formula sections are about 15 bytes and the baseline incumbents are 33, 48, and 65, so the formulas are evaluated, not skipped.

### Vectors

Kernel tests only. Do not depend on `sxp-encoder`. A test-local LSB packer, not the kernel packer, builds sections. On the three fixture formulas it must equal `formula_section`. Then commit those bytes.

Valid rows, one per family, in `spec/vectors/formula-blocks.hex` (name, then hex). Re-encode and compare. Also `read_blocked_file` on a one-Block file:

- each fixture family at nominal 256, 1024, and 2048, remaining equal to the nominal, payload `0x5A` repeated
- each fixture family at nominal 256, remaining 1, payload `[0x5A]`

Rejection rows, full section present unless the row is a truncation. One fault each. Payload is absent on pure section rejects, and `remaining` is 256 so the canonical check is not the first failure:

| Row | Fault | Error |
| --- | --- | --- |
| `poly-q-17`, `poly-q-31` | degree 0, stored `Q − 8` is 17 or 31, cw 1, coefficient 0 | `InvalidParameter` |
| `poly-cw32-min` | degree 0, Q 8, cw 32, coefficient bits `0x8000_0000` | `InvalidParameter` |
| `poly-cw8-neg1` | degree 0, Q 8, cw 8, coefficient −1, then a legal one-byte payload round-trip | decoded coefficient −1 |
| `poly-cw8-127` | coefficient 127 | decoded 127 |
| `poly-cw9-neg128` | coefficient −128 | decoded −128 |
| `poly-cw8-128` | constructor rejects 128 | `InvalidParameter` before encode |
| `poly-pad` | degree 0, Q 8, cw 1, coefficient 0, first padding bit set | `NonZeroPadding` |
| `poly-extra-coeff` | degree 0 with the second coefficient's bit set inside the padding | `NonZeroPadding` |
| `trig-q-17` | order 1, stored `Q − 8` is 17, cw 1, omega 0, three zero coefficients | `InvalidParameter` |
| `trig-cw30`, `trig-cw31`, `trig-cw32` | stored cw 30, 31, 32, remaining fields zero | `InvalidParameter` |
| `trig-cw29-min` | cw 29, `c0 = −2^28`, `a1 = 0`, `b1 = 0` | `InvalidParameter` |
| `rec-mmod-ffffff` | order 1, stored `m_mod − 2` is `0x00FF_FFFF`, cw 1, coefficients 0, `s0` 0 | `InvalidParameter` |
| `rec-cw26` through `rec-cw32` | order 1, `m_mod` 2, that cw, zeros | `InvalidParameter` |
| `rec-cw25-min` | order 1, `m_mod` 2, cw 25, `a = −2^24`, `b = 0`, `s0 = 0` | `InvalidParameter` |
| `rec-s0-eq` | order 1, `m_mod` 2, cw 1, `a = 0`, `b = 0`, `s0 = 2` | `InvalidParameter` |
| `rec-s1-eq` | order 2, `m_mod` 2, cw 1, coefficients 0, `s0 = 0`, `s1 = 2` | `InvalidParameter` |
| `rec-order1-pad` | legal order 1, a padding bit set where an order-2 `s1` would have been | `NonZeroPadding` |

Truncation: encode the polynomial fixture at nominal 256, remaining 1, payload `[0x5A]`. Every proper prefix of that encoding returns `TruncatedBlock` while the prefix is inside the section, and `TruncatedCoder` once the prefix reaches the coder payload but not its end. Do not accept `UnsupportedKind` for `0x10`, `0x11`, `0x14`, or `0x18`. Those one-byte headers are `TruncatedBlock`. `0x11` is also a non-canonical length for remaining 1, and truncation still wins. Kind 7 (`0x1C`) stays `ReservedLength` when the length code is 3, and `ReservedKind` when the length code is legal. Keep `Error::UnsupportedKind` in the enum. `decode_block` does not return it.

`spec/vectors/index.toml` does not gain these rows. The block tests load the hex file the way they load `block-baseline-headers.hex`.

### Decoder independence

`crates/sxp-kernel/tests/formula_block.rs` imports `sxp_kernel` only. It does not mention a Codebook, a recipe, or a hash. `read_blocked_file` on each winning trace's blocked encoding, built with `write_blocked_file` from the kernel, returns the trace. The file starts `8d 01`. The codebook hash is not an argument to that call.

`encode_codebook` on each winning trace starts with `8d 01` and round-trips through `read_blocked_file`. Codebook `version` is 1 and the file version nibble is 1. They are different fields: the test decodes the file without passing `content_sha256`.

### Project Structure Notes

- `crates/sxp-kernel/src/formula_wire.rs` — packer, `section_len`, `formula_section`. Private module. Re-export `section_len` and `formula_section` from `lib.rs`.
- `crates/sxp-kernel/src/block.rs` — `BlockKind`, `encode_block`, `decode_block`, `MAX_ENCODED_BLOCK`.
- `crates/sxp-kernel/src/polynomial.rs`, `trigonometric.rs`, `recurrence.rs` — `pub(crate)` accessors. `predict_span` on `Recurrence` is public.
- `crates/sxp-kernel/tests/formula_block.rs` — wire vectors. `crates/sxp-kernel/tests/block.rs` — the four `UnsupportedKind` asserts change to `TruncatedBlock`.
- `crates/sxp-encoder/src/codebook.rs` — recipe, dedup, hash.
- `crates/sxp-encoder/src/select.rs` — `SearchCandidate`, span kind, `select_codebook`, `encode_codebook`, `span_cells`.
- `crates/sxp-encoder/src/error.rs` — `AbsentRecipe`, `UnresolvedProduction`, `CapExceeded`, `MissingFamily`. `Display` is the variant name. `Kernel` still forwards the kernel name.
- `crates/sxp-encoder/src/lib.rs` — re-exports. Lint block unchanged.
- `crates/sxp-encoder/tests/codebook.rs` and the `select.rs` updates.
- `src/` does not call `assert`, `unwrap`, `expect`, or `panic`. Integration tests may `assert`.
- Index with `get`. No float literal in `codebook.rs` or `formula_wire.rs`.

### Previous story intelligence

Story 1.9 left formula kinds, ids at or above 259, and field writers for this story. Its header prune is the only allowed shortcut. A cost error, including `BlockTooLarge`, returns immediately. Do not catch it and keep searching.

`Error::Display` is the variant name. Match that in the new encoder errors.

`threads == 0` is one calling thread. It never reaches `ThreadPoolBuilder`. `threads >= 2` uses a local pool of that size. `ThreadPoolBuilder::build` failure is `Error::ThreadPool`.

`N >= LENGTH_LIMIT` is `Error::Kernel(LengthOutOfRange)` before the search. `N == 0` is Whole-file raw, file length 7, and does not call `write_blocked_file`. An empty baseline list after that check is `NoCandidate`. An empty codebook is `MissingFamily` at assemble time, not at search time. `select_codebook` with an empty baseline slice and a valid codebook is legal: only formulas are candidates.

Whole-file raw still wins when `block_bytes >= N`. The three winning traces are shorter than their blocked size only if a formula Block is somehow larger than the trace. Assert `whole_file_raw` is false on those three, and assert the file round-trips. If a trace comes out Whole-file raw, the formula lost and the test fails.

Story 1.8's `decode_block` docs list kind `>= 4` as `UnsupportedKind`. Replace that list with the check order above.

Both-platform `./ci.sh` is the gate. Story 1.9 ran aarch64 under `qemu-aarch64` with `/tmp/serexpai-cross/bin` on `PATH`. Seed for story 1.9 generated bytes stays `0x5111_0009`. This story's traces come from `predict`, not from that generator. Do not retune the story 1.9 brute-force lengths. Add the formula oracle only on the 32, 48, and 64 byte traces.

The story 1.9 oracle and the header-prune counting fixtures must still pass after `candidate_id` returns `Option` and `SpanTable` stores `kind`.

### Git intelligence

`HEAD` at story creation is `e6bdfbc680e3a4db9701de6b1d2b3edee1ddc0f9`, `feat: select baselines by exact size (v0.0.15)`. Recent titles:

- `e6bdfbc` feat: select baselines by exact size (v0.0.15)
- `08af68a` feat: round-trip one baseline block (v0.0.14)
- `c05a8b0` feat: adapt an order-0 probability model (v0.0.13)
- `d06ecd6` feat: code decisions with the binary range coder (v0.0.12)
- `aa2fb8f` feat: evaluate a recurrence formula (v0.0.11)

Capture `baseline_commit` when implementation starts and do not overwrite it.

### Latest technical information

Stay on toolchain `1.99.1`, rayon `1.12.0`, and `sha2` `0.11.1`. `sha2 = "=0.11.1"` is already the pin in `sxp-bench` and in the kernel dev-dependencies. Do not `rustup update`. Do not take a newer hash crate. SHA-256 has no parameter to choose. No new web API is involved.

### Project context reference

No `project-context.md` exists. The kernel lint block, AD-8, AD-10, AD-13, and spec seed §3.2 are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.10, S1.10-AC01–AC07]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-4, AD-8, AD-10, AD-13, AD-15]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §3.1, §3.2, §4]
- [Source: `docs/decision_logs/decision_log_2026_W44.md` — D-01, D-02, D-07, D-09, 2026-11-01]
- [Source: `_bmad-output/implementation-artifacts/1-9-select-baselines-by-exact-size.md` — DP key, header prune, ids 0..=258]
- [Source: `crates/sxp-kernel/src/block.rs` — `encode_block`, `decode_block`, `next_block`, `MAX_ENCODED_BLOCK`]
- [Source: `crates/sxp-encoder/src/select.rs` — `choose_candidate`, `SpanTable`, `select_baselines`]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- Fixture polynomial and recurrence sections are 15 and 13 bytes. A same-span tie at cost 5 uses a degree-0 cw-1 polynomial, whose section is 3 bytes, so header prune still evaluates it.
- `(2048, 2048, 1)` is 4097 formulas and is `CapExceeded`. `(2048, 2047, 1)` is the legal total.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- The kernel frames polynomial, trigonometric, and recurrence Blocks. Kinds 0–3 keep their existing headers. Short kind 4/5/6 inputs are `TruncatedBlock` before a domain check.
- `generate_codebook("fixture-v1")` emits three formulas, ids 259–261. `production` and `benchmark` return `UnresolvedProduction`. The content hash is `a8f53ca8a7799035822e1fe6bf27962136b6af9ccf9e75b8785562d638cfef9b`.
- Each fixture formula wins its own trace against all baselines and the other formulas. Header prune records `Skipped` as a cell, not a fake size. Whole-file raw still wins an empty input.
- `./ci.sh` passed on `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` on 2026-11-03. `baseline_commit` is `e6bdfbc680e3a4db9701de6b1d2b3edee1ddc0f9`.
- Code review applied 7 patches. Domain checks now win over padding. File round-trips cover the three nominals and the three winning traces. A repeated candidate id with two kinds is `NoCandidate`. The edge-case pass found nothing unhandled.

### File List

- Cargo.lock
- crates/sxp-encoder/Cargo.toml
- crates/sxp-encoder/src/codebook.rs
- crates/sxp-encoder/src/error.rs
- crates/sxp-encoder/src/lib.rs
- crates/sxp-encoder/src/select.rs
- crates/sxp-encoder/tests/codebook.rs
- crates/sxp-encoder/tests/select.rs
- crates/sxp-kernel/src/block.rs
- crates/sxp-kernel/src/formula_wire.rs
- crates/sxp-kernel/src/lib.rs
- crates/sxp-kernel/src/polynomial.rs
- crates/sxp-kernel/src/recurrence.rs
- crates/sxp-kernel/src/trigonometric.rs
- crates/sxp-kernel/tests/block.rs
- crates/sxp-kernel/tests/formula_block.rs
- spec/vectors/codebook-fixture-v1.sha256
- spec/vectors/formula-blocks.hex
- _bmad-output/implementation-artifacts/1-10-search-a-regenerable-formula-codebook.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-11-03: Story context created. Fixture formulas are the three measured traces. Production recipe stays unresolved.
- 2026-11-03: Implemented formula Block framing, the fixture codebook, and exact-size search. Both target CI runs passed.
- 2026-11-03: Addressed code review findings — 7 patches applied. The edge-case layer returned no findings.

## Story completion status

done. Formula Blocks, the fixture codebook, and exact-size search are implemented. Code review applied 7 patches. Both target CI runs passed again after those patches.
