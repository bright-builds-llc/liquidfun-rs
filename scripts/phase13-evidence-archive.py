#!/usr/bin/env python3
"""Bind a provider ZIP digest to the exact checked Phase 13 bundle, without extraction."""

import datetime
import hashlib
import json
import pathlib
import stat
import sys
import zipfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "phase15-candidate-evidence"))
from common import normalized

MAX_ARCHIVE = 128 * 1024 * 1024
MAX_MEMBER = 64 * 1024 * 1024
MAX_TOTAL = 512 * 1024 * 1024
MAX_ENTRIES = 4096


def require(condition, message):
    if not condition:
        raise ValueError(message)


def regular_path(path):
    for ancestor in (path, *path.parents):
        require(not ancestor.is_symlink(), f"symlink path component: {ancestor}")
    require(path.is_file(), f"not a regular file: {path}")


def digest_stream(stream):
    digest = hashlib.sha256()
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
        digest.update(chunk)
    return digest.hexdigest()


def validate(archive, root, provider_digest, expires_at):
    regular_path(archive)
    for ancestor in (root, *root.parents):
        require(not ancestor.is_symlink(), f"symlink bundle component: {ancestor}")
    require(root.is_dir(), "bundle root is not a directory")
    require(archive.stat().st_size <= MAX_ARCHIVE, "archive exceeds size bound")
    expiration = datetime.datetime.strptime(expires_at, "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=datetime.timezone.utc
    )
    require(expiration > datetime.datetime.now(datetime.timezone.utc), "artifact expired")
    with archive.open("rb") as raw:
        digest = digest_stream(raw)
        require(provider_digest == f"sha256:{digest}", "provider archive digest mismatch")
        raw.seek(0)
        with zipfile.ZipFile(raw) as zipped:
            entries = zipped.infolist()
            require(0 < len(entries) <= MAX_ENTRIES, "archive entry count exceeds bound")
            require(sum(item.file_size for item in entries) <= MAX_TOTAL, "expanded size exceeds bound")
            names = set()
            files = {}
            for item in entries:
                require(not item.flag_bits & 1, "encrypted archive member")
                name = item.filename
                try:
                    pure = normalized(name[:-1] if item.is_dir() else name)
                except ValueError as error:
                    raise ValueError(f"unsafe archive path: {error}") from error
                require(str(pure) not in names, "duplicate archive member")
                names.add(str(pure))
                mode = stat.S_IFMT(item.external_attr >> 16)
                require(mode in (0, stat.S_IFREG, stat.S_IFDIR), "non-regular archive member")
                require(item.file_size <= MAX_MEMBER, "member exceeds size bound")
                if item.is_dir():
                    require(mode in (0, stat.S_IFDIR), "invalid directory mode")
                    continue
                require(mode != stat.S_IFDIR, "directory encoded as file")
                target = root.joinpath(*pure.parts)
                regular_path(target)
                require(target.stat().st_size == item.file_size, "bundle member size mismatch")
                with zipped.open(item) as source, target.open("rb") as extracted:
                    member_digest = digest_stream(source)
                    require(member_digest == digest_stream(extracted), "bundle member bytes mismatch")
                files[str(pure)] = member_digest
    actual = set()
    for entry in root.rglob("*"):
        require(not entry.is_symlink(), "symlink in extracted bundle")
        if entry.is_dir():
            continue
        regular_path(entry)
        actual.add(entry.relative_to(root).as_posix())
        require(len(actual) <= MAX_ENTRIES, "bundle entry count exceeds bound")
    require(actual == set(files), "archive and extracted bundle file sets differ")
    return {"archive_sha256": digest, "archive_path": str(archive), "files": files}


if __name__ == "__main__":
    require(len(sys.argv) == 5, "expected archive, bundle root, provider digest and expiration")
    print(json.dumps(validate(pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3], sys.argv[4]), sort_keys=True))
