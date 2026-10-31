# Workflow Decision Log — 2026 W44

Append-only record of questions asked, options offered, and choices made.
ISO 8601 calendar week 44 of 2026.

## 2026-10-30 23:14 — bmad-product-brief — Brief assumption review - secondary users

**Q:** Besides you as researcher/builder, who else does the CLI serve?

**Options:**
- Compression hobbyists/researchers (draft)
- No one else, single-user research
- Reviewers/readers of published results and bitstream spec
- Developers embedding it later as library/format
- Practitioners with signal-like data (PCM, sensors)

**Choice:** This will probably eventually be an open-source project for a large audience

**Notes:** custom answer

## 2026-10-30 23:14 — bmad-product-brief — Brief assumption review - research pass bar

**Q:** What counts as the research passing versus baselines on total encoded size?

**Options:**
- Any gain on one real class (draft)
- Minimum margin (2-5%) on one real class
- Gain on at least two distinct real classes, no regression on already-compressed data beyond raw-fallback overhead
- Win on synthetic signals plus any gain on one real class
- Leave unset until after first prototype

**Choice:** Gain on at least two distinct real classes, no regression on already-compressed data beyond raw-fallback overhead

## 2026-10-30 23:15 — bmad-product-brief — Brief open questions - gain definition

**Q:** Does gain on two real classes mean any reduction or a minimum percentage?

**Options:**
- Any reduction
- Minimum percentage vs best baseline

**Choice:** gain is any reduction

## 2026-10-30 23:15 — bmad-product-brief — Brief open questions - licence

**Q:** What licence and contribution model fit the open-source plan?

**Options:**
- (open-ended)

**Choice:** License currently not finally determined

**Notes:** left as open question in brief

## 2026-10-30 23:22 — release-manager — First release - gitignore policy

**Q:** Which BMad paths should the code branch (main) track?

**Options:**
- Gitignore _bmad-output/, _bmad/, .agents/ on main (recommended)
- Gitignore only _bmad/ and .agents/
- Track everything on main

**Choice:** Gitignore _bmad-output/, _bmad/, .agents/ on main

## 2026-10-30 23:22 — release-manager — First release - version

**Q:** Default first release 0.0.1 (patch from 0.0.0). Confirm?

**Options:**
- 0.0.1 (default)
- 0.1.0
- No version bump

**Choice:** 0.0.1 (default)

**Notes:** patch is the default bump

## 2026-10-30 23:22 — release-manager — First release - commit message

**Q:** Short or Long commit message?

**Options:**
- Short
- Long

**Choice:** Long

**Notes:** Long is the default; used without asking

## 2026-10-30 23:28 — bmad-prd — PRD working mode

**Q:** How do you want to build the PRD? (Stakes from the brief: passionate research project, open source later.)

**Options:**
- Fast path: draft full PRD from docs + brief with [ASSUMPTION] tags
- Coaching path, Vision + Features entry
- Coaching path, Journey-led entry
- Coaching path, agent suggests entry point

**Choice:** Fast path

## 2026-10-30 23:28 — bmad-prd — PRD extra inputs

**Q:** Any input beyond docs/mbsc_architecture_review.md and the existing product brief?

**Options:**
- No, those two are everything
- Yes, more to paste or point to

**Choice:** No, those two are everything

## 2026-10-30 23:30 — bmad-prd — v1 predictor families

**Q:** Which predictor families must be in the v1 codebook?

**Options:**
- All three: polynomial, trigonometric, recurrence (plus constant and previous-byte baselines)
- Polynomial and trigonometric only
- Polynomial only first
- Agent infers, tags [ASSUMPTION]

**Choice:** All three: polynomial, trigonometric, recurrence (plus baselines)

## 2026-10-30 23:30 — bmad-prd — v1 real input classes

**Q:** Which real input classes should the v1 success criterion target first?

**Options:**
- Text and PCM, with executables and already-compressed data as controls
- Text, PCM and executables all count as candidate classes
- Agent infers, tags [ASSUMPTION]

**Choice:** Text, PCM and executables all count as candidate classes

## 2026-10-30 23:30 — bmad-prd — v1 CLI behaviour

**Q:** CLI behaviour for v1:

**Options:**
- File arguments plus stdin/stdout, integrity checksum, non-zero exit on error
- File arguments only, minimal
- Agent infers, tags [ASSUMPTION]

**Choice:** File arguments only, minimal

## 2026-10-30 23:45 — release-manager — Second release - version

**Q:** Default patch bump 0.0.1 to 0.0.2. Confirm?

**Options:**
- 0.0.2 (patch, default)
- 0.1.0 (minor)
- No version bump

**Choice:** 0.0.2 (patch, default)

**Notes:** patch is the default bump; used without asking

## 2026-10-30 23:45 — release-manager — Second release - commit message

**Q:** Short or Long commit message?

**Options:**
- Short
- Long

**Choice:** Long

**Notes:** Long is the default; used without asking

## 2026-10-30 23:47 — bmad-architecture — Architecture working mode

**Q:** How do you want to work the spine?

**Options:**
- Coaching path (recommended)
- Fast path

**Choice:** Coaching path (default applied after clarification; "B+C" was read as deliverables B and C)

**Notes:** mode not explicitly picked; skill default is Coaching

## 2026-10-30 23:47 — bmad-architecture — Architecture deliverables

**Q:** Is the spine the only deliverable?

**Options:**
- Spine only
- Spine plus interactive HTML/SVG walkthrough
- Spine plus Bitstream/Evaluator spec seed outline (FR-11)
- Something else

**Choice:** Spine plus interactive HTML/SVG walkthrough plus Bitstream/Evaluator spec seed outline

**Notes:** multi-select (B+C)

## 2026-10-30 23:58 — bmad-architecture — Paradigm

**Q:** Paradigm: Normative Kernel + Encoder Shell, encoder-side depends on kernel, never reverse?

**Options:**
- Accept (recommended)
- Different framing

**Choice:** Accept

## 2026-10-30 23:58 — bmad-architecture — Implementation language (OQ3)

**Q:** Implementation language?

**Options:**
- Rust (recommended)
- C
- Python prototype first, then port
- Other

**Choice:** Rust

## 2026-10-30 23:58 — bmad-architecture — rANS ownership

**Q:** rANS ownership?

**Options:**
- Hand-written, spec-defined rANS; crate only as test oracle (recommended)
- Depend on a crate and document its behaviour in the spec

**Choice:** Hand-written, spec-defined rANS

## 2026-10-31 00:05 — bmad-architecture — Probability model (OQ2)

**Q:** Which Probability model for v1?

**Options:**
- C: adaptive order-0, reset per Block, no signalling (recommended)
- A: transmitted quantized histogram per Block
- B: parametric Laplace with transmitted scale
- Push back / combine

**Choice:** C: adaptive order-0, reset per Block, no signalling

## 2026-10-31 00:05 — bmad-architecture — Block independence invariant

**Q:** Adopt reset-per-Block so Block cost is additive and each Block independently decodable?

**Options:**
- Yes (recommended)
- No, allow state to carry across Blocks

**Choice:** Yes

## 2026-10-31 00:12 — bmad-architecture — rANS stream layout

**Q:** Stream layout?

**Options:**
- One self-contained rANS stream per Block, exact additive cost (recommended)
- One rANS stream for the whole file, approximate Block cost

**Choice:** One self-contained rANS stream per Block

## 2026-10-31 00:12 — bmad-architecture — Payload length

**Q:** Payload length per Block?

**Options:**
- Derived from decoding, not stored (recommended)
- Stored explicitly in the Block header

**Choice:** Derived from decoding, not stored

## 2026-10-31 00:18 — bmad-architecture — Block partition rule (OQ6)

**Q:** Partition rule?

**Options:**
- P1: free DP on 256-byte grid, 2-bit length code (recommended)
- P2: aligned split tree with split flags
- P3: greedy

**Choice:** P1: free DP on 256-byte grid

## 2026-10-31 00:18 — bmad-architecture — Block length set and final-Block rule

**Q:** Block length set and final-Block rule?

**Options:**
- {256,1024,2048} + reserved; actual=min(nominal,remaining); smallest covering length for last Block (recommended)
- Different lengths or rule

**Choice:** {256,1024,2048} + reserved; actual=min(nominal,remaining)

## 2026-10-31 00:30 — bmad-architecture — Series family definitions

**Q:** Formulas positionwise? Polynomial basis? Trig form? Recurrence form?

**Options:**
- Positionwise yes/no
- Polynomial: monomial / Chebyshev / both
- Trig: order M + base frequency via sine LUT with linear interpolation / different
- Recurrence: first-order / second-order / both with order field / other

**Choice:** Positionwise yes; monomial; trig as proposed; recurrence both with an order field

## 2026-10-31 00:40 — bmad-architecture — Evaluator arithmetic

**Q:** Accept Evaluator arithmetic as proposed (i64, no-overflow-by-construction, round half up, rem_euclid 256, Q per Formula in 5 bits, no FP)?

**Options:**
- Accept (recommended)
- Change something

**Choice:** Accept

## 2026-10-31 00:40 — bmad-architecture — Numeric bounds (OQ9)

**Q:** Accept numeric bounds (N<2^35, N+7 bound, ~64 KiB decoder memory, Codebook cap 4096)?

**Options:**
- Accept all (recommended)
- Change some

**Choice:** Accept all

## 2026-10-31 00:52 — bmad-architecture — Structure, conventions and verification

**Q:** Accept: four-crate workspace with zero-dep kernel; LE ints + LSB-first bits + byte-aligned Blocks; deterministic parallel encoder; in-repo Python reference decoder; TOML corpus manifest with hash-verified fetch?

**Options:**
- Accept (recommended) / Change, per item

**Choice:** Accept all five

## 2026-10-31 01:20 — bmad-architecture — Spine finalized

**Q:** (no user question) Reviewer gate and advisor pass applied the following revision without changing any user-approved choice: AD-7 realised as a bit-tree (fixed 4096 rANS total) instead of a variable-total count model, because rANS cannot invert a changing total.

**Choice:** applied; flagged to the user in the closing summary

**Notes:** spine status final; companions: bitstream-spec-seed.md, walkthrough.html

## 2026-10-31 07:15 — release-manager — Third release - version

**Q:** Default patch bump 0.0.2 to 0.0.3. Confirm?

**Options:**
- 0.0.3 (patch, default)
- 0.1.0 (minor)
- No version bump

**Choice:** 0.0.3 (patch, default)

**Notes:** patch is the default bump; used without asking

## 2026-10-31 07:15 — release-manager — Third release - commit message

**Q:** Short or Long commit message?

**Options:**
- Short
- Long

**Choice:** Long

**Notes:** Long is the default; used without asking
