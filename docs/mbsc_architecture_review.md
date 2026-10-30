# MBSC Architecture Review

MBSC has a coherent lossless reconstruction mechanism, but the specification is not yet implementation-ready. The central issue is **validation**: the architecture assumes that compact mathematical series will produce cheaply coded residuals on real byte streams, but neither the feature representation nor the proposed basis families establish that.

I would resolve the compression objective and decoding contract before expanding the lookup tables, ANN index, or SIMD rANS implementation.

## 1. What is sound

The modular residual construction is correct:

\[
\delta_n=((y_n-\hat y_n+128)\bmod256)-128,
\qquad
y_n=(\hat y_n+\delta_n)\bmod256.
\]

For any deterministic byte-valued predictor, this reconstructs every input byte exactly. Predictor quality affects compressed size, not losslessness.

The broader predictor-plus-residual architecture also has established precedent: FLAC transmits predictor information and codes the resulting residuals. The important question is whether your particular predictor representation saves enough residual bits to pay for its parameters and modeling overhead. [1](https://www.rfc-editor.org/rfc/rfc9639.pdf)

Encoder-only candidate search is a useful architectural property. If the stream contains all formula parameters and the decoder has a normative evaluator, the decoder does not need the codebook or its search index.

## 2. Correct the rate objective and entropy estimate

### The raw-fallback threshold has inconsistent units

Your objective divides bits by bytes:

\[
R(N,T)=\frac{S_{\mathrm{Header}}(T)+S_{\mathrm{rANS}}(\Delta_N)}{N}.
\]

This measures bits per input byte, so the approximate raw threshold is 8, not 1.

For a dimensionless compression ratio, use:

\[
R(N,T)=\frac{S_{\mathrm{total}}(N,T)}{8N}.
\]

More importantly, compare against an actual raw-block representation:

\[
S_{\mathrm{compressed}}(N,T)<S_{\mathrm{raw}}(N).
\]

Include framing, probability-model signaling, rANS states, padding, and payload-length fields on both sides where applicable.

Because reconstruction is always lossless, “rate selector” or “minimum-description-length selector” is clearer than “rate-distortion selector.”

### The proposed pseudo-entropy is not zero-biased

Your expression

\[
-\sum_k f_k\log_2\left(\frac{f_k+\epsilon}{N}\right)
\]

treats all residual labels symmetrically. A block containing only residual 73 scores exactly the same as a block containing only residual 0.

Also:

- The smoothed probabilities do not sum to one.
- An all-identical block receives a slightly negative score.
- Adding \(\epsilon\) does not account for model initialization or transmission overhead.
- A marginal histogram does not capture the costs of your proposed contextual model.

A useful initial screening score is empirical zero-order entropy:

\[
\widehat S_0=N\log_2N-\sum_{k:f_k>0}f_k\log_2f_k.
\]

But final selection should measure actual encoded bytes, including metadata. If you intend to favor zero or small magnitudes, specify a probability model that actually assigns them higher probability.

## 3. The largest research risk is the predictor representation

### Raw bytes are not generally the underlying signal

A WAV file may contain interleaved multibyte samples. Treating its consecutive bytes as scalar amplitudes does not preserve the sample-domain signal that a sinusoidal model is meant to describe.

FLAC instead predicts audio samples using preceding samples, with explicit handling of sample precision, predictor coefficients, and residuals. That is a materially different representation from fitting smooth functions directly to serialized bytes. [1](https://www.rfc-editor.org/rfc/rfc9639.pdf)[3](https://www.rfc-editor.org/in-notes/prerelease/rfc9639.notprepped.xml)

Similarly, labeling Chebyshev polynomials as a text basis is a hypothesis, not an established property of text.

Consequently, I would treat these as separate experiments:

| Input interpretation | Hypothesis to test |
|---|---|
| Raw byte stream | Whether formula prediction helps without format knowledge |
| Reversibly separated byte lanes | Whether serialization obscures useful structure |
| Parsed PCM sample/channel streams | Whether spectral or polynomial predictors help on actual amplitudes |
| Symbolic text | Whether numerical series outperform simpler byte/context models |

Any preprocessing must itself be reversible and fully signaled.

### Feature similarity does not establish low residual entropy

Your retrieval system assumes:

\[
d(\mathbf V(y),\mathbf V(\hat y))\text{ small}
\quad\Longrightarrow\quad
S_{\mathrm{rANS}}(y-\hat y)\text{ small}.
\]

That implication needs experimental evidence.

Mean, variance, low-frequency energy, and autocorrelation discard substantial positional information. In particular, energy features can discard sign or phase information that matters for positionwise prediction.

Measure retrieval against a more direct reference:

1. Exhaustively evaluate a manageable codebook.
2. Identify the minimum-bit-cost candidate.
3. Retrieve the top 16 through the proposed feature index.
4. Measure both candidate recall and excess compressed bytes.

This separates “the basis family is ineffective” from “the search representation misses useful candidates.”

### Specify whether lookup retrieves formulas or initializations

Two distinct systems are currently mixed together:

- A finite codebook of fully parameterized predictors.
- A codebook of candidate families or initializations followed by local parameter fitting.

The latter needs an explicit fitting procedure and computation budget. The former needs a codebook coverage strategy.

Neither search nor fitting needs to be bit-exact across encoders if the final quantized parameters are transmitted. Only the decoder-visible predictor evaluation must be normative.

## 4. Repair the bitstream contract

Several current fields cannot represent what the specification requires.

| Issue | Required resolution |
|---|---|
| Four-bit Q field versus Q in [8,24] | Specify an explicit mapping or enlarge the field |
| Four-bit count versus values 1–16 | Define an offset encoding such as `stored = count - 1` |
| M means different things across families | Define family-specific parameter layouts |
| Polynomial degree M requires M+1 coefficients | Separate degree from parameter count |
| Trigonometric order M requires 2M+1 coefficients | Specify its independent limits and serialization |
| Recurrence uses M as a modulus | Rename it, for example to m_mod |
| Raw flag replaces the block-size flag | Add a raw length or another unambiguous length mechanism |
| Final block can contain fewer than 256 bytes | Define partial-block handling |
| Header exceeds one byte | Replace “read header byte” with a defined bit/byte parser |
| rANS model is unspecified | Define transmitted tables or deterministic model reconstruction |
| Payload length appears inconsistently | Make framing and termination normative |

Also specify bit order within bytes. Little-endian multibyte integers do not, by themselves, define packed-bit ordering.

For fixed-point evaluation, define more than the Q format:

- Intermediate widths and overflow behavior.
- Exact rounding, including negative halfway cases.
- Division and shift semantics.
- Trigonometric argument reduction and LUT interpolation.
- Coefficient limits and recurrence initialization.
- Mathematical modulo behavior for negative inputs.

## 5. Context-adaptive rANS needs a precise causal model

rANS behaves as a stack: decoding reverses encoding order. To decode symbols in forward order while using a forward-adaptive probability model, an encoder can first compute and buffer the coding decisions in forward order, then perform rANS encoding backward. This approach is explicitly used in adaptive rANS implementations. [4](https://ieeexplore.ieee.org/abstract/document/10283871)[14](https://github.com/eabelyaev/ABrANS)

Your specification therefore needs to state:

1. Which previously decoded symbols determine each context.
2. When probability updates occur.
3. How frequencies are normalized and rescaled.
4. Whether models reset at block boundaries.
5. How states and renormalization bytes are interleaved.

“Local residual magnitude variance” must refer to available history, or else be transmitted. A decoder cannot choose the probability model for a symbol using that symbol’s unknown value.

Interleaving also needs explicit state and byte ordering; it is not merely a SIMD implementation detail. [5](https://arxiv.org/html/2306.12141)

## 6. Reframe search terminology and performance targets

A VP-tree is not itself an LSH scheme. LSH uses hash families designed to make nearby objects collide with higher probability; a VP-tree partitions points by metric distance. Call the component a “candidate-retrieval index” until you choose its implementation. [8](https://www.cs.princeton.edu/~hy2/teaching/fall22-cos521/notes/NNS%20&%20LSH.pdf)[9](https://codeforces.com/blog/entry/54080)

The throughput targets also need an end-to-end work model. With 16 candidates at all three lengths, one decision evaluates:

\[
16(256+1024+2048)=53{,}248
\]

predicted positions, before accounting for the number of series terms per position, feature extraction, residual histograms, and entropy coding.

That work may consume only 256 input bytes if the smallest block wins. Thus, “1.5 GB/s fixed-point evaluation” needs a denominator: input bytes consumed, candidate samples evaluated, or arithmetic operations performed.

The claimed 65% residual-zero rate on WAV/text should remain a benchmark hypothesis, not a design requirement.

## 7. The most useful next step

Before implementing any of the three proposed detailed subsystems, build a small rate-measurement prototype:

1. Implement a normative scalar predictor evaluator and modular reconstruction.
2. Use exhaustive candidate enumeration over a small codebook.
3. Add constant prediction, previous-byte prediction, and direct byte entropy coding as baselines.
4. Start with one simple, fully specified block probability model.
5. Measure complete encoded size, not histogram entropy alone.
6. Test synthetic matching signals separately from representative text, PCM, executable, and already-compressed inputs.
7. Add approximate retrieval only after exhaustive search demonstrates worthwhile gains.

The decisive experiment is whether the best available formula reduces total encoded size relative to these simpler baselines. If it does, retrieval and SIMD become valuable engineering problems. If it does not, faster lookup will not fix the underlying representation.
