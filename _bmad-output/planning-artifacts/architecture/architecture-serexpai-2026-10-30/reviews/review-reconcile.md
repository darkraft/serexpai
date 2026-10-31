# Reconciliation Review: ARCHITECTURE-SPINE vs PRD, architecture review, brief addendum

Reviewer: independent reconciler. The spine was not edited.
Spine: `architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md` (draft, 2026-10-30).
Inputs checked: the PRD (FR-1..15 consequences, SM-1..5, SM-C1..C3, Non-Goals, Open Questions), `docs/mbsc_architecture_review.md`, and the brief `addendum.md`.

Method: every testable consequence or requirement was traced to an AD, a Consistency Convention, a Deferred row, or a Capability-map entry. A gap is an item that none of these carries. Items the spine intentionally pushes to the companion Bitstream spec seed (`bitstream-spec-seed.md`) are marked **SPEC-SEED**. They count as gaps only when the spine does not say the spec seed must carry them.

Note: neither companion (`bitstream-spec-seed.md`, `walkthrough.html`) exists on disk yet. Every SPEC-SEED item below is therefore unanchored until that file is written (see G-8).

## 1. Verdict

The spine lands the structural core well. Modular residual, normative evaluator, family layouts, causal order-0 model, self-contained Blocks, exact-size selection, N+7 bound, decoder safety and determinism are all pinned by ADs. The gaps are in the quiet layer. These are the benchmark report's content, the SM-1 verdict logic, the comparator arms, the edge-case test inventory, and the "publication gate must not block the benchmark" sequencing.

## 2. Gaps (severity-ranked)

### G-1 [HIGH] SM-1 verdict rule is not carried (three-valued outcome, two comparators, two-class threshold)

- Source: PRD SM-1, FR-14 bullet 2, Glossary "Gain", UJ-1 ("see in one read whether MBSC won on at least two real classes"), Vision ("an ambiguous outcome is not acceptable").
- Spine: AD-14 says "the verdict reads `evaluation` inputs only". It does not define the verdict.
- Not carried:
  - Gain must hold against both the Baselines-only arm and the best whole-input Baseline.
  - It must hold on at least two distinct Real classes.
  - The outcome is *inconclusive*, not negative, when fewer than two Real classes are testable.
  - The verdict is stated per Real class, with per-input wins and margin.
  - A reduction that comes only from switching among Baselines does not count.
- Impact: `mbsc-bench` is free to implement a binary pass/fail. That would break the project's central deliverable and the "negative result is acceptable, ambiguous is not" tone.
- Fix: add to AD-14 a verdict rule: `positive | negative | inconclusive`, per-class Gain against both comparators, per-input wins and margin, and the two-class threshold.

### G-2 [HIGH] Whole-input Baseline arms and Whole-file raw column not defined

- Source: PRD FR-7 bullet 2, FR-14 bullet 1, Glossary "Gain" (best whole-input Baseline per input), "Baselines-only arm".
- Spine: AD-14 names only "the Baselines-only arm". AD-9 mentions "fixed-length benchmark runs".
- Not carried:
  - Each Baseline forced on for the whole input and reported on its own: constant, previous-byte, direct entropy.
  - Whether a forced whole-input Baseline uses the DP with its own allowed set (single Candidate kind) or a fixed Block length.
  - Whole-file raw mode as a reported column.
- Impact: SM-1's second comparator ("best whole-input Baseline") has no defined computation. Two implementers would get different Gain.
- Fix: in AD-14, enumerate the benchmark arms: {each Baseline forced, Baselines-only, full MBSC, Whole-file raw} × {partition rule per arm}. State that the best whole-input Baseline is the per-input minimum over the three forced arms.

### G-3 [MEDIUM] FR-14 report content dropped

- Source: PRD FR-14 bullets 3, 5 and 6, FR-8 bullet 3, OQ2, review §6 and the addendum ("65% zero residual is a benchmark hypothesis").
- Spine: AD-14's report records only Codebook version and hash, Block lengths, tool version and manifest hash. Total size is the real Bitstream. Per-Block costs are retained.
- Not carried:
  - The breakdown of Total encoded size into header, Formula-parameter, model-signalling and payload bytes.
  - The share of Blocks and bytes won by each Series family and each Baseline.
  - The per-class share of zero Residuals.
  - Per-Block model-signalling cost.
- Under AD-7 nothing is transmitted, so the *real* per-Block overhead is the adaptation learning cost plus the rANS flush/termination bytes. The report does not say it measures those, and OQ2's rationale (256-byte Blocks may lose to overhead) hangs on them.
- Fix: AD-14 gets an explicit "report fields" list: the four-way byte breakdown, with rANS termination bytes broken out. It also gets win shares, the zero-Residual share per class, and per-Block adaptation overhead.

### G-4 [MEDIUM] "Retain per-Block cost of every Candidate" is unbounded and unspecified

- Source: PRD FR-14 bullet 4, review §3 and the addendum (retrieval validation: recall and excess bytes, top 16).
- Spine: AD-14, "Per-Block costs of every Candidate are retained".
- Problem: the Codebook cap is 4096 Formulas across 3 lengths and every 256-aligned offset. That is millions of entries per megabyte of input. No storage form (in-memory only, JSON, binary sidecar) or purpose is stated.
- Not carried: the purpose (a later index is validated against the exhaustive best, so the best plus the top-k costs may suffice).
- Fix: specify the retention artifact as a compact sidecar. Either all costs, or the exhaustive best plus the cost vector per (offset, length), named for the recall/excess-bytes validation. Say it is not part of the JSON report.

### G-5 [MEDIUM] AD-15 vs the "publication gate must not block the benchmark" sequencing

- Source: PRD §6.1 ("The publication gate runs in parallel and must not block the benchmark… pays off only if SM-1 is positive"), §5 (no 0.x compatibility promise), SM-5 (owner and timing unset).
- Spine: AD-15 makes any normative change "not done until" golden vectors, the Python reference decoder and the Rust kernel all agree, on x86-64 and aarch64 CI. The Deferred table has probability-model constants "tuned before first spec freeze".
- Problem: during tuning, every model-constant change drags the vectors and the Python decoder with it. That turns the publication gate into a measurement-core dependency, which the PRD explicitly rules out.
- Fix: add a pre-freeze rule to AD-15. Version-bump is mandatory, and vector and refdec regeneration is mandatory at spec freeze, not per tuning change. Make "spec freeze" an explicit milestone before the first public release.

### G-6 [MEDIUM] Evaluator semantics beyond AD-5's headline (SPEC-SEED, but the spine does not say so)

- Source: PRD FR-2 bullet 1, review §4 and the addendum fixed-point list.
- Carried by AD-5: widths (int64), rounding (+half >> Q, floor shift), mathematical modulo (`rem_euclid`), no float, LUT as normative data, Q range and encoding, decoder rejects limit violations.
- Not carried anywhere:
  - Trigonometric argument reduction and LUT interpolation.
  - Recurrence initialization semantics (order 1 or 2).
  - Division semantics. AD-5 mentions none.
  - The coefficient limits themselves. "Family limits are independent" gives no numbers.
  - An overflow proof for the trigonometric and recurrence families. AD-5 gives the bound only for polynomial Horner; trig has LUT interpolation times 2M+1 terms, and recurrence has `m_mod` width.
- Deferred covers "Block header kind codes and field widths" but not these.
- Test gap: Verification lists property tests for AD-3 only. There are no extreme-parameter / limit-boundary vectors for the Evaluator, which FR-2 bullet 3 and SM-5 need.
- Fix: add one Deferred row, "Evaluator detail: argument reduction, interpolation, recurrence init, limits and the per-family overflow budget → spec seed (before the first spec freeze)". Add "limit-boundary Evaluator vectors" to Verification.

### G-7 [MEDIUM] Verification inventory misses SM-2, FR-9 and FR-13 edge cases

- Source: PRD SM-2 (empty, shorter than one Block, partial final Block, incompressible), FR-9 bullet 5, FR-12 bullet 2, FR-13 bullet 2, review §4 (partial-block handling).
- Spine: the Verification row has AD-3 property tests, golden vectors and the reference decoder, a determinism test, and a no-expansion test.
- Not carried:
  - A round-trip suite for: empty, 1 byte, < Block, exactly 256/1024/2048, partial final Block, all-incompressible.
  - Truncation and malformed-stream tests, including bit-flip fuzzing, that assert `Result::Err`, exit 1 and no output file.
  - A trailing-bytes-after-last-Block rejection. AD-8 and AD-9 cover read-past-end and an N mismatch, but not trailing garbage.
  - A selector-minimality property: the chosen Candidate's exact size ≤ every other evaluated Candidate, with Raw winning ties (FR-6).
- Also: AD-12 states temp-file + rename and no output file for `decompress` only. FR-12 bullet 2 requires no valid-looking partial output from `compress` too.
- Fix: extend the Verification row with the above. State in AD-12 (or the Conventions) that the temp-and-rename rule applies to both subcommands. Add "reject trailing bytes" to AD-8.

### G-8 [MEDIUM] Spec seed is the carrier for several items, but it does not exist and its contents are not enumerated

- Source: spine frontmatter `companions`, the Deferred row "Block header kind codes and field widths", PRD FR-9 and FR-11, review §4.
- Items the spine implicitly or explicitly sends to the spec seed:
  - Block header kind codes and field widths.
  - The Raw Block layout. AD-8 says a Block is "header + parameters + own rANS stream", which does not fit a Raw Block. The length is derivable from the 2-bit length code, but that is not written down.
  - Precise rANS state initialization, renormalization order and termination. AD-7 names them as spec-defined.
  - Count-field offset encodings.
  - The exact meaning of "reserved" length code 3.
- Risk: without an obligations list, SPEC-SEED items can silently vanish.
- Fix: add a short "Spec-seed obligations" list to the spine, mapping each of the FR-2, FR-3, FR-8 and FR-9 spec-must-state bullets to the AD that fixes its shape. Or write the seed.

### G-9 [LOW–MEDIUM] Codebook bound is total, not per family; "all three families present" is not stated

- Source: PRD FR-4 bullets 1 and 2 ("size per family has a stated upper bound"), OQ6.
- Spine: AD-13 caps at 4096 Formulas total across all families.
- Fix: state per-family caps (or a rule such as ≥ 1 and ≤ cap per family), and require at least one Formula per family. Otherwise SM-1 could be tested with a Codebook that has no trig or recurrence members.

### G-10 [LOW] Tuning and evaluation isolation is declared, not made checkable

- Source: PRD FR-15 bullet 4, SM-C2, SM-1.
- Spine: AD-13 and AD-14 say the Codebook is tuned only on `tuning` inputs and the verdict reads `evaluation` only. Nothing mechanical backs this: the report does not record the tuning-set identity, and nothing refuses a verdict if the recipe's tuning set overlaps evaluation.
- Fix: record the tuning-set manifest hash and the role breakdown in the report. Have the bench refuse overlap.

### G-11 [LOW] Reproducibility details (SM-4, UJ-3)

- Source: PRD SM-4 ("clean checkout… identical sizes"), UJ-3 (Mei on a different machine).
- Spine: AD-13 covers thread count. The Stack pins versions.
- Not carried:
  - Committing `Cargo.lock` and a `rust-toolchain.toml`.
  - A rule that the Codebook generation recipe uses integer-only arithmetic, or at least is hash-verified across x86-64 and aarch64. The hash makes a divergence detectable but does not say the recipe must be platform-independent.
  - A rule that the benchmark report is timestamp-free, so two clean runs can be compared.

### G-12 [LOW] Deferred table omits several PRD carry-forward constraints

- PRD OQ1 / FR-4 Out-of-Scope: "fully parameterized Formulas, no local fitting" (review §3: formulas vs initializations). Not recorded as a decision or a deferral.
- PRD §5 and review §3: any retrieval index must first be validated against exhaustive search (recall, excess bytes). Deferred says only "after a rate result exists".
- PRD §5, review §3 and the addendum: format-aware preprocessing must be reversible and fully signalled.
- PRD §6.2: the FR-8 causal-model rules apply when context-adaptive rANS arrives.
- PRD §5: no stable library API and no 0.x compatibility promise. AD-15 is the opposite default (always bump and regenerate); this is compatible, but it is nowhere stated that `mbsc-kernel` is not a public API.
- Fix: add those clauses to the existing Deferred rows.

### G-13 [LOW] Wording and consistency nits

- The Glossary says Whole-file raw has "one fixed-size header". AD-11 gives a 3–7 byte header (LEB128 N). The bound N+7 still holds; align the wording ("bounded-size").
- AD-9 claims the DP "gets an optimal partition" but also fixes the last-Block length rule ("smallest nominal length that covers the remainder"). State that the rule applies to the Block that reaches end-of-input, and that the DP is optimal subject to it.
- `compress` of N ≥ 2³⁵ is not defined. AD-11 caps N but does not say the encoder fails with an error.
- Output-file-exists behaviour for temp-and-rename is unspecified.
- The synthetic-signal documented seed (FR-15 bullet 2) and "non-redistributable corpora are referenced, not committed" (FR-15 bullet 3, `.gitignore` for `corpora/`) are implied by `fetch/` and the manifest but never stated.
- The work-model denominator (review §6, 53,248 predicted positions per decision) is carried only by the PRD's "no throughput targets" non-goal. If the benchmark reports encode time, say it names its denominator.
- SM-C3 is not in the spine's `binds` list. AD-10 and AD-2 satisfy it in substance (exact selection, screen off in benchmark).
- The review's prototype order (addendum, steps 1–7) is not an architecture concern. It belongs in epics.

## 3. Coverage matrix

Legend: ✔ carried; ◐ partly carried; ✖ not carried; SS = belongs in spec seed.

### PRD FR consequences

| FR | Consequence | Status | Carrier / note |
| --- | --- | --- | --- |
| FR-1 | all (y, ŷ) round-trip; random-Predictor property test | ✔ | AD-3, Verification |
| FR-2 | spec states widths, rounding, shifts, modulo, no float | ✔ | AD-5 |
| FR-2 | trig reduction, LUT interpolation, recurrence init, coefficient limits | ◐ SS | G-6 |
| FR-2 | vectors pass on x86-64 and aarch64 | ✔ | AD-15 |
| FR-3 | degree vs count, order vs 2M+1, `m_mod`, Q/count ranges | ✔ | AD-6, AD-5 |
| FR-4 | three families in the Codebook | ◐ | AD-13 (no per-family presence rule), G-9 |
| FR-4 | per-family upper bound | ◐ | total 4096 only, G-9 |
| FR-4 | regenerable recipe; version and hash recorded; decoder never reads it | ✔ | AD-13, AD-14, AD-1 |
| FR-5 | deterministic search | ✔ | AD-13 |
| FR-5 | length set {256, 1024, 2048}, signalled; fixed-length runs | ✔ | AD-9 |
| FR-5 | deterministic partition rule stated in the spec | ✔ | AD-9 |
| FR-5 | final partial Block, no padding | ✔ | AD-9 |
| FR-6 | chosen ≤ every other Candidate | ✔ | AD-10 (test is missing, G-7) |
| FR-6 | Raw wins unless strictly smaller, framing both sides | ✔ | AD-10 |
| FR-6 | rate in bits/byte or S/(8N), never mixed | ✔ | Conventions |
| FR-6 | screen off in benchmark | ✔ | AD-10 |
| FR-7 | constant byte per Block; previous-byte first = 0 | ✔ | AD-4 |
| FR-7 | each Baseline forced whole-input and reported | ✖ | G-2 |
| FR-7 | Baselines-only arm selectable | ◐ | AD-14 |
| FR-7 | same model and framing | ✔ | AD-2 |
| FR-8 | model reconstructed, spec says which | ✔ | AD-7 |
| FR-8 | normalization, rescale, rANS init, byte order, termination | ✔ / SS | AD-7 names them; values go to the spec |
| FR-8 | per-Block model-signalling cost measured and reported | ✖ | G-3 |
| FR-8 | adaptation context, update timing, reset | ✔ | AD-7 |
| FR-8 | no interleaved or SIMD rANS | ✔ | AD-7 |
| FR-9 | bit order, endianness, header parser | ✔ / SS | AD-11 (Block header widths: Deferred) |
| FR-9 | Raw flag with unambiguous length | ◐ SS | G-8 |
| FR-9 | payload length, framing, termination normative | ✔ | AD-8 |
| FR-9 | version field | ✔ | AD-11 |
| FR-9 | empty, < 1 Block, exactly 1 Block round-trip | ◐ | design ✔ (AD-11 empty); tests ✖ G-7 |
| FR-10 | N + constant bound, tested on random and compressed | ✔ | AD-10, Verification |
| FR-10 | Whole-file raw mode | ✔ | AD-10, AD-11 |
| FR-11 | spec document, sufficient with vectors | ✔ | `spec/`, AD-15 (the seed is unwritten, G-8) |
| FR-12 | deterministic output | ✔ | AD-13 |
| FR-12 | non-zero exit on I/O error; no partial output | ◐ | AD-12 for `decompress` only; G-7 |
| FR-13 | byte-identical round-trip | ✔ | AD-14, Verification |
| FR-13 | reject truncated or malformed; no OOB; bounded memory | ✔ | AD-12 (64 KiB is stronger than the PRD bound) |
| FR-13 | trailing bytes | ✖ | G-7 |
| FR-14 | per-input totals for each arm; round-trip verified | ◐ | G-2 |
| FR-14 | per-class aggregate; verdict with wins and margin | ◐ | G-1 |
| FR-14 | synthetic separate | ✔ | AD-14 |
| FR-14 | size breakdown; family/Baseline win shares | ✖ | G-3 |
| FR-14 | per-Block cost of every Candidate | ◐ | G-4 |
| FR-14 | zero-Residual share per class | ✖ | G-3 |
| FR-14 | Codebook version, Block lengths, tool version, manifest hash | ✔ | AD-14 |
| FR-15 | manifest fields; ≥ text, PCM, executable plus a compressed control | ✔ | `manifest.toml` fields; corpus choice is Deferred (OQ4) |
| FR-15 | synthetic signals from a documented seed | ◐ | G-13 |
| FR-15 | non-redistributable corpora referenced | ◐ | G-13 |
| FR-15 | tuning/evaluation roles | ✔ / ◐ | AD-13, AD-14; enforcement G-10 |

### Success metrics, counter-metrics, Non-Goals

| Item | Status | Note |
| --- | --- | --- |
| SM-1 | ◐ | Mechanism ✔ (AD-14, the Baselines-only arm, AD-13); verdict logic ✖ G-1, G-2 |
| SM-2 | ◐ | AD-3, AD-12; test inventory G-7 |
| SM-3 | ✔ | AD-10, Verification |
| SM-4 | ◐ | AD-13, AD-14; details G-11 |
| SM-5 | ✔ (exceeds the PRD) | AD-15; sequencing conflict G-5 |
| SM-C1 | ✔ | AD-14 |
| SM-C2 | ◐ | AD-13, AD-14; G-10 |
| SM-C3 | ✔ in substance | AD-10, AD-2, Deferred (OQ8); not in `binds` (G-13) |
| NG: not production / no throughput targets | ✔ | Deferred (OQ8); no targets in the spine |
| NG: no retrieval index | ✔ | Deferred; validation gate G-12 |
| NG: no SIMD / interleaved / context-adaptive | ✔ | AD-7, Deferred |
| NG: no format-aware preprocessing | ◐ | Deferred; reversibility clause G-12 |
| NG: no stable library API / no 0.x compat | ◐ | not stated, G-12 |

### Architecture review (`docs/mbsc_architecture_review.md`)

| Review item | Status | Note |
| --- | --- | --- |
| §1 modular residual; encoder-only search | ✔ | AD-3, AD-1 |
| §2 units (8 vs 1), `S/(8N)`; compare against real raw size with framing; "rate selector" naming | ✔ | Conventions, AD-10, Naming |
| §2 pseudo-entropy not zero-biased; entropy only as a screen | ✔ | AD-10 (the screen is "optional"; the exact formula is not restated, which is fine) |
| §3 raw bytes are not the underlying signal; separate experiments | ◐ | Deferred; reversibility G-12 |
| §3 retrieval validation (recall, excess bytes, top 16) | ◐ | AD-14 retention; purpose and gate G-4, G-12 |
| §3 formulas vs initializations | ◐ | G-12 |
| §4 Q and count mapping; family layouts; raw length; partial block; header parser; model; payload length; bit order | ✔ | AD-5, AD-6, AD-8, AD-9, AD-11; widths and kind codes go to the spec seed |
| §4 fixed-point semantics list | ◐ | G-6 |
| §5 causal model and backward encoding; reset; normalization | ✔ | AD-7, AD-8 |
| §5 interleaved rANS byte order | n/a | not used in v1 |
| §6 terminology (candidate-retrieval index) | ✔ | Deferred uses "retrieval index" |
| §6 work-model denominator | ◐ | G-13 |
| §6 65% zeros is a hypothesis | ✔ (not a requirement) / ✖ (measured) | G-3 |
| §7 prototype order | n/a | epics |

### Brief addendum

Every addendum section maps to the same items as the review above: rate and selector ✔, representation experiments ◐ (G-12), retrieval and performance ◐ (G-4, G-12, G-13), Bitstream contract gaps ✔ with the SS items in G-6 and G-8, and context-adaptive rANS ✔. The addendum adds no requirement beyond the review and the PRD.

## 4. Items that look like gaps but are not

- Memory bound: the PRD asks for "a stated multiple of Block length plus declared output size". AD-12 gives about 64 KiB independent of N, which is stronger.
- Integrity checksum: deferred (OQ7), the version field allows adding it, and the PRD agrees.
- Block header kind codes and field widths: Deferred, SS, and the spine says why they are not a divergence point.
- Probability-model constants: Deferred, with the structure fixed by AD-7.
- Corpus choice (OQ4): Deferred, with the mechanism fixed.
- SM-5 owner and timing: the spine resolves this with CI (AD-15). See G-5 for the sequencing side effect.
