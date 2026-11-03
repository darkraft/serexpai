"""Command line for the corpus fetch tool. Standard library only."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

import corpus


def _fail(code: str) -> int:
    print(code, file=sys.stderr)
    return 1


def _extract(args: argparse.Namespace) -> int:
    try:
        corpus.extract(
            Path(args.archive),
            args.kind,
            args.member,
            args.sha256,
            Path(args.out),
            args.md5,
        )
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except OSError:
        return _fail("BadManifest")
    return 0


def _pin(args: argparse.Namespace) -> int:
    try:
        archive_sha, member_sha = corpus.pin(
            Path(args.archive), args.kind, args.member, args.md5
        )
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except OSError:
        return _fail("BadManifest")
    sys.stdout.write(f"archive_sha256={archive_sha}\nmember_sha256={member_sha}\n")
    return 0


def _pcm(args: argparse.Namespace) -> int:
    wav = Path(args.wav).resolve()
    out = Path(args.out).resolve()
    raw_out = Path(args.raw_out).resolve()
    if len({wav, out, raw_out}) < 3:
        return _fail("BadManifest")
    wrote_out = False
    try:
        derived, raw = corpus.pcm16le_to_u8(wav)
        corpus._write_new(out, derived)
        wrote_out = True
        corpus._write_new(raw_out, raw)
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except OSError:
        if wrote_out:
            try:
                out.unlink(missing_ok=True)
            except OSError:
                pass
        return _fail("BadManifest")
    return 0


def _slice(args: argparse.Namespace) -> int:
    try:
        data = corpus.read_bounded(Path(args.input), corpus.MEMBER_MAX)
        result = corpus.slice_middle(data, args.length)
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except OSError:
        return _fail("BadManifest")
    body = result["bytes"]
    if not isinstance(body, bytes):
        return _fail("BadManifest")
    try:
        corpus._write_new(Path(args.out), body)
    except OSError:
        return _fail("BadManifest")
    sys.stdout.write(
        f"source_sha256={result['source_sha256']}\n"
        f"offset={result['offset']}\n"
        f"length={result['length']}\n"
        f"derived_sha256={result['derived_sha256']}\n"
    )
    return 0


def _select(args: argparse.Namespace) -> int:
    rows: list[tuple[str, int]] = []
    try:
        lines = Path(args.metadata).read_text(encoding="utf-8").splitlines()
        if lines == []:
            return _fail("BadManifest")
        for line in lines[1:]:
            if line == "":
                continue
            clip_id, raw_size = line.split(",", 1)
            rows.append((clip_id, int(raw_size)))
        taken, _total = corpus.select_pcm(rows, args.seed, args.budget, args.prefix)
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except (OSError, UnicodeDecodeError, ValueError):
        return _fail("BadManifest")
    for clip_id in taken:
        sys.stdout.write(clip_id + "\n")
    return 0


def _overlap(args: argparse.Namespace) -> int:
    try:
        hit = corpus.find_overlap(
            corpus.read_bounded(Path(args.tuning), corpus.MEMBER_MAX),
            corpus.read_bounded(Path(args.evaluation), corpus.MEMBER_MAX),
            args.k,
        )
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except OSError:
        return _fail("BadManifest")
    if hit is None:
        sys.stdout.write("miss\n")
        return 0
    sys.stdout.write(
        f"hit {hit['tuning_offset']} {hit['evaluation_offset']} {hit['length']} "
        f"{int(bool(hit['repeated_byte']))}\n"
    )
    return 0


def _gate(args: argparse.Namespace) -> int:
    try:
        manifest = tomllib.loads(Path(args.manifest).read_text(encoding="utf-8"))
        sys.stdout.write(corpus.canonical_json(corpus.gate(manifest)))
    except corpus.CorpusError as exc:
        return _fail(exc.code)
    except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError, ValueError, TypeError):
        return _fail("BadManifest")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="fetch.py")
    commands = parser.add_subparsers(dest="command", required=True)

    extract = commands.add_parser("extract")
    extract.add_argument("--archive", required=True)
    extract.add_argument("--kind", required=True)
    extract.add_argument("--member", required=True)
    extract.add_argument("--sha256", required=True)
    extract.add_argument("--md5")
    extract.add_argument("--out", required=True)
    extract.set_defaults(func=_extract)

    pin = commands.add_parser("pin")
    pin.add_argument("--archive", required=True)
    pin.add_argument("--kind", required=True)
    pin.add_argument("--member", required=True)
    pin.add_argument("--md5")
    pin.set_defaults(func=_pin)

    transform = commands.add_parser("transform")
    transform_commands = transform.add_subparsers(dest="transform", required=True)
    pcm = transform_commands.add_parser("pcm16le-to-u8")
    pcm.add_argument("--wav", required=True)
    pcm.add_argument("--out", required=True)
    pcm.add_argument("--raw-out", required=True)
    pcm.set_defaults(func=_pcm)
    sliced = transform_commands.add_parser("slice")
    sliced.add_argument("--input", required=True)
    sliced.add_argument("--length", required=True, type=int)
    sliced.add_argument("--out", required=True)
    sliced.set_defaults(func=_slice)

    select = commands.add_parser("select-pcm")
    select.add_argument("--metadata", required=True)
    select.add_argument("--seed", required=True)
    select.add_argument("--budget", required=True, type=int)
    select.add_argument("--prefix", action="append", required=True)
    select.set_defaults(func=_select)

    overlap = commands.add_parser("overlap")
    overlap.add_argument("--k", required=True, type=int)
    overlap.add_argument("tuning")
    overlap.add_argument("evaluation")
    overlap.set_defaults(func=_overlap)

    gate = commands.add_parser("gate")
    gate.add_argument("--manifest", required=True)
    gate.set_defaults(func=_gate)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    sys.exit(main())
