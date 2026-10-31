# Review: versions and technical claims (ARCHITECTURE-SPINE.md, MBSC v1)

Reviewer: independent, read-only (spine not edited). Review date: 2026-10-30.
Sources used: crates.io API (per-crate and per-version metadata, dependencies), `static.rust-lang.org/dist/channel-rust-stable.toml`, Rust blog (1.99.0 / 1.99.1 posts, feed), `rustc_span/src/edition.rs` on master, python.org release API, own arithmetic.

## Verdict

The Stack table is accurate as of 2026-10-30, except Python (3.14.4 is six patch releases behind). The one real problem is AD-5: the "below 2^52" claim is true only under a Horner formulation the spine does not state, and it is off by about 2^220 under plain integer Horner. Recurrence overflow is not bounded anywhere.

## 1. Version verification

| Item | Spine | Independently found | Status |
| --- | --- | --- | --- |
| Rust stable | 1.99.1 | `channel-rust-stable.toml`: `1.99.1 (58cf8f9c3 2026-10-13)`; blog post dated 2026-10-15. 1.99.0 was 2026-10-01. Next stable (1.100) is due about 2026-11-12 on the six-week cadence (my inference). | Confirmed |
| Edition | 2024 | `LATEST_STABLE_EDITION = Edition2024`; no 2027 edition. | Confirmed |
| rayon | 1.12.0 | max stable 1.12.0 (2026-04-14), MSRV 1.80, edition 2021, not yanked | Confirmed |
| clap | 4.6.7 | max stable 4.6.7 (2026-09-14), MSRV 1.85, edition 2024, not yanked | Confirmed |
| serde | 1.0.229 | max stable 1.0.229 (2026-07-18), MSRV 1.56 | Confirmed |
| toml | 1.1.8 | 1.1.8+spec-1.1.0 (2026-10-09), MSRV 1.85, edition 2024 | Confirmed. The crates.io version carries `+spec-1.1.0` build metadata; the Cargo requirement `1.1.8` is fine. |
| sha2 | 0.11.1 | 0.11.1 (2026-10-12), MSRV 1.88, edition 2024, depends on `digest ^0.11`, `cpufeatures ^0.3` | Confirmed. It is a 0.x crate, so a minor bump is breaking; pin with `=` or commit `Cargo.lock`. |
| proptest | 1.12.0 | 1.12.0 (2026-10-10), MSRV 1.88, edition 2021 | Confirmed |
| Python | 3.14.4 | python.org: 3.14.10 (2026-10-25) is current. 3.14.4 was only checked locally (`python3 --version`), never against the web. | Stale |
| constriction (memlog) | 0.5.0, MSRV 1.85, MIT/Apache/BSL | 0.5.0, MSRV 1.85, `MIT OR Apache-2.0 OR BSL-1.0` | Confirmed |
| criterion (memlog only) | 0.8.2 | 0.8.2 (2026-02-04) | Confirmed. It is not in the Stack, so remove or add it. |

Compatibility with Rust 1.99.1 and edition 2024:
- The highest dependency MSRV is 1.88 (sha2, proptest). 1.99.1 satisfies all of them with a wide margin.
- A dependency's edition does not constrain the workspace edition, and clap, toml and sha2 are themselves edition 2024. There is no compatibility risk.
- 1.99.1 is a point release. It fixes a missing `wcslen` symbol and a miscompiled FFI `bool` return on x86_64. Neither touches this design (no FFI, no unsafe).

## 2. Technical claims

- **Signed `>>` is arithmetic.** Confirmed. In Rust, `>>` on signed integers is an arithmetic shift and rounds toward −∞, so `(v + (1 << (Q−1))) >> Q` is floor rounding as AD-5 states. A shift amount ≥ 64 panics in debug and masks in release, but Q ≤ 24 never reaches that.
- **`i64::rem_euclid(256)`.** Confirmed. The result is always in [0, 255] for a positive divisor. It panics only for a divisor of 0 or `i64::MIN.rem_euclid(-1)`, neither of which applies here.
- **Rust `%` is not modulo.** `%` truncates and can return a negative value. AD-3's "mod 256" must be implemented with `rem_euclid` or `wrapping_sub`/`as u8`, never `%`. The spec seed should say so. (The coded symbol `δ mod 256` equals `(y − ŷ) mod 256`, so the recentring to [−128, 127] only matters if the signed δ is used elsewhere.)
- **Overflow behaviour differs between debug and release.** Rust panics on integer overflow in debug and wraps in release by default. AD-5's "impossible by construction" is the right answer, but the decoder must reject out-of-limit streams before evaluating, and the release profile should set `overflow-checks = true` (or use `checked_*`). That way a limit bug surfaces as an error and not as silent platform-identical garbage.
- **Edition 2024 reserves `gen`.** Codebook code that names something `gen` will not compile (use `r#gen` or another name).

## 3. AD-5 arithmetic check

Setup: signed 32-bit coefficients, |a| ≤ 2^31; normalized position x a 16-bit value, 0 ≤ x < 2^16; degree d ≤ 15 (16 coefficients).

| Horner form | Worst-case magnitude | Fits in i64? | Below 2^52? |
| --- | --- | --- | --- |
| Plain integer Horner, `acc = acc·x + a_k` | 2^31·(2^16)^15 ≈ **2^271** | No (d ≥ 2 already needs 2^63) | **No** |
| Fixed-point Horner, `acc = ((acc·x) >> 16) + a_k`; shift equals the position's fractional bits | `acc` ≤ 16·2^31 = 2^35, so product `acc·x` < 2^51 (computed 2^50.9); final `acc` ≈ 2^35 | Yes | **Yes** |
| Sum of products with precomputed `x^k` truncated to 16 fractional bits | 16·2^31·2^16 = 2^51 | Yes | **Yes** |
| Fixed-point Horner where the per-step shift is Q (Q in [8, 24]) rather than 16 | Q < 16 grows by 2^(16−Q) per step: Q = 8 gives about 2^(31+8·15) = 2^151 | No | **No** |

So the claim holds only for a fixed-point Horner whose per-step renormalisation equals the position's fractional-bit count (16), independent of Q, with Q applied once at the end. If x is signed 16-bit the bound only improves. The spine says neither. As written ("Horner, 32-bit coefficients, 16-bit position, degree ≤ 15") the claim is unverifiable, and the most natural reading (Q-scaled coefficients and shift by Q per step) is wrong for Q < 16.

Also unbounded in the spine:
- **Recurrence.** `a·x_{n−1} + b·x_{n−2} + c` with 32-bit a and b and `m_mod` up to 2^32 gives products near 2^63, and two of them overflow i64. A limit on `m_mod` or on the coefficient widths is needed (for example `m_mod ≤ 2^24` and |a|, |b| < 2^24 gives under 2^49).
- **Trigonometric.** (2M+1) coefficients × LUT value × accumulated sum has no stated limit on M, LUT width or interpolation width. It almost certainly fits, but "overflow impossible by construction" is only demonstrated for one family.

## 4. Findings

1. **[high] AD-5 overflow bound is unstated and wrong for the natural implementation.** Fix: put the exact Horner recurrence in AD-5 or the spec seed (`acc = ((acc·x) >> 16) + a_k`, one final `>> Q` with rounding), state the limits as a table per family, and add a golden-vector test at the extreme parameters (max coefficients, x = 65535, d = 15, Q = 8 and Q = 24).
2. **[high] Recurrence and trig families have no overflow limits.** Fix: add explicit bounds for `m_mod`, `a`, `b`, `c`, M and the LUT width, with the worst-case i64 magnitude derived in the spec, and have the decoder reject any violation (AD-5 already says it rejects limit breaks).
3. **[medium] Python is stale and was not web-verified.** Fix: change the Stack row to "3.14 (3.14.10 current, ≥ 3.14.4 required)". Use a CI matrix on the minor version, since stdlib-only decoding does not need an exact patch.
4. **[medium] Stack is missing a JSON writer and a corpus-fetch implementation.** Conventions say "TOML in, JSON out", but `serde_json` (1.0.154, 2026-10-11, MSRV 1.71) is not listed, nor are serde's `derive` feature or the fetch tool (`tools/fetch`, language and downloader unspecified). Fix: add `serde_json` to the `mbsc-bench` row and name the fetch implementation (for example a Python stdlib script with `urllib` and `hashlib`).
5. **[low] AD-1's CI check conflicts with dev-dependencies.** The "Verification" row calls for property tests (proptest) of AD-3, which live in or near the kernel, but `cargo tree -p mbsc-kernel` also lists dev-dependencies. Fix: specify `cargo tree -p mbsc-kernel -e normal` (or `--edges no-dev`), or put the proptest tests in a separate test crate. Also commit `Cargo.lock`, add `rust-toolchain.toml` pinned to 1.99.1, and set `rust-version = "1.88"` (the highest dependency MSRV) so versions don't drift.

## 5. Provenance of the committed versions

- Rust 1.99.1 and the crates.io versions in the memlog are marked "verified web 2026-10-30" or "crates.io stable (2026-10-30)". I reproduced them all and they are correct.
- Python 3.14.4 is marked "local" only, not web-checked (finding 3).
- The technical claims in AD-3, AD-5 and AD-7 (shift semantics, `rem_euclid`, the 2^52 bound) are asserted in the memlog without any recorded derivation. The 2^52 bound is the only one that fails (section 3).
