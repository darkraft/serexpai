---
name: 'MBSC Bitstream and Normative Evaluator: spec seed'
type: spec-seed
status: final
created: '2026-10-30'
updated: '2026-10-30'
governs: [FR-2, FR-3, FR-8, FR-9, FR-10, FR-11]
spine: ARCHITECTURE-SPINE.md
---

# MBSC Bitstream and Normative Evaluator: Spec Seed

This is the starting outline of the normative spec for FR-11 (`spec/bitstream.md`). It turns the spine's rules (AD-3 and AD-5 to AD-11) into concrete layouts. Items tagged **[PROPOSED]** are strawman values the author must confirm or tune. Items tagged **[MEASURE]** are frozen only after measurement on `tuning` inputs. Everything else restates a spine decision. Per AD-15, once any of this is frozen a change bumps the format version.

## 1. Conventions

- Multibyte integers are little-endian.
- Packed fields are fixed-width. Within a byte, fields fill from the least significant bit first. A field that spans bytes continues in the next byte's least significant bits.
- Signed fields are two's complement of the stated width and sign-extended on read.
- Padding bits are zero. A decoder rejects non-zero padding.
- A "section" is a run of packed fields followed by zero padding to a byte boundary.

## 2. File header

| Offset | Field | Width | Notes |
| --- | --- | --- | --- |
| 0 | magic | 8 bits | **[PROPOSED]** `0x8D` (a UTF-8 continuation byte, so a text file never starts with it) |
| 1 | version | bits 0–3 | **[PROPOSED]** 1 for the first stream format; 0.x streams carry no compatibility promise |
| 1 | mode | bits 4–5 | 0 = Blocked, 1 = Whole-file raw, 2 and 3 reserved |
| 1 | reserved | bits 6–7 | must be zero |
| 2.. | N | LEB128, 1–5 bytes | input length in bytes; N < 2³⁵; must be minimal (a multi-byte encoding whose last byte is `0x00` is rejected) |

- Whole-file raw: the header is followed by exactly N bytes. Total size is `2 + len(N) + N`, at most `N + 7`.
- The empty input (N = 0) is the 3-byte header in mode 1, because Whole-file raw wins ties (AD-10). Mode 0 with N = 0 is non-canonical and rejected.
- Blocked, N > 0: a sequence of Blocks follows until their actual lengths sum to N. Nothing may follow the last Block.

## 3. Block

A Block is a section (header and kind-specific fields, zero-padded to a byte boundary), then either the verbatim bytes (Raw) or one rANS stream.

### 3.1 Block header (5 bits)

| Field | Width | Values |
| --- | --- | --- |
| length code | 2 bits | 0 = 256, 1 = 1024, 2 = 2048, 3 = reserved (reject) |
| kind | 3 bits | 0 Raw, 1 Direct, 2 Previous-byte, 3 Constant, 4 Polynomial, 5 Trigonometric, 6 Recurrence, 7 reserved (reject) |

These kind codes are **[PROPOSED]** wire values only. Candidate ids for AD-10 are defined in the spine and do not depend on them.

The actual Block length is `min(nominal, remaining)`. A nominal length is valid only if it is canonical (AD-9): either `nominal ≤ remaining`, or it is the smallest length `≥ remaining`.

### 3.2 Kind-specific fields

| Kind | Fields after the 5-bit header |
| --- | --- |
| Raw | none; after the section, `actual length` verbatim bytes follow and the Block ends |
| Direct, Previous-byte | none; the rANS stream follows the section |
| Constant | `c` (8 bits) |
| Polynomial | `degree` (4 bits, 0–15), `q − 8` (5 bits, stored 0–16, 17–31 reject), `cw − 1` (5 bits, coefficient width 1–32), then `degree + 1` coefficients of `cw` bits, lowest power first |
| Trigonometric | `order − 1` (4 bits, order 1–16), `q − 8` (5 bits), `cw − 1` (5 bits, limited to ≤ 28), `ω` (32 bits, unsigned, phase increment in turns/2³²), then `c0`, and `a_k`, `b_k` for k = 1..order, each `cw` bits |
| Recurrence | `order − 1` (1 bit), `m_mod − 2` (24 bits; a stored value above 2²⁴ − 2 is rejected, so `m_mod` ≤ 2²⁴), `cw − 1` (5 bits, limited to ≤ 24), then `a`, `b` (order 1) or `a`, `b`, `c` (order 2) at `cw` bits each, then initial values `s0` (and `s1` for order 2) at 24 bits each |

The `cw` field widths are **[PROPOSED]** and are the main lever on parameter cost; the width trade-off is **[MEASURE]**.

Any field value outside its range, any `s0`/`s1 ≥ m_mod`, and any coefficient outside the family's limit is a decoder rejection.

## 4. Normative Evaluator

All arithmetic is on signed 64-bit integers. Shifts are arithmetic (floor) shifts. `rem_euclid` is the non-negative remainder. No value is ever wrapped, clamped or saturated; the limits below make overflow impossible, and the decoder rejects a Formula outside them. The final ŷ is `rem_euclid(value, 256)` as `u8`.

Position `n` is the Block-local index starting at 0. `u = n << 5` (so `u < 2¹⁶`, always scaled by 2048 regardless of the actual Block length).

### 4.1 Polynomial

```
acc = c[degree]
for k in degree-1 down to 0:
    acc = ((acc * u) >> 16) + c[k]
value = (acc + (1 << (q - 1))) >> q
```

Limits: each `c[k]` in `[−2³¹ + 1, 2³¹ − 1]` (a stored `−2³¹` is rejected), degree ≤ 15. Bound: `|acc| ≤ (degree + 1)·2³¹ < 2³⁵`, and `|acc·u| < 2⁵¹`.

### 4.2 Trigonometric

```
theta = (n * omega) mod 2^32            # n < 2^11, omega < 2^32: product < 2^43
S = c0 << 15
for k in 1..order:
    p = (k * theta) mod 2^32            # wrapping 32-bit multiply
    S += a[k] * sin_lut(p) + b[k] * sin_lut((p + 2^30) mod 2^32)
value = (S + (1 << (q + 14))) >> (q + 15)
```

`sin_lut(p)`: index `i = p >> 22` (10 bits, 0..1023), `f = (p >> 6) & 0xFFFF`, `v0 = LUT[i]`, `v1 = LUT[i + 1]`, result `v0 + (((v1 − v0) * f + (1 << 15)) >> 16)`. `LUT` has 1025 entries (the last equals the first so interpolation needs no wrap), values are `round(32767 · sin(2π i / 1024))` as signed 16-bit integers, so `|LUT| ≤ 32767 < 2¹⁵`. **[PROPOSED]** table size and scale. The table values are published with a SHA-256 in `spec/sine_lut.*` and are normative; the generator is not.

Limits: `|c0|, |a_k|, |b_k| < 2²⁸` (`cw ≤ 28`), order ≤ 16. Bound: each term `< 2²⁸ · 2¹⁵ = 2⁴³`, at most 33 terms, so `|S| < 2⁴⁹` plus `|c0 << 15| < 2⁴³`.

### 4.3 Recurrence

```
order 1: x[0] = s0;           x[n+1] = rem_euclid(a * x[n] + b, m_mod)
order 2: x[0] = s0; x[1] = s1; x[n+2] = rem_euclid(a * x[n+1] + b * x[n] + c, m_mod)
value = x[n]
```

Limits: `|a|, |b|, |c| < 2²⁴`, `2 ≤ m_mod ≤ 2²⁴`, `0 ≤ s0, s1 < m_mod`. Bound: `|a·x| < 2⁴⁸`, `|a·x + b·x'| < 2⁴⁹`, plus `c`. The recurrence is evaluated by iterating from `n = 0` within the Block, so it is positionwise in the sense of AD-4 (it depends on `n` and its own parameters only). `value = x[n]` is reduced by `rem_euclid(256)` like every other family.

### 4.4 Previous-byte and Constant

- Constant: ŷ = `c` for every position.
- Previous-byte: ŷ₀ = 0, and ŷₙ = the decoded byte `yₙ₋₁` within the same Block.

### 4.5 Residual

`δ = ((y − ŷ + 128) mod 256) − 128`, `y = (ŷ + δ) mod 256`. The coded symbol is `δ mod 256` (as 0..255); under Direct it is `y`.

## 5. Probability model (AD-7)

Reset at every Block start. Each symbol `s` is coded as 8 binary decisions, most significant bit first. The tree has 255 nodes indexed 1..255; decision 0 uses node 1, and after a decision with value `b` at node `m`, the next node is `2m + b`.

State per node: `p` (probability that the bit is 0, 12 bits) and `n` (saturating count).

| Constant | Value | Status |
| --- | --- | --- |
| initial `p` | 2048 | **[MEASURE]** |
| count cap `CAP` | 24 | **[MEASURE]** |
| update rate | `RECIP[n] = floor(65536 / (n + 2))`, constant at `RECIP[CAP]` once `n = CAP` | **[MEASURE]** |
| clamp | `p` kept in `[16, 4080]` | **[MEASURE]** |

Update after a decision with bit `b`:

```
target = 4096 if b == 0 else 0
p = p + (((target - p) * RECIP[n]) >> 16)       # arithmetic shift
p = clamp(p, 16, 4080)
n = min(n + 1, CAP)
```

The model updates only after the decision is coded, so the decoder always uses the same `p` the encoder used.

## 6. rANS coder (AD-7)

- State: unsigned 32-bit `x`, normalized interval `[L, 256·L)` with `L = 2²³`. Scale bits 12 (total 4096).
- Each decision has `freq` and `start`: bit 0 gets `freq = p`, `start = 0`; bit 1 gets `freq = 4096 − p`, `start = p`.
- Encoding runs over the buffered decisions of the Block in reverse order, starting from `x = L`:

```
x_max = ((L >> 12) << 8) * freq
while x >= x_max: emit_byte(x & 0xFF); x >>= 8
x = ((x / freq) << 12) + (x % freq) + start
```

- Termination and layout: the encoder collects renormalization bytes in emission order. After the first-in-order decision has been encoded, the stream is laid out as `LE32(x_final)` followed by the collected renormalization bytes in reverse emission order, so the decoder reads it strictly forward. Example vectors: one decision of bit 0 with `p = 2048` gives `00 00 00 01`; bit 1 with `p = 2048` gives `00 08 00 01`.
- Decoding reads `x` from the first 4 bytes (little-endian) and rejects `x` outside `[L, 2³¹)`, then for each decision:

```
slot = x & 4095
bit = 0 if slot < p else 1
(freq, start) as above
x = freq * (x >> 12) + slot - start
while x < L: x = (x << 8) | next_byte()
```

- After the last decision, including its renormalization step, `x` must equal `L`. Otherwise the stream is rejected. The number of bytes consumed is the Block's payload length (AD-8: payload length is derived, not stored). Reading past the end of input is a rejection.
- Per-Block rANS overhead is exactly the 4-byte flush; the Block header and parameters are on top of that.

## 7. Decoder accept and reject set (AD-11, AD-12)

The decoder rejects: bad magic; unknown version; a set reserved bit; mode 2 or 3; mode 0 with N = 0; a non-minimal or out-of-range LEB128; reserved length code or kind; a non-canonical nominal length; non-zero padding; any parameter outside its limit; `s0`/`s1 ≥ m_mod`; an rANS stream running past the end of input or ending with `x ≠ L`; a stream that ends before N bytes are decoded; and trailing bytes. All of these are covered by rejection vectors, and the Python reference decoder must reject each one too.

## 8. Test vectors (`spec/vectors/`)

| Set | Content |
| --- | --- |
| Evaluator | `(family, parameters, Block length) → ŷ sequence`, including boundary values for every limit (max coefficients, `u` at its maximum, degree 15, order 16, `q` = 8 and 24, `m_mod` at its limits) and a negative halfway case |
| Residual | all 65,536 `(y, ŷ)` pairs, as a hash of the result |
| Model | decision sequence → `p` sequence for edge counts (fresh node, `n = CAP`, clamp hit) |
| rANS | decision list → bytes, including the two 1-decision vectors of §6 and the 2048-symbol case |
| Blocks | one valid Block per kind and length code, including Raw and the partial final Block |
| Files | empty, 1 byte, 255, 256, 257, exactly one 2048 Block, a partial final Block, Whole-file raw |
| Rejection | one malformed file per rule in §7 |

## 9. Open items before freeze

1. Confirm or change the magic byte and the version value.
2. Confirm the Block header packing (a 5-bit header leaves 3 padding bits per Block; Raw Blocks pay one byte of header).
3. Tune `CAP`, `RECIP`, the clamp and the initial `p` on `tuning` inputs only.
4. Decide the LUT size (1024 intervals) and the Q15 scale after checking interpolation error against the benchmark's needs.
5. Measure the parameter cost of the `cw` scheme against fixed-width coefficients and against the minimum viable Q.
6. Decide whether the 4-byte rANS flush is shortened (a version bump, per AD-15) once the per-Block overhead is measured.
