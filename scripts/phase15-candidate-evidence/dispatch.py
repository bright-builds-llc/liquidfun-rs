"""Intent-first dispatch and read-only uncertain-dispatch reconciliation."""

from datetime import datetime, timezone
import re
from urllib.parse import quote

from common import (REPOSITORY, ROOT, WORKFLOWS, api, candidate, confined, execute,
                    positive, read_json, require, timestamp, verify_repository, write_json)


def workflow_name(value):
    require(value in [name + ".yml" for name in (*WORKFLOWS, "release")], "unsupported workflow")
    return value


def validate_ref(ref):
    require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_./-]*", ref) and ".." not in ref,
            "invalid explicit ref")
    return ref


def inputs_for(args):
    inputs = {}
    for value in args.input:
        key, separator, data = value.partition("=")
        require(separator and key not in inputs, "invalid or duplicate workflow input")
        inputs[key] = data
    expected = {"candidate_sha"}
    if args.workflow == "oracle.yml":
        expected = {"evidence_phase"}
        require(inputs == {"evidence_phase": "phase11"}, "oracle requires evidence_phase=phase11 only")
    elif args.workflow == "performance.yml":
        expected |= {"controlled_host_label", "controlled_host_identity"}
        require(inputs.get("controlled_host_label") == "performance-controlled-linux-x64", "wrong controlled host label")
        require(re.fullmatch(r"[0-9a-f]{64}", inputs.get("controlled_host_identity", "")), "missing controlled host identity")
    elif args.workflow == "release.yml":
        expected |= {name + "_run_id" for name in WORKFLOWS}
        for name in WORKFLOWS:
            positive(inputs.get(name + "_run_id", ""))
    require(set(inputs) == expected, "workflow inputs differ from declared interface")
    if "candidate_sha" in expected:
        require(inputs["candidate_sha"] == args.candidate, "input candidate differs")
    return inputs


def provider_identity(intent, logs):
    verify_repository(logs)
    workflow = api(f"repos/{REPOSITORY}/actions/workflows/{intent['workflow']}", logs)
    require(workflow.get("path") == ".github/workflows/" + intent["workflow"]
            and workflow.get("state") == "active", "workflow identity differs")
    resolved = api(f"repos/{REPOSITORY}/commits/{quote(intent['ref'], safe='')}", logs)
    require(resolved.get("sha") == intent["candidate"], "remote ref no longer resolves to candidate")
    return workflow


def validate_run(run, intent, workflow_id, terminal=False):
    require(run.get("repository", {}).get("full_name") == REPOSITORY
            and run.get("head_repository", {}).get("full_name") == REPOSITORY,
            "run repository differs")
    require(run.get("workflow_id") == workflow_id and run.get("path") == ".github/workflows/" + intent["workflow"],
            "run workflow differs")
    require(run.get("head_sha") == intent["candidate"] and run.get("head_branch") == intent["ref"],
            "run ref or candidate differs")
    require(run.get("event") == "workflow_dispatch", "run is not explicit dispatch")
    positive(run["id"])
    positive(run["run_attempt"])
    if "created_at" in intent:
        require(timestamp(run["created_at"]) >= timestamp(intent["created_at"])
                and timestamp(run["created_at"]) <= timestamp(intent["created_at"]) + 600,
                "run outside dispatch intent time window")
    if terminal:
        require(run.get("status") == "completed" and run.get("conclusion") == "success", "run not terminal success")


def dispatch(args):
    candidate(args.candidate)
    workflow_name(args.workflow)
    validate_ref(args.ref)
    inputs = inputs_for(args)
    root = confined(args.attempt_root)
    root.mkdir(parents=True, exist_ok=False)
    intent = {"repository": REPOSITORY, "candidate": args.candidate,
              "workflow": args.workflow, "ref": args.ref, "inputs": inputs}
    workflow = provider_identity(intent, root / "logs")
    # Persist before the only mutating provider call. Existing attempts are never dispatched again.
    intent["workflow_id"] = positive(workflow["id"])
    intent["created_at"] = datetime.now(timezone.utc).isoformat(timespec="seconds")
    write_json(root / "intent.json", intent)
    command = ["gh", "workflow", "run", args.workflow, "--repo", "github.com/" + REPOSITORY, "--ref", args.ref]
    for key, value in sorted(inputs.items()):
        command.extend(["--raw-field", key + "=" + value])
    execute(command, root / "logs", destination=root / "dispatch-response.txt")
    reconcile(args)


def reconcile(args):
    root = confined(args.attempt_root)
    intent = read_json(root / "intent.json")
    require(intent["candidate"] == candidate(args.candidate) and intent["repository"] == REPOSITORY,
            "attempt candidate/repository differs")
    workflow_name(intent["workflow"])
    validate_ref(intent["ref"])
    logs = root / "logs"
    verify_repository(logs)
    response = root / "dispatch-response.txt"
    require(response.is_file() and not response.is_symlink() and response.stat().st_size <= 8 * 1024 * 1024,
            "uncertain dispatch: missing bounded authoritative response; do not redispatch")
    urls = re.findall(r"https://github[.]com/[^\s/]+/[^\s/]+/actions/runs/[0-9]+", response.read_text())
    require(len(set(urls)) == 1 and urls[0].startswith(f"https://github.com/{REPOSITORY}/actions/runs/"),
            "uncertain dispatch: require one authoritative run URL in the persisted response; do not redispatch")
    run_id = positive(urls[0].rsplit("/", 1)[1])
    run = api(f"repos/{REPOSITORY}/actions/runs/{run_id}", logs)
    require(run.get("id") == run_id, "provider returned a different run ID")
    validate_run(run, intent, intent["workflow_id"])
    binding = {"run_id": run["id"], "run_attempt": run["run_attempt"], "candidate": args.candidate}
    if (root / "binding.json").exists():
        require(read_json(root / "binding.json") == binding, "bound run/attempt changed; use a new collection attempt")
    else:
        write_json(root / "binding.json", binding)
    print(f"reconciled run {run['id']} attempt {run['run_attempt']}; no readiness claim")
