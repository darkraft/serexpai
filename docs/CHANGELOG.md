# Changelog

All notable changes to this project are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: `0.n.m`, patch bump by default.

## [Unreleased]

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
