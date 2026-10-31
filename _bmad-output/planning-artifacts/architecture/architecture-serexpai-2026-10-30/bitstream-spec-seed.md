---
name: 'MBSC Bitstream and Normative Evaluator: spec seed'
type: spec-seed
status: final (revised 2026-10-31 from open-questions research)
created: '2026-10-30'
updated: '2026-10-31'
governs: [FR-2, FR-3, FR-8, FR-9, FR-10, FR-11]
spine: ARCHITECTURE-SPINE.md
---

# MBSC Bitstream and Normative Evaluator: Spec Seed

The product that implements this format is SerExpAI. MBSC (Multi-Basis Series Compression System) is the idea that format represents. This is the starting outline of the normative spec for FR-11 (`spec/bitstream.md`). It turns the spine's rules (AD-3 and AD-5 to AD-11) into concrete layouts. Items tagged **[PROPOSED]** are strawman values the author must confirm or tune. Items tagged **[MEASURE]** are frozen only after measurement on `tuning` inputs. Everything else restates a spine decision. Per AD-15, once any of this is frozen a change bumps the format version.

## 1. Conventions

- Multibyte integers are little-endian.
- Packed fields are fixed-width. Within a byte, fields fill from the least significant bit first. A field that spans bytes continues in the next byte's least significant bits.
- Signed fields are two's complement of the stated width and sign-extended on read.
- Padding bits are zero. A decoder rejects non-zero padding.
- A "section" is a run of packed fields followed by zero padding to a byte boundary.

## 2. File header

| Offset | Field | Width | Notes |
| --- | --- | --- | --- |
| 0 | magic | 8 bits | `0x8D` (a UTF-8 continuation byte, so a text file never starts with it; no file(1) rule tests it at offset 0) |
| 1 | version | bits 0–3 | 1 for the first stream format; 0 is invalid; 15 is reserved for a future extended-version layout and rejected by v1 decoders; 0.x streams carry no compatibility promise |
| 1 | mode | bits 4–5 | 0 = Blocked, 1 = Whole-file raw, 2 and 3 reserved |
| 1 | reserved | bits 6–7 | must be zero |
| 2.. | N | LEB128, 1–5 bytes | input length in bytes; N < 2³⁵; must be minimal (a multi-byte encoding whose last byte is `0x00` is rejected) |

- Every file ends with a 4-byte trailer `LE32(crc)`, where `crc` is CRC-32/ISO-HDLC (the gzip, PNG and zlib polynomial; check value `0xCBF43926` for `"123456789"`) over the N decoded bytes. Nothing may follow it.
- Whole-file raw: the header is followed by exactly N bytes, then the CRC. Total size is `2 + len(N) + N + 4`, at most `N + 11`.
- The empty input (N = 0) is the 3-byte header in mode 1 plus the CRC of no bytes, `00 00 00 00`: 7 bytes, because Whole-file raw wins ties (AD-10). Mode 0 with N = 0 is non-canonical and rejected.
- Blocked, N > 0: a sequence of Blocks follows until their actual lengths sum to N, then the CRC.

## 3. Block

A Block is a section (header and kind-specific fields, zero-padded to a byte boundary), then either the verbatim bytes (Raw) or one coder stream (§6).

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
| Direct, Previous-byte | none; the coder stream follows the section |
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

State per node: `p` (probability that the bit is 0, scaled by 65536, 16 bits, in [1, 65535]) and `n` (saturating count).

| Constant | Value | Status |
| --- | --- | --- |
| initial `p` | 32768; a scalar, or an 8-entry per-depth table adopted only if it beats the best scalar; one for all Candidates, every entry in [16, 65520] | **[MEASURE]** |
| count cap `CAP` | 24; candidates {24, 32, 60, 128, 255} | **[MEASURE]** |
| `RECIP` | normative table of `CAP + 1` entries, each in [1, 65535], published as data with a hash; candidate schedules `floor(65536 / (n + 2))` (DELTA = 1) and `floor(131072 / (2n + 3))` (DELTA = 0.5); `RECIP[CAP]` applies once `n = CAP` | **[MEASURE]** |

Coder frequencies for the decision at a node:

```
freq0 = max(p >> 4, 1)        # in [1, 4095] because p <= 65535
freq1 = 4096 - freq0
```

Update after a decision with bit `b`, in u32 or i64 arithmetic (`(65536 − p)·R` can exceed 2³¹, which overflows i32, but stays below 2³²):

```
R = RECIP[n]
if b == 0: p = p + (((65536 - p) * R) >> 16)
else:      p = p - ((p * R) >> 16)
n = min(n + 1, CAP)
```

Both operands are non-negative, so no signed-shift semantics are involved, and because `R < 65536` the update keeps `p` in [1, 65535] with no clamp. A long run of one bit value stalls at the same distance from certainty for either value (within 1/4096; at `CAP` = 24, `freq0` stalls at 4094 after zeros and `freq1` at 4095 after ones). The model updates only after the decision is coded, so the decoder always uses the same `p` the encoder used.

The earlier 12-bit rule (`p += ((target − p)·RECIP[n]) >> 16`, clamp [16, 4080]) is withdrawn: its signed floor shift let P(1) reach the clamp while P(0) stalled at 4070, which made a perfectly predicted 2048-byte Block cost 22.2 bytes of model code length against 5.3 bytes under this rule.

## 6. Entropy coder (AD-7)

Format version 1 codes the decisions of §5 with the binary range coder below, using `freq0` and `freq1` out of a total of 4096. The coder is not signalled in the stream, so any later change of coder is a format version bump (AD-15). The kernel keeps the coder behind one interface (encode decisions, decode decisions, exact size). An rANS coder with the same overhead is kept for reference in Appendix A; it is not part of format v1. Coder rules:

- Each Block's coder stream is self-contained: it starts from the coder's fixed initial state, ends with its own flush, and no coder state crosses a Block boundary.
- The format has no field that selects flushing or retaining coder state, and no field that selects a symbol width or a probability table.
- **Reference decode procedure**, which every MBSC implementation (kernel, Python reference decoder, clean-room decoder) follows. For each decision, in order: take `freq0` from the model; decide the bit from the current state; update the state for that bit; refill bytes until the state is back in range; then update the model. No update is deferred to a later iteration, no pending-symbol state is kept, and no test gates whether a decision is decoded.
- The number of bytes consumed is the Block's payload length (AD-8: payload length is derived, not stored). Reading past the end of input is a rejection.

### 6.1 Binary range coder (normative)

The carry-propagating range coder of Martin (1979) in the form used by LZMA, with 12-bit frequencies. Its stream is a byte sequence written and read most significant byte first; it is coder payload, not a multibyte integer field, so the little-endian rule of §1 does not apply.

Encoder state: `low` (unsigned, at least 33 bits), `range` (u32) = `0xFFFFFFFF`, `cache` (u8) = 0, `cache_size` = 1. Decisions are encoded in forward order:

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

flush: call shift_low() 5 times
```

- The first emitted byte is always `0x00` and is not written; the Block's coder stream is every byte after it. The flush leaves 4 bytes, and measured excess over the ideal code length is 24 to 32 bits per Block.
- Example vectors: one decision of bit 0 with `freq0 = 2048` gives `00 00 00 00`; bit 1 with `freq0 = 2048` gives `7F FF F8 00`.
- Decoding reads `code` from the first 4 bytes (most significant first), sets `range = 0xFFFFFFFF`, and rejects `code = 0xFFFFFFFF`. Once that is rejected, `code < range` holds after every step for any input, corrupt or not, so the decoder's `mod 2^32` operations never discard bits and u32 arithmetic cannot overflow. Then for each decision:

```
bound = (range >> 12) * freq0
if code < bound: bit = 0; range = bound
else:            bit = 1; code = code - bound; range = range - bound
while range < 2^24:
    range = (range << 8) mod 2^32
    code  = ((code << 8) | next_byte()) mod 2^32
```

- After the last decision, including its refill step, `code` must equal 0. Otherwise the stream is rejected.
- `(range >> 12) * freq0 < 2³²` and every operation fits u32 apart from the encoder's `low`, which needs u64 (or i64).
- Martin's range coding (1979) and LZMA's public-domain coder are long-standing prior art. The research found live patents on adaptive probability updates in video coding (Qualcomm US 10,554,988 and US 10,148,961; Tencent US 12,348,780); they concern §5, not the choice of coder, and are for counsel.

## 7. Decoder accept and reject set (AD-11, AD-12)

The decoder rejects: bad magic; version 0, 15 or any other unknown version; a set reserved bit; mode 2 or 3; mode 0 with N = 0; a non-minimal or out-of-range LEB128; reserved length code or kind; a non-canonical nominal length; non-zero padding; any parameter outside its limit; `s0`/`s1 ≥ m_mod`; a coder stream running past the end of input; an initial `code = 0xFFFFFFFF`; a final `code ≠ 0`; a stream that ends before N bytes are decoded or before the 4-byte CRC; a CRC mismatch; and any byte after the CRC. On rejection no output file is left (AD-12). All of these are covered by rejection vectors, and the Python reference decoder must reject each one too.

## 8. Test vectors (`spec/vectors/`)

| Set | Content |
| --- | --- |
| Evaluator | `(family, parameters, Block length) → ŷ sequence`, including boundary values for every limit (max coefficients, `u` at its maximum, degree 15, order 16, `q` = 8 and 24, `m_mod` at its limits) and a negative halfway case |
| Residual | all 65,536 `(y, ŷ)` pairs, as a hash of the result |
| Model | decision sequence → `p` and `freq0` sequence for edge counts (fresh node, `n = CAP`, saturation stall after a long run of 0 bits and of 1 bits), and the published `RECIP` table hash |
| Coder | decision list → bytes, including the two 1-decision vectors of §6.1, the 2048-symbol case and a carry that propagates through at least one pending `0xFF` byte |
| CRC | the check value for `"123456789"`, the empty input, and one file per mode |
| Blocks | one valid Block per kind and length code, including Raw and the partial final Block |
| Files | empty (7 bytes), 1 byte, 255, 256, 257, exactly one 2048 Block, a partial final Block, Whole-file raw |
| Rejection | one malformed file per rule in §7, including version 0, version 15, a CRC mismatch, a flipped Formula-parameter bit and a flipped Raw Block bit |

Vectors, the LUT and the `RECIP` table are published under CC0 so that any implementation may embed them. A coverage table maps every rule in §7 and every limit in §3–§4 to at least one vector, and a valid-stream generator extends the set for differential fuzzing (AD-15).

## 9. Open items before freeze

Resolved 2026-10-31 (open-questions research): magic `0x8D` and version 1 confirmed, with 0 invalid and 15 reserved; the 5-bit Block header confirmed (at most 0.0117 bits per byte of padding at 256-byte Blocks); the 1025-entry Q15 LUT confirmed (max error about 1.05 LSB, RMS 0.39; 4096 intervals gain nothing); the 4-byte coder flush kept for v1 (measured excess 24 to 32 bits per Block; a minimal flush would lose the final-state check); the model structure replaced (§5); a mandatory CRC-32 trailer added (§2).

Coder (resolved 2026-10-31): format version 1 uses the binary range coder (§6.1); rANS moves to Appendix A. A prototype of §5 with both coders (400 random Blocks of 1 to 2048 bytes) gave exact round-trips and payload consumption, the same 4-byte flush, mean excess 28.3 bits per Block for the range coder and 29.3 for rANS, and 0 of 3,000 single-byte payload mutations accepted by either end check. The range coder runs exercised 39,093 carries, 159 of them through pending `0xFF` bytes.

Still open:

1. Tune `CAP`, the `RECIP` schedule and the initial `p` on `tuning` inputs only, scored on the Baselines-only arm, then publish `RECIP` as data.
2. Measure the parameter cost of the `cw` scheme against fixed-width coefficients and against the minimum viable Q, and whether a narrower ω than 32 bits pays off.
3. Revisit a shorter flush (a version bump, per AD-15) only if the benchmark shows the flush share is material on the classes that decide SM-1.

## Appendix A. rANS coder (informative; not part of format v1)

Kept for reference and for a possible later format version. Adopting it requires a freedom-to-operate review of Microsoft US 11,234,023 B2 and its family (`../../research/rans-patent-claim-chart-2026-10-31.md`) and a format version bump. It follows the §6 coder rules, and its prototype overhead matched the range coder's (mean excess 29.3 against 28.3 bits per Block).


- State: unsigned 32-bit `x`, normalized interval `[L, 256·L)` with `L = 2²³`. Scale bits 12 (total 4096).
- Each decision has `freq` and `start`: bit 0 gets `freq = freq0`, `start = 0`; bit 1 gets `freq = freq1`, `start = freq0` (§5).
- This is the byte-wise rANS of Duda (2013) as realized in ryg_rans `rans_byte.h` (2014), cited as prior art. The format has no signalled or selective flush.
- Encoding runs over the buffered decisions of the Block in reverse order, starting from `x = L`:

```
x_max = ((L >> 12) << 8) * freq
while x >= x_max: emit_byte(x & 0xFF); x >>= 8
x = ((x / freq) << 12) + (x % freq) + start
```

- Termination and layout: the encoder collects renormalization bytes in emission order. After the first-in-order decision has been encoded, the stream is laid out as `LE32(x_final)` followed by the collected renormalization bytes in reverse emission order, so the decoder reads it strictly forward. Reference vectors: one decision of bit 0 with `freq0 = 2048` (`p` = 32768) gives `00 00 00 01`; bit 1 with `freq0 = 2048` gives `00 08 00 01`.
- Decoding reads `x` from the first 4 bytes (little-endian) and rejects `x` outside `[L, 2³¹)`, then for each decision:

```
slot = x & 4095
bit = 0 if slot < freq0 else 1
(freq, start) as above
x = freq * (x >> 12) + slot - start
while x < L: x = (x << 8) | next_byte()
```

- After the last decision, including its renormalization step, `x` must equal `L`. Otherwise the stream is rejected.
- Per-Block rANS overhead is the 4-byte flush; measured excess over the ideal code length is 24 to 32 bits per Block. The Block header and parameters are on top of that.
