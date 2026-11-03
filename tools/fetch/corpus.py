"""Corpus fetch, transforms, overlap, and the corpus gate.

The standard library only. Nothing is downloaded by import.
"""

from __future__ import annotations

import bz2
import gzip
import hashlib
import io
import json
import lzma
import os
import stat
import tarfile
import wave
import zipfile
import zlib
from pathlib import Path

MEMBER_MAX = 64 * 1024 * 1024
ARCHIVE_MAX = 4 * 1024 * 1024 * 1024

KINDS = frozenset(
    {"tar", "tar.gz", "tar.bz2", "tar.xz", "zip", "deb", "direct", "kernel-predict"}
)
TAR_MODES = {
    "tar": "r:",
    "tar.gz": "r:gz",
    "tar.bz2": "r:bz2",
    "tar.xz": "r:xz",
}

TEXT_EVAL_LENGTH = 3_145_728
TEXT_RULE = (
    "Evaluation inputs stay fixed. A tuning input that fails the overlap "
    "check is dropped and replaced by another Calgary or Canterbury input "
    "that passes. No stripping, no moving evaluation slices, no allowlist."
)

REQUIRED = {
    "canterbury-alice29": ("text", "tuning", "identity", "tar.gz"),
    "canterbury-lcet10": ("text", "tuning", "identity", "tar.gz"),
    "canterbury-plrabn12": ("text", "tuning", "identity", "tar.gz"),
    "canterbury-cp.html": ("text", "tuning", "identity", "tar.gz"),
    "canterbury-xargs.1": ("text", "tuning", "identity", "tar.gz"),
    "calgary-book1": ("text", "tuning", "identity", "tar.gz"),
    "calgary-book2": ("text", "tuning", "identity", "tar.gz"),
    "calgary-paper1": ("text", "tuning", "identity", "tar.gz"),
    "calgary-paper2": ("text", "tuning", "identity", "tar.gz"),
    "silesia-dickens": ("text", "evaluation", "slice", "zip"),
    "silesia-webster": ("text", "evaluation", "slice", "zip"),
    "silesia-xml": ("text", "evaluation", "slice", "zip"),
    "silesia-reymont": ("text", "evaluation", "slice", "zip"),
    "lj-tuning": ("pcm", "tuning", "pcm16le-to-u8", "tar.bz2"),
    "lj-evaluation": ("pcm", "evaluation", "pcm16le-to-u8", "tar.bz2"),
    "lj-tuning-raw16": ("pcm", "tuning", "pcm16le-raw", "tar.bz2"),
    "lj-evaluation-raw16": ("pcm", "evaluation", "pcm16le-raw", "tar.bz2"),
    "calgary-obj1": ("executable", "tuning", "identity", "tar.gz"),
    "calgary-obj2": ("executable", "tuning", "identity", "tar.gz"),
    "canterbury-sum": ("executable", "tuning", "identity", "tar.gz"),
    "debian-tuning-amd64": ("executable", "tuning", "identity", "deb"),
    "debian-tuning-arm64": ("executable", "tuning", "identity", "deb"),
    "silesia-ooffice": ("executable", "evaluation", "slice", "zip"),
    "debian-busybox-static-amd64": ("executable", "evaluation", "slice", "deb"),
    "debian-busybox-static-arm64": ("executable", "evaluation", "slice", "deb"),
    "debian-ripgrep-amd64": ("executable", "evaluation", "slice", "deb"),
    "debian-ripgrep-arm64": ("executable", "evaluation", "slice", "deb"),
    "enwik8-zip": ("compressed", "evaluation", "identity", "direct"),
    "flac-subset": ("compressed", "evaluation", "identity", "direct"),
    "synthetic-polynomial": ("synthetic", "synthetic", "kernel-predict", "kernel-predict"),
    "synthetic-trigonometric": ("synthetic", "synthetic", "kernel-predict", "kernel-predict"),
    "synthetic-recurrence": ("synthetic", "synthetic", "kernel-predict", "kernel-predict"),
}


class CorpusError(Exception):
    def __init__(self, code: str) -> None:
        self.code = code
        super().__init__(code)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def md5_bytes(data: bytes) -> str:
    return hashlib.md5(data).hexdigest()


def read_bounded(path: Path, limit: int) -> bytes:
    if not path.is_file():
        raise CorpusError("NotRegular")
    with path.open("rb") as handle:
        data = handle.read(limit + 1)
    if len(data) > limit:
        raise CorpusError("TooLarge")
    return data


def sha256_path(path: Path, limit: int) -> str:
    if not path.is_file():
        raise CorpusError("NotRegular")
    digest = hashlib.sha256()
    seen = 0
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if chunk == b"":
                break
            seen += len(chunk)
            if seen > limit:
                raise CorpusError("TooLarge")
            digest.update(chunk)
    return digest.hexdigest()


_TEMP_SEQ = 0


def _temp_file(directory: Path, name: str) -> Path:
    global _TEMP_SEQ
    _TEMP_SEQ += 1
    return directory / f".{name}.{os.getpid()}.{_TEMP_SEQ}.tmp"


def _cache_component(name: str) -> None:
    if name in {"", ".", ".."} or "/" in name or "\0" in name:
        raise CorpusError("PathTraversal")


def resolve_cache(repo_root: Path) -> Path:
    override = os.environ.get("SEREXPAI_CACHE")
    if override is not None:
        return assert_cache(Path(override), repo_root)
    xdg = os.environ.get("XDG_CACHE_HOME")
    if xdg:
        candidate = Path(xdg) / "serexpai"
        if candidate.is_absolute():
            return assert_cache(candidate, repo_root)
    home = os.environ.get("HOME", "")
    return assert_cache(Path(home) / ".cache" / "serexpai", repo_root)


def assert_cache(cache: Path, repo_root: Path) -> Path:
    if not cache.is_absolute():
        raise CorpusError("CacheNotAbsolute")
    cache_real = cache.resolve()
    repo_real = repo_root.resolve()
    if cache_real == repo_real or repo_real in cache_real.parents:
        raise CorpusError("CacheInsideRepo")
    return cache_real


def cache_write(cache: Path, name: str, data: bytes, expected_sha256: str) -> Path:
    _cache_component(name)
    cache.mkdir(parents=True, exist_ok=True)
    dest = cache / name
    temporary = _temp_file(cache, name)
    try:
        temporary.write_bytes(data)
        if sha256_path(temporary, ARCHIVE_MAX) != expected_sha256:
            raise CorpusError("HashMismatch")
        os.replace(temporary, dest)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise
    return cache_read(dest, expected_sha256)


def cache_read(path: Path, expected_sha256: str) -> Path:
    if sha256_path(path, ARCHIVE_MAX) != expected_sha256:
        raise CorpusError("HashMismatch")
    return path


def reject_name(name: str) -> None:
    if name == "" or "\0" in name or name.startswith("/"):
        raise CorpusError("PathTraversal")
    for segment in name.split("/"):
        if segment == "..":
            raise CorpusError("PathTraversal")


def _reject_duplicate(names: list[str]) -> None:
    seen: set[str] = set()
    for name in names:
        if name in seen:
            raise CorpusError("DuplicateMember")
        seen.add(name)


def _map_tar_error(exc: Exception) -> CorpusError:
    if isinstance(exc, tarfile.ReadError):
        return CorpusError("TruncatedArchive")
    if isinstance(exc, EOFError):
        return CorpusError("TruncatedArchive")
    if isinstance(exc, gzip.BadGzipFile):
        return CorpusError("CorruptArchive")
    if isinstance(exc, lzma.LZMAError):
        return CorpusError("CorruptArchive")
    if isinstance(exc, tarfile.TarError):
        return CorpusError("CorruptArchive")
    return CorpusError("CorruptArchive")


def parse_ar(data: bytes) -> list[tuple[str, bytes]]:
    if len(data) < 8:
        raise CorpusError("TruncatedArchive")
    if not data.startswith(b"!<arch>\n"):
        raise CorpusError("CorruptArchive")
    position = 8
    entries: list[tuple[str, bytes]] = []
    names: list[str] = []
    while position < len(data):
        if position + 60 > len(data):
            raise CorpusError("TruncatedArchive")
        header = data[position : position + 60]
        position += 60
        if header[58:60] != b"`\n":
            raise CorpusError("CorruptArchive")
        name = header[0:16].decode("ascii", "replace").strip()
        if name.endswith("/"):
            name = name[:-1]
        size_text = header[48:58].decode("ascii", "replace").strip()
        try:
            size = int(size_text, 10)
        except ValueError as exc:
            raise CorpusError("CorruptArchive") from exc
        if size < 0:
            raise CorpusError("CorruptArchive")
        if size > MEMBER_MAX:
            raise CorpusError("TooLarge")
        if position + size > len(data):
            raise CorpusError("TruncatedArchive")
        blob = data[position : position + size]
        position += size
        if size % 2 == 1:
            if position >= len(data):
                raise CorpusError("TruncatedArchive")
            position += 1
        names.append(name)
        entries.append((name, blob))
    _reject_duplicate(names)
    return entries


def _read_tar_member(handle: tarfile.TarFile, member: tarfile.TarInfo) -> bytes:
    if member.issym() or member.islnk():
        raise CorpusError("UnsafeLink")
    if not member.isfile():
        raise CorpusError("NotRegular")
    if member.size < 0:
        raise CorpusError("CorruptArchive")
    if member.size > MEMBER_MAX:
        raise CorpusError("TooLarge")
    extracted = handle.extractfile(member)
    if extracted is None:
        raise CorpusError("NotRegular")
    try:
        blob = extracted.read(member.size + 1)
    except (tarfile.TarError, EOFError, gzip.BadGzipFile, lzma.LZMAError) as exc:
        raise _map_tar_error(exc) from exc
    if len(blob) > member.size:
        raise CorpusError("CorruptArchive")
    if len(blob) != member.size:
        raise CorpusError("TruncatedArchive")
    return blob


def _reject_huge_ustar(data: bytes) -> None:
    position = 0
    while position + 512 <= len(data):
        block = data[position : position + 512]
        if block == b"\x00" * 512:
            return
        text = block[124:136].split(b"\x00", 1)[0].strip()
        if text == b"":
            return
        try:
            size = int(text, 8)
        except ValueError:
            return
        if size < 0:
            raise CorpusError("CorruptArchive")
        if size > MEMBER_MAX:
            raise CorpusError("TooLarge")
        step = 512 * (1 + (size + 511) // 512)
        if step <= 0:
            raise CorpusError("CorruptArchive")
        position += step


def _decompress_capped(data: bytes, kind: str) -> bytes:
    if kind == "tar":
        return data
    if kind == "tar.gz":
        handle: gzip.GzipFile | bz2.BZ2File | lzma.LZMAFile = gzip.GzipFile(
            fileobj=io.BytesIO(data)
        )
    elif kind == "tar.bz2":
        handle = bz2.BZ2File(io.BytesIO(data))
    elif kind == "tar.xz":
        handle = lzma.LZMAFile(io.BytesIO(data))
    else:
        raise CorpusError("UnsupportedFormat")
    try:
        plain = handle.read(ARCHIVE_MAX + 1)
    except EOFError as exc:
        raise CorpusError("TruncatedArchive") from exc
    except (OSError, lzma.LZMAError, gzip.BadGzipFile) as exc:
        raise CorpusError("CorruptArchive") from exc
    finally:
        handle.close()
    if len(plain) > ARCHIVE_MAX:
        raise CorpusError("TooLarge")
    return plain


def _tar_member(data: bytes, kind: str, member: str) -> bytes:
    plain = _decompress_capped(data, kind)
    _reject_huge_ustar(plain)
    try:
        handle = tarfile.open(fileobj=io.BytesIO(plain), mode="r:")
    except (tarfile.TarError, EOFError, gzip.BadGzipFile, lzma.LZMAError) as exc:
        raise _map_tar_error(exc) from exc
    try:
        try:
            members = handle.getmembers()
        except (tarfile.TarError, EOFError, gzip.BadGzipFile, lzma.LZMAError) as exc:
            raise _map_tar_error(exc) from exc
        _reject_duplicate([item.name for item in members])
        found: bytes | None = None
        for item in members:
            reject_name(item.name)
            if item.name != member:
                continue
            if found is not None:
                raise CorpusError("DuplicateMember")
            found = _read_tar_member(handle, item)
        if found is None:
            raise CorpusError("NotRegular")
        return found
    finally:
        handle.close()


def _zip_member(data: bytes, member: str) -> bytes:
    try:
        handle = zipfile.ZipFile(io.BytesIO(data))
    except (zipfile.BadZipFile, OSError, ValueError) as exc:
        raise CorpusError("TruncatedArchive") from exc
    try:
        infos = handle.infolist()
        _reject_duplicate([info.filename for info in infos])
        found: bytes | None = None
        for info in infos:
            reject_name(info.filename)
            if info.filename != member:
                continue
            if info.compress_type not in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED):
                raise CorpusError("UnsupportedFormat")
            if info.flag_bits & 1:
                raise CorpusError("Encrypted")
            mode = (info.external_attr >> 16) & 0xFFFF
            if stat.S_IFMT(mode) == stat.S_IFLNK:
                raise CorpusError("UnsafeLink")
            if info.is_dir() or stat.S_IFMT(mode) == stat.S_IFDIR:
                raise CorpusError("NotRegular")
            if info.is_dir():
                raise CorpusError("NotRegular")
            if info.file_size > MEMBER_MAX:
                raise CorpusError("TooLarge")
            try:
                blob = handle.read(info)
            except zipfile.BadZipFile as exc:
                raise CorpusError("CorruptArchive") from exc
            except RuntimeError as exc:
                raise CorpusError("Encrypted") from exc
            except EOFError as exc:
                raise CorpusError("TruncatedArchive") from exc
            except (zlib.error, OSError, lzma.LZMAError, NotImplementedError, ValueError) as exc:
                raise CorpusError("CorruptArchive") from exc
            if len(blob) > info.file_size:
                raise CorpusError("CorruptArchive")
            if len(blob) != info.file_size:
                raise CorpusError("TruncatedArchive")
            found = blob
        if found is None:
            raise CorpusError("NotRegular")
        return found
    finally:
        handle.close()


def _deb_tar_xz(data: bytes) -> bytes:
    entries = parse_ar(data)
    xz_blob = None
    for name, blob in entries:
        if name == "data.tar.zst":
            raise CorpusError("UnsupportedFormat")
        if name == "data.tar.xz":
            xz_blob = blob
    if xz_blob is None:
        raise CorpusError("UnsupportedFormat")
    return xz_blob


def member_bytes(data: bytes, kind: str, member: str) -> bytes:
    if kind not in KINDS or kind == "kernel-predict":
        raise CorpusError("UnsupportedFormat")
    if kind == "direct":
        if member != "":
            raise CorpusError("BadManifest")
        if len(data) > MEMBER_MAX:
            raise CorpusError("TooLarge")
        return data
    if kind == "deb":
        reject_name(member)
        return _tar_member(_deb_tar_xz(data), "tar.xz", member)
    if kind == "zip":
        reject_name(member)
        return _zip_member(data, member)
    if kind in TAR_MODES:
        reject_name(member)
        return _tar_member(data, kind, member)
    raise CorpusError("UnsupportedFormat")


def _check_md5(blob: bytes, md5: str | None) -> None:
    if md5 is None or md5 == "":
        return
    if md5_bytes(blob) != md5:
        raise CorpusError("Md5Mismatch")


def extract(
    archive: Path,
    kind: str,
    member: str,
    sha256: str,
    out: Path,
    md5: str | None = None,
) -> None:
    if kind not in KINDS or kind == "kernel-predict":
        raise CorpusError("UnsupportedFormat")
    if sha256 == "":
        raise CorpusError("UnresolvedHash")
    data = read_bounded(archive, ARCHIVE_MAX)
    if sha256_bytes(data) != sha256:
        raise CorpusError("HashMismatch")
    blob = member_bytes(data, kind, member)
    _check_md5(blob, md5)
    _write_new(out, blob)


def pin(archive: Path, kind: str, member: str, md5: str | None = None) -> tuple[str, str]:
    if kind not in KINDS or kind == "kernel-predict":
        raise CorpusError("UnsupportedFormat")
    data = read_bounded(archive, ARCHIVE_MAX)
    blob = member_bytes(data, kind, member)
    _check_md5(blob, md5)
    return sha256_bytes(data), sha256_bytes(blob)


def _write_new(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = _temp_file(path.parent, path.name)
    try:
        temporary.write_bytes(data)
        os.replace(temporary, path)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def pcm16le_to_u8(wav_path: Path) -> tuple[bytes, bytes]:
    try:
        handle = wave.open(str(wav_path), "rb")
    except (wave.Error, EOFError, OSError) as exc:
        raise CorpusError("UnsupportedWav") from exc
    try:
        channels = handle.getnchannels()
        width = handle.getsampwidth()
        rate = handle.getframerate()
        comptype = handle.getcomptype()
        frames = handle.getnframes()
        chunk = getattr(handle, "_data_chunk", None)
        declared = getattr(chunk, "chunksize", None)
        if channels != 1 or width != 2 or rate != 22050 or comptype != "NONE":
            raise CorpusError("UnsupportedWav")
        if type(declared) is not int:
            raise CorpusError("UnsupportedWav")
        if declared > MEMBER_MAX:
            raise CorpusError("TooLarge")
        if declared != frames * width:
            raise CorpusError("UnsupportedWav")
        raw = handle.readframes(frames)
    except (wave.Error, OSError) as exc:
        raise CorpusError("UnsupportedWav") from exc
    finally:
        handle.close()
    if len(raw) != declared:
        raise CorpusError("UnsupportedWav")
    derived = bytearray()
    for index in range(0, len(raw), 2):
        sample = int.from_bytes(raw[index : index + 2], "little", signed=True)
        derived.append(((sample >> 8) & 0xFF) ^ 0x80)
    return bytes(derived), raw


def slice_middle(data: bytes, length: int) -> dict[str, object]:
    if type(length) is not int or length < 1 or length > len(data):
        raise CorpusError("SliceOutOfRange")
    offset = (len(data) - length) // 2
    derived = data[offset : offset + length]
    return {
        "source_sha256": sha256_bytes(data),
        "offset": offset,
        "length": length,
        "derived_sha256": sha256_bytes(derived),
        "bytes": derived,
    }


def _prefixes_overlap(tuning: set[str], evaluation: set[str]) -> bool:
    for left in tuning:
        for right in evaluation:
            if prefix_match(left, right) or prefix_match(right, left):
                return True
    return False


def _empty_or_text(
    value: object,
    field: str,
    unresolved: list[dict[str, str]],
    violations: list[dict[str, str]],
    *,
    empty_unresolved: bool,
) -> None:
    if type(value) is not str:
        violations.append({"field": field, "reason": "invalid"})
    elif value == "" and empty_unresolved:
        unresolved.append(_unresolved(field, "empty"))


def prefix_match(clip_id: str, prefix: str) -> bool:
    return clip_id == prefix or clip_id.startswith(prefix + "-")


def select_pcm(
    rows: list[tuple[str, int]],
    seed: str,
    budget: int,
    prefixes: list[str],
) -> tuple[list[str], int]:
    if type(budget) is not int or budget < 0:
        raise CorpusError("BadManifest")
    chosen: list[tuple[str, int]] = []
    for clip_id, nbytes in rows:
        if type(nbytes) is not int or nbytes < 0:
            raise CorpusError("BadManifest")
        if any(prefix_match(clip_id, prefix) for prefix in prefixes):
            chosen.append((clip_id, nbytes))

    def sort_key(item: tuple[str, int]) -> tuple[bytes, str]:
        clip_id = item[0]
        digest = hashlib.sha256(seed.encode() + clip_id.encode()).digest()
        return (digest, clip_id)

    chosen.sort(key=sort_key)
    taken: list[str] = []
    total = 0
    for clip_id, nbytes in chosen:
        if total + nbytes > budget:
            break
        taken.append(clip_id)
        total += nbytes
    return taken, total


def find_overlap(tuning: bytes, evaluation: bytes, k: int) -> dict[str, object] | None:
    if type(k) is not int or k < 2 or k % 2 == 1:
        raise CorpusError("BadManifest")
    if len(tuning) < k or len(evaluation) < k:
        return None
    window = k // 2
    stride = k - window + 1
    index: dict[bytes, list[int]] = {}
    for offset in range(0, len(evaluation) - window + 1, stride):
        key = evaluation[offset : offset + window]
        index.setdefault(key, []).append(offset)
    limit = len(tuning) - window + 1
    for start in range(limit):
        key = tuning[start : start + window]
        for origin in index.get(key, ()):
            tuning_at = start
            evaluation_at = origin
            while (
                tuning_at > 0
                and evaluation_at > 0
                and tuning[tuning_at - 1] == evaluation[evaluation_at - 1]
            ):
                tuning_at -= 1
                evaluation_at -= 1
            tuning_end = start + window
            evaluation_end = origin + window
            while (
                tuning_end < len(tuning)
                and evaluation_end < len(evaluation)
                and tuning[tuning_end] == evaluation[evaluation_end]
            ):
                tuning_end += 1
                evaluation_end += 1
            length = tuning_end - tuning_at
            if length >= k:
                run = tuning[tuning_at:tuning_end]
                return {
                    "tuning_offset": tuning_at,
                    "evaluation_offset": evaluation_at,
                    "length": length,
                    "repeated_byte": run == bytes([run[0]]) * length,
                }
    return None


def overlap_oracle(tuning: bytes, evaluation: bytes, k: int) -> bool:
    if type(k) is not int or k < 2 or k % 2 == 1:
        raise CorpusError("BadManifest")
    if len(tuning) > 4096 or len(evaluation) > 4096:
        raise CorpusError("BadManifest")
    if len(tuning) < k or len(evaluation) < k:
        return False
    for tuning_at in range(len(tuning) - k + 1):
        window = tuning[tuning_at : tuning_at + k]
        for evaluation_at in range(len(evaluation) - k + 1):
            if evaluation[evaluation_at : evaluation_at + k] == window:
                return True
    return False


def overlap_pairs(
    files: dict[str, bytes],
    rows: list[dict[str, object]],
    k: int,
) -> list[dict[str, object]]:
    if type(k) is not int or k < 2 or k % 2 == 1:
        raise CorpusError("BadManifest")
    hits: list[dict[str, object]] = []
    tuning_rows: list[dict[str, object]] = []
    evaluation_rows: list[dict[str, object]] = []
    for row in rows:
        if not isinstance(row, dict):
            raise CorpusError("BadManifest")
        if row.get("role") == "tuning":
            tuning_rows.append(row)
        elif row.get("role") == "evaluation":
            evaluation_rows.append(row)
    for tuning in tuning_rows:
        tuning_id = _require_id(tuning)
        for evaluation in evaluation_rows:
            evaluation_id = _require_id(evaluation)
            if tuning_id not in files or evaluation_id not in files:
                raise CorpusError("BadManifest")
            hit = find_overlap(files[tuning_id], files[evaluation_id], k)
            if hit is None:
                continue
            hit["tuning_id"] = tuning_id
            hit["evaluation_id"] = evaluation_id
            hits.append(hit)
    return hits


def _require_id(row: dict[str, object]) -> str:
    row_id = row.get("id")
    if type(row_id) is not str or row_id == "":
        raise CorpusError("BadManifest")
    return row_id


def counts_toward_sm1(row: dict[str, object]) -> bool:
    return (
        row.get("class") in {"text", "pcm", "executable"}
        and row.get("role") == "evaluation"
        and row.get("transform") != "pcm16le-raw"
    )


def canonical_json(document: dict[str, object]) -> str:
    return json.dumps(document, ensure_ascii=True, indent=2, sort_keys=True) + "\n"


def _parameters(row: dict[str, object]) -> dict[str, object]:
    parameters = row.get("parameters", {})
    if isinstance(parameters, dict):
        return parameters
    return {}


def _unresolved(field: str, reason: str) -> dict[str, str]:
    return {"field": field, "reason": reason}


def gate(manifest: dict[str, object]) -> dict[str, object]:
    unresolved: list[dict[str, str]] = []
    violations: list[dict[str, str]] = []
    if type(manifest.get("schema")) is not int or manifest.get("schema") != 1:
        violations.append({"field": "schema", "reason": "invalid"})
    k_table = manifest.get("k", {})
    if not isinstance(k_table, dict):
        k_table = {}
    if "bytes" not in k_table:
        unresolved.append(_unresolved("k.bytes", "absent"))
        k_bytes = 0
    else:
        raw_k = k_table.get("bytes")
        if type(raw_k) is not int or raw_k < 2 or raw_k % 2 == 1:
            violations.append({"field": "k.bytes", "reason": "invalid"})
            k_bytes = 0
        else:
            k_bytes = raw_k
    raw_status = k_table.get("status", "")
    if type(raw_status) is not str:
        violations.append({"field": "k.status", "reason": "invalid"})
        k_status = ""
        unresolved.append(_unresolved("k.status", "invalid"))
    else:
        k_status = raw_status
        if k_status != "confirmed":
            unresolved.append(_unresolved("k.status", k_status or "missing"))

    lj = manifest.get("lj", {})
    if not isinstance(lj, dict):
        lj = {}
    seed = lj.get("seed", "")
    if type(seed) is not str:
        violations.append({"field": "lj.seed", "reason": "invalid"})
    elif seed == "":
        unresolved.append(_unresolved("lj.seed", "empty"))
    if lj.get("metadata_checked") is not True:
        unresolved.append(_unresolved("lj.metadata_checked", "false"))
    if lj.get("prefixes_confirmed") is not True:
        unresolved.append(_unresolved("lj.prefixes", "unconfirmed"))

    pcm = manifest.get("pcm", {})
    if not isinstance(pcm, dict):
        pcm = {}
    prefix_sets: dict[str, set[str]] = {}
    for role in ("tuning", "evaluation"):
        role_table = pcm.get(role, {})
        if not isinstance(role_table, dict):
            role_table = {}
        prefixes = role_table.get("prefixes", [])
        if (
            not isinstance(prefixes, list)
            or any(type(item) is not str or item == "" for item in prefixes)
        ):
            violations.append({"field": "lj.prefixes", "reason": "invalid"})
            prefix_sets[role] = set()
        else:
            prefix_sets[role] = set(prefixes)
        if "byte_budget" not in role_table:
            unresolved.append(_unresolved(f"pcm.byte_budget.{role}", "absent"))
        else:
            budget = role_table.get("byte_budget")
            if type(budget) is not int or budget < 1:
                violations.append(
                    {"field": f"pcm.byte_budget.{role}", "reason": "invalid"}
                )
    if _prefixes_overlap(prefix_sets["tuning"], prefix_sets["evaluation"]):
        violations.append({"field": "lj.prefixes", "reason": "overlap"})

    replacement = manifest.get("replacement", {})
    if not isinstance(replacement, dict):
        replacement = {}
    text_rule = replacement.get("text", {})
    if not isinstance(text_rule, dict) or text_rule.get("rule") != TEXT_RULE:
        violations.append({"field": "replacement.text", "reason": "rule mismatch"})
    if "pcm" in replacement:
        violations.append({"field": "pcm.replacement_rule", "reason": "not approved"})
    else:
        unresolved.append(_unresolved("pcm.replacement_rule", "absent"))
    if "executable" in replacement:
        violations.append(
            {"field": "executable.replacement_rule", "reason": "not approved"}
        )
    else:
        unresolved.append(_unresolved("executable.replacement_rule", "absent"))

    debian = manifest.get("debian_tuning", {})
    package = debian.get("package", "") if isinstance(debian, dict) else ""
    if type(package) is not str:
        violations.append({"field": "debian_tuning.package", "reason": "invalid"})
        unresolved.append(_unresolved("debian_tuning.package", "unnamed"))
    elif package == "":
        unresolved.append(_unresolved("debian_tuning.package", "unnamed"))

    rows = manifest.get("input", [])
    if not isinstance(rows, list):
        rows = []
    by_id: dict[str, dict[str, object]] = {}
    for row in rows:
        if not isinstance(row, dict):
            violations.append({"field": "input", "reason": "not a table"})
            continue
        row_id = row.get("id", "")
        if type(row_id) is not str or row_id == "":
            violations.append({"field": "input.id", "reason": "invalid"})
            continue
        if row_id in by_id:
            violations.append({"field": f"{row_id}.id", "reason": "duplicate"})
        by_id[row_id] = row

    for row_id, expected in sorted(REQUIRED.items()):
        row = by_id.get(row_id)
        if row is None:
            violations.append({"field": f"roster.{row_id}", "reason": "missing"})
            continue
        actual = (
            row.get("class"),
            row.get("role"),
            row.get("transform"),
            row.get("source_kind"),
        )
        if actual != expected:
            violations.append({"field": f"roster.{row_id}", "reason": "mismatch"})

    executable_lengths: list[int] = []
    executable_length_missing = False
    for row_id in sorted(by_id):
        row = by_id[row_id]
        raw_kind = row.get("source_kind", "")
        if type(raw_kind) is not str:
            violations.append({"field": f"{row_id}.source_kind", "reason": "invalid"})
            kind = ""
        else:
            kind = raw_kind
        parameters = _parameters(row)
        flag = row.get("counts_toward_sm1")
        if type(flag) is not bool or flag != counts_toward_sm1(row):
            violations.append(
                {"field": f"counts_toward_sm1.{row_id}", "reason": "disagrees"}
            )
        if type(row.get("source")) is not str:
            violations.append({"field": f"{row_id}.source", "reason": "invalid"})
        if type(row.get("transform_version")) is not int or row.get("transform_version") != 1:
            violations.append(
                {"field": f"{row_id}.transform_version", "reason": "invalid"}
            )
        licence_state = row.get("licence_state")
        if licence_state == "unresolved":
            unresolved.append(_unresolved(f"{row_id}.licence", "unresolved"))
            if "licence" in row:
                violations.append({"field": f"{row_id}.licence", "reason": "present"})
        elif licence_state == "recorded":
            licence = row.get("licence")
            if type(licence) is not str or licence == "":
                violations.append({"field": f"{row_id}.licence", "reason": "empty"})
        else:
            violations.append({"field": f"{row_id}.licence", "reason": "invalid"})
        _empty_or_text(
            row.get("archive_sha256", ""),
            f"{row_id}.archive_sha256",
            unresolved,
            violations,
            empty_unresolved=kind != "kernel-predict",
        )
        _empty_or_text(
            row.get("derived_sha256", ""),
            f"{row_id}.derived_sha256",
            unresolved,
            violations,
            empty_unresolved=True,
        )
        _empty_or_text(
            row.get("member", ""),
            f"{row_id}.member",
            unresolved,
            violations,
            empty_unresolved=kind not in {"direct", "kernel-predict"},
        )
        if row.get("class") == "text" and row.get("role") == "evaluation":
            if "offset" not in parameters:
                unresolved.append(_unresolved(f"{row_id}.offset", "absent"))
            else:
                offset = parameters.get("offset")
                if type(offset) is not int or offset < 0:
                    violations.append({"field": f"{row_id}.offset", "reason": "invalid"})
            length = parameters.get("length")
            if type(length) is not int or length != TEXT_EVAL_LENGTH:
                violations.append({"field": f"{row_id}.length", "reason": "not 3145728"})
        if row_id.startswith("silesia-"):
            _empty_or_text(
                row.get("member_md5", ""),
                f"{row_id}.member_md5",
                unresolved,
                violations,
                empty_unresolved=True,
            )
        if row.get("class") == "pcm" and row.get("transform") == "slice":
            violations.append({"field": f"{row_id}.transform", "reason": "pcm slice"})
        if row.get("class") == "executable" and row.get("role") == "evaluation":
            if "length" not in parameters:
                executable_length_missing = True
            else:
                length = parameters.get("length")
                if type(length) is not int or length < 1:
                    violations.append(
                        {
                            "field": "executable.eval.slice_length",
                            "reason": "invalid",
                        }
                    )
                else:
                    executable_lengths.append(length)
        if row_id == "flac-subset":
            members = row.get("members")
            usable = (
                isinstance(members, list)
                and members != []
                and all(type(item) is str and item != "" for item in members)
            )
            if not usable:
                unresolved.append(_unresolved("flac-subset.members", "absent"))

    if executable_length_missing or executable_lengths == []:
        unresolved.append(_unresolved("executable.eval.slice_length", "absent"))
    if len(set(executable_lengths)) > 1:
        violations.append(
            {"field": "executable.eval.slice_length", "reason": "lengths differ"}
        )

    included = sorted(row_id for row_id, row in by_id.items() if counts_toward_sm1(row))
    excluded = sorted(
        row_id for row_id, row in by_id.items() if not counts_toward_sm1(row)
    )
    unresolved.sort(key=lambda item: item["field"])
    violations.sort(key=lambda item: item["field"])
    ready = unresolved == [] and violations == [] and k_status == "confirmed"
    return {
        "corpus_gate_ready": ready,
        "counts_toward_sm1": included,
        "excluded_from_sm1": excluded,
        "k": {"bytes": k_bytes, "status": k_status},
        "schema": 1,
        "scope": "corpus",
        "unresolved": unresolved,
        "violations": violations,
    }
