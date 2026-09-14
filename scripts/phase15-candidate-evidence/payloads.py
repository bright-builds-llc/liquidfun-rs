"""Closed artifact names, immutable inventories, and bounded ZIP materialization."""

import stat
import zipfile

from common import MAX_ARCHIVE, MAX_FILE, ROOT, digest, normalized, read_json, require


def names(workflow, run, sha):
    suffix = f"{run}-{sha}"
    if workflow == "platform":
        targets = ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
                   "aarch64-apple-darwin", "x86_64-pc-windows-msvc")
        support = read_json(ROOT / "reference/platform/support.json")
        conditional = "x86_64-apple-darwin"
        if support["conditional_targets"][0]["native_evidence"] is None:
            conditional += "-downgrade"
        return [f"phase12-package-{suffix}", f"phase12-platform-msrv-{suffix}",
                *[f"phase12-platform-{target}-{suffix}" for target in targets],
                f"phase12-platform-{conditional}-{suffix}"]
    prefixes = {
        "oracle": ["phase11-canonical", "phase11-sanitizer"],
        "safety": ["phase12-miri", "phase12-rust-sanitizer"],
        "fuzz": ["fuzz-" + target for target in ("protocol", "shapes_collision", "world_mutation", "particles", "groups_ownership")],
        "coverage": [f"phase12-{kind}-coverage" for kind in ("rust", "cpp", "differential")],
        "performance": ["phase12-performance"],
    }
    if workflow == "regressions":
        return ["phase12-regressions-" + sha]
    return [prefix + "-" + suffix for prefix in prefixes[workflow]]


def inventory(root):
    result = {}
    total = 0
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), "retained symbolic link")
        if path.is_dir():
            continue
        relative = path.relative_to(root).as_posix()
        normalized(relative)
        require(path.is_file(), "retained special file")
        limit = MAX_ARCHIVE if path.suffix == ".zip" else MAX_FILE
        size = path.stat().st_size
        require(size <= limit, "retained file exceeds bound")
        total += size
        require(len(result) < 8192 and total <= 2 * 1024**3, "retained inventory exceeds bound")
        result[relative] = {"bytes": size, "sha256": digest(path)}
    return result


def unpack(archive, destination):
    require(archive.is_file() and not archive.is_symlink() and archive.stat().st_size <= MAX_ARCHIVE,
            "missing or oversized archive")
    destination.mkdir(parents=True, exist_ok=False)
    with zipfile.ZipFile(archive) as bundle:
        entries = bundle.infolist()
        require(0 < len(entries) <= 4096, "archive member cardinality differs")
        seen = set()
        total = 0
        for entry in entries:
            relative = normalized(entry.filename.rstrip("/") if entry.is_dir() else entry.filename)
            require(str(relative) not in seen, "duplicate ZIP member")
            seen.add(str(relative))
            mode = entry.external_attr >> 16
            require(stat.S_IFMT(mode) in (0, stat.S_IFDIR, stat.S_IFREG), "ZIP link or special member")
            require(not entry.flag_bits & 1, "encrypted ZIP member")
            require(entry.file_size <= MAX_FILE, "ZIP member exceeds byte bound")
            total += entry.file_size
            require(total <= 512 * 1024**2, "expanded ZIP exceeds bound")
        for entry in entries:
            output = destination / entry.filename
            if entry.is_dir():
                output.mkdir(parents=True, exist_ok=True)
                continue
            output.parent.mkdir(parents=True, exist_ok=True)
            with bundle.open(entry) as source, output.open("xb") as target:
                count = 0
                while chunk := source.read(1024 * 1024):
                    count += len(chunk)
                    require(count <= MAX_FILE, "expanded member exceeds bound")
                    target.write(chunk)
            require(count == entry.file_size, "incomplete ZIP member")


def check_embedded_attempt(payload, attempt):
    for path in payload.rglob("*.json"):
        if path.name not in ("identity.json", "producer-identity.json", "terminal.json"):
            continue
        value = read_json(path)
        if "run_attempt" in value:
            require(value["run_attempt"] == attempt, "payload has stale provider run attempt")
