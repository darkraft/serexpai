---
baseline_commit: 09f0f5173e80ea79fae335598b8e925b0e3e2484
---

# Story 1.4: Evaluate a trigonometric formula

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want trigonometric predictions to share one normative table and interpolation rule,
so that a Formula's ŷ stream is the same byte sequence on every host that will later encode it.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or the committed hash must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence is enough for story 1.5 to start (elicitation amendment 1, 2026-11-01). It does not close AC05.

1. **S1.4-AC01.** G0 already defines order `1..=16`, Q `8..=24`, unsigned 32-bit `omega`, exactly `2·order + 1` coefficients, Block-local `n` in `0..=2047`, and the evaluator domain `|c| < 2^28`. That text is `bitstream-spec-seed.md` §3.2 and §4.2. Do not edit the seed, and do not add a second normative document. The `[PROPOSED]` tag on table size stays; this story implements the 1025-entry Q15 table the research already confirmed, and removing the tag is a later spec-freeze edit. D-01 (approved 2026-11-01) caps stored `cw` at 29. This story does not serialize `cw`. D-07 stays open. Invalid inputs return `Error::InvalidParameter`. The API takes `i32` coefficients, `u8` order and Q, `u32` omega, and `u16` position. Rustdoc states where a narrower type cannot represent a rejected neighbour (`omega ≥ 2^32`, a negative order, a negative `n`) and where a representable neighbour is rejected (`±2^28`, order `0`).
2. **S1.4-AC02.** Evaluation is `theta = (n · omega) mod 2^32`, `S = c0 << 15`, then for `k = 1..=order`, `p = (k · theta) mod 2^32` and `S += a_k · sin_lut(p) + b_k · sin_lut((p + 2^30) mod 2^32)`. Then `value = (S + (1 << (Q + 14))) >> (Q + 15)` and `ŷ = rem_euclid(value, 256)`. `S` and the products are signed `i64`. Phase reduction is the low 32 bits of a `u64` product. Shifts are arithmetic floor shifts. No floating point. No `/` and no `%` in the evaluator. `rem_euclid` is the only remainder.
3. **S1.4-AC03.** Commit a 1025-entry little-endian `i16` table at `spec/sine_lut.bin` with SHA-256 `6267f51aebd57812c68a389bb4de8f2c85da816db4338a01b21e7455ec7ca9df`. Entry 0 equals entry 1024, and every entry satisfies `|entry| ≤ 32767`. Interpolation is `i = p >> 22`, `f = (p >> 6) & 0xFFFF`, `v0 + (((v1 − v0) · f + (1 << 15)) >> 16)`. The Rust evaluator includes that file. It does not call `sin` and does not regenerate the table. The generator below is not normative.
4. **S1.4-AC04.** The fixtures below are the expected values. They cover orders 1 and 16, both Q limits, both coefficient-domain endpoints and the rejected neighbours `±2^28`, `omega` 0 and `2^32 − 1`, phase wrapping, the four quadrant knots, the last interpolation segment, interpolation rounding below/at/above the half step, final rounding below/at/above both halfway cases, and `n = 2047`. Neighbours just outside the domain return `Error::InvalidParameter` and are not clamped or wrapped.
5. **S1.4-AC05.** `check_sine_lut` returns `Ok(())` only for the committed 2050 bytes and `Error::InvalidParameter` for a one-byte mutation and for any other length. Both platforms match the committed prediction hash. The boundary test states and checks the intermediate-width bound. Polynomial, residual, and whole-file raw tests still pass.

## Tasks / Subtasks

- [x] **Normative sine table** (AC: 3, 5)
  - [x] Generate `spec/sine_lut.bin` with the script in Dev Notes. 2050 bytes, little-endian `i16`, 1025 entries. Write `spec/sine_lut.sha256` as the lowercase digest, one trailing newline: `6267f51aebd57812c68a389bb4de8f2c85da816db4338a01b21e7455ec7ca9df`. Do not add either file to `spec/vectors/index.toml`.
  - [x] If the script prints a different digest, stop. Do not edit the pin. Python `round` is half-to-even; the nearest sample is `0.00124` away from a half-integer, so `math.sin` and the decimal series agree.
  - [x] Add `crates/sxp-kernel/src/trigonometric.rs`. Load the table with `include_bytes!("../../../spec/sine_lut.bin")`. Decode entry `i` with `get` and `i16::from_le_bytes`. Do not index. Do not use `unsafe`. A missing pair is `Error::InvalidParameter`.
  - [x] `pub fn check_sine_lut(bytes: &[u8]) -> Result<(), Error>`. `Ok(())` only when `bytes` equals the included bytes. Any other length or any differing byte is `Error::InvalidParameter`. Do not hash inside the kernel. Do not add `sha2` to `[dependencies]`.
  - [x] In `lib.rs` add only `mod trigonometric;` and `pub use trigonometric::{Trigonometric, check_sine_lut};`. The module-doc sentence may name the trigonometric evaluator. The G0 hash stays `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`. Do not edit the lint block.
- [x] **Trigonometric type** (AC: 1, 2)
  - [x] `pub struct Trigonometric` with private fields `order: u8`, `q: u8`, `omega: u32`, and `coefficients: [i32; 33]`. Derive `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. Layout, lowest index first: index `0` is `c0`; for `k = 1..=order`, index `2k − 1` is `a_k` and index `2k` is `b_k`. Unused slots stay `0` and are never read.
  - [x] `pub fn new(order: u8, q: u8, omega: u32, coefficients: &[i32]) -> Result<Self, Error>`. Reject, with `Error::InvalidParameter` and no other variant, when `order` is outside `1..=16`, `q` is outside `8..=24`, or `coefficients.len() != 2 * usize::from(order) + 1`. Reject a coefficient when `coefficient <= -(1 << 28)` or `coefficient >= (1 << 28)`. Copy with `get`. Do not accept a longer slice and ignore the tail. Every `u32` omega is legal.
  - [x] Do not add an error variant. `InvalidParameter` already exists. Do not edit `tests/support/mod.rs`.
  - [x] Rustdoc on `new`: a negative order does not fit in `u8`; order `0` fits and is rejected. `omega ≥ 2^32` does not fit in `u32`; `0` and `u32::MAX` are legal. `±(2^28 − 1)` are legal. `±2^28` fit in `i32` and are rejected. `i32::MIN` and `i32::MAX` are rejected because they are outside `|c| < 2^28`, not because of a special `i32` rule. Q is the mathematical Q in `8..=24`, not the stored `Q − 8` field. Stored `cw ≤ 29` is the story 1.10 wire cap; this constructor does not take `cw`. Rustdoc on `value` and `predict`: a negative `n` does not fit in `u16`.
  - [x] `pub fn value(&self, n: u16) -> Result<i64, Error>` and `pub fn predict(&self, n: u16) -> Result<u8, Error>`. `n > 2047` is `Error::InvalidParameter`. `predict` is `value(n)?.rem_euclid(256) as u8`. There is no Block-length parameter.
  - [x] Phase, in `value` only. `theta = (u64::from(n) * u64::from(omega)) as u32`. `S` starts at `c0 << 15` in `i64`. For `k` from 1 through `order`, `p = (u64::from(k) * u64::from(theta)) as u32`, `p_cos = (u64::from(p) + (1u64 << 30)) as u32`, then `S += a_k * sin_lut(p) + b_k * sin_lut(p_cos)`. The `as u32` casts are the specified `mod 2^32`. Do not use `wrapping_mul`. Then `value = (S + (1i64 << u32::from(q + 14))) >> u32::from(q + 15)`.
  - [x] `sin_lut`: `i = p >> 22` (0..=1023), `f = i64::from((p >> 6) & 0xFFFF)`, `v0` and `v1 = LUT[i + 1]` as `i64`, result `v0 + (((v1 - v0) * f + (1i64 << 15)) >> 16)`. Read `v0` and `v1` with `get`.
  - [x] Use plain `i64` `*` and `>>` for `S` and for the interpolation product. Do not use `wrapping_mul`, `overflowing_mul`, `checked_mul`, `/`, or `%`. Do not call `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, or `Polynomial`.
  - [x] No `unsafe`. No allocator. No new dependency. No float. The table is static data, not a decode-time allocation.
- [x] **Typed rejection** (AC: 1, 4)
  - [x] Add `crates/sxp-kernel/tests/trigonometric.rs`. Do not `mod support`.
  - [x] Reject these neighbours and accept nothing clamped into range: `order = 0` and `order = 17`; `q = 7` and `q = 25`; `n = 2048`; coefficient `1 << 28` and `-(1 << 28)` at `c0`, at `a_8` of an order-16 formula, and at `b_16`; `i32::MIN` and `i32::MAX` at `c0`; a slice shorter by one than `2·order + 1`; a slice longer by one. Include order 1 with 1 coefficient and with 5, and order 16 with 32 and with 34. A rejected `new` or `value` does not produce a `ŷ`.
  - [x] Accept `±(2^28 − 1)` at `c0`. Accept `omega = 0` and `omega = u32::MAX`.
- [x] **Pinned fixtures** (AC: 2, 3, 4)
  - [x] Assert `value` and `predict` against the `value` and `ŷ` columns. The test's `i128` recurrence, reading `spec/sine_lut.bin` itself, must reproduce the `S` column, and `Trigonometric::value` must equal that recurrence's rounded value. Do not compute the literals from `Trigonometric`. Coefficients use ASCII `-`.
  - [x] Constant-only rows. `order = 1`, `omega = 0`, `n = 0`, coefficients `[c0, 0, 0]`. `S = c0 << 15`.

    | case | Q | c0 | S | value | ŷ |
    |---|---:|---:|---:|---:|---:|
    | below −half | 8 | -129 | -4227072 | -1 | 255 |
    | at −half | 8 | -128 | -4194304 | 0 | 0 |
    | above −half | 8 | -127 | -4161536 | 0 | 0 |
    | below +half | 8 | 127 | 4161536 | 0 | 0 |
    | at +half | 8 | 128 | 4194304 | 1 | 1 |
    | above +half | 8 | 129 | 4227072 | 1 | 1 |
    | c = 2^28−1 | 8 | 268435455 | 8796092989440 | 1048576 | 0 |
    | c = −(2^28−1) | 8 | -268435455 | -8796092989440 | -1048576 | 0 |
    | below +half | 24 | 8388607 | 274877874176 | 0 | 0 |
    | at +half | 24 | 8388608 | 274877906944 | 1 | 1 |
    | above +half | 24 | 8388609 | 274877939712 | 1 | 1 |
    | below −half | 24 | -8388609 | -274877939712 | -1 | 255 |
    | at −half | 24 | -8388608 | -274877906944 | 0 | 0 |
    | above −half | 24 | -8388607 | -274877874176 | 0 | 0 |
    | c = 2^28−1 | 24 | 268435455 | 8796092989440 | 16 | 16 |
    | c = −(2^28−1) | 24 | -268435455 | -8796092989440 | -16 | 240 |

  - [x] Harmonic rows. `Q = 8`. `S` is the pre-round accumulator.

    | case | order | omega | n | coefficients | S | value | ŷ |
    |---|---:|---:|---:|---|---:|---:|---:|
    | a at phase 0 | 1 | 0 | 0 | `[0, 129, 0]` | 0 | 0 | 0 |
    | b at phase 0 | 1 | 0 | 0 | `[0, 0, 129]` | 4226943 | 1 | 1 |
    | sine 90° | 1 | 1073741824 | 1 | `[0, 129, 0]` | 4226943 | 1 | 1 |
    | sine 180° | 1 | 2147483648 | 1 | `[0, 129, 0]` | 0 | 0 | 0 |
    | sine 270° | 1 | 3221225472 | 1 | `[0, 129, 0]` | -4226943 | -1 | 255 |
    | cosine of 90° | 1 | 1073741824 | 1 | `[0, 0, 129]` | 0 | 0 | 0 |
    | wrapped b2 | 2 | 2147483648 | 1 | `[0, 0, 0, 0, 129]` | 4226943 | 1 | 1 |
    | wrapped a2 | 2 | 2147483648 | 1 | `[0, 0, 0, 129, 0]` | 0 | 0 | 0 |
    | interp below | 1 | 2097088 | 1 | `[0, 4194304, 0]` | 419430400 | 50 | 50 |
    | interp half | 1 | 2097152 | 1 | `[0, 4194304, 0]` | 423624704 | 51 | 51 |
    | interp above | 1 | 2097216 | 1 | `[0, 4194304, 0]` | 423624704 | 51 | 51 |
    | last segment | 1 | 4294967295 | 1 | `[0, 8388608, 0]` | 0 | 0 | 0 |
    | n = 2047 quadrant | 1 | 3221225472 | 2047 | `[0, 129, 0]` | 4226943 | 1 | 1 |
    | same formula, n = 0 | 1 | 3221225472 | 0 | `[0, 129, 0]` | 0 | 0 | 0 |

  - [x] `b` at phase 0 is `value = 1` because `sin_lut(2^30) = 32767` and `129 · 32767 = 4226943`. `a` at phase 0 is `value = 0` because `sin_lut(0) = 0`. Swapping `a_k` with `b_k` fails those two rows. Do not copy the polynomial rule that `n = 0` ignores every coefficient after `c0`.
  - [x] Wrapped `b2` is `value = 1` because `p = (2 · 2^31) mod 2^32 = 0` and `sin_lut(2^30) = 32767`. An unmasked `k · theta` of `2^32` is not a `u32` phase. Assert `S`, not only `ŷ`.
  - [x] Interpolation half is `p = 2097152` (`i = 0`, `f = 32768`). `sin_lut` is `101`. The same product without `+ (1 << 15)` is `100`, and `value` would be `50`. Assert `value = 51`. Below (`f = 32767`) is `100` and `value = 50`. Above (`f = 32769`) is `101` and `value = 51`.
  - [x] Last segment is `p = 2^32 − 1` (`i = 1023`, `f = 65535`, `v0 = −201`, `v1 = 0`). Biased interpolation is `0`, so `value = 0`. Dropping the bias yields `−1` and `value = −1` at `a1 = 8388608`. Assert `value = 0`.
  - [x] `n = 2047`, `omega = 3221225472` is the 90° knot (`theta = 2^30`), `ŷ = 1`. The same coefficients at `n = 0` are `ŷ = 0`. The mistaken scale `theta = ((n << 5) · omega) mod 2^32` is `0` here, which is the `n = 0` prediction. Assert the pair `predict(2047) = 1` and `predict(0) = 0`. That contrast is the "phase uses `n`, not `u = n << 5`" check. `predict` has no length argument.
  - [x] Quadrant knots, read from the committed table: entry 0 = 0, entry 1 = 201, entry 256 = 32767, entry 512 = 0, entry 768 = −32767, entry 1023 = −201, entry 1024 = 0. Max `|LUT[i + 1] − LUT[i]|` is 201.
  - [x] Order-16 alternating coefficients: index `k` is `268435455` when `k` is even and `−268435455` when `k` is odd, for `k = 0..=32`. `omega = 16777217` (`0x01000001`). Assert:

    | Q | n | S | value | ŷ |
    |---:|---:|---:|---:|---:|
    | 8 | 0 | 149529285853200 | 17825280 | 0 |
    | 8 | 1 | 116637084245685 | 13904224 | 96 |
    | 8 | 2047 | 174557133935580 | 20808832 | 128 |
    | 24 | 0 | 149529285853200 | 272 | 16 |
    | 24 | 1 | 116637084245685 | 212 | 212 |
    | 24 | 2047 | 174557133935580 | 318 | 62 |

- [x] **Width bound** (AC: 5)
  - [x] In the test, walk order 16 at `n = 2047`, `omega = u32::MAX`, for three coefficient vectors: all `268435455`, all `−268435455`, and the alternating vector. Use an `i128` recurrence over the committed table. On every term assert `|c0 << 15| < 2^43` and `|a_k · sin_lut| < 2^43` and `|b_k · sin_lut| < 2^43`, and assert `|S| < 2^49`. Assert `Trigonometric::value` equals the rounded oracle.
  - [x] Write the bound next to those asserts: seed §4.2 says each term `< 2^28 · 2^15 = 2^43`, at most 33 terms, so `|S| < 2^49`, and `|c0 << 15| < 2^43`. The strict bound holds because `|c| ≤ 2^28 − 1` and `|LUT| ≤ 32767`, so `|c · LUT| ≤ (2^28 − 1) · 32767 = 8795824553985` and `|c0 << 15| ≤ (2^28 − 1) · 32768 = 8796092989440`, both `< 2^43 = 8796093022208`. `|S| ≤ 8796092989440 + 32 · 8795824553985 = 290262478716960 < 2^49`. Do not assert the spine's coarser "below 2^62". Do not edit the seed.
  - [x] The implementation stays on `i64`. A wrapping multiply fails the oracle.
- [x] **Table integrity and prediction hash** (AC: 3, 5)
  - [x] Decode `spec/sine_lut.bin` in the test. Assert length 2050, entry 0 equals entry 1024, the seven knot values above, `|entry| ≤ 32767`, and max adjacent absolute difference 201. Hash the bytes with `sha2 = "=0.11.1"` and compare to `include_str!(...).trim()` of `spec/sine_lut.sha256`. Encode hex with `write!(s, "{b:02x}")`. Do not `format!("{:x}", digest)`.
  - [x] `check_sine_lut` on those bytes is `Ok(())`. Flip byte 4 of a copy and assert `Error::InvalidParameter`. A 2049-byte slice and a 2051-byte slice are the same error.
  - [x] Commit `spec/vectors/trigonometric-order16-q8.sha256` containing `d465e3f2a906ca339a0be1f7192aa0d774d8e8e51b21c26a9e3996dd2b63d3f0` and one trailing newline. The test loads it with `include_str!(...).trim()`. Do not add it to `spec/vectors/index.toml`.
  - [x] The digest is SHA-256 of 2048 bytes: the alternating order-16 formula, Q=8, `omega = 16777217`, `n` from 0 through 2047, one `ŷ` per `n`. First 16 bytes are `00 60 40 40 20 80 80 00 00 a0 00 60 e0 c0 00 40`. Byte 255 is `160`. Byte 2047 is `128`. The independent oracle is the script in Dev Notes. If the Rust test disagrees with the file, rerun that script. Its stdout is the authority. Do not regenerate the artifact from `Trigonometric`.
- [x] **Property tests** (AC: 2, 4)
  - [x] Use the existing `proptest = "=1.12.0"` with `default-features = false`. Config: `cases: 256`, `rng_seed: RngSeed::Fixed(0x5111_0004)`, `failure_persistence: Some(Box::new(FileFailurePersistence::Off))`, remaining fields from `Config::default()`. Do not change seeds `0x5111_0001`, `0x5111_0002`, or `0x5111_0003`. Do not use regex strategies.
  - [x] Draw `order` in `1..=16`, `q` in `8..=24`, `n` in `0..=2047`, `omega` any `u32`, and `2·order + 1` coefficients in `-(2^28 − 1)..=2^28 − 1`. Assert `value` matches the `i128` oracle, `predict` equals `value.rem_euclid(256)` as `u8`, and every product and `|S|` stay inside the bounds above.
  - [x] One strategy also draws one illegal neighbour (`order` `0` or `17..=255`, or `q` outside `8..=24`, or one coefficient forced to `1 << 28`, `-(1 << 28)`, `i32::MAX`, or `i32::MIN`, or `n` in `2048..=65535`) and asserts `Error::InvalidParameter`. The forced coefficient is not only index 0.
- [x] **Both platforms** (AC: 5)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands, `git rev-parse HEAD`, and that the trigonometric hash test passed. Polynomial tests must still pass; do not edit them.
  - [x] `aarch64-unknown-linux-gnu` links with `aarch64-linux-gnu-gcc` and runs under `qemu-aarch64` (`.cargo/config.toml`). If those are not on `PATH`, put `/tmp/serexpai-cross/bin` ahead of `PATH`, as story 1.3 did. Do not edit `.cargo/config.toml`.
  - [x] Leave this subtask unchecked if either target did not run. Do not set the story Status to `done`. `dev-story` sets `review`. Code review sets `done` only when both runs are recorded.
  - [x] Do not edit `bitstream-spec-seed.md`, `CHANGELOG.md`, the workspace version, `polynomial.rs`, `residual.rs`, `raw.rs`, or story 1.1–1.3 tests. Do not commit unless the user asks.

### Review Findings

- [x] [Review][Patch] `value` docs call the rounded result the accumulator [crates/sxp-kernel/src/trigonometric.rs:66]
- [x] [Review][Patch] `±2^28` is not rejected at both `a_8` and `b_16` [crates/sxp-kernel/tests/trigonometric.rs:436]

## Dev Notes

### What this story builds

One validated trigonometric evaluator in `sxp-kernel`, plus the committed sine table it interpolates. It returns the rounded integer and the prediction byte for a Block-local position. It does not residual-code that byte, frame a Block, or serialize a Formula.

Out of scope: the recurrence evaluator (story 1.5); the range coder (1.6); the probability model (1.7); `encode_block` and any Block header (1.8); Codebook search and the on-wire layout `order − 1`, `Q − 8`, `cw − 1`, `omega`, coefficients (1.10). D-07 stays open: this story accepts every coefficient with `|c| < 2^28`, not a measured `cw` inside the cap. D-09 stays open: no production Codebook recipe. G0 for the untagged arithmetic is the seed. The table size is still tagged `[PROPOSED]` in the seed; implement the size this story's AC states, and do not edit the tag. Do not write `spec/bitstream.md`.

### Arithmetic

Governing text is seed §4.2, which elaborates AD-5 and AD-6. Phase uses `n`, not `u = n << 5`. A short Block does not change `theta`.

```
theta = (n * omega) mod 2^32
S = c0 << 15
for k in 1..=order:
    p = (k * theta) mod 2^32
    S += a[k] * sin_lut(p) + b[k] * sin_lut((p + 2^30) mod 2^32)
value = (S + (1 << (q + 14))) >> (q + 15)
ŷ = rem_euclid(value, 256)
```

`sin_lut(p) = v0 + (((v1 − v0) * f + (1 << 15)) >> 16)` with `i = p >> 22`, `f = (p >> 6) & 0xFFFF`, `v0 = LUT[i]`, `v1 = LUT[i + 1]`.

Rounding adds the positive half-step, then arithmetic-shifts. That rounds halfway cases toward `+∞`, including negative halfway: `Q = 8`, `c0 = −128` gives `value = 0`, not `−1`. Rust `>>` on `i64` is an arithmetic right shift. Rust `%` truncates toward zero and is the wrong operation.

The final remainder divisor is the positive constant `256`, so `rem_euclid` does not hit the `i64::MIN` / `-1` panic.

At `n = 0`, `theta = 0`, so every `a_k` is multiplied by `sin_lut(0) = 0`, but every `b_k` is multiplied by `sin_lut(2^30) = 32767`. `n = 0` still depends on the cosine coefficients.

`n * omega` for `n ≤ 2047` fits in `u64` (`< 2^43`). `k * theta` for `k ≤ 16` fits in `u64` (`< 2^36`). The low 32 bits are the phase. `|S|` fits in `i64` under the limit table. Do not widen the kernel to `i128`.

### Table generator

Run once from the repo root. Write stdout's raw bytes to `spec/sine_lut.bin`. The printed digest must equal the pin.

```python
import hashlib, math, struct, sys
K, SCALE = 1024, 32767
lut = [int(round(SCALE * math.sin(2 * math.pi * i / K))) for i in range(K + 1)]
assert lut[0] == lut[K] == 0
assert max(abs(v) for v in lut) <= 32767
assert max(abs(lut[i + 1] - lut[i]) for i in range(K)) == 201
blob = b"".join(struct.pack("<h", v) for v in lut)
assert len(blob) == 2050
sys.stdout.buffer.write(blob)
print(hashlib.sha256(blob).hexdigest(), file=sys.stderr)
```

Prediction-hash oracle, after the bin exists. Stdout is the digest.

```python
import hashlib, struct
blob = open("spec/sine_lut.bin", "rb").read()
lut = list(struct.unpack("<1025h", blob))

def sin_lut(p):
    i = (p >> 22) & 1023
    f = (p >> 6) & 0xFFFF
    v0, v1 = lut[i], lut[i + 1]
    return v0 + (((v1 - v0) * f + (1 << 15)) >> 16)

CMAX = 2**28 - 1
coeffs = [CMAX if k % 2 == 0 else -CMAX for k in range(33)]
omega = 0x01000001

def yhat(n):
    theta = (n * omega) & 0xFFFFFFFF
    S = coeffs[0] << 15
    for k in range(1, 17):
        p = (k * theta) & 0xFFFFFFFF
        S += coeffs[2 * k - 1] * sin_lut(p)
        S += coeffs[2 * k] * sin_lut((p + (1 << 30)) & 0xFFFFFFFF)
    return ((S + (1 << 22)) >> 23) % 256

out = bytes(yhat(n) for n in range(2048))
assert out[:16].hex() == "006040402080800000a00060e0c00040"
assert out[255] == 160 and out[2047] == 128
print(hashlib.sha256(out).hexdigest())
```

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable` on `src/`. Tests may use `expect` and `assert`.

Public surface that must keep its signatures: `read_whole_file_raw`, `write_whole_file_raw`, `encode_whole_file_raw_header`, `decode_header`, `Header`, `HeaderBytes`, `encode_leb128`, `decode_leb128`, `Leb128`, `LENGTH_LIMIT`, `Crc32`, `crc32_iso_hdlc`, `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, `Polynomial`, and every existing `Error` variant plus `Error::name`.

`sxp-kernel` has an empty `[dependencies]` table. `ci.sh` fails unless `cargo tree -e normal` and `-e build` for `sxp-kernel` are each one line, and unless `crates/sxp-kernel/build.rs` is absent. `sha2` and `proptest` are already dev-dependencies. Do not add another.

`Error::Display` is the variant name only.

### Project Structure Notes

- Evaluator: `crates/sxp-kernel/src/trigonometric.rs`.
- Export: `crates/sxp-kernel/src/lib.rs`, one `mod` and one `pub use`.
- Tests: `crates/sxp-kernel/tests/trigonometric.rs`.
- Normative table: `spec/sine_lut.bin` and `spec/sine_lut.sha256`.
- Prediction hash: `spec/vectors/trigonometric-order16-q8.sha256`.
- Do not put trigonometric math in `sxp-encoder`. Do not add a `Formula` enum that also covers polynomial or recurrence. Do not fold this module into `polynomial.rs`.

### Previous story intelligence

Story 1.3 put the polynomial in its own kernel module, exported it with `pub use`, and tested it from `tests/` without `mod support`. The prediction artifact is a one-line SHA-256 file loaded by `include_str`, not a `[[vector]]` row, because `support` asserts 24 index records. `Error::InvalidParameter` is already the rejection variant; do not add another. `as u8` after `rem_euclid` is the existing cast. Integration tests own `proptest` and `sha2`; the library target stays dependency-free.

Story 1.3's review patches were exact checks: the width-bound comment had to state the seed identity and the strict `2^35 − 16` figure, and `i32::MIN` had to be rejected at the leading slot, a middle slot, and a property-test index. Do the same here for `±2^28`, and state the strict `2^43` / `2^49` figures. Prefer an exact `value` literal over an approximate `ŷ` check. A fixture that can pass for a swapped `a_k`/`b_k` pair will be rejected.

The polynomial's `n = 0` collapse to `c0` does not apply. The polynomial's `u = n << 5` does not apply. Phase is `n * omega`.

### Git intelligence

`HEAD` is `09f0f5173e80ea79fae335598b8e925b0e3e2484`, `feat: evaluate a polynomial formula (v0.0.9)`. The working tree was clean when this story was written. Stories 1.1–1.3 are in that history. Do not revert them. There is no trigonometric module to extend. Recent titles:

- `09f0f51` feat: evaluate a polynomial formula (v0.0.9)
- `fb5a3fd` feat: round-trip whole-file raw and reconstruct predictions (v0.0.8)
- `5a58bbb` Add the pinned workspace and whole-file raw fixtures.
- `f1b1ed1` docs: record implementation-readiness decisions (v0.0.7)
- `a21fbd5` docs: add SerExpAI v1 epics and align Gain with the spec (v0.0.6)

### Latest technical information

Stay on the pinned toolchain `1.99.1` (`rust-toolchain.toml`). Do not `rustup update`. Rust 1.99.1 signed `>>` is an arithmetic right shift. `rem_euclid` returns a non-negative remainder. `u64 as u32` keeps the low 32 bits. Do not add `num-traits`, `libm`, or a big-integer crate. The test oracle's `i128` and the committed table replace an external numeric library. `proptest 1.12.0` and `sha2 0.11.1` are already pinned; do not run `cargo update`.

### Project context reference

No `project-context.md` exists in this tree. The kernel lint block in `lib.rs` and AD-5 / AD-6 in the architecture spine are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.4, FR-2, FR-3, D-01, D-07, elicitation amendments 1, 3, and 7]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §3.2 Trigonometric row, §4.2]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-5, AD-6, crate table]
- [Source: `_bmad-output/planning-artifacts/research/open-questions-research-2026-10-31.md` — §6.6, Appendix A.1, 1025-entry Q15 confirmation]
- [Source: `_bmad-output/implementation-artifacts/1-3-evaluate-a-polynomial-formula.md` — module export, hash artifact, proptest profile, both-platform gate, review patches]
- [Source: Rust reference, operator expressions — signed `>>` is arithmetic; `%` truncates toward zero; `as u32` from `u64` truncates]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created
- `Trigonometric` in `sxp-kernel` validates order `1..=16`, Q `8..=24`, every `u32` omega, and exactly `2·order + 1` coefficients in `(-(2^28 - 1))..=(2^28 - 1)`. Layout is `c0`, then `a_k`, `b_k`. `value` is signed-i64 accumulation with phase `(n * omega) mod 2^32` and the committed 1025-entry sine table. `predict` is `rem_euclid(value, 256)`. No Block-length argument. `check_sine_lut` rejects any other byte string. `Error::InvalidParameter` is reused; no new error variant.
- Pinned fixtures, the `i128` width bound (`|term| < 2^43`, `|S| < 2^49`), sine-table SHA-256 `6267f51aebd57812c68a389bb4de8f2c85da816db4338a01b21e7455ec7ca9df`, and prediction SHA-256 `d465e3f2a906ca339a0be1f7192aa0d774d8e8e51b21c26a9e3996dd2b63d3f0` pass. `n = 2047`, `omega = 3221225472`, `a1 = 129` predicts `1`; the same formula at `n = 0` predicts `0`, which is what `u = n << 5` would produce.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-01, including `table_integrity_and_prediction_hash` and the polynomial tests.
- `./ci.sh aarch64-unknown-linux-gnu` passed on 2026-11-01 under `qemu-aarch64`, including the same hash test. Linker and runner came from `/tmp/serexpai-cross/bin` ahead of `PATH`. `HEAD` is still `09f0f5173e80ea79fae335598b8e925b0e3e2484`. This story is uncommitted.
- Code review patches applied: `value` is documented as the rounded result, halfway toward `+∞`, and `±2^28` is rejected at both `a_8` and `b_16`. Both CI targets were rerun after those patches and passed.

### File List

- `crates/sxp-kernel/src/trigonometric.rs`
- `crates/sxp-kernel/src/lib.rs`
- `crates/sxp-kernel/tests/trigonometric.rs`
- `spec/sine_lut.bin`
- `spec/sine_lut.sha256`
- `spec/vectors/trigonometric-order16-q8.sha256`

## Change Log

- 2026-11-01: Kernel trigonometric evaluator, committed sine table, invalid-parameter rejection, pinned fixtures, width bound, and both-platform prediction hash.
- 2026-11-01: Addressed code review findings — 2 patches applied.

## Story completion status

done. Both target CI runs passed after the review patches, including the trigonometric hash test and the existing polynomial tests. Formula framing stays with later stories.
