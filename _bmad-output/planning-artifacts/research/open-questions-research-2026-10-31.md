---
name: 'MBSC v1: research on the six open questions'
type: research-report
created: '2026-10-31'
answers: SPEC-serexpai "Open Questions" (corpora, licence, checksum, encode time, SM-5, spec-seed constants)
inputs:
  - _bmad-output/specs/spec-serexpai/SPEC.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md
  - _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md
  - _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/addendum.md
status: draft for author decision
---

# MBSC v1: research on the six open questions

This report gives 3 to 5 candidate answers per open question, each with evidence, MBSC-specific trade-offs and primary sources, then a ranked recommendation. Two kinds of evidence are used:

- **Cited:** primary sources fetched on 2026-10-31 (RFCs, official repositories, corpus pages, papers). Every link in this report was reached unless marked otherwise.
- **Local check:** small numeric experiments I ran against the spec-seed definitions (sine LUT error, bit-model behaviour, rANS sizes, corruption detection, file(1) magic). Method and code are in Appendix A. They are not a substitute for the MEASURE step on `tuning` inputs; they test the *structure* of the constants, not their final values.

Anything I could not verify is marked **[UNVERIFIED]**. Nothing here is legal advice.

## Executive summary

| # | Question | Recommendation (rank 1) | Changes the current spec or architecture? |
|---|---|---|---|
| 1 | Corpora | Text: tune on Canterbury/Calgary text, evaluate on Silesia `dickens` and `webster`. PCM: **8-bit PCM derived by a documented transform from LJ Speech** (public domain, plain WAV), split by chapter. Executables: tune on Calgary `obj1`/`obj2` and Canterbury `sum`, evaluate on Silesia `ooffice` plus a pinned modern ELF (amd64 and arm64) from snapshot.debian.org. Control: CC0 FLAC test-bench files plus `enwik8.zip`. At least 3 evaluation inputs per class, 4 to 16 MiB per class. | Needs one clarifying rule: a corpus-side, documented 16-to-8-bit transform is corpus preparation, not compressor preprocessing. |
| 2 | Licence and contribution model | Code and Python decoder under **MIT OR Apache-2.0** (the Rust norm, with an explicit patent grant); `spec/bitstream.md` under **CC BY 4.0** (or OWFa 1.0 if a patent promise on the spec is wanted); vectors and the LUT under **CC0**; **DCO** sign-off with inbound = outbound, no CLA. Document rANS prior art (Duda 2013, ryg_rans 2014) in the spec. | No. |
| 3 | Integrity checksum | Before public release, add an **optional whole-file CRC-32 (IEEE, gzip/PNG polynomial)** signalled by the reserved bit 6 of byte 1, 4 bytes at end of file, default on in `compress` and in every benchmark arm. Never per-Block. | Yes: the bound becomes N + 11 with the checksum (N + 7 without), and the "no checksum in v1" assumption ends at public release. |
| 4 | Encode-time budget | Name a deterministic work unit (coded decisions per input byte) and report it. Budget ≤ 24 h wall-clock per full benchmark on one named host, logged outside the report. Reach it first with exactness-preserving speed-ups (ŷ tables precomputed once per Formula, shared model pass, exact header-cost pruning), then by capping corpus size, and only last by lowering the Codebook cap with sidecar-measured excess bytes. | Clarify that a provably admissible lower-bound prune is not an "entropy screen" under AD-10. |
| 5 | Independent decoder (SM-5) | At spec freeze and before first public release: a **clean-room decoder** written only from `bitstream.md` and the vectors, by a different person or a fresh, sandboxed AI agent from a different model family, in a language other than Rust. Every ambiguity it hits is logged as a spec issue. Add a **zstd-`decodecorpus`-style valid-stream generator** with three-way differential fuzzing in CI. Invite external implementations after release. | Clarifies "who" and "when". The in-repo Python decoder by the same author does not count as independent on its own. |
| 6 | Spec-seed constants | Keep magic `0x8D`, the 5-bit Block header and the rANS core (L = 2²³, 12-bit total, 4-byte flush). Keep the 1025-entry Q15 LUT. **Change the bit model:** 16-bit internal probability, symmetric non-negative update, coder frequency `clamp(p >> 4, 1, 4095)`, then tune CAP among {24 … 255} on `tuning`. **Widen or escape the 4-bit version.** | Yes: AD-7's "12-bit probability" wording, spec seed §5 update rule and clamp, and §2 version packing. |

---

## Q1. Corpora: text, PCM, executables and the already-compressed control

### Constraints that shape the answer

- CAP-13 requires source, licence, SHA-256 and a tuning/evaluation role per input. The fetch tool is **Python standard library only** and nothing is vendored. Two consequences of the stdlib rule: Python 3.14 has `wave`, `tarfile` (gz/bz2/xz) and `zipfile`, but `audioop` is gone (local check: `ModuleNotFoundError` on Python 3.14.4), and there is no FLAC decoder. **FLAC-packed audio cannot be a PCM source** for a stdlib-only fetch tool. It can only serve as the already-compressed control.
- SM-1 needs two testable Real classes, so class choice is the main lever on an "inconclusive" outcome. The addendum already argues that text is unlikely to show Gain and that raw 16-bit PCM is weak without byte-lane separation.
- Encode time (Q4) bounds corpus size. At my estimated 1 to 5 ms per input byte per core, 10 MiB costs about 3 to 15 core-hours.

### Candidate A: classic compression corpora only (Canterbury, Calgary, Silesia)

- **Answer:** text, executables and an image-like control from the three standard corpora. Tune on Canterbury/Calgary, evaluate on Silesia.
- **Evidence:** Silesia has 12 files, 211,938,580 bytes total, each with a published MD5. Examples: `dickens` 10,192,446 B, `webster` 41,458,703 B, `ooffice` (a Windows DLL from OpenOffice 1.01) 6,152,192 B, and `mozilla` (tarred Tru64 executables and archives) 51,220,480 B ([Silesia page](https://sun.aei.polsl.pl/~sdeor/index.php?page=silesia)). Canterbury has 11 files ("will not be changed so that it can be used as a benchmark in future"), including the SPARC executable `sum` (38,240 B). Calgary has 14 files, including `obj1` (VAX, 21,504 B) and `obj2` (Mac, 246,814 B) ([Canterbury descriptions](https://corpus.canterbury.ac.nz/descriptions/)).
- **Trade-offs for MBSC:** these corpora are frozen and widely cited, which is good for SM-4 and comparability. Neither page states a licence. Record "none stated, fetched from origin, not redistributed" (the addendum says the same). There is **no PCM** at all, so this candidate alone gives at most text plus executables, and text is the class least likely to show Gain. The executables are old (VAX, SPARC, Alpha, Win32).
- **Risk:** a likely "inconclusive" or "negative" verdict driven by class choice, not by the method.

### Candidate B: add 8-bit PCM derived from a public-domain WAV speech corpus (LJ Speech)

- **Answer:** PCM class = LJ Speech clips converted by a documented, deterministic transform (16-bit signed little-endian, keep the high byte, add 128 to get unsigned 8-bit as in 8-bit WAV). Tune on low chapter IDs, evaluate on high chapter IDs.
- **Evidence:**
  - LJ Speech has 13,100 clips (23:55:17 in total), "single-channel 16-bit PCM WAV with a sample rate of 22050 Hz". The licence reads "public domain in the US … There are no restrictions on its use", and the recordings are from LibriVox ([LJ Speech](https://keithito.com/LJ-Speech-Dataset/)). Mozilla Data Collective lists it as CC0-1.0 at 3.00 GB ([MDC listing](https://mozilladatacollective.com/datasets/cmonjhxee01ako007kohpbg34), secondary). The page warns that the source recordings were 128 kbps MP3, so they "may contain artifacts".
  - Precedent: Delétang et al. (2023) converted LibriSpeech int16 to 1-byte samples and coded 2048-byte chunks: "We manually reduce the sample size to 1 byte … chunks of size 2048 bytes". They also used enwik9 and a random control in the same table ([arXiv 2309.10668](https://arxiv.org/abs/2309.10668), App. B).
- **Trade-offs for MBSC:** 8-bit samples make each byte one smooth sample, which is exactly the regime where a polynomial or trigonometric Formula over a 256 to 2048-sample Block can beat previous-byte and direct coding. It is also the class most likely to deliver one of the two SM-1 classes. WAV inside `.tar.bz2` is stdlib-readable. The transform must be in the fetch tool, versioned and hashed (hash both the source archive and each derived file). The report must say the class is "8-bit PCM derived from 16-bit speech" and should also report the raw 16-bit files as a separate, non-counted row, so the derivation cannot hide a negative 16-bit result.
- **Risks:** (1) It may be read as format-aware preprocessing, which the SPEC lists as a non-goal. The distinction to write down is that the *corpus* is defined as 8-bit PCM, and the compressor sees bytes. (2) The MP3-sourced audio has band-limited spectra that may flatter smooth predictors. (3) The archive size of about 2.6 to 3 GB means the tool must stream the tar and stop early [UNVERIFIED: exact archive size and member order]. (4) Speaker diversity is one speaker; chapter splits keep tuning and evaluation byte-disjoint but not speaker-disjoint.

### Candidate C: CC BY 4.0 speech datasets with official splits (LibriSpeech, Speech Commands)

- **Answer:** use dataset-native, speaker-disjoint splits: LibriSpeech `dev-clean` for tuning and `test-clean` for evaluation, or the Speech Commands validation and test lists.
- **Evidence:** LibriSpeech is "License: CC BY 4.0", about 1000 h of 16 kHz read speech, with `dev-clean` 337 MB and `test-clean` 346 MB, and an MD5 list for the archives ([OpenSLR 12](https://www.openslr.org/12/)). Google Speech Commands v0.02 is CC BY 4.0, 105,829 one-second 16-bit, 16 kHz WAV clips ([Warden 2018, arXiv 1804.03209](https://arxiv.org/abs/1804.03209); [TFDS catalog](https://www.tensorflow.org/datasets/catalog/speech_commands), secondary).
- **Trade-offs:** official splits are the cleanest protocol answer to "how do papers split". But **LibriSpeech audio is FLAC inside tar.gz**, which a stdlib-only fetch tool cannot decode, so it fails the AD constraint unless a Python FLAC decoder is written (out of scope). Speech Commands is WAV in tar.gz (stdlib-readable) and CC BY 4.0, which requires attribution in the manifest and report. Its one-second clips are short and mostly silence-padded [UNVERIFIED: padding share], which favours the Constant Baseline.
- **Risk:** CC BY attribution duties, and the archive is about 2.3 to 2.7 GB.

### Candidate D: a modern, pinned executable set from snapshot.debian.org

- **Answer:** evaluation executables are a pinned Debian binary package (for example a statically linked utility) for **amd64 and arm64**, fetched from snapshot.debian.org by timestamp, verified by the SHA-256 in the archive's signed index, with the ELF extracted by the stdlib tool.
- **Evidence:** snapshot.debian.org "allows access to old packages based on dates and version numbers" with stable timestamped URLs ([snapshot.debian.org](https://snapshot.debian.org/)). Debian `.deb` files are `ar` archives containing `data.tar.*`, extractable with stdlib `tarfile` once the simple `ar` header is parsed [UNVERIFIED: compression of `data.tar` for the chosen package; `.zst` members would need a non-stdlib decoder].
- **Trade-offs:** current x86-64 and AArch64 code matches the project's two target platforms and is far more representative than VAX or SPARC objects. Licences are the packages' own, and since nothing is redistributed this only needs recording. Pin the snapshot timestamp so SM-4 holds.
- **Risk:** if a package uses `data.tar.zst`, the stdlib tool cannot unpack it, so choose a package and suite with `.xz` members.

### Candidate E: the already-compressed control

Options, each real third-party compressed data rather than self-generated:

1. **FLAC test-bench files.** CC0: "has waived all copyright and related or neighboring rights" ([flac-test-files README](https://raw.githubusercontent.com/ietf-wg-cellar/flac-test-files/main/README.txt)). They are FLAC-compressed (entropy-coded Rice residuals), small, and include an 8-bit-per-sample file (subset file 23) ([subset README](https://raw.githubusercontent.com/ietf-wg-cellar/flac-test-files/main/subset/README.txt)). Best licence of all candidates.
2. **`enwik8.zip`.** Deflate, 36,445,475 bytes, published next to enwik8's MD5 and SHA-1 ([About the test data](https://mattmahoney.net/dc/textdata.html)).
3. **Kodak PNGs** (deflate inside PNG). The mirror says they were "released by the Eastman Kodak Company for unrestricted usage" ([r0k.us Kodak](https://r0k.us/graphics/kodak/)). This is a secondary claim [UNVERIFIED with Kodak].
4. **Locally generated `.xz`/`.zst`** of a tuning input. Reproducible only if the compressor version is pinned and the output hash recorded; it adds a tool dependency to the fetch step. Weakest.
5. **Seeded random bytes.** These are already in the synthetic set. They are incompressible but not "already compressed", and the SPEC keeps synthetic signals out of Real classes.

**Rejected:** EBU SQAM, whose terms say "You may not copy, modify, merge, publish, distribute…" and restrict use to internal R&D ([EBU terms PDF](https://tech.ebu.ch/files/live/sites/tech/files/shared/testmaterial/use%20of%20EBU%20AUDIO%20test%20sequences.pdf)). Also the ktf CDDA set, which is 64 commercial album sources ([revision 6 CDDA](http://www.audiograaf.nl/losslesstest/Lossless%20audio%20codec%20comparison%20-%20revision%206%20-%20cdda.html)).

### How state-of-the-art work chooses and splits data (evidence for the split rule)

- **Benchmarks that allow tuning to the test data.** The Hutter Prize says "Compressors/decompressors … may be tuned specifically to this benchmark" ([Hutter rules](http://prize.hutter1.net/hrules.htm)). The LTCB selects options "for maximum compression" on enwik9 ([LTCB](https://mattmahoney.net/dc/text.html)). This is exactly what MBSC's SM-C2 forbids, so MBSC cannot copy the most famous protocol.
- **Train/test from ML practice.** Delétang et al. train on enwik8 and evaluate on enwik9 and note that "enwik8 is included in enwik9" ([arXiv 2309.10668](https://arxiv.org/abs/2309.10668)). This overlap would fail AD-14's "no tuning input is an evaluation input" check if hashes were compared on content rather than on files. **MBSC should assert byte-disjointness, not only file-hash disjointness.** If enwik is used, evaluate on a slice starting after byte 10⁸.
- **Fixed, frozen test sets.** Canterbury's main corpus "will not be changed" ([Canterbury](https://corpus.canterbury.ac.nz/descriptions/)). The AOM codec common test conditions publish fixed test-set directories ([media.xiph.org/video/aomctc](https://media.xiph.org/video/aomctc/)).
- **Report per file.** Canterbury warns not to average collections that can grow and to report per file ([Canterbury](https://corpus.canterbury.ac.nz/descriptions/)). MBSC's report already gives per-input wins and margins.

### Ranked recommendation (Q1)

1. **Mix B + A + D + E1/E2.**
   - Text: tune on Canterbury `alice29.txt`, `lcet10.txt`, `plrabn12.txt` and Calgary `book1`, `book2`, `paper1`, `paper2`; evaluate on Silesia `dickens` and `webster`.
   - PCM: tune on the first half of LJ Speech's clip-ID prefixes and evaluate on the second half (for example LJ001 to LJ025 and LJ026 to LJ050 [UNVERIFIED: exact prefix range]), both as derived 8-bit PCM. Also report the raw 16-bit versions as non-counted rows.
   - Executables: tune on Calgary `obj1`/`obj2` and Canterbury `sum`; evaluate on Silesia `ooffice` and a pinned Debian amd64 and arm64 ELF.
   - Control: FLAC test-bench subset files and `enwik8.zip`.
   - Sizing: aim for ≥ 3 evaluation inputs per class and 4 to 16 MiB per class.
2. **A + D only, with no PCM.** Simpler and licence-cleaner, but it risks an inconclusive SM-1 because the stronger PCM class is missing.
3. **C (Speech Commands) instead of LJ Speech.** Better speaker-disjoint split, but CC BY duties and short, padded clips.
4. **LibriSpeech.** Best-known split, but blocked by the stdlib-only fetch tool (FLAC).

**Justification:** SM-1 rewards picking classes where a positionwise smooth predictor can plausibly win. 8-bit PCM is the one class with direct precedent (Delétang et al. used 1-byte LibriSpeech samples in 2048-byte chunks) and a clean public-domain WAV source that a stdlib tool can read. Text and executables still come from frozen, hash-published corpora, so the verdict stays comparable and reproducible. Cross-corpus tuning (Canterbury/Calgary to Silesia) is the strongest available guard against the tuning leakage that LTCB-style protocols allow. The derived-PCM rule and byte-disjointness should be written into the manifest schema.

---

## Q2. Licence and contribution model before first public release

### What comparable projects do (verified)

| Project | Code licence | Contribution gate | Spec/document licence |
|---|---|---|---|
| zstd | Dual "BSD OR GPLv2" ([README](https://github.com/facebook/zstd), [LICENSE](https://raw.githubusercontent.com/facebook/zstd/dev/LICENSE), [COPYING](https://raw.githubusercontent.com/facebook/zstd/dev/COPYING)) | Meta CLA ("complete the Contributor License Agreement") ([CONTRIBUTING](https://raw.githubusercontent.com/facebook/zstd/dev/CONTRIBUTING.md)) | Permissive notice: "Permission is granted to copy and distribute this document for any purpose …, provided that … any substantive changes or deletions from the original are clearly marked" ([format doc](https://raw.githubusercontent.com/facebook/zstd/dev/doc/zstd_compression_format.md)); also RFC 8878 under the IETF Trust |
| xz | 0BSD for liblzma and the CLI tools; format specs public domain or 0BSD ([COPYING](https://raw.githubusercontent.com/tukaani-project/xz/master/COPYING)) | none found | public domain / 0BSD |
| brotli | MIT ([LICENSE](https://raw.githubusercontent.com/google/brotli/master/LICENSE)) | Google CLA ([CONTRIBUTING](https://raw.githubusercontent.com/google/brotli/master/CONTRIBUTING.md)) | RFC 7932 (IETF Trust) |
| lz4 | `lib/` BSD-2, everything else GPL-2.0-or-later ([LICENSE](https://raw.githubusercontent.com/lz4/lz4/dev/LICENSE)) | not checked | same permissive notice as zstd ([frame format](https://raw.githubusercontent.com/lz4/lz4/dev/doc/lz4_Frame_format.md)) |
| FLAC | libFLAC BSD-like (Xiph); tools GPL; docs GNU FDL ([README](https://raw.githubusercontent.com/xiph/flac/master/README.md), [COPYING.Xiph](https://raw.githubusercontent.com/xiph/flac/master/COPYING.Xiph)) | not checked | RFC 9639 (IETF Trust) |
| Monkey's Audio | 3-Clause BSD ([licence](https://www.monkeysaudio.com/license.html)) | n/a | n/a |
| rav1e | BSD-2 ([LICENSE](https://raw.githubusercontent.com/xiph/rav1e/master/LICENSE)) plus the AOMedia Patent License 1.0 in [PATENTS](https://github.com/xiph/rav1e/blob/master/PATENTS) (royalty-free licence to Necessary Claims with defensive termination) | not checked | AV1 spec (AOM) |
| ruzstd (Rust zstd) | MIT; has an explicit AI-contribution policy ([README](https://raw.githubusercontent.com/KillingSpark/zstd-rs/master/Readme.md)) | inbound = outbound | n/a |
| jxl-oxide (Rust JPEG XL) | "Dual-licensed under MIT and Apache 2.0" ([README](https://raw.githubusercontent.com/tirr-c/jxl-oxide/main/README.md)) | n/a | n/a |

The Rust API Guidelines (C-PERMISSIVE) recommend `license = "MIT OR Apache-2.0"` with an inbound = outbound contribution clause. They also say "Crates that desire perfect license compatibility with Rust are not recommended to choose only the Apache license" ([Rust API Guidelines](https://rust-lang.github.io/api-guidelines/necessities.html)).

### Patent landscape for ANS (verified facts; this is not a freedom-to-operate opinion)

- **Microsoft US 11,234,023 B2**, "Features of range asymmetric number system encoding and decoding". Filed 2019-06-28, granted 2022-01-25, status Active, anticipated expiration 2039-06-28. Family filings exist in EP, CN, KR and PCT, and a continuation was published as US 2022/0109891 A1 ([Google Patents](https://patents.google.com/patent/US11234023B2/en); [continuation](https://patents.google.com/patent/US20220109891A1/en)).
  - **Claim 1** is directed to a decoder "using a two-phase structure for RANS decoding operations" with selective state update and selective merging of input ([claims, same page](https://patents.google.com/patent/US11234023B2/en)).
  - The description also covers "selectively flushing (or retaining) the state of a RANS decoder on a fragment-by-fragment basis", with initial state information of "four bytes".
  - **The continuation's claims and status are [UNVERIFIED]**, and so is the status of the EP, CN and KR members.
- The USPTO had issued a final obviousness rejection on 2020-10-27 before the grant ([The Register, 2021](https://www.theregister.com/software/2021/03/13/third-times-a-harm-microsoft-tries-to-get-twice-rejected-encoding-patent-past-skeptical-examiners/408210)). Duda said it "looks like just the description of the standard algorithm". Google abandoned an earlier ANS application in 2018 ([The Register, 2022](https://www.theregister.com/software/2022/02/17/alarm-raised-after-microsoft-wins-data-encoding-patent/394460)).
- **Prior art MBSC should cite in the spec:** Duda's ANS paper ([arXiv 1311.2540](https://arxiv.org/abs/1311.2540), 2013), and ryg_rans `rans_byte.h` ("public domain - Fabian 'ryg' Giesen 2014"), which already has L = 2²³, 32-bit state, byte renormalisation and a 4-byte flush ([ryg_rans](https://raw.githubusercontent.com/rygorous/ryg_rans/master/rans_byte.h)). Both predate the 2019 filing.
- **MBSC-specific note:** MBSC flushes and resets at *every* Block unconditionally, with no syntax element choosing flush or retain. That differs from the "selectively" signalled flush in the description, but only a patent attorney can assess claim scope.
- **What a licence can and cannot do:** Apache-2.0 §3's grant covers only patents "licensable by such Contributor" ([Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)). No licence choice can neutralise a third party's patent.

### Candidate A: MIT OR Apache-2.0 for code; CC BY 4.0 for the spec; CC0 for vectors; DCO

- **Answer:** Rust crates and the Python reference decoder under `MIT OR Apache-2.0`. `spec/bitstream.md` under CC BY 4.0. `spec/vectors/`, `spec/sine_lut.*` and the manifest schema under CC0. Contributions certified with DCO 1.1 sign-off (`Signed-off-by`) and inbound = outbound.
- **Evidence:** this is the Rust ecosystem default (Rust API Guidelines; jxl-oxide). Apache-2.0 §3 adds a contributor patent grant with litigation termination, and §5 makes contributions inbound = outbound by default ([Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)). DCO 1.1 certifies the right to submit "under the open source license indicated in the file" ([developercertificate.org](https://developercertificate.org/)).
- **Trade-offs:** maximises later library embedding (the SPEC assumes a large audience). CC0 vectors let independent decoders (SM-5) ship the vectors with no attribution friction. Dual licensing costs nothing at runtime.
- **Risk:** a CC BY spec gives no patent promise, so an implementer relies only on Apache-2.0 §3 for the code. GPLv2-only projects can still use the MIT arm.

### Candidate B: single permissive licence (0BSD, BSD-2 or MIT), like xz, rav1e and brotli

- **Answer:** one short licence everywhere, for example 0BSD as xz switched to.
- **Evidence:** xz uses 0BSD and notes that an acknowledgement is "polite … but it is not legally required" ([xz COPYING](https://raw.githubusercontent.com/tukaani-project/xz/master/COPYING)).
- **Trade-offs:** simplest possible. But there is no express patent grant from contributors, which matters more for a format whose entropy coder has a live patent family around it.
- **Risk:** low legal friction, slightly weaker patent posture than A.

### Candidate C: dual "BSD OR GPLv2", like zstd

- **Answer:** copy zstd's dual licence to allow inclusion in GPLv2-only trees such as the Linux kernel.
- **Trade-offs:** useful only if kernel inclusion is a goal. For a Rust research CLI it adds complexity without benefit. Rust crates conventionally use MIT OR Apache-2.0.

### Candidate D: copyleft (GPL-3.0 or MPL-2.0), like cmix and paq8 (GPL)

- **Answer:** keep derivatives open.
- **Trade-offs:** contradicts the SPEC assumption that v1 "should not close off … later library embedding". The addendum already recommends permissive terms.

### Candidate E: spec under OWFa 1.0 (copyright plus patent non-assert) instead of CC BY

- **Answer:** publish `bitstream.md` under the Open Web Foundation Final Specification Agreement.
- **Evidence:** OWFa 1.0 grants copyright and an "irrevocabl[e] promise not to assert my Granted Claims" for implementations of the specification ([OWFa 1.0](https://www.openwebfoundation.org/the-agreements/the-owf-1-0-agreements-granted-claims/owfa-1-0)).
- **Trade-offs:** gives implementers the author's own patent promise covering the spec as such, which CC BY cannot. It is less familiar than CC BY and still cannot cover third-party patents.

### Contribution model: DCO or CLA

CLAs are used by corporate-backed projects that may relicense: Meta for zstd, Google for brotli, and Google's CLA also asks contributors to say if they know their code "infringes on other people's patents" ([brotli CONTRIBUTING](https://raw.githubusercontent.com/google/brotli/master/CONTRIBUTING.md)). For a single-author project with no relicensing plan, DCO + inbound = outbound is lighter and does not deter contributors. In 2026 an explicit **AI-assisted contribution policy** is also common practice: a human must have "read, verified and decided" the change ([ruzstd](https://raw.githubusercontent.com/KillingSpark/zstd-rs/master/Readme.md), citing the LLVM policy).

### Ranked recommendation (Q2)

1. **Candidate A** (MIT OR Apache-2.0 code, CC BY 4.0 spec, CC0 vectors and LUT, DCO, plus an AI-contribution note). Upgrade the spec to **OWFa 1.0 (Candidate E)** if the author wants to give an explicit patent promise on the format.
2. **Candidate B** (0BSD everywhere).
3. **Candidate E** as the spec licence combined with B for code.
4. **Candidate C.**
5. **Candidate D.**

**Justification:** the decisive factors are the Rust-ecosystem default, the SPEC's wish not to close off embedding, and the ANS patent climate. Apache-2.0 is the only mainstream permissive licence with an express contributor patent grant, and dual licensing with MIT keeps GPLv2 compatibility, as the Rust guidelines recommend. Splitting the licence by artefact type (code, prose, normative data) mirrors what zstd and lz4 do for their format documents and makes SM-5 vectors freely embeddable. Separately from the licence, cite Duda (2013) and ryg_rans (2014) in the spec's rANS section and avoid adding a *signalled, selective* flush or retain feature; that keeps MBSC away from the Microsoft description language. Get an attorney's view only if commercial distribution is planned.

---

## Q3. Integrity checksum before first public release

### What the comparable formats do (verified)

| Format | Mandatory checks | Optional checks | Notes |
|---|---|---|---|
| gzip (RFC 1952) | CRC32 + ISIZE in the trailer ("A compliant compressor must produce files with correct ID1, ID2, CM, CRC32, and ISIZE") | header CRC16 (FHCRC) | [RFC 1952 §2.3.1, §2.3.1.2](https://www.rfc-editor.org/rfc/rfc1952.html) |
| xz | CRC32 of Stream Flags and of each Block header | content check None / CRC32 (4 B) / CRC64 (8 B) / SHA-256 (32 B); "Implementations SHOULD support at least … None and … CRC32" | header CRC separates "corrupt" from "unsupported" ([xz-file-format §2.1.1.2–3](https://tukaani.org/xz/xz-file-format.txt)) |
| zstd (RFC 8878) | none | 4-byte Content_Checksum = low 32 bits of XXH64; a decoder "may also ignore … the checksum" | [RFC 8878 §3.1.1](https://www.rfc-editor.org/rfc/rfc8878.html) |
| lz4 frame | 1-byte header checksum (second byte of xxh32) | per-block xxh32 (4 B) and content xxh32 (4 B), "Its usage is recommended" | [lz4 frame format](https://raw.githubusercontent.com/lz4/lz4/dev/doc/lz4_Frame_format.md) |
| FLAC (RFC 9639) | CRC-8 on every frame header (x⁸+x²+x+1), CRC-16 on every frame (x¹⁶+x¹⁵+x²+1) | MD5 of the unencoded audio in STREAMINFO (0 = unknown) | [RFC 9639 §9.1.8, §9.3, §8.2](https://www.rfc-editor.org/rfc/rfc9639.html) |
| brotli (RFC 7932) | none in the stream; the RFC's CRC-32 appendix only identifies the dictionary and transform tables | none | [RFC 7932 App. C](https://www.rfc-editor.org/rfc/rfc7932.html) |
| PNG | CRC-32 on every chunk | n/a | "CRC at the end of every chunk provides a robust defence against accidentally corrupted data" ([PNG 3rd ed.](https://www.w3.org/TR/png-3/)) |

On detection strength: the IEEE 802.3 CRC-32 has Hamming distance HD = 4 up to 91,607-bit messages and HD = 3 beyond, so it detects all 3-bit errors in a 2048-byte Block (16,384 bits) ([Koopman, DSN 2002](https://users.ece.cmu.edu/%7Ekoopman/networks/dsn02/dsn02_koopman.pdf)). Like any 32-bit CRC it detects all bursts of 32 bits or fewer, and a random corruption escapes with probability about 2⁻³² (standard CRC theory).

### What MBSC already detects without a checksum (local check, Appendix A.4)

- **rANS payload corruption is caught almost always.** I implemented spec seed §5–§6 exactly, encoded 6,000 random 256-byte residual Blocks, XOR-mutated one random byte in each rANS stream, and decoded. **0 of 6,000** were accepted: the decoder rejected each for `x ≠ L` at the end, an out-of-range initial state, or reading past the end. That is an upper bound on undetected payload corruption of about 0.05% (95% one-sided, rule of three).
- **Header and parameter corruption is NOT caught.** In MBSC the residual model and rANS decoding depend only on decoded symbols, never on ŷ (AD-7 and §5). So a flipped bit in a Formula coefficient, the Constant byte `c`, `q`, `ω`, or a kind swap between Direct (1) and Previous-byte (2) decodes "successfully" to wrong bytes, unless the value leaves its limit table. This follows directly from the spec. A Trigonometric Formula of order 16 at `cw = 28` carries about 122 bytes of unprotected parameters per Block.
- **Raw Blocks and Whole-file raw** have no check at all.

So framing alone leaves a real silent-corruption surface. This is the evidence against the SPEC assumption that "malformed-input detection relies on framing" once real users are involved.

### Candidate A: no checksum (status quo for v1)

- **Answer:** keep relying on framing and the rANS end-state check.
- **Trade-offs:** zero bytes, N + 7 kept, simplest spec. Brotli does the same. But parameter-field and Raw corruption is silent (above). For a public compressor that is a poor default.
- **Fit:** fine for the internal rate experiment, where every round-trip is verified by the benchmark anyway (AD-14).

### Candidate B: optional whole-file CRC-32 behind reserved bit 6 of byte 1

- **Answer:**
  - **Signal:** bit 6 of byte 1, currently reserved and required to be zero, becomes `has_checksum`.
  - **Payload:** when the bit is set, 4 bytes `LE32(CRC-32/ISO-HDLC(decoded data))` follow the last Block, or follow the N raw bytes in Whole-file raw mode.
  - **Default:** on in `compress` and in **every** benchmark arm, so `compress` and the full-MBSC arm still produce identical bytes (AD-10).
  - **Decoding:** the decoder must verify the CRC when it is present.
- **Evidence:** this is the same polynomial as gzip, PNG and xz's CRC32 check. Python's stdlib has it (`zlib.crc32(b"123456789") == 0xCBF43926`, local check), so the stdlib reference decoder and an independent decoder get it for free. In the zero-dependency kernel it is about 20 lines of table code. The xz precedent shows the value of "reserved bits must be zero" for adding exactly such a field later.
- **Trade-offs for MBSC:**
  - **Bound:** N + 11 with the checksum, N + 7 without. CAP-8, AD-10, AD-11 and SM-3 must state both.
  - **Rate fidelity:** +4 bytes per file in every arm. SM-1 compares summed sizes across arms on the same inputs, so the comparison is unaffected. Absolute bits per byte rise by 32/N, which is below 10⁻⁵ bpb on a 4 MiB input.
  - **Determinism:** the CRC is deterministic, so SM-4 is unaffected.
  - **SM-5:** one extra field, implemented with stdlib code in Python.
- **Risk:** small spec change and a version bump (AD-15).

### Candidate C: mandatory whole-file CRC-32 (gzip style)

- **Answer:** always append 4 bytes; no flag.
- **Trade-offs:** one fewer branch and reject rule, always protected, bound N + 11 always. The empty input grows from 3 to 7 bytes. Loses the ability to emit check-free streams for the experiment.

### Candidate D: per-Block CRC (FLAC CRC-16 or lz4 block xxh32 style)

- **Answer:** 2 or 4 bytes after every Block.
- **Trade-offs:** at 256-byte Blocks a CRC-32 costs 32/256 = **0.125 bpb**, and a CRC-16 costs 0.0625 bpb. That is the same order as the per-Block learning cost I measured (Appendix A.2) and larger than many plausible Formula gains. It penalises short Blocks in the DP and biases the length choice, which distorts the per-Block rate measurement that is the point of v1. **Reject.** FLAC can afford it because its frames are about 4096 samples × 2 channels × 2 bytes.

### Candidate E: optional XXH64-low-32 (zstd) or xxh32 (lz4) instead of CRC-32

- **Trade-offs:** faster on big inputs, but MBSC decoding is not speed-bound, and xxHash is **not in the Python stdlib**. The reference decoder and every independent decoder would need a hand-written xxHash. Same 4 bytes as B.

(SHA-256, 32 bytes, as offered by xz, is overkill: without a signature it gives no tamper-proofing beyond CRC's accident detection.)

### Ranked recommendation (Q3)

1. **B, optional whole-file CRC-32**, enabled by default everywhere, added before first public release as a format version bump.
2. **C, mandatory CRC-32.**
3. **A, none.** Acceptable only for internal, pre-release benchmark runs.
4. **E, xxHash.**
5. **D, per-Block CRC.** Reject.

**Justification:** the local check shows the rANS end-state test already catches payload corruption, but by design it cannot see header, parameter or Raw-Block corruption. A 4-byte trailing checksum closes that gap at a cost that is invisible to SM-1, because every arm pays it equally. CRC-32 is chosen over xxHash because the reference decoder must stay stdlib-only and CRC-32 is in `zlib`. Using the reserved bit follows xz's own design for adding checks. Per-Block checks are rejected because they would measurably bias the Block-length experiment.

---

## Q4. Encode-time budget for exhaustive search

### MBSC's work model [ESTIMATE, not measured]

**Count of byte codings.** Per 256-aligned offset the DP evaluates every Candidate at up to 3 lengths, 256 + 1024 + 2048 = 3328 bytes, which is 13 byte codings per input byte per Candidate. Candidates: 4096 Formulas + 256 Constants + Direct + Previous-byte = 4355. That gives about **56,600 byte codings per input byte**.

**Cost per byte coding.** Each byte coding is 8 model decisions plus 8 rANS encode steps, one with a division, plus a ŷ evaluation. At 50 to 200 ns per byte coding, this is **about 3 to 11 ms per input byte per core, or roughly 90 to 350 B/s/core**. Exactness-preserving reuse (below) brings this to an estimated 1 to 5 ms.

**Corpus-level cost.** With 16 cores, a 10 MiB evaluation corpus costs about 3 to 30 core-hours, or 10 minutes to 2 hours of wall-clock time. These are back-of-envelope figures; the SPEC says to set the budget after the first measurement.

### What research compressors spend (verified where cited)

| Compressor (mode) | Encode cost | Source |
|---|---|---|
| cmix v21 | 622,949 ns/B on enwik9 (≈ 1.6 KB/s, ≈ 173 h for 1 GB), 30,950 MB RAM | [LTCB table](https://mattmahoney.net/dc/text.html) (tester's machine; "Speed is approximate and has no effect on ranking") |
| cmix-neif 1.0 (2026) | 175,392 ns/B, 9,750 MB | same |
| paq8px_v206fix1 -12L | 291,916 ns/B | same |
| nncp v3.2 | 241,871 ns/B | same |
| xz 5.2.1 -9e (1 GiB dictionary) | 5,876 ns/B | same |
| zstd 0.6.0 -22 --ultra | 701 ns/B | same |
| Hutter Prize limit | "less than 70'000/T hours" (T = Geekbench5 score; T≈1427 single-core gives ≈ 49 h) with ≤ 10 GB RAM | [Hutter rules](http://prize.hutter1.net/hrules.htm) |
| FLAC `-e` / `-p` | documented only as "exhaustive model search (expensive!)" and "exhaustive search of LP coefficient quantization (expensive!)"; `-8` = `-l 12 -b 4096 -m -r 6 -A "subdivide_tukey(3)"` | [flac manual](https://xiph.org/flac/documentation_tools_flac.html) |
| Zopfli | "2 to 3 orders of magnitude more [CPU] than zlib at maximum quality" for 3–8% smaller output | [Google Developers blog](https://developers.googleblog.com/compress-data-more-densely-with-zopfli/) |
| Lossless audio (OptimFROG, Monkey's Audio extra high/insane, FLAC -8) | reported as CPU time in % of playback length or × realtime; OptimFROG the slowest and best by a small margin; Monkey's `-c4000` within 0.006 points of OptimFROG `--bestnew` in revision 4 | [ktf revision 4 PDF](http://audiograaf.nl/losslesstest/Lossless%20audio%20codec%20comparison%20-%20revision%204.pdf), [revision 6](http://www.audiograaf.nl/losslesstest/Lossless%20audio%20codec%20comparison%20-%20revision%206%20-%20cdda.html) [UNVERIFIED: exact per-codec × realtime values; only plots were accessible] |

Takeaways:

- Full-cap MBSC exhaustive search is plausibly **in cmix's cost class per byte, or slower**.
- Research results are routinely accepted at 10⁵ to 10⁶ ns/B when the claim is about rate, not speed.
- The accepted way to report time is per-byte cost on a named machine, kept *outside* the ranking (LTCB) or normalised by a CPU score (Hutter).

### Candidate A: name a deterministic work denominator, budget wall-clock separately

- **Answer:** the report records a host-independent count: model decisions and rANS steps per input byte, per arm. A separate, non-normative timing log records wall-clock, CPU model, thread count and toolchain. Budget: **≤ 24 h wall-clock for one full benchmark run on one named host.**
- **Trade-offs:** this satisfies the SPEC Non-goal ("no throughput targets until a work model names the denominator") and CAP-12 ("no wall-clock or host-dependent values" in the report). SM-4 stays exact because the timing log is excluded from determinism checks. It mirrors LTCB practice.

### Candidate B: exactness-preserving speed-ups first (no change to selection)

1. **Precompute ŷ once per Formula.** Formulas are positionwise in `n` only (AD-4), so each Formula's ŷ₀…ŷ₂₀₄₇ is identical at every offset. A 4096 × 2048-byte table is 8 MiB. This removes all evaluator cost from the search loop. Exact.
2. **Share the model pass across lengths.** The model resets at Block start and is causal, so the first 256 bytes of a 2048-byte Candidate produce exactly the decisions and probabilities of the 256-byte Candidate at the same offset. One 2048-byte model pass therefore serves all three lengths (2048 instead of 3328 bytes of model work, −38%). Only the backward rANS encode is per length. Exact.
3. **Exact header-cost pruning.** If a Candidate's header and parameter bytes alone are ≥ the incumbent's exact size for that span, skip it. For example a Trigonometric order 16 at `cw = 28` needs about 122 bytes and cannot win a 256-byte Block that Direct codes in 100 bytes. Exact.
4. **Lower-bound pruning on rANS size.** Local check: over 400 random Blocks, `8·exact_bytes − ideal_bits` fell in **[24.02, 31.99] bits** (Appendix A.3). This fits the intuition that the final state holds between 23 and 31 bits. A *proven* bound of the form `exact ≥ ceil((ideal + 23)/8)` would let a running ideal-code-length sum abort a Candidate early with no change in selection. **[Needs a proof and a property test before use.]**
5. **Reciprocal table for rANS division** (as in ryg_rans `RansEncSymbolInit`, [rans_byte.h](https://raw.githubusercontent.com/rygorous/ryg_rans/master/rans_byte.h)), applied to the 4095 possible frequencies. Exact.

- **Trade-offs:** none for fidelity. Item 4 must be proven, or AD-10's "no entropy screen" should state explicitly that admissible bounds are allowed. Items 1 to 3 and 5 are pure engineering.

### Candidate C: cap the corpus, not the Codebook

- **Answer:** fix evaluation size per class (for example 4 to 16 MiB) and tuning size (for example ≤ 4 MiB per class), and choose inputs to fit the 24 h budget.
- **Trade-offs:** preserves the full Codebook, which is the scientific object, at the cost of statistical power. Fewer inputs make "any reduction counts" fragile, so keep ≥ 3 evaluation inputs per class.

### Candidate D: lower the Codebook cap, measured with the cost sidecar

- **Answer:** if A to C still miss the budget, reduce to 1024 per family (≈ 3,331 Candidates, −24%). Report the excess bytes against the full cap on `tuning` inputs using the AD-14 sidecar.
- **Trade-offs:** changes the experiment, though the SPEC explicitly allows it ("the Codebook cap … can be lowered"). The sidecar makes the loss measurable rather than assumed.

### Candidate E: heuristic entropy screen (the deferred item)

- **Answer:** pre-rank Candidates by histogram entropy and exactly code only the top k.
- **Trade-offs:** violates SM-C3 ("selection uses exact size with no entropy screen"). Reject for v1.

### Ranked recommendation (Q4)

1. **A + B together:** a deterministic work denominator in the report, wall-clock in a side log, a ≤ 24 h per full benchmark budget, and exact speed-ups 1, 2, 3 and 5 implemented before any measurement. Item 4 follows only with a proof.
2. **C:** size the corpus to the budget.
3. **D:** a lower Codebook cap, measured with the sidecar.
4. **E:** entropy screen. Reject.

**Justification:** the field's precedent (LTCB, Hutter Prize, Zopfli, FLAC `-e`) accepts very slow encoders when the claim is about rate, so long as time is reported honestly and kept out of the score. The single biggest and cheapest win is a structural property MBSC already has: positionwise Formulas plus per-Block model reset make ŷ tables and the model prefix shareable, which removes most of the redundant work without touching selection. Lowering the cap changes what is being measured, so it should come last and be quantified through the sidecar that AD-14 already requires.

---

## Q5. Independent-decoder check (SM-5): who and when

### What standards bodies and projects do (verified)

- **IETF:**
  - Advancing to Internet Standard requires "at least two independent interoperating implementations with widespread deployment" ([RFC 6410 §2.2](https://www.rfc-editor.org/rfc/rfc6410.html)).
  - RFC 9639 (FLAC) says decoding "has been implemented independently for many different platforms" and points to a public decoder testbench that "can also be used to test conformance" ([RFC 9639](https://www.rfc-editor.org/rfc/rfc9639.html), §11 and [testbench](https://github.com/ietf-wg-cellar/flac-test-files)). The testbench has `subset`, `uncommon` and `faulty` groups, and decoders "should not crash or freeze" on faulty files.
- **zstd:**
  - The repo ships an **educational decoder**, "a self-contained implementation … according to the Zstandard format specification", laid out to match it ([educational_decoder](https://raw.githubusercontent.com/facebook/zstd/dev/doc/educational_decoder/README.md)).
  - It also ships `decodecorpus`, a "Tool to generate valid Zstandard frames, for verifying decoder implementations" ([tests/README](https://raw.githubusercontent.com/facebook/zstd/dev/tests/README.md)).
  - RFC 8878 says "It is highly recommended to fuzz-test … decoder implementations" ([RFC 8878 §8](https://www.rfc-editor.org/rfc/rfc8878.html)).
  - Independent decoders exist, for example ruzstd ("A pure Rust implementation … as defined in RFC8878"), fuzzed with a corpus seeded by `decodecorpus` ([ruzstd](https://raw.githubusercontent.com/KillingSpark/zstd-rs/master/Readme.md)).
- **AV1 (Argon Streams):** a stream generator driven by a "pseudo-code Specification" with a coverage report linking "for every equation … a bitstream which exercises it", including "every transition in the arithmetic coding engine". Expected outputs are verified by MD5 ([AOM verification tool](http://aomedia.org/av1-video-decoder-verification-tool/), [GitLab](https://gitlab.com/AOMediaCodec/argon-streams)).
- **JPEG XL:** ISO/IEC 18181-3:2025 defines conformance as test bitstreams plus reference decoded images ([ISO](https://www.iso.org/standard/87633.html)), with the cases published in [libjxl/conformance](https://github.com/libjxl/conformance). An independent Rust decoder (jxl-oxide, "A spec-conforming JPEG XL decoder") is tested against it ([jxl-oxide](https://raw.githubusercontent.com/tirr-c/jxl-oxide/main/README.md)).
- **Independence is weaker than it looks.** In Knight and Leveson's experiment, 27 versions written independently from one specification failed together on 1,255 tests against about 127 expected under independence ([IEEE TSE 1986](https://doi.org/10.1109/tse.1986.6312924)). Common misreadings of a spec are correlated, and they are even more correlated when the same person writes both implementations.

### Candidate A: the in-repo Python reference decoder by the author (current plan)

- **Answer:** SM-5 is satisfied by `tools/refdec`, "written from the spec and not translated from the Rust" (AD-15).
- **Trade-offs:** cheap, and it runs in CI on both architectures. But it shares the author's mental model: by Knight and Leveson's result, a spec ambiguity the author resolved one way in Rust will very likely be resolved the same way in Python. It checks *consistency*, not *spec completeness*.
- **Fit:** necessary, not sufficient.

### Candidate B: a clean-room decoder from `bitstream.md` and vectors only, by a different person or a fresh sandboxed AI agent

- **Answer:** at spec freeze, give an implementer only `spec/bitstream.md`, `spec/sine_lut.*` and `spec/vectors/`, with no repository, Rust or Python access. Use a third language (for example C or Go), so it is not a translation. The implementer must log every ambiguity or assumption as a spec issue. If it is an AI agent, use a different model family from the one that helped write the kernel, run it in an empty workspace, and record the prompt and model ID.
- **Trade-offs:** tests the *document*, which is what SM-5 and CAP-9 actually claim: "an independent decoder written from the document and vectors". It is cheap and repeatable at every format version bump (AD-15). Risk: a strong model may silently "fix" spec gaps from prior knowledge of ryg_rans or LZMA. Requiring an ambiguity log and comparing it against the rejection vectors mitigates this.

### Candidate C: an external human implementer before release

- **Answer:** recruit a volunteer or paid reviewer to write a decoder from the document.
- **Trade-offs:** closest to the IETF "different code bases" standard. Slow and dependent on availability, so it should not gate the benchmark (CAP-9 already says "never blocking the benchmark").

### Candidate D: valid-stream generator and three-way differential fuzzing in CI

- **Answer:** an encoder-side generator (like `decodecorpus` or Argon) emits random *valid* Bitstreams covering every kind, length code, limit boundary and partial final Block, plus mutations. The Rust kernel, the Python decoder and the clean-room decoder must agree byte-for-byte, or all three must reject. Add coverage-guided fuzzing of the kernel's parse entry points.
- **Trade-offs:** turns SM-5 from a one-off into a regression guard. A coverage table that maps every spec rule (§7 reject set, AD-5 limits) to at least one vector is the Argon idea at MBSC scale. It finds disagreements but cannot by itself prove the spec is complete.

### Candidate E: public conformance suite and call for implementations after release

- **Answer:** publish `spec/vectors/` (CC0, see Q2) as a conformance set in the style of the FLAC testbench, with `valid`, `edge` and `faulty` groups, and invite implementations.
- **Trade-offs:** costs nothing, but evidence arrives only after release.

### Ranked recommendation (Q5)

1. **B (clean-room) + D (generator and differential fuzzing)** as the SM-5 gate at spec freeze, before first public release, re-run at every format version bump. **A** stays as the always-on CI decoder.
2. **C (external human).** Upgrade from B when a willing implementer exists; treat as the gold standard.
3. **E (public suite)** after release.
4. **A alone.** Not sufficient for the SM-5 claim.

**Justification:** SM-5's wording ("written from the document") is a claim about the document's sufficiency. Only an implementer who never saw the code can test that, and Knight and Leveson show that the same author's second implementation does not provide the needed independence. Every major format pairs a document with a generator or conformance set and fuzzing: zstd's `decodecorpus` and educational decoder, FLAC's testbench, AV1's Argon, JPEG XL's Part 3. MBSC can do the same cheaply because its decoder is small. The timing follows CAP-9: required at spec freeze and before public release, never blocking the benchmark.

---

## Q6. Spec-seed constants: confirm PROPOSED, set MEASURE

### 6.1 Magic byte `0x8D` (PROPOSED)

- **Local check, file(1):** in the upstream `file/file` magic sources at commit `584caf71` (2026-10-30), **no rule tests offset 0 for byte `0x8D`**. The only `\x8D` hits are inside longer Apple II disk-image strings that start with `\x01`, plus a partition-type table. The local `file-5.46` reports `data` for `8D 01 …` and `8D 11 …` ([file/file](https://github.com/file/file)).
- **Encoding argument:** `0x8D` is a UTF-8 continuation byte, so no valid UTF-8 text starts with it (the seed's rationale holds). It is a C1 control in ISO-8859-1. It is also the x86 `LEA` opcode, so a raw machine-code blob can start with it; ELF files cannot, since they start with `0x7F`.
- **Comparison with other formats:** gzip uses 2 bytes `1f 8b` ([RFC 1952](https://www.rfc-editor.org/rfc/rfc1952.html)), zstd and lz4 use 4, xz uses 6 with a first byte chosen to exclude `.lzma` and a final `00` to exclude text ([xz spec §2.1.1.1](https://tukaani.org/xz/xz-file-format.txt)), and PNG uses 8 (`89 50 4E 47 0D 0A 1A 0A`) for "early detection of some transmission errors" ([PNG](https://www.w3.org/TR/png-3/)).
- **False identification:** with byte 1 constrained (version 1, mode ∈ {0, 1}, reserved bits 0), only `0x01` and `0x11` are valid. The chance that 2 random bytes look like an MBSC header is (1/256)·(2/256) ≈ 3·10⁻⁵, comparable to gzip's 2-byte magic.
- **Trade-off:** each extra magic byte adds 1 to the N + 7 bound and to every file. That is negligible for SM-1 but changes CAP-8.
- **Recommendation:** **confirm `0x8D`.** A PNG-style multi-byte signature buys transfer-error detection that the Q3 CRC covers better.

### 6.2 Version: 4 bits (PROPOSED)

- AD-15 bumps the version on *any* normative change, so 0.x development (LUT, model, flush, checksum) can plausibly consume more than 16 values.
- **Options:**
  - (a) Keep 4 bits, reserve 0 as invalid (it catches zeroed buffers) and 15 as "extended version follows as LEB128".
  - (b) Use a full version byte and move mode and reserved bits to byte 2. This costs 1 byte, so the bound becomes N + 8.
  - (c) Keep 4 bits as is.
- **Recommendation:** **(a).** Zero cost and no exhaustion risk.

### 6.3 Block header: 5 bits plus 3 padding bits (PROPOSED)

- The cost is 3 bits per Block, at most 0.0117 bpb at 256-byte Blocks. That is small next to the 24 to 32-bit rANS flush overhead (A.3).
- Byte alignment is what keeps Blocks additive and DP-exact (AD-8).
- **Recommendation:** **confirm.**

### 6.4 Coefficient width fields `cw` (PROPOSED/MEASURE)

- Parameters are the dominant fixed cost for rich Formulas: a Trigonometric order 16 at `cw = 28` is about 970 bits.
- FLAC uses a 4-bit coefficient-precision field plus a shift for LPC, and the encoder may search precisions (`-p`) ([RFC 9639 §9.2.6](https://www.rfc-editor.org/rfc/rfc9639.html); [flac manual](https://xiph.org/flac/documentation_tools_flac.html)), which is the same per-Block width lever.
- **Recommendation:** keep the 5-bit `cw − 1` fields. Measure their overhead against fixed widths on `tuning` only, as seed item 5 says. Consider a narrower ω (32 bits fixed is 4 bytes per Trigonometric Block) as a MEASURE item.

### 6.5 Adaptive bit model (MEASURE): structure first, then values

**What the state of the art does (verified):**

| Coder | Probability precision | Adaptation rule | Steady-state rate |
|---|---|---|---|
| LZMA / xz | 11-bit (`RC_BIT_MODEL_TOTAL_BITS 11`), init 1024 | shift `RC_MOVE_BITS 5` on non-negative operands | 1/32 ([range_common.h](https://raw.githubusercontent.com/tukaani-project/xz/master/src/liblzma/rangecoder/range_common.h)) |
| AV1 (dav1d) binary | Q15 CDF, `EC_MIN_PROB 4` | `rate = 4 + (count >> 4)`, count saturates at 32 | 1/16 rising to 1/64 ([dav1d msac.c](https://raw.githubusercontent.com/videolan/dav1d/master/src/msac.c)) |
| H.264/HEVC CABAC | 64 LPS states, p ∈ [0.01875, 0.5], α ≈ 0.95 | finite state machine | ≈ 1/20 ([Marpe et al. 2003](https://iphome.hhi.de/marpe/download/cabac_ieee03.pdf)) |
| VVC CABAC | two estimators at 10 and 14 bits, averaged | per-context shift pairs, trained jointly with initial values | dual rate ([Schwarz et al. 2021](https://doi.org/10.1109/tcsvt.2021.3072202)) |
| paq/lpaq-family direct model | high precision (ZPAQ packs a 22-bit prediction and 10-bit count) | rate `1/(count + DELTA)` with a LIMIT; "On stationary sources, the compressed size is typically larger by 1/LIMIT" | 1/LIMIT ([Mahoney, DCE §4.1.2](https://mattmahoney.net/dc/dce.html)) |

MBSC's seed rule `RECIP[n] = 65536/(n+2)` is the DCE estimator with DELTA = 1, the Laplace estimator. DCE reports that measured English-text sequences fit DELTA ≈ 0.07 to 0.1 and that the KT choice is DELTA = 0.5.

**Local check findings (Appendix A.2), all against the seed exactly as written:**

1. **The upper clamp is never reached, and the ceiling is asymmetric.** After 5,000 zero bits, `p` (P(bit = 0)) stalls at **4070**, not 4080, because `((4096 − p)·2520) >> 16` becomes 0 once 4096 − p ≤ 26. After 5,000 one bits, the signed floor shift `(−p·R) >> 16` still yields −1, so `p` reaches the clamp **16**. Zeros therefore cost 0.00919 bits per decision and ones 0.00565. The common best case, residual δ = 0 coded as `0x00`, is all zero bits, so it is the costlier direction. The behaviour also hinges on floor-shift semantics for negative numbers, a classic cross-language pitfall for SM-5.
2. **The cost floor comes from 12-bit precision.** An all-zero 2048-byte residual Block costs **22.2 bytes** of model code length under the seed (5.7 B at L = 256, 12.7 B at L = 1024).
3. **The fix is cheap.** Keep `p` as a 16-bit state (P(0)·65536), update it with the non-negative LZMA/AV1-style pair (`p += ((65536 − p)·R) >> 16` for a 0, `p −= (p·R) >> 16` for a 1), and code with `freq0 = clamp(p >> 4, 1, 4095)`. The all-zero 2048 Block then drops to **5.3 B** (−17 B per Block), and the model remains transmission-free and causal.
4. **CAP matters, in opposite directions for clean and noisy residuals.** On i.i.d. discretised-Laplace residuals at L = 2048, adaptation overhead above the zero-order entropy (bits per byte, mean of 20 Blocks) was:

   | Laplace scale b | p16, cap 24 | p16, cap 60 | p16, cap 255 |
   |---|---|---|---|
   | 0.7 | 0.087 | 0.061 | 0.059 |
   | 3 | 0.141 | 0.100 | 0.087 |
   | 12 | 0.255 | 0.205 | 0.188 |

   So a larger CAP saves up to 0.05 bpb, consistent with DCE's 1/LIMIT rule. But a larger CAP slows convergence on deterministic residuals: an all-zero 2048 Block costs 13.3 B at cap 255 against 5.3 B at cap 24. This is precisely the trade-off VVC resolves with two averaged rates.
5. **At L = 256 the learning cost dominates.** It is 0.26 to 0.78 bpb for every CAP and DELTA tried, because a fresh 255-node tree learns from 256 symbols. This is larger than the 4-byte flush (0.125 bpb) and is the main reason short Blocks will rarely win on noisy data. The initial `p` (a single 2048) is therefore a meaningful MEASURE lever. A per-depth initial table, tuned on `tuning`, is allowed by AD-7 as long as it is one table for all Candidates.

**Recommendation (structure, a spec change):** 16-bit internal probability, symmetric non-negative update, coder frequency `clamp(p16 >> 4, 1, 4095)`, and drop the [16, 4080] clamp. **Values (MEASURE on `tuning`):** CAP ∈ {24, 32, 60, 128, 255}, DELTA ∈ {0.5, 1}, initial `p` (scalar or per-depth). Optionally, if one CAP cannot serve both clean and noisy residuals, test a VVC-style average of two rates.

### 6.6 Sine LUT size and Q15 scale (PROPOSED)

- **Local check, Appendix A.1:** the seed's exact `sin_lut` with 1024 intervals (1025 entries) at scale 32767 has a **max error of 1.05 LSB and RMS 0.39 LSB** against true sine. Linear interpolation alone contributes only 0.154 LSB; table rounding plus output rounding dominate.
- **Alternatives:** 4096 intervals give max 0.99 LSB (no gain for 4× the data); 256 intervals give 3.18 LSB; scale 16383 changes nothing material.
- **What 1 LSB means:** 1 LSB of Q15 is 3·10⁻⁵ of full scale, about 0.004 byte units at amplitude 127. The LUT is normative data, so this error never affects correctness, only how closely a Formula can track a real sinusoid.
- **Intermediate width:** `(v1 − v0)·f` is below 202·65,536 < 2²⁴, so it is safe in every language.
- **Recommendation:** **confirm 1025 entries at Q15 (32767).**

### 6.7 rANS state size and flush (MEASURE)

- **Confirmation:**
  - The seed's coder is ryg_rans `rans_byte.h` exactly: `RANS_BYTE_L (1u << 23)`, `x_max = ((L >> scale_bits) << 8) * freq`, 4-byte little-endian flush, and 31 "not 32!" bits of state used ([ryg_rans](https://raw.githubusercontent.com/rygorous/ryg_rans/master/rans_byte.h)).
  - Local check: my implementation of §6 reproduces both seed example vectors, `00 00 00 01` and `00 08 00 01`.
  - Measured total excess over the ideal code length, flush included, is **24.0 to 32.0 bits per Block**, mean 27.9 over 400 Blocks (about 3.5 B), independent of Block length.
- **Shorter-flush options:**
  - (a) Keep 4 bytes.
  - (b) **Bloom's minimal flush:** start the encoder at x = 0 and flush only the bytes with bits set, so "This ANS variant will output only 1 byte on very-low-entropy data" ([cbloom, ANS Minimal Flush, 2015](http://cbloomrants.blogspot.com/2015/05/05-11-15-ans-minimal-flush.html)). It saves up to about 3 B per Block (≈ 0.09 bpb at L = 256). But it loses the `x == L` end check, which caught 6,000 of 6,000 payload mutations (Q3), and it needs a variable-length state read. A Giesen comment on that post notes that the rANS approximation is "particularly bad at x=0", so part of the saving is lost.
  - (c) **Put useful bits in the initial state** (same post), which is complex because the data only comes back at the end of decoding.
  - (d) A smaller L with byte renormalisation needs L ≥ 4096 = M, so the flush cannot drop below 3 bytes.
- **Recommendation:** **keep L = 2²³ and the 4-byte flush for v1** (it is the Deferred item "Global rANS stream, minimal flush"). Revisit (b) only if the benchmark's flush share is material on the classes that decide SM-1. Note that (b) would weaken corruption detection unless Q3's CRC is present.
- **Context, zstd FSE (tANS):** tANS final states cost only Accuracy_Log bits each, with Accuracy_Log = low4bits + 5 in the table description ([RFC 8878 §4.1.1](https://www.rfc-editor.org/rfc/rfc8878.html)). This is why tANS formats flush cheaply; they pay instead with static, transmitted tables, which MBSC's adaptive model avoids.

### Ranked recommendation (Q6)

1. **Confirm** magic `0x8D`, the 5-bit Block header, the 1025-entry Q15 LUT, the rANS core (L = 2²³, M = 4096, 4-byte flush) and the `cw` scheme with measurement. **Change** the bit model to 16-bit internal precision with a symmetric non-negative update and `freq = clamp(p >> 4, 1, 4095)`, then tune CAP, DELTA and the initial `p` on `tuning`. **Add** a version escape (0 invalid, 15 extended).
2. Same as 1, but keep the 12-bit model and only fix the asymmetry (non-negative update): smaller change, but it keeps the 22 B per 2048-Block floor on perfect matches.
3. The seed as written, with only CAP, rate and initial `p` tuned.
4. Same as 1 plus minimal flush now. Premature: it trades away the end-state check before the flush share is measured.

**Justification:** the seed's structural choices (magic, header, LUT, rANS) match well-established practice (ryg_rans, gzip/xz magic design, FLAC-style per-Block parameter widths), and my checks found no defect in them. The bit model is the exception. Its 12-bit state with a signed floor-shift update has an unreachable upper clamp and an asymmetric ceiling that taxes exactly the residual (δ = 0) the project hopes to see. Every mature adaptive coder in the table keeps more internal precision than its coder's frequency resolution (AV1 Q15, ZPAQ 22-bit, VVC 10/14-bit) and updates with non-negative operands. Fixing this is a one-time structural change; the remaining constants are then left to the `tuning` measurement as the SPEC intends.

---

## Conflicts with the current SPEC and architecture

1. **AD-7 and spec seed §5.** "Every node holds a 12-bit probability", the signed `target − p` update, and the clamp [16, 4080] should become a 16-bit state, a symmetric update and a coder-side clamp [1, 4095] (Q6.5). Evidence: the stall at 4070, the unreachable clamp, and 22.2 B against 5.3 B per all-zero 2048 Block.
2. **CAP-8, AD-10, AD-11, SM-3 and the "no integrity checksum" assumption.** If Q3's recommendation is adopted before first public release, the bound becomes **N + 11 with checksum** (N + 7 without), and reserved bit 6 of byte 1 gets a meaning.
3. **Spec seed §2 version packing.** Reserve 0 as invalid and 15 as an escape (Q6.2).
4. **AD-10, "v1 has no entropy screen".** The text should allow provably admissible lower-bound pruning that cannot change selection (Q4 B.4), or explicitly forbid it.
5. **CAP-13 and the PCM class definition.** State that a documented, fetch-side 16-to-8-bit transform defines the PCM corpus and is not compressor preprocessing. Require byte-level disjointness of tuning and evaluation, not only distinct file hashes (the enwik8 ⊂ enwik9 trap, Q1).
6. **Corpus fetch constraint (not a conflict, a consequence).** The stdlib-only fetch tool excludes FLAC-packed PCM sources such as LibriSpeech, so FLAC files can serve only as the compressed control.

## Items marked [UNVERIFIED]

- Claims and legal status of Microsoft's continuation US 2022/0109891 A1 and of the EP, CN and KR family members.
- LJ Speech archive size and member order (whether streaming extraction can stop early); Mozilla Data Collective lists 3.00 GB.
- Share of silence padding in Speech Commands clips.
- Whether a chosen Debian package uses `data.tar.xz` (stdlib-extractable) or `data.tar.zst`.
- Kodak's own licence statement for the Kodak PNG suite (only the mirror's statement was seen).
- Exact × realtime encode figures for OptimFROG, Monkey's Audio and FLAC in ktf's revision 6 (only plots were accessible).
- The LJ Speech clip-ID prefix range (LJ001 to LJ050) used to define the tuning/evaluation split.
- The rANS lower bound `exact ≥ ceil((ideal + 23)/8)`: empirically consistent over 400 Blocks, not proven.
- All MBSC encode-time numbers in Q4 are estimates from a work model, not measurements.

## References

**Corpora and datasets**
- Silesia corpus: <https://sun.aei.polsl.pl/~sdeor/index.php?page=silesia>
- Canterbury and Calgary corpora: <https://corpus.canterbury.ac.nz/descriptions/>
- Mahoney, About the test data (enwik8/9 MD5 and SHA-1): <https://mattmahoney.net/dc/textdata.html>
- Mahoney, Large Text Compression Benchmark (updated 2026-10-14): <https://mattmahoney.net/dc/text.html>
- Hutter Prize rules: <http://prize.hutter1.net/hrules.htm>
- Delétang et al., "Language Modeling Is Compression": <https://arxiv.org/abs/2309.10668>
- LJ Speech: <https://keithito.com/LJ-Speech-Dataset/>; MDC listing: <https://mozilladatacollective.com/datasets/cmonjhxee01ako007kohpbg34>
- LibriSpeech (OpenSLR 12): <https://www.openslr.org/12/>
- Warden, Speech Commands: <https://arxiv.org/abs/1804.03209>; TFDS: <https://www.tensorflow.org/datasets/catalog/speech_commands>
- FLAC decoder testbench (CC0): <https://github.com/ietf-wg-cellar/flac-test-files>
- EBU test-sequence terms: <https://tech.ebu.ch/files/live/sites/tech/files/shared/testmaterial/use%20of%20EBU%20AUDIO%20test%20sequences.pdf>
- Kodak PNG mirror: <https://r0k.us/graphics/kodak/>
- pdx-cs-sound compression-bench: <https://github.com/pdx-cs-sound/compression-bench>
- ktf lossless audio comparison r6 (CDDA): <http://www.audiograaf.nl/losslesstest/Lossless%20audio%20codec%20comparison%20-%20revision%206%20-%20cdda.html>; r4 PDF: <http://audiograaf.nl/losslesstest/Lossless%20audio%20codec%20comparison%20-%20revision%204.pdf>
- snapshot.debian.org: <https://snapshot.debian.org/>
- AOM CTC test sets: <https://media.xiph.org/video/aomctc/>

**Licences, contribution and patents**
- zstd LICENSE, COPYING, README, CONTRIBUTING: <https://raw.githubusercontent.com/facebook/zstd/dev/LICENSE>, <https://raw.githubusercontent.com/facebook/zstd/dev/COPYING>, <https://github.com/facebook/zstd>, <https://raw.githubusercontent.com/facebook/zstd/dev/CONTRIBUTING.md>
- zstd format document: <https://raw.githubusercontent.com/facebook/zstd/dev/doc/zstd_compression_format.md>
- xz COPYING: <https://raw.githubusercontent.com/tukaani-project/xz/master/COPYING>
- brotli LICENSE and CONTRIBUTING: <https://raw.githubusercontent.com/google/brotli/master/LICENSE>, <https://raw.githubusercontent.com/google/brotli/master/CONTRIBUTING.md>
- lz4 LICENSE: <https://raw.githubusercontent.com/lz4/lz4/dev/LICENSE>
- FLAC README and COPYING.Xiph: <https://raw.githubusercontent.com/xiph/flac/master/README.md>, <https://raw.githubusercontent.com/xiph/flac/master/COPYING.Xiph>
- Monkey's Audio licence: <https://www.monkeysaudio.com/license.html>
- rav1e LICENSE and PATENTS: <https://raw.githubusercontent.com/xiph/rav1e/master/LICENSE>, <https://github.com/xiph/rav1e/blob/master/PATENTS>
- ruzstd: <https://github.com/KillingSpark/zstd-rs>; jxl-oxide: <https://github.com/tirr-c/jxl-oxide>
- Rust API Guidelines, C-PERMISSIVE: <https://rust-lang.github.io/api-guidelines/necessities.html>
- Apache License 2.0: <https://www.apache.org/licenses/LICENSE-2.0>
- Developer Certificate of Origin 1.1: <https://developercertificate.org/>
- OWFa 1.0: <https://www.openwebfoundation.org/the-agreements/the-owf-1-0-agreements-granted-claims/owfa-1-0>
- US 11,234,023 B2: <https://patents.google.com/patent/US11234023B2/en>; US 2022/0109891 A1: <https://patents.google.com/patent/US20220109891A1/en>
- The Register (2021): <https://www.theregister.com/software/2021/03/13/third-times-a-harm-microsoft-tries-to-get-twice-rejected-encoding-patent-past-skeptical-examiners/408210>; (2022): <https://www.theregister.com/software/2022/02/17/alarm-raised-after-microsoft-wins-data-encoding-patent/394460>
- Duda, Asymmetric Numeral Systems: <https://arxiv.org/abs/1311.2540>

**Formats, checksums, magic numbers**
- RFC 8878 (Zstandard): <https://www.rfc-editor.org/rfc/rfc8878.html>
- RFC 9639 (FLAC): <https://www.rfc-editor.org/rfc/rfc9639.html>
- RFC 7932 (Brotli): <https://www.rfc-editor.org/rfc/rfc7932.html>
- RFC 1952 (gzip): <https://www.rfc-editor.org/rfc/rfc1952.html>
- xz file format: <https://tukaani.org/xz/xz-file-format.txt>
- LZ4 frame format: <https://raw.githubusercontent.com/lz4/lz4/dev/doc/lz4_Frame_format.md>
- PNG (Third Edition): <https://www.w3.org/TR/png-3/>
- Koopman, "32-Bit Cyclic Redundancy Codes for Internet Applications" (DSN 2002): <https://users.ece.cmu.edu/%7Ekoopman/networks/dsn02/dsn02_koopman.pdf>
- file(1) magic database: <https://github.com/file/file>

**Encoder cost**
- FLAC tool manual: <https://xiph.org/flac/documentation_tools_flac.html>
- Zopfli announcement: <https://developers.googleblog.com/compress-data-more-densely-with-zopfli/>; repository: <https://github.com/google/zopfli>

**Conformance and independent implementations**
- RFC 6410: <https://www.rfc-editor.org/rfc/rfc6410.html>
- zstd educational decoder: <https://raw.githubusercontent.com/facebook/zstd/dev/doc/educational_decoder/README.md>; tests and decodecorpus: <https://raw.githubusercontent.com/facebook/zstd/dev/tests/README.md>
- AV1 Argon Streams: <http://aomedia.org/av1-video-decoder-verification-tool/>, <https://gitlab.com/AOMediaCodec/argon-streams>
- ISO/IEC 18181-3:2025: <https://www.iso.org/standard/87633.html>; libjxl conformance: <https://github.com/libjxl/conformance>
- Knight and Leveson (1986): <https://doi.org/10.1109/tse.1986.6312924>

**Probability models and entropy coders**
- xz `range_common.h`: <https://raw.githubusercontent.com/tukaani-project/xz/master/src/liblzma/rangecoder/range_common.h>
- dav1d `msac.c`: <https://raw.githubusercontent.com/videolan/dav1d/master/src/msac.c>
- Marpe, Schwarz, Wiegand, CABAC (2003): <https://iphome.hhi.de/marpe/download/cabac_ieee03.pdf>
- Schwarz et al., "Quantization and Entropy Coding in the VVC Standard" (2021): <https://doi.org/10.1109/tcsvt.2021.3072202>
- Mahoney, *Data Compression Explained*: <https://mattmahoney.net/dc/dce.html>
- Giesen, ryg_rans `rans_byte.h`: <https://raw.githubusercontent.com/rygorous/ryg_rans/master/rans_byte.h>
- Bloom, "ANS Minimal Flush" (2015): <http://cbloomrants.blogspot.com/2015/05/05-11-15-ans-minimal-flush.html>

---

## Appendix A. Local numeric checks (method)

All checks were run on 2026-10-31 with Python 3.14.4 in a scratch directory outside the repository. They implement the spec seed's definitions literally. Seeds are fixed, so the numbers are reproducible with the code below.

**A.1 Sine LUT.** `LUT[i] = round(S·sin(2πi/K))` for i = 0…K, K ∈ {256, 1024, 4096}, S ∈ {32767, 16383}. `sin_lut(p)` follows §4.2 with `i = p >> (32 − log2 K)`, a 16-bit fraction `f`, and `v0 + (((v1 − v0)·f + 2¹⁵) >> 16)`. The error is measured against `S·sin(2πp/2³²)` over 400,000 random `p` plus every knot and every mid-interval point.

**A.2 Bit model.** The seed model is 255 nodes with `p0 = 2048`, `RECIP[n] = 65536 // (n + 2)` for n ≤ CAP, the update `p += ((target − p)·RECIP[n]) >> 16` with Python's floor shift (which matches the seed's "arithmetic shift"), and the clamp [16, 4080]. The variants are:

- **symmetric:** `p += ((4096 − p)·R) >> 16` for a 0, `p −= (p·R) >> 16` for a 1;
- **p16:** the same symmetric update on a 16-bit state, with coder frequency `clamp(p16 >> 4, 1, 4095)`;
- **cap / delta:** `RECIP[n] = 65536 // (n + 1 + DELTA)` for other CAP and DELTA values.

The code length is the sum of `−log2(freq/4096)` over decisions, MSB-first through the bit tree. Residuals are discretised Laplace with scale b (inverse-CDF, rounded, clipped to [−128, 127], stored mod 256). The overhead reported is (adaptive code length − n·H₀)/L, averaged over 20 Blocks.

**A.3 rANS.** §6 is implemented exactly (encode backwards from x = L, emit renormalisation bytes, lay out `LE32(x_final)` followed by the reversed bytes). Exact size is compared with the ideal code length on 400 random Blocks (L ∈ {256, 1024, 2048}, b ∈ {0.3 … 40}).

**A.4 Corruption detection.** For each trial: encode one 256-byte Laplace Block (b ∈ {0.7, 3}), XOR one random byte with a random non-zero value, decode per §6 with the model of §5, and accept only if decoding finishes with x = L and all bytes consumed. 3,000 trials per b.

Core of the seed model and coder as used:

```python
RECIP = [65536 // (n + 2) for n in range(CAP + 1)]           # seed: CAP = 24
def update(p, n, bit):                                        # seed §5, as written
    target = 4096 if bit == 0 else 0
    p = p + (((target - p) * RECIP[n]) >> 16)                 # floor shift
    return min(max(p, 16), 4080), min(n + 1, CAP)

L = 1 << 23
def rans_encode(decisions):                                   # seed §6
    x, out = L, []
    for bit, p in reversed(decisions):
        freq, start = (p, 0) if bit == 0 else (4096 - p, p)
        x_max = ((L >> 12) << 8) * freq
        while x >= x_max:
            out.append(x & 0xFF); x >>= 8
        x = ((x // freq) << 12) + (x % freq) + start
    return x.to_bytes(4, "little") + bytes(reversed(out))
```

**A.5 file(1).** Shallow clone of `github.com/file/file` (HEAD `584caf71c7f9b5a581d888636bdbf80c11bfd087`, 2026-10-30), then `rg` over `magic/Magdir` for offset-0 rules matching `0x8d` or `\x8d` as a first byte (none), plus `file-5.46` on two probe files starting `8D 01` and `8D 11` (both reported as `data`).
