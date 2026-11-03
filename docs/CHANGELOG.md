# Changelog

All notable changes to this project are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: `0.n.m`, patch bump by default.

## [Unreleased]

## [0.0.18] - 2026-11-03

### Added

- `tools/fetch`, a Python standard-library tool. `extract` and `pin` check a local archive before reading a member. `transform pcm16le-to-u8` and `transform slice` write derived bytes. `select-pcm` orders clips by `SHA-256(seed || id)` and stops at the budget. `overlap` reports a shared run of at least K. `gate` prints the corpus report. A typed error exits 1 with the code as the first stderr token. The tool does not download anything. `ci.sh` does not call it.
- `corpora/manifest.toml` and `corpora/gate-report.json`. The roster is the Canterbury, Calgary, Silesia, LJ Speech, Debian, enwik8, FLAC, and synthetic rows. `corpus_gate_ready` is false. K, the LJ seed, the LJ metadata check, the LJ prefixes, both PCM budgets, the Debian tuning package, and the PCM and executable replacement rules stay unresolved.
- Three 4096-byte synthetic signals in `sxp-bench`. Each one calls a kernel evaluator twice over `n` in `0..2048`. The Rust test checks the parameter tuples and the SHA-256 literals against the manifest. `counts_toward_sm1` is false for all three.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- **Bitstream format unchanged.** Format version stays 1. Kind 7 and length code 3 stay reserved. The production codebook recipe (D-09) and the model constants (D-07) stay open. The benchmark command is still absent.
- `README.md` records version 0.0.18, the fetch tool, and the unready corpus gate. The workspace manifest is 0.0.18. The binary has no `--version` flag.
- A zip member of any mode is accepted, including mode 0. A symlink is `UnsafeLink`. A directory is `NotRegular`.

## [0.0.17] - 2026-11-03

### Added

- `sxp-cli compress <src> <dst>` and `sxp-cli decompress <src> <dst>`. Compress writes `encode_codebook` for the labelled `fixture-v1` codebook, all baselines, header pruning, and no thread pool. Decompress calls `sxp_kernel::read_file` and does not load a codebook. Success exits 0 with empty stdout and stderr. `--help` and `-h` exit 0. Every other failure exits 1 with one stderr line. There is no `--version`, `--recipe`, `--threads`, or `--force`, and no stdin or stdout data path.
- Atomic install. The command writes a temp file in the destination parent, validates it, flushes, syncs, and renames it onto the destination. A failure removes only that temp file. An existing destination stays byte-identical. The same directory entry, a directory source, a missing parent, and a destination with no file name are refused before the temp file exists.
- `sxp_kernel::read_file`. It reads the shared 7-byte header and dispatches to the existing whole-file or blocked reader. A declared length at or above `2^35` is `Leb128Overflow`. Compress refuses a source whose followed metadata length is at or above that limit and does not read it.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- **Bitstream format unchanged.** Format version stays 1. The CLI emits streams the 0.0.16 decoder already accepts. Kind 7 and length code 3 stay reserved. The production codebook recipe (D-09) and the model constants (D-07) stay open. The benchmark command is still absent.
- `README.md` records version 0.0.17 and the two file commands. The workspace manifest is 0.0.17. The binary has no `--version` flag.

## [0.0.16] - 2026-11-03

### Added

- Formula Blocks in `sxp-kernel`. Kinds 4, 5, and 6 store a polynomial, trigonometric, or recurrence section and a self-contained residual. The decoder rebuilds the formula from those bytes and does not read a codebook. A recurrence block is evaluated in one forward pass. Committed section bytes are `spec/vectors/formula-blocks.hex`.
- Fixture codebook search in `sxp-encoder`. `generate_codebook("fixture-v1")` rebuilds three labelled formulas. Their canonical SHA-256 is `spec/vectors/codebook-fixture-v1.sha256`. Search appends them after the baselines, keeps the same partition key, and lets the smaller candidate id win a tie, so a baseline beats a formula of equal size. Header pruning skips a section that cannot beat the incumbent. One or two threads write the same bytes. `generate_codebook("production")` and `generate_codebook("benchmark")` stay unresolved.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- **Bitstream.** Kinds 4, 5, and 6 are decoded. They were rejected as unsupported in 0.0.14. Format version stays 1. Kind 7 and length code 3 stay reserved. The largest formula section raises the kernel's encoded-block cap to 32898 bytes. The production codebook recipe (D-09) and the model constants (D-07) stay open. The CLI still has no commands.
- `README.md` records version 0.0.16. The kernel frames a formula Block. The encoder searches the fixture codebook.

### Fixed

- A formula section with an illegal coefficient and a non-zero padding bit returns `InvalidParameter`. The domain check runs before the padding check.

## [0.0.15] - 2026-11-03

### Added

- Baseline selection in `sxp-encoder`. For each allowed span the encoder asks `sxp-kernel` for the exact encoded size of Raw, Direct, Previous-byte, and all 256 Constant blocks. The partition minimises that size, then the block count, then the length code and candidate id. Whole-file raw wins when the blocks are not strictly smaller than the input, including the empty 7-byte file. A fixed-length arm still ends on the canonical nominal, so the existing decoder accepts it. Header pruning and one or two threads write the same bytes.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- **Bitstream format unchanged.** The encoder does not pack a field. It calls the kernel's block and whole-file writers. Format version stays 1. Formula search, the benchmark, and the CLI commands are still absent.
- `README.md` records version 0.0.15. The encoder chooses baseline blocks. The CLI still has no commands.

## [0.0.14] - 2026-11-03

### Added

- **Bitstream.** Blocked mode is implemented in `sxp-kernel` for the four baseline kinds. A Block is a 5-bit header (2-bit length code, then 3-bit kind, least significant bit first), kind-specific fields, checked zero padding to the next byte, then a byte-aligned payload. Length codes are 256, 1024, and 2048. Kinds are Raw, Direct, Previous-byte, and Constant. The Constant byte follows the header immediately and straddles the two section bytes. Actual length is `min(nominal, remaining)`. A nominal length is canonical when it is at most the bytes left, or, when at most 2048 bytes are left, when it is the smallest nominal that covers them. Raw copies bytes. Direct, Previous-byte, and Constant each start a new strawman order-0 model and a new range coder. Previous-byte predicts 0, then the previous byte in the Block. Constant predicts its signalled byte. `write_blocked_file` and `read_blocked_file` round-trip a mode-0 file. Mode 0 with `N = 0` stays rejected. Header, padding, and two raw-file fixtures are `spec/vectors/block-baseline-headers.hex`, `spec/vectors/block-raw-n1.bin`, and `spec/vectors/block-raw-n2.bin`.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- Format version stays 1. Whole-file raw is unchanged, and `read_whole_file_raw` still rejects mode 0. Wire kinds 4, 5, and 6 are rejected as unsupported. Kind 7 and length code 3 are reserved. Formula parameters are still not written. The kind numbers remain the spec seed's proposed table.
- `README.md` records version 0.0.14. The kernel frames a baseline Block. The CLI still has no commands.

## [0.0.13] - 2026-11-02

### Added

- Order-0 probability model in `sxp-kernel`. A symbol is eight binary decisions, most significant bit first, through 255 nodes. Each node holds a 16-bit `p` and a saturating count. Bit 0 updates `p` by `(((65536 − p) · R) >> 16)` and bit 1 by `−((p · R) >> 16)`, in `u32`, with no clamp. `freq0 = max(p >> 4, 1)`. The strawman is provisional until D-07: scalar `p = 32768`, count cap 24, and `RECIP[n] = floor(65536 / (n + 2))`. The RECIP hash is `spec/vectors/model-recip-strawman.sha256`. The SHA-256 of symbols `0..=255` coded with that strawman is `spec/vectors/model-symbols-256.sha256`.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- **Bitstream format unchanged.** The model is reconstructed from decisions already taken. No probability table is written into a file. Block framing is still not implemented. Format version stays 1.
- `README.md` records version 0.0.13. The kernel adapts an order-0 model and still does not frame it. The CLI still has no commands.

## [0.0.12] - 2026-11-02

### Added

- Binary range coder in `sxp-kernel`. Encoder state starts at `low = 0`, `range = 0xFFFFFFFF`, `cache = 0`, and `cache_size = 1`. Each decision takes a bit and `freq0` in `1..=4095`, with total 4096. Flush calls `shift_low` five times and omits the leading `0x00`. `exact_size` is the emitted length. An invalid frequency or bit returns `InvalidParameter`. A short buffer is `TruncatedCoder`, an initial code of `0xFFFFFFFF` is `SaturatedRangeCode`, and a final code other than 0 is `NonzeroRangeCode`. A truncated refill leaves `range` below `2^24`, so a later decode and finish stay truncated. The SHA-256 of the 2048-symbol, 16384-decision schedule is `spec/vectors/range-coder-2048.sha256`.
- `docs/decision_logs/`: this release's version and commit-message choices (2026 W45).

### Changed

- **Bitstream format unchanged.** The range coder is a kernel primitive and is not framed in a file. Block headers, formula parameters, and the probability model are still not implemented. Format version stays 1.
- `README.md` records version 0.0.12. The kernel codes binary decisions and still does not write them into a bitstream. The CLI still has no commands.

## [0.0.11] - 2026-11-01

### Added

- Recurrence evaluator in `sxp-kernel`. Order is 1 or 2, `m_mod` is `2..=2^24`, each coefficient satisfies `|a|, |b|, |c| < 2^24`, and each initial state is strictly below `m_mod`. Order 1 is `x[n+1] = rem_euclid(a·x[n] + b, m_mod)`. Order 2 is `x[n+2] = rem_euclid(a·x[n+1] + b·x[n] + c, m_mod)`. `value` is `x[n]`; the prediction byte is `rem_euclid(value, 256)`. Out-of-range inputs return `InvalidParameter` and are not clamped. An order-1 formula rejects a third coefficient and a second state. The SHA-256 of the order-2, `m_mod = 2^24` prediction sequence is `spec/vectors/recurrence-order2-m24.sha256`.
- `docs/decision_logs/`: the story 1.5 review choice to apply the six patches, and this release's version and commit-message choices (2026 W44).

### Changed

- **Bitstream format unchanged.** The recurrence evaluator is not framed. Block headers, formula parameters, and the range coder are still not implemented. Format version stays 1.
- `README.md` records version 0.0.11. The kernel evaluates a polynomial, a trigonometric formula, and a recurrence formula. The CLI still has no commands.

## [0.0.10] - 2026-11-01

### Added

- Trigonometric evaluator in `sxp-kernel`. Order is `1..=16`, Q is `8..=24`, `omega` is any unsigned 32-bit phase increment, and each coefficient is in `[−(2^28 − 1), 2^28 − 1]`. Layout is `c0`, then `a_k`, `b_k`. Evaluation is signed 64-bit: `theta = (n · omega) mod 2^32`, interpolation of the committed 1025-entry sine table, arithmetic shifts, and `rem_euclid` 256. Out-of-range inputs return `InvalidParameter` and are not clamped. The table is `spec/sine_lut.bin` (SHA-256 `6267f51aebd57812c68a389bb4de8f2c85da816db4338a01b21e7455ec7ca9df`). The SHA-256 of the order-16, Q=8 alternating prediction sequence is `spec/vectors/trigonometric-order16-q8.sha256`.
- `docs/decision_logs/`: the story 1.4 review choice to apply both patches and to end that workflow, and this release's version and commit-message choices (2026 W44).

### Changed

- **Bitstream format unchanged.** The trigonometric evaluator is not framed. Block headers, formula parameters, and the range coder are still not implemented. Format version stays 1. The sine table size remains marked proposed in the spec seed; this release commits the 1025-entry Q15 table the evaluator uses.
- `README.md` records version 0.0.10. The kernel evaluates a polynomial and a trigonometric formula. The CLI still has no commands.

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
