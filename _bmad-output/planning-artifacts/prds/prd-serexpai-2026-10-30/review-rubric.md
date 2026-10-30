# PRD Quality Review — MBSC (serexpai) v1

## Overall verdict
Adequate, leaning strong. The thesis is explicit and falsifiable ("v1 is therefore a rate-measurement experiment wrapped in a usable CLI"), both outcomes are declared acceptable, scope omissions are loud, and the counter-metrics (SM-C1/C2/C3) show real awareness of how the experiment could fool itself. The main risk is that the headline success metric, SM-1, does not yet cleanly isolate what the thesis is about (the *Formula's* contribution), and that v1 carries publication-grade spec obligations (FR-2, FR-9, FR-11, SM-5) justified by an assumed audience, which competes with the measurement goal.

## Decision-readiness — adequate
Decisions are stated as decisions: fixed-point only, exhaustive search, no retrieval index, one Probability model, no streaming, no checksum (§4, §5, §6.2). Trade-offs are named with what was given up (e.g. "Raw-byte 16-bit PCM interleaves a noisy and a smooth byte … choose PCM inputs … knowing a raw-byte result may be weak"). Open Question 2 ("Phase-blocker for architecture") is a genuinely open, correctly flagged blocker.

Weakness: the PRD names "modelling overhead" in the Vision as the thing that decides the experiment, but never surfaces the sharpest tension. The shortest allowed Block is 256 bytes (FR-5), and a transmitted Probability model (FR-8, OQ2) plus a per-Block header and Formula parameters must be paid inside that. That cost is likely to dominate the result, and the PRD does not flag it. A few Open Questions are also not really open.

### Findings
- **medium** Central overhead tension is not called out (§1, FR-5, FR-8, OQ 2, OQ 6) — Block lengths 256/1024/2048 and an undecided, possibly transmitted, Probability model interact directly: per-Block signalling at 256 bytes can exceed any Formula gain. No `[NOTE FOR PM]` marks this, and OQ 2 and OQ 6 are listed separately. *Fix:* add a `[NOTE FOR PM]` joining OQ 2 and OQ 6 and state the decision rule (e.g. minimum Block length is chosen after measuring per-Block overhead of the chosen model).
- **low** OQ 1 is not open (§8 item 1, FR-4 Out of Scope) — "v1 assumes the first" and FR-4 already commits to fully parameterized Formulas. It is a deferred later-version question. *Fix:* move it to a "Deferred" list or mark it resolved-for-v1.
- **low** OQ 7 duplicates the `[NOTE FOR PM]` in §6.2 (checksum / stdin-stdout) and mixes two unrelated questions. *Fix:* keep one location; split checksum from streaming.

## Substance over theater — strong
There is one real persona (the author) and an explicitly assumed audience; no persona theater. The Vision is specific to this product and could not swap into another PRD. NFRs are absent, and the one performance stance ("no throughput targets until an end-to-end work model names the denominator") is a decision rather than boilerplate. Counter-metrics carry real content (SM-C1 notes that synthetic-signal gain "proves only that the encoder can find a matching Formula").

### Findings
- **low** Novelty claim unhedged (§1 "novel lossless compressor") — `addendum.md` says "probably novel, with a shallow search." *Fix:* hedge in the Vision, or leave it and note the shallow search in §0.

## Strategic coherence — adequate
The thesis is clear: compact formulas, after paying parameter and modelling overhead, can beat simple baselines on real bytes. Scope follows from it (retrieval, SIMD and preprocessing deferred until "a Gain" exists), and the MVP is a coherent problem-solving/experiment scope. Counter-metrics exist and are tied to SM-1. The weaknesses are in how SM-1 would be *measured* and in a scope blend between "experiment" and "publishable spec."

### Findings
- **high** SM-1 does not attribute Gain to Formulas (SM-1, Glossary "Gain", "Rate selector", FR-5, FR-7) — The Rate selector picks per Block among Formulas *and* Baselines, while Baselines are "forced on for a whole input" (FR-7). MBSC can therefore beat the best whole-input Baseline merely by switching between constant, previous-byte and direct coding per Block (and by choosing Block length), with no Formula ever winning. SM-1 would then report success for a claim ("compact formulas … leave residuals cheap") it has not tested. *Fix:* add a comparator "MBSC with Formulas disabled" (per-Block selection among Baselines only, same Block-length choice), define Gain against it, and have the report state the share of Blocks and bytes won by each Series family.
- **medium** Gain has no margin and an ambiguous comparator (Glossary "Gain": "Any reduction … no minimum margin"; FR-14 "summed over the class's inputs") — One byte saved over a class sum, possibly from one large file, is "the research answer." "Best Baseline" is not defined as per input or per class aggregate. The trustworthy answer the Vision demands is weaker than the bar set here. *Fix:* choose a stated threshold or at least report per-input wins and the margin, define "best Baseline" at the class-aggregate level, and say how a marginal Gain is interpreted (the open-source-later decision).
- **medium** v1 scope is split between measurement and publication-grade spec (UJ-3 `[ASSUMPTION]`, FR-2, FR-9, FR-11, SM-5, §1 "negative result") — UJ-3 is an assumption, yet it is the justification for the independent-implementer standard in FR-2, FR-8, FR-9, FR-11, FR-15 and SM-5. The Vision says a negative result needs to be *reproducible*, not independently re-implementable. If the spec work slips, the measurement is blocked or the scope silently shrinks. *Fix:* tier the FRs: "must for measurement" (same-implementation determinism, round-trip, benchmark) and "publication gate" (independent-implementation spec, test vectors, SM-5), with the latter gated on a positive SM-1 or run in parallel.
- **medium** SM-1 reachability and tuning protection are deferred (OQ 4 note, SM-C2, FR-15) — SM-1 needs two Real classes, but the PRD itself expects text to lose and raw 16-bit PCM to be weak, leaving executables as the likely single win. Corpora are unchosen, and SM-C2 ("Hold out inputs or state the tuning set") is a counter-metric only, with no FR that makes the benchmark enforce a split or record the tuning set. *Fix:* add a requirement that the Corpus manifest marks tuning vs evaluation inputs, and state before corpora are chosen what SM-1 failing for reachability reasons (e.g. 8-bit PCM unavailable) would mean.

## Done-ness clarity — adequate
Most FRs have real testable consequences: FR-1 (all 256×256 byte pairs, property test), FR-6 (chosen size ≤ every other Candidate), FR-9 (empty, sub-Block and one-Block inputs round-trip), and FR-14 (mismatch fails the run). That is above typical. The gaps are in quantities left as adjectives and in a few under-specified behaviours that story creation will trip over.

### Findings
- **medium** Variable Block length has no partitioning semantics (FR-5: "Block length is one of a small defined set … and the Rate selector chooses among them", Glossary "Block") — How a mixed 256/1024/2048 sequence is formed (greedy, DP, aligned boundaries?) and how the partial final Block interacts are unstated, yet the result is "deterministic" (FR-5) and reported in FR-14. *Fix:* state the partition rule, or fix one Block length in v1 and vary it across benchmark runs instead.
- **medium** Bounds expressed as adjectives (FR-4 "small enough to search exhaustively"; FR-10 "a defined, documented overhead" and "a small constant"; FR-13 "never … allocates unbounded memory on hostile input"; SM-3) — A story writer cannot write the FR-10 test, the SM-3 check or the hostile-input test without numbers. *Fix:* give provisional bounds (e.g. Codebook ≤ N Formulas per family, Bitstream ≤ N + K bytes with K fixed, decoder memory ≤ a stated multiple of Block size) or explicitly mark them as set by architecture.
- **low** Under-specified Baselines (FR-7, Glossary "Predictor") — Which constant the constant Predictor uses (fixed, or a per-Block parameter) and what previous-byte prediction yields at the first byte of a Block are not stated. The mechanism for "forced on for a whole input" is also not in the CLI: §4.6 lists only `compress`, `decompress` and `benchmark` over file arguments. *Fix:* define these two Baselines in one sentence each and state how forcing is exposed (benchmark-only is fine).
- **low** "Every supported platform" is undefined (FR-2 test vectors) — *Fix:* name the v1 platform set (e.g. x86-64 and aarch64 Linux).
- **low** FR-6 screen vs exactness — "zero-order entropy only as an optional screen" can prune Candidates, which weakens the "≤ every other evaluated Candidate" guarantee and SM-1's meaning. *Fix:* state that the screen is off in benchmark mode.

## Scope honesty — strong
§5 Non-Goals and §6.2 do real work and match each other; every deferred item has a rationale ("premature before a rate result"). Inferences are tagged `[ASSUMPTION]` inline and the §9 index round-trips (13 inline tags, 13 index entries, all matching). Open-items density is 8 Open Questions, 13 assumptions and 2 `[NOTE FOR PM]` (23 total). That is acceptable for an exploratory, draft-status research project, but several items sit under SM-1 (OQ 2, OQ 4, OQ 6) and need resolving before architecture.

### Findings
- **low** Product name unconfirmed but untagged (title note "Confirm the product name") — It is a prose aside, not an `[ASSUMPTION]` or `[NOTE FOR PM]`, and not in OQs or §9. *Fix:* index it as an assumption or add an Open Question.
- **low** `[NOTE FOR PM]` density is low relative to real tensions — only OQ 4 and §6.2 use it; the Gain-attribution and overhead tensions above warrant them.

## Downstream usability — adequate
For a chain-top PRD feeding architecture and stories, the structure is good: global FR numbering, "Realizes UJ-n" on every FR, SMs naming the FRs they validate, and a Glossary that is mostly used consistently. Drift and undefined symbols remain, along with one dangling reference to an external document.

### Findings
- **medium** Undefined symbols and an external claim carried into an FR (FR-14 "the review's 65% claim for WAV and text"; FR-6 `S_total / (8N)`; FR-3 `Q`, `m_mod`; Glossary lacks "Rate", "N", "Q") — Pulled out alone, FR-14 cannot be understood without the architecture review, and `S_total` and `N` are never defined (Glossary has "Total encoded size" only). *Fix:* state the claim in a sentence in FR-14 and add Glossary entries for Rate, N, S_total (= Total encoded size) and Q.
- **low** Three "raw" notions (Glossary "Raw Block"; FR-10 "whole-file raw mode"; FR-14 "a raw Bitstream") are not reconciled, and "raw" is also used for Candidate options (Glossary "Candidate"). *Fix:* add a Glossary entry for whole-file raw mode and use it in FR-14.
- **low** Two documents called "addendum" (§0 "brief addendum holds … rate formulas, bitstream gaps, retrieval validation" vs OQ 4/OQ 5 `addendum.md`) — §0 says the FRs "point to" the brief addendum, but none do, so FR-9 and FR-6 have no pointer to the formulas and gaps. *Fix:* name the files explicitly and add pointers from FR-6 and FR-9.
- **low** UJ-3 protagonist is unnamed ("A reviewer") and `[ASSUMPTION]`, so the rubric's named-protagonist rule is not met; UJ-1 and UJ-2 are both "Darian."

## Shape fit — adequate
This is a hobby/solo research CLI with a technical-capability spec shape. The PRD's rigor is deliberately above hobby level, and the substance bar is met. Three UJs, two of them with one protagonist, is slight overhead but not harmful as an FR-tagging device. The over-formalization risk is in spec-layer detail and in the independent-implementation bar (see the strategic-coherence scope finding), not in persona or journey density.

### Findings
- **low** Implementation-level detail is in FRs (FR-3 "counts are stored as `count − 1`", "Q … through a defined mapping or a widened field"; FR-8 rANS "renormalization byte order and termination") — these are architecture or bitstream-spec decisions, and listing them as testable consequences blurs PRD vs spec. *Fix:* keep them as requirements on the spec's contents (as FR-2 does) and drop the specific encodings.

## Mechanical notes
- **IDs:** FR-1…FR-15, SM-1…SM-5, SM-C1…SM-C3, UJ-1…UJ-3 are contiguous and unique. All "Validates" and "Realizes" references resolve. Features 4.2, 4.5 and 4.6 list more UJs than their FRs do (e.g. 4.2 "Realizes UJ-1, UJ-2" while FR-4 to FR-6 cite UJ-1 only); harmless.
- **Assumptions Index:** 13 inline `[ASSUMPTION]` tags (§2.1, UJ-3, FR-2, FR-4, FR-5, FR-10, FR-11, FR-12, FR-13, FR-14, FR-15, §5, SM-5) and 13 index entries; round-trips cleanly. The FR-13 and FR-15 tags also carry "Confirm"/"see OQ 4" text beyond the index summary.
- **Glossary drift:** "Formula" is defined with "fully quantized parameters" but used as "fully parameterized Formulas" (FR-4, §8); "Block length" vs "nominal length" in the Block entry; "Rate" and `S_total`, `N`, `Q` undefined (see above). "Candidate" includes "raw options" while Baseline lists three items and Raw Block is separate.
- **Cross-refs:** FR-4 → OQ 1, FR-15 → OQ 4, §0 → §3/§4/§9 resolve. "The review" is used before it is defined in FR-14 (it is introduced only in §0 as `docs/mbsc_architecture_review.md`).
- **Required sections:** Vision, users, journeys, glossary, FRs, non-goals, MVP scope, SMs, open questions and assumptions are all present. There are no NFR sections. For this product the stances in FR-10, FR-13, SM-C3 and §5 largely cover them, but see the unbounded-quantity finding.

## Finding counts
- critical: 0
- high: 1
- medium: 7
- low: 12
