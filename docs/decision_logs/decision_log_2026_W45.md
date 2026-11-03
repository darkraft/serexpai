# Workflow Decision Log — 2026 W45

Append-only record of questions asked, options offered, and choices made.
ISO 8601 calendar week 45 of 2026.

## 2026-11-02 04:09 — release-manager — v0.0.12 version bump

**Q:** Which version bump should this release use?

**Options:**
- Patch 0.0.11 to 0.0.12 (default)
- Minor 0.0.11 to 0.1.0

**Choice:** Patch 0.0.11 to 0.0.12 (default)

**Notes:** Binary range coder in the kernel only. Bitstream format version stays 1. Workspace Cargo.toml, Cargo.lock, README, and CHANGELOG carry 0.0.12. The CLI has no --version.

## 2026-11-02 04:09 — release-manager — Commit message for v0.0.12

**Q:** Which commit message should this release use?

**Options:**
- Short: feat: code decisions with the binary range coder (v0.0.12)
- Long: feat: code decisions with the binary range coder (v0.0.12) plus body covering exact size, pinned vectors, truncated refill, and the unchanged bitstream layout

**Choice:** Long: feat: code decisions with the binary range coder (v0.0.12) plus body covering exact size, pinned vectors, truncated refill, and the unchanged bitstream layout

**Notes:** Default long message. User ordered the release and a push.

## 2026-11-02 22:58 — release-manager — Version bump for the order-0 model release

**Q:** Which version bump should this release use?

**Options:**
- Patch 0.0.12 to 0.0.13 (default)
- Minor 0.0.12 to 0.1.0

**Choice:** Patch 0.0.12 to 0.0.13 (default)

**Notes:** User did not ask for a minor bump. Bitstream layout is unchanged.

## 2026-11-02 22:58 — release-manager — Commit message for v0.0.13

**Q:** Which commit message should the release use?

**Options:**
- Short
- Long (default)

**Choice:** Long (default)

**Notes:** User invoked release-manager and did not ask for Short.

## 2026-11-03 10:16 — release-manager — Version bump for the baseline-block release

**Q:** Which version bump should this release use?

**Options:**
- Patch 0.0.13 to 0.0.14 (default)
- Minor 0.0.13 to 0.1.0

**Choice:** Patch 0.0.13 to 0.0.14 (default)

**Notes:** User did not ask for a minor bump. Blocked mode is implemented for four baseline kinds. Format version stays 1. Workspace Cargo.toml, Cargo.lock, README, and CHANGELOG carry 0.0.14. The CLI has no --version.

## 2026-11-03 10:16 — release-manager — Commit message for v0.0.14

**Q:** Which commit message should this release use?

**Options:**
- Short: feat: round-trip one baseline block (v0.0.14)
- Long: feat: round-trip one baseline block (v0.0.14) plus body covering the 5-bit header, per-block model reset, and format version 1

**Choice:** Long: feat: round-trip one baseline block (v0.0.14) plus body covering the 5-bit header, per-block model reset, and format version 1

**Notes:** Default long message. User ordered the release and a push.

## 2026-11-03 12:17 — release-manager — Version bump for the baseline-selection release

**Q:** Which version bump for the working-tree release since 0.0.14?

**Options:**
- Patch 0.0.14 to 0.0.15 (default)
- Minor 0.0.14 to 0.1.0

**Choice:** Patch 0.0.14 to 0.0.15 (default)

**Notes:** User invoked release-manager and ordered a push. The encoder chooses existing baseline blocks. Format version stays 1. Workspace Cargo.toml, Cargo.lock, README, and CHANGELOG carry 0.0.15. The CLI has no --version.

## 2026-11-03 12:17 — release-manager — Commit message for v0.0.15

**Q:** Which commit message should the v0.0.15 release use?

**Options:**
- Short: feat: select baselines by exact size (v0.0.15)
- Long: feat: select baselines by exact size (v0.0.15) plus body covering exact block size, the partition key, the whole-file raw tie, and format version 1

**Choice:** Long: feat: select baselines by exact size (v0.0.15) plus body covering exact block size, the partition key, the whole-file raw tie, and format version 1

**Notes:** User invoked release-manager and ordered a push. Long is the skill default.

## 2026-11-03 13:35 — bmad-code-review — Story 1.10 review — how to handle patch findings

**Q:** How should the seven patch findings be handled?

**Options:**
- Apply every patch
- Leave as action items
- Walk through each patch

**Choice:** Apply every patch

**Notes:** The opening request was CS, DS and CR 1.10 until everything is fixed.

## 2026-11-03 13:56 — release-manager — Version bump for the story 1.10 working-tree release

**Q:** Which version bump should v0.0.16 use?

**Options:**
- Patch 0.0.15 to 0.0.16 (default)
- Minor 0.0.15 to 0.1.0

**Choice:** Patch 0.0.15 to 0.0.16 (default)

**Notes:** User invoked release-manager and ordered a push. Formula blocks and fixture codebook search land. Format version stays 1. Workspace Cargo.toml, Cargo.lock, README, and CHANGELOG carry 0.0.16. The CLI has no --version.

## 2026-11-03 13:56 — release-manager — Commit message for v0.0.16

**Q:** Which commit message should the v0.0.16 release use?

**Options:**
- Short: feat: search a regenerable formula codebook (v0.0.16)
- Long: feat: search a regenerable formula codebook (v0.0.16) plus body covering formula framing, fixture search, the baseline tie, and format version 1

**Choice:** Long: feat: search a regenerable formula codebook (v0.0.16) plus body covering formula framing, fixture search, the baseline tie, and format version 1

**Notes:** Long is the release-manager default. User did not ask for Short.

## 2026-11-03 15:57 — bmad-code-review — Story 1.11 review — aarch64 evidence before patches

**Q:** aarch64 CI has not run. Waive it and apply the eight review patches, or stop until the cross toolchain is installed?

**Options:**
- Waive aarch64, apply the eight patches, and leave the aarch64 acceptance unchecked
- Stop until gcc-aarch64-linux-gnu and qemu-user are installed

**Choice:** Installed gcc-aarch64-linux-gnu and qemu-user, then ran QEMU_LD_PREFIX=/usr/aarch64-linux-gnu ./ci.sh aarch64-unknown-linux-gnu. That pre-patch run passed. Apply the eight patches and re-run both targets.

**Notes:** The loader is at /usr/aarch64-linux-gnu/lib/ld-linux-aarch64.so.1, so qemu needs QEMU_LD_PREFIX. Do not mark the story done on the pre-patch pass.

## 2026-11-03 16:20 — bmad-code-review — Story 1.11 review complete

**Q:** What would you like to do next?

**Options:**
- Start the next story — run dev-story for the next ready-for-dev story
- Re-run code review
- Done — end the workflow

**Choice:** Done — end the workflow

## 2026-11-03 16:16 — release-manager — Version bump for the file-command release

**Q:** Which version bump should v0.0.17 use?

**Options:**
- Patch 0.0.16 to 0.0.17 (default)
- Minor 0.0.16 to 0.1.0

**Choice:** Patch 0.0.16 to 0.0.17 (default)

**Notes:** User invoked release-manager and ordered a push. compress and decompress land. Format version stays 1. Workspace Cargo.toml, Cargo.lock, README, and CHANGELOG carry 0.0.17. The CLI has no --version.

## 2026-11-03 16:16 — release-manager — Commit message for v0.0.17

**Q:** Which commit message should the v0.0.17 release use?

**Options:**
- Short: feat: compress and decompress a file (v0.0.17)
- Long: feat: compress and decompress a file (v0.0.17) plus body covering rename install, fixture-v1 compress, and format version 1

**Choice:** Long: feat: compress and decompress a file (v0.0.17) plus body covering rename install, fixture-v1 compress, and format version 1

**Notes:** Long is the default. The user invoked release-manager and ordered a push, which approves the release commit.
