---
name: workflow-decision-log
description: >-
  Logs every question the agent asks the user, the options/possibilities
  presented, and the eventual choice made, into an append-only markdown
  decision log bucketed by ISO 8601 calendar week. Use automatically whenever
  presenting the user with structured choices (AskQuestion) or discrete options
  during any multi-step workflow (BMad skills, planning flows, release prep,
  etc.), recording each question, its options, and the selected answer
  immediately after the user responds.
---

# Workflow Decision Log

Maintain an append-only record of every decision point in a workflow: the
**question** asked, the **possibilities** offered, and the **choice** the user
made. This builds an auditable trail of how a workflow's direction was decided.

## When to log

Log an entry **every time** you ask the user to choose between options and they
respond. This includes:

- Any `AskQuestion` tool call (one entry per question in the call).
- Any time you present discrete options in prose and the user picks one.
- Workflow routing/branch decisions in multi-step skills (e.g. BMad).

Log **after** the user answers, so the entry captures the actual choice. If the
user picks a custom/"Other" answer, record their exact wording as the choice.

## Where to log

Logs are bucketed by **ISO 8601 calendar week**, one file per week, under
`{project-root}/docs/decision_logs/`.

**Naming convention (required):**

```
docs/decision_logs/decision_log_<ISO_YEAR>_W<WW>.md
```

- `<ISO_YEAR>` is the ISO week-numbering year, `<WW>` is the zero-padded ISO week
  (`01`–`53`). Example: a decision made on 2026-06-19 lands in
  `docs/decision_logs/decision_log_2026_W25.md`.
- Use the ISO **week-numbering year**, not the calendar year — they differ for a
  few days around Jan 1 (the helper script handles this via `datetime.isocalendar()`).
- Each weekly file is append-only and gets a `# Workflow Decision Log — <YYYY> W<WW>`
  header on first write.

Notes:

- Avoid `.cursor/` — Cursor's agent sandbox protects that directory, so writes
  there require elevated permission on every turn. Keep logs under `docs/`.
- A relative `log_dir` is anchored to the **project root** — the nearest ancestor
  holding `.git`, `pyproject.toml`, or `package.json` — so a command run from a
  subdirectory still writes to the one canonical log. Absolute paths are used as
  given. If no marker is found, the current working directory is the fallback.
- If a repo keeps no `docs/` tree, pass a `log_dir` that matches its conventions
  (e.g. `.decision_logs`); the script creates the directory on first write.
- The correct weekly file is chosen automatically from the entry's date, so a
  long session that crosses a week boundary rolls over on its own.

## How to log

Prefer the helper script — it computes the correct weekly file, appends
atomically, and creates the file with a header on first use. Pass a JSON payload
on stdin:

```bash
python3 .cursor/skills/workflow-decision-log/scripts/log_decision.py <<'JSON'
{
  "log_dir": "docs/decision_logs",
  "workflow": "bmad-product-brief",
  "entries": [
    {
      "context": "Discovery — choosing brief scope",
      "question": "Which brief do you want me to facilitate?",
      "options": ["Product brief", "Spike brief", "Both", "Not sure"],
      "choice": ["Both"],
      "notes": "spike framed as Milestone 0"
    }
  ]
}
JSON
```

`log_dir`, `workflow`, `context`, and `notes` are optional (`log_dir` defaults to
`docs/decision_logs`); `question`, `options`, and `choice` are the core fields.
`options` and `choice` may be a string or a list. An optional `timestamp`
(`YYYY-MM-DD HH:MM`, or just `YYYY-MM-DD`) overrides "now" and selects the weekly
file. A single entry may be passed at the top level instead of `entries`. An
explicit `log_path` is an escape hatch that overrides the weekly-file computation
entirely — use it only when a caller must target one exact file.

**Match the host project's toolchain.** In a repo standardized on uv, run
`uv run python <script>`; with a project virtualenv, use that interpreter. The
script is dependency-free and stdlib-only, so bare `python3` works anywhere.

This repository's copy is `.cursor/skills/workflow-decision-log/scripts/log_decision.py`.
Invoke that path from the repository root. It is the copy committed alongside
this repo's logs. Use `~/.cursor/skills/workflow-decision-log/scripts/log_decision.py`
only when that project file is absent.

## Backfill (migrate existing flat logs)

When a legacy flat log exists (e.g. a single `decision-log.md`), migrate it into
the weekly files — entries are routed by their own timestamps, idempotently
(existing sections are not duplicated):

```bash
python3 .cursor/skills/workflow-decision-log/scripts/log_decision.py --backfill decision-log.md
```

- Every dated `## ` or `### ` section is appended to its ISO-week file; the leading
  timestamp may be `<YYYY-MM-DD HH:MM>` or date-only `<YYYY-MM-DD>`. Undated prose
  following a dated section travels with it; anything before the **first** dated
  section (a title, preamble) is reported and left for you to copy by hand.
- The legacy file is rewritten as a pointer listing the weekly files it fed into.
  Pass `--keep-legacy` to leave the legacy file untouched, or `--log-dir <dir>`
  to target a non-default directory.
- Re-running is safe: sections already present in a weekly file are skipped.

### Fallback (no script execution)

If you cannot run the script, append this block directly to the correct weekly
file `docs/decision_logs/decision_log_<ISO_YEAR>_W<WW>.md`, creating it with a
`# Workflow Decision Log — <YYYY> W<WW>` header if it does not exist:

```markdown
## <YYYY-MM-DD HH:MM> — <workflow> — <context>

**Q:** <question>

**Options:**
- <option 1>
- <option 2>

**Choice:** <selected option(s) / verbatim custom answer>

**Notes:** <optional>
```

## Principles

- Append only — never rewrite or delete prior entries.
- One entry per question, even when a single `AskQuestion` call asks several.
- Record the choice verbatim, including custom "Other" text.
- Never log secrets or personal/patient data — record the shape of the decision,
  not sensitive payloads.
- Logging is a side effect: do it quietly and continue the workflow. Do not
  interrupt the user to announce each log write.
