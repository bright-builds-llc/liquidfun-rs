"""Restore explicitly attested release inputs before checking ready public docs."""

import hashlib
import json
import re
import sys
import uuid

from common import (MAX_ARCHIVE, REPOSITORY, ROOT, api, candidate, confined, digest,
                    execute, normalized, positive, read_json, require, verify_repository, write_json)
from dispatch import validate_ref, validate_run
from payloads import unpack
from retention import check_artifact

RECORDS = ("source-candidate.json", "candidate-manifest.json", "audit-report.json")
RELEASE_KINDS = {"package", "docs", "notices", "corpus_closure", "compatibility_closure"}


def marker(text, label):
    lines = [line for line in text.splitlines() if line.startswith(label + ":")]
    require(len(lines) == 1, f"require exactly one {label} marker")
    match = re.fullmatch(re.escape(label) + r": `([0-9a-f]{40})`", lines[0])
    require(match is not None, f"invalid {label} marker")
    return match.group(1)


def explicit_attestation(text):
    if not any(line.startswith("Attestation commit:") for line in text.splitlines()):
        return None
    return marker(text, "Attestation commit")


def tracked_bytes(relative):
    path = ROOT
    for part in normalized(relative).parts:
        path /= part
        require(not path.is_symlink(), "tracked record path contains a symbolic link")
    require(path.is_file() and path.stat().st_size <= 4 * 1024**2, "missing or oversized tracked record")
    return path.read_bytes()


def git(args, logs, limit=8 * 1024**2):
    return execute(["git", "-C", str(ROOT), *args], logs, limit).read_bytes()


def preflight(attestation, logs):
    candidate(attestation)
    git(["merge-base", "--is-ancestor", attestation, "HEAD"], logs)
    records = {}
    for name in RECORDS:
        relative = "reference/release/" + name
        data = tracked_bytes(relative)
        object_name = attestation + ":" + relative
        require(int(git(["cat-file", "-s", object_name], logs)) == len(data), "A record byte size differs")
        require(git(["cat-file", "blob", object_name], logs, 4 * 1024**2) == data, "A record bytes differ")
        records[name] = data
    source = json.loads(records[RECORDS[0]])
    require(source.get("schema_version") == 1 and source.get("ready") is True, "source record is not ready")
    sha = candidate(source["source_candidate_commit"])
    require(sha != attestation, "A must differ from source C")
    git(["merge-base", "--is-ancestor", sha, attestation], logs)
    tree = git(["ls-tree", "-r", "-z", "--full-tree", sha], logs)
    require(hashlib.sha256(tree).hexdigest() == source["source_tree_sha256"], "source tree hash differs")
    for name, field in [(RECORDS[1], "candidate_manifest_sha256"), (RECORDS[2], "audit_report_sha256")]:
        require(hashlib.sha256(records[name]).hexdigest() == source[field], "tracked record hash differs")
    for relative in ("README.md", "COMPATIBILITY.md", "RELEASE.md"):
        text = tracked_bytes(relative).decode()
        require(marker(text, "Source candidate") == sha and marker(text, "Attestation commit") == attestation,
                "public C/A identities differ")
        require(text.splitlines().count("Status: **release-ready**") == 1, "missing or ambiguous ready status")
    manifest = json.loads(records[RECORDS[1]])
    require(manifest.get("candidate_commit") == sha and len(manifest["items"]) == 19, "manifest candidate or count differs")
    release = [item for item in manifest["items"] if item["producer"]["workflow"] == "release.yml"]
    require(len(release) == 5 and {item["kind"] for item in release} == RELEASE_KINDS, "release producer set differs")
    runs = {positive(item["producer"]["run_id"]) for item in release}
    require(len(runs) == 1, "release producer run IDs differ")
    base = f"target/phase12-release/{sha}"
    for item in manifest["items"]:
        relative = normalized(item["artifact_path"])
        require(str(relative.parent) == base + "/artifacts" and relative.suffix == ".json", "artifact path is outside fixed release root")
    return sha, runs.pop(), records, manifest


def release_run(run_id, sha, logs):
    verify_repository(logs)
    workflow = api(f"repos/{REPOSITORY}/actions/workflows/release.yml", logs)
    require(workflow.get("path") == ".github/workflows/release.yml", "release workflow path differs")
    run = api(f"repos/{REPOSITORY}/actions/runs/{run_id}", logs)
    ref = validate_ref(run["head_branch"])
    validate_run(run, {"workflow": "release.yml", "candidate": sha, "ref": ref}, workflow["id"], terminal=True)
    require(run["id"] == run_id, "provider run ID differs")
    jobs = api(f"repos/{REPOSITORY}/actions/runs/{run_id}/attempts/{run['run_attempt']}/jobs?per_page=100", logs)
    require(jobs["total_count"] == len(jobs["jobs"]) == 1, "release job cardinality differs")
    job = jobs["jobs"][0]
    require(job.get("name") == "Construct frozen release-candidate evidence" and job.get("head_sha") == sha
            and job.get("run_id") == run_id and job.get("run_attempt") == run["run_attempt"]
            and job.get("status") == "completed" and job.get("conclusion") == "success", "release job identity differs")
    return run


def restore(sha, run_id, records, manifest, attempt):
    logs = attempt / "logs"
    destination = confined(str(ROOT / "target/phase12-release" / sha))
    require(not destination.exists(), "release restoration destination already exists")
    run = release_run(run_id, sha, logs)
    listing = api(f"repos/{REPOSITORY}/actions/runs/{run_id}/artifacts?per_page=100", logs)
    require(listing["total_count"] == len(listing["artifacts"]) <= 100, "incomplete release artifact listing")
    name = f"phase12-release-{run_id}-{sha}"
    matches = [item for item in listing["artifacts"] if item.get("name") == name]
    require(len(matches) == 1, "missing or duplicated exact release artifact")
    item = matches[0]
    check_artifact(item, run, name)
    require(item["workflow_run"].get("head_branch") == run["head_branch"], "artifact ref metadata differs")
    archive = attempt / (name + ".zip")
    execute(["gh", "api", f"repos/{REPOSITORY}/actions/artifacts/{item['id']}/zip"], logs, MAX_ARCHIVE, archive)
    require(archive.stat().st_size == item["size_in_bytes"], "release archive size differs")
    require(item.get("digest") == "sha256:" + digest(archive), "release archive SHA-256 differs")
    latest = release_run(run_id, sha, logs)
    require(latest["run_attempt"] == run["run_attempt"] and latest["head_branch"] == run["head_branch"], "release attempt/ref changed during restoration")
    fresh = api(f"repos/{REPOSITORY}/actions/artifacts/{item['id']}", logs)
    check_artifact(fresh, latest, name)
    require(fresh.get("digest") == item["digest"] and fresh.get("id") == item["id"]
            and fresh.get("size_in_bytes") == archive.stat().st_size
            and fresh["workflow_run"].get("head_branch") == latest["head_branch"], "release artifact changed during restoration")
    write_json(attempt / "provider.json", {"run": run, "artifact": fresh})
    unpack(archive, destination)
    for name in RECORDS[1:]:
        require((destination / name).read_bytes() == records[name], "downloaded record differs from A")
    identity = read_json(destination / "audit-identity.json")
    require(identity.get("candidate_commit") == sha and positive(identity.get("run_id")) == run_id
            and identity.get("producer_workflow") == "release.yml" and identity.get("producer_job") == "release-candidate"
            and identity.get("ready") is True, "release audit identity differs")
    for name, field in [(RECORDS[1], "manifest_sha256"), (RECORDS[2], "audit_report_sha256")]:
        require(identity.get(field) == hashlib.sha256(records[name]).hexdigest(), "release identity record digest differs")
    require(identity.get("package_sha256") == digest(destination / "package/liquidfun.crate"), "release identity package differs")
    for item in manifest["items"]:
        if item["kind"] == "package":
            artifact = read_json(ROOT / item["artifact_path"])
            require(artifact["claims"]["archive_path"] == f"target/phase12-release/{sha}/package/liquidfun.crate", "package path is outside fixed release root")


def main():
    require(len(sys.argv) == 1, "docs CI check accepts no arguments")
    attestation = explicit_attestation(tracked_bytes("README.md").decode())
    attempt = confined(str(ROOT / "target/phase15-docs-ci" / uuid.uuid4().hex))
    attempt.mkdir(parents=True, exist_ok=False)
    command = ["cargo", "xtask", "docs", "check"]
    if attestation is not None:
        sha, run_id, records, manifest = preflight(attestation, attempt / "logs")
        write_json(attempt / "intent.json", {"candidate": sha, "attestation": attestation, "release_run_id": run_id})
        restore(sha, run_id, records, manifest, attempt)
        command += ["--attestation-commit", attestation]
    output = execute(command, attempt / "logs", timeout=600)
    print(output.read_text(), end="")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"docs CI: {error}", file=sys.stderr)
        sys.exit(1)
