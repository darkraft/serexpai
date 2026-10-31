# MBSC PRD Addendum

Depth that informs the PRD, architecture and the evaluation protocol but does not belong in the PRD narrative. Source: web-research digest, 2026-10-30. Findings are unverified beyond the cited pages; the author should confirm licences before redistributing anything.

## Comparables and what they imply

- **FLAC (RFC 9639)** is the closest structural relative. Per block it chooses constant, verbatim, fixed predictor (orders 0-4, no stored coefficients) or LPC (up to 32 stored coefficients), codes the residual with partitioned Rice codes, and the encoder chooses by estimated bits. MBSC's constant, raw fallback and per-block encoder choice mirror it. <https://datatracker.ietf.org/doc/rfc9639/>
- **Audio gap is small:** the best codecs sit only 2-3 percentage points ahead of FLAC (OptimFROG 64.55% of raw size vs FLAC -8 66.49% on 8,047 rock WAVs). Do not expect large margins on real PCM.
- **General compressors on PCM** help little on real music (xz about 1.40x, zstd 1.21x, FLAC 1.97x on one guitar clip) but do well on pure sines (xz about 9.3x), which is the case a formula codebook should also win. <https://github.com/pdx-cs-sound/compression-bench>
- **Time-series prior art** is the nearest to polynomial-fit prediction: Sprintz (delta/FIRE prediction), polynomial extrapolation for simulation data (DCC 2000), per-block order 0-3 predictor choice (UC Davis 2012). The researcher found nothing fitting trigonometric or recurrence formulas to raw bytes. Treat as probably novel, though the search was shallow.
- **Context mixing** (cmix, paq8px) is the upper bound on text and a different design. On enwik9, cmix-class is about 92-114 MB, paq8px -12L 124.7 MB, xz -9e about 197-212 MB.
- **rANS:** Duda, <https://arxiv.org/abs/1311.2540>.

**Inference to carry into benchmark expectations:**

- Text bytes are not smooth, so a polynomial or sinusoid predicting `y mod 256` will probably lose to order-0 or previous-byte baselines on text.
- 16-bit PCM read as raw bytes interleaves a low byte (noisy) and a high byte (smooth). PCM is therefore a likely win only for 8-bit PCM or after a reversible byte-lane split.
- Byte-lane separation is out of scope for v1 (separate experiment), so a PCM result on raw bytes may be weak.
- Report a class with no gain honestly. SM-1 needs two real classes, so the choice of classes matters.

## Candidate corpora (not yet chosen)

| Corpus | Contents | Licence status |
|---|---|---|
| Canterbury | 11 files, 2.81 MB, frozen | none stated |
| Calgary | 18 files (14 standard) | none stated |
| Silesia | 12 files, 211.9 MB; text, executables (mozilla, ooffice), databases, images | none on the corpus page |
| enwik8 / enwik9 | Wikipedia XML, published MD5 and SHA-1 | no corpus licence; Wikipedia text generally CC BY-SA |
| FLAC test files | PCM conformance files | CC0, but not a compression benchmark |
| ktf CDDA set | 64 sources | no redistribution licence found |
| SQAM | PCM audio | testing and evaluation only |

- Practical guidance: do not vendor corpora; ship download-and-verify scripts with checksums (matches FR-15). Generate synthetic PCM (sine, chirp) for repo-safe tests.
- No standard already-compressed corpus turned up. Silesia's `mozilla` and `ooffice` cover executables; for the already-compressed control, use the author's own `.zst`/`.jpg` files or generate them from a fixed seed.

## CLI and bitstream conventions seen in comparable tools

- Magic: zstd 4-byte `0xFD2FB528`; xz `FD 37 7A 58 5A 00` (first byte rules out the old LZMA format, last rules out text); gzip `1f 8b`.
- Versioning: xz reserved bits must be zero and a decoder errors if one is set; a matching CRC with reserved bits set means "unsupported version", not "corrupt". Useful model for FR-9's format version.
- Checksums: zstd optional XXH64 (low 32 bits); xz none, CRC32, CRC64 or SHA-256; brotli none. Informed the CRC-32 decision (PRD FR-9).
- Exit codes: gzip 0 success, 1 error, 2 warning; zstd 0 and 1.
- Common flags to consider for later: `-d`, `-c`, `-k`, `-t` (test), `-l`, `-v`, `-q`, `-f`, stdin/stdout filtering.
- Streaming: zstd and brotli are streamable with bounded memory; MBSC's per-block design fits framing for it later.

## Licences of comparable tools (informed the licence decision, PRD §6.1)

- zstd: BSD or GPLv2
- xz: 0BSD since Feb 2024 (earlier public domain)
- brotli: MIT
- Monkey's Audio: BSD-3 since 2023
- libFLAC: BSD-like (other FLAC tools GPL)
- cmix and paq8: GPL
- OptimFROG: proprietary

Permissive (MIT, Apache-2.0, 0BSD, BSD) matches mainstream tools and lets others embed MBSC; avoid linking GPL code if permissive terms are wanted.
