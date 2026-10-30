# Input reconciliation: Product Brief (MBSC) vs PRD

- **Input:** `briefs/brief-serexpai-2026-10-30/brief.md` plus its `addendum.md`.
- **Compared against:** `prd.md` and `addendum.md` in this folder.
- **Missing input:** the brief folder has no `.memlog.md`. Only `brief.md` and `addendum.md` exist, so no decision log was reconciled. The brief's own text was treated as the decisions.
- **PRD files not edited.**

## Verdict

Coverage is high. Success criteria, scope in and out, vision, open questions 1–5 and the bitstream gaps all carry over. The items below are silent drops or distortions.

## Gaps

### G1. The research answer no longer isolates the formula (distortion, high)
- **Brief:** the success criterion says "the **best available formula** reduces total encoded size" against constant, previous-byte and direct-entropy baselines. The central claim is that *compact series* pay for themselves. The addendum prototype order and the brief's executive summary both frame the experiment as "do the formula predictors reduce total size".
- **PRD:** SM-1, FR-14 and "Gain" (§3) compare **MBSC total output** with the **best Baseline**.
  - FR-5 and the glossary make Baselines *Candidates* inside MBSC's per-Block search.
  - FR-7 lets a Baseline be forced for a whole input.
  - MBSC could therefore show a "Gain" only by switching between baselines per Block, or by choosing among Block lengths, without any Formula winning.
  - Nothing requires reporting Formula-attributable gain. For example: MBSC with Formulas against MBSC restricted to Baselines, or the share of Blocks and bytes won by a Formula.
- **Suggested fix:** define Gain against MBSC-with-baselines-only (same Block lengths and switching), or require the per-class report to show Formula win share. Add this to SM-1 and FR-14.

### G2. "Do not close off the eventual audience" is contradicted by PRD non-user and non-goal framing (distortion, medium)
- **Brief:** "decisions about the CLI, the bitstream specification and reproducible benchmarks should not close off that audience". The Vision includes a **published bitstream spec**, and CLI users and contributors are the eventual audience.
- **PRD:**
  - §2.2 lists "Library embedders" as non-users, and §2.2 and §5 promise **no bitstream compatibility between versions**. The brief never says either.
  - Only the compatibility promise is tagged `[ASSUMPTION]`. The library non-user is stated as fact.
  - FR-11's assumption cites the audience goal as the reason for the spec document, while §5 disclaims format stability. The two pull against each other and the PRD does not reconcile them.
- **Suggested fix:** tag the library non-user as an assumption. State that the format-version field (FR-9) is how the PRD keeps the audience door open despite no compatibility promise.

### G3. PRD tightens determinism that the brief addendum deliberately left loose (silent change, low–medium)
- **Brief addendum:** "Search and fitting needn't be bit-exact across encoders; only decoder-side evaluation is normative." This is a deliberate decision. It leaves room for later fitting or retrieval with non-bit-exact search.
- **PRD:** FR-5 and FR-12 require deterministic search and identical Bitstreams for the same input and Codebook, and SM-4 requires identical sizes. This is reasonable for v1 reproducibility, but it is neither flagged as a departure from the brief nor scoped to "v1 exhaustive search only". It could wrongly constrain the later fitting or retrieval work.
- **Suggested fix:** note that encoder determinism is a v1 reproducibility choice, and that the normative requirement stays decoder-side only.

### G4. Specifics dropped from brief and addendum (minor, mostly deferred to the brief addendum)
- **rANS ordering (addendum "Context-adaptive rANS"):** rANS decodes in the reverse order of encoding. Forward-adaptive models need decisions buffered forward, then encoded backward. FR-8 states the causality rule but not this ordering constraint. It matters for the later context-adaptive step, and also for v1 if any adaptation exists.
- **Concrete bitstream defects not carried as numbers:** Q is a 4-bit field over [8,24], and count is a 4-bit field over 1–16. FR-3 says "defined mapping or widened field" and "count − 1" without the ranges. The PRD points to the brief addendum for these, but the §0 pointer is generic.
- **Retrieval validation protocol:** the brief says retrieve the **top 16** through the feature index, then measure recall and excess compressed bytes. §5 keeps recall and excess bytes but drops "top 16" and the "gates any index work" framing.
- **Pseudo-entropy flaws:** the review notes the proposed pseudo-entropy is not zero-biased, scores all-73s the same as all-0s, has probabilities that don't sum to 1, and ignores overhead. FR-6 keeps only "optional screen". The exact screen formula `N·log2 N − Σ f_k·log2 f_k` is left to the brief addendum.
- **Brief qualitative points not carried:** the Chebyshev "text basis" hypothesis, and the principle that serialized bytes are often not the underlying signal. The second is carried only indirectly through Open Question 4.

## Confirmed carried over (no action)
- Lossless round-trip, including empty, partial and incompressible inputs: SM-2, FR-9.
- Raw fallback bound and true-size comparison: FR-6, FR-10, SM-3.
- Two real classes, any reduction counts, no minimum margin: SM-1.
- Per-class reporting with synthetic signals kept separate, and the 65% zero-rate claim treated as a hypothesis: FR-14.
- Scope in and out lists, including no throughput targets until a denominator is named: §5, §6.
- Open questions 1–5: PRD §8 items 1–5.
- Vision and negative-result outcome: §1.
