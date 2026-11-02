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
