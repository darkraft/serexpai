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
