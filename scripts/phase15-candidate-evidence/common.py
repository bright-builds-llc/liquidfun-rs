"""Bounded local I/O shared by candidate orchestration adapters."""

from pathlib import Path
import hashlib
import json
import os
import re
import selectors
import signal
import subprocess
import time
import uuid

REPOSITORY = "bright-builds-llc/liquidfun-rs"
ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ("platform", "oracle", "safety", "fuzz", "regressions", "coverage", "performance")
MAX_FILE = 64 * 1024 * 1024
MAX_ARCHIVE = 256 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def candidate(value):
    require(re.fullmatch(r"[0-9a-f]{40}", value), "candidate must be full lowercase SHA")
    return value


def positive(value):
    require(re.fullmatch(r"[1-9][0-9]*", str(value)), "run/attempt ID must be positive")
    return int(value)


def normalized(value):
    require(isinstance(value, str) and value and not value.startswith("/"), "relative path required")
    require(all(part not in ("", ".", "..") for part in value.split("/")), "non-normalized path")
    require(not any(char in value for char in "\\\r\n\x00\t"), "unsafe path character")
    return Path(value)


def confined(value):
    path = Path(value)
    if path.is_absolute():
        value = str(path.relative_to(ROOT))
    relative = normalized(value)
    require(relative.parts[0] == "target" and len(relative.parts) > 1, "attempt must be beneath target/")
    path = ROOT
    for part in relative.parts:
        path = path / part
        require(not path.is_symlink(), "symbolic link in attempt path")
    return path


def read_json(path):
    require(path.is_file() and not path.is_symlink() and path.stat().st_size <= 8 * 1024 * 1024,
            f"missing or unbounded JSON: {path}")
    return json.loads(path.read_bytes())


def write_json(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def execute(args, logs, limit=8 * 1024 * 1024, destination=None, timeout=120):
    """Preserve bounded stdout/stderr and fail on timeout, overflow, or nonzero status."""
    logs.mkdir(parents=True, exist_ok=True)
    token = uuid.uuid4().hex
    output = destination if destination is not None else logs / (token + ".stdout")
    error = logs / (token + ".stderr")
    with output.open("xb") as stdout, error.open("xb") as stderr:
        process = subprocess.Popen(args, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   start_new_session=True)
        started = time.monotonic()
        violation = None
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ, [stdout, limit, 0])
            selector.register(process.stderr, selectors.EVENT_READ, [stderr, MAX_FILE, 0])
            while selector.get_map():
                if time.monotonic() - started > timeout:
                    violation = "command timed out"
                    break
                for key, _ in selector.select(0.05):
                    chunk = os.read(key.fileobj.fileno(), 65536)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    stream, bound, count = key.data
                    stream.write(chunk[:max(0, bound - count)])
                    key.data[2] += len(chunk)
                    if key.data[2] > bound:
                        violation = "command output exceeds bound"
                        break
                if violation:
                    break
        if violation:
            terminate_group(process)
        try:
            code = process.wait(timeout=max(0.1, timeout - (time.monotonic() - started)))
        except subprocess.TimeoutExpired:
            terminate_group(process)
            code = process.wait()
            violation = "command timed out"
        process.stdout.close()
        process.stderr.close()
    write_json(logs / (token + ".command.json"), {
        "args": args, "exit_code": code, "failure": violation,
        "stdout": str(output.relative_to(ROOT)), "stderr": str(error.relative_to(ROOT)),
    })
    require(violation is None and output.stat().st_size <= limit and error.stat().st_size <= MAX_FILE,
            violation or "command output exceeds bound")
    require(code == 0, f"command failed ({code}); retained logs: {error}")
    return output


def terminate_group(process):
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        require(process.poll() is not None, "command process disappeared before terminal status")


def api(endpoint, logs):
    return read_json(execute(["gh", "api", endpoint], logs))


def verify_repository(logs):
    data = api(f"repos/{REPOSITORY}", logs)
    require(data.get("full_name") == REPOSITORY, "provider repository differs")


def timestamp(value):
    from datetime import datetime
    return datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp()
