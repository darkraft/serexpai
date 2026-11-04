# SerExpAI

**SerExpAI** (repository `serexpai`) is a lossless compressor.
Version **0.0.19**. `sxp-kernel` reads and writes whole-file raw, reconstructs a byte from any prediction, evaluates polynomial, trigonometric, and recurrence formulas, codes binary decisions with the range coder, adapts an order-0 probability model, and round-trips a baseline or formula Block in blocked mode. `sxp-encoder` selects Raw, Direct, Previous-byte, and Constant blocks by exact encoded size, and searches a labelled fixture codebook of regenerable formulas. `sxp-cli compress` writes that search, and `sxp-cli decompress` restores the file. `sxp-cli benchmark` reports six arms on the three synthetic signals and stays exploratory while the corpus gate is open. `tools/fetch` checks a committed corpus manifest and does not download it. The corpus gate is not ready.

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

## Commands

`sxp-cli compress <src> <dst>` writes a bitstream. It searches the labelled
fixture codebook with header pruning and no thread pool. `sxp-cli decompress
<src> <dst>` restores the original bytes through the kernel. Success replaces
the destination by renaming a temp file in the same directory. A failure leaves
an existing destination unchanged. `--help` and `-h` print usage and exit 0.
`sxp-cli benchmark <out-dir> <manifest> <gate>` writes `report.json`,
`annex.json`, `timing.log`, and one `.sxpc` sidecar per encoded input. The
output directory must already exist. Stdout is the report. The default report
is `exploratory/unreportable` with a null verdict. It does not download the
corpus and does not install a tuned model into `compress`.

There is no `--version`, `--recipe`, `--threads`, or `--force`.

Arguments are files. There is no standard-input or standard-output data path.

`python3 tools/fetch/fetch.py` reads local fixtures and the manifest. `extract` and `pin` check an archive. `transform pcm16le-to-u8` and `transform slice` write derived bytes. `select-pcm` orders clips under a budget. `overlap` looks for a shared run. `gate` prints the corpus report and exits 0 even when the gate is not ready. A typed error exits 1. The tool does not open a network connection. Python tests are host-only and are not part of `ci.sh`.

## What a bitstream contains

Whole-file raw is implemented. A file is magic `0x8D`, a flags byte (version 1,
raw mode), a minimal unsigned LEB128 length, the original bytes, and a CRC-32 of
those bytes. Output is at most N + 11 bytes. The empty file is 7 bytes.

The kernel can evaluate a polynomial formula, a trigonometric formula, and a
recurrence formula. The trigonometric evaluator uses the committed 1025-entry
sine table. It codes binary decisions with the range coder and reports the
exact byte length. It also adapts one order-0 model: 255 nodes, eight decisions
per byte, and a provisional strawman (`p = 32768`, count cap 24) that D-07 has
not frozen.

Blocked mode writes those results into a file. Baseline kinds are Raw, Direct,
Previous-byte, and Constant. Formula kinds are Polynomial, Trigonometric, and
Recurrence: the section stores the parameters, and the payload is the
range-coded residual. A Block starts with a 5-bit header, then checked zero
padding, then the payload. Format version stays 1. Kind 7 and length code 3
stay reserved. The production codebook recipe and the model constants are still
open.

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
and recurrence formulas produce the same prediction bytes on both targets, the
binary range coder round-trips fixed decision schedules on both targets,
the order-0 model reproduces the pinned RECIP and 256-symbol hashes on both
targets, and a blocked file of baseline or formula Blocks round-trips on both
targets. The encoder selects among the baselines by exact encoded size,
partitions the input, lets whole-file raw win a tie, and searches the labelled
fixture codebook the same way. `sxp-cli compress` and `sxp-cli decompress`
round-trip a file on both targets. The corpus manifest and its gate report
are committed, and the corpus gate stays not ready. `sxp-cli benchmark` writes
an exploratory comparison of the six arms. The production recipe is still absent.

The epic breakdown and the implementation-readiness report (2026-11-01, NEEDS
WORK) are on the `serexpai-bmad` branch. Reportable evaluation stays gated on
overlap K, the LJ Speech prefix check, the Debian tuning package, model
constants, the production Codebook recipe, and counsel review.
