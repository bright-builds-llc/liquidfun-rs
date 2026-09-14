"""Readiness CI restores only the explicitly attested release bundle."""

from pathlib import Path
from datetime import datetime, timedelta, timezone
import contextlib
import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts/phase15-candidate-evidence"))
import docs_ci


class DocsCiTests(unittest.TestCase):
    def test_explicit_marker_requires_one_full_identity(self):
        # Arrange / Act / Assert
        for text in ["Attestation commit: `HEAD`", ("Attestation commit: `" + "a" * 40 + "`\n") * 2,
                     "Attestation commit: `" + "a" * 40 + "`\nAttestation commit: `" + "b" * 40 + "`"]:
            with self.subTest(text=text), self.assertRaises(ValueError):
                docs_ci.explicit_attestation(text)

    def test_absent_marker_keeps_default_mode(self):
        # Arrange / Act / Assert
        self.assertIsNone(docs_ci.explicit_attestation("Status: **not release-ready**\n"))


class RestoreTests(unittest.TestCase):
    def setUp(self):
        self.directory = Path(tempfile.mkdtemp(prefix="docs-ci-", dir=ROOT / "target"))
        self.root = self.directory / "checkout"
        self.root.mkdir()
        self.bin = self.directory / "bin"
        self.bin.mkdir()
        (self.bin / "gh").symlink_to(Path(__file__).with_name("fake-gh.py"))
        (self.bin / "cargo").write_text('#!/usr/bin/env python3\nimport os,sys,json\nfrom pathlib import Path\np=Path(os.environ["PHASE15_FAKE_STATE"]).parent/"cargo.jsonl"\nwith p.open("a") as f: f.write(json.dumps(sys.argv[1:])+"\\n")\nprint("fixture full-audit invocation")\n')
        (self.bin / "cargo").chmod(0o755)
        self.state_path = self.directory / "state.json"
        self.environment = patch.dict(os.environ, {"PATH": str(self.bin) + os.pathsep + os.environ["PATH"],
                                                  "PHASE15_FAKE_STATE": str(self.state_path)})
        self.environment.start()
        self.root_patch = patch.object(docs_ci, "ROOT", self.root)
        self.root_patch.start()
        for name in ("README.md", "COMPATIBILITY.md", "RELEASE.md"):
            (self.root / name).write_text("Status: **not release-ready**\n")
        self.git("init", "--quiet")
        self.git("config", "user.name", "Docs CI Test")
        self.git("config", "user.email", "docs@example.invalid")
        self.git("add", ".")
        self.git("-c", "core.hooksPath=/dev/null", "commit", "--quiet", "-m", "C")
        self.sha = self.git("rev-parse", "HEAD").decode().strip()
        self.base = f"target/phase12-release/{self.sha}"
        kinds = sorted(docs_ci.RELEASE_KINDS) + [f"other-{number}" for number in range(14)]
        items = [{"kind": kind, "producer": {"workflow": "release.yml" if kind in docs_ci.RELEASE_KINDS else "oracle.yml", "run_id": "42"},
                  "artifact_path": self.base + "/artifacts/" + kind + ".json"} for kind in kinds]
        self.manifest = {"candidate_commit": self.sha, "items": items}
        self.records = self.root / "reference/release"
        self.records.mkdir(parents=True)
        (self.records / "audit-report.json").write_text(json.dumps({"decision": "ready", "candidate_commit": self.sha}))
        self.write_manifest()
        self.attestation = self.commit_records()
        self.project()
        self.git("add", "README.md", "COMPATIBILITY.md", "RELEASE.md")
        self.git("-c", "core.hooksPath=/dev/null", "commit", "--quiet", "-m", "D")
        self.archive = self.directory / "release.zip"
        self.make_archive()
        now = datetime.now(timezone.utc)
        repo = docs_ci.REPOSITORY
        self.api = f"repos/{repo}"
        self.run = {"id": 42, "workflow_id": 7, "run_attempt": 2, "repository": {"full_name": repo},
                    "head_repository": {"full_name": repo}, "head_sha": self.sha, "head_branch": "frozen-source",
                    "path": ".github/workflows/release.yml", "event": "workflow_dispatch", "status": "completed",
                    "conclusion": "success", "run_started_at": now.isoformat()}
        self.artifact = {"id": 77, "name": f"phase12-release-42-{self.sha}", "expired": False,
                         "size_in_bytes": self.archive.stat().st_size, "digest": "sha256:" + self.hash(self.archive.read_bytes()),
                         "created_at": now.isoformat(), "expires_at": (now + timedelta(days=1)).isoformat(),
                         "workflow_run": {"id": 42, "head_sha": self.sha, "head_branch": "frozen-source"}}
        self.state = {"responses": {
            self.api: {"full_name": repo}, self.api + "/actions/workflows/release.yml": {"id": 7, "path": ".github/workflows/release.yml"},
            self.api + "/actions/runs/42": self.run,
            self.api + "/actions/runs/42/attempts/2/jobs": {"total_count": 1, "jobs": [{"name": "Construct frozen release-candidate evidence",
                "head_sha": self.sha, "run_id": 42, "run_attempt": 2, "status": "completed", "conclusion": "success"}]},
            self.api + "/actions/runs/42/artifacts": {"total_count": 1, "artifacts": [self.artifact]},
            self.api + "/actions/artifacts/77": self.artifact,
            self.api + "/actions/artifacts/77/zip": {"binary_file": str(self.archive)},
        }}

    def tearDown(self):
        self.root_patch.stop()
        self.environment.stop()
        result = self._outcome.result
        if any(test is self for test, _ in result.errors + result.failures):
            print(f"preserved docs CI test fixture: {self.directory}", file=sys.stderr)
        else:
            shutil.rmtree(self.directory)

    def git(self, *args):
        return subprocess.run(["git", "-C", str(self.root), *args], check=True, capture_output=True, timeout=15).stdout

    @staticmethod
    def hash(data):
        return hashlib.sha256(data).hexdigest()

    def write_manifest(self):
        (self.records / "candidate-manifest.json").write_text(json.dumps(self.manifest))
        source = {"schema_version": 1, "ready": True, "source_candidate_commit": self.sha,
                  "source_tree_sha256": self.hash(self.git("ls-tree", "-r", "-z", "--full-tree", self.sha)),
                  "candidate_manifest_sha256": self.hash((self.records / "candidate-manifest.json").read_bytes()),
                  "audit_report_sha256": self.hash((self.records / "audit-report.json").read_bytes())}
        (self.records / "source-candidate.json").write_text(json.dumps(source))

    def commit_records(self):
        self.git("add", "reference/release")
        self.git("-c", "core.hooksPath=/dev/null", "commit", "--quiet", "-m", "A")
        return self.git("rev-parse", "HEAD").decode().strip()

    def project(self):
        for name in ("README.md", "COMPATIBILITY.md", "RELEASE.md"):
            (self.root / name).write_text(f"Status: **release-ready**\nSource candidate: `{self.sha}`\nAttestation commit: `{self.attestation}`\n")

    def make_archive(self, package_path=None):
        package = b"exact package fixture"
        identity = {"candidate_commit": self.sha, "run_id": "42", "producer_workflow": "release.yml",
                    "producer_job": "release-candidate", "ready": True,
                    "manifest_sha256": self.hash((self.records / "candidate-manifest.json").read_bytes()),
                    "audit_report_sha256": self.hash((self.records / "audit-report.json").read_bytes()),
                    "package_sha256": self.hash(package)}
        with zipfile.ZipFile(self.archive, "w") as bundle:
            for item in self.manifest["items"]:
                data = {"claims": {"archive_path": package_path or self.base + "/package/liquidfun.crate"}}
                bundle.writestr("artifacts/" + Path(item["artifact_path"]).name, json.dumps(data))
            for name in docs_ci.RECORDS[1:]:
                bundle.write(self.records / name, name)
            bundle.writestr("audit-identity.json", json.dumps(identity))
            bundle.writestr("package/liquidfun.crate", package)
            bundle.writestr("package/package-identity.json", "{}")

    def invoke(self, native=False):
        self.state_path.write_text(json.dumps(self.state))
        arguments = ["docs-ci", "--native-inventory"] if native else ["docs-ci"]
        with patch.object(sys, "argv", arguments), contextlib.redirect_stdout(io.StringIO()):
            docs_ci.main()

    def test_native_gate_restores_validates_and_rechecks_existing_exact_bytes(self):
        # Arrange / Act
        self.invoke(native=True)
        self.invoke(native=True)
        # Assert
        calls = [json.loads(line) for line in (self.directory / "cargo.jsonl").read_text().splitlines()]
        expected = [["xtask", "docs", "check", "--attestation-commit", self.attestation],
                    ["xtask", "inventory", "check", "--attestation-commit", self.attestation]]
        self.assertEqual(calls, expected * 2)
        self.assertEqual(len(list((self.root / "target/phase15-docs-ci").glob("*/*.zip"))), 2)

    def test_native_default_preserves_the_original_inventory_gate(self):
        # Arrange
        (self.root / "README.md").write_text("Status: **not release-ready**\n")
        # Act
        self.invoke(native=True)
        # Assert
        self.assertEqual(json.loads((self.directory / "cargo.jsonl").read_text()), ["xtask", "inventory", "check"])
        self.assertFalse((self.directory / "calls.jsonl").exists())

    def test_native_reuse_rejects_changed_restored_bytes(self):
        # Arrange
        self.invoke(native=True)
        (self.root / self.base / "package/liquidfun.crate").write_bytes(b"tampered")
        # Act / Assert
        with self.assertRaisesRegex(ValueError, "existing restored bytes differ"):
            self.invoke(native=True)

    def assert_rejected(self, marker):
        with self.assertRaisesRegex((ValueError, OSError), marker):
            self.invoke()
        self.assertFalse((self.directory / "cargo.jsonl").exists())

    def test_restores_exact_release_layout_then_invokes_explicit_full_validator(self):
        # Arrange / Act
        self.invoke()
        # Assert
        destination = self.root / self.base
        self.assertEqual(len(list((destination / "artifacts").glob("*.json"))), 19)
        self.assertEqual((destination / "package/liquidfun.crate").read_bytes(), b"exact package fixture")
        self.assertEqual(json.loads((self.directory / "cargo.jsonl").read_text()), ["xtask", "docs", "check", "--attestation-commit", self.attestation])
        self.assertEqual(len(list((self.root / "target/phase15-docs-ci").glob("*/*.zip"))), 1)
        with self.assertRaisesRegex(ValueError, "already exists"):
            self.invoke()

    def test_a_record_bytes_cannot_be_substituted(self):
        # Arrange / Act / Assert
        source = self.records / "source-candidate.json"
        source.write_bytes(source.read_bytes() + b" ")
        self.assert_rejected("A record byte size differs")

    def test_wrong_archive_digest_prevents_validation(self):
        # Arrange / Act / Assert
        self.artifact["digest"] = "sha256:" + "0" * 64
        self.assert_rejected("archive SHA-256 differs")

    def test_missing_archive_prevents_validation(self):
        # Arrange / Act / Assert
        self.state["responses"][self.api + "/actions/artifacts/77/zip"] = {"binary_file": str(self.directory / "missing.zip")}
        self.assert_rejected("command failed")

    def test_old_artifact_from_previous_attempt_is_rejected(self):
        # Arrange / Act / Assert
        self.artifact["created_at"] = (datetime.now(timezone.utc) - timedelta(days=1)).isoformat()
        self.assert_rejected("predates current attempt")

    def test_expired_artifact_is_rejected(self):
        # Arrange / Act / Assert
        self.artifact["expired"] = True
        self.assert_rejected("expired")

    def test_provider_attempt_job_must_match(self):
        # Arrange / Act / Assert
        self.state["responses"][self.api + "/actions/runs/42/attempts/2/jobs"]["jobs"][0]["run_attempt"] = 1
        self.assert_rejected("job identity differs")

    def test_package_path_cannot_escape_original_release_root(self):
        # Arrange / Act / Assert
        self.make_archive("target/other/liquidfun.crate")
        self.artifact.update(size_in_bytes=self.archive.stat().st_size, digest="sha256:" + self.hash(self.archive.read_bytes()))
        self.assert_rejected("package path is outside fixed release root")

    def test_manifest_paths_are_bound_before_download(self):
        # Arrange / Act / Assert
        self.manifest["items"][0]["artifact_path"] = "target/other/forged.json"
        self.write_manifest()
        self.attestation = self.commit_records()
        self.project()
        self.assert_rejected("artifact path is outside fixed release root")

    def test_manifest_requires_exact_nineteen_entries(self):
        # Arrange / Act / Assert
        self.manifest["items"].pop()
        self.write_manifest()
        self.attestation = self.commit_records()
        self.project()
        self.assert_rejected("manifest candidate or count differs")

    def test_missing_package_payload_prevents_validation(self):
        # Arrange
        with zipfile.ZipFile(self.archive) as original:
            members = [(entry.filename, original.read(entry)) for entry in original.infolist()
                       if entry.filename != "package/liquidfun.crate"]
        incomplete = self.directory / "incomplete.zip"
        with zipfile.ZipFile(incomplete, "w") as bundle:
            for name, data in members:
                bundle.writestr(name, data)
        self.state["responses"][self.api + "/actions/artifacts/77/zip"] = {"binary_file": str(incomplete)}
        self.artifact.update(size_in_bytes=incomplete.stat().st_size, digest="sha256:" + self.hash(incomplete.read_bytes()))
        # Act / Assert
        self.assert_rejected("No such file")

    def test_wrong_provider_source_candidate_prevents_download(self):
        # Arrange / Act / Assert
        self.run["head_sha"] = "b" * 40
        self.assert_rejected("run ref or candidate differs")

    def test_changed_provider_attempt_during_download_prevents_materialization(self):
        # Arrange
        original = docs_ci.release_run
        calls = 0

        def changed_attempt(*args):
            nonlocal calls
            calls += 1
            run = original(*args)
            if calls == 2:
                run["run_attempt"] += 1
            return run

        # Act / Assert
        with patch.object(docs_ci, "release_run", changed_attempt):
            self.assert_rejected("attempt/ref changed")
        self.assertFalse((self.root / self.base).exists())

    def test_failed_full_validator_remains_a_failure(self):
        # Arrange
        cargo = self.bin / "cargo"
        cargo.write_text(cargo.read_text() + "sys.exit(9)\n")
        # Act / Assert
        with self.assertRaisesRegex(ValueError, "command failed \\(9\\)"):
            self.invoke()
        self.assertTrue((self.directory / "cargo.jsonl").is_file())


if __name__ == "__main__":
    unittest.main()
