---
id: SPEC-serexpai
companions:
  - glossary.md
  - acceptance-criteria.md
  - corpora.md
  - ../../planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md
  - ../../planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md
  - ../../planning-artifacts/prds/prd-serexpai-2026-10-30/addendum.md
sources:
  - ../../planning-artifacts/prds/prd-serexpai-2026-10-30/prd.md
  - ../../planning-artifacts/briefs/brief-serexpai-2026-10-30/brief.md
  - ../../planning-artifacts/briefs/brief-serexpai-2026-10-30/addendum.md
  - ../../planning-artifacts/research/open-questions-research-2026-10-31.md
  - ../../planning-artifacts/research/rans-patent-claim-chart-2026-10-31.md
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability only — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# SerExpAI v1 (repo `serexpai`)

## Why

A **vision to realize** and a **pain to solve** for one person, the author (researcher and builder). SerExpAI is the product: a lossless CLI compressor. MBSC (Multi-Basis Series Compression System) is the idea it represents. SerExpAI predicts each byte of a Block from a compact formula (polynomial, trigonometric, recurrence) and entropy-codes the modular residual with a binary range coder. It is apparently novel; its central claim is untested: that formulas, once parameter and modelling overhead are paid, leave residuals cheap enough to beat simple baselines on real byte streams. v1 is a rate-measurement experiment wrapped in a usable CLI, and must give one defensible answer: does the best formula reduce Total encoded size against constant, previous-byte and direct-entropy baselines on real inputs? Positive result: retrieval, format-aware preprocessing and SIMD entropy coding become worthwhile and the project grows toward an open-source compressor with a published Bitstream spec. Negative result: the repo stays as a reproducible negative result. An unreproducible or ambiguous outcome is the only unacceptable one.

## Capabilities

- **CAP-1**
  - **intent:** The system reconstructs every byte exactly from a predicted byte plus a modular Residual, for any deterministic Predictor.
  - **success:** Decoding the Residual yields y for all 65,536 (y, ŷ) pairs, and a property test over random Blocks and Predictors round-trips with zero mismatches.
- **CAP-2**
  - **intent:** Encoder and decoder compute identical ŷ for any valid Formula parameters via one Normative Evaluator, with a distinct parameter layout per Series family.
  - **success:** Published test vectors (parameters, Block length, ŷ sequence), including every limit boundary and a negative halfway case, pass on x86-64 and aarch64, and an independently written evaluator matches them; out-of-range parameters are rejected, never clamped.
- **CAP-3**
  - **intent:** The encoder splits input into Blocks and exhaustively evaluates every Candidate of a versioned, reproducible Codebook (all three families) for each Block, with Block length chosen from {256, 1024, 2048} by a deterministic partition rule.
  - **success:** For a given input and Codebook the search result is identical across runs and thread counts; each fixed Block length can also be run alone; the final partial Block is coded without padding beyond its signalled length.
- **CAP-4**
  - **intent:** The Rate selector picks, per Block, the Candidate with the smallest actual encoded size, with Raw Block winning unless strictly smaller.
  - **success:** A property test shows the chosen Candidate's exact encoded size is ≤ every other evaluated Candidate covering the same span; Rate is reported in bits per input byte (raw threshold 8) or as `S_total / (8N)`, never mixed.
- **CAP-5**
  - **intent:** Constant, previous-byte and direct byte entropy coding run as Candidates through the same framing and Probability model as Formulas.
  - **success:** Each Baseline can be forced for a whole input and its Total encoded size reported alone; the Baselines-only arm is selectable.
- **CAP-6**
  - **intent:** Residuals (and bytes under direct coding) are coded with exactly one fully specified causal Probability model and one fully specified binary range coder.
  - **success:** The spec states model signalling, frequency normalization, coder initialization, renormalization byte order and termination for the range coder; an independent decoder reproduces the model and coder test vectors; the per-Block fixed cost (header and coder flush) and the model's adaptation overhead are measured and reported; a node saturated by a long run of 0 bits and one saturated by 1 bits give coder frequencies for the repeated bit that differ by at most 1/4096 (no asymmetric ceiling).
- **CAP-7**
  - **intent:** The Bitstream is self-describing, unambiguous and integrity-checked: versioned, with defined bit and byte order, header parsing, Raw Block length, framing, termination and a whole-file CRC-32.
  - **success:** Empty input, input shorter than one Block, exactly one Block, and a partial final Block all round-trip; a decoder rejects every malformed case listed in the spec seed's reject set, including a CRC mismatch, so a flipped bit in Formula parameters or a Raw Block is detected.
- **CAP-8**
  - **intent:** MBSC never expands an input beyond a documented overhead, via Whole-file raw mode.
  - **success:** A test asserts output ≤ N + 11 bytes on random and already-compressed inputs (format versions 1–14).
- **CAP-9**
  - **intent:** The repo contains a standalone document defining the Bitstream and Normative Evaluator, with golden vectors and an in-repo reference decoder.
  - **success:** A clean-room decoder, written only from the document, LUT and vectors by someone other than the author (or a fresh sandboxed AI agent of a different model family) in a language other than Rust or Python, decodes generated valid Bitstreams and rejects the rejection vectors, with every ambiguity it hit logged as a spec issue; kernel, Python reference decoder and clean-room decoder agree under differential fuzzing on both platforms. This is the publication gate: required at spec freeze, before public release and on every format version bump, never blocking the benchmark.
- **CAP-10**
  - **intent:** The user can compress a file to a Bitstream file with `compress`.
  - **success:** Same input and Codebook yield byte-identical output; an I/O error exits non-zero with a message and leaves no valid-looking partial output.
- **CAP-11**
  - **intent:** The user can restore the original bytes with `decompress`.
  - **success:** Output is byte-identical for every Input class; truncated, mutated or malformed Bitstreams exit non-zero with a message and leave no output file; decoder heap working set stays ≤ 64 KiB independent of N.
- **CAP-12**
  - **intent:** The user can run `benchmark` over the Corpus manifest and read, in one report, whether MBSC achieved Gain per Real input class.
  - **success:** Per input, the report gives actual Bitstream size for each whole-input Baseline, Baselines-only arm, full MBSC and Whole-file raw, every round-trip verified (a mismatch fails the run); verdict is positive, marginal, inconclusive or negative per SM-1; the report holds the size breakdown, per-family and per-Baseline win shares, zero-Residual share per class, a deterministic work count (model decisions and coder steps per input byte per arm), and per-Block Candidate-cost sidecar, with no wall-clock or host-dependent values (those go to a separate timing log). A full run completes within 24 h on one named host.
- **CAP-13**
  - **intent:** Benchmark inputs are defined and reproducible: a Corpus manifest of real inputs plus synthetic signals, per `corpora.md`.
  - **success:** The manifest names text, 8-bit PCM, executable and already-compressed control inputs, each with source, licence, SHA-256, role and any documented corpus-side transform (with source and derived hashes); a checker proves tuning and evaluation inputs share no contiguous run of K bytes or more; synthetic signals regenerate from a documented seed; a clean checkout reproduces identical Total encoded sizes.

## Constraints

- The decoder needs neither the Codebook nor floating point, and the normative kernel has zero third-party runtime dependencies and no unsafe code; the decode path never reaches encoder, bench or CLI-side search code.
- Size means the real Bitstream byte length, headers, parameters, model signalling, coder flush and framing included; histogram entropy alone is never a size.
- Every Candidate is encoded through one kernel `encode_block`, so Gain is attributable to the Predictor alone.
- Formulas depend only on Block-local position and their own parameters; Blocks are self-contained and the model resets at each Block start.
- The Probability model is causal: a decoder never selects a model from the symbol being decoded. Its integer arithmetic is defined in u32 or i64; every product is below 2³², so i32 arithmetic and signed 32-bit shifts are forbidden. v1 has no interleaved or SIMD coding, no hardware pipelining and no context modelling beyond the one model.
- Synthetic signals are never merged into a Real input class. Codebook and model constants are tuned on `tuning` inputs only; SM-1 is decided on `evaluation` inputs only; tuning and evaluation inputs are content-disjoint, not merely hash-distinct. Model constants are tuned on the Baselines-only arm so they cannot favour Formula residuals; Formula widths are tuned on full MBSC; ties go to the simpler setting (smaller CAP, DELTA = 1, scalar initial probability).
- Corpus-side transforms (such as 16-to-8-bit PCM) are corpus preparation, documented and hashed; the compressor itself sees bytes only. The fetch tool is Python standard library only, which excludes FLAC- or zstd-packed sources.
- Encoder output is deterministic and independent of thread count; selection uses exact size. No heuristic or entropy screen may change which Candidate wins; a lower-bound prune is allowed only with a written proof that it is admissible.
- Any change to the Evaluator, sine LUT, RECIP table, model constants, coder, checksum or Bitstream layout bumps the format version and regenerates the golden vectors in the same change.
- Targets are Linux x86-64 and aarch64 with a pinned Rust toolchain and committed `Cargo.lock`.
- Licensing: code and Python decoder MIT OR Apache-2.0; spec document CC BY 4.0; vectors, LUT and RECIP table CC0; contributions under DCO sign-off, no CLA, with a human having read, verified and decided every AI-assisted change. The spec cites Martin (1979) and the public-domain LZMA coder as range-coder prior art, and Duda (2013) and ryg_rans (2014) for the informative rANS appendix, and adds no signalled selective-flush feature.
- Format v1 uses an LZMA-style binary range coder, behind one kernel interface and not signalled in the stream. rANS is kept only as an informative appendix of the spec seed: it showed no rate gain, and adopting it later requires a freedom-to-operate review of Microsoft US 11,234,023 B2 and its family plus a format version bump.
- Commercial use is planned. Before the first public release, counsel reviews the adaptive probability update against the live adaptive-coding patents the claim chart found (Qualcomm US 10,554,988 and US 10,148,961; Tencent US 12,348,780), plus general clearance.
- Patent safeguards bind every MBSC decoder: per decision it takes the coder frequency, decides the bit, updates the state for that bit, refills until the state is back in range, then updates the model, with no deferred update, no pending-symbol state and no test gating whether a decision is decoded. No coder state crosses a Block boundary, and the format has no field choosing to flush or retain coder state, a symbol width or a probability table. Changing any of this requires a new freedom-to-operate review.

## Non-goals

- Production-grade compression or competing with zstd/xz on ratio or throughput; no throughput targets until a work model names the denominator.
- A candidate-retrieval index (VP-tree, ANN, LSH). Any later index must first be validated against exhaustive search, measuring recall and excess compressed bytes.
- SIMD or interleaved entropy coding; context-adaptive modelling beyond the one Probability model.
- Format-aware preprocessing (parsed WAV channels, byte-lane separation, symbolic text); each is a separate later experiment and must be reversible and fully signalled.
- Local parameter fitting from candidate initializations; v1 Codebook holds fully parameterized Formulas.
- A stable library API or Bitstream compatibility between 0.x versions; stdin/stdout streaming; per-Block checksums or cryptographic integrity.

## Success signal

- **SM-1:** on `evaluation` inputs, full MBSC's summed Total encoded size is smaller than both the Baselines-only arm and the best whole-input Baseline on at least two distinct Real input classes (any reduction counts; a gain from switching among Baselines does not). Fewer than two testable Real classes means *inconclusive*, not negative. Going ahead with the growth path also needs, in each of the two classes, Gain of at least 1% of the class's Total encoded size and wins on a majority of its evaluation inputs; a positive result below that is *marginal*. A win on exactly one Real class is negative and opens only the deferred PCM experiments, not the open-source compressor path. Approved 2026-11-01.
- **SM-2:** 100% of benchmark and test inputs, including empty, sub-Block, partial-final-Block and incompressible, decode byte-identically. **SM-3:** already-compressed inputs never exceed the CAP-8 bound. **SM-4:** rerunning the benchmark from a clean checkout yields identical Total encoded sizes. **SM-5:** an independent decoder passes (CAP-9).
- Counter-metrics, not to optimize: synthetic-signal Gain (proves only the encoder can match a Formula), Codebook fit to the evaluation corpora, and encoder speed.

## Assumptions

- SerExpAI is the product name. MBSC (Multi-Basis Series Compression System) is the idea it represents. The repository is `serexpai`. Cargo crates use the prefix `sxp-`.
- The eventual open-source audience is large (author's wording); v1 should not close it off, including later library embedding.
- Block lengths 256/1024/2048 come from the architecture review. Magic `0x8D`, the 4-bit version (0 invalid, 15 reserved), the 5-bit Block header, the 1025-entry Q15 sine LUT and the 4-byte per-Block coder flush are confirmed by the 2026-10-31 research and freeze with the spec.
- Gain aggregates summed Total encoded size per class. The best whole-input Baseline (chosen per input) is a consistency check only: the Baselines-only arm offers a superset of its Candidates under the same exact-size selection and is never larger, so SM-1's "smaller than both" is equivalent to "smaller than the Baselines-only arm".
- Encode-cost figures (about 1–5 ms per input byte per core after exact speed-ups) are work-model estimates, not measurements.
- `docs/mbsc_architecture_review.md` was not re-read; its content reaches this spec through the brief addendum, PRD and architecture spine.

## Open Questions

- Which values win on `tuning` inputs under the agreed procedure: count cap (24, 32, 60, 128 or 255), RECIP schedule (DELTA 0.5 or 1), initial probability (scalar or 8-entry per-depth table), the coefficient width inside the approved caps (trigonometric `cw` at most 29, recurrence `cw` at most 25), and trigonometric ω width? Strawman `p = 32768` and cap 24 are not the contract. Left unfrozen on 2026-11-01.
- Is K = 1 KiB right for the overlap checker? Left proposed on 2026-11-01. Exploratory checks may use 1 KiB. A reportable evaluation stays blocked until K is confirmed.
- Do LJ Speech prefixes run LJ001–LJ050 as assumed (confirm against `metadata.csv` before fixing the sample seed)? Left unconfirmed on 2026-11-01. The Debian 13 package for the modern tuning executable is also still unnamed.
- What is the Codebook's composition within the cap (Formulas per family, Q values, parameter ranges)? It comes from the documented integer-only recipe, tuned on `tuning` inputs. Left unwritten on 2026-11-01. Tests use a labelled fixture recipe only.

Resolved 2026-11-01: the growth path is Gain of at least 1% of the class Total encoded size and wins on a majority of that class's evaluation inputs. A one-class win is negative and opens only the deferred PCM experiments. Stored trigonometric `cw` is at most 29 and recurrence `cw` is at most 25. On a 24-hour miss, shrink the corpus first without changing frozen evaluation inputs, then lower the Codebook cap.
