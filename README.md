# SerExpAI

**SerExpAI** (repository `serexpai`) is a lossless compressor.
Version **0.0.12**. `sxp-kernel` reads and writes whole-file raw, reconstructs a byte from any prediction, evaluates polynomial, trigonometric, and recurrence formulas, and codes binary decisions with the range coder. The command-line tool compiles and does not compress or decompress yet.

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

- `sxpc` writes a bitstream file from an input file.
- `sxpd` restores the original bytes.
- `sxpb` reports total encoded size per input class, for SerExpAI and for the
  baselines (constant prediction, previous-byte prediction, direct entropy coding),
  and checks that every round-trip is byte-identical.

Arguments are files. There is no standard-input or standard-output streaming in v1.

## What a bitstream contains

Whole-file raw is implemented. A file is magic `0x8D`, a flags byte (version 1,
raw mode), a minimal unsigned LEB128 length, the original bytes, and a CRC-32 of
those bytes. Output is at most N + 11 bytes. The empty file is 7 bytes.

The kernel can evaluate a polynomial formula, a trigonometric formula, and a
recurrence formula. The trigonometric evaluator uses the committed 1025-entry
sine table. It also codes binary decisions with the range coder and reports
the exact byte length. None of those results is written into a file yet. The
rest of format v1 is still design: per-block headers, formula parameters, and
a range-coder payload inside a Block. Those documents are listed below. This
release does not change the bitstream layout.

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

The Cargo workspace exists: `sxp-kernel`, `sxp-encoder`, `sxp-bench`, and
`sxp-cli`. `ci.sh` formats, lints, and tests through `cargo` and `rustup` on
`x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. Whole-file raw
round-trips, a predicted byte reconstructs exactly, polynomial, trigonometric,
and recurrence formulas produce the same prediction bytes on both targets, and
the binary range coder round-trips fixed decision schedules on both targets.
Block framing, the encoder, the benchmark, and the three commands above are
still absent.

The epic breakdown and the implementation-readiness report (2026-11-01, NEEDS
WORK) are on the `serexpai-bmad` branch. Reportable evaluation stays gated on
overlap K, the LJ Speech prefix check, the Debian tuning package, model
constants, the production Codebook recipe, and counsel review.
