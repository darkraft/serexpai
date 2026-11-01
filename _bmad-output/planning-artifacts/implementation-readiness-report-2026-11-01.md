---
stepsCompleted:
  - step-01-document-discovery
  - step-02-prd-analysis
  - step-03-epic-coverage-validation
  - step-04-ux-alignment
  - step-05-epic-quality-review
  - step-06-final-assessment
date: 2026-11-01
project: SerExpAI
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/prd.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/addendum.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/reconcile-architecture-review.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/reconcile-brief.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/reconcile-spec.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/review-rubric.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/review-rubric-2026-10-31.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/epics.md
uxDocument: missing
excludedDocuments:
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/prd.before.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/.memlog.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/walkthrough.html
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/.memlog.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/reviews/review-adversary.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/reviews/review-reconcile.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/reviews/review-rubric.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/reviews/review-versions.md
  - docs/mbsc_architecture_review.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-11-01
**Project:** SerExpAI

## Document Inventory

### PRD

- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/prd.md`
- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/addendum.md`
- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/reconcile-architecture-review.md`
- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/reconcile-brief.md`
- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/reconcile-spec.md`
- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/review-rubric.md`
- `_bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/review-rubric-2026-10-31.md`

### Architecture

- `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md`

### Epics and Stories

- `_bmad-output/planning-artifacts/epics.md`

### UX

No UX document. The assessment records this gap and continues.

### Left out of the assessment

- `prd.before.md` and the PRD `.memlog.md`
- Architecture companions: `bitstream-spec-seed.md`, `walkthrough.html`, `.memlog.md`, and the `reviews/` folder
- `docs/mbsc_architecture_review.md`

## PRD Analysis

Source of the numbered requirements: `prds/prd-serexpai-2026-10-30/prd.md` (status final, updated 2026-10-31). Identifiers below keep the PRD's own `FR-n` form. Requirement text includes each FR's statement and its testable consequences. The addendum, reconciliation notes, and quality reviews were read in full. They do not add a second numbered FR list. The reconciliation notes and the 2026-10-31 quality review describe earlier drafts; the current `prd.md` already contains the fixes those notes asked for. They are traceability records.

### Functional Requirements

FR-1: Modular residual reconstruction. The encoder computes Residuals and the decoder reconstructs bytes with modular arithmetic for any Predictor. Realizes UJ-2. Consequences (testable): For every byte value y and every predicted value ŷ in 0..255, decoding the Residual yields y. A property test over random Blocks and random Predictors round-trips with zero mismatches.

FR-2: Normative Evaluator semantics. The Normative Evaluator is specified exactly enough that two independent implementations produce identical ŷ for any valid parameters. Realizes UJ-3. Consequences (testable): The spec states: intermediate integer widths, overflow behaviour, rounding (including negative halfway cases), division and shift semantics, trigonometric argument reduction and lookup-table interpolation, coefficient limits, recurrence initialization, and mathematical (non-negative) modulo for negative inputs. The decoder uses no floating point. A published test-vector set of (parameters, Block length, ŷ sequence) passes on every supported platform: Linux on x86-64 and aarch64. A decoder rejects any parameter outside its limits rather than clamping it; each limit has a boundary vector and a rejection vector.

FR-3: Family-specific parameter layouts. Each Series family has its own parameter layout. Realizes UJ-3. Consequences (testable): Polynomial degree is stored separately from its parameter count (degree + 1 coefficients). Trigonometric order is stored separately from its 2·order + 1 coefficients, with independent limits. The recurrence modulus has its own name (`m_mod`) and is never confused with a degree or order. Every field covers the v1 ranges (Q in [8, 24] and counts 1 to 16) through a defined encoding (AC CAP-2).

FR-4: Codebook. A Codebook of fully parameterized Formulas from all three Series families is defined and versioned, small enough to search exhaustively. Realizes UJ-1. Consequences (testable): The Codebook contains Formulas from polynomial, trigonometric and recurrence families. The Codebook holds at most 4096 Formulas, at most 2048 per family, so benchmark encoding fits the time budget of FR-14 (assumption: encode cost is a work-model estimate until the first measurement). The Codebook is reproducible from a documented recipe, so a reviewer can regenerate it. Codebook identity and version are recorded in the benchmark report; the decoder never reads the Codebook. Out of scope: local parameter fitting from candidate initializations (assumption: v1 uses fully parameterized Formulas; see Open Question 1).

FR-5: Exhaustive per-Block search. For each Block the encoder evaluates every Codebook Formula and every Baseline. Realizes UJ-1. Consequences (testable): The search result for a Block is deterministic for a given input and Codebook, and identical whatever the thread count; a test compares a parallel run with a single-threaded one. Block length is one of 256, 1024 or 2048 bytes, signalled per Block in the Bitstream, and the Rate selector chooses among them. The rule that partitions an input into Blocks of mixed lengths is deterministic, selects on exact size, and is stated in the spec (AC CAP-3). The benchmark can also run each fixed Block length alone. Each Block decodes from its own header and payload alone: Formulas use Block-local positions, and no model or coder state carries across a Block boundary. The final partial Block is handled without padding the output beyond its signalled length.

FR-6: Rate selector on actual size. The Rate selector minimizes Total encoded size; the exact rules are in AC CAP-4. Realizes UJ-1. Consequences (testable): For each Block, the chosen Candidate's actual encoded size is less than or equal to that of every other evaluated Candidate covering the same span; the partition rule (FR-5) decides which lengths are compared. A Raw Block wins whenever compressed size is not strictly smaller than the true Raw Block size, framing included on both sides. On equal size, a Baseline beats a Formula, and Whole-file raw mode beats everything, including for the empty input. Gain is therefore never awarded to a Formula on a tie. Rate is reported as bits per input byte (raw threshold 8), or as the ratio `S_total / (8N)`; the two are never mixed. v1 has no entropy screen or other heuristic that can change which Candidate wins, so the guarantee above covers all Candidates and `compress` produces the same bytes as the full-MBSC benchmark arm. Speed-ups that provably cannot change the selection are allowed.

FR-7: Baseline Predictors and direct entropy coding. The encoder supports constant prediction, previous-byte prediction and direct byte entropy coding as Candidates. Realizes UJ-1. Consequences (testable): Constant prediction uses a per-Block constant byte carried in the Bitstream. Previous-byte prediction predicts the preceding byte, and the first byte of a Block is predicted as 0. Each Baseline can be forced on for a whole input through `benchmark`, and its Total encoded size reported on its own. A forced-Baseline arm still uses Raw Blocks, the same partition rule and the Whole-file raw check; only its Candidate set differs. The Baselines-only arm (Glossary) can be selected through `benchmark`. Baselines use the same Probability model and framing as Formulas.

FR-8: Single specified Probability model and binary range coder. Residuals (and bytes under direct entropy coding) are coded with one Probability model and a binary range coder, specified completely. The model's update rule, the coder's mapping and the decoder's per-decision order are in the spec (AC CAP-6 and the patent safeguards in SPEC Constraints). Realizes UJ-3. Consequences (testable): The Probability model is reconstructed deterministically from already-decoded symbols; nothing about it is transmitted. The model is adaptive order-0, reset at every Block start. The spec states the model's update rule, how probabilities map to coder frequencies, and the coder's initialization, renormalization byte order and termination. Model constants are tuned on tuning inputs only and scored on the Baselines-only arm, so they cannot favour Formula residuals. Formula coefficient and frequency widths are scored on full MBSC. Ties go to the simpler setting, and the tuning grid and every score are recorded. The per-Block fixed cost (header and coder flush) and the model's adaptation overhead are measured and reported, because at 256-byte Blocks they can exceed any Formula gain. A decoder never selects a model from the value of the symbol it is decoding. Interleaved or SIMD entropy coding is not used in v1. rANS is not used in v1. Adopting it later would need a freedom-to-operate review of Microsoft US 11,234,023 B2 and a format version bump; v1's decoder structure follows the patent safeguards in the spec. Changing the decoder's per-decision structure, or adding a field for coder-state retention, symbol width or probability tables, needs a new freedom-to-operate review first.

FR-9: Complete Bitstream contract. The Bitstream has a normative layout, defined in the spec seed and checked by AC CAP-7. Realizes UJ-3. Consequences (testable): The spec defines bit order within bytes and endianness of multibyte integers, and a header parser (bit-level or byte-level, stated explicitly). A raw flag is accompanied by an unambiguous Raw Block length. Payload length, framing and termination are normative and stated once. A Bitstream carries a format version. Any change to the Normative Evaluator, normative tables, model constants, coder, checksum or layout bumps the format version and regenerates the test vectors in the same change. Every Bitstream ends with a CRC-32 of the decoded bytes, and a decoder rejects a mismatch, so corruption that framing cannot see (Formula parameters, kind codes, Raw Blocks) is detected. The empty input, an input shorter than one Block and an input of exactly one Block all round-trip.

FR-10: Bounded raw fallback. MBSC never expands an input beyond a defined, documented overhead. Realizes UJ-2. Consequences (testable): The Bitstream is at most N + 11 bytes for format versions 1 to 14 (15 is reserved), and a test asserts it on random and already-compressed inputs. A Whole-file raw mode makes the bound a small constant, not a per-Block cost.

FR-11: Bitstream specification document. The repository includes a standalone document describing the Bitstream and the Normative Evaluator. Realizes UJ-3. Consequences (testable): The document is sufficient, with the test vectors, to write an independent decoder. The repository includes a Python reference decoder (standard library only) written from the document, not translated from the Rust, and a generator of valid Bitstreams that also produces mutated copies of them for rejection testing. Every reject rule and limit is covered by at least one vector. The Rust decoding kernel (`sxp-kernel`) and the Python decoder agree on every vector in CI on both platforms. Before spec freeze this check does not block the benchmark; it must pass at spec freeze and before any public release. The document is a v1 deliverable, so the audience goal is not closed off. It is published under CC BY 4.0, and the test vectors and normative tables under CC0.

FR-12: `compress`. The user can compress a file to a Bitstream file. Realizes UJ-2. Consequences (testable): Output is deterministic: the same input and Codebook yield the same Bitstream. This is a v1 choice for reproducibility; the format does not require different encoders to agree, only decoders (the Normative Evaluator). The command exits non-zero with a message on any I/O error and writes no partial output file as if it were valid. The command refuses inputs of 2³⁵ bytes or more with a non-zero exit.

FR-13: `decompress`. The user can decompress a Bitstream file to the original bytes. Realizes UJ-2. Consequences (testable): Round-trip output equals the original file byte for byte for every Input class. A truncated, corrupted or malformed Bitstream is rejected with a non-zero exit and a message, and leaves no output file. Detection combines framing checks with the CRC of FR-9. The decoder never reads out of bounds, and its heap working set is at most 64 KiB, independent of N.

FR-14: `benchmark`. The user can run the benchmark over the Corpus manifest and get a per-class report. Realizes UJ-1. Consequences (testable): For each input the benchmark reports Total encoded size for each whole-input Baseline, for the Baselines-only arm, for full MBSC and for Whole-file raw mode, and verifies the decoded bytes equal the input; a mismatch fails the run. Results are aggregated per Input class (Total encoded size summed over the class's inputs; assumption: summing weights inputs by size; equal-sized slices in corpora.md keep weights equal for text and executables). The verdict line states, per Real input class, whether Gain was achieved, and reports per-input wins and the margin. The overall verdict is positive, marginal, inconclusive or negative, as defined by SM-1. Synthetic signals are reported in their own section, never merged into a Real input class aggregate. The report breaks Total encoded size into header, Formula parameter, coder payload and coder flush bytes, reports the model's adaptation overhead separately, and gives the share of Blocks and bytes won by each Series family and each Baseline. The benchmark writes a sidecar file with the encoded cost of every evaluated Candidate per Block, so a later retrieval index can be validated against the exhaustive best (recall and excess compressed bytes). It is written by default for inputs up to a size stated in the spec, and on request above it, and the report states which inputs have one. The report records the share of zero Residuals per class, so the review's claim that about 65% of residuals are zero on PCM audio and text is measured, not assumed. The report records Codebook version and hash, Block lengths, tool version, Corpus manifest hash, tuning-input hashes and the overlap checker's result (FR-15). The report gives a deterministic Work count per arm (model decisions and coder steps per input byte). Wall-clock time and host details go to a separate timing log, kept out of the report so reruns stay identical. A full benchmark run completes within 24 hours on one host, named in the timing log before the first full run. If it does not, the corpus shrinks first (keeping the FR-15 minimum per class), then the Codebook cap is lowered, and the report states the excess bytes this costs on tuning inputs.

FR-15: Corpus manifest and synthetic signals. The benchmark inputs are defined and reproducible. Realizes UJ-3. Consequences (testable): The Corpus manifest names text, PCM and executable inputs, plus already-compressed data as a control, each with source, licence and content hash. The Real input classes are text, 8-bit PCM and executables, with tuning and evaluation inputs drawn from different sources; the roster is fixed in the spec companion `corpora.md`. Each Real input class has at least three evaluation inputs (size range in `corpora.md`); a class with fewer cannot be tested for SM-1. The report notes known correlation between evaluation inputs of a class. Any Corpus-side transform (16-bit to 8-bit PCM, slicing a large file) is documented and hashed in the manifest. Raw 16-bit PCM is reported beside the derived class but does not count toward SM-1. Synthetic signals matching each Series family are generated from a documented seed. Corpora that cannot be redistributed are referenced by hash and source, not committed. The Corpus manifest marks each input as tuning or evaluation. Codebook tuning uses only tuning inputs, and SM-1 is decided on evaluation inputs only. A checker proves that no tuning input shares a run of K bytes or more with any evaluation input (K proposed as 1 KiB; Open Question 3). A failure blocks the benchmark until the tuning input is replaced; evaluation inputs never change.

Total FRs: 15

### Non-Functional Requirements

The PRD has no numbered NFR section. The 2026-10-31 quality review records that, and the current PRD keeps every numeric quality bound inside an FR, a success metric, or a non-goal. The items below are those bounds, quoted so coverage validation can see them. Each one traces to the FR or SM named on it. They are not a second requirement set.

NFR-1 (performance, from FR-14 and FR-4): A full benchmark run completes within 24 hours on one host, named in the timing log before the first full run. If it does not, the corpus shrinks first (keeping the FR-15 minimum per class), then the Codebook cap is lowered, and the report states the excess bytes this costs on tuning inputs. The Codebook holds at most 4096 Formulas, at most 2048 per family, so benchmark encoding fits that budget. Encode cost is a work-model estimate until the first measurement.

NFR-2 (performance stance, from §5 and SM-C3): v1 does not set throughput targets. FR-14's Work count names the denominator; a target can follow once one is needed. Do not trade rate-measurement fidelity (exhaustive search, actual-size selection) for speed in v1, even when the FR-14 time budget is tight.

NFR-3 (reliability, from FR-9, FR-10, FR-13, and §5): Every Bitstream ends with a CRC-32 of the decoded bytes, and a decoder rejects a mismatch. The Bitstream is at most N + 11 bytes for format versions 1 to 14 (15 is reserved). A truncated, corrupted, or malformed Bitstream is rejected with a non-zero exit and a message, and leaves no output file. The decoder never reads out of bounds, and its heap working set is at most 64 KiB, independent of N. v1 has no per-Block checksums and no cryptographic integrity; the whole-file CRC detects accidental corruption only.

NFR-4 (portability, from FR-2 and SM-4): Evaluator test vectors pass on Linux on x86-64 and aarch64. Rerunning the benchmark from the Corpus manifest on a clean checkout, on any supported platform and thread count, yields identical Total encoded sizes. Search results are identical whatever the thread count.

NFR-5 (integrity of outputs, from FR-12): `compress` exits non-zero with a message on any I/O error and writes no partial output file as if it were valid. It refuses inputs of 2³⁵ bytes or more with a non-zero exit. The same input and Codebook yield the same Bitstream.

NFR-6 (legal and licensing, from §6.1, FR-8, and FR-11): Code and the Python reference decoder are under MIT OR Apache-2.0. The spec document is under CC BY 4.0. Test vectors and normative tables are under CC0. Contributions are by DCO sign-off with no CLA, and a human who has read, verified, and decided on every AI-assisted change. rANS is not used in v1; adopting it later needs a freedom-to-operate review of Microsoft US 11,234,023 B2 and a format version bump. Changing the decoder's per-decision structure, or adding a field for coder-state retention, symbol width, or probability tables, needs a new freedom-to-operate review first. Open Question 6 requires counsel to review the adaptive probability update against Qualcomm US 10,554,988 and US 10,148,961 and Tencent US 12,348,780 before the first public release.

NFR-7 (usability of the v1 interface, from §4.6 and §5): The CLI is `compress`, `decompress`, and `benchmark` over file arguments. There is no stdin/stdout streaming in v1. v1 is not a library with a stable API and promises no Bitstream compatibility between 0.x versions. The format-version field lets a decoder reject unknown versions.

Total NFRs: 7, all embedded in the FRs, success metrics, and non-goals above. The PRD does not number them separately.

### Additional Requirements

**User journeys**

- UJ-1. Darian runs the experiment. Darian, the author, points the benchmark at the corpora, gets a per-class table of total encoded size for every baseline and for MBSC, and can see in one read whether MBSC won on at least two real input classes.
- UJ-2. Darian round-trips a file. Darian compresses a file with `compress`, decompresses the bitstream with `decompress`, and confirms the output is byte-identical, including when the file is empty, tiny, or already compressed.
- UJ-3. Mei reproduces the result. Assumption: an outside reviewer will attempt reproduction. Mei, a compression researcher, reads the bitstream spec and the corpus manifest, reruns the benchmark on their machine, and gets the same sizes, so the claim stands or falls on evidence.

**Success metrics (each names the FRs it validates)**

- SM-1: Research answer. On evaluation inputs, full MBSC achieves Gain (any reduction in Total encoded size) over the Baselines-only arm on at least two distinct Real input classes, so a reduction that comes only from switching among Baselines does not count. The best whole-input Baseline is reported alongside as a consistency check. If fewer than two Real input classes can be tested, the result is inconclusive, not negative; otherwise, fewer than two classes with Gain is negative. Any reduction answers the research question, but going ahead with the growth path in §1 also needs, in each of the two classes, a Gain of at least a minimum share of the class's Total encoded size and wins on a majority of the class's evaluation inputs; a positive result below that is reported as marginal. Darian sets both numbers before the first evaluation run. What a win on exactly one Real input class means is Open Question 7. Validates FR-5, FR-6, FR-7, FR-14, FR-15.
- SM-2: Lossless round-trip. 100% of benchmark inputs and test inputs (including empty, shorter than one Block, partial final Block, and incompressible) decode byte-identically. Validates FR-1, FR-2, FR-9, FR-13, FR-14.
- SM-3: No regression on incompressible data. On already-compressed inputs, Total encoded size does not exceed the N + 11 bound of FR-10. Validates FR-6, FR-10.
- SM-4: Reproducibility. Rerunning the benchmark from the Corpus manifest on a clean checkout, on any supported platform and thread count, yields identical Total encoded sizes. Validates FR-12, FR-14, FR-15.
- SM-5: Spec sufficiency. A clean-room decoder, written only from the spec document, normative tables, and test vectors, decodes generated valid Bitstreams and rejects the rejection vectors. Someone other than the author writes it (or a fresh, sandboxed AI agent from a model family other than the one that helped write the kernel), in a language other than Rust or Python, and logs every ambiguity as a spec issue. It runs at spec freeze, before the first public release, and on every format version bump. The author's own Python reference decoder does not satisfy SM-5 on its own. Validates FR-2, FR-9, FR-11.
- SM-C1: Synthetic-signal compression is a counter-metric. Large Gain on synthetic signals proves only that the encoder can find a matching Formula; it is not evidence for the research answer. Counterbalances SM-1.
- SM-C2: Codebook fit to the benchmark corpora is a counter-metric. Tuning the Codebook or the model on the same inputs that decide SM-1 inflates the result. Tuning uses tuning inputs only, content-disjoint from evaluation inputs (FR-15), and model constants are scored on the Baselines-only arm (FR-8). Counterbalances SM-1.
- SM-C3: Encoder speed is a counter-metric. Do not trade rate-measurement fidelity for speed in v1, even when the FR-14 time budget is tight. Counterbalances SM-1 and the FR-14 budget.

**Non-goals (§5)**

- v1 is not a production compressor and is not competing on throughput or ratio against zstd, xz, or similar.
- v1 does not build a candidate-retrieval index, whether VP-tree, ANN, or LSH. Any later index must first be validated against exhaustive search, measuring candidate recall and excess compressed bytes.
- v1 does not do SIMD or interleaved entropy coding, or context-adaptive modelling beyond the one simple Probability model.
- v1 does not do format-aware preprocessing (parsed WAV channels, reversibly separated byte lanes, symbolic text). Each is a separate later experiment and any such preprocessing must be reversible and fully signalled. Corpus-side transforms that define a benchmark input are not preprocessing.
- v1 does not set throughput targets.
- v1 is not a library with a stable API, and promises no Bitstream compatibility between 0.x versions.
- v1 has no per-Block checksums and no cryptographic integrity.

**MVP split (§6)**

- Measurement core: FR-1 to FR-10 and FR-12 to FR-15.
- Publication gate: FR-11 and SM-5. It runs in parallel and must not block the benchmark. It is mandatory before any public release, and its effort pays off only if SM-1 is positive.
- Also in scope: the licence and contribution model in §6.1.
- Out of scope for MVP: candidate-retrieval index, SIMD and interleaved entropy coding, context-adaptive modelling, format-aware preprocessing, rANS (informative appendix of the spec seed only), and stdin/stdout streaming.

**Assumptions (§9)**

- The eventual open-source audience is large.
- Shipping a CLI in v1 does not close off embedding as a library later.
- A reviewer (Mei) reproduces the result from the spec and manifest.
- v1 Codebook holds fully parameterized Formulas; no local fitting.
- Encode cost fits the FR-14 budget (an estimate until the first measurement).
- Per-class aggregation is Total encoded size summed over the class's inputs.

**Open questions still open in the current PRD (§8)**

1. Deferred, not blocking v1: lookup of fully parameterized Formulas versus local fitting. v1 uses fully parameterized Formulas (FR-4).
2. Which values win on tuning inputs for the model's count cap, rate schedule, and initial probability, and for the coefficient widths and trigonometric frequency width? Owner: Darian, before spec freeze.
3. Is K = 1 KiB the right threshold for the tuning/evaluation overlap checker (FR-15)? Owner: Darian, before the first benchmark run.
4. What is the Codebook's composition within the cap (Formulas per family, Q values, parameter ranges)? The recipe is still to be written. Owner: Darian, before the first benchmark run.
5. Do the LJ Speech section prefixes run LJ001 to LJ050? Owner: Darian, before the first benchmark run.
6. Legal review of the adaptive probability update before the first public release. Owner: Darian, before the first public release.
7. What a win on exactly one Real input class means. SM-1 calls it negative; the question of whether a PCM-only win should open the deferred PCM experiments is still open. Owner: Darian, before the first evaluation run.
8. SM-1's go-threshold numbers: the minimum Gain share of a class's Total encoded size, and the majority of evaluation inputs that must win. Owner: Darian, before the first evaluation run.

**Addendum inferences (research expectations, not FRs)**

- Text bytes are not smooth, so a polynomial or sinusoid predicting `y mod 256` will probably lose to order-0 or previous-byte baselines on text.
- 16-bit PCM read as raw bytes interleaves a noisy low byte and a smooth high byte. PCM is a likely win only for 8-bit PCM or after a reversible byte-lane split, and byte-lane separation is out of scope for v1.
- Report a class with no gain honestly. SM-1 needs two real classes.
- Do not vendor corpora; ship download-and-verify scripts with checksums.
- Comparable-tool conventions (magic numbers, reserved bits, checksums, exit codes) informed FR-9 and the licence choice. They are not extra product requirements.

**Constraints the PRD delegates outside this file set**

The PRD states that the canonical technical contract is `_bmad-output/specs/spec-serexpai/` (`SPEC.md`, `acceptance-criteria.md`, `corpora.md`) and that the bitstream spec seed holds the normative Bitstream draft. Those files were left out of the confirmed inventory. Many FR consequences say the exact rule "is stated in the spec" (FR-3 encoding, FR-5 partition rule, FR-6 selector rules, FR-8 model and coder, FR-9 layout, FR-14 sidecar size limit, FR-15 roster). Coverage validation of the PRD text can proceed. Validation of those delegated details needs the spec, which this assessment is not reading.

### PRD Completeness Assessment

The current PRD is complete enough to trace: 15 globally numbered FRs, each with testable consequences and a user journey, a glossary, an explicit non-goal list, a measurement-core versus publication-gate split, success metrics that name the FRs they validate, an assumptions index, and open questions with owners and deadlines.

Clarity is high on the experiment itself. Gain is defined against the Baselines-only arm, ties cannot award Gain to a Formula, numeric bounds exist (N + 11, 64 KiB, 4096 Formulas, 24 hours, 2³⁵-byte refusal), and the earlier contradictions called out by the reconciliation notes (model signalling, unbounded sidecar, missing tie-break, unset expansion bound, rANS-era pointers) are resolved in the current `prd.md` text.

What is still incomplete for implementation, inside the PRD's own open questions:

- The Codebook recipe (Open Question 4) and the model constants (Open Question 2) are unset. FR-4 and FR-8 cannot be implemented as a specific codebook or model until those exist. Both are owned by Darian and timed before the first benchmark run or spec freeze.
- K (Open Question 3) and the LJ Speech prefix check (Open Question 5) are unset before the first benchmark run.
- SM-1's go-threshold numbers (Open Question 8) and the meaning of a one-class win (Open Question 7) are unset before the first evaluation run. The research answer itself is defined; the growth-path decision is not.
- Legal review (Open Question 6) is required before public release and does not block the measurement core.

The PRD is a requirements document that points at a spec for normative detail. That delegation is explicit. It is a coverage boundary for this assessment, not a hole inside `prd.md`.

## Epic Coverage Validation

Source: `epics.md`, FR Coverage Map and the stories under Epic 1 and Epic 2. The epics inventory restates FR-1 through FR-15. It adds no FR number that is absent from the PRD.

### Coverage Matrix

| FR Number | PRD Requirement | Epic Coverage | Status |
| --------- | --------------- | ------------- | ------ |
| FR-1 | Modular residual reconstruction. The encoder computes Residuals and the decoder reconstructs bytes with modular arithmetic for any Predictor. | Epic 1, Story 1.2 | ✓ Covered |
| FR-2 | Normative Evaluator semantics. The Normative Evaluator is specified exactly enough that two independent implementations produce identical ŷ for any valid parameters. | Epic 1, Stories 1.3, 1.4, 1.5 | ✓ Covered |
| FR-3 | Family-specific parameter layouts. Each Series family has its own parameter layout. | Epic 1, Story 1.10 (group B), with the layouts exercised by Stories 1.3, 1.4, 1.5 | ✓ Covered |
| FR-4 | Codebook. A Codebook of fully parameterized Formulas from all three Series families is defined and versioned, small enough to search exhaustively. | Epic 1, Story 1.10 (group A). Amendment D-09 keeps the production recipe as an owner decision before reportable evaluation. | ✓ Covered |
| FR-5 | Exhaustive per-Block search. For each Block the encoder evaluates every Codebook Formula and every Baseline. | Epic 1, Stories 1.8, 1.9, and 1.10 (group C) | ✓ Covered |
| FR-6 | Rate selector on actual size. The Rate selector minimizes Total encoded size. | Epic 1, Story 1.9, with the Formula-versus-Baseline tie in Story 1.10 and the full-MBSC path in Story 1.11 | ✓ Covered |
| FR-7 | Baseline Predictors and direct entropy coding. The encoder supports constant prediction, previous-byte prediction, and direct byte entropy coding as Candidates. | Epic 1, Stories 1.8, 1.9, and 1.13 (group A) | ✓ Covered |
| FR-8 | Single specified Probability model and binary range coder. Residuals and direct-coded bytes are coded with one Probability model and a binary range coder, specified completely. | Epic 1, Stories 1.6 and 1.7, with tuning evidence in Story 1.13 (group C). rANS stays out of the v1 format in Story 2.1. | ✓ Covered |
| FR-9 | Complete Bitstream contract. The Bitstream has a normative layout, defined in the spec seed and checked by AC CAP-7. | Epic 1, Stories 1.1 and 1.8. The standalone statement of that layout is Epic 2, Story 2.1. | ✓ Covered |
| FR-10 | Bounded raw fallback. MBSC never expands an input beyond a defined, documented overhead. | Epic 1, Stories 1.1 and 1.11 | ✓ Covered |
| FR-11 | Bitstream specification document. The repository includes a standalone document describing the Bitstream and the Normative Evaluator. | Epic 2, Stories 2.1, 2.2, and 2.4 | ✓ Covered |
| FR-12 | `compress`. The user can compress a file to a Bitstream file. | Epic 1, Story 1.11 | ✓ Covered |
| FR-13 | `decompress`. The user can decompress a Bitstream file to the original bytes. | Epic 1, Story 1.11 | ✓ Covered |
| FR-14 | `benchmark`. The user can run the benchmark over the Corpus manifest and get a per-class report. | Epic 1, Story 1.13 | ✓ Covered |
| FR-15 | Corpus manifest and synthetic signals. The benchmark inputs are defined and reproducible. | Epic 1, Story 1.12 | ✓ Covered |

### Missing Requirements

No PRD functional requirement is absent from the epics.

### Critical Missing FRs

None.

### High Priority Missing FRs

None.

### FRs in epics but not in the PRD

None. The epics document's functional-requirement inventory is FR-1 through FR-15, the same set as the PRD.

### Coverage Statistics

- Total PRD FRs: 15
- FRs covered in epics: 15
- Coverage percentage: 100%

## UX Alignment Assessment

### UX Document Status

Not found. Searched `{planning_artifacts}/*ux*.md`, `{planning_artifacts}/*ux*/index.md`, and UI-style filenames under the planning artifacts. Nothing matched. The epics document records the same fact: no UX design contract is an input.

### Alignment Issues

No UX document exists to align against the PRD or the architecture.

The user-facing surface that does exist is the v1 CLI. The PRD specifies it in §4.6 and FR-12, FR-13, and FR-14: `compress`, `decompress`, and `benchmark` over file arguments, deterministic output, non-zero exit with a message, no partial output left behind, and no stdin/stdout streaming. The architecture spine accounts for that surface: `sxp-cli` with clap 4.6.7, exit 0 on success and 1 on any error with a message on stderr, no logging framework and no config file, the benchmark report on stdout and in a JSON file, and deployment as a local CLI with no service.

### Warnings

A separate UX document is missing. The implied interface is a command-line tool for the author, not a web or mobile UI. The PRD and the architecture both specify that CLI, so the gap does not leave an undesigned screen. It does leave visual design, interaction flows beyond the three subcommands, and accessibility out of this assessment. That matches the earlier decision to continue and record the missing UX document.

## Epic Quality Review

Reviewed `epics.md` against the create-epics-and-stories checks: user value, epic independence, forward dependencies, story size, acceptance criteria, starter-template and greenfield setup, and database timing. There is no database in this product, so the table-creation check does not apply.

### Best Practices Compliance Checklist

**Epic 1 — Measure whether formulas beat baselines**

- [x] Epic delivers user value. The author gets a lossless CLI and an auditable answer to whether formulas beat baselines.
- [x] Epic can function independently for a private benchmark. Clean-room review and counsel clearance are explicitly not required (E1-AC08).
- [ ] Stories appropriately sized. Story 1.13 bundles four acceptance groups that could each stand as a reviewable story.
- [x] No forward story dependency that blocks an earlier story from completing. The Story 2.1 coupling is governed by the 2026-11-01 amendment; see Major Issues.
- [x] Database tables created when needed. Not applicable.
- [x] Clear acceptance criteria. They are specific and testable. They are not written as Given/When/Then.
- [x] Traceability to FRs maintained. FR-1 through FR-10 and FR-12 through FR-15.

**Epic 2 — Publish a bitstream someone else can implement**

- [x] Epic delivers user value. An independent implementer gets a standalone contract and a clean-room check.
- [x] Epic can function using Epic 1 output. It does not require a later epic.
- [ ] Stories appropriately sized. Story 2.1 splits a draft contract from a frozen spec inside one story, and Story 2.4 is a full clean-room program.
- [x] No forward dependency inside the epic that prevents an earlier story from completing. Story 2.3's release check is testable with missing-evidence fixtures before Story 2.4 passes.
- [x] Database tables created when needed. Not applicable.
- [x] Clear acceptance criteria, with the same Given/When/Then gap as Epic 1.
- [x] Traceability to FRs maintained. FR-11.

### 🔴 Critical Violations

None.

Neither epic is a technical milestone. Epic 1's outcome is the author's measurement. Epic 2's outcome is a bitstream someone else can implement. Story 1.1 is completable on its own: whole-file raw round-trip, including the empty stream, does not wait on Story 1.8. Later stories consume earlier output (1.7 uses 1.6, 1.8 uses 1.1 and 1.6 and 1.7, 1.9 uses 1.8, 1.10 uses 1.3–1.5 and 1.9, 1.11 uses the encode path, 1.13 uses 1.9–1.12, 2.2 and 2.4 use 2.1). That direction matches the rule.

### 🟠 Major Issues

1. **Epic 1 is still worded as depending on Epic 2 Story 2.1 group A.** S2.1-AC05 says completing group A is an Epic 1 dependency, and delivery checklist item 1 says to complete that group before kernel paths. Epic N is not supposed to need Epic N+1. The 2026-11-01 amendment governs the conflict: `bitstream-spec-seed.md` is the G0 draft, and kernel work on a clause that is not tagged PROPOSED or MEASURE does not wait for a second document. Private benchmarking does not wait for Epic 2. **Remediation:** rewrite checklist item 1 and S2.1-AC05 so the story text matches the amendment. Leave tagged clauses under D-01, D-02, and D-07.

2. **Story 1.13 is one user outcome packed into four independently reviewable groups.** Group A is the arms, group B is the report and sidecars, group C is the freeze and verdict, and group D is the 24-hour budget. The parent story completes only when every group passes, so a failure in the budget protocol keeps the whole "report whether formulas beat baselines" story open. **Remediation:** keep the user outcome as the epic acceptance criterion, and split the four groups into stories if the backlog needs items that can be finished one at a time. Story 1.12 (fetch, overlap, synthetics) and Story 2.1 (draft contract versus frozen spec) have the same shape at a smaller scale.

3. **Affected stories are not acceptance-ready while D-01 through D-09 are proposals.** The revision says the proposed dispositions are not approvals. D-01 blocks family serialization and production Codebook acceptance. D-02 blocks block writer and parser implementation. D-03 blocks partition acceptance. D-04, D-05, D-06, D-07, and D-09 block reportable evaluation. D-08 blocks public release. Stories must not invent those answers. **Remediation:** implement only the paths whose decisions are already approved. Do not treat a proposed disposition as the contract.

### 🟡 Minor Concerns

1. **Acceptance criteria are not Given/When/Then.** They are numbered, testable statements (for example S1.2-AC01 lists the 65,536 pairs, the residual formula, and the reconstruction). Error cases and measurable outcomes are present. The gap is format, not vagueness. **Remediation:** optional. Rewrite only if a later workflow requires BDD phrasing.

2. **Stories 1.2 through 1.7 are kernel slices.** Each is phrased as the author wanting a verified property (exact reconstruction, one family of predictions, the coder, the model). Each can be completed without a later story. The author gets no new command until Story 1.8, and no file commands until Story 1.11. **Remediation:** leave them if the author-as-verifier is the accepted user. Do not promote them into their own epic.

3. **S1.8-AC02 mentions "later Formula kinds".** Story 1.8 still completes with Raw, Direct, Previous-byte, and Constant. The forward mention is an interface note, not a wait for Story 1.10. **Remediation:** none required.

4. **Greenfield setup lives inside Story 1.1 group A** (workspace, pinned toolchain, `ci.sh`, kernel dependency check). The architecture spine specifies no third-party starter template, so a separate "clone the starter" story is not required. Setup, the development toolchain, and CI are early. **Remediation:** none required.

### Recommendations

- Align the Story 2.1 wording with amendment 3 before implementation of the kernel starts, so a reader of the story does not wait on Epic 2 for untagged clauses.
- Treat D-01 through D-09 as open gates. The quality of the acceptance criteria does not close them.
- Split Story 1.13 only if the team wants each group to finish on its own. The user outcome can stay a single epic.

## Summary and Recommendations

### Overall Readiness Status

NEEDS WORK

The functional requirements are covered. All 15 PRD FRs have a story. There is no critical epic-structure violation. Implementation of untagged kernel clauses, whole-file raw, and a private benchmark can proceed under the gates the epics already state.

Reportable evaluation, family serialization, block-field packing, and the partition rule are not ready. Those paths wait on owner decisions that are still proposals (D-01 through D-09, PRD Open Questions 2 through 8). Starting them now would mean a story inventing an answer the epics forbid.

### Critical Issues Requiring Immediate Action

No critical structural defect blocks the untagged paths.

Before the paths they govern:

- D-01, D-02, and D-03 are unapproved. Family serialization, block writer and parser work, and partition acceptance stay closed until the owner picks one contract for each.
- Story 2.1 group A and delivery checklist item 1 still say Epic 1 waits on Epic 2. Amendment 3 says the opposite for untagged clauses. Those two texts need to match before kernel work starts, or an implementer will wait on the wrong artifact.
- Open Questions 3, 4, and 5 (overlap K, the Codebook recipe, LJ Speech prefixes) are unset. Reportable evaluation cannot start without them. Open Questions 7 and 8 (one-class win, go-threshold numbers) are unset and block a scientific verdict. Open Question 6 (counsel review) blocks public release and does not block the measurement core.

### Recommended Next Steps

1. Done on 2026-11-01 in `epics.md`: E1-AC08, S2.1-AC01, S2.1-AC05, and delivery checklist item 1 now say the spec seed is the G0 draft, and untagged clauses do not wait for `spec/bitstream.md`. Owner gates below are unchanged.
2. Before any reportable benchmark, record approved answers for the Codebook recipe, K, the LJ Speech prefixes and sample seed, and the modern tuning executable. Do not let a story fill those in.
3. Before the first evaluation run, set the two SM-1 go-threshold numbers and what a win on exactly one Real input class means. Leave the counsel review on the public-release gate.

### Final Note

This assessment identified 13 issues across 3 categories: 7 open PRD questions (Open Questions 2–8), 1 missing UX document, and 5 epic-quality findings (3 major, 2 minor). Address the owner gates before the work those gates name. Untagged kernel work and whole-file raw can proceed as-is. These findings can be used to improve the artifacts, or the unblocked paths can start while the gates stay closed.

**Date:** 2026-11-01
**Assessor:** Implementation Readiness (`bmad-check-implementation-readiness`)
**Report:** `_bmad-output/planning-artifacts/implementation-readiness-report-2026-11-01.md`

## Decisions recorded after the assessment

Recorded 2026-11-01 in `epics.md`, `prd.md`, `SPEC.md`, and `bitstream-spec-seed.md`.

- D-01 approved. Evaluator ranges stay. Stored trigonometric cw is at most 29 and recurrence cw is at most 25.
- D-02 approved. 5-bit header, kind-specific fields, then checked zero padding to the next byte.
- D-03 approved. The final partial span uses the smallest nominal length in {256, 1024, 2048} that covers the bytes left.
- D-05 approved. Growth path: at least 1% of the class Total encoded size and a majority of that class's evaluation inputs. A one-class win is negative and opens only the deferred PCM experiments.
- D-06 approved. Shrink the corpus first without changing frozen evaluation inputs, then lower the Codebook cap.
- Left open: K, LJ Speech metadata, the Debian tuning package, model constants (D-07), counsel (D-08), and the production Codebook recipe (D-09).
- Story 1.13 stays one story with four groups.

