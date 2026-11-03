---
baseline_commit: d02eb72fa551edf169a57ed89d01b2113439d9e3
---

# Story 1.12: Fetch the corpus manifest

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want verified, reproducible inputs, transforms, and overlap evidence,
so that a later benchmark can trust its bytes without vendoring them.

## Acceptance Criteria

1. **S1.12-AC01.** `tools/fetch/` is Python standard library only and refuses a cache inside the repository. `corpora/manifest.toml` is TOML 1.0. Every input row records source, member or direct-file identity, licence or `licence_state = "unresolved"`, source and derived hashes or an empty hash that the gate calls unresolved, class, role, transform name and version, and parameters. `python3 tools/fetch/test_fetch.py` uses fixture files only. It does not call `urlopen` and does not open a socket. `ci.sh` stays cargo and rustup only.

2. **S1.12-AC02.** Archive SHA-256 is checked before any member is read out. Extraction accepts only the named regular member and rejects `..`, absolute names, symlinks, hard links, duplicate names, encrypted zips, non-regular zip modes, truncated archives, corrupt archives, members over 64 MiB, and formats that would need a decoder. A `.deb` is an `ar` archive whose `data.tar.xz` member is then read with those same rules. `data.tar.zst` is rejected. `./usr/bin/...` is a legal member name. A direct file (the FLAC subset bytes, `enwik8.zip`) is hashed whole and never decoded. When `member_md5` is set, that MD5 is checked. `pin` prints archive and member SHA-256 and does not write the manifest. Normal fetch refuses a row whose `archive_sha256` is empty.

3. **S1.12-AC03.** `pcm16le-to-u8` v1 accepts only mono, 16-bit, little-endian, 22,050 Hz, uncompressed WAV whose `nframes * 2` equals the data length. Output is the high byte XOR `0x80` and has no WAV header. The same frames are also kept as raw little-endian sample bytes for a non-counted row. Any other WAV shape raises `UnsupportedWav`. `slice` v1 uses middle offset `floor((size - length) / 2)` and records source SHA-256, offset, length, and derived SHA-256.

4. **S1.12-AC04.** The committed manifest has the roster below. Text evaluation rows each declare length 3145728. PCM rows are whole clips, never `slice`. The five executable evaluation rows are `silesia-ooffice`, `debian-busybox-static-amd64`, `debian-busybox-static-arm64`, `debian-ripgrep-amd64`, and `debian-ripgrep-arm64`. Their shared slice length stays unresolved. PCM selection orders `SHA-256(utf-8(seed) || utf-8(clip id))` ascending and stops at the first clip that would exceed the derived-byte budget. The production seed and both PCM budgets are empty. Caveats are on the rows: one PCM speaker, 128 kbps MP3 origin, and the same Debian packages built for two architectures. `counts_toward_sm1` is a static flag, not a function of the cache.

5. **S1.12-AC05.** The overlap check compares every tuning row with every evaluation row, across classes. It finds a shared contiguous run of at least K even when the run's phase is not aligned and even when the two file hashes differ. A brute-force oracle on small inputs agrees. A run of K − 1 is clean. Evaluation rows are immutable. The only approved replacement rule is the text rule from `corpora.md`. PCM and Debian-executable replacement rules stay unresolved. A hit reports both offsets, the run length, and whether the run is one repeated byte. Repeated bytes are not allowlisted.

6. **S1.12-AC06.** K is 1024 with status `proposed`. That status is exploratory only. LJ metadata, LJ prefixes, the LJ seed, both PCM budgets, the Debian tuning package, and the two unapproved replacement rules stay unresolved. `corpora/gate-report.json` is the canonical `gate()` document: `scope` is `corpus`, `corpus_gate_ready` is false, and it names every unresolved field. It does not claim full G2. Regenerating the report matches the committed bytes.

7. **S1.12-AC07.** `sxp-bench` generates three 4096-byte signals by calling the kernel evaluators twice over `n` in `0..2048`. The parameter tuple is the seed and is copied into the manifest. The second half equals the first half. SHA-256 literals in the Rust test equal `derived_sha256` in the manifest. Class is `synthetic`, role is `synthetic`, and `counts_toward_sm1` is false. `./ci.sh` runs this test on the host. `./ci.sh aarch64-unknown-linux-gnu` runs it under qemu. Python tests are host-only and are not part of `ci.sh`.

## Tasks / Subtasks

- [x] Fetch library and CLI (AC: 1, 2, 3)
  - [x] `tools/fetch/corpus.py` and `tools/fetch/fetch.py`. Stdlib imports only. No `requirements.txt`, no `audioop`, no `extractall`
  - [x] Cache, pin, extract, deb/`ar`, direct-file, both transforms, typed error codes
  - [x] `tools/fetch/test_fetch.py` covers the fixture matrix below. `SEREXPAI_CACHE` in tests is an absolute temp dir
- [x] Manifest, selection, overlap, gate (AC: 4, 5, 6)
  - [x] Commit `corpora/manifest.toml` and `corpora/gate-report.json`
  - [x] Selection, inventory checks, overlap, and `gate()` behave as specified
  - [x] The test rewrites nothing. It compares `gate()` to the committed JSON
- [x] Synthetic signals (AC: 7)
  - [x] `crates/sxp-bench/src/synthetic.rs` and a unit test that reads the manifest
  - [x] Same deny/forbid attributes as `sxp-encoder`. No new crate dependency
- [x] Acceptance commands (AC: 1, 7)
  - [x] `python3 tools/fetch/test_fetch.py` exits 0 on Python >= 3.14.4
  - [x] `./ci.sh` exits 0. Do not add a Python invocation to `ci.sh`

### Review Findings

- [x] [Review][Patch] Hash the cache temp before replace, and unlink it on failure [tools/fetch/corpus.py:156]
- [x] [Review][Patch] Cache names must be one path component [tools/fetch/corpus.py:128]
- [x] [Review][Patch] Stream the cache hash with a byte cap [tools/fetch/corpus.py:102]
- [x] [Review][Patch] `read_bounded` requires a regular file and one `read(limit + 1)` [tools/fetch/corpus.py:92]
- [x] [Review][Patch] `ar` rejects an oversized member before copying it [tools/fetch/corpus.py:233]
- [x] [Review][Patch] Cap decompressed tar at `ARCHIVE_MAX`, then scan ustar sizes [tools/fetch/corpus.py:284]
- [x] [Review][Patch] Zip allows only stored and deflated members [tools/fetch/corpus.py:349]
- [x] [Review][Patch] Zip open and inflate errors map to `TruncatedArchive` or `CorruptArchive` [tools/fetch/corpus.py:339]
- [x] [Review][Patch] Integers are real ints. Present-but-malformed gate fields are violations; absent fields stay unresolved [tools/fetch/corpus.py:651]
- [x] [Review][Patch] Metadata, overlap, and gate failures become `BadManifest` [tools/fetch/fetch.py:86]
- [x] [Review][Patch] A missing overlap file is `BadManifest` [tools/fetch/corpus.py:617]
- [x] [Review][Patch] Any omitted executable eval length leaves `slice_length` unresolved [tools/fetch/corpus.py:803]
- [x] [Review][Patch] Offset, flac members, licence, schema, budget, and prefix overlap follow the gate rules [tools/fetch/corpus.py:686]
- [x] [Review][Patch] `counts_toward_sm1` must be a real bool [tools/fetch/corpus.py:754]
- [x] [Review][Patch] `transform_version` is int 1 and `source` is a string. Recorded licence is a non-empty string; unresolved licence is absent [tools/fetch/corpus.py:766]
- [x] [Review][Patch] Empty `member` is unresolved on every archive row [tools/fetch/corpus.py:787]
- [x] [Review][Patch] WAV compares the data-chunk size and maps `OSError` to `UnsupportedWav` [tools/fetch/corpus.py:472]
- [x] [Review][Patch] PCM CLI refuses the same resolved wav, out, and raw paths, and deletes a partial out [tools/fetch/fetch.py:44]
- [x] [Review][Patch] Slice and overlap CLI read with a cap. Slice prints the four fields [tools/fetch/fetch.py:62]
- [x] [Review][Patch] Temp names include pid and a counter [tools/fetch/corpus.py:122]
- [x] [Review][Patch] Rust test embeds the three SHA-256 literals and checks the parameter tuples [crates/sxp-bench/tests/synthetic.rs:5]
- [x] [Review][Patch] Tests cover raw-16 `counts_toward_sm1`, unequal executable lengths, and a `..` cache name [tools/fetch/test_fetch.py]
- [x] [Review][Defer] Overlap can do quadratic work on repeated runs shorter than K [tools/fetch/corpus.py:545] — deferred, pre-existing
- [x] [Review][Defer] The corpus gate can later read ready with no overlap evidence [tools/fetch/corpus.py:840] — deferred, pre-existing
- [x] [Review][Decision] Zip Unix mode — Accept every mode value, including 0. A symlink stays `UnsafeLink`. A directory stays `NotRegular`.
- [x] [Review][Patch] A negative ustar size is `CorruptArchive` instead of a stuck scan [tools/fetch/corpus.py:276]
- [x] [Review][Patch] A short member is `TruncatedArchive`. A declared size above 64 MiB is `TooLarge` before the short-buffer check [tools/fetch/corpus.py:233]
- [x] [Review][Patch] A truncated compressed tar and a zip `EOFError` are `TruncatedArchive` [tools/fetch/corpus.py:298]
- [x] [Review][Patch] Non-string pins, ids, kinds, and the Debian package are violations. Empty strings stay unresolved [tools/fetch/corpus.py:535]
- [x] [Review][Patch] Prefix overlap uses the same prefix rule as PCM selection. A malformed prefix list is a violation [tools/fetch/corpus.py:754]
- [x] [Review][Patch] The Rust test checks coefficient and state arrays [crates/sxp-bench/tests/synthetic.rs:55]
- [x] [Review][Patch] Overlap rows without a string id are `BadManifest` [tools/fetch/corpus.py:605]
- [x] [Review][Patch] A failed PCM raw write removes only the file this run created [tools/fetch/fetch.py:44]
- [x] [Review][Patch] Extract and pin map filesystem errors to `BadManifest` [tools/fetch/fetch.py:18]
- [x] [Review][Patch] `counts_toward_sm1` is checked on every row [tools/fetch/corpus.py:839]
- [x] [Review][Patch] `overlap_pairs` rejects an odd or tiny K before it looks at the rows [tools/fetch/corpus.py:644]

## Dev Notes

### What this story builds

A stdlib fetch tool, a committed manifest of the `corpora.md` roster, a corpus-scoped gate report that stays not ready, an overlap checker, and three kernel-generated synthetic signals.

Out of scope: downloading the real corpora, filling published MD5 or SHA-256 from memory, confirming K, checking `metadata.csv`, naming the Debian tuning package, inventing a PCM or executable replacement rule, the `benchmark` command, Gain, sidecars, verdicts, `tools/refdec`, edits to `bitstream-spec-seed.md`, `README.md`, `CHANGELOG.md`, or the workspace version. Do not put Python in `ci.sh`. The workspace rule is that `ci.sh` calls cargo and rustup only. Story 1.13 owns the report. D-09 stays open, so this story must not set a field named `g2_ready`.

### Cache and CLI

Resolved cache path:

1. `SEREXPAI_CACHE` if set. It must be absolute.
2. Else `$XDG_CACHE_HOME/serexpai` when `XDG_CACHE_HOME` is absolute.
3. Else `$HOME/.cache/serexpai`.

Refuse with `CacheInsideRepo` when the resolved real path is the repo root or a path under it. Compare real paths, not spellings. Refuse a relative `SEREXPAI_CACHE` with `CacheNotAbsolute`. Tests set `SEREXPAI_CACHE` to an absolute temp directory and never use the default.

Writes go to a temp file in the cache, then `os.replace` onto the cache name. A cache hit recomputes SHA-256 and raises `HashMismatch` on disagreement.

```text
python3 tools/fetch/fetch.py extract --archive FILE --kind KIND --member NAME --sha256 HEX [--md5 HEX] --out FILE
python3 tools/fetch/fetch.py pin --archive FILE --kind KIND --member NAME [--md5 HEX]
python3 tools/fetch/fetch.py transform pcm16le-to-u8 --wav FILE --out FILE --raw-out FILE
python3 tools/fetch/fetch.py transform slice --input FILE --length N --out FILE
python3 tools/fetch/fetch.py select-pcm --metadata FILE --seed S --budget N --prefix P
python3 tools/fetch/fetch.py overlap --k N A.bin B.bin
python3 tools/fetch/fetch.py gate --manifest corpora/manifest.toml
```

`pin` prints two lines, `archive_sha256=<64 lowercase hex>` and `member_sha256=<64 lowercase hex>`, and no other file. It does not update the manifest. `gate` prints the canonical JSON to stdout. Exit 0 on success. Exit 1 on a typed error, with the code as the first stderr token.

### Typed errors

`CorpusError.code` is one of: `HashMismatch`, `Md5Mismatch`, `PathTraversal`, `UnsafeLink`, `NotRegular`, `DuplicateMember`, `CorruptArchive`, `TruncatedArchive`, `UnsupportedFormat`, `TooLarge`, `Encrypted`, `UnsupportedWav`, `SliceOutOfRange`, `CacheInsideRepo`, `CacheNotAbsolute`, `UnresolvedHash`, `BadManifest`.

### Archives

`MEMBER_MAX` is 64 MiB. `ARCHIVE_MAX` is 4 GiB. Stop reading an archive after `ARCHIVE_MAX + 1` bytes and raise `TooLarge`. A declared member size above `MEMBER_MAX` is `TooLarge` before the bytes are copied. Read at most the declared size. More bytes than declared is `CorruptArchive`.

Kinds:

| Kind | Behavior |
| --- | --- |
| `tar`, `tar.gz`, `tar.bz2`, `tar.xz` | `tarfile` of that mode. Never `extractall` |
| `zip` | `zipfile`. Reject flag bit 0 (encrypted). Any mode value is accepted, including 0. A symlink mode is `UnsafeLink`. A directory is `NotRegular` |
| `deb` | `ar` reader, then the `data.tar.xz` member as `tar.xz` |
| `direct` | The file is the input. `member` is `""`. No decoder |

Reject kind `flac`, `zst`, `zstd`, `7z`, `rar`, and a gzip that is not a tar, with `UnsupportedFormat`. A `.flac` direct-file row is kind `direct`, not kind `flac`. The bytes stay compressed. That is the already-compressed control.

Before opening any member, SHA-256 the whole archive. Mismatch is `HashMismatch` and the output path is not created. Empty expected SHA-256 on this path is `UnresolvedHash`.

`ar` layout: magic `!<arch>\n`, then 60-byte headers (name 16, mtime 12, uid 6, gid 6, mode 8, size 10, end `` `\n ``). Size is decimal. Data follows, padded to an even offset. Require a member whose stripped name is `data.tar.xz`. `data.tar.zst` is `UnsupportedFormat`. Duplicate names are `DuplicateMember`.

Member names: reject an empty name, a name containing a NUL, an absolute name, or any `..` segment after splitting on `/`. A leading `./` is legal. Do not resolve the name against the filesystem.

Accept the member only when it is a regular file. Symlink and hard link are `UnsafeLink`. A directory is `NotRegular`. Then check `member_md5` if the argument is present. MD5 is of the member bytes, which is the Silesia check. Do not invent published MD5 values. Leave them empty in the committed manifest.

Normal fetch of a manifest row with empty `archive_sha256` raises `UnresolvedHash` and does not open the source. `pin` is the only path that runs without a pinned SHA-256. `pin` still checks MD5 when one is supplied.

### Transforms

`pcm16le-to-u8` v1, via `wave`:

- Require `nchannels == 1`, `sampwidth == 2`, `framerate == 22050`, `comptype == "NONE"`.
- Require `nframes * 2` equal to the data length. Otherwise `UnsupportedWav`.
- For each little-endian sample, emit `(sample >> 8) & 0xFF ^ 0x80`.
- Fixtures: −32768 → `0x00`, −1 → `0x7F`, 0 → `0x80`, 32767 → `0xFF`.
- `--raw-out` is those sample bytes in file order, with no header. A stereo, 8-bit, 44,100 Hz, or compressed WAV is `UnsupportedWav` and writes neither file.

`slice` v1:

- `length` in `1..=size`, else `SliceOutOfRange`.
- Middle offset is `(size - length) // 2`.
- Derived bytes are `source[offset:offset + length]`.
- The result object records source SHA-256, offset, length, and derived SHA-256.

Text evaluation rows in the manifest use this length: 3145728. Their offsets stay empty until the source sizes are known. Do not compute an offset from a size you do not have.

### PCM selection

Input is a CSV with header `id,bytes`. `bytes` is the derived u8 length.

1. Keep rows whose id starts with one of the given prefixes. Prefix `LJ001` matches `LJ001-0001` and does not match `LJ0010-0001`. The boundary is `-` or end of string.
2. Sort by `SHA-256(seed.encode() + id.encode())` ascending, then by id ascending.
3. Walk that order. If `sum + bytes > budget`, stop. Do not skip a clip that does not fit.
4. Return the ids taken and the sum.

The production manifest records prefixes `LJ001` through `LJ025` for tuning and `LJ026` through `LJ050` for evaluation, `seed = ""`, and `byte_budget` absent. Empty seed means the gate field `lj.seed` is unresolved. Do not invent clip ids or a budget. "About 4 MiB" and "about 8.5 MiB" are not numbers this story may freeze. The fixture test uses a tiny CSV, seed `fixture-seed`, and an explicit budget.

A PCM row whose transform is `slice` is a violation. PCM evaluation inputs are whole clips.

### Roster

Commit these ids. Hashes you do not have are `""`. Licence that `corpora.md` states as none stated is the literal `none stated`. Debian package licences use `licence_state = "unresolved"` and omit `licence`.

Text tuning, transform `identity` v1, class `text`, role `tuning`, `counts_toward_sm1 = false`:

- `canterbury-alice29`, `canterbury-lcet10`, `canterbury-plrabn12`, `canterbury-cp.html`, `canterbury-xargs.1`
- `calgary-book1`, `calgary-book2`, `calgary-paper1`, `calgary-paper2`

Text evaluation, transform `slice` v1, length 3145728, class `text`, role `evaluation`, `counts_toward_sm1 = true`:

- `silesia-dickens`, `silesia-webster`, `silesia-xml`, `silesia-reymont`

PCM, transform `pcm16le-to-u8` v1, class `pcm`. One tuning row `lj-tuning` and one evaluation row `lj-evaluation`. `counts_toward_sm1` is false on tuning and true on evaluation. Companion rows `lj-tuning-raw16` and `lj-evaluation-raw16` use transform `pcm16le-raw` v1, `counts_toward_sm1 = false`. Caveat string on all four: `LJ Speech was sourced from 128 kbps MP3; one speaker`.

Executable tuning, class `executable`, role `tuning`, `counts_toward_sm1 = false`:

- `calgary-obj1`, `calgary-obj2`, `canterbury-sum` (identity)
- `debian-tuning-amd64`, `debian-tuning-arm64` (kind `deb`, package name `""`)

Executable evaluation, class `executable`, role `evaluation`, transform `slice` v1, `counts_toward_sm1 = true`. Shared length is absent, not a guessed number:

- `silesia-ooffice`
- `debian-busybox-static-amd64`, `debian-busybox-static-arm64`
- `debian-ripgrep-amd64`, `debian-ripgrep-arm64`

Caveat on every executable evaluation row: `same packages built for two architectures`.

Compressed control, class `compressed`, role `evaluation`, kind `direct`, transform `identity` v1, `counts_toward_sm1 = false`:

- `enwik8-zip` (the zip bytes, not the inflated text)
- `flac-subset` (member list absent, so the row is unresolved)

Synthetic rows are specified under AC07. They are the only rows with a known `derived_sha256` at the end of this story.

`counts_toward_sm1` is true only when class is `text`, `pcm`, or `executable`, role is `evaluation`, and transform is not `pcm16le-raw`. It does not look at the cache or at whether a hash is filled in. Readiness is the unresolved list.

Source URLs, recorded as strings, hashes left empty:

- Canterbury tar: `https://corpus.canterbury.ac.nz/resources/cantrbry.tar.gz`
- Calgary tar: `https://corpus.canterbury.ac.nz/resources/calgary.tar.gz`
- Silesia zip: `http://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip`
- enwik8 zip: `http://mattmahoney.net/dc/enwik8.zip`
- LJ Speech tar.bz2: `https://data.keithito.com/data/speech/LJSpeech-1.1.tar.bz2`
- Debian rows: `https://snapshot.debian.org/` with the package fields empty

Do not fetch these URLs. Do not type MD5 values from memory.

### Replacement rules

Record only this approved rule, under `replacement.text`:

> Evaluation inputs stay fixed. A tuning input that fails the overlap check is dropped and replaced by another Calgary or Canterbury input that passes. No stripping, no moving evaluation slices, no allowlist.

`replacement.pcm` and `replacement.executable` are absent. The gate lists `pcm.replacement_rule` and `executable.replacement_rule` as unresolved. Do not describe a Debian package rotation. That decision is still open.

### Overlap

`K` is a parameter. Production exploratory `K` is 1024. The tests use `K = 16`.

Let `L = K // 2` and `s = K - L + 1`. Index each evaluation file's windows of length `L` only at offsets that are multiples of `s`. A shared run of `K` bytes always contains one such window. Scan each tuning file at every byte offset and look up the exact `L` bytes. On a hit, extend backward and forward. The first confirmed run of at least `K` is the witness for that pair. Stop. Do not build a dict of every `K`-byte window.

The witness is `{tuning_id, evaluation_id, tuning_offset, evaluation_offset, length, repeated_byte}` where `repeated_byte` is true when every byte of the run is the same. Callers do not drop those hits.

The brute-force oracle checks every alignment of two files that are each at most 4096 bytes. The optimized function must return the same hit-or-miss on those inputs. Tests:

- A run of exactly `K` at each phase `0 .. s-1`, and the oracle agrees.
- A run of `K - 1` is a miss.
- Two different SHA-256 values can still hit.
- A hit that would cross a 4096-byte boundary is still a hit. The checker has no chunk size.
- A long run of `0x00` is reported and is not ignored.

Empty files do not overlap. `K < 2` or an odd `K` is `BadManifest` for the library function. 1024 is legal.

### Gate report

`gate(manifest) -> dict` then canonical JSON: `json.dumps(obj, sort_keys=True, indent=2, ensure_ascii=True) + "\n"`.

```json
{
  "corpus_gate_ready": false,
  "counts_toward_sm1": [],
  "excluded_from_sm1": [],
  "k": {"bytes": 1024, "status": "proposed"},
  "schema": 1,
  "scope": "corpus",
  "unresolved": [{"field": "k.status", "reason": "proposed"}],
  "violations": []
}
```

`corpus_gate_ready` is true only when `unresolved` and `violations` are both empty and `k.status` is `confirmed`. The committed file is false. `k.status = proposed` is an unresolved field even though `k.bytes` is 1024.

Unresolved on the committed manifest, at least:

- `k.status`
- `lj.seed`
- `lj.metadata_checked`
- `lj.prefixes`
- `pcm.byte_budget.tuning`
- `pcm.byte_budget.evaluation`
- `pcm.replacement_rule`
- `debian_tuning.package`
- `executable.replacement_rule`
- `executable.eval.slice_length`
- `flac-subset.members`
- each empty `archive_sha256` and each empty `derived_sha256`
- each empty text-evaluation offset
- each Silesia `member_md5`

Violations are for broken rules, not for missing pins. A correct committed manifest has an empty `violations` array. Examples that must be violations in tests, on a fixture manifest, not on the committed one: a PCM row with transform `slice`, a text evaluation length other than 3145728, `counts_toward_sm1 = true` on a synthetic or raw-16 row, a second evaluation length among executable rows when a length is set.

`excluded_from_sm1` lists every id where the static flag is false. `counts_toward_sm1` lists the rest. Synthetic ids are excluded. The test loads `corpora/gate-report.json` and compares bytes to a fresh `gate()` call.

### Synthetic signals

`crates/sxp-bench/src/synthetic.rs`. Each signal is 4096 bytes: `predict` for `n` in `0..2048`, then the same call again. Recurrence uses `predict_span(2048)` for each half. Do not use floating point. Do not reimplement Horner, the sine LUT, or `rem_euclid`.

| Id | Constructor |
| --- | --- |
| `synthetic-polynomial` | `Polynomial::new(1, 8, &[0, 65536])` |
| `synthetic-trigonometric` | `Trigonometric::new(1, 8, 0x0100_0000, &[32768, 32000, 0])` |
| `synthetic-recurrence` | `Recurrence::new(2, 256, &[1, 1, 0], &[0, 1])` |

The seed is that parameter tuple. Put it in the manifest `parameters` table. `class = "synthetic"`, `role = "synthetic"`, `transform = "kernel-predict"`, `transform_version = 1`, `counts_toward_sm1 = false`, `licence = "none stated"`.

The Rust test asserts the second half equals the first, hashes the bytes with the existing `sha2` pin, and asserts the lowercase hex equals the manifest `derived_sha256`. Parse `corpora/manifest.toml` with the pinned `toml` crate. A mismatch fails the test. Compute the hashes by running the test once, write the literals into both places, and re-run. Do not leave a `todo`.

`sxp-bench` `lib.rs` gets the same `forbid(unsafe_code)` and clippy deny list as `sxp-encoder`. Index with `get`. No `unwrap` in the library. Tests may use `unwrap`.

### Fixture matrix

Build the archives in the test with `tarfile`, `zipfile`, and the `ar` writer implied by the reader. No network.

| Case | Required result |
| --- | --- |
| tar.gz member, good SHA-256 | output bytes match, parent file has no extra member |
| flipped archive byte | `HashMismatch`, output absent |
| member name `../x` | `PathTraversal` |
| member name `/tmp/x` | `PathTraversal` |
| symlink or hard link | `UnsafeLink` |
| two headers with the same name | `DuplicateMember` |
| truncated tar | `TruncatedArchive` |
| kind `flac` | `UnsupportedFormat` |
| zip with the symlink Unix mode | `UnsafeLink` |
| zip flag bit 0 | `Encrypted` |
| declared size above 64 MiB | `TooLarge` |
| good MD5 | success |
| bad MD5 | `Md5Mismatch` |
| deb whose data member is `data.tar.xz` and path `./usr/bin/rg` | extracted bytes match |
| deb whose data member is `data.tar.zst` | `UnsupportedFormat` |
| direct file | SHA-256 of the whole file, no decode |
| empty `archive_sha256` on extract | `UnresolvedHash` |
| `pin` without a SHA-256 | stdout has both hashes, manifest path unchanged |
| WAV samples −32768, −1, 0, 32767 | `00 7f 80 ff` |
| stereo or 44100 Hz | `UnsupportedWav`, no output |
| raw-out | exact little-endian frames |
| slice length 4 of a 10-byte file | offset 3 |
| cache path inside the repo | `CacheInsideRepo` |
| relative `SEREXPAI_CACHE` | `CacheNotAbsolute` |

Also assert the test process never calls `urllib.request.urlopen` or `socket.socket`. Patch both before importing the code under test, or patch them at the start of the run.

### Project Structure Notes

- Manifest path is `corpora/manifest.toml`. `.gitignore` ignores `corpus/`, which is a different directory. Do not download into `corpora/`. Add nothing under `corpora/` except the manifest, the gate report, and the existing `.gitkeep`.
- `sxp-bench` already depends on `sxp-kernel`, `toml`, and `sha2`. Use those pins. Do not add a crate.
- Python 3.14.4 is the minimum. `tomllib` reads the manifest. Keep the file in the TOML 1.0 subset: strings, integers, bools, arrays, tables. No dates.
- Integration tests may use `unwrap`. The new Rust library code may not.
- Platforms for the synthetic test: `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`. Python tests run on the host interpreter only. State that in the completion notes. Do not claim aarch64 executed `test_fetch.py`.

### Preserve from 1.11

- `ci.sh` still refuses a toolchain other than 1.99.1 and still runs fmt, clippy `-D warnings`, tests, and the kernel tree check.
- Do not change compress, decompress, or the kernel evaluators. Call them.
- The deferred temp-unlink note in `deferred-work.md` stays deferred. Do not "fix" it here.
- Do not bump the workspace version and do not edit `CHANGELOG.md`.

### Git intelligence

Recent commits are one story each: `d02eb72` compress and decompress, `799ca19` codebook, `e6bdfbc` baseline selection, `08af68a` baseline blocks, `c05a8b0` model. Land this story the same way: behavior in `tools/fetch`, `corpora/manifest.toml`, and `sxp-bench`, with tests.

### Latest tech

Python on this machine is 3.14.4, which is the architecture floor. `audioop` is not in 3.14. `wave` reads little-endian PCM only, so a big-endian WAV fails `comptype` or `sampwidth` checks rather than a byte swap. `tarfile` data extraction must use `extractfile` plus a bounded read, not `extract` and not `extractall`. The pinned `toml` crate is 1.1.8 and `sha2` is 0.11.1. Do not upgrade them.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.12]
- [Source: _bmad-output/specs/spec-serexpai/corpora.md]
- [Source: _bmad-output/specs/spec-serexpai/acceptance-criteria.md#CAP-13]
- [Source: _bmad-output/planning-artifacts/prds/prd-serexpai-2026-10-30/prd.md#FR-15]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md#AD-14]
- [Source: docs/decision_logs/decision_log_2026_W44.md] K proposed, LJ prefixes unchecked, Debian tuning package unnamed
- [Source: _bmad-output/implementation-artifacts/1-11-compress-and-decompress-a-file.md]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- `python3 tools/fetch/test_fetch.py` passed, 23 tests, on Python 3.14.4. Host only. Re-run after the review patches, still 23 passed.
- `./ci.sh` passed on `x86_64-unknown-linux-gnu` before the review patches.
- `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu` passed before the review patches. Synthetic hashes matched. `commands.rs` 316.89s, `select.rs` 127.67s.
- After the review patches: `cargo test -p sxp-bench --locked --test synthetic` and `cargo clippy -p sxp-bench --all-targets --locked -- -D warnings` passed. The same synthetic test passed under `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu` for `aarch64-unknown-linux-gnu`. `corpora/gate-report.json` still matches `gate()`. `violations` is empty. Python tests are 24 and host only.

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- `tools/fetch` verifies archive SHA-256 before reading a member, reads `.deb` `data.tar.xz` via `ar`, and refuses an empty pin on the extract path. `pin` prints hashes and does not write the manifest.
- `pcm16le-to-u8` v1 and middle `slice` v1 match the fixtures. PCM selection stops at the first clip that does not fit.
- The overlap checker indexes half-windows on a stride and agrees with the brute-force oracle. Repeated-byte hits are reported.
- `corpora/gate-report.json` has `corpus_gate_ready` false. K, the LJ seed, both PCM budgets, and the Debian tuning package stay unresolved.
- Three 4096-byte signals come from the kernel evaluators. Their SHA-256 values are in the manifest and the Rust test. `counts_toward_sm1` is false for all three.
- `ci.sh` was not changed. Python tests are not part of it.

### File List

- corpora/gate-report.json
- corpora/manifest.toml
- crates/sxp-bench/src/lib.rs
- crates/sxp-bench/src/synthetic.rs
- crates/sxp-bench/tests/synthetic.rs
- tools/fetch/corpus.py
- tools/fetch/fetch.py
- tools/fetch/test_fetch.py
- _bmad-output/implementation-artifacts/1-12-fetch-the-corpus-manifest.md
- _bmad-output/implementation-artifacts/deferred-work.md
- _bmad-output/implementation-artifacts/sprint-status.yaml

### Change Log

- 2026-11-03: Story context created. Fetch stays on the Python standard library. K, the LJ seed, PCM budgets, and the Debian tuning package stay unresolved, so the corpus gate stays not ready. `ci.sh` is unchanged because it calls cargo and rustup only.
- 2026-11-03: Implemented fetch, the manifest, the corpus gate, and the three synthetic signals. `python3 tools/fetch/test_fetch.py`, `./ci.sh`, and the aarch64 qemu run all exited 0.
- 2026-11-03: Review patches applied. Cache writes hash the temp before replace, archive reads stay capped, and malformed gate fields are violations. Two overlap items are deferred. The committed gate report still has an empty `violations` array.
- 2026-11-03: Second review. Short archives report `TruncatedArchive`, a negative ustar size reports `CorruptArchive`, and non-string gate fields are violations. Zip members of every mode are accepted. A symlink stays `UnsafeLink` and a directory stays `NotRegular`.

## Story completion status

done. Fetch, the unpinned manifest, and the synthetic signals are in place. The corpus gate stays not ready. Zip members of every mode are accepted. A symlink is still `UnsafeLink` and a directory is still `NotRegular`. Python tests ran on the host only.
