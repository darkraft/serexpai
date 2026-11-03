## Deferred from: code review of 1-11-compress-and-decompress-a-file.md (2026-11-03)

- A failed temp unlink keeps the original error token. `abandon` ignores `remove` errors. Returning `Io` instead would hide `CrcMismatch`, `BadMagic`, or `CrossDevice`, and stderr is one line. The leftover path is the temp file, not the destination.
