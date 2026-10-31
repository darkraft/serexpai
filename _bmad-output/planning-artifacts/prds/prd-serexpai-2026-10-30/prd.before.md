---
title: "PRD: MBSC"
status: final
created: 2026-10-30
updated: 2026-10-30
---

# PRD: MBSC
*Working title. [ASSUMPTION: MBSC is the product name; the repository is `serexpai`.]*

## 0. Document Purpose

This PRD defines v1 of MBSC for the author (researcher and builder) and for downstream workflow owners: UX or CLI design, architecture, and epics and stories. It builds on two inputs and does not duplicate them: the architecture review at `docs/mbsc_architecture_review.md` (the only specification input) and the product brief at `_bmad-output/planning-artifacts/briefs/brief-serexpai-2026-10-30/`. The brief's `addendum.md` (the *brief addendum* below) holds the technical detail (rate formulas, bitstream gaps, retrieval validation) that FR-6, FR-8 and FR-9 point to. This PRD's own `addendum.md` holds the research digest (comparables, corpora, licences). Vocabulary is anchored in the Glossary (§3). Features group the functional requirements (§4), with FRs numbered globally. Inferences are tagged `[ASSUMPTION]` inline and indexed in §9.

## 1. Vision

MBSC is a lossless compressor and decompressor, apparently novel (a shallow literature search found no prior work fitting trigonometric or recurrence formulas to raw bytes), delivered as a command-line tool. It predicts each byte of a block from a compact mathematical formula (polynomial, trigonometric or recurrence), then entropy-codes the modular residual with rANS. Reconstruction is exact for any deterministic predictor, so a poor predictor costs size, never correctness. The encoder searches a codebook; the decoder only evaluates the formula whose parameters are in the bitstream, so it needs neither the codebook nor an index.

The project's central claim is untested: that compact formulas, once their parameters and modelling overhead are paid for, leave residuals cheap enough to beat simple baselines on real byte streams. v1 is therefore a rate-measurement experiment wrapped in a usable CLI. It must produce a trustworthy answer to one question: does the best available formula reduce total encoded size against constant, previous-byte and direct entropy-coding baselines, on real inputs?

If it does, approximate retrieval, reversible format-aware preprocessing and SIMD entropy coding become worthwhile engineering and the project grows into an open-source compressor with a published bitstream spec. If it does not, the repository stays as a reproducible negative result. Both outcomes are acceptable; an unreproducible or ambiguous outcome is not.

## 2. Target User

### 2.1 Jobs To Be Done

- **Primary (the author):** find out, with defensible measurements, whether formula-as-predictor compression works on real byte streams, and have a correct, usable compressor if it does.
- **Eventual (open-source audience):** `[ASSUMPTION: a large audience of CLI users, researchers and contributors, per the author's own wording]` reproduce the published results, read the bitstream spec, and trust that the round-trip is lossless.

### 2.2 Non-Users (v1)

- Anyone needing a production-grade compressor: v1 has no throughput targets and no stability promise for the bitstream format.
- Library embedders: v1 ships a CLI, not a stable library API `[ASSUMPTION: this does not close off embedding later]`.

### 2.3 Key User Journeys

- **UJ-1. Darian runs the experiment.** Darian, the author, points the benchmark at the corpora, gets a per-class table of total encoded size for every baseline and for MBSC, and can see in one read whether MBSC won on at least two real input classes.
- **UJ-2. Darian round-trips a file.** Darian compresses a file with `compress`, decompresses the bitstream with `decompress`, and confirms the output is byte-identical, including when the file is empty, tiny or already compressed.
- **UJ-3. Mei reproduces the result.** `[ASSUMPTION]` Mei, a compression researcher, reads the bitstream spec and the corpus manifest, reruns the benchmark on their machine, and gets the same sizes, so the claim stands or falls on evidence.

## 3. Glossary

- **Block** — A contiguous span of input bytes coded as one unit. Has a length; the final Block of an input may be shorter than the nominal length (a partial Block).
- **Predictor** — A deterministic function that, for each position in a Block, yields a predicted byte ŷ. A Formula, constant prediction or previous-byte prediction.
- **Formula** — A Predictor from one Series family with fully parameterized, quantized parameters, evaluated by the Normative Evaluator.
- **Series family** — Polynomial, trigonometric or recurrence. Each has its own parameter layout.
- **Normative Evaluator** — The fixed-point routine, defined exactly by the spec, that evaluates a Formula. The decoder and the encoder must produce identical ŷ from identical parameters on every platform.
- **Codebook** — The finite set of Formulas the encoder searches. Encoder-only: never read by the decoder.
- **Candidate** — A Predictor (or the direct-coding and raw options) considered for one Block during search.
- **Residual** — δ = ((y − ŷ + 128) mod 256) − 128 for input byte y; reconstruction is y = (ŷ + δ) mod 256.
- **Rate selector** — The encoder rule that picks, per Block, the Candidate with the smallest actual encoded size. A minimum-description-length rule, not rate-distortion, because reconstruction is always lossless.
- **Raw Block** — A Block stored uncompressed with its framing and an unambiguous length.
- **Baseline** — A comparison method inside MBSC's own coding path: constant prediction, previous-byte prediction, or direct byte entropy coding (no Predictor).
- **Probability model** — The single, fully specified model v1 uses to turn Residuals into rANS symbol probabilities.
- **Bitstream** — The self-describing compressed output: framing, per-Block headers, Formula parameters, model signalling and rANS payload.
- **Total encoded size** — Every byte of the Bitstream: headers, parameters, model signalling, rANS state and padding, framing and length fields. Never histogram entropy alone.
- **Input class** — A category of input for reporting: synthetic signal, text, PCM, executable, already-compressed data. Synthetic signals are always reported separately from real classes.
- **Real input class** — Text, PCM or executable, drawn from corpora in the Corpus manifest.
- **Corpus manifest** — The versioned list of benchmark inputs: name, Input class, source, licence and content hash.
- **Baselines-only arm** — Full MBSC with Formulas disabled. Every other Candidate stays available, including Baselines, Raw Blocks and Whole-file raw mode. It uses the same Block-length set, partition rule and coding path, applied independently (not copied from full MBSC's partition), so it is the best result achievable without Formulas. It is the comparator for Gain, so that Gain is attributable to Formulas.
- **Whole-file raw mode** — A Bitstream that stores the entire input uncompressed behind one fixed-size header. It bounds worst-case expansion.
- **N** — Input length in bytes. **Rate** — Total encoded size in bits divided by N (bits per input byte); the raw threshold is 8. Written `S_total / (8N)` when expressed as a dimensionless ratio, where `S_total` is Total encoded size.
- **Q** — The fixed-point quantization (number of fractional bits) of a Formula's parameters.
- **Gain** — Any reduction in Total encoded size, summed over the inputs of an Input class, for full MBSC versus the Baselines-only arm and versus the best whole-input Baseline, defined as the Baseline with the smallest Total encoded size for each input, summed over the class `[ASSUMPTION: the stronger of the two possible comparators]`. No minimum margin; the report also states per-input wins and the margin so a marginal Gain can be interpreted.

## 4. Features

### 4.1 Predictors and Residual Reconstruction

**Description:** The core of the design: every Block is reconstructed exactly from ŷ plus modular Residuals, whatever the Predictor. The three Series families are evaluated by one Normative Evaluator in fixed-point arithmetic so that encoder and decoder agree bit for bit. Realizes UJ-2.

**Functional Requirements:**

#### FR-1: Modular residual reconstruction

The encoder computes Residuals and the decoder reconstructs bytes with modular arithmetic for any Predictor. Realizes UJ-2.

**Consequences (testable):**
- For every byte value y and every predicted value ŷ in 0..255, decoding the Residual yields y.
- A property test over random Blocks and random Predictors round-trips with zero mismatches.

#### FR-2: Normative Evaluator semantics

The Normative Evaluator is specified exactly enough that two independent implementations produce identical ŷ for any valid parameters. Realizes UJ-3.

**Consequences (testable):**
- The spec states: intermediate integer widths, overflow behaviour, rounding (including negative halfway cases), division and shift semantics, trigonometric argument reduction and lookup-table interpolation, coefficient limits, recurrence initialization, and mathematical (non-negative) modulo for negative inputs.
- The decoder uses no floating point `[ASSUMPTION: fixed-point only]`.
- A published test-vector set of (parameters, Block length, ŷ sequence) passes on every supported platform `[ASSUMPTION: Linux on x86-64 and aarch64]`.

#### FR-3: Family-specific parameter layouts

Each Series family has its own parameter layout. Realizes UJ-3.

**Consequences (testable):**
- Polynomial degree is stored separately from its parameter count (degree + 1 coefficients).
- Trigonometric order is stored separately from its 2·order + 1 coefficients, with independent limits.
- The recurrence modulus has its own name (`m_mod`) and is never confused with a degree or order.
- Every field is wide enough, or uses a defined mapping, to represent the full supported range of Q and of counts (the review's examples: Q in [8,24], counts 1 to 16; see the brief addendum).

### 4.2 Codebook and Encoder Search

**Description:** The encoder splits input into Blocks and, for each Block, evaluates every Candidate in a small Codebook exhaustively, then lets the Rate selector pick the smallest. Approximate retrieval is out of scope: exhaustive search is the reference that any later index must be measured against. Realizes UJ-1, UJ-2.

**Functional Requirements:**

#### FR-4: Codebook

A Codebook of fully parameterized Formulas from all three Series families is defined and versioned, small enough to search exhaustively. Realizes UJ-1.

**Consequences (testable):**
- The Codebook contains Formulas from polynomial, trigonometric and recurrence families.
- The Codebook size per family has a stated upper bound that keeps benchmark encoding within the time budget (Open Question 8); the bound is set in architecture.
- The Codebook is reproducible from a documented recipe, so a reviewer can regenerate it.
- Codebook identity and version are recorded in the benchmark report; the decoder never reads the Codebook.

**Out of Scope:**
- Local parameter fitting from candidate initializations `[ASSUMPTION: v1 uses fully parameterized Formulas; see Open Question 1]`.

#### FR-5: Exhaustive per-Block search

For each Block the encoder evaluates every Codebook Formula and every Baseline. Realizes UJ-1.

**Consequences (testable):**
- The search result for a Block is deterministic for a given input and Codebook.
- Block length is one of a small defined set `[ASSUMPTION: 256, 1024 and 2048 bytes, the lengths named in the review; each Block's length is signalled in the Bitstream]`, and the Rate selector chooses among them.
- The rule that partitions an input into Blocks of mixed lengths (aligned boundaries, greedy or dynamic programming) is deterministic and stated in the spec `[ASSUMPTION: set in architecture; see Open Question 6]`. The benchmark can also run each fixed Block length alone.
- The final partial Block is handled without padding the output beyond its signalled length.

#### FR-6: Rate selector on actual size

The Rate selector minimizes Total encoded size, using zero-order entropy only as an optional screen. The formulas for rate, the raw comparison and the screen are in the brief addendum. Realizes UJ-1.

**Consequences (testable):**
- For each Block, the chosen Candidate's actual encoded size is less than or equal to that of every other evaluated Candidate covering the same span; the partition rule (FR-5) decides which lengths are compared.
- A Raw Block wins whenever compressed size is not strictly smaller than the true Raw Block size, framing included on both sides.
- Rate is reported as bits per input byte (raw threshold 8), or as the ratio `S_total / (8N)`; the two are never mixed.
- The entropy screen is off in `benchmark` mode, so the guarantee above covers all Candidates.

### 4.3 Baselines

**Description:** Baselines run through the same coding path as Formulas, so that any difference in Total encoded size is attributable to the Predictor and nothing else. Realizes UJ-1.

**Functional Requirements:**

#### FR-7: Baseline Predictors and direct entropy coding

The encoder supports constant prediction, previous-byte prediction and direct byte entropy coding as Candidates. Realizes UJ-1.

**Consequences (testable):**
- Constant prediction uses a per-Block constant byte carried in the Bitstream `[ASSUMPTION]`. Previous-byte prediction predicts the preceding byte, and the first byte of a Block is predicted as 0 `[ASSUMPTION]`.
- Each Baseline can be forced on for a whole input through `benchmark`, and its Total encoded size reported on its own.
- The Baselines-only arm (Glossary) can be selected through `benchmark`.
- Baselines use the same Probability model and framing as Formulas.

### 4.4 Residual Entropy Coding

**Description:** v1 uses exactly one simple, fully specified Probability model with rANS. Context-adaptive modelling beyond that is out of scope. Whatever the model is, it must be causal: a symbol's model may depend only on information the decoder already has.

**Functional Requirements:**

#### FR-8: Single specified Probability model and rANS coder

Residuals (and bytes under direct entropy coding) are coded with one Probability model and rANS, specified completely. The brief addendum lists the causal-model rules (reverse-order decode, buffering of coding decisions). Realizes UJ-3.

**Consequences (testable):**
- The Probability model is either transmitted in the Bitstream or reconstructed deterministically from information the decoder already holds; the spec says which.
- The spec states how frequencies are normalized and rescaled, the rANS state initialization, renormalization byte order and termination.
- Per-Block model signalling cost is measured and reported, because at 256-byte Blocks it can exceed any Formula gain (Open Question 2).
- If any adaptation exists, the spec states which already-decoded symbols form each context, when updates occur, and whether the model resets at Block boundaries. A decoder never selects a model from the value of the symbol it is decoding.
- Interleaved or SIMD rANS is not used in v1.

### 4.5 Bitstream and Framing

**Description:** The Bitstream is self-describing and unambiguous. Each of the review's listed contract gaps is closed in a written spec that someone other than the author could implement. Realizes UJ-2, UJ-3.

**Functional Requirements:**

#### FR-9: Complete Bitstream contract

The Bitstream has a normative layout; the review's list of contract gaps is in the brief addendum. Realizes UJ-3.

**Consequences (testable):**
- The spec defines bit order within bytes and endianness of multibyte integers, and a header parser (bit-level or byte-level, stated explicitly).
- A raw flag is accompanied by an unambiguous Raw Block length.
- Payload length, framing and termination are normative and stated once.
- A Bitstream carries a format version.
- The empty input, an input shorter than one Block and an input of exactly one Block all round-trip.

#### FR-10: Bounded raw fallback

MBSC never expands an input beyond a defined, documented overhead. Realizes UJ-2.

**Consequences (testable):**
- The spec states the maximum Bitstream size as a function of N (for example N plus a fixed constant), and a test asserts it on random and already-compressed inputs. The numeric constant is set in architecture before implementation.
- `[ASSUMPTION: a Whole-file raw mode is available so the bound is a small constant, not a per-Block cost.]`

#### FR-11: Bitstream specification document

The repository includes a standalone document describing the Bitstream and the Normative Evaluator. Realizes UJ-3.

**Consequences (testable):**
- The document is sufficient, with the test vectors, to write an independent decoder.
- `[ASSUMPTION: the document is part of v1 deliverables because the audience goal should not be closed off.]`

### 4.6 Command-Line Interface

**Description:** A minimal CLI with `compress`, `decompress` and `benchmark` subcommands taking file arguments. No standard-input or standard-output streaming in v1. Realizes UJ-1, UJ-2.

**Functional Requirements:**

#### FR-12: `compress`

The user can compress a file to a Bitstream file. Realizes UJ-2.

**Consequences (testable):**
- Output is deterministic: the same input and Codebook yield the same Bitstream. This is a v1 choice for reproducibility; the format does not require different encoders to agree, only decoders (the Normative Evaluator).
- The command exits non-zero with a message on any I/O error and writes no partial output file as if it were valid `[ASSUMPTION]`.

#### FR-13: `decompress`

The user can decompress a Bitstream file to the original bytes. Realizes UJ-2.

**Consequences (testable):**
- Round-trip output equals the original file byte for byte for every Input class.
- A truncated or malformed Bitstream is rejected with a non-zero exit and a message; the decoder never reads out of bounds, and its memory use is bounded by a stated multiple of the Block length plus the declared output size (the numeric bound is set in architecture). `[ASSUMPTION: no integrity checksum in v1; detection relies on framing. Confirm.]`

### 4.7 Benchmark and Evaluation Protocol

**Description:** The benchmark is the deliverable that answers the research question. It reports Total encoded size per Input class for every Baseline and for MBSC, verifies every round-trip, and keeps synthetic signals apart from real inputs. Realizes UJ-1, UJ-3.

**Functional Requirements:**

#### FR-14: `benchmark`

The user can run the benchmark over the Corpus manifest and get a per-class report. Realizes UJ-1.

**Consequences (testable):**
- For each input the benchmark reports Total encoded size for each whole-input Baseline, for the Baselines-only arm, for full MBSC and for Whole-file raw mode, and verifies the decoded bytes equal the input; a mismatch fails the run.
- Results are aggregated per Input class (Total encoded size summed over the class's inputs `[ASSUMPTION]`). The verdict line states, per Real input class, whether Gain was achieved, and reports per-input wins and the margin.
- Synthetic signals are reported in their own section, never merged into a Real input class aggregate.
- The report breaks Total encoded size into header, Formula parameter, model signalling and payload bytes, and gives the share of Blocks and bytes won by each Series family and each Baseline.
- The report retains, per Block, the encoded cost of every evaluated Candidate, so a later retrieval index can be validated against the exhaustive best (recall and excess compressed bytes).
- The report records the share of zero Residuals per class, so the review's claim that about 65% of residuals are zero on WAV and text is measured, not assumed.
- The report records Codebook version, Block lengths, tool version and Corpus manifest hash.

#### FR-15: Corpus manifest and synthetic signals

The benchmark inputs are defined and reproducible. Realizes UJ-3.

**Consequences (testable):**
- The Corpus manifest names at least text, PCM and executable inputs, plus already-compressed data as a control, each with source, licence and content hash `[ASSUMPTION: the actual corpora are chosen during architecture or the first prototype; see Open Question 4]`.
- Synthetic signals matching each Series family are generated from a documented seed.
- Corpora that cannot be redistributed are referenced by hash and source, not committed.
- The Corpus manifest marks each input as tuning or evaluation. Codebook tuning uses only tuning inputs, and SM-1 is decided on evaluation inputs only.

## 5. Non-Goals (Explicit)

- v1 is not a production compressor and is not competing on throughput or ratio against zstd, xz or similar.
- v1 does not build a candidate-retrieval index, whether VP-tree, ANN or LSH. Any later index must first be validated against exhaustive search, measuring candidate recall and excess compressed bytes.
- v1 does not do SIMD or interleaved rANS, or context-adaptive modelling beyond the one simple Probability model.
- v1 does not do format-aware preprocessing (parsed WAV channels, reversibly separated byte lanes, symbolic text). Each is a separate later experiment and any such preprocessing must be reversible and fully signalled.
- v1 does not set throughput targets until an end-to-end work model names the denominator (input bytes, candidate samples or operations).
- v1 is not a library with a stable API, and promises no Bitstream compatibility between 0.x versions `[ASSUMPTION]`; the format-version field lets a decoder reject unknown versions, and the spec document (FR-11) remains a goal.

## 6. MVP Scope

### 6.1 In Scope

*Measurement core:* FR-1 to FR-10 and FR-12 to FR-15. *Publication gate:* FR-11, the independent-implementation test vectors in FR-2, and SM-5. The publication gate runs in parallel and must not block the benchmark. It is mandatory before any public release, but its effort pays off only if SM-1 is positive.

- Normative Evaluator and modular reconstruction (FR-1 to FR-3).
- Small exhaustively searched Codebook of three Series families and the Rate selector (FR-4 to FR-6).
- Baselines: constant, previous-byte, direct entropy coding (FR-7).
- One specified Probability model with rANS (FR-8).
- Complete Bitstream contract, raw fallback bound (FR-9 to FR-10); the spec document (FR-11) is the publication gate below.
- CLI `compress`, `decompress`, `benchmark` over file arguments (FR-12 to FR-14).
- Corpus manifest and synthetic signals (FR-15).

### 6.2 Out of Scope for MVP

- Candidate-retrieval index, SIMD and interleaved rANS, context-adaptive rANS, and format-aware preprocessing (see §5): deferred until a rate result exists. A retrieval index is meaningful only if exhaustive search shows a Gain; the causal-model rules in FR-8 apply when context-adaptive rANS arrives.
- stdin/stdout streaming: later version.
- Integrity checksum `[NOTE FOR PM: confirm whether a checksum is wanted before the first public release; see Open Question 7]`.
- Licence and contribution model: to be decided before the first public release.

## 7. Success Metrics

Each SM cross-references the FR(s) it validates.

**Primary**
- **SM-1: Research answer.** On evaluation inputs, full MBSC achieves Gain (any reduction in Total encoded size) over both the Baselines-only arm and the best whole-input Baseline on at least two distinct Real input classes. A reduction that comes only from switching among Baselines does not count. If fewer than two Real input classes can be tested (for example no suitable PCM), the result is *inconclusive*, not negative. Validates FR-5, FR-6, FR-7, FR-14, FR-15.
- **SM-2: Lossless round-trip.** 100% of benchmark inputs and test inputs (including empty, shorter than one Block, partial final Block and incompressible) decode byte-identically. Validates FR-1, FR-2, FR-9, FR-13, FR-14.
- **SM-3: No regression on incompressible data.** On already-compressed inputs, Total encoded size does not exceed the documented overhead bound of FR-10. Validates FR-6, FR-10.

**Secondary**
- **SM-4: Reproducibility.** Rerunning the benchmark from the Corpus manifest on a clean checkout yields identical Total encoded sizes. Validates FR-12, FR-14, FR-15.
- **SM-5: Spec sufficiency.** An independent decoder written from the spec and test vectors decodes a sample of Bitstreams `[ASSUMPTION: the author or a reviewer does this once before release; owner and timing unset]`. Validates FR-2, FR-9, FR-11.

**Counter-metrics (do not optimize)**
- **SM-C1: Synthetic-signal compression.** Large Gain on synthetic signals proves only that the encoder can find a matching Formula; it is not evidence for the research answer. Counterbalances SM-1.
- **SM-C2: Codebook fit to the benchmark corpora.** Tuning the Codebook on the same inputs that decide SM-1 inflates the result. Hold out inputs or state the tuning set. Counterbalances SM-1.
- **SM-C3: Encoder speed.** Do not trade rate-measurement fidelity (exhaustive search, actual-size selection) for speed in v1. Counterbalances SM-4.

## 8. Open Questions

1. **Deferred, not blocking v1:** does lookup retrieve fully parameterized Formulas (needs a coverage strategy) or candidate families fitted locally (needs a fitting procedure and compute budget)? v1 uses the first (FR-4).
2. Which single Probability model is used in v1, and is it transmitted or reconstructed? Must be resolved in architecture. `[NOTE FOR PM]` This interacts with Open Question 6: per-Block signalling at 256 bytes can exceed any Formula gain, so the minimum Block length should be chosen after measuring the chosen model's per-Block overhead.
3. Implementation language for the CLI. Decide in architecture.
4. Which real corpora represent text, PCM and executables for SM-1, and what is the control for already-compressed data? Candidates and licence status are in `addendum.md`. `[NOTE FOR PM]` Raw-byte 16-bit PCM interleaves a noisy and a smooth byte, and byte-lane separation is out of scope, so choose PCM inputs (for example 8-bit PCM) knowing a raw-byte result may be weak; text may also lose to the simple baselines. SM-1 needs two Real classes, so settle this before the first benchmark run.
5. Licence and contribution model. Decide before the first public release. Comparable tools use permissive licences (MIT, BSD, 0BSD); see `addendum.md`.
6. Block length set, the Block partition rule, and the Codebook size and composition (Formulas per family, Q values, parameter ranges).
7. Is an integrity checksum wanted in the Bitstream? (Separate from stdin/stdout support, a later-version question.)
8. What time budget is acceptable for exhaustive encoding of the benchmark corpora? Set after the first measurement.
9. Numeric bounds the FRs defer to architecture: Whole-file raw header size and the FR-10 expansion bound, the Codebook size cap, and the decoder memory multiple.

## 9. Assumptions Index

- Title: MBSC is the product name.
- §2.1: the eventual audience is a large open-source community.
- §2.2: not closing off embedding as a library later.
- §2.3 UJ-3: a reviewer (Mei) reproduces the result from the spec and manifest.
- §4.1 FR-2: supported platforms are Linux on x86-64 and aarch64; the decoder uses no floating point.
- §4.2 FR-4: v1 Codebook holds fully parameterized Formulas; no local fitting.
- §4.2 FR-5: Block lengths are 256, 1024 and 2048 bytes, signalled per Block; the partition rule is set in architecture.
- §4.3 FR-7: constant prediction uses a per-Block constant carried in the Bitstream; previous-byte prediction yields 0 for the first byte of a Block.
- §4.5 FR-10: a Whole-file raw mode bounds expansion to a small constant.
- §4.5 FR-11: the Bitstream spec document is a v1 deliverable (publication gate).
- §4.6 FR-12: a failed compression leaves no partial output that looks valid.
- §4.6 FR-13: no integrity checksum in v1.
- §3 Gain and §4.7 FR-14: per-class aggregation is Total encoded size summed over the class's inputs; the best whole-input Baseline is chosen per input.
- §4.7 FR-15: corpora are selected during architecture or the first prototype.
- §5: no Bitstream compatibility between 0.x versions.
- §7 SM-5: owner and timing of the independent decoder check are unset.
