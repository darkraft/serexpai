# Changelog

All notable changes to this project are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: `0.n.m`, patch bump by default.

## [Unreleased]

## [0.0.9] - 2026-11-01

### Added

- Polynomial evaluator in `sxp-kernel`. Degree is `0..=15`, Q is `8..=24`, and each coefficient is in `[−2^31 + 1, 2^31 − 1]`, lowest power first. Evaluation is signed 64-bit Horner with `u = n << 5`, arithmetic shifts, and `rem_euclid` 256. Out-of-range inputs return `InvalidParameter` and are not clamped. The SHA-256 of the degree-15, Q=24 alternating prediction sequence is `spec/vectors/polynomial-degree15-q24.sha256`.
- `docs/decision_logs/`: the story 1.3 review choice to apply both patches, and this release's version and commit-message choices (2026 W44).

### Changed

- **Bitstream format unchanged.** The evaluator is not framed. Block headers, formula parameters, and the range coder are still not implemented. Format version stays 1.
- `README.md` records version 0.0.9. The kernel evaluates a polynomial. The CLI still has no commands.

## [0.0.8] - 2026-11-01

### Added

- **Bitstream format.** Whole-file raw is implemented in `sxp-kernel`. A file is magic `0x8D`, flags for version 1 and raw mode, a minimal unsigned LEB128 length below `2^35`, the original bytes, and CRC-32/ISO-HDLC of those bytes. The empty file is 7 bytes. Size is `N + 6 + leb128_len(N)`, at most `N + 11`. The block layout, formula parameters, and range coder are unchanged and still not implemented.
- Cargo workspace for `sxp-kernel`, `sxp-encoder`, `sxp-bench`, and `sxp-cli`, pinned to Rust 1.99.1 with an exact lockfile. `ci.sh` runs formatting, clippy with warnings denied, tests, and the kernel dependency check through `cargo` and `rustup`, including `aarch64-unknown-linux-gnu` under qemu-user.
- Committed whole-file raw fixtures and named reject vectors under `spec/vectors/`, recorded in a commit that contains no framing code.
- Modular residual reconstruction in `sxp-kernel`. For bytes `y` and `ŷ`, `δ = ((y − ŷ + 128) mod 256) − 128`, the coded symbol is `δ mod 256`, and `y = (ŷ + δ) mod 256`. Direct entropy coding emits `y` and does not take a residual. The SHA-256 of all 65,536 symbol bytes, `y` then `ŷ`, is `spec/vectors/residual-symbols.sha256`. Block framing and the range coder are still not implemented, so the bitstream layout is unchanged.
- `docs/decision_logs/`: the story 1.1 directory choice and this release's version and commit-message choices (2026 W44).

### Changed

- `README.md` records version 0.0.8. The kernel round-trips whole-file raw and reconstructs a byte from any prediction. The CLI still has no commands.

## [0.0.7] - 2026-11-01

### Added

- Implementation-readiness assessment for SerExpAI v1: **NEEDS WORK**. All 15 functional requirements have a story, and there is no critical epic-structure violation. Untagged kernel work, whole-file raw, and a private benchmark can proceed. The report lives under `_bmad-output/` and is preserved on the `serexpai-bmad` record branch, not on `main`.
- `docs/decision_logs/`: the readiness document-set choices, owner decisions D-01, D-02, D-03, D-05 and D-06, the gates left open, the choice to keep Story 1.13 as one story, and this release's version and commit-message choices (2026 W44).

### Changed

- **Bitstream format (design only, no code yet).** D-01, D-02 and D-03 are approved. Evaluator ranges stay; stored trigonometric coefficient width is at most 29 and recurrence width is at most 25. A Block is a 5-bit header, kind-specific fields, then checked zero padding to the next byte. The final partial span uses the smallest nominal length in {256, 1024, 2048} that covers the bytes left. Nothing is implemented, so there is no compatibility break.
- D-05 and D-06 are approved. Growth path is at least 1% of a class's total encoded size and a majority of its evaluation inputs; a one-class win is negative and opens only the deferred PCM experiments. A budget miss shrinks the corpus first without changing frozen evaluation inputs, then lowers the Codebook cap.
- Left open: overlap K, the LJ Speech prefix check, the Debian tuning package, model constants (D-07), counsel (D-08), and the production Codebook recipe (D-09).
- `README.md` records version 0.0.7. The readiness check is no longer the next step.

## [0.0.6] - 2026-11-01

### Added

- Epic and story breakdown for SerExpAI v1. Epic 1 measures whether formulas beat baselines (stories 1.1–1.13). Epic 2 publishes a bitstream someone else can implement (stories 2.1–2.4). The breakdown includes a proposed acceptance-criteria revision; decisions D-01–D-09 stay open. It lives under `_bmad-output/` and is preserved on the `serexpai-bmad` record branch, not on `main`.
- Workflow decision-log skill at `.cursor/skills/workflow-decision-log/`, so later decision entries are written by the copy committed with the repository.
- `docs/decision_logs/`: epic inputs, the `sxp-` crate-prefix confirmation, the Gain definition, the story breakdown, the acceptance-criteria revision, and this release's version and commit-message choices (2026 W44).

### Changed

- Architecture spine aligned with the PRD and the spec. The crate naming row is `sxp-<layer>`. Gain for a real input class is any reduction versus the baselines-only arm. The best whole-input baseline stays in the report as a consistency check and cannot decide Gain.
- `README.md` records version 0.0.6 and that the epic breakdown exists. The next step is an implementation-readiness check. The bitstream format is unchanged.

## [0.0.5] - 2026-10-31

### Changed

- The product name is SerExpAI. MBSC (Multi-Basis Series Compression System) is the idea that product represents. The repository stays `serexpai`.
- The planned Cargo crates are `sxp-kernel`, `sxp-encoder`, `sxp-bench` and `sxp-cli`.
- `README.md` names the product, the idea and those crates. The spec, glossary, PRD, architecture spine and bitstream spec seed on the `serexpai-bmad` record branch say the same. The bitstream format is unchanged.

## [0.0.4] - 2026-10-31

### Added

- Spec kernel for MBSC v1 (`SPEC.md`, glossary, acceptance criteria, corpora companion) distilled from the PRD, architecture spine and product brief. It lives under `_bmad-output/` and is preserved on the `serexpai-bmad` record branch, not on `main`.
- Research on the six open questions, and a claim chart of Microsoft US 11,234,023 B2 prepared to brief counsel. The pre-update PRD (`prd.before.md`) is kept beside the PRD on the record branch only.
- `docs/decision_logs/`: spec source and gap-handling choices, the open-question walkthrough, the coder and checksum decisions, the PRD reconciliation choices, and this release's version and commit-message choices (2026 W44).
- `README.md`: what MBSC is, the planned commands, and where the design documents live. It states that v1 uses a range coder and that there is no code yet.

### Changed

- **Bitstream format (design only, no code yet).** Format v1 entropy-codes with an LZMA-style binary range coder, not rANS. rANS is an informative appendix and would need a patent review plus a format version bump to adopt. Every file ends with a CRC-32 of the decoded bytes, so the expansion bound is N + 11 (the empty file is 7 bytes). The probability model holds a 16-bit probability with a symmetric update; the earlier 12-bit rule stalled and taxed perfect predictions.
- Architecture spine and bitstream spec seed revised to match: range coder, CRC, corpora, clean-room decoder check, and a legal review of the adaptive probability update before the first public release.
- PRD reconciled with the spec and re-finalized: range coder, CRC, chosen corpora, licence, SM-1 go threshold (a positive result below it is *marginal*), and eight open questions, none blocking the next phase.

### Security

- v1 avoids the decoder structure added to win allowance of US 11,234,023 B2 (no deferred state update, no gated symbol generation, no flush-or-keep field, no coder state across Blocks). This is a design safeguard for counsel, not a legal opinion.

## [0.0.3] - 2026-10-31

### Added

- Architecture spine for MBSC v1 (final): Normative Kernel + Encoder Shell, enforced by a four-crate Cargo workspace, with 15 invariants (AD-1 to AD-15) covering the Evaluator, probability model, Block layout, partition, Rate selector, framing, safety, determinism, benchmark integrity and versioning. It lives under `_bmad-output/` and is preserved on the `serexpai-bmad` record branch, not on `main`.
- Bitstream and Normative Evaluator spec seed (draft values tagged PROPOSED or MEASURE), an interactive HTML walkthrough of the architecture, four reviewer reports and the architecture memlog, in the same location.
- `docs/decision_logs/`: architecture working-mode, paradigm, language, rANS ownership, probability model, stream layout, partition, family-definition, Evaluator, numeric-bound and structure decisions, plus this release's version and commit-message choices (2026 W44).

### Changed

- Bitstream format (design only, no code yet): the probability model is an adaptive order-0 bit tree with a fixed rANS total of 4096, reset at every Block, with one rANS stream per Block. A variable-total count model was rejected because rANS cannot invert a changing total. The format is not frozen; the spec seed marks which constants are still to be measured.

## [0.0.2] - 2026-10-30

### Added

- PRD for MBSC v1 (final), with addendum, review and reconciliation records and decision memlog. v1 is a rate-measurement experiment CLI with 15 FRs and 5 success metrics. SM-1 compares full MBSC against a Baselines-only arm and the best whole-input baseline, so any gain is attributable to formulas. The PRD lives under `_bmad-output/` and is preserved on the `serexpai-bmad` record branch, not on `main`.
- `docs/decision_logs/`: PRD working-mode, input, predictor-family, input-class and CLI-behaviour decisions, plus this release's version and commit-message choices (2026 W44).

### Changed

- None. No code or bitstream format exists yet.

## [0.0.1] - 2026-10-30

### Added

- `docs/mbsc_architecture_review.md`: architecture review of the MBSC lossless compressor design (predictor-plus-residual coding with context-adaptive rANS). It is the specification input for the project.
- Product brief for MBSC, with addendum and decision memlog: v1 is a rate-measurement experiment delivered as a CLI. Success means a lossless round-trip plus a total-size gain over the baselines on at least two real input classes.
- `docs/decision_logs/`: weekly workflow decision log (2026 W44).
- `docs/CHANGELOG.md`.
- `.gitignore`: keeps BMad tooling and planning paths (`_bmad-output/`, `_bmad/`, `.agents/`), secrets and benchmark corpora off the code branch.

### Changed

- None. This is the first release; there is no code or bitstream format yet.
