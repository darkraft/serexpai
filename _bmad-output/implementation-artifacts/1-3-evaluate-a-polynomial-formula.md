---
baseline_commit: fb5a3fdb2b5d9c3849b35e3931666321eaaa94cb
---

# Story 1.3: Evaluate a polynomial formula

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want polynomial predictions to agree on every supported platform,
so that a Formula's ŷ stream is the same byte sequence on every host that will later encode it.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or the committed hash must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence is enough for story 1.4 to start (elicitation amendment 1, 2026-11-01). It does not close AC04.

1. **S1.3-AC01.** G0 already defines degree `0..=15`, Q `8..=24`, exactly degree + 1 coefficients in `[−2^31 + 1, 2^31 − 1]`, and Block-local position `n` in `0..=2047`. That text is `bitstream-spec-seed.md` §4 and §4.1. Do not edit the seed, and do not add a second normative document. Invalid inputs return `Error::InvalidParameter`. The API takes `i32` coefficients, `u8` degree and Q, and `u16` position; the rustdoc states where a narrower type cannot represent a rejected neighbour (`+2^31`, a negative degree, a negative `n`).
2. **S1.3-AC02.** Evaluation is `u = n << 5`, then signed-i64 Horner `acc = c[degree]`, and for `k` from `degree − 1` down to `0`, `acc = ((acc · u) >> 16) + c[k]`, then `value = (acc + (1 << (Q − 1))) >> Q`, then `ŷ = rem_euclid(value, 256)`. Shifts are arithmetic floor shifts. No floating point. No `/` and no `%` in the evaluator. `rem_euclid` is the only remainder.
3. **S1.3-AC03.** The fixtures below are the expected values. They cover degrees 0 and 15, both Q limits, both coefficient-domain endpoints, `n = 0` and `n = 2047`, and rounding below, at, and above the positive and negative halfway cases. `Q = 8` and `c0 = −128` produces `value = 0`. Neighbours just outside the domain return `Error::InvalidParameter` and are not clamped or wrapped.
4. **S1.3-AC04.** The boundary test states and checks the intermediate-width bound. Both platforms match the committed prediction hash. A Formula's predictions for `n = 0..256` do not take a Block length, and they differ from the same coefficients evaluated with the 256-byte scaling `u = n << 8`.

## Tasks / Subtasks

- [x] **Polynomial type and Horner** (AC: 1, 2)
  - [x] Add `crates/sxp-kernel/src/polynomial.rs`. In `lib.rs` add only `mod polynomial;` and one `pub use`. Do not edit the lint block. Do not edit the G0 hash line. The module-doc sentence may name the polynomial evaluator; the hash stays `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`.
  - [x] `pub struct Polynomial` with private fields `degree: u8`, `q: u8`, and `coefficients: [i32; 16]`. Derive `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. Coefficients are lowest power first: index `0` is `c0`, index `degree` is the leading coefficient. Unused slots stay `0` and are never read.
  - [x] `pub fn new(degree: u8, q: u8, coefficients: &[i32]) -> Result<Self, Error>`. Reject, with `Error::InvalidParameter` and no other variant, when `degree > 15`, `q` is outside `8..=24`, `coefficients.len() != usize::from(degree) + 1`, or any included coefficient equals `i32::MIN`. Copy the slice into the array with `get`. Do not accept a longer slice and ignore the tail.
  - [x] Rustdoc on `new` states: `+2^31` does not fit in `i32` and cannot be passed; a negative degree does not fit in `u8`. `i32::MAX` (`2^31 - 1`) is legal. `i32::MIN` (`-2^31`) is representable and is rejected. Q is the mathematical Q in `8..=24`, not the stored `Q - 8` field. Rustdoc on `value` and `predict` states that a negative `n` does not fit in `u16`.
  - [x] `pub fn value(&self, n: u16) -> Result<i64, Error>` and `pub fn predict(&self, n: u16) -> Result<u8, Error>`. `n > 2047` is `Error::InvalidParameter`. `predict` is `value(n)?.rem_euclid(256) as u8`. There is no Block-length parameter.
  - [x] Horner, in `value` only: `u = i64::from(n) << 5`. `acc` starts at `i64::from` of coefficient `degree`. For `k` from `degree − 1` down to `0`, `acc = ((acc * u) >> 16) + i64::from(coefficient k)`. Then `value = (acc + (1i64 << u32::from(q - 1))) >> u32::from(q)`. Read each coefficient with `get`; a missing slot is `Error::InvalidParameter`, not a panic and not a zero fill.
  - [x] Use plain `i64` `*` and `>>`. Do not use `wrapping_mul`, `overflowing_mul`, `checked_mul`, `wrapping_shr`, `/`, or `%`. Do not call `signed_residual`, `residual_symbol`, `reconstruct`, or `direct_symbol`.
  - [x] No `unsafe`. No allocator. No new dependency. No float.
- [x] **Typed rejection** (AC: 1, 3)
  - [x] Add `InvalidParameter` to `Error` in `crates/sxp-kernel/src/error.rs`, immediately before `Io`. `name()` returns `"InvalidParameter"`. `Display` stays the variant name. Do not add a payload, a source string, or `#[non_exhaustive]`.
  - [x] Do not edit `tests/support/mod.rs`. `named_error` matches `&str` and does not need a new arm. Do not add a row to `spec/vectors/index.toml`.
  - [x] Tests reject these neighbours and accept nothing clamped into range: `degree = 16`; `q = 7` and `q = 25`; `n = 2048`; `i32::MIN` as a coefficient; a slice shorter by one than `degree + 1`; a slice longer by one. Include `degree = 0` with an empty slice and with two coefficients. A rejected `new` or `value` does not produce a `ŷ`.
- [x] **Pinned fixtures** (AC: 2, 3)
  - [x] Add `crates/sxp-kernel/tests/polynomial.rs`. Do not `mod support`.
  - [x] Assert `Polynomial::value` and `predict` against the `value` and `ŷ` columns. `acc` is not returned by the API. The test's `i128` recurrence must reproduce the `acc` column, and `Polynomial::value` must equal that recurrence's rounded value. Coefficients are listed lowest power first, using ASCII `-`. Do not compute the literals from `Polynomial`.

    | case | degree | Q | n | coefficients | acc | value | ŷ |
    |---|---:|---:|---:|---|---:|---:|---:|
    | c0 = -128 | 0 | 8 | 0 | `[-128]` | -128 | 0 | 0 |
    | below +half | 0 | 8 | 0 | `[127]` | 127 | 0 | 0 |
    | at +half | 0 | 8 | 0 | `[128]` | 128 | 1 | 1 |
    | above +half | 0 | 8 | 0 | `[129]` | 129 | 1 | 1 |
    | below -half | 0 | 8 | 0 | `[-129]` | -129 | -1 | 255 |
    | at -half | 0 | 8 | 0 | `[-128]` | -128 | 0 | 0 |
    | above -half | 0 | 8 | 0 | `[-127]` | -127 | 0 | 0 |
    | c = 2^31-1, Q=8 | 0 | 8 | 0 | `[2147483647]` | 2147483647 | 8388608 | 0 |
    | c = -2^31+1, Q=8 | 0 | 8 | 0 | `[-2147483647]` | -2147483647 | -8388608 | 0 |
    | below +half | 0 | 24 | 0 | `[8388607]` | 8388607 | 0 | 0 |
    | at +half | 0 | 24 | 0 | `[8388608]` | 8388608 | 1 | 1 |
    | above +half | 0 | 24 | 0 | `[8388609]` | 8388609 | 1 | 1 |
    | below -half | 0 | 24 | 0 | `[-8388609]` | -8388609 | -1 | 255 |
    | at -half | 0 | 24 | 0 | `[-8388608]` | -8388608 | 0 | 0 |
    | above -half | 0 | 24 | 0 | `[-8388607]` | -8388607 | 0 | 0 |
    | c = 2^31-1, Q=24 | 0 | 24 | 0 | `[2147483647]` | 2147483647 | 128 | 128 |
    | c = -2^31+1, Q=24 | 0 | 24 | 0 | `[-2147483647]` | -2147483647 | -128 | 128 |
    | shift wipes c1=1 | 1 | 8 | 2047 | `[0, 1]` | 0 | 0 | 0 |
    | shift keeps c1=129 | 1 | 8 | 2047 | `[0, 129]` | 128 | 1 | 1 |

  - [x] Alternating degree-15 coefficients: index `k` is `2147483647` when `k` is even and `−2147483647` when `k` is odd, for `k = 0..=15`. Assert:

    | Q | n | acc | value | ŷ |
    |---:|---:|---:|---:|---:|
    | 8 | 0 | 2147483647 | 8388608 | 0 |
    | 8 | 2047 | 8359991 | 32656 | 144 |
    | 24 | 0 | 2147483647 | 128 | 128 |
    | 24 | 2047 | 8359991 | 0 | 0 |

  - [x] `n = 0` depends only on `c0`. The degree-15, Q=8, `n = 0` row equals the degree-0 row for `c = 2147483647`. Reversing the alternating vector (leading coefficient placed at index 0) yields `value = −32656` and `ŷ = 112` at Q=8, `n = 2047`. Assert the forward row. A reversed implementation fails that `value`.
  - [x] The `c1 = 1` row is `value = 0` because `(1 · 65504) >> 16 = 0`. The `c1 = 129` row is `value = 1` because the product `8450016` shifts to `128`. Do not drop the `>> 16` to make the linear term visible. `ŷ` alone does not catch the dropped shift: both rows can still reduce to `0` or `1` for other bugs, so assert `value`.
- [x] **Width bound** (AC: 4)
  - [x] In the test, walk degree 15 at `n = 2047` for three coefficient vectors: all `2147483647`, all `−2147483647`, and the alternating vector. Use an `i128` recurrence in the test as the oracle. On every Horner step assert `|acc| < 2^35` and `|acc · u| < 2^51`, and assert `Polynomial::value` equals the `i128` oracle's rounded value.
  - [x] Write the bound next to those asserts: seed §4.1 says `|acc| ≤ (degree + 1)·2^31 < 2^35` and `|acc·u| < 2^51`. The strict `< 2^35` holds because each `|c| ≤ 2^31 − 1`, so `|acc| ≤ (degree + 1)·(2^31 − 1) = 2^35 − 16` at degree 15. Do not assert the spine's coarser "below 2^52". Do not edit the seed to fix the `2^31` wording.
  - [x] The implementation stays on `i64`. The measured worst product on these vectors is `2102834748549920`, which is less than `2^51` (`2251799813685248`) and less than `i64::MAX`. A wrapping multiply fails the oracle.
- [x] **Prediction hash and Block-length prefix** (AC: 4)
  - [x] Commit `spec/vectors/polynomial-degree15-q24.sha256` containing the lowercase hex digest and one trailing newline. The test loads it with `include_str!(...).trim()`. Do not add it to `spec/vectors/index.toml`.
  - [x] The digest is SHA-256 of 2048 bytes: the alternating degree-15 formula, Q=24, `n` from 0 through 2047, one `ŷ` per `n`. Pinned value: `ef3f1382e12c907e44d684035a02ff063d0ee18d36a165790887b1fc1860f65e`. First 16 bytes are `80 80 80 80 80 80 80 80 80 7f 7f 7f 7f 7f 7f 7f` (nine `128`s, then `127`; `n = 8` is still `128`, `n = 9` is `127`). Byte 255 is `114`. The independent oracle is:

    ```python
    import hashlib
    CMAX, CMIN = 2**31 - 1, -(2**31) + 1
    coeffs = [CMAX if k % 2 == 0 else CMIN for k in range(16)]

    def yhat(n):
        u, acc = n << 5, coeffs[15]
        for k in range(14, -1, -1):
            acc = ((acc * u) >> 16) + coeffs[k]
        return ((acc + (1 << 23)) >> 24) % 256

    blob = bytes(yhat(n) for n in range(2048))
    assert blob[:16].hex() == "8080808080808080807f7f7f7f7f7f7f"
    assert blob[255] == 114
    print(hashlib.sha256(blob).hexdigest())
    ```

    If the Rust test disagrees with the file, rerun this command. Its stdout is the authority. Do not regenerate the artifact from `Polynomial`.
  - [x] Hash with the existing dev-dependency `sha2 = "=0.11.1"`. Encode hex the same way as `tests/residual.rs` (`write!(s, "{b:02x}")` or raw digest bytes). Do not `format!("{:x}", digest)`. Do not add a crate. Do not run `cargo update`.
  - [x] `predict` has no length argument. Assert the literal prefix, not a comparison of `predict` with itself: `predict(0..=8)` is `128`, `predict(9..=15)` is `127`, and `predict(255)` is `114`. A test-side evaluation of the same coefficients with `u = n << 8` (the 256-byte scaling) yields `127` at `n = 2`, where `predict(2)` is `128`. Assert that pair. That contrast is the "prefix unchanged by actual Block length" check.
- [x] **Property tests** (AC: 2, 3)
  - [x] In `tests/polynomial.rs`, use the existing `proptest = "=1.12.0"` with `default-features = false`. Config: `cases: 256`, `rng_seed: RngSeed::Fixed(0x5111_0003)`, `failure_persistence: Some(Box::new(FileFailurePersistence::Off))`, remaining fields from `Config::default()`. Do not change seeds `0x5111_0001` or `0x5111_0002`. Do not use regex strategies.
  - [x] Draw `degree` in `0..=15`, `q` in `8..=24`, `n` in `0..=2047`, and `degree + 1` coefficients in `i32::MIN + 1..=i32::MAX`. Assert `value` matches the `i128` oracle, `predict` equals `value.rem_euclid(256)` as `u8`, and every Horner product stays inside the bound above.
  - [x] One strategy also draws one illegal neighbour (`degree` `16..=255`, or `q` outside `8..=24`, or a coefficient forced to `i32::MIN`, or `n` in `2048..=65535`) and asserts `Error::InvalidParameter`.
- [x] **Both platforms** (AC: 4)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands, `git rev-parse HEAD`, and that the polynomial hash test passed.
  - [x] `aarch64-unknown-linux-gnu` links with `aarch64-linux-gnu-gcc` and runs under `qemu-aarch64` (`.cargo/config.toml`). If those are not on `PATH`, put `/tmp/serexpai-cross/bin` ahead of `PATH`, as story 1.2 did. Do not edit `.cargo/config.toml`.
  - [x] Leave this subtask unchecked if either target did not run. Do not set the story Status to `done`. `dev-story` sets `review`. Code review sets `done` only when both runs are recorded.
  - [x] Do not edit `bitstream-spec-seed.md`, `CHANGELOG.md`, the workspace version, `residual.rs`, `raw.rs`, or story 1.1/1.2 tests. Do not commit unless the user asks.

### Review Findings

- [x] [Review][Patch] Width-bound text omits the seed identity and `2^35 - 16` [crates/sxp-kernel/tests/polynomial.rs:18]
- [x] [Review][Patch] `i32::MIN` is rejected only for a one-coefficient polynomial [crates/sxp-kernel/tests/polynomial.rs:163]

## Dev Notes

### What this story builds

One validated polynomial evaluator in `sxp-kernel`. It returns the rounded integer and the prediction byte for a Block-local position. It does not residual-code that byte, frame a Block, or serialize a Formula.

Out of scope: trigonometric and recurrence evaluators (stories 1.4 and 1.5); the range coder (1.6); the probability model (1.7); `encode_block` and any Block header (1.8); Codebook search and the on-wire layout `degree`, `Q − 8`, `cw − 1`, coefficients (1.10). D-07 stays open: this story accepts every legal coefficient, not a measured `cw` inside the cap. D-09 stays open: no production Codebook recipe. G0 for this untagged clause is the seed. Do not write `spec/bitstream.md`.

### Arithmetic

Governing text is seed §4 and §4.1, which elaborates AD-5. `u = n << 5` is always the 2048-point scale. `n = 2047` gives `u = 65504`, which is less than `2^16`. A short Block does not change `u`.

```
acc = c[degree]
for k in degree-1 down to 0:
    acc = ((acc * u) >> 16) + c[k]
value = (acc + (1 << (q - 1))) >> q
ŷ = rem_euclid(value, 256)
```

Rounding adds the positive half-step, then arithmetic-shifts. That rounds halfway cases toward `+∞`, including negative halfway: `Q = 8`, `c0 = −128` gives `value = 0`, not `−1`. Rust `>>` on `i64` is an arithmetic right shift (floor division by `2^k`). Rust `%` truncates toward zero and is the wrong operation. `(-1i64).rem_euclid(256)` is `255`.

The final remainder divisor is the positive constant `256`, so `rem_euclid` does not hit the `i64::MIN` / `-1` panic.

At `n = 0`, `u = 0`, and the Horner walk collapses to `c0`. Higher coefficients must not change that prediction. That is also why `value` is part of the public API: several domain endpoints share a `ŷ` of `0` or `128` and are only distinguished before the remainder.

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable` on `src/`. Tests may use `expect` and `assert`.

Public surface that must keep its signatures: `read_whole_file_raw`, `write_whole_file_raw`, `encode_whole_file_raw_header`, `decode_header`, `Header`, `HeaderBytes`, `encode_leb128`, `decode_leb128`, `Leb128`, `LENGTH_LIMIT`, `Crc32`, `crc32_iso_hdlc`, `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, and every existing `Error` variant plus `Error::name`.

`sxp-kernel` has an empty `[dependencies]` table. `ci.sh` fails unless `cargo tree -e normal` and `-e build` for `sxp-kernel` are each one line, and unless `crates/sxp-kernel/build.rs` is absent. `sha2` and `proptest` are already dev-dependencies. Do not add another.

`Error::Display` is the variant name only. Follow that for `InvalidParameter`.

### Project Structure Notes

- Evaluator: `crates/sxp-kernel/src/polynomial.rs`.
- Error variant: `crates/sxp-kernel/src/error.rs` only.
- Export: `crates/sxp-kernel/src/lib.rs`, one `mod` and one `pub use`.
- Tests: `crates/sxp-kernel/tests/polynomial.rs`.
- Hash artifact: `spec/vectors/polynomial-degree15-q24.sha256`.
- Do not put polynomial math in `sxp-encoder`. The encoder will call this type later. Do not add a `Formula` enum that also covers trigonometric or recurrence.

### Previous story intelligence

Story 1.2 put the residual in its own kernel module, exported it with `pub use`, and tested it from `tests/` without `mod support`. The exhaustive artifact is a one-line SHA-256 file loaded by `include_str`, not a `[[vector]]` row, because `support` asserts 24 index records. Clippy `-D warnings` rejects `unwrap` in `src/` and rejected an `unreachable` pattern in the vector parser. `as u8` after `rem_euclid` is the existing cast. Integration tests own `proptest` and `sha2`; the library target stays dependency-free.

Story 1.2's review patches were exact checks (signed `δ` on every pair, reject blank and escaped vector fields). Prefer an exact `value` literal over an approximate `ŷ` check. The same review will reject a fixture that can pass for a swapped coefficient order.

### Git intelligence

`HEAD` is `fb5a3fdb2b5d9c3849b35e3931666321eaaa94cb`, `feat: round-trip whole-file raw and reconstruct predictions (v0.0.8)`. The working tree was clean when this story was written. Story 1.1 framing and story 1.2 residual are in that commit. Do not revert them. Recent history is docs and that kernel commit; there is no polynomial module to extend.

### Latest technical information

Stay on the pinned toolchain `1.99.1` (`rust-toolchain.toml`). Rust 1.99.1 (2026-10-15) does not change `i64` shift or `rem_euclid`. Signed `>>` is an arithmetic right shift. `rem_euclid` returns a non-negative remainder. Do not `rustup update`. Do not add `num-traits` or a big-integer crate. The test oracle's `i128` and the committed hash replace an external numeric library.

### Project context reference

No `project-context.md` exists in this tree. The kernel lint block in `lib.rs` and AD-1 / AD-5 in the architecture spine are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.3, FR-3, D-01, D-07, elicitation amendments 1 and 3]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §3.2 Polynomial row, §4, §4.1]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-4, AD-5, AD-6, crate table]
- [Source: `_bmad-output/implementation-artifacts/1-2-reconstruct-every-byte-from-its-prediction.md` — module export, hash artifact, proptest profile, both-platform gate]
- [Source: Rust reference, operator expressions — signed `>>` is arithmetic; `%` truncates toward zero]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created
- `Polynomial` in `sxp-kernel` validates degree `0..=15`, Q `8..=24`, exactly `degree + 1` lowest-power-first `i32` coefficients, and rejects `i32::MIN`. `value` is signed-i64 Horner with `u = n << 5` and arithmetic shifts. `predict` is `rem_euclid(value, 256)`. No Block-length argument. `Error::InvalidParameter` is the only new error.
- Pinned fixtures, the `i128` width bound (`|acc| < 2^35`, `|acc·u| < 2^51`), and SHA-256 `ef3f1382e12c907e44d684035a02ff063d0ee18d36a165790887b1fc1860f65e` pass. The 256-byte scaling `u = n << 8` differs at `n = 2` (`127` vs `128`).
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-01, including `prediction_hash_and_block_length_prefix`.
- `./ci.sh aarch64-unknown-linux-gnu` passed on 2026-11-01 under `qemu-aarch64`, including the same hash test. Linker and runner came from `/tmp/serexpai-cross/bin` ahead of `PATH`. `HEAD` is still `fb5a3fdb2b5d9c3849b35e3931666321eaaa94cb`. This story is uncommitted.
- Code review patches applied: the width-bound comment states the seed identity and `2^35 - 16`, and `i32::MIN` is rejected at the leading slot of a degree-15 formula, at a middle slot, and at a random index in the property test. Both CI targets were rerun after those patches and passed.

### File List

- `crates/sxp-kernel/src/polynomial.rs`
- `crates/sxp-kernel/src/error.rs`
- `crates/sxp-kernel/src/lib.rs`
- `crates/sxp-kernel/tests/polynomial.rs`
- `spec/vectors/polynomial-degree15-q24.sha256`

### Change Log

- 2026-11-01: Kernel polynomial evaluator, invalid-parameter rejection, pinned fixtures, width bound, and both-platform prediction hash.
- 2026-11-01: Addressed code review findings — 2 patches applied.

## Story completion status

done. Both target CI runs passed after the review patches, including the polynomial hash test. Formula framing and residual wiring stay with later stories.
