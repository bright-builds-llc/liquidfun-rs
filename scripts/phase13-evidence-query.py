#!/usr/bin/env python3
"""Query Phase 13 provider metadata through the shared bounded process supervisor."""

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / "phase15-candidate-evidence"))
from common import confined, execute


def query(endpoint, retained, maybe_command=None, timeout=120):
    """Retain bounded metadata and classified process outcomes before returning."""
    retained = confined(retained)
    logs = confined(retained.parent / (retained.stem + "-query"))
    command = maybe_command
    if command is None:
        command = ["gh", "api", "--hostname", "github.com", endpoint]
    return execute(
        command,
        logs,
        limit=1024 * 1024,
        destination=retained,
        timeout=timeout,
    )


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise ValueError("expected exact API endpoint and retained output path")
    query(sys.argv[1], Path(sys.argv[2]))
