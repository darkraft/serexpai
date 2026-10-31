#!/usr/bin/env python3
"""Append workflow decision entries to per-week markdown decision logs.

Logs are bucketed by **ISO 8601 calendar week**. The file for a given date is:

    <log_dir>/decision_log_<ISO_YEAR>_W<WW>.md      e.g. decision_log_2026_W25.md

Default `log_dir` is `docs/decision_logs/`, resolved against the project root (the
nearest ancestor containing `.git`, `pyproject.toml`, or `package.json`) so the log
lands in the same place whether the command runs from the root or a subdirectory.
Absolute paths are used as given. Each weekly file is append-only and is created
with a header on first use.

USAGE
-----

1) Append (reads a JSON payload from stdin):

    python3 log_decision.py <<'JSON'
    {
      "log_dir": "docs/decision_logs",          # optional (default shown)
      "workflow": "bmad-product-brief",         # optional
      "timestamp": "2026-06-19 00:12",          # optional; defaults to now
      "entries": [
        {
          "context": "Discovery — choosing scope",
          "question": "Which brief?",
          "options": ["Product", "Spike", "Both"],
          "choice": ["Both"],
          "notes": "optional"
        }
      ]
    }
    JSON

   A single entry may be passed at the top level instead of `entries`.
   `options` and `choice` accept either a string or a list of strings.
   `log_path` (a full file path) is still honored as a back-compat escape hatch;
   when set it overrides the weekly-file computation.

2) Backfill an existing flat log into weekly files:

    python3 log_decision.py --backfill <legacy.md> \
        [--log-dir docs/decision_logs] [--keep-legacy]

   Splits the legacy file on its dated `## ` / `### ` sections, routes each to the
   weekly file for its leading `YYYY-MM-DD` or `YYYY-MM-DD HH:MM` timestamp
   (idempotent — existing sections are not duplicated), and rewrites the legacy file
   as a pointer to the weekly files unless `--keep-legacy` is given.

   Undated prose that follows a dated section travels with that section. Anything
   before the *first* dated section is not migrated and is reported instead.
"""

import json
import os
import re
import sys
from datetime import datetime

DEFAULT_LOG_DIR = "docs/decision_logs"
TS_FORMAT = "%Y-%m-%d %H:%M"
DATE_FORMAT = "%Y-%m-%d"
# Matches section headers: "## 2026-06-19 00:12 — …" or "### 2026-06-30 — …"
HEADER_TS_RE = re.compile(r"^#{2,3}\s+(\d{4}-\d{2}-\d{2}(?: \d{2}:\d{2})?)")
SECTION_START_RE = re.compile(r"^#{2,3}\s+\d{4}-\d{2}-\d{2}")
ROOT_MARKERS = (".git", "pyproject.toml", "package.json")


def as_list(value):
    if value is None:
        return []
    if isinstance(value, (list, tuple)):
        return [str(v) for v in value]
    return [str(value)]


def project_root(start=None):
    """Nearest ancestor of `start` holding a project marker, else `start` itself."""
    current = os.path.abspath(start or os.getcwd())
    while True:
        if any(os.path.exists(os.path.join(current, m)) for m in ROOT_MARKERS):
            return current
        parent = os.path.dirname(current)
        if parent == current:
            return os.path.abspath(start or os.getcwd())
        current = parent


def resolve_path(path):
    """Anchor a relative log path to the project root; absolute paths pass through."""
    if os.path.isabs(path):
        return path
    return os.path.join(project_root(), path)


def display_path(path):
    """Shorten a path for messages and pointer files; keeps absolute if outside cwd."""
    relative = os.path.relpath(path, os.getcwd())
    return path if relative.startswith(os.pardir) else relative


def week_filename(dt):
    """ISO calendar-week file name for a datetime (uses ISO year, not calendar year)."""
    iso_year, iso_week, _ = dt.isocalendar()
    return f"decision_log_{iso_year:04d}_W{iso_week:02d}.md"


def week_header(dt):
    iso_year, iso_week, _ = dt.isocalendar()
    return (
        f"# Workflow Decision Log — {iso_year:04d} W{iso_week:02d}\n\n"
        "Append-only record of questions asked, options offered, and choices made.\n"
        f"ISO 8601 calendar week {iso_week:02d} of {iso_year:04d}.\n"
    )


def parse_dt(text):
    text = text.strip()
    try:
        return datetime.strptime(text[:16], TS_FORMAT)
    except (ValueError, TypeError):
        pass
    try:
        return datetime.strptime(text[:10], DATE_FORMAT)
    except (ValueError, TypeError):
        return None


def render_entry(entry, workflow, timestamp):
    context = entry.get("context") or ""
    title_bits = [timestamp]
    if workflow:
        title_bits.append(str(workflow))
    if context:
        title_bits.append(str(context))
    lines = ["", "## " + " — ".join(title_bits), ""]

    question = (entry.get("question") or "").strip()
    lines.append("**Q:** " + (question or "(unspecified)"))
    lines.append("")

    options = as_list(entry.get("options"))
    lines.append("**Options:**")
    if options:
        lines.extend("- " + opt for opt in options)
    else:
        lines.append("- (none recorded)")
    lines.append("")

    choices = as_list(entry.get("choice"))
    lines.append(
        "**Choice:** " + (", ".join(choices) if choices else "(none recorded)")
    )

    notes = entry.get("notes")
    if notes:
        lines.append("")
        lines.append("**Notes:** " + str(notes))

    lines.append("")
    return "\n".join(lines)


def ensure_file(path, header):
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    if not os.path.exists(path):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(header)


def append_block(path, header, block_text, dedupe_header_line=None):
    """Append block_text to path, creating it with header if new.

    If dedupe_header_line is given and already present in the file, skip (idempotent).
    Returns True if written, False if skipped as duplicate.
    """
    ensure_file(path, header)
    if dedupe_header_line:
        with open(path, encoding="utf-8") as handle:
            if dedupe_header_line in handle.read():
                return False
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(block_text)
    return True


def leading_content(content):
    """Non-trivial lines before the first dated section — these are never migrated."""
    kept = []
    for line in content.splitlines():
        if SECTION_START_RE.match(line):
            break
        if line.strip() and not line.startswith("# "):
            kept.append(line)
    return kept


def split_sections(content):
    """Split a flat log into (header_line, block_text) tuples, one per dated section.

    Undated prose following a dated section stays with that section rather than being
    dropped, so migration never loses text that belongs to a decision.
    """
    sections = []
    cur = None
    for line in content.splitlines():
        if SECTION_START_RE.match(line):
            if cur is not None:
                sections.append(cur)
            cur = [line]
        elif cur is not None:
            cur.append(line)
    if cur is not None:
        sections.append(cur)
    out = []
    for block in sections:
        # Trim trailing blank lines, then frame with a leading + trailing newline.
        while block and not block[-1].strip():
            block.pop()
        out.append((block[0], "\n" + "\n".join(block) + "\n"))
    return out


def do_backfill(legacy_path, log_dir, keep_legacy):
    if not os.path.exists(legacy_path):
        sys.stderr.write(f"error: legacy log not found: {legacy_path}\n")
        return 1
    with open(legacy_path, encoding="utf-8") as handle:
        content = handle.read()

    sections = split_sections(content)
    if not sections:
        sys.stderr.write(f"error: no dated sections found in {legacy_path}\n")
        return 1

    preamble = leading_content(content)
    if preamble:
        sys.stderr.write(
            f"warning: {len(preamble)} line(s) before the first dated section are "
            "not migrated; copy them by hand if they matter\n"
        )

    log_dir = resolve_path(log_dir)
    written, skipped, undated = 0, 0, 0
    targets = []
    for header_line, block_text in sections:
        m = HEADER_TS_RE.match(header_line)
        dt = parse_dt(m.group(1)) if m else None
        if dt is None:
            undated += 1
            sys.stderr.write(
                f"warning: no timestamp in section, skipped: {header_line}\n"
            )
            continue
        path = os.path.join(log_dir, week_filename(dt))
        if append_block(
            path, week_header(dt), block_text, dedupe_header_line=header_line
        ):
            written += 1
        else:
            skipped += 1
        if path not in targets:
            targets.append(path)

    if not keep_legacy:
        pointer = ["# Workflow Decision Log (migrated)", ""]
        pointer.append(
            f"Entries migrated to ISO-week files under `{display_path(log_dir)}/`:"
        )
        pointer.append("")
        pointer.extend("- " + display_path(p) for p in sorted(targets))
        pointer.append("")
        with open(legacy_path, "w", encoding="utf-8") as handle:
            handle.write("\n".join(pointer))

    sys.stderr.write(
        f"backfill: {written} written, {skipped} already present, {undated} undated "
        f"across {len(targets)} weekly file(s)\n"
    )
    return 0


def do_append(payload):
    if not isinstance(payload, dict):
        sys.stderr.write("error: payload must be a JSON object\n")
        return 1

    workflow = payload.get("workflow")
    log_dir = payload.get("log_dir") or DEFAULT_LOG_DIR

    ts_text = payload.get("timestamp")
    dt = parse_dt(ts_text) if ts_text else None
    if dt is None:
        dt = datetime.now()
    timestamp = dt.strftime(TS_FORMAT)

    entries = payload.get("entries")
    if entries is None:
        entries = [payload]
    if not isinstance(entries, list) or not entries:
        sys.stderr.write("error: no entries to log\n")
        return 1

    # Back-compat: explicit log_path overrides weekly computation.
    log_path = payload.get("log_path") or os.path.join(log_dir, week_filename(dt))
    log_path = resolve_path(log_path)
    header = week_header(dt)

    ensure_file(log_path, header)
    with open(log_path, "a", encoding="utf-8") as handle:
        count = 0
        for entry in entries:
            if isinstance(entry, dict):
                handle.write(render_entry(entry, workflow, timestamp))
                count += 1

    sys.stderr.write(f"logged {count} decision(s) to {display_path(log_path)}\n")
    return 0


def main():
    argv = sys.argv[1:]
    if argv and argv[0] == "--backfill":
        if len(argv) < 2:
            sys.stderr.write("error: --backfill requires a legacy log path\n")
            return 1
        legacy_path = argv[1]
        log_dir = DEFAULT_LOG_DIR
        keep_legacy = False
        rest = argv[2:]
        i = 0
        while i < len(rest):
            if rest[i] == "--log-dir" and i + 1 < len(rest):
                log_dir = rest[i + 1]
                i += 2
            elif rest[i] == "--keep-legacy":
                keep_legacy = True
                i += 1
            else:
                sys.stderr.write(f"error: unknown arg: {rest[i]}\n")
                return 1
        return do_backfill(legacy_path, log_dir, keep_legacy)

    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError) as exc:
        sys.stderr.write(f"error: invalid JSON on stdin: {exc}\n")
        return 1
    return do_append(payload)


if __name__ == "__main__":
    sys.exit(main())
