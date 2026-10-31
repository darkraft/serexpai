# Acceptance criteria — MBSC v1

Per-capability testable consequences from the PRD (FR-n), refined by the architecture spine (AD-n, binding where they differ). Each line is checkable by a test or demonstration.

## CAP-1 (FR-1)
- Decode of the Residual yields y for every y, ŷ in 0..255.
- Property test over random Blocks and Predictors: zero mismatches.

## CAP-2 (FR-2, FR-3; AD-5, AD-6)
- Spec states: intermediate widths, overflow behaviour, rounding including negative halves, division and shift semantics, trig argument reduction and LUT interpolation, coefficient limits, recurrence initialization, non-negative modulo.
- Decoder uses no floating point. Test vectors pass on Linux x86-64 and aarch64.
- Polynomial degree stored separately from its degree + 1 coefficients; trigonometric order separately from its 2·order + 1 coefficients; recurrence modulus named `m_mod`. Every field covers its full range (Q in [8, 24], counts 1 to 16, via offset encoding).
- Boundary vector and rejection vector for every limit.

## CAP-3 (FR-4, FR-5; AD-9, AD-13)
- Codebook contains polynomial, trigonometric and recurrence Formulas, is regenerated from a documented integer-only recipe, has a version and content hash, and records its tuning-input hashes. Cap: ≤ 4096 Formulas total, ≤ 2048 per family.
- Block length per Block is signalled; the partition rule is deterministic and stated in the spec; benchmark can run each fixed Block length alone.
- Parallel run is byte-identical to a single-threaded run.

## CAP-4 (FR-6; AD-10)
- Chosen Candidate's exact size ≤ every other Candidate for the same span.
- Raw Block wins unless strictly smaller, framing counted on both sides; ties never favour a Formula over a Baseline; Whole-file raw wins ties and the empty input.
- No heuristic or entropy screen, so `compress` and the full-MBSC `benchmark` arm produce identical bytes. Allowed exact speed-ups: ŷ tables precomputed once per Formula; one model probability sequence shared across Block lengths at an offset (the coder encode stays per Candidate and length); skipping a Candidate whose header and parameter bytes alone reach the incumbent's exact size; reciprocal-table division. A lower-bound prune on coder size needs a written proof first (the empirical `exact ≥ ceil((ideal + 23)/8)` held for both coders in the prototype but is unproven).

## CAP-5 (FR-7; AD-2, AD-4)
- Constant: per-Block constant byte in the Bitstream. Previous-byte: predicts the preceding byte within the Block; first byte predicted as 0.
- Baselines use the same Probability model and framing as Formulas.

## CAP-6 (FR-8; AD-7)
- Model is adaptive order-0 over an 8-decision bit tree, rebuilt by the decoder from decoded symbols, nothing transmitted, reset every Block; coder total 4096, 32-bit `range` and `code` (encoder `low` needs at least 33 bits), byte renormalization; coder hand-written in the kernel.
- Node probability is 16-bit (`p16` = P(0)·65536) updated in u32 or i64 arithmetic: on 0, `p += ((65536 − p)·R) >> 16`; on 1, `p −= (p·R) >> 16`, with `R = RECIP[n]` from a normative table whose entries lie in [1, 65535]. Coder frequency `freq0 = max(p >> 4, 1)`, `freq1 = 4096 − freq0`. Initial `p` lies in [16, 65520] (scalar or 8-entry per-depth table, one for all Candidates).
- Model vectors cover a fresh node, `n = CAP`, and saturation stalls for both bit values.
- Tuning procedure, `tuning` inputs only: grid over CAP {24, 32, 60, 128, 255} × DELTA {0.5, 1} × initial `p` (best scalar, then an 8-entry per-depth table adopted only if it beats that scalar), scored by the Baselines-only arm's summed Total encoded size; `cw` and ω widths scored by full MBSC. Ties go to the smaller CAP, DELTA = 1 and the scalar. The grid and every score are recorded.
- Coder (seed §6.1): LZMA-style binary range coder; decisions encoded in forward order, carry via cache, first zero byte dropped, 4-byte flush per Block, behind one kernel interface. The spec states initialization, renormalization byte order and termination. rANS (seed Appendix A) is not implemented for v1.
- Coder tests: exact round-trip and exact payload consumption on random Blocks of every length; at least one carry through a pending `0xFF` byte; the two 1-decision vectors (`00 00 00 00`, `7F FF F8 00`).
- Every MBSC decoder follows the seed's reference decode procedure (no deferred update, no pending-symbol state, no gating test); a code review checklist item and the clean-room brief state it.
- Per-Block fixed cost (header and coder flush) and adaptation overhead measured and reported (at 256-byte Blocks they can exceed any Formula gain); the model itself transmits nothing.

## CAP-7 (FR-9; AD-8, AD-11)
- Spec defines bit order, endianness, header parser, Raw Block length, payload length (derived, not stored), framing, termination; Bitstream carries a 4-bit format version (v1 = 1; 0 invalid; 15 reserved for extension).
- Every file ends with `LE32(CRC-32/ISO-HDLC(decoded bytes))`, in both modes; nothing follows it. The empty input is 7 bytes.
- Decoder rejects: version 0, 15 or unknown, set reserved bit, non-minimal LEB128, bytes after the CRC, stream ending before N bytes or before the CRC, CRC mismatch, initial coder `code = 0xFFFFFFFF` or final `code ≠ 0`, non-canonical nominal length, reserved codes, non-zero padding, out-of-limit parameters.
- A single flipped bit in Formula parameters, the Constant byte, a kind code or a Raw Block is rejected.
- Round-trip: empty, 1 byte, 255, 256, 257, exactly one 2048 Block, partial final Block.

## CAP-8 (FR-10; AD-10)
- Output ≤ N + 11 bytes for every input (2 header bytes + at most 5 LEB128 bytes + 4 CRC bytes, versions 1–14); test on random and already-compressed inputs. `compress` refuses N ≥ 2³⁵.

## CAP-9 (FR-11; AD-15)
- Document plus LUT, RECIP table and vectors suffice for an independent decoder; Python reference decoder (standard library only, written from the spec, not translated from Rust) agrees with the kernel on golden vectors. Agreement runs in CI but does not block the benchmark before spec freeze; required green at freeze and before public release.
- SM-5 gate: a clean-room decoder in a third language (for example C or Go), by a different person or a fresh sandboxed AI agent of a different model family (prompt and model ID recorded), given only the spec, LUT, RECIP table and vectors. It logs every ambiguity as a spec issue. Run at spec freeze, before first public release and on every format version bump.
- A valid-stream generator emits random valid Bitstreams covering every kind, length code, limit boundary and partial final Block, plus mutations; kernel, Python and clean-room decoders must agree byte for byte or all reject. A coverage table maps every reject rule and limit to at least one vector.
- After release, `spec/vectors/` is published as a conformance set in `valid`, `edge` and `faulty` groups.

## CAP-10 (FR-12; AD-12)
- Deterministic output. Writes a temp file in the target directory and renames on success; exit 0 on success, 1 on error with stderr message.

## CAP-11 (FR-13; AD-12)
- Kernel parse/decode entry points return `Result` and cannot panic on any byte sequence (`unwrap_used`, `panic`, `indexing_slicing` denied).
- Heap working set ≤ 64 KiB, asserted with a counting allocator. Mutation test over valid streams runs through the kernel and the Python reference decoder.

## CAP-12 (FR-14; AD-14)
- Arms: Whole-file raw; forced Constant / Previous-byte / Direct (+ Raw Block); Baselines-only; full MBSC. All use the same partition DP, selection key and Whole-file raw check, differing only in Candidate set.
- Report: size breakdown (header, parameter, coder payload, coder flush); adaptation overhead (ideal adaptive code length minus zero-order entropy, flush separate); win shares by family and Baseline by Blocks and bytes; zero-Residual share per class (tests the ~65% claim for WAV and text); Codebook version and hash, Block lengths, tool version, Corpus manifest hash, tuning-input hashes; the CAP-13 overlap checker's result (K and pass).
- Report goes to stdout and a JSON file; synthetic signals in a separate section; verdict per Real class with per-input wins and margin; overall verdict positive, marginal, inconclusive or negative (SM-1). Raw 16-bit PCM appears as non-counted rows beside the derived 8-bit PCM class.
- Work count per arm (model decisions and coder steps per input byte) is in the report; wall-clock, CPU model, thread count and toolchain go to a separate timing log excluded from determinism checks. Budget: one full run ≤ 24 h on one named host. If it is missed: first shrink the corpus (keeping ≥ 3 evaluation inputs per Real class), then lower the Codebook cap, reporting the excess bytes against the full cap on `tuning` inputs from the sidecar.
- Sidecar: `u16` matrix of exact size per Block offset, length code and Candidate id, layout documented, written by default for inputs ≤ 4 MiB and opt-in above, report states which inputs have one. Purpose: validate any later retrieval index (top-16 recall and excess bytes).

## CAP-13 (FR-15; AD-14)
- Manifest TOML; reports JSON. Non-redistributable corpora referenced by hash and source, not committed; fetched by a hash-verifying standard-library script. Inputs and roles per `corpora.md`.
- A derived input records its transform name and version, the source archive hash and the derived file hash.
- Overlap checker: no contiguous run of ≥ K bytes is shared between any tuning and any evaluation input; failure blocks the benchmark.
- Synthetic signals (one per Series family, e.g. sine, chirp) from integer-only code with a documented seed.

## Delivery sequencing
- Measurement core: CAP-1 to CAP-8, CAP-10 to CAP-13. Publication gate: CAP-9, running in parallel, mandatory before any public release, paying off only if SM-1 is positive.
- Review's prototype order: evaluator and modular reconstruction; exhaustive small Codebook; Baselines; one model; measure full encoded size; test synthetic separately from real; retrieval only if exhaustive search shows Gain.
- Numeric bounds deferred to architecture were set there: Whole-file raw header and FR-10 bound (N + 11 with the mandatory CRC), Codebook cap, decoder memory (64 KiB).
