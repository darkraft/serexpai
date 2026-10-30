---
name: release-manager
description: >-
  Coordinates a SerExpAI (MBSC compressor) release: syncs docs with the
  codebase, runs the project's lint/format/test gate once a toolchain exists,
  writes CHANGELOG entries from the chosen commit range, bumps 0.n.m versions
  (patch by default, minor only when the user asks), proposes exactly two commit
  messages and commits with the Long one by default, stages only
  release-relevant paths with .gitignore / secret / corpus discipline, commits
  to the active branch AND appends a paired dual-record commit (staged files
  plus all BMAD modelling artifacts, including any gitignored _bmad-output/) to
  the append-only serexpai-bmad branch, and never pushes unless explicitly
  ordered. Use when the user asks for a release, release prep, cutting a
  version, changelog update, docs alignment, or "release manager" workflow.
---

# Release manager (SerExpAI)

Adapted from the Levitality release-manager. FastAPI, Ruff, uv, PHI and Ralph specifics were removed or turned into placeholders because this repo has no code, language or toolchain yet. Items marked **[TOOLCHAIN TBD]** must be filled in once architecture picks the CLI language.

## Scope

Orchestrate a **local** release prep: align documentation with the repo contents, run the quality gate (when one exists), produce a changelog for the chosen range, bump **0.n.m** versions, output **exactly two** commit message propositions, stage only appropriate paths with **.gitignore**, **secret**, and **corpus** discipline, **commit** when instructed, and **never `git push`** unless the user explicitly orders a push.

Every commit is a **double record**: the ordinary commit lands on the currently active branch (respecting `.gitignore`), and a paired **record commit** lands on the append-only **`serexpai-bmad`** branch containing everything in the active-branch commit **plus all BMAD modelling artifacts** (`_bmad-output/` planning docs, `_bmad/`, `.agents/`, `docs/decision_logs/`). This preserves modelling history even if the code branch ignores those paths. See [reference.md](reference.md) for the plumbing recipe. The record branch is built through a **temporary index** and never needs a checkout, stash, or working-tree change.

## Preconditions

- Working tree changes reflect what should be documented (or note gaps in the changelog).
- Never stage secrets or large/proprietary benchmark corpora (see [reference.md](reference.md)).
- **[TOOLCHAIN TBD]** Use the project's chosen package/build tool from the repo root. Until one is chosen, steps 4 and 6 are skipped and the skip is stated in the output.
- **No commits yet / no `.gitignore` yet:** this repo started with no commits and no `.gitignore`. On the first run, state both facts and ask once whether `_bmad-output/`, `_bmad/` and `.agents/` should be gitignored on the code branch. If yes, the dual record is what preserves them. If no, they are tracked normally and the overlay is a harmless no-op. Do not create `.gitignore` without that answer.

## Workflow (ordered)

### 1. Baseline

Decide what "since last release" means, and confirm if unclear:

- **First release (no `HEAD` yet, the current state):** everything staged or untracked that belongs in the repo. There is no previous version, so there is no "since" range.
- **Working-tree release (default once commits exist):** all **staged and unstaged** changes vs current `HEAD`.
- **Multi-commit range:** `git log <ref>..HEAD` where `<ref>` is `main`, `origin/main`, the last tag, or a SHA the user names.
- **Exactly the last commit's scope:** `git show HEAD` (single commit only).

### 2. Inventory changes

Run `git log`, `git diff`, and `git status` for the chosen range (on the first release, `git status` and a file listing). Classify items as **features**, **bugfixes**, **docs**, **chore**, **breaking**. Note new modules, new tests, new BMAD artifacts (briefs, PRD, architecture) and any change to the bitstream format. A bitstream change always gets a prominent changelog line.

### 3. Documentation vs repo contents

Reconcile high-signal docs with what actually exists:

| Area | Typical paths |
|------|----------------|
| Source review / spec input | `docs/mbsc_architecture_review.md` |
| Product brief and addendum | `_bmad-output/planning-artifacts/briefs/*/brief.md`, `addendum.md` |
| PRD, architecture, epics (when they exist) | `_bmad-output/planning-artifacts/` |
| Decision logs | `docs/decision_logs/` |
| CLI usage / run instructions | `README.md` (when it exists) |
| Bitstream specification (when it exists) | to be set once published |

**Method:**

1. List what exists in the areas above; do not invent documents that don't exist yet.
2. Update prose only where it is **wrong or misleading** (stale status, renamed files, answered open questions). Prefer minimal edits aligned with existing tone.
3. When a brief or PRD changes status, make sure its frontmatter (`status`, `updated`) matches.
4. When a CLI exists, keep its documented commands and flags in sync with the actual `--help` output.

### 4. Interface alignment **[TOOLCHAIN TBD]**

Replaces the Levitality FastAPI step. Once a CLI exists:

1. Verify documented subcommands (`compress`, `decompress`, benchmark) and flags against actual `--help` output.
2. Align version strings: the manifest file, the CLI `--version` output and the README line (see the table in [reference.md](reference.md)).
3. Check that the **bitstream version/magic** documented in the spec matches the code. A format change requires at least a documented decision on compatibility, since the decoder is normative.

Until a CLI exists, skip and say so.

### 5. CHANGELOG and README

**Changelog:** `docs/CHANGELOG.md` (Keep a Changelog style with `[Unreleased]`). Create it on the first release if missing.

1. Under `[Unreleased]`, add entries in **Added / Changed / Fixed / Removed / Security** derived from the range chosen in steps 1–2.
2. On **release cut** (only when the user requests): move `[Unreleased]` into `## [x.y.z] - YYYY-MM-DD` and leave a fresh empty `[Unreleased]`.

**README:** root `README.md` (when it exists) stays consistent with the version, build and run commands, and links to the docs.

### 6. Quality gate **[TOOLCHAIN TBD]**

Replaces the Levitality Ruff step. Once a toolchain exists, define here the exact commands for: lint, format, tests, and the **round-trip check** (compress then decompress is byte-identical on the test corpus). Lossless round-trip is a release blocker, not a warning.

**Done when:** every defined command exits **0**. Until a toolchain exists, skip, and state "no quality gate defined yet" in the output. Never claim a gate passed that was not run.

### 7. Version bump — patch by default

Whenever you will **change the published version** (release cut, tag, or syncing version strings with a new `x.y.z`), **default to a patch bump** and do **not** ask:

| Choice | Effect (example from `0.1.35`) | When |
|--------|-------------------------------|------|
| **Patch** (third segment) — **default** | `0.n.m` → `0.n.m+1` — e.g. `0.1.35` → `0.1.36` | Always, unless the user says otherwise |
| **Minor** (second segment) | `0.n.m` → `0.n+1.0` — e.g. `0.1.35` → `0.2.0` | Only when the user explicitly asks for a minor bump (e.g. "minor", "0.2.0") |

**No version exists yet (first release):** treat the current version as `0.0.0`, so the default first release is **`0.0.1`**. Confirm this once, then follow the default behaviour afterwards.

Do **not** infer a minor bump from changelog wording; breaking-looking changes, including bitstream format changes, still default to patch unless the user asks. Announce the chosen version in the output. If the user asked for docs/changelog **without** a bump, skip this step and leave version strings unchanged. Apply the bump to every location in the version table in [reference.md](reference.md), then log it per the `workflow-decision-log` skill (choice: patch, noted as the default).

### 8. Two commit messages

Produce **exactly two** propositions — **Short** (conventional one-liner) and **Long** (subject plus body bullets) — both grounded in the changelog bullets (features + bugfixes). Do not add a third option in the same output block. **Long is the default.**

### 9. User choice

**Default to the Long message** and do not ask. Use Short only if the user asks for it (e.g. "use Short") or supplies their own line. Show both propositions, state which one is used, and log the choice (noted as the default) per the `workflow-decision-log` skill before committing. The user's approval of the staged set is still required (step 11).

### 10. Staging

Include (as applicable): `docs/`, source and test directories once they exist, `README.md`, `docs/CHANGELOG.md`, the version manifest, and any files reformatted by the formatter.

On a repo with **no `.gitignore` answer yet** (see Preconditions), include `_bmad-output/`, `_bmad/` and `.agents/` only if the user chose to track them.

Exclude: `.env`, secrets, large or proprietary benchmark corpora, compressed benchmark outputs, build artifacts, local-only scratch.

See [reference.md](reference.md) for `.gitignore` handling, `git add -n` dry runs, and the secret/corpus scan. Never `git add -f` an ignored file unless the user explicitly requests it and understands why. **Sole standing exception:** the dual-record step force-adds the enumerated BMAD paths into its *temporary index only* (never the real index).

### 11. Commit (active branch)

`git commit` only after the user approves both the message and the staged set. Use the single chosen message. The first release creates the initial commit on `main`.

### 12. Dual record (`serexpai-bmad`)

Immediately after the active-branch commit succeeds, append the paired record commit via the temporary-index recipe in [reference.md](reference.md): tree = the just-made commit's tree **overlaid with** the BMAD artifact paths; parents = previous `serexpai-bmad` head (when the branch exists) **and** the active-branch commit. Message = the same subject prefixed `bmad-record:` plus a `Paired-With: <sha>` trailer.

The secret scan applies to the overlay too, since planning docs sometimes quote config. Run this step by default; skip only if the user opts out for a given release. **Never rebase, amend, or force-push `serexpai-bmad`** — it is an append-only record.

### 13. Push

**Do not** `git push` unless the user clearly says to push (e.g. "push to origin"). The configured remote is `origin` (`https://github.com/darkraft/serexpai`). When a push is ordered, push **both** refs: the active branch and `serexpai-bmad`.

## Outputs (every run)

1. Proposed changelog section (markdown).
2. New version string and the files touched for the bump.
3. **Two** labeled commit message propositions (Short, Long).
4. Recommended `git add` paths (or "already staged") and any paths to **avoid**.
5. Quality-gate status (each command's exit code, or "no quality gate defined yet").
6. Both SHAs of the double record — active-branch commit and its paired `serexpai-bmad` record commit — with the overlay paths that differed between them.

## Checklist

```
Release prep:
- [ ] First run only: no-HEAD / no-.gitignore situation stated; gitignore question answered
- [ ] Baseline range agreed (first release / working tree / range / last commit)
- [ ] Changes inventoried and classified
- [ ] Docs match actual repo contents; brief/PRD frontmatter status current
- [ ] CLI docs match --help and bitstream version (or "no CLI yet")
- [ ] docs/CHANGELOG.md (+ README) updated
- [ ] Quality gate run and exit 0 (or "no gate defined yet"); round-trip check included once code exists
- [ ] If version changes: patch bump by default (first release 0.0.1; minor only on explicit request); logged
- [ ] Short + Long proposals shown; Long used by default (Short only on request); choice logged
- [ ] Only intentional files staged; no secrets, no corpora
- [ ] Active-branch commit made only after approval
- [ ] Paired serexpai-bmad record commit appended (or explicitly skipped)
- [ ] No push unless explicitly ordered (then both refs)
```

## Additional resources

- Staging, secret/corpus heuristics, dual-record plumbing, version locations: [reference.md](reference.md)
