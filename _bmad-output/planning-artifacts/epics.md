---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
revisionStatus: acceptance-criteria-review-proposed
revisionDate: 2026-11-01
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/prd.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md
  - _bmad-output/specs/spec-serexpai/SPEC.md
  - _bmad-output/specs/spec-serexpai/corpora.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md
---

# SerExpAI - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for SerExpAI, decomposing the requirements from the PRD, UX Design if it exists, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

FR-1: Modular residual reconstruction. For every input byte y and every predicted byte ŷ in 0..255, the encoder computes the Residual δ = ((y − ŷ + 128) mod 256) − 128 and the decoder reconstructs y = (ŷ + δ) mod 256. The coded symbol is δ mod 256. Under direct entropy coding the coded symbol is the byte itself. A property test over random Blocks and random Predictors round-trips with zero mismatches. The same two kernel functions serve encoder and decoder.

FR-2: Normative Evaluator semantics. One fixed-point Normative Evaluator, living only in the kernel and called by the encoder, produces identical ŷ for any valid parameters on Linux x86-64 and aarch64. The spec states intermediate integer widths (signed 64-bit), overflow behaviour (impossible by the limit tables; a stored value outside its range is rejected, never clamped or wrapped), rounding including negative halfway cases (toward +∞), division and shift semantics (arithmetic floor shifts; no division other than rem_euclid), trigonometric argument reduction and lookup-table interpolation, coefficient limits, recurrence initialization, and mathematical non-negative modulo for negative inputs. Position input is u = n << 5, always scaled by the maximum Block length 2048, never by the actual Block length. The decoder uses no floating point. A published test-vector set of (parameters, Block length, ŷ sequence) includes a boundary vector and a rejection vector for every limit, plus a negative halfway case, and passes on both platforms.

FR-3: Family-specific parameter layouts. Polynomial stores degree separately from degree + 1 coefficients. Trigonometric stores order separately from 2·order + 1 coefficients, plus one base-frequency phase increment, with limits independent of the polynomial. Recurrence stores order (1 or 2), coefficients, initial values, and its own modulus m_mod, never overloaded as a degree or order. Q for polynomial and trigonometric Formulas is in [8, 24], stored as Q − 8. Counts use a stated offset encoding (stored = count − 1) covering 1 to 16. Signed fields are two's complement. Every field covers the v1 ranges through a defined encoding.

FR-4: Versioned exhaustive Codebook. A Codebook of fully parameterized Formulas from the polynomial, trigonometric and recurrence families is defined, versioned and reproducible from a documented integer-only recipe, so a reviewer can regenerate it. It holds at most 4096 Formulas and at most 2048 per family. Codebook identity (version and content hash) is recorded in the benchmark report. The decoder never reads the Codebook. Local parameter fitting is out of scope. The recipe, Formulas per family, Q values and parameter ranges are still to be written and are tuned only on tuning inputs before the first benchmark run.

FR-5: Exhaustive per-Block search and deterministic partition. For each Block the encoder evaluates every Codebook Formula and every Baseline. The search result is deterministic for a given input and Codebook and identical at any thread count. Block length is one of 256, 1024 or 2048 bytes, signalled per Block, and the Rate selector chooses among them. The partition rule is deterministic, selects on exact size, and is stated in the spec. The benchmark can also run each fixed Block length alone. Each Block decodes from its own header and payload alone: Formulas use Block-local positions, and no model or coder state crosses a Block boundary. The final partial Block is handled without padding the output beyond its signalled length.

FR-6: Rate selector on actual encoded size. For each Block, the chosen Candidate's actual encoded size is less than or equal to every other evaluated Candidate covering the same span. The encoder never estimates size; it asks the kernel for the exact size of the Block it would write. A Raw Block wins whenever compressed size is not strictly smaller, framing included on both sides. On equal size, a Baseline beats a Formula, and Whole-file raw mode beats everything, including the empty input, so Gain is never awarded to a Formula on a tie. Rate is reported as bits per input byte (raw threshold 8) or as S_total / (8N), never mixed. v1 has no entropy screen or other heuristic that can change which Candidate wins, so compress produces the same bytes as the full-MBSC benchmark arm. Speed-ups that provably cannot change the selection are allowed.

FR-7: Baseline Predictors and direct entropy coding. The encoder supports constant prediction (a per-Block constant byte in the Bitstream), previous-byte prediction (preceding byte; first byte of a Block predicted as 0) and direct byte entropy coding as Candidates on the same Probability model and framing as Formulas. Each Baseline can be forced for a whole input through benchmark, and its Total encoded size reported alone. A forced-Baseline arm still uses Raw Blocks, the same partition rule and the Whole-file raw check. The Baselines-only arm (Formulas disabled; Baselines, Raw Blocks and Whole-file raw remain; partition applied independently) is selectable through benchmark.

FR-8: One causal Probability model and a hand-written binary range coder. Residuals and direct-coded bytes are coded with one adaptive order-0 model, reset at every Block start, and a binary range coder. Each symbol is 8 binary decisions, most significant bit first, through a 255-node bit tree. The model is reconstructed only from already-decoded symbols; nothing about it is transmitted; a decoder never selects a model from the symbol it is decoding. Model update arithmetic is defined in u32 or i64 (every product below 2^32); i32 arithmetic and signed 32-bit shifts are forbidden. The coder total is the fixed constant 4096; freq0 = max(p >> 4, 1) and freq1 = 4096 − freq0. The coder is LZMA-style (32-bit range, carry via cache, forward order, 4-byte flush per Block), not signalled in the stream, and kept behind one kernel interface (encode decisions, decode decisions, exact size). Interleaved or SIMD entropy coding is not used. rANS is not used in v1. Model constants (count cap, RECIP schedule, initial probability) are tuned on tuning inputs only and scored on the Baselines-only arm; Formula coefficient and frequency widths are scored on full MBSC; ties go to the simpler setting; the tuning grid and every score are recorded. The per-Block fixed cost and the model's adaptation overhead are measured and reported. Patent safeguards bind every decoder: per decision, take freq0, decide the bit, update the state, refill until the state is back in range, then update the model, with no deferred update, no pending-symbol state and no test that gates whether a decision is decoded. No coder state crosses a Block boundary, and the format has no field for coder-state retention, symbol width or probability tables.

FR-9: Complete Bitstream contract. The Bitstream has one normative layout: little-endian multibyte integers, LSB-first packed fields, zero padding that is checked, a magic byte 0x8D, a format version (v1 = 1; 0 invalid; 15 reserved), a mode (Blocked or Whole-file raw), N as a minimal LEB128 with N < 2^35, per-Block headers, and a mandatory 4-byte CRC-32/ISO-HDLC trailer of the decoded bytes. A raw flag is accompanied by an unambiguous Raw Block length. Payload length is derived, not stored. A decoder rejects a CRC mismatch, an unknown version, a non-canonical length, trailing bytes, and the other malformed cases in the spec seed reject set. Any change to the Normative Evaluator, normative tables, model constants, coder, checksum or layout bumps the format version and regenerates the test vectors in the same change. A Codebook change does not. The empty input, an input shorter than one Block, an input of exactly one Block, and a partial final Block all round-trip.

FR-10: Bounded raw fallback. The Bitstream is at most N + 11 bytes for format versions 1 to 14, asserted on random and already-compressed inputs. Whole-file raw mode makes the bound a small constant (header + N bytes + CRC), not a per-Block cost. Whole-file raw wins ties, including the empty input, which is the 7-byte Whole-file raw stream. Blocked mode with N = 0 is rejected.

FR-11: Standalone Bitstream and Normative Evaluator document. The repository includes spec/bitstream.md, sufficient with the test vectors, sine LUT and RECIP table for an independent decoder. An in-repo Python reference decoder (standard library only), written from the document and not translated from the Rust, plus a generator of valid Bitstreams and mutated copies, covers every reject rule and limit. The Rust kernel and the Python decoder agree on every vector on both platforms. Before spec freeze this check runs in CI but does not block the benchmark; it must pass at spec freeze and before any public release. The document is published under CC BY 4.0; test vectors, the LUT and the RECIP table under CC0. The author's Python decoder does not by itself satisfy the clean-room sufficiency bar (SM-5).

FR-12: compress. The user can compress a file to a Bitstream file. The same input and Codebook yield the same Bitstream, independent of thread count. An I/O error exits non-zero with a message and does not leave a partial output file that looks valid. The command refuses inputs of 2^35 bytes or more with a non-zero exit. Success writes a temporary file in the target directory and renames it into place.

FR-13: decompress. The user can decompress a Bitstream file to the original bytes. Round-trip output equals the original file byte for byte for every Input class. A truncated, corrupted or malformed Bitstream is rejected with a non-zero exit and a message, and leaves no output file. The decoder never reads out of bounds. Its heap working set is at most 64 KiB, independent of N. Parsing and decoding return Result and cannot panic on any byte sequence.

FR-14: benchmark. The user can run the benchmark over the Corpus manifest and get a per-class report of Total encoded size (the byte length of the real Bitstream) for each whole-input Baseline, the Baselines-only arm, full MBSC and Whole-file raw mode. Every round-trip is verified by decoding through the kernel; a mismatch fails the run. Results aggregate per Input class as summed Total encoded size. The verdict line states, per Real input class, whether Gain was achieved, with per-input wins and the margin. The overall verdict is positive, marginal, inconclusive or negative as defined by SM-1. Synthetic signals stay in their own section. The report breaks size into header, Formula parameter, coder payload and coder flush, reports adaptation overhead separately, and gives the share of Blocks and bytes won by each Series family and each Baseline, plus the share of zero Residuals per class. It records Codebook version and hash, Block lengths, tool version, Corpus manifest hash, tuning-input hashes and the overlap checker's result. It gives a deterministic Work count per arm (model decisions and coder steps per input byte). Wall-clock time and host details go to a separate timing log, excluded from determinism checks. A sidecar records the encoded cost of every evaluated Candidate per Block, written by default for inputs up to 4 MiB and on request above that; the report states which inputs have one. A full run completes within 24 hours on one host named in the timing log before the first full run. If it does not, the corpus shrinks first (keeping the per-class minimum), then the Codebook cap is lowered, and the report states the excess bytes this costs on tuning inputs.

FR-15: Corpus manifest and synthetic signals. The Corpus manifest (corpora/manifest.toml) names text, 8-bit PCM and executable inputs, plus already-compressed data as a control, each with source, licence, content hash, role (tuning or evaluation) and any Corpus-side transform. Each Real input class has at least three evaluation inputs. Synthetic signals matching each Series family are generated by integer-only code from a documented seed and are never merged into a Real input class. Corpora that cannot be redistributed are referenced by hash and source, not committed. Codebook tuning uses only tuning inputs. SM-1 is decided on evaluation inputs only. A checker proves that no tuning input shares a contiguous run of K bytes or more with any evaluation input (K proposed as 1 KiB). A failure blocks the benchmark until the tuning input is replaced; evaluation inputs never change. Distinct hashes alone do not suffice. The roster, slice rules and corpus-side transforms are the ones in corpora.md.

### NonFunctional Requirements

NFR-1: Lossless round-trip (SM-2). 100% of benchmark inputs and test inputs, including empty, shorter than one Block, partial final Block and incompressible, decode byte-identically.

NFR-2: No expansion on incompressible data (SM-3). On already-compressed inputs, Total encoded size does not exceed the N + 11 bound.

NFR-3: Reproducibility (SM-4). Rerunning the benchmark from the Corpus manifest on a clean checkout, on either supported platform and at any thread count, yields identical Total encoded sizes. rust-toolchain.toml pins the Rust version and Cargo.lock is committed.

NFR-4: Spec sufficiency (SM-5). At spec freeze, before the first public release, and on every format version bump, a clean-room decoder written only from spec/bitstream.md, the LUT, the RECIP table and the vectors — by a different person, or a fresh sandboxed agent of a different model family, with prompt and model ID recorded, in a language other than Rust or Python — decodes generated valid Bitstreams and rejects the rejection vectors, logging every ambiguity as a spec issue. CI also runs three-way differential fuzzing (kernel, Python reference decoder, clean-room decoder): all decode identically or all reject. This gate must not block the benchmark.

NFR-5: Research answer (SM-1). On evaluation inputs, full MBSC achieves Gain (any reduction in summed Total encoded size versus the Baselines-only arm) on at least two distinct Real input classes. A reduction that comes only from switching among Baselines does not count. The best whole-input Baseline is reported as a consistency check. If fewer than two Real input classes can be tested, the result is inconclusive, not negative; otherwise fewer than two classes with Gain is negative. Going ahead with the growth path also needs, in each of the two classes, a Gain of at least 1% of the class's Total encoded size and wins on a majority of that class's evaluation inputs; a positive result below those numbers is marginal. A win on exactly one Real class is negative and opens only the deferred PCM experiments, not the open-source compressor path. Approved 2026-11-01 (D-05).

NFR-6: Counter-metrics are not optimization targets. Synthetic-signal Gain is not evidence for the research answer. The Codebook and model constants are not tuned on the inputs that decide SM-1. Encoder speed is not bought by dropping exhaustive search or exact-size selection, even when the 24-hour budget is tight.

NFR-7: Supported platforms are Linux x86-64 (x86_64-unknown-linux-gnu) and aarch64 (aarch64-unknown-linux-gnu). Published vectors pass on both.

NFR-8: v1 sets no throughput target and does not compete with zstd or xz on ratio or speed. The only time bound is FR-14's 24-hour full-benchmark budget, met by shrinking the corpus and then the Codebook, not by changing which Candidate wins.

NFR-9: Decoder safety. The kernel's parsing and decoding entry points cannot panic on any byte sequence. clippy::unwrap_used, clippy::panic and clippy::indexing_slicing are denied in the kernel. Heap working set during decode is at most 64 KiB independent of N, asserted with a counting allocator. Malformed input exits non-zero, writes a message on stderr, and leaves no output file.

NFR-10: Licence, contribution and commercial posture. Code and the Python reference decoder are MIT OR Apache-2.0. The spec document is CC BY 4.0. Test vectors, the sine LUT and the RECIP table are CC0. Contributions use DCO sign-off with no CLA. A human reads, verifies and decides every AI-assisted change. v1 must not close off later commercial use or later library embedding. Before the first public release, counsel reviews the adaptive probability update against Qualcomm US 10,554,988 and US 10,148,961 and Tencent US 12,348,780, plus general clearance. Changing the decoder's per-decision structure, or adding a field for coder-state retention, symbol width or probability tables, needs a new freedom-to-operate review first. Adopting rANS later needs a freedom-to-operate review of Microsoft US 11,234,023 B2 and its family, plus a format version bump.

NFR-11: CLI errors. compress and decompress exit 0 on success and 1 on any error, with a message on stderr. There is no logging framework and no config file. The benchmark report goes to stdout and a JSON file. Manifests are TOML.

NFR-12: v1 promises no stable library API and no Bitstream compatibility between 0.x versions. A decoder rejects an unknown format version. There is no stdin/stdout streaming, no per-Block checksum, and no cryptographic integrity. The whole-file CRC detects accidental corruption only.

### Additional Requirements

- **Greenfield layout, no external starter template.** Epic 1 Story 1 creates the Cargo workspace from the architecture structural seed, not from a third-party template. Layout: workspace Cargo.toml, committed Cargo.lock, rust-toolchain.toml pinning Rust 1.99.1 edition 2024, ci.sh, crates/sxp-kernel, crates/sxp-encoder, crates/sxp-bench, crates/sxp-cli, spec/bitstream.md, spec/sine_lut.* (normative LUT plus hash), spec/vectors/, tools/refdec/ (Python reference decoder, stdlib only), tools/fetch/ (hash-verified corpus fetch, stdlib only), corpora/manifest.toml. Deployment is a local CLI built with cargo build --release and distributed as source. There is no service. The only network access is the corpus fetch tool.
- **Crate names and dependency direction.** Cargo crates use the prefix `sxp-`. The architecture spine's consistency table is aligned to that prefix. `sxp-kernel` owns the Normative Evaluator, sine LUT, modular residual math, Probability model, binary range coder, Block encode/decode, and every Bitstream read and write primitive, including the file header and Block headers. sxp-encoder owns the Codebook, Baselines, Candidate enumeration, Block-partition DP and Rate selector; it composes kernel writers and never serialises a field itself. sxp-bench owns the Corpus manifest, synthetic signals, benchmark arms and the report. sxp-cli owns compress, decompress and benchmark. Encoder-side code depends on the kernel, never the reverse. The decode path is sxp-cli to sxp-kernel only. sxp-kernel has zero third-party runtime dependencies and #![forbid(unsafe_code)]. CI fails if cargo tree -p sxp-kernel -e normal lists any external crate. Dev-dependencies such as proptest are allowed.
- **Pinned stack.** Rust stable 1.99.1, edition 2024. rayon 1.12.0 in sxp-encoder. clap 4.6.7 in sxp-cli. serde 1.0.229, serde_json 1.0.154, toml 1.1.8 and sha2 0.11.1 in sxp-bench. proptest 1.12.0 as a dev-dependency. Python 3.14 (at least 3.14.4) for the reference decoder and the fetch tool, standard library only, which excludes FLAC- or zstd-packed sources as Real inputs.
- **CI contract.** One local script, ci.sh, runs cargo fmt --check, cargo clippy -D warnings, the tests, the kernel dependency check, and the golden vectors in Rust and in the Python reference decoder, on x86-64 and on aarch64 (native runner or qemu-user). The CI host is not chosen. Before spec freeze, kernel/Python agreement does not block the benchmark; it must be green at spec freeze and before any public release.
- **One coding path.** Formula, constant, previous-byte, direct entropy coding and Raw Block are Candidates of one kernel encode_block. A Candidate differs only in how its ŷ stream is produced. A Raw Block skips prediction and the coder stream. Gain must be attributable to the Predictor alone.
- **Kernel purity.** No global mutable state. Kernel functions are pure and take explicit inputs. Errors are typed Results with plain error enums; only the shell adds context strings. Format terms (Block, Formula, Candidate, Residual, Codebook) match the PRD glossary in code, spec and report. Family fields use the names degree, order, m_mod and q.
- **Polynomial evaluator, exact recurrence.** acc starts at c_d; for k = d−1 down to 0, acc ← ((acc · u) >> 16) + c_k; then (acc + (1 << (Q−1))) >> Q. Each coefficient is in [−2^31 + 1, 2^31 − 1] (stored −2^31 is rejected). Degree is at most 15. With those limits every intermediate stays below 2^52.
- **Trigonometric evaluator.** theta = (n * omega) mod 2^32. S starts at c0 << 15. For k = 1..order, p = (k * theta) mod 2^32 (wrapping 32-bit multiply), and S adds a_k * sin_lut(p) + b_k * sin_lut((p + 2^30) mod 2^32). value = (S + (1 << (Q + 14))) >> (Q + 15). sin_lut uses a 1025-entry table (index i = p >> 22, fraction f = (p >> 6) & 0xFFFF, linear interpolation with a rounding bias of 1 << 15). LUT values are round(32767 · sin(2π i / 1024)) as signed 16-bit, |LUT| ≤ 32767, published with a SHA-256; the generator is not normative. |c0|, |a_k|, |b_k| < 2^28. Stored cw is at most 29 (D-01, approved 2026-11-01). Order is at most 16. omega is 32 bits unsigned. Table size and scale are marked proposed in the spec seed and confirmed by the 2026-10-31 research as freezing with the spec.
- **Recurrence evaluator.** Order 1: x[0] = s0; x[n+1] = rem_euclid(a * x[n] + b, m_mod). Order 2: x[0] = s0; x[1] = s1; x[n+2] = rem_euclid(a * x[n+1] + b * x[n] + c, m_mod). value = x[n], then rem_euclid(256). |a|, |b|, |c| < 2^24, stored cw is at most 25 (D-01, approved 2026-11-01), 2 ≤ m_mod ≤ 2^24, and 0 ≤ s0, s1 < m_mod. It is iterated from n = 0 inside the Block and depends only on n and its own parameters.
- **Block header layout.** A Block is byte-aligned. The 5-bit header is a 2-bit length code (0 = 256, 1 = 1024, 2 = 2048, 3 = reserved and rejected) and a 3-bit kind (0 Raw, 1 Direct, 2 Previous-byte, 3 Constant, 4 Polynomial, 5 Trigonometric, 6 Recurrence, 7 reserved and rejected). Kind codes are proposed wire values; Candidate ids used for tie-breaks do not depend on them. Actual length is min(nominal, remaining). A nominal length is valid only when it is canonical.
- **Kind-specific fields.** Raw: no fields; then the verbatim bytes. Direct and Previous-byte: no fields; then the coder stream. Constant: one byte c. Polynomial: degree (4 bits, 0–15), q − 8 (5 bits, stored 0–16, 17–31 rejected), cw − 1 (5 bits, width 1–32), then degree + 1 coefficients of cw bits, lowest power first. Trigonometric: order − 1 (4 bits, order 1–16), q − 8, cw − 1 (5 bits, stored cw 1–29; 30–32 rejected), omega (32 bits), then c0 and a_k, b_k for k = 1..order. Recurrence: order − 1 (1 bit), m_mod − 2 (24 bits; stored values above 2^24 − 2 rejected), cw − 1 (5 bits, stored cw 1–25; 26–32 rejected), then a, b (order 1) or a, b, c (order 2), then s0 and, for order 2, s1 at 24 bits. Any field outside its range, any s0 or s1 ≥ m_mod, and any coefficient outside the family limit is a rejection. D-01 fixed these caps on 2026-11-01. The cw a production Codebook uses inside the caps is still measured before freeze (D-07).
- **Partition DP.** The encoder runs a one-dimensional DP over every 256-aligned offset using exact Block sizes. At offset o with r = N − o bytes left, allowed transitions are every nominal length L ≤ r, and, when r ≤ 2048, the single smallest nominal length ≥ r. The decoder rejects a non-canonical nominal length and the reserved length code. The Baselines-only arm and fixed-length runs use the same DP with their own allowed length set, applied independently. A final transition is coded with the canonical nominal, the smallest length in {256, 1024, 2048} that is ≥ r, so every run decodes through the same kernel.
- **Tie-break key.** The DP minimises (total size, Block count, lexicographic sequence of (length code, Candidate id)). Candidate ids, independent of wire kind codes: Raw = 0, Direct = 1, Previous-byte = 2, Constant with byte c = 3 + c (all 256 constants are evaluated), then Formulas from 259 by family and Codebook index. A Raw Block wins any tie. Whole-file raw is compared with the whole Blocked output and wins ties.
- **Allowed exact speed-ups.** Precomputed ŷ tables per Formula; one model probability sequence shared across Block lengths at an offset (coder encode stays per Candidate and length); skipping a Candidate whose header and parameter bytes alone reach the incumbent's exact size; reciprocal-table division. A lower-bound prune on coder size is allowed only once a written proof shows it is admissible. The empirical bound exact ≥ ceil((ideal + 23) / 8) is unproven and is not used until proved.
- **Range coder procedure.** Encoder state: low at least 33 bits, range = 0xFFFFFFFF, cache = 0, cache_size = 1. encode uses bound = (range >> 12) * freq0, then the LZMA shift_low carry path. Flush calls shift_low five times. The first emitted byte is always 0x00 and is not written; the flush leaves 4 bytes. Decoder reads code from the first 4 bytes, most significant first, rejects code = 0xFFFFFFFF, and after the last decision requires code = 0. Reference vectors: one decision of bit 0 with freq0 = 2048 yields 00 00 00 00; bit 1 with freq0 = 2048 yields 7F FF F8 00. CRC-32/ISO-HDLC check value for "123456789" is 0xCBF43926.
- **Model constants are not frozen.** Initial p (default strawman 32768, scalar, or an 8-entry per-depth table only if it beats the best scalar; every entry in [16, 65520]), count cap (strawman 24; candidates 24, 32, 60, 128, 255) and the RECIP table (schedules floor(65536 / (n + 2)) and floor(131072 / (2n + 3))) are measured on tuning inputs, scored on the Baselines-only arm, with ties to the simpler setting (smaller cap, DELTA = 1, scalar initial probability). They are published as data with a hash before spec freeze. Formula cw and whether omega may be narrower than 32 bits are measured on full MBSC before freeze, only among layouts that represent the accepted parameter domain. Stories must not treat the strawman numbers as the frozen contract.
- **Reject set, each with a vector.** Bad magic; version 0, 15 or any unknown version; a set reserved bit; mode 2 or 3; mode 0 with N = 0; a non-minimal or out-of-range LEB128; reserved length code or kind; a non-canonical nominal length; non-zero padding; any parameter outside its limit; s0 or s1 ≥ m_mod; a coder stream running past the end of input; initial code = 0xFFFFFFFF; final code ≠ 0; a stream that ends before N bytes are decoded or before the CRC; a CRC mismatch; any byte after the CRC. A coverage table maps every reject rule and every limit to at least one vector.
- **Vector sets required in spec/vectors/.** Evaluator vectors (boundary values for every limit and a negative halfway case); residual hash of all 65,536 (y, ŷ) pairs; model vectors (fresh node, n = CAP, saturation after a long run of 0s and of 1s, RECIP hash); coder vectors (the two 1-decision vectors, the 2048-symbol case, and a carry through at least one pending 0xFF); CRC vectors (the check value, the empty input, one file per mode); one valid Block per kind and length code, including Raw and a partial final Block; files for empty (7 bytes), 1 byte, 255, 256, 257, exactly one 2048-byte Block, a partial final Block, and Whole-file raw.
- **Spec-before-kernel gate.** Before implementation of the kernel starts, the spec defines: magic and version/mode packing, the Block header and kind codes, each family's layout and limit table, the Raw Block layout, range-coder initialization and termination, the CRC trailer, the model constants and RECIP table (or an explicit unfrozen-strawman status), the LUT, the canonical-length constraints, and the v1 partition DP with its tie key (the DP is informative, because any encoder may choose a different valid partition).
- **Benchmark arms.** Whole-file raw has no Candidates (header + N bytes + CRC). Forced Constant, Previous-byte and Direct each add that Baseline plus Raw Block. Baselines-only is all three Baselines plus Raw Block. Full MBSC is the Codebook plus Baselines plus Raw Block. The best whole-input Baseline is the smallest of the three forced arms per input and is reported as a consistency check. Gain for a Real input class is any reduction in summed Total encoded size of full MBSC versus the Baselines-only arm. The Baselines-only arm offers a superset of that Baseline's Candidates under the same exact-size selection, so the arm is never larger and the check can never decide Gain. The SPEC's "smaller than both" wording is equivalent to "smaller than the Baselines-only arm". The report still publishes both figures. SM-1 is decided on evaluation inputs only. The architecture spine's AD-14 matches this rule.
- **Adaptation-overhead definition.** The ideal adaptive code length is the sum of −log2(freq / 4096) over the Block's decisions, minus the Block's zero-order entropy, with the coder flush reported separately.
- **Sidecar layout.** The Candidate-cost sidecar is a u16 matrix indexed by offset, length code and Candidate id. It is written by default for inputs up to 4 MiB (about 100 bytes per input byte) and opt-in above that.
- **Work-count and timing split.** The report's Work count is host-independent. Wall-clock time and host details live only in the timing log. The named host for the 24-hour budget is recorded in that log before the first full run.
- **Corpus roster.** Nothing is vendored. tools/fetch downloads, verifies hashes, and applies corpus-side transforms. Archives are verified in full before extraction; only manifest-listed members are extracted. Where a source publishes only MD5 (Silesia), the fetch tool checks that MD5 on first fetch and the manifest pins SHA-256. Text tuning: Canterbury alice29.txt, lcet10.txt, plrabn12.txt, cp.html, xargs.1, plus Calgary book1, book2, paper1, paper2. Text evaluation: 3 MiB middle slices of Silesia dickens, webster, xml and reymont. PCM tuning: about 30 LJ Speech clips (about 4 MiB) sampled from prefixes LJ001–LJ025. PCM evaluation: about 60 LJ Speech clips (about 8.5 MiB) from LJ026–LJ050. Executable tuning: Calgary obj1 and obj2, Canterbury sum, and one modern pinned Debian 13 dynamically linked package binary for amd64 and arm64 that passes the overlap check and is not busybox-static or ripgrep. Executable evaluation: equal middle slices of Silesia ooffice and of Debian 13 busybox-static and ripgrep, each for amd64 and arm64 (5 inputs). Already-compressed control, evaluation only: FLAC test-bench subset files and enwik8.zip (36,445,475 bytes). Licences are recorded per input, including "none stated".
- **Corpus selection and transforms.** LJ Speech clips are ordered by SHA-256(seed ‖ clip ID) and taken until the role's byte budget; the seed string is recorded in the manifest. Mean clip length is 6.57 s, about 145 KB after conversion. Executable evaluation slice length is the smallest of the five files, rounded down to a multiple of 256 bytes, recorded when the packages are pinned. If five times that length is under 4 MiB, an input is added or replaced until the class reaches 4 MiB. Each .deb must carry data.tar.xz. Middle-slice offset is floor((size − length) / 2). Text and executable evaluation inputs are equal-sized middle slices. PCM evaluation inputs are whole clips. Each Real class is 4 to 16 MiB with at least 3 evaluation inputs. pcm16le-to-u8 v1: mono 16-bit signed little-endian WAV at 22,050 Hz becomes one byte per sample, the high byte XOR 0x80, raw samples with no WAV header. The raw 16-bit sample bytes of the same clips are benchmarked and reported as non-counted rows. slice v1 records transform name and version, source hash, offset, length and derived hash. The report states known within-class correlation (one PCM speaker; the same packages built for two architectures) and the LJ Speech caveat (sourced from 128 kbps MP3; content-disjoint but not speaker-disjoint).
- **Overlap policy.** K = 1 KiB is proposed, not confirmed. Evaluation inputs are fixed. A tuning input that shares a run of K bytes or more with any evaluation input is dropped and replaced by another Calgary or Canterbury input that passes. No stripping, no moving evaluation slices, no allowlist. enwik8 is the first 10^8 bytes of enwik9, so distinct hashes are not a sufficient check.
- **Gates the owner must close, which stories must not invent.** Confirm LJ Speech prefixes LJ001–LJ050 against metadata.csv and fix the sample seed before the first benchmark run. Choose the Debian package that supplies the modern tuning executable before that run. Confirm or replace K = 1 KiB before that run; until then exploratory checks may use 1 KiB and G2 stays blocked. Write the Codebook recipe before that run. Freeze model constants and the chosen coefficient widths before spec freeze. Legal review before the first public release. Closed on 2026-11-01: the growth path is Gain of at least 1% of the class Total encoded size and wins on a majority of that class's evaluation inputs, and a one-class win is negative and opens only the deferred PCM experiments.
- **Informative only, not format v1.** The spec seed's rANS appendix is not implemented in the v1 Bitstream. Adopting it is a later format version plus a freedom-to-operate review.
- **Verification the architecture requires.** Property tests for residual reconstruction and for Rate-selector minimality. Golden, boundary and rejection vectors. Round-trip of empty, shorter-than-one-Block, exactly-one-Block and partial-final-Block inputs. Truncation, mutation, CRC-mismatch and trailing-byte rejection. A determinism test of parallel versus single-threaded encode. A no-expansion test (N + 11) on random and already-compressed inputs. The corpus overlap check. Valid-stream generation and differential fuzzing. A mutation test over valid streams through the kernel and the Python reference decoder.

### UX Design Requirements

None. SerExpAI v1 is a command-line tool. No UX design contract (DESIGN.md / EXPERIENCE.md or a legacy UX spec) is an input to this breakdown.

### FR Coverage Map

FR-1: Epic 1 — Exact residual reconstruction
FR-2: Epic 1 — One normative evaluator, same ŷ on both platforms
FR-3: Epic 1 — Separate parameter layouts per series family
FR-4: Epic 1 — A versioned, regenerable codebook
FR-5: Epic 1 — Exhaustive per-block search and the partition rule
FR-6: Epic 1 — Selection on exact encoded size
FR-7: Epic 1 — Constant, previous-byte, and direct-coding baselines
FR-8: Epic 1 — The order-0 model and the binary range coder
FR-9: Epic 1 — The bitstream the tool writes, including the CRC
FR-10: Epic 1 — The N+11 raw-fallback bound
FR-11: Epic 2 — The standalone spec, the Python reference decoder, and the publication gate
FR-12: Epic 1 — compress
FR-13: Epic 1 — decompress
FR-14: Epic 1 — The per-class benchmark report
FR-15: Epic 1 — The corpus manifest and synthetic signals

## Revision status and interpretation

Owner approvals from 2026-11-01 close D-01, D-02, D-03, D-05, and D-06. D-04, D-07, D-08, and D-09 stay open, and implementation must not invent them. The requirements inventory above is retained for traceability. Where an older sentence still conflicts with the register, the register governs.

Existing story identifiers are retained. Large stories are organized into separately reviewable acceptance groups rather than assigned new story identifiers. Group completion is recorded independently; the parent story completes only when all of its required groups pass. Source-document paths in the front matter identify upstream inputs; this edit does not claim those upstream documents have been updated.

### Decision register

| ID | Decision required | Proposed disposition | Required before |
|---|---|---|---|
| D-01 | Signed Formula coefficient representability | **Approved 2026-11-01.** Evaluator ranges stay `|trig coefficients| < 2^28` and `|recurrence coefficients| < 2^24`. Stored trigonometric cw is at most 29 and recurrence cw is at most 25, so those ranges are representable. Polynomial cw up to 32 still rejects a stored −2^31. The cw a production Codebook uses inside those caps remains a D-07 measurement. | Family serialization and production Codebook acceptance |
| D-02 | Block field alignment | **Approved 2026-11-01.** A 5-bit length/kind header, then kind-specific fields, then checked zero padding to the next byte. The payload begins byte-aligned. Kind codes remain proposed wire values. | Block writer/parser implementation |
| D-03 | Fixed-length final Block | **Approved 2026-11-01.** Full Blocks stay 256, 1024, or 2048. The final partial span uses the smallest nominal length in {256, 1024, 2048} that is ≥ the bytes left. | Partition implementation acceptance |
| D-04 | Corpus owner gates | **Open.** K stays proposed: exploratory checks may use 1 KiB, and G2 stays blocked. LJ Speech prefixes stay unconfirmed until `metadata.csv` is checked. The Debian tuning package stays unnamed until one passes the overlap check. No placeholder closes a gate. | Reportable evaluation freeze |
| D-05 | Research classification | **Approved 2026-11-01.** The growth path needs, in each of two Real classes, Gain of at least 1% of that class's Total encoded size and wins on a majority of that class's evaluation inputs. A two-class result below that is marginal. A win on exactly one Real class is negative. It opens only the deferred PCM experiments, not the open-source compressor path. | First reportable evaluation |
| D-06 | Budget-driven corpus changes | **Approved 2026-11-01.** On a miss, shrink the corpus first, keep each class's minimum, and do not change frozen evaluation inputs; then lower the Codebook cap. Changing evaluation inputs is a new experiment revision. | A reduced-corpus evaluation |
| D-07 | Model and layout freeze | **Open.** Model constants and the chosen coefficient widths stay unfrozen until the tuning measurement exists. Strawman `p = 32768` and count cap 24 are not the contract. | Reportable evaluation and spec freeze |
| D-08 | Legal release clearance | **Open.** Record the required counsel review and its scope; this document makes no freedom-to-operate conclusion. | First public release and review-triggering changes |
| D-09 | Production Codebook recipe | **Open.** The production recipe stays unwritten. Tests use a labelled fixture recipe only. Reportable evaluation still requires an approved recipe, version, and hash. | Production Codebook acceptance and G2 |

Story 1.13 stays one story with four acceptance groups. Where a decision changes an upstream requirement, the PRD, the spec, the spec seed, and this inventory are updated together.

### Gate model

- G0 — Draft normative contract: header packing, modes, Block layouts, legal parameter domains, evaluator arithmetic, coder initialization/normalization/termination, checksum, canonical lengths, reject rules, and normative table identities are documented before the affected kernel implementation. Provisional constants are explicitly labelled and hashed. Encoder partition policy is documented but informative to independent decoders.
- G1 — Implementation verification: relevant story tests pass, exact-size and partition oracles agree, and deterministic vectors pass on Linux x86-64 and aarch64. Early native-only work is permitted, but both-platform evidence is required for story completion where specified.
- G2 — Experiment freeze: an immutable experiment manifest records source/tool revision, toolchain, corpus and transform hashes, overlap policy/result, Codebook recipe/version/hash, model/table/layout hashes, tuning evidence, owner decisions, thresholds, and benchmark protocol. It exists before reportable evaluation starts.
- G3 — Reportable evaluation: the G2 configuration runs without unrecorded changes; all arms round-trip; the deterministic report and separate timing log are complete. Exploratory runs are explicitly unreportable and are not scientific “inconclusive” results.
- G4 — Spec freeze: no unresolved normative constants or format decisions; Python agreement and clean-room verification pass. Private research benchmarking does not require G4.
- G5 — Public release: G4 plus current licensing, human review, and legal clearance evidence pass an executable release-readiness check.

Changes after G2 create a new experiment revision and invalidate affected evaluation results. A negative scientific result can complete Epic 1; missing governance cannot. Failure to meet the timing budget is recorded separately from the scientific verdict and does not authorize heuristic search.

### Acceptance evidence convention

Every AC below has a stable ID. Its test, command, fixture, or review artifact must be linked in the implementation's acceptance record. A test may satisfy several ACs if the mapping is explicit. An acceptance record identifies the tested revision and normative asset hashes.

- Valid-stream fixtures include exact encoded bytes and decoded bytes. Rejection fixtures identify their intended reject-rule ID.
- Cross-platform checks compare bytes and deterministic metrics, not just successful execution. Thread-count checks include 1, 2, and at least one larger available count, recorded in the evidence.
- Safety evidence combines typed errors, static/lint checks, adversarial fixtures, counting-allocator tests, and recorded fuzzing profiles; finite testing is not represented as a proof for every byte sequence.
- Decoder heap accounting includes decoder-owned working buffers and dynamically allocated tables, excludes caller-owned input and streamed output storage, and must not hide full-file allocation inside the decoder. Record the accounting boundary and peak.
- Differential testing accepts either identical decoded bytes or unanimous rejection. Arbitrary mutations are not assumed to be invalid.
- Deterministic reports exclude timestamps, durations, host-specific fields, and unstable serialization/order. Logs and acceptance evidence may contain those fields separately.

### Change classification

A change to normative decoding semantics, accepted wire encodings, normative tables/constants, coder, checksum, or layout requires the prescribed format-version bump and regenerated vectors. A Codebook-only change changes its own identity, not the format version. An implementation fix restoring the existing contract or an editorial clarification preserving it does not automatically require a format bump; record the classification and rerun affected gates. Any intentional compatibility-affecting change during provisional development is also explicitly versioned, never silently shipped under an unchanged contract.

## Epic List

### Epic 1: Measure whether formulas beat baselines

Deliver a lossless, bounded-expansion CLI and an auditable experiment comparing full MBSC with Baselines-only. The scientific outcome may be positive, marginal, negative, or legitimately inconclusive under the approved policy. Missing thresholds, unresolved format decisions, or unfrozen configuration yield an exploratory/unreportable run, not a scientific verdict.

FR coverage: FR-1 through FR-10 and FR-12 through FR-15. Cross-cutting coverage includes NFR-1 through NFR-3, NFR-5 through NFR-9, NFR-11, and applicable NFR-12 constraints. Epic 2 owns independent-decoder and publication assurance.

### Epic 2: Publish a bitstream someone else can implement

Deliver a standalone specification, LUT, RECIP table, vectors, Python reference decoder, clean-room implementation evidence, and enforceable release checks. The draft normative contract is an early dependency of affected Epic 1 work; clean-room completion and legal clearance are not dependencies of private benchmarking.

FR coverage: FR-11. Cross-cutting coverage includes NFR-4, NFR-7, NFR-10, and publication/compatibility constraints from NFR-12.

## Epic 1: Measure whether formulas beat baselines

### Epic acceptance criteria

- E1-AC01: Stories 1.1–1.13 pass their required acceptance groups. All owner decisions needed for reportable evaluation are closed with recorded approvals.
- E1-AC02: Lossless CLI behavior, malformed-input handling, atomic destination handling, the N + 11 bound, and decoder working-memory limits pass on both supported platforms.
- E1-AC03: Same-span Candidate selection and global partitioning match independent bounded reference oracles, including the complete tie key.
- E1-AC04: Identical input/configuration/Codebook yields identical bitstreams across the supported platforms and recorded thread-count matrix. Clean-checkout benchmark reproduction yields identical sizes and deterministic report content.
- E1-AC05: A G2-frozen experiment executes all required arms, verifies every result through the kernel, and emits the deterministic report, required sidecars, timing log, and provenance.
- E1-AC06: The report applies the approved verdict policy, distinguishes exploratory status from scientific inconclusive status, and may complete the research objective with a negative result.
- E1-AC07: A named-host full run satisfies the 24-hour budget. If a preceding attempt exceeded it, the reduction protocol, new experiment identity where applicable, and tuning excess-byte cost are recorded. Scientific results and budget compliance are separate fields; failure of the latter keeps this AC open.
- E1-AC08: Epic 1 does not require clean-room or counsel approval for private runs. For a clause that is not tagged PROPOSED or MEASURE, G0 is `bitstream-spec-seed.md`, not completion of Story 2.1. Reportable evaluation requires G2.

### Story 1.1: Round-trip a file through whole-file raw

As the author, I want a file to survive a whole-file raw bitstream so that the worst case is lossless and size-bounded before predictors exist.

Acceptance group A — Workspace and contract:

- S1.1-AC01: A clean checkout builds the four `sxp-` crates using the pinned toolchain/edition and committed lockfile. Required repository directories and approved dependency pins from the inventory are present. Build failures do not silently substitute dependency versions.
- S1.1-AC02: The kernel forbids unsafe code, has zero third-party normal/runtime dependencies, and exposes typed errors without shell context strings or global mutable state. `ci.sh` runs formatting, clippy with warnings denied, tests, and the kernel dependency check; the acceptance record includes both supported targets.
- S1.1-AC03: Before raw framing implementation, G0 defines exact magic/version/mode/reserved-bit packing, minimal LEB128, payload order, and little-endian CRC trailer. Commit an exact seven-byte empty-stream fixture and the CRC check fixture for `123456789`.

Acceptance group B — Round-trip and bounds:

- S1.1-AC04: Empty, short, random, already-compressed fixture, and boundary-length inputs round-trip. Under the intended two-byte fixed header, raw size is exactly N + 6 + leb128_len(N), and at most N + 11 for N < 2^35. Verify every LEB128 width boundary with writer/parser fixtures without allocating enormous inputs.
- S1.1-AC05: Declared lengths at or above 2^35, non-minimal/overflowing/truncated LEB128, bad magic, unsupported/reserved versions, reserved bits/modes, truncated payload/trailer, CRC mismatch, and trailing bytes return typed errors. Each rule has an identified fixture.
- S1.1-AC06: Stream-to-sink tests demonstrate at most 64 KiB decoder-owned heap under the documented accounting boundary; increasing N does not increase working memory. Malformed-input regressions and the recorded fuzz profile produce no panic or out-of-bounds access.
- S1.1-AC07: This writer emits whole-file raw only; blocked-mode writing remains unavailable until Story 1.8. Both-platform exact-byte fixtures pass.

### Story 1.2: Reconstruct every byte from its prediction

As the author, I want every predicted byte to reconstruct the original exactly so that prediction quality affects size, never correctness.

- S1.2-AC01: Exhaustively test all 65,536 pairs y, ŷ in 0..255. Compute δ = ((y − ŷ + 128) mod 256) − 128, emit symbol = δ mod 256, recover signed δ as symbol for symbol < 128 and symbol − 256 otherwise, and reconstruct y = (ŷ + δ) mod 256 using mathematical modulo.
- S1.2-AC02: Explicit fixtures cover both wrap directions and y = 0, ŷ = 128 producing δ = −128 and symbol 128. Random Block/predictor property tests have zero mismatches.
- S1.2-AC03: Encoder and decoder use the same kernel residual/reconstruction functions. Direct entropy coding emits y without invoking residual prediction.
- S1.2-AC04: Publish the exhaustive symbol sequence hash with pair ordering y ascending, then ŷ ascending, one unsigned symbol byte per pair. Both platforms match the committed artifact.

### Story 1.3: Evaluate a polynomial formula

As the author, I want polynomial predictions to agree on every supported platform.

- S1.3-AC01: G0 defines degree 0..15, Q 8..24, exactly degree + 1 coefficients in [−2^31 + 1, 2^31 − 1], and Block-local position n in 0..2047. Invalid typed inputs return errors; an API using narrower types documents where unrepresentable values are rejected.
- S1.3-AC02: Evaluation uses u = n << 5, signed-i64 Horner acc = ((acc · u) >> 16) + c[k], then value = (acc + (1 << (Q − 1))) >> Q and ŷ = rem_euclid(value, 256). Shifts are arithmetic floor shifts; no floating point or division other than rem_euclid is used.
- S1.3-AC03: Independent expected-value fixtures cover degrees 0 and 15, both Q limits, every coefficient-domain boundary, n = 0 and 2047, and rounding below/at/above positive and negative halfway cases. Q = 8 and c0 = −128 produces value 0. Invalid adjacent values are rejected without clamping/wrapping.
- S1.3-AC04: A written intermediate-width bound accompanies boundary tests. Both-platform vectors match exactly, and a Formula's prediction prefix is unchanged by actual Block length.

### Story 1.4: Evaluate a trigonometric formula

As the author, I want trigonometric predictions to share one normative table and interpolation rule.

- S1.4-AC01: G0 defines order 1..16, Q 8..24, unsigned 32-bit omega, exactly 2·order + 1 coefficients, n in 0..2047, and the evaluator domain `|c| < 2^28`. D-01 was approved on 2026-11-01: stored cw is at most 29, and serialization uses that pair. The cw a production Codebook uses inside the cap stays under D-07.
- S1.4-AC02: Evaluate theta = (n · omega) mod 2^32; start S = c0 · 2^15; for k = 1..order add a_k · sin_lut(p) + b_k · sin_lut((p + 2^30) mod 2^32), where p = (k · theta) mod 2^32. Final rounding is (S + (1 << (Q + 14))) >> (Q + 15), followed by rem_euclid 256. Signed accumulation is i64; phase wrapping is explicit.
- S1.4-AC03: The normative 1025-entry signed-i16 LUT has equal endpoints, |entry| ≤ 32767, and a committed hash. Interpolation uses i = p >> 22, f = (p >> 6) & 0xFFFF, and v0 + (((v1 − v0) · f + (1 << 15)) >> 16). Table generation is non-normative and may not silently replace the committed table.
- S1.4-AC04: Independent fixtures cover both order/Q limits, coefficient limits and invalid neighbors, omega 0 and 2^32 − 1, phase wrapping, quadrants, the last interpolation segment, interpolation/final rounding boundaries, and n = 2047. Unsupported parameters return typed errors without clamping.
- S1.4-AC05: Integrity validation rejects a modified LUT. Both-platform fixtures match, intermediate-width evidence is recorded, and all earlier evaluator vectors still pass.

### Story 1.5: Evaluate a recurrence formula

As the author, I want recurrence predictions to keep order, initial state, and modulus distinct.

- S1.5-AC01: G0 defines order 1 or 2, m_mod in 2..=2^24, 0 ≤ s_i < m_mod, n in 0..2047, and `|a|, |b|, |c| < 2^24`. D-01 was approved on 2026-11-01: stored cw is at most 25. Order-1 c/s1 fields are absent by type or explicitly rejected rather than silently consumed. The cw a production Codebook uses inside the cap stays under D-07.
- S1.5-AC02: Order 1 initializes x[0] = s0 and computes x[n+1] = rem_euclid(a·x[n] + b, m_mod). Order 2 initializes x[0] = s0, x[1] = s1 and computes x[n+2] = rem_euclid(a·x[n+1] + b·x[n] + c, m_mod). ŷ = rem_euclid(x[n], 256); intermediates are i64.
- S1.5-AC03: Independent sequences cover both orders, both modulus limits, initial states 0 and m_mod − 1, negative coefficients, legal coefficient boundaries, and invalid adjacent values. Unsupported order, negative/out-of-range state, invalid modulus, or invalid coefficients return typed errors.
- S1.5-AC04: Every Block restarts at its own initial values. A preceding Block has no effect. Both-platform vectors and previous-family regressions pass; intermediate-width evidence is recorded.

### Story 1.6: Code decisions with the binary range coder

As the author, I want a specified binary range coder with exact byte counts and independently decodable decisions.

- S1.6-AC01: G0 specifies all state widths, bound computation, normalization threshold/loop, carry/cache transitions, initialization, and termination. State starts low with at least 33 bits, range = 0xFFFFFFFF, cache = 0, cache_size = 1. The coder exposes encode, decode, and exact-size operations behind one kernel interface; no coder ID is signalled.
- S1.6-AC02: freq0 is in 1..4095 and total = 4096; invalid API frequencies are rejected. For fixed decision/frequency schedules, decoding reproduces every bit and exact-size counting equals emitted length, including carry handling and termination.
- S1.6-AC03: Flush invokes shift_low five times; the initial invariant 0x00 is omitted and fixed termination cost is four bytes. Exact fixtures include bit 0/freq0 2048 → 00 00 00 00 and bit 1/freq0 2048 → 7F FF F8 00, frequencies 1 and 4095, and carry through pending 0xFF.
- S1.6-AC04: Define the 2048-symbol fixture as 2048 byte symbols expanded into a committed 16,384-decision schedule with explicit frequencies. Commit expected bytes/hash and test randomized schedules as well.
- S1.6-AC05: Decoder initialization reads four bytes MSB-first, rejects initial code 0xFFFFFFFF, and requires final code 0. Truncation at every fixture byte boundary returns an error when required bytes are missing. Concatenated-stream tests verify exact consumption without reading the next frame.
- S1.6-AC06: A trace verifies take frequency → decide bit → update coder state → refill. This layer does not update the probability model. No rANS, deferred symbol state, or cross-Block coder state is introduced. Both-platform vectors pass.

### Story 1.7: Adapt an order-0 probability model

As the author, I want one causal model reconstructed from already decoded decisions.

- S1.7-AC01: G0 specifies 255 bit-tree nodes, MSB-first traversal, initial counts, RECIP indexing, count increment timing, saturation, and initialization shape. Immutable configuration is explicit and hashed; the strawman p = 32768 and CAP = 24 remain labelled provisional until D-07 closes.
- S1.7-AC02: Each new Block resets every node. Each symbol uses eight decisions; node choice uses depth and the already decoded prefix, never the not-yet-decoded symbol. No probability tables or model selection fields are transmitted.
- S1.7-AC03: With selected rate R, bit 0 updates p by +(((65536 − p)·R) >> 16); bit 1 updates p by −((p·R) >> 16). Use u32/i64, no i32 product or signed-32 shift. Legal configurations preserve p in [1,65535] without clamp; freq0 = max(p >> 4,1), freq1 = 4096 − freq0. Reject configurations violating required bounds.
- S1.7-AC04: Commit finite, fully specified fresh-node, CAP-boundary, all-zero/all-one saturation fixtures with initial state, bit counts, exact expected states/frequencies, and RECIP hash. Test the required strawman symmetry bound using the committed schedule rather than an unspecified “long run.”
- S1.7-AC05: Encoder/decoder traces agree on node, count, p, and frequencies for every decision. Integration verifies frequency → decision → coder state update → refill → exactly one model update; no pending-symbol/deferred-update or decode-gating test exists.
- S1.7-AC06: Test scalar and any supported per-depth initialization, every tree path, Block resets, both-platform vectors, and configuration integrity. Correctness evidence is separate from tuning-score evidence.

### Story 1.8: Round-trip one baseline block

As the author, I want Raw, Direct, Previous-byte, and Constant Blocks to share one framing/coding interface.

- S1.8-AC01: D-02 was approved on 2026-11-01. Commit exact header/field/padding byte fixtures before implementation. The 2-bit length code and 3-bit kind use LSB-first packing. Kind-specific fields follow the 5-bit header immediately, checked zero padding runs to the next byte, and the payload begins byte-aligned.
- S1.8-AC02: One kernel encode_block interface handles the four baseline kinds and later Formula kinds. Raw skips prediction/model/coder; Direct codes original bytes; Previous-byte predicts 0 first then the previous decoded byte; Constant uses its one signalled byte. Model/coder state resets per Block.
- S1.8-AC03: Nominal lengths are 256/1024/2048 and actual length is min(nominal, remaining). Canonicality is checked from remaining length. For independent Block decoding the API receives actual length or equivalent remaining-file context as well as the Block bytes; no previous Block state is required.
- S1.8-AC04: Exact vectors cover each baseline kind × each length code, partial finals, and actual lengths 1,255,256,257,1023,1024,1025,2047,2048 where canonical. Multi-Block streams consume exactly each payload and preserve the next header/trailer.
- S1.8-AC05: Reject reserved length/kind codes, non-canonical nominal lengths, non-zero padding, invalid/truncated payloads, blocked mode N = 0, and file-level CRC/trailing-byte errors. Coded payload length follows normative coder consumption, not a stored length field.
- S1.8-AC06: Changing a preceding Block cannot change current-Block predictions, model state, output, or consumed payload. Both-platform fixtures and working-memory tests pass.

### Story 1.9: Select baselines by exact size

As the author, I want exact-size selection and deterministic global partitioning.

- S1.9-AC01: D-03 was approved on 2026-11-01. The final partial span uses the smallest nominal length in {256, 1024, 2048} that is ≥ the bytes left. Document allowed transitions over 256-aligned nonterminal offsets: full L ≤ remaining from the permitted length set, plus that canonical final transition. Deduplicate identical transitions at exact length boundaries. Include fixed-length examples around every length boundary.
- S1.9-AC02: Every transition cost equals kernel-emitted Block bytes including framing, parameters, padding, payload, and flush. DP minimizes (total size, Block count, lexicographic sequence of (length code, Candidate ID)). IDs are Raw 0, Direct 1, Previous-byte 2, Constant c = 3+c; all 256 constants are considered.
- S1.9-AC03: An independent exhaustive oracle on bounded synthetic cost graphs and small real fixtures matches the full selected key and partition. Same-span property tests additionally verify no Candidate is smaller than the selected Candidate.
- S1.9-AC04: Tie fixtures cover Raw versus coded, baseline ordering, constants, Block-count ties, lexicographic partition ties, and whole-file raw. Compare complete blocked and whole-file raw sizes including file header/CRC; whole-file raw wins ties, including empty input.
- S1.9-AC05: Fixed-length arms use the approved final-Block exception, if any, and remain valid for the common decoder. Single-threaded and recorded parallel configurations yield identical complete output bytes on both architectures.
- S1.9-AC06: An unoptimized reference mode is retained for correctness tests. Allowed exact optimizations produce identical selected keys/bytes. No entropy heuristic or unproven lower bound changes the winner; proof-dependent optimizations are disabled until reviewed evidence is recorded.

### Story 1.10: Search a regenerable formula codebook

As the author, I want a reproducible Codebook searched exhaustively without local parameter fitting.

Acceptance group A — Codebook generation:

- S1.10-AC01: A documented integer-only recipe emits all three families with at most 4096 total and 2048 per family. Define stable family/entry order, duplicate policy, canonical content serialization, version, and content hash. Generation is reproducible across platforms.
- S1.10-AC02: Reject absent recipes, invalid Formulas, cap violations, and unresolved production configuration. Tests use an explicitly labelled fixture recipe with at least one Formula per family; it cannot masquerade as the approved benchmark recipe. Production tuning uses tuning inputs only.

Acceptance group B — Formula Block layouts:

- S1.10-AC03: D-01 and D-02 were approved on 2026-11-01. Trigonometric cw is at most 29 and recurrence cw is at most 25. Kernel writers/parsers own every field; the encoder serializes none itself. Polynomial stores degree/Q/width and degree+1 coefficients; trigonometric stores order/Q/width/omega and 2·order+1 coefficients; recurrence stores order/modulus/width/coefficients/initial states independently. The production width inside those caps stays under D-07.
- S1.10-AC04: Exact valid and rejection vectors cover every family field, sign extension, width boundary, count encoding, parameter-domain boundary, reserved encoding, invalid state, zero padding, and truncation point. Include at least one valid Formula Block for every family/length combination. Decode successfully after removing all Codebook files.

Acceptance group C — Exhaustive selection:

- S1.10-AC05: Every baseline and Formula is logically considered for every allowed span. Formula Candidate IDs start at 259 under the documented canonical family/entry order; a baseline wins a same-span size tie against a Formula. Wire kind codes 4/5/6 do not define tie ordering.
- S1.10-AC06: Exact pruning is permitted only under the documented admissibility rules. Compare optimized mode with an unoptimized exhaustive oracle for Candidate, partition, size, and complete bytes; specify how skipped costs are represented in sidecars.
- S1.10-AC07: Results match across platforms and thread counts. Format identity is independent of Codebook identity; the decoder never reads the recipe, Codebook, or encoder state.

### Story 1.11: Compress and decompress a file

As the author, I want lossless file commands with atomic output behavior and safe failures.

- S1.11-AC01: For the committed input matrix, compress then decompress reproduces exact bytes, compress equals the full-MBSC encoding path, and bitstreams are at most N+11 bytes. Include empty, short, exact-Block, partial-final, random, and already-compressed cases across platforms/thread counts.
- S1.11-AC02: Document destination overwrite policy and same-file/alias handling before CLI acceptance. On any error, no new destination is installed, an existing destination remains byte-identical, and only temporary files created by that invocation are cleaned up. Never delete a pre-existing destination merely to satisfy “no output.”
- S1.11-AC03: Successful commands write a temporary file in the destination directory, complete all validation including CRC, close/flush successfully, and rename into place according to the documented overwrite policy. Inject read, write, flush/close, and rename failures and verify cleanup/preservation.
- S1.11-AC04: Compress refuses uncompressed source length ≥2^35; decompress rejects declared decoded N ≥2^35. Do not apply the uncompressed limit blindly to encoded file byte length. Tests use sparse/mock inputs or header fixtures rather than huge allocations.
- S1.11-AC05: Success exits 0; all errors exit 1 with stderr diagnostics. Truncated, malformed, corrupted, trailing-byte, and CRC-failing inputs do not expose partial decoded output at the destination.
- S1.11-AC06: Kernel parsing/decoding uses Result, forbids unsafe code, and denies unwrap_used, panic, and indexing_slicing. Recorded adversarial/fuzz tests and counting-allocator tests meet the no-panic/out-of-bounds and ≤64 KiB working-heap acceptance evidence.
- S1.11-AC07: The runtime decode path is CLI → kernel only and does not load a Codebook or call encoder logic. There is no stdin/stdout data streaming, config file, or logging framework; benchmark stdout remains a separate command requirement.

### Story 1.12: Fetch the corpus manifest

As the author, I want verified, reproducible inputs, transforms, and overlap evidence.

Acceptance group A — Fetch and transforms:

- S1.12-AC01: tools/fetch uses the pinned Python standard library only and an explicit cache outside the repository. Manifest entries record source/member, licence (including none stated), source and derived hashes, role/class, transform/version, and parameters. CI uses local fixtures, not network access.
- S1.12-AC02: Verify full archive hashes before extraction, extract only listed regular members, and reject path traversal, unsafe links, corrupt/truncated archives, hash mismatches, and unsupported formats. Apply the documented source-MD5 check when required and pin SHA-256.
- S1.12-AC03: pcm16le-to-u8 v1 accepts only the documented mono signed-16-bit little-endian 22,050 Hz WAV format, emits high-byte XOR 0x80 without a WAV header, and preserves raw sample bytes for non-counted rows. Unsupported formats fail explicitly. Slice v1 records source hash, offset, length, derived hash; middle offset is floor((size−length)/2).

Acceptance group B — Corpus validity and leakage:

- S1.12-AC04: Validate the inventory's roster, equal-slice rules, PCM whole-clip rule, executable selection, each Real class's 4–16 MiB and ≥3 evaluation inputs, and documented correlations/caveats. Deterministic PCM selection follows the recorded metadata, prefixes, hash ordering, seed, and byte-budget rule.
- S1.12-AC05: Exact overlap tests detect shared contiguous runs of at least K, including across internal chunk boundaries and despite different file hashes. A small brute-force oracle validates the optimized checker. Evaluation inputs stay fixed within an experiment; replacements follow the approved per-class tuning policy.
- S1.12-AC06: Proposed K supports exploratory checks only. Confirmed K, LJ metadata/prefixes/seed, Debian tuning package, and replacement rules must be recorded without placeholders for G2. A machine-readable gate report identifies every unresolved field and violated constraint.

Acceptance group C — Synthetic data:

- S1.12-AC07: Integer-only generators with documented seeds produce repeatable signals matching all three families. Store/report them separately from Real classes; their Gain never contributes to SM-1. Commit expected hashes and both-platform results.

### Story 1.13: Report whether formulas beat baselines

As the author, I want an auditable comparison and either a correctly classified research verdict or explicit unreportable status.

Acceptance group A — Arms and correctness:

- S1.13-AC01: Execute Whole-file raw, forced Constant/Previous-byte/Direct, Baselines-only, and full MBSC with the inventory's Candidate sets and independent partitioning. Every output is a real Bitstream, is kernel-decoded, and equals the source; mismatch fails the run and prevents a successful verdict artifact.
- S1.13-AC02: For each input verify S_full ≤ S_baselines-only ≤ min(S_forced Constant,S_forced Previous,S_forced Direct) ≤ S_whole-file raw. Publish the best forced-baseline size as a consistency check, not the Gain comparator.

Acceptance group B — Deterministic report and sidecars:

- S1.13-AC03: Emit versioned JSON and stdout results with per-input sizes, class sums, Gain bytes/share, per-input wins/margins, and separate synthetic, compressed-control, and non-counted PCM rows. Gain compares full MBSC with Baselines-only on evaluation inputs only.
- S1.13-AC04: Byte accounting includes file/Block headers, all baseline/Formula parameters, alignment padding, raw payload, coder payload, coder flush, and CRC, with no omissions or double counting. Define attribution of packed fields; components sum exactly to the real file size. Whole-file raw has a complete decomposition too.
- S1.13-AC05: Define adaptive ideal length as the decision sum of −log2(freq/4096); subtract the selected Block symbol stream's zero-order entropy in bits to report adaptation overhead. Report coder flush separately. State raw/direct/residual inclusion rules and denominators for zero-residual and family/baseline Block/byte shares. Define deterministic numeric rounding/serialization for derived floating-point metrics.
- S1.13-AC06: Record tool/revision, format/table/layout identity, Codebook recipe/version/hash, manifest/source/transform/tuning hashes, overlap K/result, owner decisions, thresholds, experiment ID, and allowed Block lengths. Work counts define model decisions and coder steps per input byte, including exhaustive-search work and optimization treatment. Wall time, host, and timestamps appear only in the timing log.
- S1.13-AC07: Sidecars default on for N ≤4 MiB and are opt-in above it. Define the versioned u16 matrix schema: endianness, offset/length/Candidate axes and metadata, cell validity, sentinel semantics, and overflow handling. Every evaluated Candidate has an exact cost. Legal header-pruned Candidates are explicitly distinguished from evaluated Candidates; a dense audit mode disables pruning and records all legal exact costs. Report which inputs have sidecars and their coverage mode.
- S1.13-AC08: Fixture reports reconcile every total and sidecar cell, satisfy arm invariants, and reproduce identical deterministic content across clean checkouts/platforms/thread counts. Separate logs may differ.

Acceptance group C — Freeze and verdict:

- S1.13-AC09: G2 validation occurs before reportable evaluation. Exploratory runs may print sizes but state exploratory/unreportable plus missing gates. A missing corpus gate (K, LJ Speech metadata, the Debian package, or the production Codebook recipe) or provisional model constants are not mapped to scientific inconclusive. The growth-path numbers and the one-class rule are already set (D-05).
- S1.13-AC10: A table-driven verdict function implements the approved policy. Fewer than two eligible classes is inconclusive. Zero winning classes is negative. One winning class is negative and opens only the deferred PCM experiments. Two classes with Gain below the growth bar are marginal. The growth bar is Gain of at least 1% of that class's Total encoded size and wins on a majority of that class's evaluation inputs. Persist the policy inputs and explanation with the result; do not substitute any other threshold.
- S1.13-AC11: Model tuning uses Baselines-only; legal coefficient/frequency-width choices use full MBSC; tuning inputs only. Persist the grid, every score, tie ordering, selected configuration, and hashes. No evaluation-driven retuning is accepted under the same experiment identity.

Acceptance group D — Budget:

- S1.13-AC12: The timing log names the host before the budget run and records scope, start/end, completion, and ≤24-hour result. On excess, follow D-06, approved 2026-11-01: shrink the corpus first, keep each class's minimum, and do not change frozen evaluation inputs; then lower the Codebook cap. Changing evaluation inputs starts a new experiment revision. Publish tuning excess-byte costs and the revised provenance. No heuristic selection shortcut is permitted.

## Epic 2: Publish a bitstream someone else can implement

### Epic acceptance criteria

- E2-AC01: Stories 2.1–2.4 pass, all normative assets are complete/versioned/hashed/licensed, and no normative placeholders remain at G4.
- E2-AC02: Rust, Python, and clean-room implementations agree on valid/rejection suites and defined three-way fuzz profiles on both supported platforms.
- E2-AC03: The clean-room provenance record establishes permitted inputs, identity, isolation, resolved ambiguities, tested revisions, and asset hashes.
- E2-AC04: An executable G5 release-readiness check fails on absent/stale legal, licensing, human-review, spec, or decoder-conformance evidence. A marker alone is not approval.
- E2-AC05: Relevant format changes invalidate previous gate evidence and trigger required versioning/revalidation. Private benchmarking remains independent of G4/G5.

### Story 2.1: Write the bitstream specification

As an independent implementer, I want a standalone normative document without needing Rust source.

Acceptance group A — Early draft contract:

- S2.1-AC01: The G0 draft for every clause that is not tagged PROPOSED or MEASURE is `bitstream-spec-seed.md`. Kernel work on an untagged clause uses that seed and does not wait for `spec/bitstream.md`. Group A promotes the seed into `spec/bitstream.md`, carrying exact magic/version/mode packing, byte/bit order, both modes, N encoding, Block fields/alignment, family limits, residual/evaluator semantics, coder procedure, model update/count rules, canonical lengths, checksum, and reject rules. Clauses tagged PROPOSED or MEASURE stay out of kernel acceptance until their tag is resolved. D-01 and D-02 were approved on 2026-11-01. D-07 still blocks treating strawman model constants, or a chosen cw inside the approved caps, as frozen. Unrelated draft work can complete.
- S2.1-AC02: Provisional constants/tables are labelled with status and hash. Separate normative decoder constraints from informative encoder search/partition policy. rANS remains informative and outside the implemented format.

Acceptance group B — Standalone frozen specification:

- S2.1-AC03: At G4 no normative TBD/unfrozen value remains. A decoder's accepted input language, consumption boundaries, and decoded output are determined by the document, LUT, RECIP table, and vectors alone. Include standalone rejection/limit IDs and a complete coverage map.
- S2.1-AC04: Name/version/hash every normative asset, document fixed-point widths/rounding/overflow, signed encodings, initialization/termination, and version policy. The document is CC BY 4.0; LUT/RECIP/vectors are CC0.
- S2.1-AC05: Group A promotes the existing spec seed into `spec/bitstream.md`. That promotion is not a prerequisite for kernel work on untagged clauses, and it is not a clean-room or legal release gate. Committing a draft does not count as an approved public release; a release action requires G5.

### Story 2.2: Check the kernel with a Python reference decoder

As the author, I want an executable specification check independent of Rust implementation details.

- S2.2-AC01: tools/refdec uses the pinned Python standard library only. Record source materials and implementation provenance establishing it was written from the document/assets rather than translated from Rust. It neither links nor invokes the kernel to decide decoding results.
- S2.2-AC02: A seeded valid-stream generator covers both file modes, every kind/length, boundary parameters, partial finals, coder carry cases, and all required file sizes. Mutators and targeted malformed constructors retain reproducible inputs and expected intended rules.
- S2.2-AC03: A coverage table maps every reject-rule and limit ID to valid-boundary and rejection vectors where applicable. Targeted fixtures exercise the intended rule without unrelated earlier failures masking it where practical. Exact decoded output/consumption expectations are recorded.
- S2.2-AC04: Kernel and Python either decode identical bytes or both reject on all vectors and the recorded generated/mutated campaign. Mutations are not automatically classified as invalid. Save every disagreement as a minimized/reproducible regression fixture.
- S2.2-AC05: ci.sh runs the check on x86-64 and aarch64. Pre-freeze failure is visible but does not block private benchmarking; passing is mandatory at G4 and before public release. Acceptance records include revisions, asset hashes, seeds, corpus, and campaign profile.

### Story 2.3: Record the licence and counsel gate

As the author, I want enforceable publication checks without blocking private research runs.

- S2.3-AC01: License files/identifiers cover code and Python as MIT OR Apache-2.0, spec as CC BY 4.0, and vectors/LUT/RECIP as CC0. Validate coverage for generated/distributed artifacts and contribution instructions.
- S2.3-AC02: Contribution policy requires DCO sign-off without CLA and records human review/verification/approval for AI-assisted changes. Release evidence identifies the approved revision and reviewer record rather than merely asserting review occurred.
- S2.3-AC03: The counsel checklist includes Qualcomm US 10,554,988 and US 10,148,961, Tencent US 12,348,780, and general clearance. Record date, reviewed revision/scope, decision, and unresolved conditions without requiring confidential advice to be public. No automated test is represented as a legal conclusion.
- S2.3-AC04: Review-triggering decoder-structure or format-field changes invalidate affected clearance pending new review. Future rANS adoption requires review of Microsoft US 11,234,023 B2 and family plus a format bump; rANS is not implemented here.
- S2.3-AC05: An executable release-readiness check fails for missing/stale legal approval, unmet conditions, missing license/human-review evidence, or failed G4 conformance. Test each missing/stale-evidence case with fixtures. Release creation runs this check; a public-release marker alone cannot bypass it.
- S2.3-AC06: Private benchmark commands do not read a release marker or require counsel clearance. Distribution/publication actions remain gated by the recorded release policy.

### Story 2.4: Pass the clean-room decoder gate

As an independent implementer, I want a decoder built only from the public format contract to demonstrate specification sufficiency.

- S2.4-AC01: Use a language other than Rust/Python and a different person or fresh sandboxed agent from a different model family. Record implementer identity or model ID, prompt, permitted files, tool/environment access, isolation measures, and relevant transcript/provenance. Rust/Python implementation sources are not permitted inputs.
- S2.4-AC02: Permitted inputs are the standalone specification, LUT, RECIP table, vectors, and explicitly documented neutral tooling. The author's Python decoder is not a substitute. Specification ambiguities become tracked issues, are resolved in the document, and are revalidated before gate acceptance.
- S2.4-AC03: Accept generated valid streams and reject identified rejection vectors, with exact decoded output. Run seeded three-way tests on generated valid streams, targeted malformed streams, arbitrary bytes, and mutations: all decode identical bytes or all reject.
- S2.4-AC04: Define and record a bounded deterministic CI profile and larger release profile, including generators, seed sets, case counts/resource limits, and retained regressions. Both supported platforms pass; crashes/panics, disagreements, or unresolved normative ambiguities fail the gate.
- S2.4-AC05: Classify findings as implementation defects, editorial clarifications preserving semantics, or normative-contract changes. Only the latter automatically require the prescribed format bump and regenerated vectors; all require affected revalidation. Maintain clean-room isolation during corrections by passing contract updates and neutral failing fixtures, not implementation patches.
- S2.4-AC06: Gate evidence records specification/assets hashes, all decoder revisions, platforms, campaign profiles/seeds/results, and ambiguity closure. The gate is mandatory at G4, before first public release, and on format-version bumps. Relevant changes invalidate stale evidence; private benchmarking does not wait for this gate.

## Delivery dependency and evidence checklist

1. For clauses that are not tagged PROPOSED or MEASURE, implement from `bitstream-spec-seed.md`. Do not wait for Story 2.1 group A. D-01, D-02, and D-03 were approved on 2026-11-01; use those contracts for serialization, Block packing, and the final partial span. The chosen coefficient width inside the D-01 caps stays under D-07. Story 2.1 group A promotes the seed into `spec/bitstream.md`.
2. Complete raw framing, residuals, evaluators, coder, and model; develop Python checks alongside them.
3. Complete baseline integration and independent exact-partition oracles.
4. Complete Formula serialization, Codebook validation, and exhaustive search integration.
5. Complete atomic CLI behavior and both-platform deterministic/safety evidence.
6. Validate corpus, close owner gates, and run tuning without evaluation leakage.
7. Save G2, execute G3, and preserve deterministic reports/sidecars separately from timing logs.
8. Complete G4 standalone-spec and clean-room evidence, then G5 before public release.

Acceptance records must link each epic/story AC to passing tests or reviewed artifacts. No owner decision is implicitly approved by generation of this document.

## Elicitation amendments (2026-11-01)

These amend the proposed revision above. They do not themselves approve D-01 through D-09. Approvals dated 2026-11-01 are in the decision register. Where an acceptance criterion or checklist item conflicts with an amendment, the amendment governs how that item is read, and the register governs where an amendment still says a decision is open.

1. **Unblocking versus closing a story.** Native-target evidence is enough for the next story to start. A story that requires both platforms stays open until x86-64 and aarch64 evidence is recorded. That wait does not freeze earlier native work.
2. **Deterministic report scope.** Byte counts, arm sizes, and integer work counts are the deterministic comparison. Adaptation overhead that uses −log2 is a labelled annex with a specified rounding, and it is excluded from the byte-identity check. S1.13-AC05 and S1.13-AC08 are read this way.
3. **G0 is the existing spec seed.** `bitstream-spec-seed.md` is the G0 draft for every clause that is not tagged PROPOSED or MEASURE. Story 2.1 group A promotes that seed into `spec/bitstream.md`. Kernel work on an untagged clause does not wait for a second document. Tagged clauses stay unresolved until their own decision closes. D-01 and D-02 closed on 2026-11-01. D-07 is still open. Checklist item 1, E1-AC08, S2.1-AC01, and S2.1-AC05 state this directly as of 2026-11-01.
4. **D-09, codebook recipe.** The production codebook's composition is an owner decision. A fixture recipe may be used in tests. Reportable evaluation requires an approved recipe, version, and hash. On 2026-11-01 the production recipe was left unwritten. This decision is still required before production codebook acceptance and before G2.
5. **File header versus block packing.** The file header is the seed's magic byte, version/mode byte, minimal LEB128, payload, and CRC. D-02 governs only where a block's kind-specific fields sit relative to padding. S1.1-AC04's size formula applies to that file header.
6. **Python checks stay off the private-benchmark path.** Checklist item 2 may proceed beside the kernel. A Python disagreement does not block stories 1.1–1.13 or a private benchmark. It blocks G4.
7. **D-01 is approved.** Evaluator fixtures use trigonometric coefficients below 2²⁸ and recurrence coefficients below 2²⁴. Stored cw may be as wide as 29 (trigonometric) and 25 (recurrence) so those limits are representable. The narrower seed caps (cw ≤ 28 and cw ≤ 24) are not the contract. Which width a production Codebook uses inside the caps stays under D-07.
8. **Selector inequality.** S1.13-AC02 is an invariant of exact-size selection. A violation fails the run as a selector defect. It is not a scientific verdict.
