# PRD ↔ SPEC reconciliation extract (2026-10-31)

PRD: `prd.md` (+ `addendum.md`), updated 2026-10-31. Spec: `_bmad-output/specs/spec-serexpai/SPEC.md` with `acceptance-criteria.md` (AC), `corpora.md`, `glossary.md`. Architecture spine (AD-n) is cited only where the AC points to it. Pre-update PRD (`/tmp/prd.before.md`) was used to see which passages were edited today. Detail that the PRD hands to the spec by reference is not flagged.

Summary: 9 contradictions, 13 omissions, 3 over-copied items.

## Contradictions

### C1. FR-8 body still describes rANS causal-model rules
- **PRD:** §4.4 FR-8 body: "The brief addendum lists the causal-model rules (reverse-order decode, buffering of coding decisions)."
- **Spec:** AC CAP-6 "decisions encoded in forward order, carry via cache"; SPEC Constraints, patent safeguards: "no deferred update, no pending-symbol state". Reverse-order decode and buffered decisions are rANS properties (brief addendum §"Context-adaptive rANS", line 57), and buffering is exactly the deferred-update pattern the safeguards forbid.
- **Fix:** Replace the sentence with: "The causal-model rules and the decoder's per-decision order are stated in the spec (SPEC Constraints, patent safeguards; AC CAP-6)."

### C2. PRD still points to the brief addendum (rANS-era) as its technical source
- **PRD:** §0 ("the architecture review … the only specification input"; "the brief addendum … holds the technical detail … that FR-6, FR-8 and FR-9 point to"); FR-3 ("see the brief addendum"); FR-6 ("formulas for rate and the raw comparison are in the brief addendum"); FR-8; FR-9 ("the review's list of contract gaps is in the brief addendum").
- **Spec:** SPEC header "Canonical contract … Source documents … are for traceability only"; brief addendum is listed under `sources:`, not `companions:`. The brief addendum still says "rANS states" in the raw comparison (line 23), keeps zero-order entropy "as a screen" (line 25, contradicting FR-6 and AC CAP-4), proposes a 4-bit Q field (line 48) and a "transmitted or deterministic rANS model" (line 52). SPEC Assumptions also say the review was not re-read.
- **Fix:** In §0 call the spec the canonical technical contract and the review/brief addendum traceability sources. Repoint FR-3 → AC CAP-2, FR-6 → AC CAP-4, FR-8 → AC CAP-6, FR-9 → AC CAP-7 (and the spec seed). Keep the brief addendum only as "rationale".

### C3. FR-14 size breakdown lists a byte category that does not exist and drops the one that does
- **PRD:** §4.7 FR-14: "breaks Total encoded size into header, Formula parameter, model signalling and payload bytes". FR-8: "Per-Block model signalling cost is measured and reported".
- **Spec:** AC CAP-12 (AD-14): breakdown is "header, parameter, coder payload, coder flush" plus "adaptation overhead (ideal adaptive code length minus zero-order entropy, flush separate)". AC CAP-6 / PRD §8: the model transmits nothing, so there are no model-signalling bytes; the fixed per-Block cost is the 4-byte coder flush (SPEC Assumptions).
- **Fix:** FR-14: "header, Formula parameter, coder payload and coder flush bytes, plus the model's adaptation overhead (reported separately from the flush)". FR-8: reword "model signalling cost" to "per-Block fixed cost (header, flush) and adaptation overhead". Note: AC CAP-6 itself still says "Per-Block model signalling cost measured and reported", so the spec wording should be aligned too.

### C4. FR-14 puts the per-Block Candidate costs in the report, for every input
- **PRD:** FR-14: "The report retains, per Block, the encoded cost of every evaluated Candidate".
- **Spec:** AC CAP-12: a separate sidecar (`u16` matrix by Block offset, length code, Candidate id), "written by default for inputs ≤ 4 MiB and opt-in above, report states which inputs have one"; CAP-12 success: "per-Block Candidate-cost sidecar".
- **Fix:** "The benchmark writes a per-Block Candidate-cost sidecar (by default for inputs up to a size stated in the spec, opt-in above), and the report states which inputs have one."

### C5. FR-15 fixes the overlap threshold that the spec (and PRD OQ 2) leave open
- **PRD:** FR-15: "no tuning input shares a run of 1 KiB or more with any evaluation input". §8 OQ 2 lists "the 1 KiB overlap threshold" among "values [that] win on tuning inputs".
- **Spec:** corpora.md Rules "K = 1 KiB, PROPOSED" and Unresolved; AC CAP-13 "≥ K bytes"; SPEC Open Questions "Is K = 1 KiB right for the overlap checker?" — a sanity question, not a value tuned on tuning inputs.
- **Fix:** FR-15: "a run of K bytes or more (K proposed as 1 KiB; OQ 2)". Move K out of OQ 2's "which values win on tuning inputs" list into its own question ("Is K = 1 KiB right?").

### C6. FR-3 presents settled field ranges as "the review's examples"
- **PRD:** FR-3: "(the review's examples: Q in [8,24], counts 1 to 16; see the brief addendum)". Glossary Q has no range.
- **Spec:** AC CAP-2 "Every field covers its full range (Q in [8, 24], counts 1 to 16, via offset encoding)"; spec glossary "Q … in [8, 24]".
- **Fix:** FR-3: "Every field covers the v1 ranges: Q in [8, 24] and counts 1 to 16 (encoding in the spec)." Add "in [8, 24]" to Glossary Q.

### C7. §4.4 / FR-8 still hedge on a model the spec has fixed
- **PRD:** §4.4 Description "Whatever the model is, it must be causal"; FR-8 "If any adaptation exists, the spec states … whether the model resets at Block boundaries"; FR-8 "how frequencies are normalized and rescaled".
- **Spec:** AC CAP-6 "adaptive order-0 over an 8-decision bit tree, rebuilt by the decoder …, reset every Block"; SPEC Constraints "the model resets at each Block start"; the update is a probability with a RECIP-table rate and count cap, not frequency-count rescaling. PRD §8 already records "adaptive order-0, rebuilt by the decoder, nothing transmitted".
- **Fix:** State at requirement level: "v1's model is adaptive order-0, rebuilt by the decoder from decoded symbols, nothing transmitted, reset at every Block start; the spec states its update rule, contexts and coder mapping." Drop "if any adaptation exists" and "rescaled".

### C8. §6.1 places FR-2's test vectors in the publication gate
- **PRD:** §6.1 "*Publication gate:* FR-11, the independent-implementation test vectors in FR-2, and SM-5."
- **Spec:** AC Delivery sequencing "Measurement core: CAP-1 to CAP-8 … Publication gate: CAP-9"; CAP-2 (FR-2, FR-3) success, including the published vectors on both platforms and the independent evaluator match, is in the measurement core.
- **Fix:** Either "Publication gate: FR-11 and SM-5" (FR-2 vectors stay core) or move the independent-evaluator check in CAP-2 to CAP-9 in the spec. Pick one so both documents agree.

### C9. Codebook composition is an open question in the PRD but not in the spec
- **PRD:** §8 OQ 3 "What is the Codebook's composition within the cap …? Owner: Darian, before the first benchmark run."
- **Spec:** SPEC Open Questions lists only the tuning constants, K and the LJ Speech prefixes; AC CAP-3 / AD-13 treat composition as the output of the documented integer-only recipe.
- **Fix:** Add the question to SPEC Open Questions (preferred, since nothing in the spec fixes it), or reword PRD OQ 3 as "set by the recipe; no PRD decision needed".

## Omissions

### O1. Out-of-range parameters are rejected, never clamped
- **PRD:** FR-2 / FR-3 (coefficient limits named, behaviour at the limit not).
- **Spec:** CAP-2 success "out-of-range parameters are rejected, never clamped"; AC CAP-2 "Boundary vector and rejection vector for every limit"; AC CAP-7 reject set "out-of-limit parameters".
- **Fix:** Add to FR-2: "A decoder rejects any parameter outside its limits rather than clamping it; each limit has a boundary vector and a rejection vector."

### O2. Format-version bump rule (and FTO trigger)
- **PRD:** FR-9 (format version), FR-8 (only rANS adoption bumps the version), §5.
- **Spec:** SPEC Constraints "Any change to the Evaluator, sine LUT, RECIP table, model constants, coder, checksum or Bitstream layout bumps the format version and regenerates the golden vectors in the same change"; patent safeguards "Changing any of this requires a new freedom-to-operate review".
- **Fix:** Add to FR-9: "Any change to the Evaluator, normative tables, model constants, coder, checksum or layout bumps the format version and regenerates the test vectors in the same change." Add to FR-8: "Changing the decoder's per-decision structure or adding coder-state, symbol-width or probability-table fields needs a new freedom-to-operate review."

### O3. Blocks are self-contained
- **PRD:** FR-2 (mentions "recurrence initialization"), FR-5, FR-8.
- **Spec:** SPEC Constraints "Formulas depend only on Block-local position and their own parameters; Blocks are self-contained and the model resets at each Block start"; patent safeguards "No coder state crosses a Block boundary".
- **Fix:** Add to FR-5 (or FR-2): "Each Block decodes from its own header and payload alone: Formulas use Block-local positions, and no model or coder state carries across a Block boundary."

### O4. Determinism must hold across thread counts
- **PRD:** FR-5 "deterministic for a given input and Codebook"; FR-12.
- **Spec:** CAP-3 success "identical across runs and thread counts"; AC CAP-3 "Parallel run is byte-identical to a single-threaded run"; SPEC Constraints "independent of thread count".
- **Fix:** FR-5 / FR-12: "…and identical whatever the thread count; a test compares a parallel run against a single-threaded one."

### O5. Tie-break rules of the Rate selector
- **PRD:** FR-6 (covers only Raw Block vs compressed).
- **Spec:** AC CAP-4 "ties never favour a Formula over a Baseline; Whole-file raw wins ties and the empty input".
- **Fix:** Add to FR-6: "On equal size, a Baseline beats a Formula, and Whole-file raw beats everything (including for the empty input)." This is what keeps SM-1's "Gain attributable to Formulas" honest.

### O6. Input-size limit and scope of the N + 11 bound
- **PRD:** FR-10 / SM-3 ("at most N + 11 bytes", unconditional); FR-12.
- **Spec:** CAP-8 success "(format versions 1–14)"; AC CAP-8 "`compress` refuses N ≥ 2³⁵".
- **Fix:** FR-12: "`compress` refuses inputs of 2³⁵ bytes or more with a non-zero exit." FR-10: "for every format version up to 14 (15 is reserved)".

### O7. Python reference decoder, valid-stream generator and differential agreement
- **PRD:** FR-11 (document only); SM-5 refers to "generated valid Bitstreams" and "the author's own Python reference decoder", but no FR creates either.
- **Spec:** CAP-9 "golden vectors and an in-repo reference decoder"; "kernel, Python reference decoder and clean-room decoder agree under differential fuzzing on both platforms"; AC CAP-9 Python decoder "standard library only, written from the spec, not translated from Rust", valid-stream generator with mutations, coverage table mapping every reject rule and limit to a vector, CI agreement not blocking before freeze.
- **Fix:** Add consequences to FR-11: a Python reference decoder written from the spec; a valid-Bitstream generator with mutations; every reject rule and limit covered by a vector; kernel and Python decoders agree in CI, required green at spec freeze and before public release.

### O8. Report verdict is three-valued
- **PRD:** FR-14 "The verdict line states, per Real input class, whether Gain was achieved".
- **Spec:** CAP-12 success "verdict is positive, inconclusive or negative per SM-1".
- **Fix:** FR-14: "…and an overall verdict of positive, inconclusive or negative as defined by SM-1."

### O9. What makes a Real input class "testable"
- **PRD:** SM-1 "If fewer than two Real input classes can be tested …"; FR-15.
- **Spec:** corpora.md Rules "At least 3 evaluation inputs per Real class, 4 to 16 MiB per class"; equal-size text/executable slices for equal weight; "the report states [within-class correlation] beside per-input wins"; AC CAP-12 budget fallback keeps "≥ 3 evaluation inputs per Real class".
- **Fix:** FR-15: "Each Real input class has at least three evaluation inputs (size range in `corpora.md`); a class with fewer cannot be tested for SM-1. The report notes known correlation between evaluation inputs of a class."

### O10. Tuning-procedure rules and their audit trail
- **PRD:** FR-8 and SM-C2 (model constants on Baselines-only arm); FR-15 (overlap checker); FR-14 (report metadata).
- **Spec:** SPEC Constraints "Formula widths are tuned on full MBSC; ties go to the simpler setting (smaller CAP, DELTA = 1, scalar initial probability)"; AC CAP-6 "The grid and every score are recorded"; AC CAP-3 Codebook "records its tuning-input hashes"; AC CAP-12 report records "tuning-input hashes; the CAP-13 overlap checker's result (K and pass)"; AC CAP-13 checker "failure blocks the benchmark".
- **Fix:** FR-8: "Formula coefficient and frequency widths are scored on full MBSC; ties go to the simpler setting; the tuning grid and every score are recorded." FR-14: add tuning-input hashes, Codebook hash and the overlap-checker result to the recorded metadata. FR-15: "an overlap failure blocks the benchmark until the tuning input is replaced".

### O11. What to do if the 24-hour budget is missed
- **PRD:** FR-4 (cap justified by the FR-14 budget), FR-14 (24 h on one named host).
- **Spec:** AC CAP-12 "If it is missed: first shrink the corpus (keeping ≥ 3 evaluation inputs per Real class), then lower the Codebook cap, reporting the excess bytes against the full cap on `tuning` inputs from the sidecar."
- **Fix:** Add to FR-14: "If a full run exceeds the budget, the corpus shrinks first (keeping the FR-15 minimum per class), then the Codebook cap is lowered, and the report states the excess bytes this costs on tuning inputs."

### O12. Forced-Baseline arms share the full selection machinery
- **PRD:** FR-7 ("Each Baseline can be forced on for a whole input"); Glossary Gain ("best whole-input Baseline").
- **Spec:** AC CAP-12 "forced Constant / Previous-byte / Direct (+ Raw Block) … All use the same partition DP, selection key and Whole-file raw check, differing only in Candidate set".
- **Fix:** FR-7: "A forced-Baseline arm still uses Raw Blocks, the same partition rule and the Whole-file raw check; only the Candidate set differs." This fixes the comparator SM-1 is decided against.

### O13. Glossary gaps
- **PRD:** §3 Glossary; "work count" is used in FR-14 and §5 but not defined; Q has no range (see C6).
- **Spec:** glossary "Work count — Host-independent encode cost: model decisions and coder steps per input byte, per arm. Wall-clock lives only in the timing log."
- **Fix:** Add a Work count entry to §3.

## Over-copied detail

### X1. Partition algorithm in FR-5
- **PRD:** FR-5 "dynamic programming over a 256-byte grid on exact Block sizes".
- **Spec:** CAP-3 states only "a deterministic partition rule"; the algorithm is AD-9 ("Partition by free dynamic programming on a 256-byte grid"), referenced from AC CAP-3.
- **Fix:** "The partition rule is deterministic, selects on exact size, and is stated in the spec (AC CAP-3)." Drop the grid and the algorithm name.

### X2. Corpus roster in FR-15
- **PRD:** FR-15 "Canterbury and Calgary for tuning, Silesia slices for text and executable evaluation, 8-bit PCM derived from LJ Speech, two pinned Debian binaries for amd64 and arm64, and FLAC test files plus `enwik8.zip` as the control."
- **Spec:** corpora.md Inputs table. The PRD copy is already lossy: PCM tuning comes from LJ Speech, not Canterbury/Calgary; executable evaluation is Silesia `ooffice` plus `busybox-static` and `ripgrep` for two architectures (5 inputs), not "two pinned Debian binaries".
- **Fix:** Keep the class-level requirement (text, 8-bit PCM, executable, already-compressed control; tuning and evaluation drawn from different sources) and point to `corpora.md` for the roster.

### X3. Header-byte decomposition of the expansion bound
- **PRD:** FR-10 "(a header of up to 7 bytes plus the 4-byte CRC)"; Glossary Whole-file raw "behind a 3–7 byte header".
- **Spec:** AC CAP-8 "2 header bytes + at most 5 LEB128 bytes + 4 CRC bytes"; spec glossary.
- **Fix:** Keep "at most N + 11 bytes" as the requirement; drop the byte breakdown (or move it to a pointer to AC CAP-8). Low priority: harmless while the format is frozen, but it will drift if the header changes.
