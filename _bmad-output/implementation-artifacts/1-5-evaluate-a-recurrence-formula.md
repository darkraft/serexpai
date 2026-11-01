---
baseline_commit: 277f5a166a8f92e4513b5d518f778e6fdb2d84ca
---

# Story 1.5: Evaluate a recurrence formula

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want recurrence predictions to keep order, initial state, and modulus distinct,
so that a Formula's ŷ stream is the same byte sequence on every host, and an order-1 Formula cannot silently consume `c` or `s1`.

## Acceptance Criteria

Stable IDs from the epic breakdown. A test or the committed hash must be traceable to each ID. This story is not done until x86_64 and aarch64 evidence both exist. Native evidence was enough to start this story (elicitation amendment 1, 2026-11-01). It does not close AC04.

1. **S1.5-AC01.** G0 already defines order 1 or 2, `m_mod` in `2..=2^24`, `0 ≤ s_i < m_mod`, Block-local `n` in `0..=2047`, and `|a|, |b|, |c| < 2^24`. That text is `bitstream-spec-seed.md` §3.2 and §4.3. Do not edit the seed, and do not add a second normative document. D-01 (approved 2026-11-01) caps stored `cw` at 25. This story does not serialize `cw`. D-07 stays open. Invalid inputs return `Error::InvalidParameter`. The API takes `u8` order, `u32` modulus, `i32` coefficients, `u32` initial states, and `u16` position. Rustdoc states where a narrower type cannot represent a rejected neighbour (a negative order, a negative modulus, a negative state, a negative `n`, `m_mod > u32::MAX`) and where a representable neighbour is rejected (`m_mod` of `0`, `1`, or `2^24 + 1`; `|coefficient| ≥ 2^24`; `s_i ≥ m_mod`; order `0` or `3`). Order 1 rejects a third coefficient and a second state. Those fields are not stored and not read.
2. **S1.5-AC02.** Order 1 sets `x[0] = s0` and `x[n+1] = rem_euclid(a·x[n] + b, m_mod)`. Order 2 sets `x[0] = s0`, `x[1] = s1`, and `x[n+2] = rem_euclid(a·x[n+1] + b·x[n] + c, m_mod)`. `value = x[n]` and `ŷ = rem_euclid(value, 256)`. Products and the linear combination are signed `i64`. `rem_euclid` is the only remainder. No floating point. No `/` and no `%`. No `u = n << 5`.
3. **S1.5-AC03.** The fixtures below are the expected values. They cover both orders, both modulus limits, initial states `0` and `m_mod − 1`, negative coefficients, both legal coefficient endpoints, and the rejected neighbours. Unsupported order, an out-of-range state, an invalid modulus, or an invalid coefficient returns `Error::InvalidParameter` and is not clamped or wrapped. A negative state does not fit in `u32`; the tested rejection is the representable neighbour `s_i = m_mod`.
4. **S1.5-AC04.** `predict` has no Block-length argument and no cursor. Calling it, or calling it on a different Formula, does not change a later call. `n = 0` is the initial state, not one application of the recurrence. Both platforms match the committed prediction hash. The boundary test states and checks the intermediate-width bound. Polynomial, trigonometric, residual, and whole-file raw tests still pass.

## Tasks / Subtasks

- [x] **Recurrence type** (AC: 1, 2)
  - [x] Add `crates/sxp-kernel/src/recurrence.rs`. `pub struct Recurrence` with private fields `order: u8`, `m_mod: u32`, `coefficients: [i32; 3]`, and `state: [u32; 2]`. Derive `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`. Layout, lowest index first: index `0` is `a`, index `1` is `b`, index `2` is `c`. State index `0` is `s0`, index `1` is `s1`. Unused slots stay `0`.
  - [x] `pub fn new(order: u8, m_mod: u32, coefficients: &[i32], state: &[u32]) -> Result<Self, Error>`. Reject, with `Error::InvalidParameter` and no other variant, when `order` is outside `1..=2`, `m_mod` is outside `2..=1 << 24`, `coefficients.len() != usize::from(order) + 1`, or `state.len() != usize::from(order)`. Reject a coefficient when `coefficient <= -(1 << 24)` or `coefficient >= (1 << 24)`. Reject a state when `state >= m_mod`. Copy with `get`. Do not accept a longer slice and ignore the tail. Order 1 with three coefficients is an error even when the third is `0`. Order 1 with two states is an error even when the second is `0`.
  - [x] The order-1 arm of `value` reads coefficient indices `0` and `1` and state index `0` only. It does not read `c` or `s1`. Do not add `Order1` / `Order2` types. Do not add a `q` or `cw` field.
  - [x] Do not add an error variant. `InvalidParameter` already exists. Do not edit `tests/support/mod.rs`.
  - [x] Rustdoc on `new`: a negative order does not fit in `u8`; order `0` fits and is rejected. A negative modulus does not fit in `u32`; `0` and `1` fit and are rejected; `2` and `2^24` are legal; `2^24 + 1` and `u32::MAX` fit and are rejected. A negative state does not fit in `u32`; `s_i = m_mod − 1` is legal; `s_i = m_mod` fits whenever `m_mod ≤ 2^24` and is rejected. `±(2^24 − 1)` are legal. `±2^24` fit in `i32` and are rejected. `i32::MIN` and `i32::MAX` are rejected because they are outside `|c| < 2^24`, not because of a special `i32` rule. Stored `cw ≤ 25` is the story 1.10 wire cap; this constructor does not take `cw`. Rustdoc on `value` and `predict`: a negative `n` does not fit in `u16`. There is no Block-length parameter. Each call restarts at `s0` and `s1`.
  - [x] `pub fn value(&self, n: u16) -> Result<i64, Error>` and `pub fn predict(&self, n: u16) -> Result<u8, Error>`. `n > 2047` is `Error::InvalidParameter`. `predict` is `value(n)?.rem_euclid(256) as u8`.
  - [x] Order 1, in `value` only. `x = i64::from(s0)`. Repeat `n` times: `x = (i64::from(a) * x + i64::from(b)).rem_euclid(i64::from(m_mod))`. Order 2: `n == 0` returns `i64::from(s0)`. Otherwise `prev = i64::from(s0)`, `curr = i64::from(s1)`, and repeat `n − 1` times: `next = (i64::from(a) * curr + i64::from(b) * prev + i64::from(c)).rem_euclid(i64::from(m_mod))`, then `prev = curr`, `curr = next`, and the result is `curr`. Promote before multiplying. `a * x` with both factors still `i32` overflows at the legal endpoints.
  - [x] Use plain `i64` `*` and `rem_euclid`. Do not use `wrapping_mul`, `overflowing_mul`, `checked_mul`, `/`, or `%`. Do not call `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, `Polynomial`, or `Trigonometric`.
  - [x] No `unsafe`. No allocator. No new dependency. No float. No closed form and no cached prefix.
  - [x] In `lib.rs` add only `mod recurrence;` and `pub use recurrence::Recurrence;`. The module-doc sentence may name the recurrence evaluator. The G0 hash stays `f4bea1e5b729db1df957563fd3e8e5bec70c0ab0a382f0f0afc7f9e82196e12e`. Do not edit the lint block.
- [x] **Typed rejection** (AC: 1, 3)
  - [x] Add `crates/sxp-kernel/tests/recurrence.rs`. Do not `mod support`.
  - [x] Reject these neighbours and accept nothing clamped into range: `order = 0` and `order = 3`; `m_mod = 0`, `m_mod = 1`, `m_mod = (1 << 24) + 1`, and `m_mod = u32::MAX`; `n = 2048`; coefficient `1 << 24` and `-(1 << 24)` at `a` and at `b` of an order-1 formula, and at `a`, `b`, and `c` of an order-2 formula; `i32::MIN` and `i32::MAX` at `a`; state `s0 = m_mod` for both modulus limits; order 2 with `s1 = m_mod` while `s0 = 0`. Include order 1 with 1 coefficient and with 3, order 2 with 2 coefficients and with 4, order 1 with 0 states and with 2, and order 2 with 1 state and with 3. A slice of three coefficients whose third is `0` is still an error for order 1. A rejected `new` or `value` does not produce a `ŷ`.
  - [x] Accept `±(2^24 − 1)` at `a`, `b`, and `c`. Accept `m_mod = 2` and `m_mod = 1 << 24`. Accept `s_i = 0` and `s_i = m_mod − 1`.
- [x] **Pinned fixtures** (AC: 2, 3)
  - [x] Assert `value` and `predict` against the `value` and `ŷ` columns. Where `linear` is present, the test's `i128` recurrence must reproduce that pre-remainder combination, and `Recurrence::value` must equal `linear.rem_euclid(m_mod)` as `i64`. Do not compute the literals from `Recurrence`. Coefficients use ASCII `-`. `M = 2^24 = 16777216`. `A = 2^24 − 1 = 16777215`.
  - [x] Order 1. `linear` is the combination that produced `x[n]`. `n = 0` has no step.

    | case | m_mod | a | b | s0 | n | linear | value | ŷ |
    |---|---:|---:|---:|---:|---:|---:|---:|---:|
    | zero | 2 | 0 | 0 | 0 | 0 |  | 0 | 0 |
    | zero at end | 2 | 0 | 0 | 0 | 2047 | 0 | 0 | 0 |
    | toggle | 2 | 1 | 1 | 0 | 0 |  | 0 | 0 |
    | toggle step | 2 | 1 | 1 | 0 | 1 | 1 | 1 | 1 |
    | toggle end | 2 | 1 | 1 | 0 | 2047 | 1 | 1 | 1 |
    | state m−1 | 2 | 1 | 0 | 1 | 0 |  | 1 | 1 |
    | state m−1 held | 2 | 1 | 0 | 1 | 2047 | 1 | 1 | 1 |
    | negative a | 5 | -1 | 0 | 1 | 1 | -1 | 4 | 4 |
    | negative a end | 5 | -1 | 0 | 1 | 2047 | -1 | 4 | 4 |
    | roles | 7 | 2 | 3 | 2 | 1 | 7 | 0 | 0 |
    | not `n << 5` | 5 | 2 | 1 | 0 | 1 | 1 | 1 | 1 |
    | counter | 16777216 | 1 | 1 | 0 | 256 | 256 | 256 | 0 |
    | counter end | 16777216 | 1 | 1 | 0 | 2047 | 2047 | 2047 | 255 |
    | s0 = m−1 | 16777216 | 0 | 0 | 16777215 | 0 |  | 16777215 | 255 |
    | s0 stepped | 16777216 | 0 | 0 | 16777215 | 1 | 0 | 0 | 0 |
    | a, b endpoints | 16777216 | 16777215 | -16777215 | 16777215 | 1 | 281474926379010 | 2 | 2 |
    | negative a endpoint | 16777216 | -16777215 | 16777215 | 1 | 1 | 0 | 0 | 0 |

  - [x] `negative a` is `value = 4` because `rem_euclid(-1, 5) = 4`. Rust `%` yields `-1`, and `rem_euclid(-1, 256)` yields `255`. Assert `value = 4`, not only `ŷ`.
  - [x] `roles` is `value = 0` because `rem_euclid(2·2 + 3, 7) = 0`. Using `b` as the multiplier and `a` as the addend is `rem_euclid(8, 7) = 1`. Assert `value = 0`.
  - [x] `not n << 5` is `value = 1`. The same coefficients iterated `(1 << 5) = 32` times land on `x[32] = 0`, because the sequence `0,1,3,2` has period 4. Assert `value(1) = 1`. `predict` has no length argument.
  - [x] `s0 = m−1` at `n = 0` is the initial state `16777215`, `ŷ = 255`. One step with `a = b = 0` is `0`. An implementation that always applies the recurrence once is wrong at `n = 0`.
  - [x] `a, b endpoints` is `value = 2` because `(2^24 − 1)·(2^24 − 2) = 281474926379010` and that residue modulo `2^24` is `2`. Returning the unreduced combination is not `value`.
  - [x] `counter` at `n = 256` is `value = 256`, `ŷ = 0`. `counter end` is `value = 2047`, `ŷ = 255`.
  - [x] Order 2. `linear` is present only for `n ≥ 2`.

    | case | m_mod | a | b | c | s0 | s1 | n | linear | value | ŷ |
    |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
    | fib | 2 | 1 | 1 | 0 | 0 | 1 | 0 |  | 0 | 0 |
    | fib s1 | 2 | 1 | 1 | 0 | 0 | 1 | 1 |  | 1 | 1 |
    | fib step | 2 | 1 | 1 | 0 | 0 | 1 | 2 | 1 | 1 | 1 |
    | fib end | 2 | 1 | 1 | 0 | 0 | 1 | 2047 | 1 | 1 | 1 |
    | states m−1 | 2 | 1 | 0 | 1 | 1 | 1 | 0 |  | 1 | 1 |
    | states stepped | 2 | 1 | 0 | 1 | 1 | 1 | 2 | 2 | 0 | 0 |
    | swap | 7 | 2 | 3 | 0 | 1 | 2 | 2 | 7 | 0 | 0 |
    | c | 7 | 2 | 3 | 5 | 1 | 2 | 2 | 12 | 5 | 5 |
    | negative b | 5 | 0 | -1 | 0 | 1 | 0 | 2 | -1 | 4 | 4 |
    | negative c | 5 | 0 | 0 | -3 | 0 | 0 | 2 | -3 | 2 | 2 |
    | c endpoint | 16777216 | 0 | 0 | 16777215 | 0 | 0 | 2 | 16777215 | 16777215 | 255 |
    | c negative endpoint | 16777216 | 0 | 0 | -16777215 | 0 | 0 | 2 | -16777215 | 1 | 1 |
    | hash n = 0 | 16777216 | 16777215 | -16777215 | 1 | 0 | 16777215 | 0 |  | 0 | 0 |
    | hash n = 1 | 16777216 | 16777215 | -16777215 | 1 | 0 | 16777215 | 1 |  | 16777215 | 255 |
    | hash n = 2 | 16777216 | 16777215 | -16777215 | 1 | 0 | 16777215 | 2 | 281474943156226 | 2 | 2 |
    | hash n = 3 | 16777216 | 16777215 | -16777215 | 1 | 0 | 16777215 | 3 | -281474909601794 | 16777214 | 254 |

  - [x] `swap` is `value = 0` because `rem_euclid(2·s1 + 3·s0, 7) = 0`. Swapping `a` with `b` is `rem_euclid(8, 7) = 1`. Assert `value = 0`.
  - [x] `c` is the same `a`, `b`, and state with `c = 5`, so `value = 5`. Dropping `c` yields the `swap` row.
  - [x] `negative b` is `value = 4` because `rem_euclid(-s0, 5) = 4`. `negative c` is `value = 2` because `rem_euclid(-3, 5) = 2`. Rust `%` keeps the negative combination. Assert `value`.
  - [x] `c negative endpoint` is `value = 1` because `rem_euclid(-(2^24 − 1), 2^24) = 1`.
  - [x] `hash n = 2` is `value = 2` because `a·s1 + c = (2^24 − 1)^2 + 1 = 281474943156226`. `hash n = 3` carries a negative combination. The next four states are `5, 16777210, 12, 16777199`.
- [x] **Width bound** (AC: 4)
  - [x] In the test, walk order 2 at `m_mod = 2^24` through `n = 2047`, for three coefficient vectors: all `16777215`, all `−16777215`, and `a = 16777215`, `b = −16777215`, `c = 16777215`. States are `s0 = s1 = 16777215` for the first two, and `s0 = 0`, `s1 = 16777215` for the mixed vector. Use an `i128` recurrence. On every step assert `|a·x| ≤ 281474943156225` and `|a·x| < 2^48`, and assert `|a·x + b·x' + c| ≤ 562949903089665` and `|a·x + b·x' + c| < 2^49`. Assert `Recurrence::value` equals the oracle.
  - [x] Write the bound next to those asserts: seed §4.3 says `|a·x| < 2^48` and `|a·x + b·x'| < 2^49`, plus `c`. The strict bound holds because `|a|, |b|, |c| ≤ 2^24 − 1` and `x ≤ 2^24 − 1`, so `|a·x| ≤ (2^24 − 1)^2 = 281474943156225 < 2^48 = 281474976710656`, `|a·x + b·x'| ≤ 562949886312450 < 2^49`, and `|a·x + b·x' + c| ≤ 562949903089665 < 2^49 = 562949953421312`. Do not assert the spine's coarser "below 2^62". Do not edit the seed.
  - [x] The implementation stays on `i64`. A wrapping multiply fails the oracle.
- [x] **Prediction hash** (AC: 4)
  - [x] Commit `spec/vectors/recurrence-order2-m24.sha256` containing `cd04b2a24404c1687a35dd3496a599bf54b706d3c075047f1fd3872aa7d0a0e4` and one trailing newline. The test loads it with `include_str!(...).trim()`. Do not add it to `spec/vectors/index.toml`.
  - [x] The digest is SHA-256 of 2048 bytes: order 2, `m_mod = 2^24`, `a = 16777215`, `b = −16777215`, `c = 1`, `s0 = 0`, `s1 = 16777215`, `n` from 0 through 2047, one `ŷ` per `n`. First 16 bytes are `00 ff 02 fe 05 fa 0c ef 1e d2 4d 86 c8 bf 0a b6`. Byte 0 is `0`. Byte 1 is `255`. Byte 2 is `2`. Byte 255 is `150`. Byte 2047 is `111`. The independent oracle is the script in Dev Notes. If the Rust test disagrees with the file, rerun that script. Its stdout is the authority. Do not regenerate the artifact from `Recurrence`.
  - [x] Hash with `sha2 = "=0.11.1"`. Encode hex with `write!(s, "{b:02x}")`. Do not `format!("{:x}", digest)`.
- [x] **Property tests** (AC: 2, 3)
  - [x] Use the existing `proptest = "=1.12.0"` with `default-features = false`. Config: `cases: 256`, `rng_seed: RngSeed::Fixed(0x5111_0005)`, `failure_persistence: Some(Box::new(FileFailurePersistence::Off))`, remaining fields from `Config::default()`. Do not change seeds `0x5111_0001`, `0x5111_0002`, `0x5111_0003`, or `0x5111_0004`. Do not use regex strategies.
  - [x] Draw `order` in `1..=2`, `m_mod` in `2..=2^24`, `n` in `0..=2047`, `order + 1` coefficients in `-(2^24 − 1)..=2^24 − 1`, and `order` states in `0..m_mod`. Assert `value` matches the `i128` oracle, `predict` equals `value.rem_euclid(256)` as `u8`, and every product and linear combination stay inside the bounds above.
  - [x] One strategy also draws one illegal neighbour (`order` `0` or `3..=255`, or `m_mod` in `{0, 1}` or `2^24 + 1..=u32::MAX`, or one coefficient forced to `1 << 24`, `-(1 << 24)`, `i32::MAX`, or `i32::MIN`, or one state forced to `m_mod` when `m_mod < u32::MAX`, or `n` in `2048..=65535`) and asserts `Error::InvalidParameter`. The forced coefficient is not only index 0. The forced state is not only `s0` when `order = 2`.
- [x] **Block restart** (AC: 4)
  - [x] Build the order-1 `negative a` formula and the order-2 `c` formula. `predict(2047)` on the first is `4`. `predict(2)` on the second is `5`. `predict(0)` on the first is then `1`, and `predict(2047)` is still `4`. Two calls on one value do not share a cursor.
- [x] **Both platforms** (AC: 4)
  - [x] Run `./ci.sh x86_64-unknown-linux-gnu` and `./ci.sh aarch64-unknown-linux-gnu`. Record both commands, `git rev-parse HEAD`, and that the recurrence hash test passed. Polynomial and trigonometric tests must still pass; do not edit them.
  - [x] `aarch64-unknown-linux-gnu` links with `aarch64-linux-gnu-gcc` and runs under `qemu-aarch64` (`.cargo/config.toml`). If those are not on `PATH`, put `/tmp/serexpai-cross/bin` ahead of `PATH`, as story 1.4 did. Do not edit `.cargo/config.toml`.
  - [x] Leave this subtask unchecked if either target did not run. Do not set the story Status to `done`. `dev-story` sets `review`. Code review sets `done` only when both runs are recorded.
  - [x] Do not edit `bitstream-spec-seed.md`, `CHANGELOG.md`, the workspace version, `polynomial.rs`, `trigonometric.rs`, `residual.rs`, `raw.rs`, or story 1.1–1.4 tests. Do not commit unless the user asks.

### Review Findings

- [x] [Review][Patch] `value` docs say the result is before `rem_euclid` [crates/sxp-kernel/src/recurrence.rs:80]
- [x] [Review][Patch] `new` docs omit order 3, `m_mod > u32::MAX`, and `s_i >= m_mod` [crates/sxp-kernel/src/recurrence.rs:26]
- [x] [Review][Patch] `new` docs omit the "not a special i32 rule" clause [crates/sxp-kernel/src/recurrence.rs:32]
- [x] [Review][Patch] state rejection tests only use `s == m_mod` [crates/sxp-kernel/tests/recurrence.rs:383]
- [x] [Review][Patch] illegal `n` property discards `predict`'s result [crates/sxp-kernel/tests/recurrence.rs:578]
- [x] [Review][Patch] order 0 is not rejected when the slice lengths fit [crates/sxp-kernel/tests/recurrence.rs:305]

## Dev Notes

### What this story builds

One validated recurrence evaluator in `sxp-kernel`. It returns `x[n]` and the prediction byte for a Block-local position. It does not residual-code that byte, frame a Block, or serialize a Formula.

Out of scope: the range coder (story 1.6); the probability model (1.7); `encode_block` and any Block header (1.8); Codebook search and the on-wire layout `order − 1`, `m_mod − 2`, `cw − 1`, coefficients, `s0`, `s1` (1.10). D-07 stays open: this story accepts every coefficient with `|c| < 2^24`, not a measured `cw` inside the cap. D-09 stays open: no production Codebook recipe. G0 for this untagged arithmetic is the seed. Do not write `spec/bitstream.md`.

### Arithmetic

Governing text is seed §4.3, which elaborates AD-4, AD-5, and AD-6. Position is the iteration count `n`. It is not `u = n << 5`. A short Block does not change `x[n]`. AD-5's polynomial sentence about `u = n << 5` does not apply to this family. The spine's "below 2^62" is the coarse envelope; the numbers to test are the seed's `2^48` and `2^49`.

```
order 1:
    x = s0
    repeat n times:
        x = rem_euclid(a * x + b, m_mod)
order 2:
    x[0] = s0
    x[1] = s1
    x[n+2] = rem_euclid(a * x[n+1] + b * x[n] + c, m_mod)
value = x[n]
ŷ = rem_euclid(value, 256)
```

`n = 0` returns `s0` for both orders. Order 2 at `n = 1` returns `s1` and does not apply the combination. There is no Q and no rounding bias. `value` is already in `0..m_mod`.

Rust `%` truncates toward zero. `rem_euclid` is the non-negative remainder. `(-1) % 5` is `-1`; `(-1).rem_euclid(5)` is `4`. The final remainder divisor is the positive constant `256`. The modulus divisor is `m_mod ≥ 2`, so `rem_euclid` does not hit the `rhs == 0` panic or the `i64::MIN` / `-1` panic.

`|a·x|` fits in `i64` under the limit table. Do not widen the kernel to `i128`.

### Prediction-hash oracle

Run from the repo root after the story is implemented, only if the Rust digest disagrees with the pin. Stdout is the digest.

```python
import hashlib
M = 2**24
A = M - 1
a, b, c = A, -A, 1
s0, s1 = 0, A

def step(prev, curr):
    return (a * curr + b * prev + c) % M

xs = [s0, s1]
for _ in range(2046):
    xs.append(step(xs[-2], xs[-1]))
out = bytes(x % 256 for x in xs)
assert out[:16].hex() == "00ff02fe05fa0cef1ed24d86c8bf0ab6"
assert out[0] == 0 and out[1] == 255 and out[2] == 2
assert out[255] == 150 and out[2047] == 111
print(hashlib.sha256(out).hexdigest())
```

### Kernel shape to preserve

`lib.rs` forbids unsafe code and denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, and `unreachable` on `src/`. Tests may use `expect` and `assert`.

Public surface that must keep its signatures: `read_whole_file_raw`, `write_whole_file_raw`, `encode_whole_file_raw_header`, `decode_header`, `Header`, `HeaderBytes`, `encode_leb128`, `decode_leb128`, `Leb128`, `LENGTH_LIMIT`, `Crc32`, `crc32_iso_hdlc`, `signed_residual`, `residual_symbol`, `reconstruct`, `direct_symbol`, `Polynomial`, `Trigonometric`, `check_sine_lut`, and every existing `Error` variant plus `Error::name`.

`sxp-kernel` has an empty `[dependencies]` table. `ci.sh` fails unless `cargo tree -e normal` and `-e build` for `sxp-kernel` are each one line, and unless `crates/sxp-kernel/build.rs` is absent. `sha2` and `proptest` are already dev-dependencies. Do not add another.

`Error::Display` is the variant name only. `tests/support/mod.rs` asserts `index.toml` has 24 records. A new `[[vector]]` row fails that test.

### Project Structure Notes

- Evaluator: `crates/sxp-kernel/src/recurrence.rs`.
- Export: `crates/sxp-kernel/src/lib.rs`, one `mod` and one `pub use`.
- Tests: `crates/sxp-kernel/tests/recurrence.rs`.
- Prediction hash: `spec/vectors/recurrence-order2-m24.sha256`.
- Do not put recurrence math in `sxp-encoder`. Do not add a `Formula` enum that also covers polynomial or trigonometric. Do not fold this module into `polynomial.rs`.

### Previous story intelligence

Story 1.4 put the trigonometric evaluator in its own kernel module, exported it with `pub use`, and tested it from `tests/` without `mod support`. The prediction artifact is a one-line SHA-256 file loaded by `include_str`, not a `[[vector]]` row. `Error::InvalidParameter` is the rejection variant. `as u8` after `rem_euclid` is the existing cast. Integration tests own `proptest` and `sha2`. The next free proptest seed is `0x5111_0005`.

Story 1.4's review patches were doc precision and a missing coefficient slot: `±2^28` had to be rejected at both `a_8` and `b_16`, and `value` had to be documented as the rounded result. Do the same here for `±2^24` at `a`, `b`, and `c`. Prefer an exact `value` literal over an approximate `ŷ` check. A fixture that can pass with `a` and `b` swapped, or with `c` dropped, will be rejected.

The polynomial's `u = n << 5` does not apply. The trigonometric phase `n * omega` does not apply. `n = 0` is `s0`, not a collapsed constant term. Order 1 must not read `c` or `s1`.

### Git intelligence

`HEAD` is `277f5a166a8f92e4513b5d518f778e6fdb2d84ca`, `feat: evaluate a trigonometric formula (v0.0.10)`. Stories 1.1–1.4 are in that history. Do not revert them. There is no recurrence module to extend. Recent titles:

- `277f5a1` feat: evaluate a trigonometric formula (v0.0.10)
- `09f0f51` feat: evaluate a polynomial formula (v0.0.9)
- `fb5a3fd` feat: round-trip whole-file raw and reconstruct predictions (v0.0.8)
- `5a58bbb` Add the pinned workspace and whole-file raw fixtures.
- `f1b1ed1` docs: record implementation-readiness decisions (v0.0.7)

### Latest technical information

Stay on the pinned toolchain `1.99.1` (`rust-toolchain.toml`). Do not `rustup update`. `i64::rem_euclid` returns a non-negative remainder and panics if `rhs == 0` or if the division overflows (`i64::MIN` with a divisor of `-1`). This domain never hits either panic: `m_mod ≥ 2` and the final divisor is `256`. Rust `%` truncates toward zero and is the wrong operation. Do not add `num-traits` or a big-integer crate. The test oracle's `i128` replaces an external numeric library. `proptest 1.12.0` and `sha2 0.11.1` are already pinned; do not run `cargo update`.

### Project context reference

No `project-context.md` exists in this tree. The kernel lint block in `lib.rs` and AD-4 / AD-5 / AD-6 in the architecture spine are the coding standard.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md` — Story 1.5, FR-2, FR-3, D-01, D-07, elicitation amendments 1, 3, and 7]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md` — §3.2 Recurrence row, §4.3]
- [Source: `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` — AD-4, AD-5, AD-6, crate table]
- [Source: `_bmad-output/implementation-artifacts/1-4-evaluate-a-trigonometric-formula.md` — module export, hash artifact, proptest profile, both-platform gate, review patches]
- [Source: Rust reference, `i64::rem_euclid` — non-negative remainder; `%` truncates toward zero]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created
- `Recurrence` in `sxp-kernel` validates order `1..=2`, `m_mod` in `2..=2^24`, coefficients in `(-(2^24 - 1))..=(2^24 - 1)`, and one initial state per order, each strictly below `m_mod`. Order 1 rejects a third coefficient and a second state. `value` iterates from `s0`/`s1` with signed `i64` and `rem_euclid`. `predict` is `rem_euclid(value, 256)`. No Block-length argument and no cursor. `Error::InvalidParameter` is reused; no new error variant.
- Pinned fixtures, the `i128` width bound (`|a·x| < 2^48`, `|a·x + b·x' + c| < 2^49`), and prediction SHA-256 `cd04b2a24404c1687a35dd3496a599bf54b706d3c075047f1fd3872aa7d0a0e4` pass. `rem_euclid(-1, 5)` is `4`. `n = 1` on the period-4 sequence is `1`, which `(n << 5)` steps would turn into `0`.
- `./ci.sh x86_64-unknown-linux-gnu` passed on 2026-11-01, including `prediction_hash_matches_the_committed_digest` and the polynomial and trigonometric tests.
- `./ci.sh aarch64-unknown-linux-gnu` passed on 2026-11-01 under `qemu-aarch64`, including the same hash test. Linker and runner came from `/tmp/serexpai-cross/bin` ahead of `PATH`. `HEAD` is still `277f5a166a8f92e4513b5d518f778e6fdb2d84ca`. This story is uncommitted.
- Code review patches applied: `value` is documented as `x[n]` after the modulus remainder, `new` documents order 3, `m_mod > u32::MAX`, `s_i >= m_mod`, and the `i32` endpoint rationale, and the tests reject a fitting order-0 slice, states above `m_mod`, and `predict` on illegal `n`. Both CI targets were rerun after those patches and passed.

### File List

- `crates/sxp-kernel/src/recurrence.rs`
- `crates/sxp-kernel/src/lib.rs`
- `crates/sxp-kernel/tests/recurrence.rs`
- `spec/vectors/recurrence-order2-m24.sha256`

### Change Log

- 2026-11-01: Kernel recurrence evaluator, invalid-parameter rejection, pinned fixtures, width bound, and both-platform prediction hash.
- 2026-11-01: Addressed code review findings — 6 patches applied.

## Story completion status

done. Both target CI runs passed after the review patches, including the recurrence hash test and the existing polynomial and trigonometric tests. Formula framing stays with later stories.
