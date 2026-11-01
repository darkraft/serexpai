---
title: "PRD: SerExpAI"
status: final
created: 2026-10-30
updated: 2026-11-01
---

# PRD: SerExpAI
*SerExpAI is the product name. MBSC (Multi-Basis Series Compression System) is the idea it represents. The repository is `serexpai`. Cargo crates use the prefix `sxp-`.*

## 0. Document Purpose

This PRD defines v1 of SerExpAI for the author (researcher and builder) and for downstream workflow owners: UX or CLI design, architecture, and epics and stories. The canonical technical contract is the spec at `_bmad-output/specs/spec-serexpai/`: `SPEC.md` and its companions, chiefly `acceptance-criteria.md` (cited below as AC with a capability number) and `corpora.md`. The *spec seed* is the draft of the normative Bitstream document, at `_bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md`. This PRD was reconciled with it on 2026-10-31 and points to it for technical detail. The PRD's original inputs (the architecture review at `docs/mbsc_architecture_review.md` and the product brief at `_bmad-output/planning-artifacts/briefs/brief-serexpai-2026-10-30/` with its addendum) are kept for rationale and traceability only; some of their technical detail predates the spec. This PRD's own `addendum.md` holds the research digest (comparables, corpora, licences). Vocabulary is anchored in the Glossary (§3). Features group the functional requirements (§4), with FRs numbered globally. Inferences are tagged `[ASSUMPTION]` inline and indexed in §9.

## 1. Vision

SerExpAI is a lossless compressor and decompressor, delivered as a command-line tool. It represents MBSC (Multi-Basis Series Compression System), an idea apparently novel in this form (a shallow literature search found no prior work fitting trigonometric or recurrence formulas to raw bytes). It predicts each byte of a block from a compact mathematical formula (polynomial, trigonometric or recurrence), then entropy-codes the modular residual with a binary range coder. Reconstruction is exact for any deterministic predictor, so a poor predictor costs size, never correctness. The encoder searches a codebook; the decoder only evaluates the formula whose parameters are in the bitstream, so it needs neither the codebook nor an index.

The project's central claim is untested: that compact formulas, once their parameters and modelling overhead are paid for, leave residuals cheap enough to beat simple baselines on real byte streams. v1 is therefore a rate-measurement experiment wrapped in a usable CLI. It must produce a trustworthy answer to one question: does the best available formula reduce total encoded size against constant, previous-byte and direct entropy-coding baselines, on real inputs?

If it does so by a margin large enough to act on (SM-1's go threshold), approximate retrieval, reversible format-aware preprocessing and SIMD entropy coding become worthwhile engineering and the project grows into an open-source compressor with a published bitstream spec. Commercial use is planned, so licence and patent decisions must not close it off. If it does not, the repository stays as a reproducible negative result. Both outcomes are acceptable; an unreproducible or ambiguous outcome is not.

## 2. Target User

### 2.1 Jobs To Be Done

- **Primary (the author):** find out, with defensible measurements, whether formula-as-predictor compression works on real byte streams, and have a correct, usable compressor if it does.
- **Eventual (open-source audience):** `[ASSUMPTION: a large audience of CLI users, researchers and contributors, per the author's own wording]` reproduce the published results, read the bitstream spec, and trust that the round-trip is lossless.
- **Commercial use (planned):** SerExpAI may be used or distributed commercially. v1 does not serve that use directly, but its licence, contribution model and patent posture must not close it off.

### 2.2 Non-Users (v1)

- Anyone needing a production-grade compressor: v1 has no throughput targets and no stability promise for the bitstream format.
- Library embedders: v1 ships a CLI, not a stable library API `[ASSUMPTION: this does not close off embedding later]`.

### 2.3 Key User Journeys

- **UJ-1. Darian runs the experiment.** Darian, the author, points the benchmark at the corpora, gets a per-class table of total encoded size for every baseline and for MBSC, and can see in one read whether MBSC won on at least two real input classes.
- **UJ-2. Darian round-trips a file.** Darian compresses a file with `compress`, decompresses the bitstream with `decompress`, and confirms the output is byte-identical, including when the file is empty, tiny or already compressed.
- **UJ-3. Mei reproduces the result.** `[ASSUMPTION: an outside reviewer will attempt reproduction]` Mei, a compression researcher, reads the bitstream spec and the corpus manifest, reruns the benchmark on their machine, and gets the same sizes, so the claim stands or falls on evidence.

## 3. Glossary

- **SerExpAI** — The product name. The repository is `serexpai`. Cargo crates use the prefix `sxp-`.
- **MBSC** — Multi-Basis Series Compression System. The idea SerExpAI represents. In the benchmark, "full MBSC" means that idea with Formulas enabled.
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
- **Probability model** — The single, fully specified model v1 uses to turn Residuals into the binary range coder's frequencies.
- **Bitstream** — The self-describing compressed output: framing, per-Block headers, Formula parameters, coder payload and a CRC-32 trailer.
- **Total encoded size** — Every byte of the Bitstream: headers, parameters, coder payload, coder flush and padding, framing, length fields and the CRC trailer. Never histogram entropy alone.
- **Input class** — A category of input for reporting: synthetic signal, text, PCM, executable, already-compressed data. Synthetic signals are always reported separately from real classes.
- **Real input class** — Text, PCM or executable, drawn from corpora in the Corpus manifest.
- **Corpus manifest** — The versioned list of benchmark inputs: name, Input class, source, licence, content hash, role (tuning or evaluation) and any Corpus-side transform.
- **Corpus-side transform** — A documented, versioned, hashed conversion the fetch tool applies when defining a corpus input (for example 16-bit to 8-bit PCM, or a slice of a large file). It is corpus preparation, not compressor preprocessing.
- **Baselines-only arm** — Full MBSC with Formulas disabled. Every other Candidate stays available (the Baselines and Raw Blocks), and so does the arm-level Whole-file raw option. It uses the same Block-length set, partition rule and coding path, applied independently (not copied from full MBSC's partition), so it is the best result achievable without Formulas. It is the comparator for Gain, so that Gain is attributable to Formulas.
- **Whole-file raw mode** — A Bitstream that stores the entire input uncompressed behind a short header, followed by the CRC trailer. It bounds worst-case expansion (FR-10).
- **N** — Input length in bytes. **Rate** — Total encoded size in bits divided by N (bits per input byte); the raw threshold is 8. Written `S_total / (8N)` when expressed as a dimensionless ratio, where `S_total` is Total encoded size.
- **Q** — The fixed-point quantization (number of fractional bits) of a Formula's parameters, in [8, 24].
- **Work count** — Host-independent encode cost: model decisions and coder steps per input byte, per benchmark arm. Wall-clock time lives only in the timing log.
- **Gain** — Any reduction in Total encoded size, summed over the inputs of an Input class, for full MBSC versus the Baselines-only arm. The best whole-input Baseline (the Baseline with the smallest Total encoded size for each input, summed over the class) is reported as a consistency check: the Baselines-only arm offers a superset of that Baseline's Candidates under the same exact-size selection, so the arm is never larger and the check can never decide Gain. No minimum margin; the report also states per-input wins and the margin so a marginal Gain can be interpreted.

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
- The decoder uses no floating point.
- A published test-vector set of (parameters, Block length, ŷ sequence) passes on every supported platform: Linux on x86-64 and aarch64.
- A decoder rejects any parameter outside its limits rather than clamping it; each limit has a boundary vector and a rejection vector.

#### FR-3: Family-specific parameter layouts

Each Series family has its own parameter layout. Realizes UJ-3.

**Consequences (testable):**
- Polynomial degree is stored separately from its parameter count (degree + 1 coefficients).
- Trigonometric order is stored separately from its 2·order + 1 coefficients, with independent limits.
- The recurrence modulus has its own name (`m_mod`) and is never confused with a degree or order.
- Every field covers the v1 ranges (Q in [8, 24] and counts 1 to 16) through a defined encoding (AC CAP-2).

### 4.2 Codebook and Encoder Search

**Description:** The encoder splits input into Blocks and, for each Block, evaluates every Candidate in a small Codebook exhaustively, then lets the Rate selector pick the smallest. Approximate retrieval is out of scope: exhaustive search is the reference that any later index must be measured against. Realizes UJ-1, UJ-2.

**Functional Requirements:**

#### FR-4: Codebook

A Codebook of fully parameterized Formulas from all three Series families is defined and versioned, small enough to search exhaustively. Realizes UJ-1.

**Consequences (testable):**
- The Codebook contains Formulas from polynomial, trigonometric and recurrence families.
- The Codebook holds at most 4096 Formulas, at most 2048 per family, so benchmark encoding fits the time budget of FR-14 `[ASSUMPTION: encode cost is a work-model estimate until the first measurement]`.
- The Codebook is reproducible from a documented recipe, so a reviewer can regenerate it.
- Codebook identity and version are recorded in the benchmark report; the decoder never reads the Codebook.

**Out of Scope:**
- Local parameter fitting from candidate initializations `[ASSUMPTION: v1 uses fully parameterized Formulas; see Open Question 1]`.

#### FR-5: Exhaustive per-Block search

For each Block the encoder evaluates every Codebook Formula and every Baseline. Realizes UJ-1.

**Consequences (testable):**
- The search result for a Block is deterministic for a given input and Codebook, and identical whatever the thread count; a test compares a parallel run with a single-threaded one.
- Block length is one of 256, 1024 or 2048 bytes, signalled per Block in the Bitstream, and the Rate selector chooses among them.
- The rule that partitions an input into Blocks of mixed lengths is deterministic, selects on exact size, and is stated in the spec (AC CAP-3). The benchmark can also run each fixed Block length alone.
- Each Block decodes from its own header and payload alone: Formulas use Block-local positions, and no model or coder state carries across a Block boundary.
- The final partial Block is handled without padding the output beyond its signalled length.

#### FR-6: Rate selector on actual size

The Rate selector minimizes Total encoded size; the exact rules are in AC CAP-4. Realizes UJ-1.

**Consequences (testable):**
- For each Block, the chosen Candidate's actual encoded size is less than or equal to that of every other evaluated Candidate covering the same span; the partition rule (FR-5) decides which lengths are compared.
- A Raw Block wins whenever compressed size is not strictly smaller than the true Raw Block size, framing included on both sides.
- On equal size, a Baseline beats a Formula, and Whole-file raw mode beats everything, including for the empty input. Gain is therefore never awarded to a Formula on a tie.
- Rate is reported as bits per input byte (raw threshold 8), or as the ratio `S_total / (8N)`; the two are never mixed.
- v1 has no entropy screen or other heuristic that can change which Candidate wins, so the guarantee above covers all Candidates and `compress` produces the same bytes as the full-MBSC benchmark arm. Speed-ups that provably cannot change the selection are allowed.

### 4.3 Baselines

**Description:** Baselines run through the same coding path as Formulas, so that any difference in Total encoded size is attributable to the Predictor and nothing else. Realizes UJ-1.

**Functional Requirements:**

#### FR-7: Baseline Predictors and direct entropy coding

The encoder supports constant prediction, previous-byte prediction and direct byte entropy coding as Candidates. Realizes UJ-1.

**Consequences (testable):**
- Constant prediction uses a per-Block constant byte carried in the Bitstream. Previous-byte prediction predicts the preceding byte, and the first byte of a Block is predicted as 0.
- Each Baseline can be forced on for a whole input through `benchmark`, and its Total encoded size reported on its own. A forced-Baseline arm still uses Raw Blocks, the same partition rule and the Whole-file raw check; only its Candidate set differs.
- The Baselines-only arm (Glossary) can be selected through `benchmark`.
- Baselines use the same Probability model and framing as Formulas.

### 4.4 Residual Entropy Coding

**Description:** v1 uses exactly one simple, fully specified Probability model with a binary range coder. Context-adaptive modelling beyond that is out of scope. The model is causal: a symbol's model depends only on information the decoder already has. Realizes UJ-2, UJ-3.

**Functional Requirements:**

#### FR-8: Single specified Probability model and binary range coder

Residuals (and bytes under direct entropy coding) are coded with one Probability model and a binary range coder, specified completely. The model's update rule, the coder's mapping and the decoder's per-decision order are in the spec (AC CAP-6 and the patent safeguards in SPEC Constraints). Realizes UJ-3.

**Consequences (testable):**
- The Probability model is reconstructed deterministically from already-decoded symbols; nothing about it is transmitted.
- The model is adaptive order-0, reset at every Block start. The spec states the model's update rule, how probabilities map to coder frequencies, and the coder's initialization, renormalization byte order and termination.
- Model constants are tuned on tuning inputs only and scored on the Baselines-only arm, so they cannot favour Formula residuals. Formula coefficient and frequency widths are scored on full MBSC. Ties go to the simpler setting, and the tuning grid and every score are recorded.
- The per-Block fixed cost (header and coder flush) and the model's adaptation overhead are measured and reported, because at 256-byte Blocks they can exceed any Formula gain.
- A decoder never selects a model from the value of the symbol it is decoding.
- Interleaved or SIMD entropy coding is not used in v1.
- rANS is not used in v1. Adopting it later would need a freedom-to-operate review of Microsoft US 11,234,023 B2 and a format version bump; v1's decoder structure follows the patent safeguards in the spec. Changing the decoder's per-decision structure, or adding a field for coder-state retention, symbol width or probability tables, needs a new freedom-to-operate review first.

### 4.5 Bitstream and Framing

**Description:** The Bitstream is self-describing and unambiguous. Each of the review's listed contract gaps is closed in a written spec that someone other than the author could implement. Realizes UJ-2, UJ-3.

**Functional Requirements:**

#### FR-9: Complete Bitstream contract

The Bitstream has a normative layout, defined in the spec seed and checked by AC CAP-7. Realizes UJ-3.

**Consequences (testable):**
- The spec defines bit order within bytes and endianness of multibyte integers, and a header parser (bit-level or byte-level, stated explicitly).
- A raw flag is accompanied by an unambiguous Raw Block length.
- Payload length, framing and termination are normative and stated once.
- A Bitstream carries a format version. Any change to the Normative Evaluator, normative tables, model constants, coder, checksum or layout bumps the format version and regenerates the test vectors in the same change.
- Every Bitstream ends with a CRC-32 of the decoded bytes, and a decoder rejects a mismatch, so corruption that framing cannot see (Formula parameters, kind codes, Raw Blocks) is detected.
- The empty input, an input shorter than one Block and an input of exactly one Block all round-trip.

#### FR-10: Bounded raw fallback

MBSC never expands an input beyond a defined, documented overhead. Realizes UJ-2.

**Consequences (testable):**
- The Bitstream is at most N + 11 bytes for format versions 1 to 14 (15 is reserved), and a test asserts it on random and already-compressed inputs.
- A Whole-file raw mode makes the bound a small constant, not a per-Block cost.

#### FR-11: Bitstream specification document

The repository includes a standalone document describing the Bitstream and the Normative Evaluator. Realizes UJ-3.

**Consequences (testable):**
- The document is sufficient, with the test vectors, to write an independent decoder.
- The repository includes a Python reference decoder (standard library only) written from the document, not translated from the Rust, and a generator of valid Bitstreams that also produces mutated copies of them for rejection testing. Every reject rule and limit is covered by at least one vector.
- The Rust decoding kernel (`sxp-kernel`) and the Python decoder agree on every vector in CI on both platforms. Before spec freeze this check does not block the benchmark; it must pass at spec freeze and before any public release.
- The document is a v1 deliverable, so the audience goal is not closed off. It is published under CC BY 4.0, and the test vectors and normative tables under CC0.

### 4.6 Command-Line Interface

**Description:** A minimal CLI with `compress`, `decompress` and `benchmark` subcommands taking file arguments. No standard-input or standard-output streaming in v1. Realizes UJ-1, UJ-2.

**Functional Requirements:**

#### FR-12: `compress`

The user can compress a file to a Bitstream file. Realizes UJ-2.

**Consequences (testable):**
- Output is deterministic: the same input and Codebook yield the same Bitstream. This is a v1 choice for reproducibility; the format does not require different encoders to agree, only decoders (the Normative Evaluator).
- The command exits non-zero with a message on any I/O error and writes no partial output file as if it were valid.
- The command refuses inputs of 2³⁵ bytes or more with a non-zero exit.

#### FR-13: `decompress`

The user can decompress a Bitstream file to the original bytes. Realizes UJ-2.

**Consequences (testable):**
- Round-trip output equals the original file byte for byte for every Input class.
- A truncated, corrupted or malformed Bitstream is rejected with a non-zero exit and a message, and leaves no output file. Detection combines framing checks with the CRC of FR-9. The decoder never reads out of bounds, and its heap working set is at most 64 KiB, independent of N.

### 4.7 Benchmark and Evaluation Protocol

**Description:** The benchmark is the deliverable that answers the research question. It reports Total encoded size per Input class for every Baseline and for MBSC, verifies every round-trip, and keeps synthetic signals apart from real inputs. Realizes UJ-1, UJ-3.

**Functional Requirements:**

#### FR-14: `benchmark`

The user can run the benchmark over the Corpus manifest and get a per-class report. Realizes UJ-1.

**Consequences (testable):**
- For each input the benchmark reports Total encoded size for each whole-input Baseline, for the Baselines-only arm, for full MBSC and for Whole-file raw mode, and verifies the decoded bytes equal the input; a mismatch fails the run.
- Results are aggregated per Input class (Total encoded size summed over the class's inputs `[ASSUMPTION: summing weights inputs by size; equal-sized slices in corpora.md keep weights equal for text and executables]`). The verdict line states, per Real input class, whether Gain was achieved, and reports per-input wins and the margin. The overall verdict is positive, marginal, inconclusive or negative, as defined by SM-1.
- Synthetic signals are reported in their own section, never merged into a Real input class aggregate.
- The report breaks Total encoded size into header, Formula parameter, coder payload and coder flush bytes, reports the model's adaptation overhead separately, and gives the share of Blocks and bytes won by each Series family and each Baseline.
- The benchmark writes a sidecar file with the encoded cost of every evaluated Candidate per Block, so a later retrieval index can be validated against the exhaustive best (recall and excess compressed bytes). It is written by default for inputs up to a size stated in the spec, and on request above it, and the report states which inputs have one.
- The report records the share of zero Residuals per class, so the review's claim that about 65% of residuals are zero on PCM audio and text is measured, not assumed.
- The report records Codebook version and hash, Block lengths, tool version, Corpus manifest hash, tuning-input hashes and the overlap checker's result (FR-15).
- The report gives a deterministic Work count per arm (model decisions and coder steps per input byte). Wall-clock time and host details go to a separate timing log, kept out of the report so reruns stay identical. A full benchmark run completes within 24 hours on one host, named in the timing log before the first full run. If it does not, the corpus shrinks first (keeping the FR-15 minimum per class), frozen evaluation inputs stay unchanged, and then the Codebook cap is lowered. Changing evaluation inputs is a new experiment revision. The report states the excess bytes this costs on tuning inputs. Approved 2026-11-01 (D-06).

#### FR-15: Corpus manifest and synthetic signals

The benchmark inputs are defined and reproducible. Realizes UJ-3.

**Consequences (testable):**
- The Corpus manifest names text, PCM and executable inputs, plus already-compressed data as a control, each with source, licence and content hash. The Real input classes are text, 8-bit PCM and executables, with tuning and evaluation inputs drawn from different sources; the roster is fixed in the spec companion `corpora.md`.
- Each Real input class has at least three evaluation inputs (size range in `corpora.md`); a class with fewer cannot be tested for SM-1. The report notes known correlation between evaluation inputs of a class.
- Any Corpus-side transform (16-bit to 8-bit PCM, slicing a large file) is documented and hashed in the manifest. Raw 16-bit PCM is reported beside the derived class but does not count toward SM-1.
- Synthetic signals matching each Series family are generated from a documented seed.
- Corpora that cannot be redistributed are referenced by hash and source, not committed.
- The Corpus manifest marks each input as tuning or evaluation. Codebook tuning uses only tuning inputs, and SM-1 is decided on evaluation inputs only. A checker proves that no tuning input shares a run of K bytes or more with any evaluation input (K proposed as 1 KiB; Open Question 3). A failure blocks the benchmark until the tuning input is replaced; evaluation inputs never change.

## 5. Non-Goals (Explicit)

- v1 is not a production compressor and is not competing on throughput or ratio against zstd, xz or similar.
- v1 does not build a candidate-retrieval index, whether VP-tree, ANN or LSH. Any later index must first be validated against exhaustive search, measuring candidate recall and excess compressed bytes.
- v1 does not do SIMD or interleaved entropy coding, or context-adaptive modelling beyond the one simple Probability model.
- v1 does not do format-aware preprocessing (parsed WAV channels, reversibly separated byte lanes, symbolic text). Each is a separate later experiment and any such preprocessing must be reversible and fully signalled. Corpus-side transforms that define a benchmark input (§3) are not preprocessing.
- v1 does not set throughput targets. FR-14's Work count names the denominator; a target can follow once one is needed.
- v1 is not a library with a stable API, and promises no Bitstream compatibility between 0.x versions; the format-version field lets a decoder reject unknown versions, and the spec document (FR-11) is mandatory before any public release (§6.1).
- v1 has no per-Block checksums and no cryptographic integrity; the whole-file CRC of FR-9 detects accidental corruption only.

## 6. MVP Scope

### 6.1 In Scope

*Measurement core:* FR-1 to FR-10 and FR-12 to FR-15. *Publication gate:* FR-11 and SM-5. The publication gate runs in parallel and must not block the benchmark. It is mandatory before any public release, but its effort pays off only if SM-1 is positive.

- Normative Evaluator and modular reconstruction (FR-1 to FR-3).
- Small exhaustively searched Codebook of three Series families and the Rate selector (FR-4 to FR-6).
- Baselines: constant, previous-byte, direct entropy coding (FR-7).
- One specified Probability model with a binary range coder (FR-8).
- Complete Bitstream contract with a mandatory CRC-32, and the raw fallback bound (FR-9 to FR-10); the spec document (FR-11) belongs to the publication gate above.
- CLI `compress`, `decompress`, `benchmark` over file arguments (FR-12 to FR-14).
- Corpus manifest and synthetic signals (FR-15).
- Licence and contribution model: code and the Python reference decoder under MIT OR Apache-2.0, the spec document under CC BY 4.0, test vectors and normative tables under CC0, contributions by DCO sign-off with no CLA, and a human who has read, verified and decided on every AI-assisted change.

### 6.2 Out of Scope for MVP

- Candidate-retrieval index, SIMD and interleaved entropy coding, context-adaptive modelling, and format-aware preprocessing (see §5): deferred until a rate result exists. A retrieval index is meaningful only if exhaustive search shows a Gain; the causal-model rules in FR-8 apply when context-adaptive modelling arrives.
- rANS: kept as an informative appendix of the spec seed only (FR-8).
- stdin/stdout streaming: later version.

## 7. Success Metrics

Each SM cross-references the FR(s) it validates.

**Primary**
- **SM-1: Research answer.** On evaluation inputs, full MBSC achieves Gain (any reduction in Total encoded size) over the Baselines-only arm on at least two distinct Real input classes, so a reduction that comes only from switching among Baselines does not count. The best whole-input Baseline is reported alongside as a consistency check (§3 Gain). If fewer than two Real input classes can be tested (for example no suitable PCM), the result is *inconclusive*, not negative; otherwise, fewer than two classes with Gain is *negative*. Any reduction answers the research question, but going ahead with the growth path in §1 also needs, in each of the two classes, a Gain of at least 1% of the class's Total encoded size and wins on a majority of the class's evaluation inputs; a positive result below that is reported as *marginal*. A win on exactly one Real input class is negative. It opens only the deferred PCM experiments in §5, not the open-source compressor path. Approved 2026-11-01 (D-05). Validates FR-5, FR-6, FR-7, FR-14, FR-15.
- **SM-2: Lossless round-trip.** 100% of benchmark inputs and test inputs (including empty, shorter than one Block, partial final Block and incompressible) decode byte-identically. Validates FR-1, FR-2, FR-9, FR-13, FR-14.
- **SM-3: No regression on incompressible data.** On already-compressed inputs, Total encoded size does not exceed the N + 11 bound of FR-10. Validates FR-6, FR-10.

**Secondary**
- **SM-4: Reproducibility.** Rerunning the benchmark from the Corpus manifest on a clean checkout, on any supported platform and thread count, yields identical Total encoded sizes. Validates FR-12, FR-14, FR-15.
- **SM-5: Spec sufficiency.** A clean-room decoder, written only from the spec document, normative tables and test vectors, decodes generated valid Bitstreams and rejects the rejection vectors. Someone other than the author writes it (or a fresh, sandboxed AI agent from a model family other than the one that helped write the kernel), in a language other than Rust or Python, and logs every ambiguity as a spec issue. It runs at spec freeze, before the first public release and on every format version bump. The author's own Python reference decoder does not satisfy SM-5 on its own. Validates FR-2, FR-9, FR-11.

**Counter-metrics (do not optimize)**
- **SM-C1: Synthetic-signal compression.** Large Gain on synthetic signals proves only that the encoder can find a matching Formula; it is not evidence for the research answer. Counterbalances SM-1.
- **SM-C2: Codebook fit to the benchmark corpora.** Tuning the Codebook or the model on the same inputs that decide SM-1 inflates the result. Tuning uses tuning inputs only, content-disjoint from evaluation inputs (FR-15), and model constants are scored on the Baselines-only arm (FR-8). Counterbalances SM-1.
- **SM-C3: Encoder speed.** Do not trade rate-measurement fidelity (exhaustive search, actual-size selection) for speed in v1, even when the FR-14 time budget is tight. Counterbalances SM-1 and the FR-14 budget.

## 8. Open Questions

Resolved on 2026-10-31 and carried in the spec (`_bmad-output/specs/spec-serexpai/`): the Probability model (adaptive order-0, rebuilt by the decoder, nothing transmitted), the implementation language (Rust), the corpora (`corpora.md`), the licence and contribution model (§6.1), the Block lengths and partition rule (FR-5), the Codebook cap (FR-4), the integrity checksum (FR-9), the encode time budget (FR-14), and the numeric bounds (FR-10, FR-13). The entropy coder was also settled as a binary range coder (FR-8).

Resolved on 2026-11-01: SM-1's growth path is a Gain of at least 1% of the class's Total encoded size and wins on a majority of that class's evaluation inputs (former Open Question 8). A win on exactly one Real input class is negative and opens only the deferred PCM experiments, not the open-source compressor path (former Open Question 7). On a 24-hour budget miss, the corpus shrinks first, frozen evaluation inputs stay fixed, and the Codebook cap is lowered second; changing evaluation inputs is a new experiment. Stored trigonometric cw is at most 29 and recurrence cw is at most 25, so the evaluator ranges stay representable. The cw used inside those caps is still measured (Open Question 2).

Still open:

1. **Deferred, not blocking v1:** does lookup retrieve fully parameterized Formulas (needs a coverage strategy) or candidate families fitted locally (needs a fitting procedure and compute budget)? v1 uses the first (FR-4).
2. Which values win on tuning inputs for the measured constants: the model's count cap, rate schedule and initial probability, the coefficient width inside the approved caps (trigonometric cw at most 29, recurrence cw at most 25), and the trigonometric frequency width? The strawman initial probability 32768 and count cap 24 are not the contract. Owner: Darian, before spec freeze. Left unfrozen on 2026-11-01.
3. Is K = 1 KiB the right threshold for the tuning/evaluation overlap checker (FR-15)? Owner: Darian, before the first benchmark run. Left proposed on 2026-11-01: exploratory checks may use 1 KiB, and a reportable evaluation stays blocked until K is confirmed.
4. What is the Codebook's composition within the cap (Formulas per family, Q values, parameter ranges)? It comes from a documented, integer-only recipe tuned on tuning inputs; the recipe is still to be written. Owner: Darian, before the first benchmark run. Left unwritten on 2026-11-01. Tests use a labelled fixture recipe only.
5. Do the LJ Speech section prefixes run LJ001 to LJ050, as the tuning and evaluation split assumes? Confirm against the dataset's metadata before fixing the sample seed. Owner: Darian, before the first benchmark run. Left unconfirmed on 2026-11-01, until `metadata.csv` is checked. The Debian 13 package for the modern tuning executable is also still unnamed.
6. Legal review before the first public release: counsel reviews the adaptive probability update against live adaptive-coding patents (Qualcomm US 10,554,988 and US 10,148,961; Tencent US 12,348,780), plus general clearance. A permissive licence cannot grant rights to third-party patents, so open-source users would be exposed too. Owner: Darian, before the first public release.
7. **Resolved 2026-11-01.** A win on exactly one Real input class is negative. It opens only the deferred PCM experiments (byte-lane separation), not the open-source compressor path. The LJ Speech audio was sourced from 128 kbps MP3, which may flatter smooth predictors; that caveat still belongs next to a PCM result.
8. **Resolved 2026-11-01.** The growth-path bar is a Gain of at least 1% of the class's Total encoded size and wins on a majority of that class's evaluation inputs. A positive result below that is marginal.

## 9. Assumptions Index

- §2.1: the eventual open-source audience is large.
- §2.2: not closing off embedding as a library later.
- §2.3 UJ-3: a reviewer (Mei) reproduces the result from the spec and manifest.
- §4.2 FR-4: v1 Codebook holds fully parameterized Formulas; no local fitting.
- §4.2 FR-4: encode cost fits the FR-14 budget (an estimate until the first measurement).
- §4.7 FR-14: per-class aggregation is Total encoded size summed over the class's inputs.
