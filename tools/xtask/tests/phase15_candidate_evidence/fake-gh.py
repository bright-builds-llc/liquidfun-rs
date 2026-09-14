#!/usr/bin/env python3
"""Deterministic gh substitute; refuses requests absent from the test fixture."""

import json
import os
from pathlib import Path
import sys
from datetime import datetime, timezone

state_path = Path(os.environ["PHASE15_FAKE_STATE"])
state = json.loads(state_path.read_text())
with (state_path.parent / "calls.jsonl").open("a") as stream:
    stream.write(json.dumps(sys.argv[1:]) + "\n")
if sys.argv[1:3] == ["workflow", "run"]:
    created = datetime.now(timezone.utc).isoformat()
    for response in state["responses"].values():
        if isinstance(response, dict) and "workflow_runs" in response:
            for run in response["workflow_runs"]:
                run["created_at"] = created
        if isinstance(response, dict) and "head_sha" in response:
            response["created_at"] = created
    state_path.write_text(json.dumps(state))
    if not state.get("missing_dispatch_url"):
        print("https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/42")
    if state.get("duplicate_dispatch_url"):
        print("https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/43")
    sys.exit(state.get("dispatch_exit", 0))
endpoint = sys.argv[2]
key = endpoint.split("?", 1)[0]
if key not in state["responses"]:
    print("unexpected fake gh request: " + endpoint, file=sys.stderr)
    sys.exit(3)
response = state["responses"][key]
if isinstance(response, dict) and "binary_file" in response:
    sys.stdout.buffer.write(Path(response["binary_file"]).read_bytes())
else:
    print(json.dumps(response))
