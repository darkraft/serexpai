"""Fixture tests for the fetch tool. No network."""

from __future__ import annotations

import hashlib
import io
import os
import socket
import stat
import subprocess
import sys
import tarfile
import tempfile
import unittest
import urllib.request
import wave
import zipfile
from pathlib import Path


def _bomb(*_args, **_kwargs):
    raise AssertionError("network")


urllib.request.urlopen = _bomb
socket.socket = _bomb
socket.create_connection = _bomb

sys.path.insert(0, str(Path(__file__).resolve().parent))

import corpus  # noqa: E402
import fetch  # noqa: E402

REPO = Path(__file__).resolve().parents[2]
MANIFEST = REPO / "corpora" / "manifest.toml"
GATE_REPORT = REPO / "corpora" / "gate-report.json"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_tar(members: list[tuple[object, bytes]], mode: str) -> bytes:
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode=mode) as handle:
        for item, payload in members:
            if isinstance(item, tarfile.TarInfo):
                handle.addfile(item, io.BytesIO(payload) if payload else None)
            else:
                info = tarfile.TarInfo(item)
                info.size = len(payload)
                handle.addfile(info, io.BytesIO(payload))
    return buffer.getvalue()


def ustar_header(name: str, size: int) -> bytes:
    header = bytearray(512)
    encoded = name.encode()
    header[: len(encoded)] = encoded

    def put_octal(at: int, width: int, value: int) -> None:
        text = f"{value:0{width - 1}o}".encode()
        header[at : at + width - 1] = text
        header[at + width - 1] = 0

    put_octal(100, 8, 0o644)
    put_octal(108, 8, 0)
    put_octal(116, 8, 0)
    put_octal(124, 12, size)
    put_octal(136, 12, 0)
    header[156] = ord("0")
    header[257:265] = b"ustar\x0000"
    header[148:156] = b"        "
    checksum = sum(header)
    header[148:156] = f"{checksum:06o}\0 ".encode()
    return bytes(header)


def build_ar(entries: list[tuple[str, bytes]]) -> bytes:
    out = bytearray(b"!<arch>\n")
    for name, blob in entries:
        header = bytearray(b" " * 60)
        header[0:16] = name.encode().ljust(16)[:16]
        header[48:58] = str(len(blob)).encode().ljust(10)[:10]
        header[58:60] = b"`\n"
        out += header
        out += blob
        if len(blob) % 2 == 1:
            out += b"\n"
    return bytes(out)


def build_deb(member: str, payload: bytes, data_name: str = "data.tar.xz") -> bytes:
    xz_bytes = write_tar([(member, payload)], "w:xz")
    return build_ar([("debian-binary", b"2.0\n"), (data_name, xz_bytes)])


def wav_bytes(samples: list[int], rate: int = 22050, channels: int = 1) -> bytes:
    buffer = io.BytesIO()
    with wave.open(buffer, "wb") as handle:
        handle.setnchannels(channels)
        handle.setsampwidth(2)
        handle.setframerate(rate)
        raw = b"".join(int(sample).to_bytes(2, "little", signed=True) for sample in samples)
        handle.writeframes(raw)
    return buffer.getvalue()


class FetchTests(unittest.TestCase):
    def test_tar_gz_extracts_only_the_named_member(self) -> None:
        payload = b"alice"
        data = write_tar([("alice29.txt", payload), ("other.txt", b"nope")], "w:gz")
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "c.tar.gz"
            out = Path(tmp) / "out.bin"
            archive.write_bytes(data)
            corpus.extract(archive, "tar.gz", "alice29.txt", sha256(data), out)
            self.assertEqual(out.read_bytes(), payload)
            names = sorted(path.name for path in Path(tmp).iterdir())
            self.assertEqual(names, ["c.tar.gz", "out.bin"])

    def test_flipped_byte_is_hash_mismatch_and_writes_nothing(self) -> None:
        data = write_tar([("a", b"xyz")], "w:gz")
        flipped = bytearray(data)
        flipped[0] ^= 0x01
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "c.tar.gz"
            out = Path(tmp) / "out.bin"
            archive.write_bytes(flipped)
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.extract(archive, "tar.gz", "a", sha256(data), out)
            self.assertEqual(caught.exception.code, "HashMismatch")
            self.assertFalse(out.exists())

    def test_path_traversal_and_absolute_names(self) -> None:
        for name in ("../x", "/tmp/x"):
            data = write_tar([(name, b"bad")], "w:")
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.member_bytes(data, "tar", name)
            self.assertEqual(caught.exception.code, "PathTraversal")

    def test_symlink_and_hardlink(self) -> None:
        link = tarfile.TarInfo("link")
        link.type = tarfile.SYMTYPE
        link.linkname = "a"
        data = write_tar([("a", b"target"), (link, b"")], "w:")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(data, "tar", "link")
        self.assertEqual(caught.exception.code, "UnsafeLink")

        hard = tarfile.TarInfo("hard")
        hard.type = tarfile.LNKTYPE
        hard.linkname = "a"
        data = write_tar([("a", b"target"), (hard, b"")], "w:")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(data, "tar", "hard")
        self.assertEqual(caught.exception.code, "UnsafeLink")

    def test_duplicate_member(self) -> None:
        data = write_tar([("a", b"one"), ("a", b"two")], "w:")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(data, "tar", "a")
        self.assertEqual(caught.exception.code, "DuplicateMember")

    def test_truncated_tar(self) -> None:
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(b"not-a-tar", "tar", "a")
        self.assertEqual(caught.exception.code, "TruncatedArchive")
        short = ustar_header("a", 10) + b"abcd"
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(short, "tar", "a")
        self.assertEqual(caught.exception.code, "TruncatedArchive")
        cut = write_tar([("a", b"hello world")], "w:gz")[:24]
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(cut, "tar.gz", "a")
        self.assertEqual(caught.exception.code, "TruncatedArchive")

    def test_negative_ustar_size_and_short_ar_are_typed(self) -> None:
        for text in (b"-1000", b"-2000"):
            header = bytearray(ustar_header("a", 0))
            header[124:136] = text.ljust(12, b"\0")
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.member_bytes(bytes(header), "tar", "a")
            self.assertEqual(caught.exception.code, "CorruptArchive")
        raw = bytearray(b"!<arch>\n")
        ar_header = bytearray(b" " * 60)
        ar_header[0:16] = b"big".ljust(16)
        ar_header[48:58] = str(corpus.MEMBER_MAX + 1).encode().ljust(10)
        ar_header[58:60] = b"`\n"
        raw += ar_header
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.parse_ar(bytes(raw))
        self.assertEqual(caught.exception.code, "TooLarge")

    def test_unsupported_flac_kind(self) -> None:
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(b"fLaC", "flac", "")
        self.assertEqual(caught.exception.code, "UnsupportedFormat")

    def test_zip_symlink_and_encrypted(self) -> None:
        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as handle:
            info = zipfile.ZipInfo("link")
            info.create_system = 3
            info.external_attr = (stat.S_IFLNK | 0o777) << 16
            handle.writestr(info, b"target")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(buffer.getvalue(), "zip", "link")
        self.assertEqual(caught.exception.code, "UnsafeLink")

        buffer = io.BytesIO()
        with zipfile.ZipFile(buffer, "w") as handle:
            info = zipfile.ZipInfo("secret")
            handle.writestr(info, b"nope")
        raw = bytearray(buffer.getvalue())
        local = raw.find(b"PK\x03\x04")
        central = raw.find(b"PK\x01\x02")
        raw[local + 6 : local + 8] = (1).to_bytes(2, "little")
        raw[central + 8 : central + 10] = (1).to_bytes(2, "little")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(bytes(raw), "zip", "secret")
        self.assertEqual(caught.exception.code, "Encrypted")

        for mode in (0, 0o644, stat.S_IFREG | 0o644, 0x20):
            buffer = io.BytesIO()
            with zipfile.ZipFile(buffer, "w") as handle:
                info = zipfile.ZipInfo("plain")
                info.create_system = 0 if mode == 0x20 else 3
                info.external_attr = mode << 16 if mode != 0x20 else mode
                handle.writestr(info, b"bytes")
            self.assertEqual(corpus.member_bytes(buffer.getvalue(), "zip", "plain"), b"bytes")

    def test_declared_size_above_member_max(self) -> None:
        header = ustar_header("big", corpus.MEMBER_MAX + 1)
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(header, "tar", "big")
        self.assertEqual(caught.exception.code, "TooLarge")

    def test_md5_match_and_mismatch(self) -> None:
        payload = b"dickens"
        data = write_tar([("dickens", payload)], "w:")
        digest = hashlib.md5(payload).hexdigest()
        self.assertEqual(corpus.member_bytes(data, "tar", "dickens"), payload)
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "a.tar"
            out = Path(tmp) / "out"
            archive.write_bytes(data)
            corpus.extract(archive, "tar", "dickens", sha256(data), out, digest)
            self.assertEqual(out.read_bytes(), payload)
            out.unlink()
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.extract(archive, "tar", "dickens", sha256(data), out, "0" * 32)
            self.assertEqual(caught.exception.code, "Md5Mismatch")
            self.assertFalse(out.exists())

    def test_deb_extracts_dotted_member_and_rejects_zst(self) -> None:
        payload = b"\x7fELF"
        data = build_deb("./usr/bin/rg", payload)
        self.assertEqual(corpus.member_bytes(data, "deb", "./usr/bin/rg"), payload)
        zst = build_deb("./usr/bin/rg", payload, "data.tar.zst")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.member_bytes(zst, "deb", "./usr/bin/rg")
        self.assertEqual(caught.exception.code, "UnsupportedFormat")

    def test_direct_file_is_not_decoded(self) -> None:
        payload = b"fLaC-bytes-not-decoded"
        self.assertEqual(corpus.member_bytes(payload, "direct", ""), payload)
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "enwik8.zip"
            out = Path(tmp) / "out"
            archive.write_bytes(payload)
            corpus.extract(archive, "direct", "", sha256(payload), out)
            self.assertEqual(out.read_bytes(), payload)

    def test_empty_sha256_is_unresolved(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "a"
            archive.write_bytes(b"abc")
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.extract(archive, "direct", "", "", Path(tmp) / "out")
            self.assertEqual(caught.exception.code, "UnresolvedHash")

    def test_pin_prints_hashes_and_does_not_touch_manifest(self) -> None:
        payload = b"member"
        data = write_tar([("a", payload)], "w:gz")
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / "a.tar.gz"
            sentinel = Path(tmp) / "manifest.toml"
            archive.write_bytes(data)
            sentinel.write_text("untouched\n", encoding="utf-8")
            before = sentinel.read_bytes()
            archive_sha, member_sha = corpus.pin(archive, "tar.gz", "a")
            self.assertEqual(archive_sha, sha256(data))
            self.assertEqual(member_sha, sha256(payload))
            self.assertEqual(sentinel.read_bytes(), before)
            completed = subprocess.run(
                [
                    sys.executable,
                    str(Path(__file__).with_name("fetch.py")),
                    "pin",
                    "--archive",
                    str(archive),
                    "--kind",
                    "tar.gz",
                    "--member",
                    "a",
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(completed.returncode, 0)
            self.assertEqual(
                completed.stdout,
                f"archive_sha256={archive_sha}\nmember_sha256={member_sha}\n",
            )
            self.assertEqual(sentinel.read_bytes(), before)

    def test_pcm_samples_and_rejection(self) -> None:
        samples = [-32768, -1, 0, 32767]
        raw = b"".join(sample.to_bytes(2, "little", signed=True) for sample in samples)
        with tempfile.TemporaryDirectory() as tmp:
            wav_path = Path(tmp) / "clip.wav"
            out = Path(tmp) / "u8"
            raw_out = Path(tmp) / "raw"
            wav_path.write_bytes(wav_bytes(samples))
            code = fetch.main(
                [
                    "transform",
                    "pcm16le-to-u8",
                    "--wav",
                    str(wav_path),
                    "--out",
                    str(out),
                    "--raw-out",
                    str(raw_out),
                ]
            )
            self.assertEqual(code, 0)
            self.assertEqual(out.read_bytes(), bytes([0x00, 0x7F, 0x80, 0xFF]))
            self.assertEqual(raw_out.read_bytes(), raw)

            stereo = Path(tmp) / "stereo.wav"
            stereo.write_bytes(wav_bytes([0, 0], channels=2))
            bad_out = Path(tmp) / "bad-u8"
            bad_raw = Path(tmp) / "bad-raw"
            code = fetch.main(
                [
                    "transform",
                    "pcm16le-to-u8",
                    "--wav",
                    str(stereo),
                    "--out",
                    str(bad_out),
                    "--raw-out",
                    str(bad_raw),
                ]
            )
            self.assertEqual(code, 1)
            self.assertFalse(bad_out.exists())
            self.assertFalse(bad_raw.exists())

            fast = Path(tmp) / "fast.wav"
            fast.write_bytes(wav_bytes([0], rate=44100))
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.pcm16le_to_u8(fast)
            self.assertEqual(caught.exception.code, "UnsupportedWav")

    def test_slice_middle_offset(self) -> None:
        result = corpus.slice_middle(b"0123456789", 4)
        self.assertEqual(result["offset"], 3)
        self.assertEqual(result["bytes"], b"3456")
        self.assertEqual(result["length"], 4)
        self.assertEqual(result["source_sha256"], sha256(b"0123456789"))
        self.assertEqual(result["derived_sha256"], sha256(b"3456"))

    def test_cache_rules_and_reread(self) -> None:
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.assert_cache(Path("relative"), REPO)
        self.assertEqual(caught.exception.code, "CacheNotAbsolute")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.assert_cache(REPO / "corpora", REPO)
        self.assertEqual(caught.exception.code, "CacheInsideRepo")
        with tempfile.TemporaryDirectory() as tmp:
            cache = Path(tmp)
            previous = os.environ.get("SEREXPAI_CACHE")
            os.environ["SEREXPAI_CACHE"] = str(cache)
            try:
                self.assertEqual(corpus.resolve_cache(REPO), cache)
            finally:
                if previous is None:
                    os.environ.pop("SEREXPAI_CACHE", None)
                else:
                    os.environ["SEREXPAI_CACHE"] = previous
            self.assertTrue(cache.is_absolute())
            payload = b"cached"
            digest = sha256(payload)
            path = corpus.cache_write(cache, "blob", payload, digest)
            self.assertEqual(corpus.cache_read(path, digest), path)
            path.write_bytes(b"tampered")
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.cache_read(path, digest)
            self.assertEqual(caught.exception.code, "HashMismatch")
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.cache_write(cache, "../escape", b"nope", digest)
            self.assertEqual(caught.exception.code, "PathTraversal")

    def test_pcm_selection_stops_and_respects_the_prefix_boundary(self) -> None:
        rows = [
            ("LJ001-0001", 100),
            ("LJ001-0002", 200),
            ("LJ001-0003", 50),
            ("LJ0010-0001", 10),
            ("LJ026-0001", 10),
        ]
        seed = "fixture-seed"
        ordered = sorted(
            [row for row in rows if row[0].startswith("LJ001-")],
            key=lambda item: hashlib.sha256(seed.encode() + item[0].encode()).digest(),
        )
        self.assertGreaterEqual(len(ordered), 2)
        budget = ordered[0][1] + ordered[1][1] - 1
        taken, total = corpus.select_pcm(rows, seed, budget, ["LJ001"])
        self.assertEqual(taken, [ordered[0][0]])
        self.assertEqual(total, ordered[0][1])
        self.assertNotIn("LJ0010-0001", taken)
        self.assertNotIn("LJ026-0001", taken)

    def test_overlap_phases_oracle_and_repeated_bytes(self) -> None:
        k = 16
        stride = k - (k // 2) + 1
        for phase in range(stride):
            run = bytes([0x2A]) * k
            evaluation = bytes([0x01]) * phase + run + bytes([0x03]) * stride
            tuning = bytes([0x04]) * 3 + run + bytes([0x05]) * 3
            hit = corpus.find_overlap(tuning, evaluation, k)
            self.assertIsNotNone(hit)
            self.assertEqual(
                corpus.overlap_oracle(tuning, evaluation, k),
                hit is not None,
            )
        short = bytes([0x2A]) * (k - 1)
        tuning = bytes([0x04]) * 3 + short + bytes([0x05]) * 3
        evaluation = bytes([0x01]) * 4 + short + bytes([0x03]) * 4
        self.assertIsNone(corpus.find_overlap(tuning, evaluation, k))
        self.assertFalse(corpus.overlap_oracle(tuning, evaluation, k))

        shared = bytes([0x7E]) * k
        left = b"aaa" + shared + b"bbb"
        right = b"zzz" + shared + b"yyy"
        self.assertNotEqual(sha256(left), sha256(right))
        self.assertIsNotNone(corpus.find_overlap(left, right, k))

        crossing = bytes([0x01]) * 4090 + bytes([0x02]) * k + bytes([0x03]) * 8
        other = bytes([0x09]) * 5 + bytes([0x02]) * k
        hit = corpus.find_overlap(other, crossing, k)
        self.assertIsNotNone(hit)
        assert hit is not None
        self.assertGreaterEqual(int(hit["tuning_offset"]), 0)

        zeros = bytes(k + 8)
        hit = corpus.find_overlap(zeros, zeros, k)
        self.assertIsNotNone(hit)
        assert hit is not None
        self.assertTrue(hit["repeated_byte"])
        self.assertIsNone(corpus.find_overlap(b"", b"abc", k))
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.find_overlap(b"abcd", b"abcd", 3)
        self.assertEqual(caught.exception.code, "BadManifest")

    def test_overlap_is_cross_role_and_keeps_evaluation_fixed(self) -> None:
        rows = [
            {"id": "tune-text", "role": "tuning", "class": "text"},
            {"id": "eval-exec", "role": "evaluation", "class": "executable"},
            {"id": "synth", "role": "synthetic", "class": "synthetic"},
        ]
        shared = bytes([0x11]) * 16
        files = {
            "tune-text": shared + b"T",
            "eval-exec": b"E" + shared,
            "synth": shared,
        }
        hits = corpus.overlap_pairs(files, rows, 16)
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.overlap_pairs({}, [], 3)
        self.assertEqual(caught.exception.code, "BadManifest")
        with self.assertRaises(corpus.CorpusError) as caught:
            corpus.overlap_pairs({}, [{"id": "tune-text", "role": "tuning"}], 0)
        self.assertEqual(caught.exception.code, "BadManifest")
        self.assertEqual(len(hits), 1)
        self.assertEqual(hits[0]["tuning_id"], "tune-text")
        self.assertEqual(hits[0]["evaluation_id"], "eval-exec")
        text = tomllib_replacement()
        self.assertIn("Evaluation inputs stay fixed", text)
        self.assertNotIn("pcm", load_manifest()["replacement"])
        self.assertNotIn("executable", load_manifest()["replacement"])

    def test_gate_report_matches_and_stays_unready(self) -> None:
        manifest = load_manifest()
        document = corpus.gate(manifest)
        rendered = corpus.canonical_json(document)
        self.assertEqual(GATE_REPORT.read_text(encoding="utf-8"), rendered)
        self.assertFalse(document["corpus_gate_ready"])
        self.assertEqual(document["scope"], "corpus")
        self.assertEqual(document["k"], {"bytes": 1024, "status": "proposed"})
        fields = {item["field"] for item in document["unresolved"]}
        for name in (
            "k.status",
            "lj.seed",
            "lj.metadata_checked",
            "lj.prefixes",
            "pcm.byte_budget.tuning",
            "pcm.byte_budget.evaluation",
            "pcm.replacement_rule",
            "debian_tuning.package",
            "executable.replacement_rule",
            "executable.eval.slice_length",
            "flac-subset.members",
        ):
            self.assertIn(name, fields)
        self.assertEqual(document["violations"], [])
        excluded = set(document["excluded_from_sm1"])
        for synthetic in (
            "synthetic-polynomial",
            "synthetic-trigonometric",
            "synthetic-recurrence",
        ):
            self.assertIn(synthetic, excluded)
            self.assertNotIn(synthetic, document["counts_toward_sm1"])
        self.assertIn("lj-evaluation-raw16", excluded)
        self.assertIn("enwik8-zip", excluded)
        self.assertIn("silesia-dickens", document["counts_toward_sm1"])

    def test_fixture_manifest_violations(self) -> None:
        manifest = load_manifest()
        rows = manifest["input"]
        pcm = next(row for row in rows if row["id"] == "lj-evaluation")
        pcm["transform"] = "slice"
        violations = {item["field"] for item in corpus.gate(manifest)["violations"]}
        self.assertIn("lj-evaluation.transform", violations)

        manifest = load_manifest()
        text = next(row for row in manifest["input"] if row["id"] == "silesia-dickens")
        text["parameters"]["length"] = 100
        violations = {item["field"] for item in corpus.gate(manifest)["violations"]}
        self.assertIn("silesia-dickens.length", violations)

        manifest = load_manifest()
        synthetic = next(
            row for row in manifest["input"] if row["id"] == "synthetic-polynomial"
        )
        synthetic["counts_toward_sm1"] = True
        violations = {item["field"] for item in corpus.gate(manifest)["violations"]}
        self.assertIn("counts_toward_sm1.synthetic-polynomial", violations)

        manifest = load_manifest()
        raw16 = next(row for row in manifest["input"] if row["id"] == "lj-evaluation-raw16")
        raw16["counts_toward_sm1"] = True
        violations = {item["field"] for item in corpus.gate(manifest)["violations"]}
        self.assertIn("counts_toward_sm1.lj-evaluation-raw16", violations)

        manifest = load_manifest()
        seen = 0
        for row in manifest["input"]:
            if row["class"] == "executable" and row["role"] == "evaluation":
                row["parameters"] = {"length": 256 if seen == 0 else 512}
                seen += 1
        self.assertGreaterEqual(seen, 2)
        violations = {item["field"] for item in corpus.gate(manifest)["violations"]}
        self.assertIn("executable.eval.slice_length", violations)

    def test_cli_gate_matches_the_library(self) -> None:
        completed = subprocess.run(
            [
                sys.executable,
                str(Path(__file__).with_name("fetch.py")),
                "gate",
                "--manifest",
                str(MANIFEST),
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(completed.stdout, GATE_REPORT.read_text(encoding="utf-8"))


def load_manifest() -> dict:
    import tomllib

    return tomllib.loads(MANIFEST.read_text(encoding="utf-8"))


def tomllib_replacement() -> str:
    return load_manifest()["replacement"]["text"]["rule"]


if __name__ == "__main__":
    unittest.main()
