"""Retain exact provider archives and independently revalidate them before use."""

import uuid

from common import (MAX_ARCHIVE, REPOSITORY, ROOT, WORKFLOWS, api, candidate, confined,
                    digest, execute, positive, read_json, require, timestamp, verify_repository, write_json)
from dispatch import validate_ref, validate_run
from payloads import check_embedded_attempt, inventory, names, unpack

JOB_NAMES = {
    "oracle": ["Phase 11 canonical Linux oracle", "Phase 11 fail-fast sanitizer"],
    "safety": ["Miri pure-Rust subsets", "Rust address sanitizer subsets"],
    "coverage": ["Rust source coverage", "C++ oracle source coverage", "Differential semantic leaves"],
    "regressions": ["Replay every reviewed named regression"],
    "performance": ["Produce candidate-bound paired performance evidence"],
    "fuzz": ["Fuzz " + target for target in ("protocol", "shapes_collision", "world_mutation", "particles", "groups_ownership")],
    "platform": ["Create one reviewed package artifact", "Verify exact artifact on canonical Rust 1.92 Linux",
                 *["Verify exact artifact on " + target for target in ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-pc-windows-msvc")],
                 "Resolve Intel macOS native evidence freshness"],
}


def current_run(workflow, run_id, sha, ref, logs):
    verify_repository(logs)
    identity = api(f"repos/{REPOSITORY}/actions/workflows/{workflow}.yml", logs)
    require(identity.get("path") == f".github/workflows/{workflow}.yml", "workflow path differs")
    run = api(f"repos/{REPOSITORY}/actions/runs/{positive(run_id)}", logs)
    validate_run(run, {"workflow": workflow + ".yml", "candidate": sha, "ref": ref}, identity["id"], terminal=True)
    require(run["id"] == int(run_id), "returned run ID differs")
    jobs = api(f"repos/{REPOSITORY}/actions/runs/{run_id}/attempts/{run['run_attempt']}/jobs?per_page=100", logs)
    require(jobs["total_count"] == len(jobs["jobs"]) <= 100, "incomplete job listing")
    expected = JOB_NAMES[workflow].copy()
    if workflow == "platform":
        support = read_json(ROOT / "reference/platform/support.json")
        expected.append("Record unavailable Intel macOS support" if support["conditional_targets"][0]["native_evidence"] is None
                        else "Verify exact artifact on conditional Intel macOS")
    for name in expected:
        matches = [job for job in jobs["jobs"] if job["name"] == name]
        require(len(matches) == 1, f"missing/duplicated provider job: {name}")
        job = matches[0]
        require(job.get("head_sha") == sha and job.get("run_id") == int(run_id)
                and job.get("run_attempt") == run["run_attempt"]
                and job.get("status") == "completed" and job.get("conclusion") == "success",
                "provider job identity or terminal status differs")
    return run, jobs


def check_artifact(item, run, name):
    require(item.get("name") == name and item.get("expired") is False, "artifact missing or expired")
    require(0 < item.get("size_in_bytes", 0) <= MAX_ARCHIVE, "artifact size exceeds bound")
    require(item.get("workflow_run", {}).get("id") == run["id"]
            and item["workflow_run"].get("head_sha") == run["head_sha"], "artifact run identity differs")
    require(timestamp(item["created_at"]) >= timestamp(run["run_started_at"]), "artifact predates current attempt")
    require(timestamp(item["expires_at"]) > __import__("time").time(), "artifact already expired")
    positive(item["id"])


def collect(args):
    sha = candidate(args.candidate)
    validate_ref(args.ref)
    require(args.workflow in WORKFLOWS, "collect expects producer workflow stem")
    run_id = positive(args.run_id)
    attempt = positive(args.run_attempt)
    root = confined(args.attempt_root)
    root.mkdir(parents=True, exist_ok=True)
    collection = {"candidate": sha, "repository": REPOSITORY}
    if (root / "collection.json").exists():
        require(read_json(root / "collection.json") == collection, "collection candidate differs")
    else:
        write_json(root / "collection.json", collection)
    producer = confined(str(root / "producers" / args.workflow))
    producer.mkdir(parents=True, exist_ok=False)
    bundle = producer / "bundle"
    logs = bundle / "logs"
    write_json(producer / "intent.json", {**collection, "ref": args.ref, "workflow": args.workflow,
                                         "run_id": run_id, "run_attempt": attempt})
    run, jobs = current_run(args.workflow, run_id, sha, args.ref, logs)
    require(run["run_attempt"] == attempt, "requested attempt is not provider current attempt")
    write_json(bundle / "run.json", run)
    write_json(bundle / "jobs.json", jobs)
    listing = api(f"repos/{REPOSITORY}/actions/runs/{run_id}/artifacts?per_page=100", logs)
    require(listing["total_count"] == len(listing["artifacts"]) <= 100, "incomplete artifact listing")
    write_json(bundle / "artifacts.json", listing)
    (bundle / "archives").mkdir()
    for name in names(args.workflow, run_id, sha):
        matches = [item for item in listing["artifacts"] if item.get("name") == name]
        require(len(matches) == 1, "missing or duplicated exact artifact name")
        item = matches[0]
        check_artifact(item, run, name)
        archive = bundle / "archives" / (name + ".zip")
        execute(["gh", "api", "--hostname", "github.com", f"repos/{REPOSITORY}/actions/artifacts/{item['id']}/zip"],
                logs, MAX_ARCHIVE, archive)
        require(archive.stat().st_size == item["size_in_bytes"], "partial artifact download")
        require(item.get("digest") == "sha256:" + digest(archive), "provider archive SHA-256 differs")
        unpack(archive, bundle / "payloads" / name)
        check_embedded_attempt(bundle / "payloads" / name, attempt)
    execute(["gh", "api", "--hostname", "github.com", f"repos/{REPOSITORY}/actions/runs/{run_id}/attempts/{attempt}/logs"],
            logs, MAX_ARCHIVE, bundle / "provider-logs.zip")
    latest, _ = current_run(args.workflow, run_id, sha, args.ref, logs)
    require(latest["run_attempt"] == attempt, "provider attempt changed during collection")
    write_json(producer / "retained.json", {"intent": read_json(producer / "intent.json"), "files": inventory(bundle)})
    print(f"retained {args.workflow} run {run_id} attempt {attempt}; release audit still required")


def validate_retained(args):
    sha = candidate(args.candidate)
    root = confined(args.attempt_root)
    require(read_json(root / "collection.json") == {"candidate": sha, "repository": REPOSITORY}, "collection identity differs")
    require(set(path.name for path in (root / "producers").iterdir()) == set(WORKFLOWS), "all seven producers required")
    logs = root / "validations" / uuid.uuid4().hex
    # Payload authority is checked by the frozen checkout's existing closed validator.
    head = execute(["git", "rev-parse", "HEAD"], logs).read_text().strip()
    require(head == sha, "validate-retained requires checkout of the explicit candidate")
    dirty = execute(["git", "status", "--porcelain", "--untracked-files=all"], logs).read_text()
    require(not dirty, "validate-retained requires a clean candidate checkout")
    version = execute([args.validator_bash, "--version"], logs).read_text()
    require(version.startswith("GNU bash, version ") and int(version.split("version ", 1)[1].split(".")[0]) >= 4,
            "semantic validator requires GNU Bash >=4")
    runs = []
    for workflow in WORKFLOWS:
        producer = confined(str(root / "producers" / workflow))
        bundle = producer / "bundle"
        retained = read_json(producer / "retained.json")
        intent = retained["intent"]
        require(intent == read_json(producer / "intent.json") and intent["candidate"] == sha
                and intent["repository"] == REPOSITORY and intent["workflow"] == workflow, "retained identity differs")
        require(retained["files"] == inventory(bundle), "retained payload/log/archive inventory differs")
        run, _ = current_run(workflow, intent["run_id"], sha, intent["ref"], logs)
        require(run["run_attempt"] == intent["run_attempt"], "retained attempt is stale at provider")
        artifacts = read_json(bundle / "artifacts.json")["artifacts"]
        for name in names(workflow, run["id"], sha):
            matches = [item for item in artifacts if item.get("name") == name]
            require(len(matches) == 1, "retained artifact metadata is missing or duplicated")
            item = matches[0]
            fresh = api(f"repos/{REPOSITORY}/actions/artifacts/{positive(item['id'])}", logs)
            check_artifact(fresh, run, name)
            require(fresh["id"] == item["id"], "provider artifact ID differs")
            archive = bundle / "archives" / (name + ".zip")
            require(fresh.get("digest") == "sha256:" + digest(archive), "retained bytes differ from provider digest")
        runs.append(str(run["id"]))
    destination = confined(args.destination) if args.destination else root / "materialized"
    destination.mkdir(parents=True, exist_ok=False)
    for workflow, run in zip(WORKFLOWS, runs):
        bundle = root / "producers" / workflow / "bundle"
        for name in names(workflow, run, sha):
            unpack(bundle / "archives" / (name + ".zip"), destination / name)
    execute([args.validator_bash, str(ROOT / "scripts/phase15-candidate-evidence/validate.sh"),
             sha, str(destination), *runs], logs, timeout=600)
    print(f"retained producer identities validated: {destination}; release audit and C/A attestation still required")
