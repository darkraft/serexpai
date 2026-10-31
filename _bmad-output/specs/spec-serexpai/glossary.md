# Glossary — MBSC v1

Terms are used exactly as written in code, spec and report.

- **Block** — Contiguous span of input bytes coded as one unit; the final Block may be shorter (partial Block).
- **Predictor** — Deterministic function yielding predicted byte ŷ per Block position: a Formula, constant prediction or previous-byte prediction.
- **Formula** — A Predictor from one Series family with fully parameterized, quantized parameters, evaluated by the Normative Evaluator.
- **Series family** — Polynomial, trigonometric or recurrence; each has its own parameter layout (`degree`, `order`, `m_mod`, `q`).
- **Normative Evaluator** — Fixed-point routine, defined exactly by the spec, evaluating a Formula identically on every platform.
- **Codebook** — Finite set of Formulas the encoder searches. Encoder-only; the decoder never reads it.
- **Candidate** — A Predictor, direct coding or Raw Block considered for one Block.
- **Residual** — δ = ((y − ŷ + 128) mod 256) − 128; y = (ŷ + δ) mod 256. The coded symbol is δ mod 256 (under direct coding, y itself).
- **Rate selector** — Encoder rule picking per Block the Candidate with the smallest actual encoded size (minimum description length, not rate-distortion).
- **Raw Block** — Block stored uncompressed with its framing and an unambiguous length.
- **Baseline** — Comparison method inside MBSC's own coding path: constant, previous-byte, or direct byte entropy coding.
- **Probability model** — The single fully specified model turning Residuals into binary-coder frequencies.
- **Bitstream** — Self-describing compressed output: file header, Block headers, Formula parameters, coder payload, and the 4-byte CRC-32 trailer.
- **Total encoded size** — Every byte of the Bitstream; never histogram entropy alone.
- **Input class** — Reporting category: synthetic signal, text, PCM, executable, already-compressed.
- **Real input class** — Text, PCM or executable drawn from the Corpus manifest.
- **Corpus manifest** — Versioned list of benchmark inputs: name, class, source, licence, SHA-256, role (`tuning` or `evaluation`).
- **Baselines-only arm** — Full MBSC with Formulas disabled; all other Candidates (Baselines, Raw Blocks, Whole-file raw) available; same Block lengths, partition rule and coding path applied independently. Comparator that makes Gain attributable to Formulas.
- **Whole-file raw mode** — Bitstream storing the whole input uncompressed behind a 3–7 byte header, followed by the 4-byte CRC trailer; bounds worst-case expansion at N + 11.
- **N** — Input length in bytes. **Rate** — Total encoded size in bits ÷ N (bits per input byte; raw threshold 8). As a ratio: `S_total / (8N)`.
- **Q** — Fixed-point quantization (fractional bits) of a Formula's parameters, in [8, 24].
- **Gain** — Any reduction in Total encoded size, summed over an Input class, for full MBSC versus both the Baselines-only arm and the best whole-input Baseline (smallest Total encoded size per input, summed over the class). No minimum margin; per-input wins and margin are also reported.
- **Work count** — Host-independent encode cost: model decisions and coder steps per input byte, per arm. Wall-clock lives only in the timing log.
- **Corpus-side transform** — Documented, versioned, hashed conversion applied by the fetch tool when defining a corpus input (e.g. 16-to-8-bit PCM). Not compressor preprocessing.
- **Kernel / Shell** — `mbsc-kernel` is the normative decode-side crate; encoder, bench and CLI crates depend on it, never the reverse.
