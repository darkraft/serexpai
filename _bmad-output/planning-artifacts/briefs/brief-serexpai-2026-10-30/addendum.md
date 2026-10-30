# MBSC Brief Addendum

Technical depth from `docs/mbsc_architecture_review.md` that belongs in the PRD and architecture, not the brief. It opens with the recommended action plan and what the review finds sound. Every later section covers an open issue.

## Recommended prototype order (from the review)

1. Normative scalar evaluator plus modular reconstruction.
2. Exhaustive enumeration over a small codebook.
3. Constant, previous-byte and direct entropy baselines.
4. One simple, fully specified block probability model.
5. Measure complete encoded size.
6. Test synthetic signals separately from text, PCM, executables and already-compressed inputs.
7. Add approximate retrieval (see "Candidate retrieval and performance") only if exhaustive search shows gains.

## What the review finds sound

- Modular residual: `δ = ((y − ŷ + 128) mod 256) − 128`, `y = (ŷ + δ) mod 256`. This reconstructs every byte for any deterministic byte-valued predictor, so predictor quality affects size, not losslessness.
- Predictor-plus-residual has precedent in FLAC. The open question is whether this predictor representation saves enough residual bits to pay for its parameters and modelling overhead.
- Encoder-only candidate search: if the stream carries all formula parameters and the decoder has a normative evaluator, the decoder needs neither the codebook nor its index.

## Rate objective and selector

- Rate is bits per input byte, so the raw threshold is 8, not 1. Use `R = S_total / (8N)` for a dimensionless ratio. Compare `S_compressed(N,T) < S_raw(N)`, counting framing, model signalling, rANS states, padding and payload-length fields on both sides.
- Call it a rate (minimum-description-length) selector, not rate-distortion, because reconstruction is always lossless.
- The proposed pseudo-entropy is not zero-biased: it scores a block of all 73s the same as a block of all 0s, its smoothed probabilities don't sum to 1, and it ignores model overhead. Use zero-order entropy `N·log2 N − Σ f_k·log2 f_k` only as a screen, and select on actual encoded bytes.

## Predictor representation (experiments to run separately)

| Input interpretation | Hypothesis |
|---|---|
| Raw byte stream | Formula prediction helps without format knowledge |
| Reversibly separated byte lanes | Serialization obscures useful structure |
| Parsed PCM sample/channel streams | Spectral or polynomial predictors help on real amplitudes |
| Symbolic text | Numerical series beat simpler byte/context models |

Any preprocessing must be reversible and fully signalled.

## Candidate retrieval and performance

- Feature similarity does not imply low residual entropy. Mean, variance, low-frequency energy and autocorrelation discard positional information, including sign or phase. Validate retrieval against the exhaustive minimum-bit-cost candidate: retrieve the top 16 through the feature index, then measure recall and excess compressed bytes. This gates any index work.
- Decide whether lookup retrieves fully parameterized formulas or families that are then fitted locally. Search and fitting needn't be bit-exact across encoders; only decoder-side evaluation is normative.
- A VP-tree is not LSH. Call it a "candidate-retrieval index" until an implementation is chosen.
- Work model: 16 candidates at three block lengths require evaluating 16 × (256 + 1024 + 2048) = 53,248 predicted positions per decision, before per-position series terms, features, histograms and coding. That work may consume only 256 input bytes if the smallest block wins, so a throughput target such as "1.5 GB/s" needs a named denominator: input bytes, candidate samples or operations.
- The claimed 65% residual-zero rate on WAV and text is a benchmark hypothesis, not a design requirement.

## Bitstream contract gaps to resolve

- Q field is 4 bits but Q ranges over [8,24]: define a mapping or widen the field.
- Count field is 4 bits but values run 1–16: store `count − 1`.
- `M` means different things across families: use family-specific layouts. Separate polynomial degree from parameter count (M+1), and trigonometric order from its 2M+1 coefficients. Rename the recurrence modulus (e.g. `m_mod`).
- Raw flag replaces the block-size flag: add an unambiguous raw length.
- Define the partial final block, the header parser (bit-level or byte-level), packed-bit order, a transmitted or deterministic rANS model, and normative framing and termination.
- Fixed-point semantics: intermediate widths, overflow, rounding (including negative halves), division and shift behaviour, trig argument reduction and LUT interpolation, coefficient limits, recurrence initialization, mathematical modulo for negatives.

## Context-adaptive rANS

- rANS decodes in reverse order of encoding. For forward-adaptive models, buffer coding decisions in forward order, then encode backward.
- The spec must state: which decoded symbols form each context, when updates occur, normalization and rescaling, model resets at block boundaries, and state/renormalization byte interleaving.
- "Local residual magnitude variance" must use already-decoded history or be transmitted. A decoder can't select a model from symbols it hasn't decoded yet.
