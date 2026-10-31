# SerExpAI

**SerExpAI** (repository `serexpai`) is a lossless compressor, still at the design stage.
Version **0.0.6**. There is no code and no command-line tool yet.

It represents **MBSC** (Multi-Basis Series Compression System): the idea of predicting
each byte of a block from a compact formula (polynomial, trigonometric or recurrence)
and entropy-coding the modular residual with a binary range coder.
Reconstruction is exact for any deterministic predictor, so a poor predictor costs
size, never correctness. The encoder searches a codebook of formulas. The decoder
only evaluates the formula whose parameters are in the bitstream, so it needs
neither the codebook nor an index.

The central claim is untested: that those formulas, once their parameters and
modelling overhead are paid for, leave residuals cheap enough to beat simple
baselines on real byte streams. v1 is a rate-measurement experiment wrapped in a
CLI. A positive result, by a margin large enough to act on, is the reason to grow
SerExpAI into a practical compressor. A negative result stays in the repository as a
reproducible negative result. An unreproducible result is the outcome to avoid.

v1 is not a production compressor. It has no throughput target and promises no
bitstream compatibility between 0.x versions.

## Planned commands

Once the CLI exists:

- `compress` writes a bitstream file from an input file.
- `decompress` restores the original bytes.
- `benchmark` reports total encoded size per input class, for SerExpAI and for the
  baselines (constant prediction, previous-byte prediction, direct entropy coding),
  and checks that every round-trip is byte-identical.

Arguments are files. There is no standard-input or standard-output streaming in v1.

## What a bitstream contains

A self-describing stream: a versioned header, per-block headers and formula
parameters, a range-coder payload, and a CRC-32 of the original bytes. Output is
at most N + 11 bytes. The format is specified in the design documents below and
is not implemented yet.

## Documents

On this branch:

- [`docs/mbsc_architecture_review.md`](docs/mbsc_architecture_review.md) is the
  architecture review that started the project. Parts of it predate later
  decisions, including the choice of a range coder over rANS.
- [`docs/CHANGELOG.md`](docs/CHANGELOG.md) records releases.
- [`docs/decision_logs/`](docs/decision_logs/) records how design choices were made.

The product brief, PRD, architecture spine, bitstream spec seed, the spec
(`SPEC.md`) and the epic and story breakdown are modelling artifacts. They are
kept off this branch and preserved on the append-only `serexpai-bmad` branch,
under `_bmad-output/`. The spec is the contract later implementation builds
against. The brief's mention of rANS is superseded: format v1 uses a range coder.

## Status

Design only. The planned Cargo crates are `sxp-kernel`, `sxp-encoder`,
`sxp-bench` and `sxp-cli`. The epic and story breakdown is on the
`serexpai-bmad` branch. The next step is an implementation-readiness check,
then the measurement core: the evaluator, the codebook search, the range coder,
and the three commands above.
