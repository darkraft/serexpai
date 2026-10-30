---
title: "Product Brief: MBSC"
status: complete
created: 2026-10-30
updated: 2026-10-30
---

# Product Brief: MBSC

Source of truth: `docs/mbsc_architecture_review.md`. Technical detail from it is preserved in `addendum.md`.

## Executive Summary

MBSC is a novel lossless compressor and decompressor, delivered as a command-line tool. It predicts each input byte from a compact mathematical series (polynomial, trigonometric or recurrence), then entropy-codes the modular residual `y - ŷ` with rANS. The full design uses context-adaptive rANS; v1 uses one simple, fully specified model. Reconstruction is exact for any deterministic predictor, so predictor quality only changes compressed size. The encoder searches a codebook of candidate formulas, and the decoder only needs a normative evaluator, not the codebook or its index.

The project is passionate research that should end as a usable tool. Its central claim is untested: that compact series, once their parameters and modelling overhead are paid for, yield residuals cheap enough to beat simple baselines on real byte streams. An architecture review found the design coherent but not implementation-ready, and recommended answering that question with a small rate-measurement prototype before building lookup tables, an ANN index or SIMD rANS.

This brief frames the first version as that experiment, wrapped in a CLI. If the formula predictors reduce total encoded size against the baselines, retrieval and SIMD become worthwhile engineering. If they don't, the project has produced an honest negative result cheaply.

## The Problem

General-purpose compressors model bytes through dictionaries and context mixing. Predictor-plus-residual coding is proven in domains such as FLAC audio, where the predictor works on parsed samples, not serialized bytes. Whether compact series fitted directly to raw bytes can compete is unknown, and the review finds the evidence missing. Serialized bytes are often not the underlying signal: interleaved multibyte WAV samples are one example. Likewise, labelling Chebyshev polynomials a text basis is only a hypothesis.

Without that evidence, engineering effort on retrieval indexes, SIMD kernels and bitstream details may go into a representation that saves nothing.

## The Solution

A CLI that compresses a file into a self-describing bitstream and restores it byte-exactly.

- **Encoder:** splits the input into blocks, evaluates candidate series predictors (exhaustively over a small codebook at first), and picks the option with the smallest total size. The options include raw fallback and baselines such as constant and previous-byte prediction.
- **Stream:** carries every formula parameter plus framing and a fully specified probability model, so the decoder needs no codebook.
- **Decoder:** runs a normative fixed-point predictor evaluator, then adds the residual modulo 256 to reconstruct each byte.
- **Benchmark mode:** reports total encoded size against the baselines on synthetic signals, text, PCM, executables and already-compressed data.

## What Makes This Different

The idea is the formula-as-predictor representation, with a decoder that needs no codebook. The moat is a research hypothesis, not an established advantage. The review is explicit that the link between feature similarity and low residual entropy, and the retrieval premise built on it, are hypotheses to benchmark, not design requirements.

## Who This Serves

- **Primary:** the author, as researcher and builder, who wants a trustworthy answer on whether the approach works and a usable tool if it does.
- **Eventual audience:** a large open-source community of CLI users, researchers and contributors. v1 serves the author's research question, but decisions about the CLI, the bitstream specification and reproducible benchmarks should not close off that audience.

## Success Criteria

- **Correctness:** lossless round-trip on every input, including empty files, partial final blocks and incompressible data. Raw fallback never expands the input beyond a defined, documented overhead.
- **Research answer:** on representative corpora, the best available formula reduces *total* encoded size (headers, model signalling, rANS state and padding included) against constant, previous-byte and direct entropy-coding baselines on **at least two distinct real input classes** (for example PCM and text). Any reduction counts as a gain; no minimum margin is required. Already-compressed inputs must show no regression beyond the defined raw-fallback overhead.
- **Rate accounting:** the selector minimizes actual encoded bytes, and raw fallback is compared against the true size of a raw block, framing included. Formulas are in the addendum.
- **Reporting:** results are reproducible and reported per input class, with synthetic signals that match the predictor families kept separate from text, PCM, executables and already-compressed data. The claimed 65% residual-zero rate on WAV and text is a benchmark hypothesis, not a requirement.

## Scope

**In for v1:**
- Normative scalar predictor evaluator with modular reconstruction.
- Small, exhaustively searched codebook.
- Baselines: constant, previous-byte, direct byte entropy coding.
- One simple, fully specified block probability model.
- Complete bitstream contract: bit order, field widths and offsets, partial blocks, framing and termination.
- CLI `compress`, `decompress` and benchmark.

**Out for v1:**
- ANN or other candidate-retrieval index.
- SIMD or interleaved rANS.
- Context-adaptive modelling beyond one simple model.
- Format-aware preprocessing, such as parsed WAV channels, reversibly separated byte lanes or symbolic text. The review treats each as a separate experiment, and any such preprocessing must be reversible and fully signalled.
- Throughput targets, until an end-to-end work model names the denominator (input bytes, candidate samples or operations).

## Vision

If the experiment succeeds, MBSC grows into approximate retrieval over larger codebooks, reversible format-aware preprocessing and fast SIMD entropy coding, ending as a practical open-source compressor for a large audience, with a published bitstream spec. Any retrieval index must first be validated against exhaustive search, measuring recall and excess compressed bytes. If the experiment fails, the repo stays as a reproducible negative result.

## Open Questions

- Does lookup retrieve fully parameterized formulas, or candidate families that are then fitted locally? The first needs a coverage strategy, the second a fitting procedure and compute budget.
- Which rANS probability model is the single simple one for v1, and how is it transmitted or reconstructed?
- Implementation language for the CLI? Decide in architecture.
- Which real input classes and corpora count as representative for the two-class success criterion?
- Licence and contribution model: not yet determined. Decide before the first public release.
