# Input reconciliation: `docs/mbsc_architecture_review.md`

Compared against `prd.md`, `addendum.md` and the brief addendum (`briefs/brief-serexpai-2026-10-30/addendum.md`). The PRD preserves nearly all of the review. That covers the rate-unit fix, MDL selector naming, pseudo-entropy critique, bitstream contract gaps, fixed-point semantics, causal rANS, retrieval validation and the work-model note.

Items intentionally out of v1 scope are excluded: retrieval index, SIMD or interleaved rANS, format-aware preprocessing, throughput targets. Items already in the brief addendum are excluded too.

## Gaps (PRD silently drops or distorts)

### G1. Cross-length candidate comparison is undefined (distortion, medium)
- **Input (§6 work model, §2):** with 256, 1024 and 2048-byte candidates, "that work may consume only 256 input bytes if the smallest block wins". A decision's candidates therefore cover different input spans.
- **PRD:** FR-5 lets the Rate selector choose among Block lengths. FR-6 requires "the chosen Candidate's actual encoded size is less than or equal to every other evaluated Candidate's".
- **Problem:** raw encoded sizes of Candidates covering 256 vs 2048 bytes are not comparable. FR-6 as written is either trivially satisfied by always picking 256, or ill-defined. The PRD needs a normalised metric (bits per input byte over the covered span) or a segmentation rule (for example a dynamic program over the whole input). It also needs a tie-break and a determinism rule. The PRD never says how the selector handles span differences or how the next Block's start is determined.
- **Suggested handling:** add a FR-5/FR-6 consequence and an Open Question under Q6 (Block length set).

### G2. SM-1 "Gain" can be earned by baseline switching, not by Formulas (distortion, high)
- **Input (§1, §2, §7 "decisive experiment"):** "whether the best available formula reduces total encoded size relative to these simpler baselines". Predictor quality must pay for its own parameters and modelling overhead.
- **PRD:** the Rate selector chooses per Block among Formulas, Baselines and Raw (Candidate glossary, FR-5, FR-7). SM-1 and FR-14 compare MBSC (which includes per-Block Baseline Candidates) against the best whole-input Baseline.
- **Problem:** a Gain could come entirely from per-Block mixing of constant, previous-byte and direct coding, with zero Formula wins. That would answer a different question from the review's decisive one, and the vision statement ("does the best available formula reduce...") would be falsely confirmed.
- **Suggested handling:** the benchmark needs a Formula-excluded MBSC arm (Baselines-only, per-Block switching) as the comparator. Alternatively, report the Gain attributable to Formulas.

### G3. No overhead or winner breakdown, so the "pays for its parameters" question and the diagnostic split are unobservable (silent drop, medium)
- **Input (§1, §3 retrieval-vs-basis separation, §7):** the core question is whether residual savings exceed parameter and model overhead. The review wants to separate "basis family is ineffective" from "search representation misses useful candidates", using the exhaustive minimum-cost candidate as the oracle.
- **PRD:** FR-14 reports only Total encoded size per class, plus the residual-zero share. It records neither the split (header, parameter, model-signalling and rANS payload bytes), nor the per-class Block-winner distribution (Formula by family, per Baseline, Raw), nor per-Block minimum-cost candidate data.
- **Problem:** a negative or marginal result cannot be diagnosed as "overhead eats the gain" vs "formulas don't predict". The PRD also says exhaustive search is the reference a later index must be measured against (§5), but nothing requires the oracle results (per-Block best candidate and its cost) to be retained or emitted for that later comparison.
- **Suggested handling:** add FR-14 consequences for a per-class size breakdown, a winner histogram, and an exportable per-Block oracle table.

## Minor / not flagged as gaps
- The PRD attributes "decoder uses no floating point" to the review (FR-2 assumption). The review only discusses fixed-point evaluation. This is harmless, but the attribution is looser than the source.
- The review's note that the polynomial "Chebyshev text basis" is a hypothesis is carried only indirectly (OQ4 note: text may lose). The PRD does not state the polynomial basis or argument normalisation, though FR-2 and FR-4 may cover that in architecture.
