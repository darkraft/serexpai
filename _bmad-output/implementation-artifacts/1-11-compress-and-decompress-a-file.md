---
baseline_commit: 799ca1921abe32a845b4f52a443adf0503848625
---

# Story 1.11: Compress and decompress a file

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author,
I want lossless file commands with atomic output and safe failures,
so that a bad input never replaces a destination the user already has.

## Acceptance Criteria

1. **S1.11-AC01.** `sxp-cli compress SRC DST` writes the bytes of `encode_codebook` for `generate_codebook("fixture-v1")`, `all_baselines()`, `LengthPolicy::All`, `SearchMode::HeaderPrune`, and `threads = 0`. `sxp-cli decompress` restores the source bytes. Encoded length is at most `N + 11`. The committed matrix is empty, 1 byte, 255, 256, 257, 1024, 1025, 2048, one fixed-seed random input, and that random input's own compressed bytes. `threads` 0, 1, and 2 and `SearchMode::Reference` produce the same bytes as the command. Both `./ci.sh` targets pass.

2. **S1.11-AC02.** The overwrite and alias rules in Dev Notes are crate rustdoc on the install module, not a new markdown file. On every error the destination directory entry is byte-identical, no new destination appears, and the only file removed is the temp file this invocation created. A pre-existing destination is never deleted to satisfy "no output."

3. **S1.11-AC03.** Success creates a temp file in the destination parent, writes the full payload, finishes validation including CRC, flushes, `sync_all`s, closes, then `rename`s onto the destination. Injected read, write, flush, sync/close, and rename failures leave the destination unchanged and remove only this invocation's temp file.

4. **S1.11-AC04.** Compress refuses when the source metadata length is `>= sxp_kernel::LENGTH_LIMIT` (`1 << 35`) and does not read the bytes. Decompress does not apply that limit to the encoded file's byte length. Decompress rejects a declared `N >= 2^35` as `Leb128Overflow`. Tests use a fake length and a header fixture, not a multi-gigabyte allocation.

5. **S1.11-AC05.** Success exits 0 with empty stdout and empty stderr. Every failure, including argument parsing, exits 1 with a non-empty stderr line and empty stdout. `--help`, `-h`, and `<command> --help` exit 0 with usage on stdout. Truncated, bad-magic, CRC-mismatched, and trailing-byte inputs do not create or modify the destination.

6. **S1.11-AC06.** Kernel parse and decode stay on `Result`. `sxp-kernel` and the new `sxp-cli` library keep `forbid(unsafe_code)` and the existing clippy deny list (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo`, `unimplemented`, `unreachable`). `read_file` is covered by the counting-allocator tests at `N = 0` or one Block and at 1 MiB, peak at most 64 KiB, and the peak does not grow with `N`. A recorded arbitrary-byte decompress run does not panic and does not install a destination.

7. **S1.11-AC07.** Decompress calls `sxp_kernel::read_file` only. That module does not import `sxp_encoder` or `sxp_bench`. The command has no stdin or stdout data path, no config file, no logging crate, and no `benchmark` subcommand. No `--recipe`, `--threads`, or `--force` flag.

## Tasks / Subtasks

- [x] Kernel mode dispatch (AC: 4, 6, 7)
  - [x] Add `read_file` in a new `crates/sxp-kernel/src/file.rs` and export it. Do not change `read_blocked_file` or `read_whole_file_raw` behavior
  - [x] `pub(crate) fn read_header_prefix` in `raw.rs` owns the existing `read_header` match arms. `read_file` calls it. `Ok` with `used != len` is `Leb128Overflow`. Retry only `TruncatedHeader` and `TruncatedLeb128` while `len < 7`
  - [x] Replay those buffered bytes through `std::io::Chain` into the existing mode-0 or mode-1 reader. Do not buffer the body
  - [x] Counting-allocator cases call `read_file` for both modes
- [x] CLI surface (AC: 5, 7)
  - [x] `sxp-cli` becomes a library plus the existing binary name `sxp-cli`. `main` calls `process::exit` on the library's code
  - [x] `clap = "=4.6.7"` via `try_get_matches_from`. Map help to exit 0 / stdout. Map every other parse or runtime failure to exit 1 / stderr. Do not use clap's process exit (code 2)
  - [x] Subcommands `compress` and `decompress`, each with exactly two required `PathBuf` positionals
  - [x] Same deny/forbid attributes as the kernel, on `lib.rs` and `main.rs`
- [x] Atomic install (AC: 2, 3)
  - [x] Rustdoc on the install module states the policy below
  - [x] `Fs` trait plus `StdFs` and an in-memory `MemFs` with fail points. No `tempfile` crate
  - [x] Temp name prefix `.sxp-tmp-`, `create_new`, in the destination parent
  - [x] Failure injection tests for read, write, flush, sync/close, and rename
- [x] Commands and matrix (AC: 1, 4, 5, 6, 7)
  - [x] Compress reads the source, checks metadata length, calls `encode_codebook` as in AC01, writes, round-trips through `read_file`, then renames
  - [x] Decompress streams `read_file` into the temp file. Declared-length rejection happens inside the kernel
  - [x] Committed matrix, `N + 11`, thread and `Reference` equality, malformed destinations, arbitrary-byte decompress
  - [x] `./ci.sh` passed on `x86_64-unknown-linux-gnu`. `aarch64-unknown-linux-gnu` did not link: `aarch64-linux-gnu-gcc` is not installed and `sudo` cannot install it. Superseded the same day: both targets passed again after the review patches. aarch64 was `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu`, exit 0.

### Review Findings

- [x] [Review][Decision] aarch64 CI did not run — resolved 2026-11-03. `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu` passed on the pre-patch tree. The loader is `/usr/aarch64-linux-gnu/lib/ld-linux-aarch64.so.1`.
- [x] [Review][Patch] Do not collapse `..` when choosing the temp parent [crates/sxp-cli/src/fs.rs]
- [x] [Review][Patch] A symlink parent that points at a directory is a directory [crates/sxp-cli/src/install.rs]
- [x] [Review][Patch] Destination stat errors other than absence fail closed [crates/sxp-cli/src/install.rs]
- [x] [Review][Patch] A directory source is IsADirectory, not Usage [crates/sxp-cli/src/install.rs]
- [x] [Review][Patch] Same-file spelling uses absolute lexical paths [crates/sxp-cli/src/install.rs]
- [x] [Review][Patch] MemFs follows symlink chains and relative targets [crates/sxp-cli/src/fs.rs]
- [x] [Review][Patch] Failure tests assert that open failures create no temp file [crates/sxp-cli/tests/commands.rs]
- [x] [Review][Patch] A destination ending in `/.` is Usage [crates/sxp-cli/src/install.rs]
- [x] [Review][Defer] A failed temp unlink keeps the original error token [crates/sxp-cli/src/install.rs:112] — deferred, pre-existing

## Dev Notes

### What this story builds

The kernel gains one dispatcher, `read_file`, so the CLI does not reimplement the header. The CLI gains `compress` and `decompress` with temp-file rename. Selection, framing, the fixture codebook, and both existing readers stay as story 1.10 left them.

Out of scope: `benchmark`, corpus fetch, sidecars, reports, a production recipe, a D-07 width, `--recipe`, `--threads`, `--force`, stdin/stdout bytes, a config file, a logging crate, a new crate dependency, a format bump, edits to `bitstream-spec-seed.md`, README, or `CHANGELOG.md`. Do not remove the unused `sxp-bench` dependency from `sxp-cli` (the spine's crate graph). Do not call `sxp_bench` from either command.

`read_blocked_file` and `read_whole_file_raw` still write decoded bytes before the CRC check. Rollback is the CLI deleting its temp file. Do not "fix" those readers to buffer the body.

### Overwrite and alias policy

This is the policy AC02 requires. Put it in the install module's rustdoc.

- Success replaces the destination directory entry via `rename` of a temp file in the same parent. That is the only overwrite. There is no prompt and no `--force`.
- Failure does not call `remove` on the destination. An existing destination stays byte-identical. A missing destination stays missing.
- The temp file is created with `create_new` so an existing path is never opened for writing. Its name is `.sxp-tmp-<dest-file-name>.<pid>.<n>`. On `AlreadyExists`, bump `n` at most 100 times, then return `AlreadyExists`. On failure, remove that path only. A sibling that already existed, including another `.sxp-tmp-*` file, stays.
- `rename` that returns `EXDEV` or any other error is a failure. Do not copy onto the destination and do not delete it.
- Refuse, before creating a temp file, when any of these hold:
  - the destination has no file name (`foo/` or `..`) — token `Usage`
  - the destination parent is missing — token `NotFound`. An empty parent means `.`
  - the destination parent exists and is not a directory — token `NotADirectory`
  - the destination exists and `symlink_metadata` says it is a directory — token `IsADirectory`. A socket or device — token `Unsupported`. A symlink is allowed; `rename` replaces the symlink itself
  - the source is missing — token `NotFound`. Followed `metadata` says the source is a directory (including a symlink to a directory) — token `IsADirectory`
  - the two paths are the same directory entry: absolute lexical paths compare equal, or both exist and `(st_dev, st_ino)` from `metadata` (follows symlinks) match. Hard links and a symlink to the same inode are refusals — token `SameFile`
- Opening the source follows symlinks. Open it before creating the temp file. The command never opens the destination path itself for write or truncate.
- The new file uses the process create mode. It does not copy the old destination's mode. The parent directory is not fsynced after `rename`. A failure after a successful `rename` cannot be rolled back. State both in the rustdoc and do neither.
- Compress uses the followed metadata length (`metadata`, not `symlink_metadata`) for the `2^35` check. A symlink to a huge file is refused. Decompress ignores encoded-file length.

Stderr is one line, no timestamp and no clap paragraph:

```text
sxp-cli: <compress|decompress>: <token>
```

With no subcommand, the line is `sxp-cli: Usage`. Argument errors on a known subcommand use token `Usage`. Other tokens: kernel or encoder `Display` (the variant name), `NotFound`, `AlreadyExists`, `IsADirectory`, `NotADirectory`, `Unsupported`, `SameFile`, `CrossDevice`, `LengthChanged`, `ValidationMismatch`, `Io`. Stdout is empty except for help.

`Command` uses `subcommand_required(true)` and `disable_help_subcommand(true)`. Do not set `arg_required_else_help`. `help` is not a third subcommand.

### Compress steps

1. Parse. On usage or help, return before any IO.
2. Apply the alias rules. Stat the source length. If `len >= LENGTH_LIMIT`, return `LengthOutOfRange` and do not read.
3. Open the source and read it with `Read::take(len + 1)` into a `Vec<u8>`. If the count is not `len`, token `LengthChanged`. A read error here creates no temp file.
4. `generate_codebook("fixture-v1")` then `encode_codebook(input, &all_baselines(), &codebook, LengthPolicy::All, SearchMode::HeaderPrune, 0)`. Do not call `generate_codebook("production")` or `"benchmark"`. Do not call `encode_baselines`.
5. Create the temp file and write every encoded byte through an unbuffered handle. No `BufWriter`.
6. Re-read the temp file through `sxp_kernel::read_file` into a `Write` that compares against `input` and rejects a short or long decode. Token `ValidationMismatch`. This is the CRC validation. A mismatch or read error deletes the temp file and does not flush or rename.
7. `flush`, `sync_all`, close.
8. `rename` the temp file onto the destination.

The fixture recipe is the only legal codebook while D-09 is open. The rustdoc says the command's recipe name is `fixture-v1` and its label is `fixture`, not `production` and not `benchmark`. A later recipe change is an encoder input change, not a format bump.

### Decompress steps

1. Parse and apply the same alias rules. Do not compare the encoded file's length with `LENGTH_LIMIT`.
2. Open the source. A missing source is `NotFound` and creates no temp file.
3. Create the temp file, then `read_file` from the source into it.
4. On `Ok`: `flush`, `sync_all`, close, `rename`.
5. On `Err`: delete the temp file. The kernel may already have written plaintext into it. The destination must not observe those bytes.

`crates/sxp-cli/src/decompress.rs` imports `sxp_kernel` only. A round-trip test decompresses a file produced by compress without a codebook path on disk.

### `read_file`

```rust
pub fn read_file(input: impl Read, output: impl Write) -> Result<(), Error>
```

- Add `pub(crate) fn read_header_prefix` in `raw.rs` that returns the buffered bytes, the length, and the `Header`, using today's `read_header` match arms. `read_header` and `read_file` both call it. `read_stream_header` in `block.rs` keeps those same arms. Do not invent a third truncation policy.
- On `Ok((header, used))` with `used == len`, chain `buf[..len]` in front of the unread input. Mode 1 calls `read_whole_file_raw`. Mode 0 calls `read_blocked_file`. `Ok` with `used != len` is `Leb128Overflow` and writes nothing.
- Retry only `TruncatedHeader` and `TruncatedLeb128` while `len < 7` and the reader is not at EOF. At EOF, return the truncation error from `decode_header` and write nothing.
- A declared `N >= 2^35` is `Leb128Overflow` from `decode_leb128`. Do not add a second check that can disagree.
- `Chain` and `Cursor` only. No `Vec` sized to `N`.

Mode 1 with `N = 0` is the 7-byte empty stream `8D 11 00 00 00 00 00`. Mode 0 with `N = 0` is `BlockedEmpty` before any body read.

### CLI layout

| Path | Role |
| --- | --- |
| `crates/sxp-cli/src/lib.rs` | `run` and `run_with`. Deny lints |
| `crates/sxp-cli/src/main.rs` | `process::exit(sxp_cli::run())` |
| `crates/sxp-cli/src/args.rs` | clap 4.6.7 |
| `crates/sxp-cli/src/fs.rs` | `Fs`, `StdFs`, `MemFs` |
| `crates/sxp-cli/src/install.rs` | policy rustdoc, temp file, rename |
| `crates/sxp-cli/src/compress.rs` | encoder call |
| `crates/sxp-cli/src/decompress.rs` | kernel only |
| `crates/sxp-kernel/src/file.rs` | `read_file` |
| `crates/sxp-cli/tests/commands.rs` | matrix, injection, malformed |
| `crates/sxp-kernel/tests/heap_bound.rs` | `read_file` mode 1 |
| `crates/sxp-kernel/tests/block_heap.rs` | `read_file` mode 0 |

`Cargo.toml` `[[bin]]` name stays `sxp-cli`. Add no dependency. `clap` is already `=4.6.7`. Rust stays 1.99.1. Do not upgrade clap or rayon.

`run_with` takes an `Fs` and returns `{ code, stdout, stderr }` so tests never spawn a process. `run` uses `StdFs` and writes the buffers to the real stdio handles.

Help is a clap `ErrorKind::DisplayHelp` (and `DisplayVersion` if one appears). Print it to stdout and return 0. Every other clap error prints to stderr and returns 1.

### Failure injection

`MemFs` stores bytes, an inode, and a reported length that can disagree with the stored bytes (the `2^35` case). Fail points: `Read`, `Write`, `Flush`, `Sync`, `Rename`. Each test starts with a destination whose bytes are `b"OLD"` where a destination exists, plus an unrelated sibling `.sxp-tmp-keep`.

| Command | Injection | When | Required end state |
| --- | --- | --- | --- |
| both | Read | opening the source, before create | no new file, `OLD` intact, sibling intact |
| compress | Read | validation re-read of the temp file | temp gone, `OLD` intact |
| decompress | Read | body read after the temp file exists | temp gone, `OLD` intact |
| both | Write | first write to the temp file | temp gone, `OLD` intact |
| both | Flush | after a successful write and, for compress, after validation | temp gone, `OLD` intact |
| both | Sync | after a successful flush | temp gone, `OLD` intact |
| both | Rename | after close | temp gone, `OLD` intact |

Also assert a missing destination stays missing, and a pre-existing destination is not removed when decompress sees `BadMagic`.

### Committed matrix

Build inputs in the test, not as new corpus files. Random bytes are this LCG, 64 bytes, seed `0x11_11_u64`: `state = state.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1)`, emit `(state >> 56) as u8`.

| Input | Why |
| --- | --- |
| 0 bytes | empty Whole-file raw, 7 bytes |
| 1, 255 | shorter than a Block |
| 256, 1024, 2048 | exact nominals |
| 257, 1025 | partial final |
| 64-byte LCG | random |
| compress(those 64 bytes) | already compressed, still `<= N + 11` |

For every row, compress then decompress equals the input, and `HeaderPrune` at `threads` 0, 1, and 2 equals `Reference` at `threads` 0. The CLI command itself uses `threads = 0` and does not build a rayon pool. The thread comparison calls `encode_codebook` directly.

Malformed decompress fixtures, each exit 1, stderr contains the token, destination unchanged:

| Bytes | Token |
| --- | --- |
| empty | `TruncatedHeader` |
| `00` | `BadMagic` |
| empty stream `8D 11 00 00 00 00 00` with the last byte flipped to `01` | `CrcMismatch` |
| that same valid empty stream plus `00` | `TrailingBytes` |
| `spec/vectors/reject/leb-overflow.bin` (`8D 11 80 80 80 80 80 01`) | `Leb128Overflow` |

`leb-overflow.bin` is the existing declared-`N` fixture. `read_file` must reject it without writing a body. Do not allocate `2^35` bytes. Do not add a new vector file.

Compress source whose `MemFs` reported length is `1 << 35` and whose stored body is empty: exit 1, token `LengthOutOfRange`, and the read fail-point counter stays 0. The same reported length on decompress of a valid 7-byte empty stream still decodes (the limit is not the encoded length).

Arbitrary bytes: a recorded loop of 32 seeds. No `proptest` dev-dependency. Each case is `8D 11` plus 8 LCG bytes, decompressed into a missing destination. Assert exit 1, empty destination, and no panic. If a seed is a valid stream, change that seed so the case stays a rejection. The prefix is what pushes the decoder past the magic byte.

### Heap

`BLOCK_WINDOW` is 36864. `read_file` must not allocate another body buffer. `heap_bound.rs` and `block_heap.rs` already exist and each has one `#[test]`. Call `read_file` inside that same test, sequentially, on the same sizes those files already use (`0` and 1 MiB for mode 1, `256` and 1 MiB for mode 0). Keep `<= 64 * 1024` and `large <= small`. Do not add a second `#[test]` in those files. Do not copy a third global allocator into `sxp-cli`.

### Preserve from 1.10

- Codebook `version` 1 and `content_sha256` are encoder-side. The file version nibble stays 1. Decompress never sees the recipe, the label, or the hash.
- Baseline ids 0..=258 and formula ids 259, 260, 261 stay. Wire kinds are not ids.
- Header prune is the only prune. A cost error aborts. The CLI does not add a prune and does not catch `BlockTooLarge` as a skip.
- `threads == 0` builds no pool. `Reference` and `HeaderPrune` must match.
- `generate_codebook` accepts only `"fixture-v1"`. Fixture `cw` values are not a D-07 choice.
- `N == 0` searches no formula. Whole-file raw wins that tie.

### Project Structure Notes

- Decoder crate edge stays `sxp-cli` → `sxp-kernel`. The package may depend on `sxp-encoder`; the decompress module must not use that edge.
- Kernel `Io(ErrorKind)` drops the message. The CLI's own IO errors use the token table above, not the kernel `Io` name, except when a kernel reader returns `Error::Io`.
- Integration tests may use `unwrap`. Library and binary code may not.
- Platforms: `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` via `./ci.sh`. Unix `st_dev` / `st_ino` are required. Do not add a Windows branch.

### Git intelligence

Recent commits are one story each: `799ca19` codebook, `e6bdfbc` baseline selection, `08af68a` baseline blocks, `c05a8b0` model, `d06ecd6` range coder. Follow that shape. Land the behavior in the existing crates. Do not bump the workspace version and do not edit `CHANGELOG.md`.

### Latest tech

Stay on the pinned stack. Rust 1.99.1, clap 4.6.7, rayon 1.12.0. Clap 4's default error exit is 2; this story cannot use it. Do not take a newer clap. `std::fs::rename` on Linux replaces a file in the same directory atomically. That is the install mechanism. Do not add `tempfile` or `atomicwrites`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.11]
- [Source: _bmad-output/planning-artifacts/epics.md#FR-12], FR-13, NFR-9, NFR-11, NFR-12
- [Source: _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/ARCHITECTURE-SPINE.md#AD-12]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-serexpai-2026-10-30/bitstream-spec-seed.md] file header, LEB128, CRC
- [Source: crates/sxp-kernel/src/raw.rs] rollback comment on `write_whole_file_raw`
- [Source: crates/sxp-kernel/src/block.rs] `read_blocked_file` CRC comment
- [Source: crates/sxp-encoder/src/select.rs] `encode_codebook`
- [Source: crates/sxp-encoder/src/codebook.rs] `generate_codebook("fixture-v1")` only
- [Source: _bmad-output/implementation-artifacts/1-10-search-a-regenerable-formula-codebook.md]

## Dev Agent Record

### Agent Model Used

Grok 4.7

### Debug Log References

- `./ci.sh` passed on `x86_64-unknown-linux-gnu` on 2026-11-03.
- `./ci.sh aarch64-unknown-linux-gnu` failed before tests: linker `aarch64-linux-gnu-gcc` was not found. `qemu-aarch64` is also absent. `sudo` requires a password, so the cross tools were not installed.
- Later on 2026-11-03 the cross tools were installed. `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu` passed before the review patches (commands.rs 318.34s) and again after them (commands.rs 339.87s, select.rs 128.50s, exit 0). Native `./ci.sh` also passed after the patches (commands.rs 56.28s).

### Completion Notes List

- Ultimate context engine analysis completed - comprehensive developer guide created.
- `read_file` replays the shared 7-byte header into the existing mode reader. Counting-allocator tests cover it for both modes and stay at or under 64 KiB.
- `sxp-cli compress` writes `encode_codebook` for `fixture-v1`, all baselines, header prune, and `threads = 0`. `decompress` calls `read_file` only.
- Success renames a temp file in the destination parent. Failures delete that temp file and leave an existing destination byte-identical.
- Native `./ci.sh` passed. The aarch64 run did not link.
- Review patches keep `..` in the temp parent, accept a symlink parent that follows to a directory, fail closed on destination stat errors other than absence, report a directory source as `IsADirectory`, compare same-file spellings with absolute lexical paths, and follow MemFs symlink chains. Open failures create no temp file. A destination ending in `/.` is `Usage`.
- Both `./ci.sh` targets passed after those patches.

### File List

- crates/sxp-cli/src/args.rs
- crates/sxp-cli/src/compress.rs
- crates/sxp-cli/src/decompress.rs
- crates/sxp-cli/src/error.rs
- crates/sxp-cli/src/fs.rs
- crates/sxp-cli/src/install.rs
- crates/sxp-cli/src/lib.rs
- crates/sxp-cli/src/main.rs
- crates/sxp-cli/tests/commands.rs
- crates/sxp-kernel/src/block.rs
- crates/sxp-kernel/src/file.rs
- crates/sxp-kernel/src/lib.rs
- crates/sxp-kernel/src/raw.rs
- crates/sxp-kernel/tests/block_heap.rs
- crates/sxp-kernel/tests/heap_bound.rs
- crates/sxp-kernel/tests/read_file.rs
- _bmad-output/implementation-artifacts/1-11-compress-and-decompress-a-file.md
- _bmad-output/implementation-artifacts/sprint-status.yaml
- _bmad-output/implementation-artifacts/deferred-work.md

### Change Log

- 2026-11-03: Story context created. Compress uses the fixture codebook until D-09 closes. Overwrite and same-file rules are specified here because no earlier document decided them. Compress validates the temp file before flush, matching AC03.
- 2026-11-03: Implemented `read_file` and the compress/decompress commands. Native `./ci.sh` passed. The aarch64 linker is not installed on this machine.
- 2026-11-03: Applied the eight review patches. Post-patch `./ci.sh` and `QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu` both exited 0. Temp-unlink errors stay deferred.

## Story completion status

done. Compress and decompress install output by rename. Both `./ci.sh` targets passed after the review patches. One deferred item remains: a failed temp unlink keeps the original error token.
