# Adversarial Review — Architecture Spine, MBSC v1

Reviewer stance: adversary. Method: for each AD, build two units one level down that each obey every AD to the letter and still cannot interoperate or still disagree on a PRD testable consequence. Every finding below is a pair, the reason both are "conformant", the PRD consequence it breaks, and a concrete AD fix. The spine was not edited.

Inputs read: `ARCHITECTURE-SPINE.md`, `prd.md`. The spine's companion `bitstream-spec-seed.md` is listed in front matter and `companions:` but **does not exist on disk** (only `ARCHITECTURE-SPINE.md` is in the folder), so everything the spine defers to it is currently undefined (see F3).

## Verdict

The spine fixes the right structure (crate graph, one coding path, additive Blocks), but it is not yet sufficient to make two conformant decoders produce the same bytes. Evaluator semantics, count-to-rANS mapping, header field layouts and the decoder's accept/reject set are each left to a spec that does not exist. Encoder-side tie-breaking and the benchmark arms are also underspecified enough to break FR-5/FR-6/FR-12 determinism and SM-1 attribution. Fixable with about eight AD edits; none requires a redesign.

## Severity summary

| ID | Sev | Hole | Area |
| --- | --- | --- | --- |
| F1 | critical | Adaptive counts → rANS frequencies/total is unstated; non-power-of-two totals may break the standard rANS invariant | AD-7, AD-8 |
| F2 | critical | Evaluator: "normalized position" vs AD-4 and Block length; Horner rounding; trig phase wrap; limits unlisted; out-of-range field values | AD-4, AD-5, AD-6 |
| F3 | high | Header and Block-header layouts deferred, but they are the primary decoder divergence point (spec seed missing) | AD-6, AD-11, Deferred |
| F4 | high | Decoder accept/reject set undefined (non-canonical LEB128, trailing bytes, last-Block nominal, final rANS state, reserved length code) | AD-8, AD-9, AD-11, AD-12 |
| F5 | high | Tie-break "total order" is not a total order over partitions; Candidate id undefined; Whole-file raw tie | AD-9, AD-10, AD-13 |
| F6 | high | Benchmark arms (Baselines-only, forced single Baseline, fixed length) not enumerated; comparator strength differs between implementers | AD-9, AD-14 |
| F7 | medium | Two owners of file-header/length semantics (kernel parse vs encoder writer vs DP) | AD-1, crate table |
| F8 | medium | Constant prediction is degenerate under a symmetric order-0 model; Direct ≡ Constant(0) | AD-2, AD-4, AD-7 |
| F9 | medium | Codebook: total cap vs PRD per-family bound; floating point in the recipe breaks cross-platform SM-4; hash serialization | AD-13 |
| F10 | medium | Entropy screen makes `compress` output differ from `benchmark` sizes | AD-10, AD-13, FR-12 |
| F11 | medium | Report breakdown (header / parameter / model signalling / payload) cannot be computed uniquely; "model signalling" is zero by construction | AD-7, AD-14, FR-8, FR-14 |
| F12 | medium | AD-2/AD-8 contradiction for Raw Block (no rANS stream) | AD-2, AD-8 |
| F13 | low | `cargo tree -p mbsc-kernel` includes dev-dependencies, so the AD-1 CI gate and AD-3 property tests collide | AD-1, Stack |
| F14 | low | Misc: JSON writer dependency missing from Stack; LUT single source; "fixed-size header" wording in PRD vs LEB128; N ≥ 2³⁵; per-Block cost retention scope | various |

---

## F1 — critical — Count → rANS mapping (AD-7, AD-8)

**Pair.** Kernel author K uses `freq(s) = count[s]`, `cum(s) = Σ_{t<s} count[t]`, `total = Σ count`, no normalization. Python reference author P reads "(cumulative, frequency, total) decisions … total not required to be a power of two" and normalizes counts to a fixed total (say 2¹²) with a floor of 1 per symbol, which is a legal reading of "counts … constants frozen in the spec". Both are "adaptive order-0 with exact integer arithmetic". The Bitstreams are mutually undecodable.

**Further splits under the same wording.**
- Cumulative order: ascending symbol value, or by count? Symbol value is `δ mod 256` (AD-3), but nothing says cumulative order is ascending in that value.
- Update timing: is the coded symbol's count incremented after coding (causal) or before? Is the first symbol coded with the initial counts?
- "Rescale on overflow": overflow of what bound, checked before or after the increment, halving rule `(c+1)>>1` vs `c>>1` with min 1, applied to all 256 counts or lazily? Each variant is a different decoder.
- rANS parameters are listed only by name ("state initialization, renormalization byte order, termination"). State width, lower bound L, byte base b, maximum total, and whether the decoder renormalizes before or after the symbol step are all open.
- **Invariant risk.** AD-7 explicitly allows a total that is not a power of two and varies per symbol. The usual rANS interval argument (state kept in [L, bL), encoder renorm bound `floor(L/T)·b·f`) needs `T | L`. If T does not divide L, the post-encode state can fall below L, the decoder then reads a byte the encoder did not write, and the stream desynchronizes. A conformant implementer following a textbook rANS (any version) will pass most tests and fail on specific totals. I believe this holds but have not proven it; the AD must force a proof or a different design.

**PRD consequences broken.** FR-8 ("spec states how frequencies are normalized and rescaled …"), FR-2/FR-11 (independent decoder from spec), SM-2, SM-5.

**Fix (new wording for AD-7).**
> The model state is 256 unsigned counts. At Block start every count is `c0`. To code symbol `s` (the byte `δ mod 256`, or the byte itself under direct coding): `freq(s) = count[s]`, `cum(s) = Σ_{t<s} count[t]` in ascending symbol value, `total = Σ_t count[t]`; counts are used as frequencies with no normalization. After coding `s`, `count[s] += inc`. The constants satisfy `256·c0 + 2048·inc ≤ T_max`, where 2048 is the maximum Block length, so no rescale path exists in v1; any future rescale rule is a format-version bump. The rANS state width, `L`, byte base, `T_max`, renormalization point (before encode / after decode), initial state, final-state check and flush layout are fixed in `spec/bitstream.md` before the first spec freeze. The spec must either (a) restrict totals so that the interval invariant provably holds (for example a power-of-two total reached by a stated integer rule), or (b) include a proof of encode∘decode invertibility for every `total ≤ T_max`, backed by an exhaustive test at all boundary totals and golden vectors at `total = T_max` and `total = 256·c0`.

Also move "rescale on overflow" out of the AD-7 rule text (it becomes dead code once the inequality above holds) and out of the Deferred row "Probability-model constants" as a structural item; only `c0`, `inc` remain tunable.

---

## F2 — critical — Normative Evaluator under-pinned (AD-4, AD-5, AD-6)

**Pair 1 (position).** AD-5 says "a 16-bit normalized position"; AD-4 says ŷₙ "depends only on Block-local position `n` and the Formula's own parameters". Implementer A normalizes `t = n·65536 / nominal_length`. Implementer B normalizes `t = n·65536 / actual_length`. Implementer C uses `t = n` (or `n << 5`). All obey both ADs ("normalized" is undefined; A and B read "Block-local" loosely). For the last Block, A and B diverge from each other and from the encoder's DP-time costing, because the nominal code (AD-9) and actual length differ. A Formula stream encoded by one cannot be decoded by another, and the Block's cost is not independent of N, which violates AD-4's own premise.

**Pair 2 (Horner).** AD-5 states the bound "polynomial Horner evaluation with 32-bit coefficients, a 16-bit normalized position and degree at most 15 stays below 2⁵²". That bound holds only if every Horner step rescales (a shift after each multiply-add). Implementer A shifts per step by 16, implementer B accumulates and shifts once by Q at the end (which overflows i64 at degree ≥ 2 without wrapping, so the "limits" must be different). AD-5's rounding rule `(v + (1 << (Q−1))) >> Q` is stated once and does not say how often it is applied or how it interacts with the position's fractional bits.

**Pair 3 (trig).** FR-2 names "trigonometric argument reduction and lookup-table interpolation" as required spec content; AD-5 mentions neither. A: phase = `(n·inc) mod 2³²`, `k·phase` wrapped mod 2³², LUT index = top bits, linear interpolation with floor. B: phase kept as an i64 (never wrapped, which AD-5's "overflow impossible by construction" encourages), argument reduced with `rem_euclid`, nearest-entry lookup. Different ŷ. In Python (unbounded ints) the wrap must be written explicitly, in Rust it is a conscious `wrapping_mul` or a bug — so the two will differ exactly where nobody tested.

**Pair 4 (limits).** "The decoder rejects any stream whose parameters break a limit", but no limit table exists, only an example. Kernel rejects degree > 15; refdec rejects only what its own bigint arithmetic cannot handle. Stream accepted by one, rejected by the other. In Rust release builds with default profile, an unchecked overflow wraps silently, so any gap in the limit table yields output that Python (arbitrary precision) cannot reproduce.

**Pair 5 (field values).** Q is "stored as Q − 8 in 5 bits" (0..31) but valid Q is [8, 24] (stored 0..16). Values 17..31 are neither reserved bits (AD-11) nor a stated limit. Kernel clamps, refdec rejects, another computes `1 << (Q−1)` with Q = 39 and shifts out of i64.

**Pair 6 (recurrence).** AD-6 gives recurrence "one family with order (1 or 2), initial values and m_mod", but not: is the state kept modulo `m_mod` (then ŷ = state rem 256? state itself if `m_mod ≤ 256`?), what are the coefficient widths/signs and Q role, and is `m_mod` required ≥ 1? Division by zero is a panic, which AD-12 forbids.

**PRD consequences broken.** FR-2 (every bullet in the "spec states" list), SM-2, SM-5, FR-13.

**Fix (AD-5 and AD-4 amendment).**
> Position `t` is the unsigned integer `n` (0 ≤ n ≤ 2047) and never depends on the Block's nominal or actual length. Evaluator step definitions (for each family: the exact recurrence of integer operations, which operation rounds, the shift applied at each Horner step and its rounding, trigonometric phase as an unsigned 32-bit value with explicit wraparound, the LUT index split, the interpolation formula and its rounding) are normative in `spec/bitstream.md`; AD-5 states only the invariants. Every stored field has an inclusive [min, max] valid range in a single limits table; **a stored value outside its range is an error, never clamped or wrapped**. All kernel arithmetic uses checked operations that map overflow to `Err` (defence in depth, even though limits make it unreachable). `m_mod ≥ 1` is a limit. The golden vectors include, per family, one vector at each limit (max degree/order, max/min Q, max coefficient magnitude, `m_mod` bounds) and one rejection vector just outside each limit, and the Python reference decoder must reject exactly the same set.

---

## F3 — high — Header and Block-header layouts are a divergence point, not Deferred

**Pair.** The Deferred row says "Block header kind codes and field widths … not a divergence point once AD-6 and AD-11 hold". They are the *only* thing a second decoder needs to parse a Block. AD-6 and AD-11 do not fix:
- the packing of "one byte of format version and mode" (4+4 bits? version in low bits or high bits? width of each?),
- the magic byte value,
- candidate kind codes and their order (Raw, direct, constant, previous-byte, three families),
- the order of fields inside a Block header and parameters,
- signed coefficient encoding (two's complement of a stated width? zigzag?) — AD-6 gives an offset encoding for counts only,
- whether a 32-bit coefficient is a "multibyte integer" (byte-aligned little-endian, AD-11) or a "packed bit field" (LSB-first, straddling bytes); AD-11 names both and says nothing about which a coefficient is,
- whether padding to the byte boundary at the end of a header/parameter section is mandatory-zero and checked.

Kernel author puts coefficients as byte-aligned LE after a bit-packed header; refdec author packs everything in one LSB-first bit stream. Both comply with AD-6 and AD-11.

**Also.** Rejecting this Deferred item is consistent with the evidence that the "companion" `bitstream-spec-seed.md` is absent: the spine currently has no document that fixes any of the above.

**PRD consequences broken.** FR-9 ("spec defines bit order … and a header parser"), FR-11, SM-5.

**Fix.** Remove the Deferred row. Add AD-11 text:
> The Bitstream is a sequence of byte-aligned sections. Within a section, fields are packed LSB-first, in the order written in `spec/bitstream.md`, fixed-width, unsigned, with signed fields stored as two's complement of their stated width. Multibyte integers outside bit-packed sections (the LEB128 N and the raw payload) are byte-aligned. Each section ends with zero padding to a byte boundary; non-zero padding is an error. `spec/bitstream.md` contains, before Epic 1 starts, the magic value, version/mode bit layout, kind-code table and field-width table. The companion `bitstream-spec-seed.md` must exist and be reviewed before any implementation story begins; if it stays a companion, list its path and checksum in the spine front matter.

---

## F4 — high — Decoder accept/reject set is undefined (AD-8, AD-9, AD-11, AD-12)

AD-12 says only "cannot panic". Two decoders that never panic can still differ on which inputs they accept, which means "decoder output differs between conformant implementations" (accept with output vs reject). Colliding pairs, each legal under the current text:

| Input | Decoder A | Decoder B |
| --- | --- | --- |
| LEB128 `N` as `0x80 0x00` (non-canonical zero) or a 5th byte with bits above 2³⁵ | accepts | rejects |
| Bytes after the last Block (N reached) | ignores | rejects |
| Last-Block nominal larger than minimal (e.g. nominal 2048 with 300 bytes left) | accepts (`min(nominal, remaining)` works) | rejects as non-canonical |
| Length code 3 (reserved) | rejects | treats as unknown nominal |
| rANS final state ≠ initial state `L` after the last symbol | accepts | rejects |
| Raw Block payload shorter than actual length | rejects | rejects (same) |
| Decoder renormalizes after the last symbol (reads more bytes) vs not | consumes different byte count, so next Block begins at a different offset | |
| Whole-file raw mode with trailing bytes beyond N | ? | ? |

AD-9's sentence "the decoder rejects a stream whose Blocks do not consume exactly N" is also vacuous as written: with `actual = min(nominal, remaining)` no stream can over-consume N; only a stream ending early, or one with trailing bytes (not mentioned), can fail.

**PRD consequences broken.** FR-13 (truncated or malformed Bitstream rejected), SM-2, SM-5.

**Fix (new AD-12 text).**
> The decoder accepts exactly the canonical encodings. It rejects: a non-minimal LEB128 or N ≥ 2³⁵; an unknown version/mode, reserved bit, reserved length code (3), unknown kind, nonzero padding; any field outside its limit-table range; a rANS final state different from the initial state; end of input before N bytes are produced; any byte after the last Block (or after N raw bytes in Whole-file raw mode). A Block whose nominal length is larger than the smallest covering nominal for the remaining bytes is **accepted** (decoded with `min(nominal, remaining)`); encoders always emit the minimal nominal. Decode consumes exactly the bytes written by the encoder for each Block; the rANS decoder renormalizes after each symbol and the spec defines the final state so the consumed count is identical in every implementation. The golden vector set includes at least one malformed vector per rejection class with the required error class, and the Python reference decoder is run against them in CI alongside the valid vectors.

---

## F5 — high — The tie-break "total order" is not total; Candidate id undefined (AD-9, AD-10, AD-13)

**Pair.** AD-10: ties "broken by the total order (fewest Blocks, then lowest Candidate id)".
- "Fewest Blocks" is a property of a partition, not of one Block's Candidate choice, so at the Block-selector level the first key is vacuous, and at the DP level it still leaves ties between partitions with equal size and equal Block count (e.g. `256+1024+…` vs `1024+256+…` of the same total, or `1024+1024` vs `2048` never ties on count, but `256×4+1024` vs `1024+256×4` does). Implementer A resolves forward (prefer the first Block longer), B backward (prefer the last Block longer). Both output deterministic, valid, minimum-size Bitstreams; they differ byte for byte, which breaks FR-12 determinism if the DP is ever re-implemented (e.g. bench vs cli) and breaks AD-13's "every reduction uses the total order of AD-10" because the DP reduction has no defined order.
- "Lowest Candidate id" is undefined. Formula ids come from the Codebook, which may change without a format bump (AD-15), so tie outcomes change when a Codebook entry is inserted. The relative order of Raw, Direct, Constant(c), Previous-byte and Formulas is not stated. If a Formula's id sorts below a Baseline's, a size tie is awarded to the Formula, and SM-1/FR-14 "share of Blocks won by each Series family" is inflated by ties with no size reduction.
- Whole-file raw vs Blocked: "takes the smaller" with no tie rule. Equal for N = 0 (both 3 bytes, two valid Bitstreams for the empty input).
- The last-Block nominal code (smallest covering) is not in any order key but changes the output byte (same size).

**PRD consequences broken.** FR-5 "search result deterministic", FR-6, FR-12, SM-1 (attribution), SM-4.

**Fix (AD-10 replacement).**
> Candidate ids are a fixed enumeration: Raw, Direct, Previous-byte, Constant(0..255), then Formulas by Codebook order (polynomial, trigonometric, recurrence). Ties are always resolved in favour of the lower id, so Formulas win only when strictly smaller than every Baseline. A partition is compared by the key `(total size, Block count, sequence of (nominal length code, Candidate id) lexicographically in file order)`; DP computes this key with tie preference going to the lexicographically smallest sequence from the start of the file. Whole-file raw wins ties against Blocked output. The empty input is emitted in Whole-file raw mode.

---

## F6 — high — Benchmark arms are not defined as data (AD-9, AD-14, FR-14, SM-1)

**Pair.** AD-14: "The Baselines-only arm is the same code with Formulas disabled". The PRD adds "Whole-file raw mode … stays available" and "applied independently", and defines Gain against "the best whole-input Baseline" (a Baseline *forced on for a whole input*). The spine never says whether:
- a forced single-Baseline run allows Raw Blocks (and Whole-file raw) as a fallback or not;
- a forced fixed-length run ({2048}) uses the DP or a plain split, and whether "the last Block smallest covering nominal" is evaluated against the allowed set or the global set ({2048} alone cannot form the "smallest" cover for a 100-byte tail if the rule is global);
- the Constant baseline's per-Block constant is chosen exhaustively (best of 256 by exact size, ties to the lowest) or heuristically (mode of the Block), see F8;
- whether Whole-file raw participates in "best whole-input Baseline".

Implementer A lets a forced Baseline fall back to Raw Blocks and Whole-file raw, so no Baseline ever exceeds raw size. Implementer B disallows fallback, so a Baseline can exceed raw. The comparator strength differs by up to the whole Block framing; SM-1 can flip. Neither violates AD-14.

**Fix (AD-14 amendment, a table).**
> The benchmark arms are exactly:
> | Arm | Candidate set | Allowed lengths | Raw Block | Whole-file raw check |
> | Full MBSC | all | {256,1024,2048} | yes | yes |
> | Baselines-only | all except Formulas | {256,1024,2048} | yes | yes |
> | Forced Baseline B (Direct, Previous-byte, Constant) | B (+ Raw) | {256,1024,2048} | yes | no |
> | Fixed length L | all | {L} | yes | yes |
> | Whole-file raw | none | none | n/a | n/a |
>
> All use the same DP and the F5 total order. "Best whole-input Baseline" per input is the minimum over the three forced-Baseline arms only. Constant(c) is evaluated for all 256 values of c, and Block-level choice is by exact size. "Smallest nominal covering the remainder" is evaluated within the arm's allowed set (and if no allowed length covers the remainder, the largest allowed length is used and the remainder is covered by additional Blocks per the DP).

(Decide the Raw/Whole-file-raw cells deliberately; the point is that each implementer reads the same cells.)

---

## F7 — medium — Two owners of file-level framing (crate table, AD-1)

The crate table assigns "Bitstream parse and write primitives" to `mbsc-kernel` and "file writer" to `mbsc-encoder`. Header serialization (magic, version/mode, LEB128 N), the Whole-file raw writer, the exact byte length of the header (needed for Whole-file raw vs Blocked comparisons and the N + 7 bound), and the `min(nominal, remaining)` / "smallest covering nominal" computations are each implemented once on the encoder side and once in the kernel parser. They can silently drift (parser accepts what the writer cannot emit, the DP uses its own copy of the length function). The round-trip test only covers what the writer emits.

**Fix (AD-8/AD-11).**
> `mbsc-kernel` owns every bit-level definition: `write_file_header`/`parse_file_header`, `leb128_len`, `block_actual_len(nominal, remaining)`, `min_covering_nominal(remaining, allowed)`, and `write_raw_file`. `mbsc-encoder`'s file writer only sequences calls to these; it contains no layout constants. A test asserts `parse(write(x)) = x` for headers over the full N range.

Also state what "exact size before the whole file is encoded" means: the file header's size depends only on N (known up front) and is identical for every Candidate, so it never changes a selection; it is added once, at the file level, after the partition DP, and only the Whole-file raw comparison needs it. Without this sentence an implementer may feed `N`-dependent bytes into per-Block cost, which breaks AD-8's additivity claim.

---

## F8 — medium — Constant prediction is degenerate; Direct ≡ Constant(0) (AD-2, AD-4, AD-7)

Under a model whose initial counts are uniform and whose update is symmetric (AD-7 as written), coding `δ = y − c` with any constant `c` is a relabelling of the symbol alphabet. The adaptive cost depends only on the multiset of symbol counts (no rescale on the 2048-length-bounded path), so Constant(c) costs the same as Direct coding for every `c`, plus one byte for the carried constant. Constant is then always strictly dominated by Direct (differences remaining only from rANS rounding at different cumulative positions, which is noise). Direct itself is exactly Constant(0) without the constant byte.

Consequences: (1) the "constant" Baseline in the PRD is never selected, so Gain "versus the best whole-input Baseline" is effectively versus Direct and Previous-byte; (2) a size difference of ±1 byte from rANS rounding can flip "wins" and the per-Candidate shares in the report; (3) if the deferred constants later make initial counts skewed towards symbol 0 (a common tuning for residuals), Direct is handicapped relative to Previous-byte and Constant, inflating Formulas' and Baselines' apparent gain.

**Fix.** Add to AD-7: "Initial counts are uniform (`c0` identical for every symbol); non-uniform priors are out of v1, because one model serves residual symbols and raw bytes alike and a skewed prior would favour Predictors over Direct coding." Add to AD-4/FR-7 mapping: "Constant prediction is retained as a PRD-required Baseline; the report notes it is dominated by Direct under the v1 model (expected, not a defect). If PM wants Constant to be meaningful, it needs a per-Block seed in the model, which is a model change and a version bump." Alternatively move Constant to Deferred with a justification linking to PRD FR-7 (needs PM sign-off).

---

## F9 — medium — Codebook cap, cross-platform determinism, hash (AD-13)

- **Per-family bound.** FR-4 requires a per-family upper bound and at least one Formula from each family. AD-13 caps the *total* at 4096. An implementer may ship 4096 polynomials and nothing else.
- **Floating point in the recipe.** AD-13 requires a "documented deterministic recipe" but does not forbid `f64::sin`/`cos`, whose results are not guaranteed identical across x86-64 and aarch64 libm. Codebook contents (and therefore the selected Formula, and therefore Total encoded size) can differ between the two target platforms, breaking SM-4 "identical Total encoded sizes", and AD-13's "byte-identical" test only compares thread counts on one machine.
- **Hash.** "Content hash recorded in every benchmark report" does not define the serialized form hashed (field order, widths, whether the Codebook order is part of the hash, which matters for F5 id order). Two bench builds can report different hashes for identical Codebooks.

**Fix (AD-13).**
> Per-family caps: `|poly| ≤ 1024, |trig| ≤ 1536, |recur| ≤ 1536` (total ≤ 4096) and each family has at least one Formula. The Codebook recipe uses integer arithmetic only (or the kernel Evaluator); no `f32`/`f64` in anything that determines which Candidates exist, their order, or their sizes. The determinism test runs on both target platforms and compares output hashes. The Codebook hash is SHA-256 over the canonical byte serialization defined in the recipe document (Formula in Codebook order, each as its bit-packed parameter block from the kernel writer).

(Numbers above are placeholders; the point is that a per-family bound exist, as FR-4 states.)

---

## F10 — medium — Entropy screen versus determinism (AD-10, AD-13, FR-12)

AD-10 makes the zero-order screen an "optional speed-up" disabled in `benchmark`. If `compress` enables it, `compress` and `benchmark` can emit different Bitstreams (and sizes) for the same input and Codebook. FR-12 requires the same input and Codebook to yield the same Bitstream; SM-C3 forbids trading fidelity for speed; users will compare `compress` output to benchmark sizes. Two conformant implementers: A enables the screen by default in `compress`, B disables it.

**Fix.** "The screen is either provably non-pruning (it may only skip a Candidate whose exact size is bounded below by the screen's value and that bound exceeds the current best) or it is off. If any lossy screen exists it is a CLI flag, default off, and its state is part of the determinism key in AD-13's test and recorded in the report."

---

## F11 — medium — Report breakdown categories are not computable uniquely (AD-7, AD-14, FR-8, FR-14)

PRD FR-14 requires "header, Formula parameter, model signalling and payload bytes" and FR-8 requires "per-Block model signalling cost … measured and reported". The spine makes the model adaptive with nothing transmitted (AD-7), so "model signalling" is identically zero. A report that prints zero looks like the PRD's Open Question 2 is answered, but the real cost at 256-byte Blocks is the **learning cost** (adaptive code length minus the Block's empirical-entropy code length), which is not a signalling cost and is not in any category. Separately, with Block headers bit-packed and byte-aligned (AD-8), bytes shared between header and parameter bits cannot be attributed to one category by two implementers the same way; padding has no category though the PRD's Total encoded size includes "padding".

**Fix (AD-14).**
> Total encoded size is partitioned into exactly: file header; per-Block header; Formula/constant parameter; rANS payload (including flush); padding. Attribution is by bit, rounding padding bits into "padding". Model signalling is reported as 0 by construction (AD-7), and the report adds an *adaptation overhead* column per class: payload bits minus the Block's static zero-order entropy on its symbols. The per-Block per-Candidate costs retained for later retrieval validation (FR-14) are for every `(offset, length)` span the DP evaluated, not only the chosen partition (state which, and the storage bound).

---

## F12 — medium — AD-2 / AD-8 contradiction for Raw Block

AD-2 says Raw Block is a Candidate of `encode_block` that "differs only in how its ŷ stream is produced". AD-8 says every Block "consists of its header, parameters and its own rANS stream". A Raw Block has no rANS stream and no ŷ. A kernel author who follows AD-8 wraps raw bytes in an rANS stream with a degenerate model (4-byte flush plus payload), and the refdec author who follows the PRD glossary ("stored uncompressed") emits bytes verbatim. Raw size and the "strictly smaller" test in AD-10 then differ.

**Fix.** AD-2/AD-8: "Raw Block is the one Candidate that has no ŷ and no rANS stream: header plus `actual_len` verbatim bytes. All other Candidates share the model, rANS stream and termination."

---

## F13 — low — AD-1 CI gate collides with AD-3/AD-15 test tooling

`cargo tree -p mbsc-kernel` lists dev-dependencies by default. AD-3 requires property tests (proptest, listed as dev dependency in Stack). If those tests live in `mbsc-kernel/tests`, the CI check in AD-1 fails; if the check is changed ad hoc to `-e normal` the guarantee is unstated. Two implementers pick different resolutions.

**Fix.** AD-1: "CI runs `cargo tree -p mbsc-kernel -e normal,build` and fails on any external crate. Property tests that need proptest live in a separate `crates/mbsc-conformance` (dev) crate, or proptest is accepted in kernel `[dev-dependencies]` only."

## F14 — low — Miscellany

- **JSON dependency.** Conventions say "JSON out" but Stack lists `serde, toml, sha2` only. One implementer adds `serde_json` (unpinned), another hand-writes JSON. Pin it or state hand-written.
- **LUT single source.** "Sine LUT's values are normative data (published with a hash)": say `spec/sine_lut.bin` is the one file embedded in the kernel (`include_bytes!`, hash asserted by a test) and read by the Python decoder; otherwise there are two copies.
- **Whole-file raw "fixed-size header".** The PRD glossary says Whole-file raw has "one fixed-size header"; AD-11 uses LEB128, a variable size (3–7 bytes). The bound N + 7 is right, but the PRD wording and FR-10's "small constant" should be reconciled (note in the traceability section, no change to the spine).
- **N ≥ 2³⁵.** AD-11 limits N < 2³⁵. The encoder behaviour for larger input (error, exit 1, no output) is not stated.
- **`benchmark` fixed-length last-Block rule** — covered by F6.
- **Decoder memory.** AD-12's 64 KiB budget is compatible with the PRD's "multiple of Block length + declared output size" (stricter), fine; but the budget has no accounting (LUT is code/const, counts 256×u32, output buffer ≤ 2048, rANS state) so the "about 64 KiB" cannot be tested. Add a test hook or define the budget as "no heap allocation larger than 2·2048 bytes plus O(1)".

---

## What held up

- Crate-graph enforcement of kernel isolation (AD-1) is sound apart from F13.
- Additivity of Blocks (AD-8) and a plain-sequence DP are sound; the N dependence of the file header is constant across Candidates (see F7 for the sentence that makes it explicit).
- `δ` / `y` reconstruction and `δ mod 256` symbol mapping (AD-3) are unambiguous, including the −128 / +128 equivalence.
- Previous-byte reads decoded `y` in-Block only, first byte predicted 0: unambiguous (AD-4).
- The N + 7 bound is correct as stated for Whole-file raw (`2 + LEB128(N) + N`, LEB128 at most 5 bytes).

## Deferred items challenged

| Deferred item | Verdict |
| --- | --- |
| Block header kind codes and field widths | **Not deferrable** (F3). Move into the spine or the seed before implementation. |
| Probability-model constants | Defer only `c0`, `inc`; structure (mapping, ordering, rescale absence, T_max, rANS parameters) must move into AD-7 (F1). |
| Global rANS stream, minimal flush | Fine to defer; but record the flush size in AD-8 (it is the per-Block fixed cost that drives the Block-length choice, OQ2/OQ6). |
| Integrity checksum (OQ7) | Fine; F4's canonical-or-reject rules are the compensating control. |
| Corpus choice, CI provider, licence, encode time budget | Fine. |

## Net new or tightened ADs requested

1. AD-7: identity count→freq mapping, ascending cumulative, uniform `c0`, `256·c0 + 2048·inc ≤ T_max`, no rescale, rANS parameters and invertibility proof (F1, F8).
2. AD-5/AD-4/AD-6: position is `n`; limits table; reject-never-clamp; checked arithmetic; Q valid range; recurrence state semantics; boundary golden vectors (F2).
3. AD-11: section/packing rules, signedness, padding-zero, magic/version/kind tables due before implementation (F3).
4. AD-12: canonical-or-reject list and malformed-vector CI (F4).
5. AD-10/AD-9/AD-13: total order over partitions, Candidate id enumeration, ties favour non-Formula, Whole-file raw tie, minimal last nominal (F5).
6. AD-14: arm table, breakdown categories, retention scope (F6, F11).
7. AD-1/AD-8/AD-2: kernel owns all layout functions; Raw Block exception; CI tree command (F7, F12, F13).
8. AD-13: per-family caps, integer-only recipe, hash serialization, screen policy (F9, F10).
