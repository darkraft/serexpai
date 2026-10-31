# Rubric review — ARCHITECTURE-SPINE.md (MBSC v1)

Reviewer: independent, read-only. Spine, PRD and `.memlog.md` read. The spine was not edited.
Companions `bitstream-spec-seed.md` and `walkthrough.html` are listed in the spine front matter but **do not exist yet** in the folder. Several deferrals lean on them (see F-3).

## Verdict

The spine is a strong, mostly enforceable decomposition. The crate graph, exact additive Block cost, and DP partition are sound, and the numeric claims check out. Its determinism contract has real holes: the tie-break order is not a total order over what it is applied to, and Candidate id is undefined. The operational envelope is also thin, and several "spec-seed" deferrals point at a file that does not exist.

## Findings (ordered by severity)

### F-1 [critical] AD-10 / AD-13 / AD-9: the "total order" is not total and Candidate id is undefined
- AD-10 says that per Block the selector breaks ties by "fewest Blocks, then lowest Candidate id". "Fewest Blocks" is a property of a partition, not of a single Block's Candidate choice. It is meaningful only inside the DP.
- Nothing orders two partitions with equal size and equal Block count, for example 1024+256 against 256+1024, or a different mix of lengths. The DP and AD-13's parallel reductions therefore have no deterministic answer. That breaks the byte-identical claim, SM-4 and FR-12.
- "Candidate id" is never defined: not its numbering, not whether it is kind code plus Codebook index, and not whether the Codebook recipe order is part of it.
- AD-10 also says Raw wins "unless the alternative is strictly smaller". That is only consistent with "lowest Candidate id" if Raw has the lowest id. This is not stated.
- The file-level choice between Blocked output and Whole-file raw has no tie rule. The empty input is the concrete case: Blocked with zero Blocks and Whole-file raw are both 3 bytes, and AD-11 does not say which mode is used.
- **Fix:** define one lexicographic key, for example `(total_size, block_count, sequence of length codes, then per-Block Candidate id)`. Define Candidate id as `(kind code, Codebook index in recipe order)` with Raw = 0 and kind codes fixed in the spine. Add: "file level: Whole-file raw wins ties; empty input is encoded as Whole-file raw (or Blocked, pick one)". State that AD-13 reduces over this key.

### F-2 [high] AD-9 last-Block rule vs the "optimal partition" claim; non-canonical streams
- "Smallest nominal length that covers the remainder" is not well defined for a DP, where the last Block may start with more than 2048 bytes remaining. The rule then constrains the DP, so "gets an optimal partition" holds only relative to that constraint.
- It is also unclear whether the decoder accepts or rejects a non-smallest nominal length on the final Block.
- **Fix:** state the exact DP transition set for the tail (a Block may be partial only if `offset + nominal ≥ N` and no smaller nominal covers). Say "optimal subject to the tail rule". State whether a non-canonical nominal is rejected (one canonical encoding is preferable).

### F-3 [high] Deferred items that can make the kernel, encoder and Python reference decoder diverge, and the companions are missing
Deferred or "spec-seed content" items that the units below must agree on bit for bit:
- Block header kind codes and field widths.
- Evaluator parameter limits. AD-5 gives only "for example" bounds, and the decoder "rejects any stream that breaks a limit", so the limit table is normative.
- The meaning of the "16-bit normalized position". If it is normalized by Block length, ŷₙ depends on the nominal length as well as `n`, which contradicts AD-4's "depends only on n and parameters".
- Sine LUT size and interpolation.
- rANS state width and renormalization bounds, the maximum total M, and the exact termination form.
- Model constants (initial count, increment, rescale threshold).

The spine says these are not "a divergence point once AD-6 and AD-11 hold". That is not true for the Python decoder and the kernel, which are written independently. `bitstream-spec-seed.md` is the claimed home and it does not exist.
- **Fix:** either deliver the seed before the spine is marked ready, or move the following into AD-5, AD-7 and AD-11 as rules: the normalization definition, rANS state width and M bound, and "the limit table lives in `spec/bitstream.md`, is normative, and is checked by an evaluator-vector set". Keep only the actual numeric values deferred, and give each a named owner file and revisit gate (the model constants already have one).

### F-4 [high] Operational and environmental envelope is nearly silent (checklist 6)
- Deployment is one sentence and the CI provider is deferred. The rest of the envelope lives only in the memlog, not the spine: the CI contract (fmt, clippy, tests, golden vectors on both architectures, aarch64 via native runner or qemu), the rustup requirement (rustc and cargo are not installed on the host), and the CLI conventions.
- Decisions missing for environments:
  - dev: toolchain pin via `rust-toolchain.toml`, and whether `Cargo.lock` is committed. This is needed for SM-4 because rayon, clap and others are otherwise version ranges.
  - CI: how aarch64 execution is obtained, and what runs before a remote exists. AD-1 and AD-15 say "CI fails if…", which is unenforceable until a remote exists.
  - benchmark runs: host, corpus cache location, report destinations, network scope.
- Decisions missing for operations and release: how the artifact is distributed (source-only?), and cleanup of temp files on failure or signal.
- `tools/fetch` has no language, dependency or hash-tool decision. It is the only networked component.
- **Fix:** add a short "Environments & Operations" section. Pin the toolchain and commit `Cargo.lock`. Add a local `xtask`/script that is the single CI contract. State that aarch64 is verified by native runner or qemu-user. Name the fetch tool (a stdlib-only Python script reusing the sha256 check, or a Rust bin in `mbsc-bench`). Say "source-only, no signing, no telemetry". Mark the CI provider as deferred with a trigger, as now.

### F-5 [medium] AD-12 memory bound is not testable as worded
- "Fixed memory budget of about 64 KiB" cannot be measured: process RSS of a Rust binary exceeds it, and "about" has no threshold.
- The PRD asks for a stated multiple of Block length (plus declared output size). The spine's stronger bound is fine, but it needs a measurement method.
- The temp-file-plus-rename rule is silent on the pre-existing target and on a decompression-bomb disk blow-up (a constant Block can expand roughly 300×).
- "Cannot panic on any byte sequence" has no enforcement mechanism.
- **Fix:** write "heap working set ≤ 64 KiB (excluding fixed I/O buffers), asserted by a counting global allocator test, no allocation sized by N". Enforce no-panic with `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]` in the kernel and a decode fuzz/proptest on random bytes. Add the existing-target behaviour (left untouched on failure).

### F-6 [medium] AD-1 enforcement check conflicts with the Stack
- The Stack lists `proptest (dev)` and Verification requires property tests for AD-3 (a kernel function). `cargo tree -p mbsc-kernel` includes dev-dependencies by default, so the AD-1 check fails if the property tests live in the kernel crate.
- AD-1 also does not stop `mbsc-cli`'s `decompress` from calling encoder code. Its "decode path is cli → kernel only" is a convention.
- **Fix:** specify `cargo tree -p mbsc-kernel -e normal,build`, or place the tests in a separate test crate. Optionally add a `cargo-deny`/script check that `decompress`'s module imports only `mbsc_kernel`.

### F-7 [medium] FR-4 / OQ6 and OQ9: Codebook cap and composition are only partly decided
- The PRD asks for the size "per family" upper bound and the composition (Formulas per family, Q values, parameter ranges). The spine sets a single 4096 total and "documented recipe".
- Nothing guarantees all three families are non-empty, which FR-4 requires.
- The recipe is not required to avoid floating point or platform libm. A float-based recipe could make the Codebook differ across x86-64 and aarch64 and silently break SM-4. The hash only detects it.
- **Fix:** state per-family caps, for example poly ≤ P, trig ≤ T, recur ≤ R with sum ≤ 4096, and "each family ≥ 1". State that the recipe is integer-only (or must reproduce the same hash on both architectures in CI). Leave the actual composition to the recipe doc, with a named owner.

### F-8 [medium] Entropy screen default and benchmark/compress parity (AD-10, FR-6, FR-12)
- AD-10 calls the screen "optional" and disabled only in `benchmark`. If `compress` enables it, the same input can produce a different Bitstream from the one `benchmark` measured, and SM-1 would not describe the shipped tool.
- **Fix:** state the default is off in v1 (or delete the screen from v1).

### F-9 [medium] Benchmark coverage gaps (AD-14 vs FR-14 / SM)
AD-14 does not cover several FR-14 items:
- the Total-encoded-size breakdown (header, parameters, model signalling, payload);
- the zero-residual share;
- per-family and per-Baseline win shares;
- forcing a single Baseline for a whole input;
- the Whole-file raw arm;
- the Gain definition (best whole-input Baseline per input, summed per class).

The kernel `encode_block` API must return a size breakdown for this to be possible. Neither the contract nor the interface between `mbsc-encoder` and `mbsc-bench` (for example an `EncoderConfig` with a Formulas-off switch and a Candidate-cost sink) is fixed. FR-15's synthetic-signal seed and fetch mechanism also have no AD. They are in the memlog only.
- **Fix:** add these lines to AD-14. State that kernel `encode_block` returns `{header, params, payload}` byte counts. Define a minimal encoder-to-bench interface (config plus cost sink) in the spine.

### F-10 [medium] AD-7 model risk is not tracked
- The PRD's OQ2 worry is per-Block cost at 256 bytes. With signalling eliminated, the equivalent cost is adaptive-learning overhead (uniform start over 256 symbols, reset every Block). Order-0 adaptation from a uniform prior may cost nearly raw size on 256-byte Blocks.
- FR-8 requires that "per-Block model signalling cost is measured"; the number is zero by construction, which is not informative.
- **Fix:** add to AD-14 a reported "learning overhead" (Block cost minus the Block's static zero-order entropy) per length and class. Record in Deferred that the init count and increment are the first tuning targets.

### F-11 [low] Numeric claims check (all follow, with caveats)
- Empty-input header: magic (1) + version/mode (1) + LEB128(0) (1) = 3 bytes. ✓ (mode ambiguity in F-1.)
- Whole-file raw maximum: 2 + 5 = 7, so N + 7. ✓ N < 2³⁵ fits in 5 LEB128 bytes. ✓ The bound is loose for N < 2²⁸ (N + 6 or less). The behaviour for N ≥ 2³⁵ (encoder refuses with exit 1?) is unstated.
- PRD Glossary says Whole-file raw has a "fixed-size header". AD-11 makes it 3–7 bytes. Note the divergence from the glossary, or fix the header at 7 bytes.
- AD-5 overflow example: 15 Horner steps with |c| < 2³², a position of at most 2¹⁶ and normalized |x| ≤ 1 gives |acc| ≲ 2³⁵ and a product ≲ 2⁵¹ < 2⁵². Plausible, but it relies on the normalization in F-3.
- FR-6 "actual size" vs AD-8: consistent. Exact size is additive per byte-aligned Block, and AD-2 obtains it from the kernel.
- AD cross-references: the AD numbers cited in the capability map and Conventions are correct. Small gaps: FR-4 maps only to AD-13 (AD-4 also binds it). FR-9 omits AD-6 and AD-9 (Block length code). FR-15 maps to AD-14 only.

### F-12 [low] SM-5 and "independent" reference decoder
The Python decoder is "written from the spec, not translated". That is not checkable by CI and does not satisfy SM-5 (an independent party). Python's unbounded integers also cannot detect i64 overflow. **Fix:** state it as an oracle only, and that SM-5 stays an open owner and timing item. Note limit checks come from the normative limit table (F-3).

### F-13 [low] Verification list
It lacks explicit boundary vectors required by FR-9 and SM-2: empty, shorter than one Block, exactly one Block, partial final Block, and trailing garbage after the last Block. Add them to the golden vectors. Add a determinism check on both architectures (same Codebook hash and Bitstream bytes).

### F-14 [low] Stack
- Versions checked on crates.io and the Rust release channel on 2026-10-31: rayon 1.12.0, clap 4.6.7, serde 1.0.229, toml 1.1.8, sha2 0.11.1, proptest 1.12.0, and Rust stable dated 2026-10-15 all match. Nothing dubious found.
- **Missing:** `serde_json` (1.0.154 current) is needed for "JSON out" and is not in the Stack (or hand-write the JSON and say so).
- `criterion` (memlog) is not in the spine. The aarch64 test mechanism (`cross` / qemu) and Python test runner are unnamed.

## Checklist summary
1. Divergence points: **partial.** F-1, F-2, F-3, F-9.
2. AD Rules enforceable: **mostly.** Gaps in AD-10/13 (F-1), AD-12 (F-5), AD-1 (F-6), AD-15 provenance (F-12).
3. Deferred cannot cause divergence: **fails.** Kind codes, limits, LUT, rANS width, model constants (F-3), and the CI contract (F-4).
4. Technology current: **pass**, with serde_json missing (F-14).
5. PRD coverage: **FR/SM all mapped.** OQ2, OQ3 and the numeric parts of OQ9 are decided. OQ6 Codebook composition and the FR-4 per-family cap are partial (F-7). FR-14 detail is thin (F-9).
6. Structural dimensions: **repo and language and crate topology decided. Deployment, infra and operations thin** (F-4).
