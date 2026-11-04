---
baseline_commit: 9ed20f4a11e05670fe03ddd28ebdb65ce297d4ae
---

# Story 1.13: Report whether formulas beat baselines

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want an auditable comparison and either a correctly classified research verdict or explicit unreportable status,
so that a missing corpus gate cannot be published as a scientific result.

## Acceptance Criteria

1. **S1.13-AC01.** `sxp_bench` runs six arms on each supplied input: Whole-file raw, forced Constant, forced Previous-byte, forced Direct, Baselines-only, and full MBSC. Candidate sets are the AD-14 table below. Each arm's bytes are a real bitstream. `sxp_kernel::read_file` decodes them and the output equals the source. A mismatch returns `RoundTripMismatch`, installs no `report.json`, and does not publish a verdict.

2. **S1.13-AC02.** For every input the file lengths satisfy `S_full ≤ S_baselines_only ≤ min(S_forced_constant, S_forced_previous, S_forced_direct) ≤ S_whole_file_raw`. The minimum of the three forced lengths is reported as `best_forced`. It is not the Gain comparator. A break returns `SelectorDefect`, installs no `report.json`, and is not a scientific verdict.

3. **S1.13-AC03.** `report.json` and stdout are the same canonical JSON bytes. The document has per-input arm sizes, class sums, Gain bytes, per-input win and margin, and separate `synthetic`, `compressed_control`, and `non_counted_pcm` arrays. Gain bytes are `S_baselines_only − S_full` on `counts_toward_sm1` evaluation inputs only.

4. **S1.13-AC04.** Every arm has a byte account whose eight components sum to that arm's `file_len`. Components are `file_header`, `block_headers`, `parameters`, `alignment_padding`, `raw_payload`, `coder_payload`, `coder_flush`, and `crc`. Packed-field attribution is the rule in Dev Notes. Whole-file raw uses the same eight fields.

5. **S1.13-AC05.** Adaptation overhead is `annex.json`, not a field inside `report.json`. Ideal length is the decision sum of `−log2(freq/4096)` in microbits. Overhead is that sum minus the selected block's zero-order symbol entropy, also in microbits. Flush bytes stay in the byte account. Inclusion rules and share denominators are the Dev Notes rules. `report.json` contains no floating-point numbers.

6. **S1.13-AC06.** The report records tool name and `CARGO_PKG_VERSION`, format version 1, layout `d02-lsb-first`, spec-seed SHA-256, fixture codebook name/version/label/hash, manifest SHA-256, gate document, overlap K and result, the decision map, the growth thresholds, experiment id `exploratory-fixture-v1`, and block lengths 256, 1024, 2048. Work counts are integers: selected model decisions, selected coder steps, search encode calls, and skipped cells. Wall time, host, and timestamps appear only in `timing.log`.

7. **S1.13-AC07.** A sidecar is written for each encoded input with `N ≤ 4194304` unless the request sets sidecar off. Above that size the sidecar is off unless the request sets `on` or `dense`. The file is little-endian `SXPC` version 1, indexed by offset, length code, and candidate id. `0xFFFF` means header-pruned. An evaluated cost above 65534 is `CostOverflow`. `dense` forces `SearchMode::Reference` and zero skipped cells. The report names each sidecar and its coverage mode.

8. **S1.13-AC08.** Fixture reports reconcile every account total and every sidecar cell with `encode_block`. Two runs with the same inputs, threads 0 and 2, and the same search mode produce identical `report.json` and identical sidecar bytes. `timing.log` may differ. The annex is outside that byte-identity check. `./ci.sh` passes on the host. If the aarch64 qemu prefix is present, `./ci.sh aarch64-unknown-linux-gnu` passes too.

9. **S1.13-AC09.** `reportable` is false while `corpora/gate-report.json` has `corpus_gate_ready: false`, while D-07 or D-09 is open, while the recipe is `fixture-v1`, or while overlap result is not `clear`. The status string is then `exploratory/unreportable`, `verdict` is null, and `missing_gates` lists the open items. That status is not the scientific verdict `inconclusive`. Do not add `g2_ready`. Do not set `corpus_gate_ready` to true.

10. **S1.13-AC10.** `classify` is a pure table. Eligible classes are `text`, `pcm`, and `executable` with at least one measured `counts_toward_sm1` evaluation input. Fewer than two eligible classes is `inconclusive`. Zero winning classes is `negative`. One winning class is `negative` and sets `opens_pcm_experiments`. Two or more winning classes with fewer than two clearing the growth bar is `marginal`. Two or more clearing the bar is `positive`. A class wins when its Gain bytes are positive. The bar is `gain_bytes * 100 >= baselines_only_bytes` and `wins * 2 > evaluation_inputs`. The denominator is the Baselines-only class sum. The function persists the counts it used. The exploratory file report does not call this result a published verdict.

11. **S1.13-AC11.** `tune` scores model constants on Baselines-only only, and only on rows whose role is `tuning`. The grid is CAP `{24, 32, 60, 128, 255}` × DELTA `{1, 0.5}` × scalar `p = 32768`, then one all-32768 per-depth table. DELTA 1 is `floor(65536 / (n + 2))`. DELTA 0.5 is `floor(131072 / (2n + 3))`. Lower summed file length wins. Ties go to smaller CAP, then DELTA 1, then scalar. An evaluation row returns `EvaluationLeak` and writes nothing. The selected config is stored in `tuning.json` and is not installed into `encode_block`, `compress`, or the six arms. Width search stays `not-run` with reason `d09-open`.

12. **S1.13-AC12.** `timing.log` names the host before any arm runs, then records `scope`, `start`, `end`, `completed`, and whether elapsed time is at most 86400 seconds. `reduce_budget` is pure: on excess, drop tuning rows first, keep every class's evaluation rows, and only then lower the codebook cap. Returning a plan that changes an evaluation input is `NewExperiment`. No test runs for 24 hours. No heuristic may replace exact-size selection.

## Tasks / Subtasks

- [x] Accounted arms (AC: 1, 2, 4, 6)
  - [x] Kernel byte account and `encode_block_model` / `decode_block_model`. `encode_block` bytes stay unchanged
  - [x] Encoder `search_arm` records cells, work counts, and the account. It calls the existing DP
  - [x] Six candidate sets. Forced Constant is Raw plus all 256 constants
  - [x] Inequality and round-trip checks fail closed
- [x] Report, annex, sidecar, verdict, tuning, budget (AC: 3, 5, 6, 7, 9, 10, 11, 12)
  - [x] Canonical `report.json`, `annex.json`, `SXPC` sidecars, `classify`, `tune`, `reduce_budget`
  - [x] Default run generates the three synthetic signals and does not fetch
  - [x] Exploratory status while the corpus gate, D-07, D-09, or overlap is open
- [x] CLI (AC: 3, 8, 12)
  - [x] `sxp-cli benchmark OUT_DIR MANIFEST GATE` writes the four artifacts through `Fs`
  - [x] Stdout equals `report.json`. Failures leave a previous report in place
  - [x] `compress` and `decompress` stay byte-identical
- [x] Tests (AC: 1–12)
  - [x] Library tests for the verdict table, accounts, sidecars, tuning ties, and budget reducer
  - [x] CLI test on `MemFs` for success, `SelectorDefect` left uninstalled, and a missing directory
  - [x] `./ci.sh` exits 0. Python stays out of `ci.sh`

### Review Findings

- [x] [Review][Patch] Reject duplicate ids, control characters, a mislabeled synthetic id, and a tuning row before search [crates/sxp-bench/src/run.rs]
- [x] [Review][Patch] Reject a sidecar length code above 2 [crates/sxp-bench/src/sidecar.rs]
- [x] [Review][Patch] Reject a control character in the host name, a backward clock, and a year outside 0000–9999; retry an interrupted read [crates/sxp-cli/src/benchmark.rs]
- [x] [Review][Patch] Account parts cover the empty file, a one-byte file, and a two-block file [crates/sxp-kernel/tests/account.rs]

## Dev Notes

### What this story builds

An exploratory benchmark report on top of the existing selector. The scientific verdict function exists and is tested on tables. The file written from the committed manifest is `exploratory/unreportable`, because the corpus is not fetched and D-07 and D-09 are open.

Out of scope: downloading corpora, filling hashes, confirming K, choosing a production codebook, freezing model constants into the wire format, editing `bitstream-spec-seed.md`, `README.md`, `CHANGELOG.md`, or the workspace version, `tools/refdec`, a Python decoder, clean-room or counsel gates, rANS, an entropy screen, a new crate, a new dependency, and any change to `compress` output. Do not put Python in `ci.sh`. Do not set `corpus_gate_ready` or add `g2_ready`.

### Pinned readings

These are the story's readings of sentences the sources left numeric-ambiguous. Do not substitute another reading.

- Forced Constant is `BlockKind::Raw` plus `BlockKind::Constant(0..=255)`. Forced Previous-byte is Raw plus Previous-byte. Forced Direct is Raw plus Direct. Baselines-only is `all_baselines()`. Full MBSC is `all_baselines()` plus `generate_codebook("fixture-v1")`. Whole-file raw calls `write_whole_file_raw` and does not search.
- Every blocked arm uses `LengthPolicy::All`, the existing DP, and the existing whole-file raw tie. Default search mode is `HeaderPrune`. `dense` uses `Reference`.
- Gain denominator for the 1% bar is the class sum of Baselines-only file lengths. Majority means `wins * 2 > evaluation_inputs`. A tie (`S_full == S_baselines_only`) is not a win. Margin bytes may be negative.
- `best_forced` ties break by the name order `forced_constant`, `forced_direct`, `forced_previous`.
- Eligible classes are only `text`, `pcm`, and `executable`.
- `fixture-v1` never makes a run reportable.

### Reuse

| Need | Use |
| --- | --- |
| Baselines-only and full search | `select_baselines`, `select_codebook`, `encode_baselines`, `encode_codebook` in `crates/sxp-encoder/src/select.rs` |
| Per-candidate cost | `span_cells` and `CostCell`. Do not write a second DP |
| Candidate ids | `candidate_id`. Raw 0, Direct 1, Previous-byte 2, Constant `3 + c`, formulas `259 + index` |
| Decode check | `sxp_kernel::read_file` |
| One block | `encode_block`. Its bytes stay the strawman encoding |
| Codebook | `generate_codebook("fixture-v1")` only. Hash `a8f53ca8a7799035822e1fe6bf27962136b6af9ccf9e75b8785562d638cfef9b` |
| Synthetics | `polynomial_signal`, `trigonometric_signal`, `recurrence_signal`, `sha256_hex` |
| Gate | Read `corpora/gate-report.json`. Do not reimplement `tools/fetch` |
| CLI errors | `CliError`, `line`, `run_with`, `MemFs` |
| Spec seed | The hash already written in `crates/sxp-kernel/src/lib.rs` |

`compress` keeps `fixture-v1`, `all_baselines()`, `LengthPolicy::All`, `SearchMode::HeaderPrune`, and `threads = 0`. Decompress still calls only the kernel.

### Arms and failure

Build each blocked arm through one new encoder function, `search_arm`, that uses the private `fill_table` / `reduce` path and also keeps every `span_cells` result. Do not copy the transition rules into `sxp-bench`.

`search_arm` returns the `Selection`, the file bytes, the byte account, the span cells, and the work counters. `file_len` equals `file.len()`.

After all six arms, decode each file with `read_file`. Then check the inequality on `file_len`. Either failure returns before any artifact is renamed into place.

Empty input: every arm is the 7-byte whole-file raw stream. The inequality holds.

### Byte account

Add `ByteAccount` in the kernel. A new accounted writer shares `push_section`, `formula_wire::append_section`, and the range coder. `encode_block` still returns the same `Vec<u8>` as before.

Attribution, with no byte in two components:

- File magic, version/mode, and LEB128 are `file_header`. The last four bytes are `crc`.
- The first byte of each block section is `block_headers`. Padding bits in that byte stay there.
- Later section bytes that hold a parameter bit are `parameters`. Padding bits that share a byte with a parameter stay in `parameters`.
- A section byte that contains only alignment padding is `alignment_padding`.
- Raw block bytes after the section, and the whole-file raw payload, are `raw_payload`.
- Coder bytes emitted before `RangeEncoder::finish` are `coder_payload`. Bytes appended by `finish` are `coder_flush`. Do not assume flush is four bytes when a carry emits more. The no-carry fixture is four bytes.
- Whole-file raw leaves `block_headers`, `parameters`, `alignment_padding`, `coder_payload`, and `coder_flush` at 0.

The eight fields sum to the file length. A test on empty, 1-byte, and one multi-block input checks that sum, and checks that concatenating the parts the writer already produced equals the file.

### Model side path

`decode_block` and `read_file` stay on `ModelConfig::strawman()`. Add `encode_block_model` and `decode_block_model`. For the strawman config, `encode_block_model` equals `encode_block` byte for byte. The tuner uses the model pair to score a config. The six benchmark arms do not. A non-strawman stream is not written to the report and is not a `compress` output.

RECIP length is `cap + 1`. Entry `n` uses the DELTA formula above. A zero rate is `InvalidParameter`.

### Work counts

Integers only, per arm, per input:

- `input_bytes` is the source length.
- `selected_model_decisions` = 8 × input bytes that sit in a selected non-raw block. Raw blocks add 0. Per input byte is the rational `(selected_model_decisions, input_bytes)`.
- `selected_coder_steps` = `selected_model_decisions` + 5 × selected coded blocks. The 5 is the five `shift_low` calls in `finish`. Per input byte is the rational `(selected_coder_steps, input_bytes)`.
- `search_encode_block_calls` = the number of `CostCell::Evaluated` cells.
- `search_skipped` = the number of `CostCell::Skipped` cells.
- Whole-file raw: all four counters are 0.

### Sidecar

Little-endian, tight packing:

```text
magic          b"SXPC"
version        u8 = 1
coverage       u8 = 1 default, 2 opt-in, 3 dense
mode           u8 = 0 header-prune, 1 reference
reserved       u8 = 0
candidate_count u16
candidate_ids  u16 × count, ascending
span_count     u32
span           offset u64, length_code u8, then u16 × count
```

Cell value is the `encode_block` length. `0xFFFF` is `Skipped` only. A cost above 65534 is `CostOverflow`, never saturated and never stored as `0xFFFF`. `Reference` writes no `0xFFFF`. Header prune may. `N == 0` writes no spans and `span_count = 0`.

Coverage: `default` when `N ≤ 4194304` and the request is auto. `off` writes no file and the report says `present: false`. `opt-in` is the request `on` when `N > 4194304`. `dense` is the request `dense` at any size.

File name is `{id}.sxpc` in `OUT_DIR`. Ids are the manifest ids. Reject an id that is empty, contains `/`, or contains `..`.

### Report JSON

One canonical document, UTF-8, keys in the struct field order below, no whitespace other than the single trailing newline. Integers have no leading zeros. Strings use `serde_json` escaping. No floats, no NaN, no maps whose key order depends on a hash table. `serde_json` on a struct is enough if every map is a struct or a `BTreeMap` serialized through a custom writer that sorts keys. Prefer one struct and `serde_json::to_vec`.

Top-level fields, in order: `report_version` (1), `status`, `verdict`, `opens_pcm_experiments`, `experiment_id`, `tool`, `tool_version`, `format_version`, `layout`, `spec_seed_sha256`, `model_constants`, `decisions`, `thresholds`, `block_lengths`, `codebook`, `manifest_sha256`, `gate`, `overlap`, `missing_gates`, `search_mode`, `threads`, `tuning`, `missing_inputs`, `inputs`, `classes`, `synthetic`, `compressed_control`, `non_counted_pcm`.

`status` is `exploratory/unreportable` or `reportable`. `verdict` is null unless `status` is `reportable`, in which case it is one of `positive`, `marginal`, `negative`, `inconclusive`. `opens_pcm_experiments` is false unless the published verdict is the one-class negative.

`decisions` is the object `d01`…`d09` with values `approved` or `open`. Approved: d01, d02, d03, d05, d06. Open: d04, d07, d08, d09.

`thresholds` records `growth_percent` 1, `majority` the string `wins * 2 > evaluation_inputs`, and `gain_denominator` the string `baselines-only class sum`.

`codebook` is name `fixture-v1`, version 1, label `fixture`, and the hash above.

`overlap` is `k` 1024, `k_status` `proposed`, `result` `not-checked`.

`missing_gates` includes every unresolved gate string plus `d07`, `d09`, and `overlap` while those stay open. Copy `unresolved` from the gate file. Do not drop entries.

`tuning` is `{ "ran": false, "reason": "no-tuning-bytes", "width_grid": "not-run", "width_reason": "d09-open" }` on the default command. When `tune` ran, `ran` is true and `reason` is null. The selected model still is not the arm model. `model_constants` stays `provisional-strawman`.

`missing_inputs` is the sorted list of manifest ids whose bytes were not encoded. Real rows with an empty derived hash are missing. Do not open the network.

Each input object: `id`, `class`, `role`, `counts_toward_sm1`, `input_bytes`, `input_sha256`, `arms`, `best_forced`, `gain_bytes`, `win`, `margin_bytes`, `sidecar`. Arm names, in order: `whole_file_raw`, `forced_constant`, `forced_previous`, `forced_direct`, `baselines_only`, `full`. Each arm holds `file_len`, `account`, and `work`. `best_forced` holds `arm` and `file_len`.

Grouping of measured rows:

- `class == "synthetic"` → `synthetic`
- `class == "compressed"` → `compressed_control`
- transform `pcm16le-raw` → `non_counted_pcm`
- `counts_toward_sm1` and role `evaluation` and class `text`, `pcm`, or `executable` → `classes`
- any other measured row → do not invent a class; put it in `missing_inputs` only if it was not encoded. A measured tuning row is a caller error (`EvaluationLeak` is for `tune` only). The benchmark encodes synthetics plus rows the caller supplied. The default caller supplies no real bytes.

`classes` entries exist only for real classes that have at least one measured SM-1 input. Each entry: `class`, `eligible`, `evaluation_inputs`, `wins`, `full_bytes`, `baselines_only_bytes`, `gain_bytes`, `clears_growth_bar`, `shares`.

Shares use the full-MBSC selected blocks of those inputs. `blocks` and `bytes` are rationals `{num, den}` for `raw`, `direct`, `previous_byte`, `constant`, `polynomial`, `trigonometric`, `recurrence`. Block denominator is the selected block count. Byte denominator is the sum of those inputs' full-arm `file_len`. `zero_residual` counts input bytes whose residual symbol is 0, over input bytes of selected previous-byte, constant, and formula blocks. Direct and raw are outside that numerator and denominator. Denominator 0 is `{num: 0, den: 0}`.

The three synthetic ids are `synthetic-polynomial`, `synthetic-trigonometric`, and `synthetic-recurrence`. Generate them. If the manifest has a non-empty `derived_sha256` for that id, it must equal `sha256_hex` of the generated bytes or the run returns `HashMismatch`.

### Annex

`annex.json` is a JSON array, one object per measured input, in the same input order. Fields: `id`, `ideal_microbits`, `entropy_microbits`, `overhead_microbits`, `flush_bytes`. Microbits are integers. Compute `−log2(freq/4096)` in `f64`, sum over decisions, multiply by `1_000_000`, and round half away from zero. Entropy uses the same rounding on the selected symbol histogram: residuals for previous-byte, constant, and formulas; the original bytes for direct; raw blocks contribute nothing. `freq` is `freq0` for bit 0 and `freq1` for bit 1. Decision order is the model's encode order. Two host runs should match. Cross-platform identity is not required. Do not put these numbers in `report.json`.

### Verdict table

`classify` sees only class rows and a `reportable` flag. When `reportable` is false it returns `Published::Exploratory` and does not return a scientific label. When `reportable` is true:

| Eligible | Winning | Clearing the bar | Label | `opens_pcm_experiments` |
| --- | --- | --- | --- | --- |
| < 2 | any | any | inconclusive | false |
| ≥ 2 | 0 | 0 | negative | false |
| ≥ 2 | 1 | 0 or 1 | negative | true |
| ≥ 2 | ≥ 2 | < 2 | marginal | false |
| ≥ 2 | ≥ 2 | ≥ 2 | positive | false |

Persist `eligible`, `winning`, `clearing`, `growth_percent`, and `gain_denominator` on the `Policy` value the test inspects. The default file report keeps `verdict` null.

### Tuning

`tune` accepts tuning bytes only. Score is the sum of Baselines-only `file_len` under `encode_block_model` for that config. Persist every grid point: `cap`, `delta` (`1` or `0.5` stored as the strings `1` and `0.5` in the JSON, not as a binary float), `initial` (`scalar` or `by_depth`), `p` 32768, `score`. Sort points by the tie key before writing so the file is deterministic. The winner is the first point after sorting by `(score, cap, delta_rank, initial_rank)` with `delta_rank` 0 for DELTA 1 and `initial_rank` 0 for scalar.

Do not call `tune` from the default CLI. A unit test uses a 32-byte tuning input and checks the winner, the tie, and `EvaluationLeak`.

Width grid is the constant `not-run` / `d09-open`. Do not emit alternate formulas.

### Budget

`timing.log` is text, one `key=value` per line, in this order: `host`, `scope`, `start`, `end`, `completed`, `budget_seconds`, `within_budget`. `scope` is `benchmark`. `budget_seconds` is `86400`. `start` and `end` are RFC 3339 with the offset the clock returns. The library takes a clock and a host string. The CLI reads the host, then the clock, then runs the arms. A test clock does not sleep.

`reduce_budget(over_budget, tuning_rows, evaluation_rows, cap)`:

- `over_budget` false returns the same rows and cap.
- `over_budget` true drops the tuning rows and keeps every evaluation row. Cap stays at the caller's cap for that step's first result. A second step, `lower_cap`, returns cap `cap / 2` when `cap > 1`, and does not change evaluation ids.
- A plan whose evaluation id set differs from the input is `NewExperiment`.

No test creates a 24-hour run. The fixture command's `within_budget` is whatever the injected elapsed time says.

### CLI

```text
sxp-cli benchmark OUT_DIR MANIFEST GATE
```

`OUT_DIR` must already be a directory. `MANIFEST` and `GATE` must be files. Missing path is `NotFound`. A directory where a file is required is `IsADirectory`. A file where `OUT_DIR` is required is `NotADirectory`.

Success writes `report.json`, `annex.json`, `timing.log`, and one `{id}.sxpc` per sidecar, by temp file plus rename in `OUT_DIR`. Stdout is the `report.json` bytes. Stderr is empty. Exit 0.

Failure exit 1, stderr `sxp-cli: benchmark: <token>`. Tokens include the existing CLI tokens plus `RoundTripMismatch`, `SelectorDefect`, `CostOverflow`, `HashMismatch`, `BadManifest`, `EvaluationLeak`, `NewExperiment`. Remove only temps this call created. Do not remove a previous `report.json` on failure.

Help and usage follow `compress`: exit 0 with help text, exit 1 with `Usage` when arguments are missing. No `--force`, no config file, no stdin payload.

`threads` for the CLI search is 0.

### Tests that must stay green

- `./ci.sh` on toolchain 1.99.1: fmt, clippy `-D warnings`, `cargo test --workspace --locked`, kernel tree check.
- `python3 tools/fetch/test_fetch.py` is unchanged. Do not edit `corpora/gate-report.json` or `corpora/manifest.toml`.
- Existing `sxp-cli` compress and decompress tests, including the round-trip matrix and the malformed fixtures.
- `cargo test -p sxp-bench --locked --test synthetic`.

New tests live in `crates/sxp-bench/tests/` and `crates/sxp-cli/tests/`. No `#[cfg(test)]` module in `src/` unless the crate already uses that pattern. This crate does not. Do not add a second `#[test]` to `heap_bound.rs` or `block_heap.rs`. Do not add a global allocator.

### Project Structure Notes

- Kernel changes are limited to accounted sizes and the model side path. No new dependency in `sxp-kernel`.
- `sha2`, `serde`, `serde_json`, and `toml` are already `sxp-bench` dependencies. Do not add another.
- `sxp-cli` already depends on `sxp-bench` and does not call it. This story is the caller.
- Format version stays 1. Codebook version stays 1.
- `threads >= 2` must use the encoder's existing local pool, never a global rayon pool.

### Previous story intelligence

- Story 1.12 left `corpus_gate_ready` false and forbade `g2_ready`. Overlap evidence is not inside `gate()`. A reportable run requires overlap `clear`, which this story does not have, so the default report stays exploratory.
- Story 1.11 pinned compress to `fixture-v1` and header prune. Changing that output fails the story.
- Story 1.10 left `generate_codebook("production")` as `UnresolvedProduction`. Do not catch that error and continue.
- `span_cells` already documents `Skipped` as sidecar sentinel `0xFFFF`. Use it.
- `read_file` writes decoded bytes before the CRC check. The benchmark's check is equality of the returned bytes. Do not change that order.
- Review patches in 1.12 made hash, archive, and gate failures typed. Mirror that: no context strings inside library errors. The CLI adds the `sxp-cli: benchmark:` prefix only.

### Git intelligence

Recent commits are one story each: `feat: fetch the corpus manifest (v0.0.18)`, then compress, codebook search, and baseline selection. Match that shape only if a later release asks for a commit. This story does not bump the version.

### Latest technical information

Workspace crate version is 0.0.18. Toolchain is 1.99.1 via `rust-toolchain.toml`. Pinned crates already in the bench manifest: `serde` 1.0.229, `serde_json` 1.0.154, `toml` 1.1.8, `sha2` 0.11.1. Do not upgrade them. `clap` stays 4.6.7. RECIP formulas are the spec seed's MEASURE rows, not a newer paper.

### Project context reference

No `project-context.md` is present. Kernel rules in `crates/sxp-kernel/src/lib.rs` and `.cursor/rules/cargo-rustup.mdc` are the standing constraints: cargo and rustup only, `forbid(unsafe_code)`, denied unwrap/expect/panic/indexing, no `build.rs` in the kernel, lockfile `--locked`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.13]
- [Source: _bmad-output/planning-artifacts/epics.md#Elicitation amendments] amendments 2, 6, and 8
- [Source: _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md#AD-14]
- [Source: _bmad-output/specs/spec-serexpai/SPEC.md#SM-1]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md] RECIP DELTA rows
- [Source: _bmad-output/implementation-artifacts/1-12-fetch-the-corpus-manifest.md]
- [Source: crates/sxp-encoder/src/select.rs]
- [Source: crates/sxp-kernel/src/lib.rs]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- 2026-11-04: `./ci.sh` exit 0 on the host. `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu` exit 0. The qemu prefix `/usr/aarch64-linux-gnu` is present. Python stayed out of `ci.sh`.

### Completion Notes List

- Six arms, accounts, sidecars, annex, `classify`, `tune`, and `reduce_budget` are in. The default report stays `exploratory/unreportable` with `verdict` null. `corpus_gate_ready` and `g2_ready` are unchanged. `compress` still encodes `fixture-v1` with the strawman model.
- AC08 thread identity follows amendment 2. Byte counts, arm sizes, work counts, and sidecar bytes match at threads 0 and 2. The schema's `threads` field records the request, so `report.json` differs in that integer only. The annex stays outside the check.
- A correct selector cannot emit `SelectorDefect` from the CLI. `sizes_hold` covers that token. The CLI install test uses `HashMismatch`, which returns on the same path, before any rename, and leaves a previous `report.json` in place.
- `tune` does not run from the CLI. `tuning_json` is the canonical `tuning.json` document. The winner is not installed into `encode_block` or `compress`.
- A decode failure and a byte mismatch both return `RoundTripMismatch`. `assert_same_evaluation` compares the evaluation id set, ignoring order.
- Host `./ci.sh` and the aarch64 qemu run both exited 0. Review fixes reject duplicate and control-character ids, reject a synthetic id that is not class `synthetic`, and reject a tuning row before search. Annex entropy stays the sum of each selected block's histogram, rounded once. A negative overhead is a signed microbit difference. Amendment 2 is how AC08 is read: arm sizes, work counts, and sidecars match across thread counts, and the `threads` field records the request.

### File List

- crates/sxp-kernel/src/block.rs
- crates/sxp-kernel/tests/account.rs
- crates/sxp-kernel/src/lib.rs
- crates/sxp-kernel/src/range_coder.rs
- crates/sxp-encoder/src/lib.rs
- crates/sxp-encoder/src/select.rs
- crates/sxp-bench/src/lib.rs
- crates/sxp-bench/src/budget.rs
- crates/sxp-bench/src/classify.rs
- crates/sxp-bench/src/error.rs
- crates/sxp-bench/src/run.rs
- crates/sxp-bench/src/sidecar.rs
- crates/sxp-bench/src/tune.rs
- crates/sxp-bench/tests/report.rs
- crates/sxp-cli/src/args.rs
- crates/sxp-cli/src/benchmark.rs
- crates/sxp-cli/src/lib.rs
- crates/sxp-cli/tests/benchmark.rs
- _bmad-output/implementation-artifacts/1-13-report-whether-formulas-beat-baselines.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-11-03: Story context created. Default benchmark is exploratory until the corpus gate, D-07, D-09, and overlap close.
- 2026-11-04: Benchmark report, sidecars, annex, tuning document, and `sxp-cli benchmark`. The published status stays exploratory.
