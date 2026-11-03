## Deferred from: code review of 1-12-fetch-the-corpus-manifest.md (2026-11-03)

- Overlap on long repeated-byte runs shorter than K does not return early. Each matching window extends and fails the K test, so the work grows with the product of those runs. Correctness is unchanged. Skip starts already covered by an earlier diagonal when the checker runs on real executables.
- `gate()` does not run the overlap checker. The pinned bytes are not in the tree. Once K is confirmed and the hashes are filled, `corpus_gate_ready` can still become true with no overlap evidence. Story 1.13 has to require that evidence before a reportable run.

## Deferred from: code review of 1-11-compress-and-decompress-a-file.md (2026-11-03)

- A failed temp unlink keeps the original error token. `abandon` ignores `remove` errors. Returning `Io` instead would hide `CrcMismatch`, `BadMagic`, or `CrossDevice`, and stderr is one line. The leftover path is the temp file, not the destination.
