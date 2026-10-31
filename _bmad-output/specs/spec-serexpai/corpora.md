# Corpora — MBSC v1

Inputs for `corpora/manifest.toml`. Evidence and rejected alternatives: `../../planning-artifacts/research/open-questions-research-2026-10-31.md` (Q1).

## Rules

- Nothing is vendored. The fetch tool (Python standard library only) downloads, verifies, and applies any corpus-side transform.
- The standard library has `wave`, `tarfile` (gz, bz2, xz) and `zipfile`, but no FLAC or zstd decoder and no `audioop`. FLAC- or zstd-packed sources cannot be Real inputs.
- Archives are downloaded in full and verified before extraction; only manifest-listed members are extracted. Where a source publishes only MD5 (Silesia), the fetch tool checks the published MD5 on first fetch and the manifest pins SHA-256.
- Tuning and evaluation inputs share no contiguous run of ≥ K bytes (K = 1 KiB, PROPOSED), checked by tool. Distinct hashes are not enough: enwik8 is the first 10⁸ bytes of enwik9.
- Overlap policy: evaluation inputs are fixed. A tuning input that fails the check is dropped and replaced by another Calgary or Canterbury input that passes. No stripping, no moving evaluation slices, no allowlist.
- Text and executable evaluation inputs are equal-sized middle slices, so each carries equal weight in the class sum. PCM evaluation inputs are whole clips, sampled not sliced; their weight follows clip length. At least 3 evaluation inputs per Real class, 4 to 16 MiB per class.
- Evaluation inputs within a class are correlated (one PCM speaker; the same packages built for two architectures); the report states this beside per-input wins.
- Licence is recorded per input; "none stated" is recorded as such.

## Inputs

| Class | Role | Inputs | Source, licence |
| --- | --- | --- | --- |
| Text | tuning | Canterbury `alice29.txt`, `lcet10.txt`, `plrabn12.txt`, `cp.html` (HTML), `xargs.1` (man-page markup); Calgary `book1`, `book2`, `paper1`, `paper2` | [Canterbury corpus](https://corpus.canterbury.ac.nz/descriptions/); none stated |
| Text | evaluation | 3 MiB middle slices of Silesia `dickens` (English prose), `webster` (HTML dictionary), `xml` (11 XML files), `reymont` (Polish text, uncompressed PDF) | [Silesia](https://sun.aei.polsl.pl/~sdeor/index.php?page=silesia); none stated |
| PCM (8-bit, derived) | tuning | about 30 LJ Speech clips (about 4 MiB), seeded sample from prefixes LJ001–LJ025, one input per clip | [LJ Speech](https://keithito.com/LJ-Speech-Dataset/) 1.1; public domain |
| PCM (8-bit, derived) | evaluation | about 60 LJ Speech clips (about 8.5 MiB), seeded sample from prefixes LJ026–LJ050, one input per clip | same |
| Executable | tuning | Calgary `obj1`, `obj2`; Canterbury `sum`; one modern pinned Debian 13 package binary for amd64 and arm64 (not `busybox-static` or `ripgrep`; dynamically linked, so it shares no statically linked libc code with `busybox-static`), passing the overlap check | Canterbury site, none stated; [snapshot.debian.org](https://snapshot.debian.org/), package licence |
| Executable | evaluation | equal middle slices of Silesia `ooffice` (Win32 DLL) and of Debian 13 (trixie) `busybox-static` (static C) and `ripgrep` (Rust; Debian likely links it dynamically against libc, checked when pinned), each for amd64 and arm64: 5 inputs | Silesia; [snapshot.debian.org](https://snapshot.debian.org/) pinned by timestamp, package licences |
| Already-compressed control | evaluation | FLAC test-bench `subset` files; `enwik8.zip` (36,445,475 B) | [flac-test-files](https://github.com/ietf-wg-cellar/flac-test-files), CC0; [Mahoney](https://mattmahoney.net/dc/textdata.html) |

## Selection rules

- **LJ Speech sample:** clips are ordered by `SHA-256(seed ‖ clip ID)` ascending, with the seed string recorded in the manifest, and taken in that order until the role's byte budget is reached. Mean clip length 6.57 s, about 145 KB after conversion.
- **Executable slices:** slice length is the smallest of the five evaluation files, rounded down to a multiple of 256 bytes, recorded when the packages are pinned. If 5 × that length is under 4 MiB, an input is added or replaced until the class reaches 4 MiB. Each `.deb` must carry `data.tar.xz`, checked when pinned.
- **Middle slice:** `offset = floor((size − length) / 2)`.

## Corpus-side transform: `pcm16le-to-u8` v1

- Input: mono 16-bit signed little-endian WAV (LJ Speech: 22,050 Hz).
- Output: one byte per sample, the high byte XOR `0x80` (unsigned 8-bit as in 8-bit WAV), raw sample bytes without a WAV header.
- The manifest records source archive hash, member name and derived file hash.
- The raw 16-bit sample bytes of the same clips are also benchmarked and reported as non-counted rows, so the derivation cannot hide a negative 16-bit result.
- Caveats on record: LJ Speech was sourced from 128 kbps MP3 (band-limited, may flatter smooth predictors); one speaker, so the split is content-disjoint but not speaker-disjoint.

## Corpus-side transform: `slice` v1

- Output: bytes `[offset, offset + length)` of one source file, unchanged.
- The manifest records the transform name and version, source hash, offset, length and derived hash.

## Rejected

- LibriSpeech (FLAC inside tar.gz; the fetch tool cannot decode it).
- EBU SQAM (terms forbid redistribution and publication).
- ktf CDDA set (commercial albums, no licence).
- Locally generated `.xz`/`.zst` controls (tool-version dependent).
- Seeded random bytes as the control (already synthetic; not "already compressed").
- Gutenberg-boilerplate stripping (replaced by the overlap policy above).

## Unresolved

- Confirm the LJ Speech prefix range LJ001–LJ050 against `metadata.csv` and fix the seed string.
- K = 1 KiB for the overlap checker.
- Which Debian package supplies the modern tuning executable.
