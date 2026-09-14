"""Exercise the production entrypoint without contacting GitHub."""

from pathlib import Path
from datetime import datetime, timedelta, timezone
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts/phase15-candidate-evidence"))
SHA = "a" * 40
REPO = "bright-builds-llc/liquidfun-rs"
API = f"repos/{REPO}"


class CandidateEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.directory = Path(tempfile.mkdtemp(prefix="candidate-", dir=ROOT / "target"))
        self.state_path = self.directory / "state.json"
        self.bin = self.directory / "bin"
        self.bin.mkdir()
        (self.bin / "gh").symlink_to(Path(__file__).with_name("fake-gh.py"))
        self.environment = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                                PHASE15_FAKE_STATE=str(self.state_path))
        self.now = datetime.now(timezone.utc)
        self.run = {
            "id": 42, "workflow_id": 7, "run_attempt": 2,
            "repository": {"full_name": REPO}, "head_repository": {"full_name": REPO},
            "head_sha": SHA, "head_branch": "main", "path": ".github/workflows/oracle.yml",
            "event": "workflow_dispatch", "status": "completed", "conclusion": "success",
            "created_at": (self.now + timedelta(seconds=2)).isoformat(), "run_started_at": self.now.isoformat(),
        }
        self.state = {"responses": {
            API: {"full_name": REPO}, API + "/commits/main": {"sha": SHA},
            API + "/actions/workflows/oracle.yml": {"id": 7, "path": ".github/workflows/oracle.yml", "state": "active"},
            API + "/actions/workflows/oracle.yml/runs": {"total_count": 1, "workflow_runs": [self.run]},
            API + "/actions/runs/42": self.run,
        }}

    def tearDown(self):
        result = self._outcome.result
        if any(test is self for test, _ in result.errors + result.failures):
            print(f"preserved failed test fixture: {self.directory}", file=sys.stderr)
            return
        # Only disposable test fixtures created by this test are removed.
        shutil.rmtree(self.directory)

    def invoke(self, mode, *arguments):
        self.state_path.write_text(json.dumps(self.state))
        result = subprocess.run(["/bin/bash", str(ROOT / "scripts/phase15-candidate-evidence.sh"), mode,
                               *arguments], cwd=ROOT, capture_output=True, text=True,
                          timeout=60, env=self.environment)
        self.state = json.loads(self.state_path.read_text())
        return result

    def dispatch(self, *extra):
        return self.invoke("dispatch", "--candidate", SHA, "--attempt-root", str(self.directory / "attempt"),
                           "--workflow", "oracle.yml", "--ref", "main", "--input", "evidence_phase=phase11", *extra)

    def assert_failed(self, result, marker):
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(marker, result.stderr)

    def test_check_validates_closed_registry(self):
        # Arrange / Act
        result = subprocess.run(
            ["/bin/bash", str(ROOT / "scripts/phase15-candidate-evidence.sh"), "check"],
            cwd=ROOT, capture_output=True, text=True, timeout=20,
        )
        # Assert
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("21 artifacts", result.stdout)

    def test_dispatch_reconciles_once_and_refuses_overwrite(self):
        # Arrange / Act
        first = self.dispatch()
        second = self.dispatch()
        # Assert
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assert_failed(second, "File exists")
        calls = [json.loads(line) for line in (self.directory / "calls.jsonl").read_text().splitlines()]
        self.assertEqual(sum(call[:2] == ["workflow", "run"] for call in calls), 1)
        self.assertEqual(json.loads((self.directory / "attempt/binding.json").read_text())["run_attempt"], 2)

    def test_dispatch_pins_github_com_despite_enterprise_environment(self):
        # Arrange
        self.environment["GH_HOST"] = "enterprise.example.invalid"
        # Act
        result = self.dispatch()
        # Assert
        self.assertEqual(result.returncode, 0, result.stderr)
        calls = [json.loads(line) for line in (self.directory / "calls.jsonl").read_text().splitlines()]
        for call in calls:
            if call[0] == "api":
                self.assertEqual(call[1:3], ["--hostname", "github.com"])
            if call[:2] == ["workflow", "run"]:
                self.assertEqual(call[call.index("--repo") + 1], "github.com/" + REPO)

    def test_wrong_repository_prevents_dispatch(self):
        # Arrange
        self.state["responses"][API]["full_name"] = "other/repository"
        # Act / Assert
        self.assert_failed(self.dispatch(), "repository differs")
        self.assertFalse((self.directory / "attempt/intent.json").exists())

    def test_wrong_remote_ref_prevents_dispatch(self):
        # Arrange
        self.state["responses"][API + "/commits/main"]["sha"] = "b" * 40
        # Act / Assert
        self.assert_failed(self.dispatch(), "ref no longer resolves")

    def test_wrong_returned_candidate_does_not_bind(self):
        # Arrange
        self.run["head_sha"] = "b" * 40
        # Act / Assert
        self.assert_failed(self.dispatch(), "ref or candidate differs")
        self.assertFalse((self.directory / "attempt/binding.json").exists())

    def test_wrong_returned_ref_does_not_bind(self):
        self.run["head_branch"] = "other"
        self.assert_failed(self.dispatch(), "ref or candidate differs")
        self.assertFalse((self.directory / "attempt/binding.json").exists())

    def test_duplicate_uncertain_dispatch_never_retries(self):
        # Arrange
        self.state["duplicate_dispatch_url"] = True
        # Act / Assert
        self.assert_failed(self.dispatch(), "one authoritative run URL")
        self.assertFalse((self.directory / "attempt/binding.json").exists())

    def test_unrelated_unique_matching_run_cannot_resolve_missing_response(self):
        # Arrange: a temporally unique match still does not establish request attribution.
        self.state["missing_dispatch_url"] = True
        # Act
        result = self.dispatch()
        reconcile = self.invoke("reconcile", "--candidate", SHA, "--attempt-root", str(self.directory / "attempt"))
        # Assert
        self.assert_failed(result, "one authoritative run URL")
        self.assert_failed(reconcile, "one authoritative run URL")
        self.assertFalse((self.directory / "attempt/binding.json").exists())

    def test_uncertain_failed_dispatch_can_only_reconcile(self):
        # Arrange
        self.state["dispatch_exit"] = 1
        # Act
        first = self.dispatch()
        reconciled = self.invoke("reconcile", "--candidate", SHA, "--attempt-root", str(self.directory / "attempt"))
        # Assert
        self.assert_failed(first, "command failed")
        self.assertEqual(reconciled.returncode, 0, reconciled.stderr)

    def test_oracle_rejects_unsupported_candidate_input(self):
        self.assert_failed(self.dispatch("--input", "candidate_sha=" + SHA), "phase11 only")

    def prepare_archives(self):
        from retention import JOB_NAMES
        jobs = [dict(name=name, id=index + 1, head_sha=SHA, run_id=42, run_attempt=2,
                     status="completed", conclusion="success") for index, name in enumerate(JOB_NAMES["oracle"])]
        self.state["responses"][API + "/actions/runs/42/attempts/2/jobs"] = {"total_count": 2, "jobs": jobs}
        artifacts = []
        for number, prefix in enumerate(("phase11-canonical", "phase11-sanitizer"), 1):
            archive = self.directory / f"source-{number}.zip"
            with zipfile.ZipFile(archive, "w") as output:
                output.writestr("identity.json", json.dumps({"run_attempt": 2}))
                output.writestr("raw.txt", "complete raw payload\n")
            artifacts.append({"id": number, "name": f"{prefix}-42-{SHA}", "expired": False,
                              "size_in_bytes": archive.stat().st_size,
                              "workflow_run": {"id": 42, "head_sha": SHA},
                              "created_at": self.now.isoformat(),
                              "expires_at": (self.now + timedelta(days=1)).isoformat(),
                              "digest": "sha256:" + hashlib.sha256(archive.read_bytes()).hexdigest()})
            self.state["responses"][API + f"/actions/artifacts/{number}/zip"] = {"binary_file": str(archive)}
        self.state["responses"][API + "/actions/runs/42/artifacts"] = {"total_count": 2, "artifacts": artifacts}
        self.state["responses"][API + "/actions/runs/42/attempts/2/logs"] = {"binary_file": str(self.directory / "source-1.zip")}
        return artifacts

    def collect(self, attempt="2"):
        return self.invoke("collect", "--candidate", SHA, "--attempt-root", str(self.directory / "retention"),
                           "--workflow", "oracle", "--ref", "main", "--run-id", "42", "--run-attempt", attempt)

    def test_collection_retains_archives_logs_and_inventory_without_overwriting(self):
        # Arrange
        self.prepare_archives()
        # Act
        first = self.collect()
        second = self.collect()
        # Assert
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assert_failed(second, "File exists")
        record = json.loads((self.directory / "retention/producers/oracle/retained.json").read_text())
        self.assertIn("provider-logs.zip", record["files"])
        self.assertIn(f"archives/phase11-canonical-42-{SHA}.zip", record["files"])

    def test_partial_download_has_no_retained_record(self):
        # Arrange
        self.prepare_archives()
        (self.directory / "source-1.zip").write_bytes(b"partial")
        # Act / Assert
        self.assert_failed(self.collect(), "partial artifact download")
        self.assertFalse((self.directory / "retention/producers/oracle/retained.json").exists())

    def test_expired_archive_fails(self):
        self.prepare_archives()[0]["expired"] = True
        self.assert_failed(self.collect(), "expired")

    def test_missing_archive_fails(self):
        self.prepare_archives().pop()
        self.assert_failed(self.collect(), "incomplete artifact listing")

    def test_current_provider_attempt_is_required(self):
        self.prepare_archives()
        self.assert_failed(self.collect("1"), "not provider current attempt")

    def test_stale_archive_attempt_is_rejected(self):
        self.prepare_archives()[0]["created_at"] = (self.now - timedelta(hours=1)).isoformat()
        self.assert_failed(self.collect(), "predates current attempt")

    def test_failed_job_cannot_be_collected(self):
        self.prepare_archives()
        self.state["responses"][API + "/actions/runs/42/attempts/2/jobs"]["jobs"][0]["conclusion"] = "failure"
        self.assert_failed(self.collect(), "terminal status differs")

    def test_archive_traversal_and_symlinks_are_rejected(self):
        from payloads import unpack
        # Arrange / Act / Assert
        for index, member in enumerate(("../escape", "a/../escape", "/escape", "a\\escape")):
            archive = self.directory / f"hostile-{index}.zip"
            with zipfile.ZipFile(archive, "w") as output:
                output.writestr(member, "bad")
            with self.assertRaises(ValueError):
                unpack(archive, self.directory / f"unpacked-{index}")
        archive = self.directory / "link.zip"
        with zipfile.ZipFile(archive, "w") as output:
            entry = zipfile.ZipInfo("link")
            entry.external_attr = 0o120777 << 16
            output.writestr(entry, "../escape")
        with self.assertRaises(ValueError):
            unpack(archive, self.directory / "link-output")

    def test_attempt_traversal_and_symlink_parent_fail(self):
        from common import confined
        with self.assertRaises(ValueError):
            confined("target/../escape")
        (self.directory / "link").symlink_to(self.directory, target_is_directory=True)
        with self.assertRaises(ValueError):
            confined(str(self.directory / "link/attempt"))

    def test_original_archive_materializes_exact_bytes_at_a_new_path(self):
        from payloads import inventory, unpack
        # Arrange
        self.prepare_archives()
        # Act
        unpack(self.directory / "source-1.zip", self.directory / "fresh-target")
        files = inventory(self.directory / "fresh-target")
        # Assert
        self.assertEqual(set(files), {"identity.json", "raw.txt"})
        self.assertEqual((self.directory / "fresh-target/raw.txt").read_text(), "complete raw payload\n")
        with self.assertRaises(FileExistsError):
            unpack(self.directory / "source-1.zip", self.directory / "fresh-target")

    def test_shell_bridge_rejects_complete_names_without_valid_identities(self):
        from common import WORKFLOWS
        from payloads import names
        # Arrange
        sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        payloads = self.directory / "payloads"
        payloads.mkdir()
        for workflow in WORKFLOWS:
            for name in names(workflow, 42, sha):
                (payloads / name).mkdir()
        # Act
        result = subprocess.run(["bash", str(ROOT / "scripts/phase15-candidate-evidence/validate.sh"),
                                 sha, str(payloads), *(["42"] * 7)], cwd=ROOT,
                                capture_output=True, text=True, timeout=20)
        # Assert
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid identity.json cardinality", result.stderr)

    def test_subprocess_output_and_time_are_bounded(self):
        from common import execute
        # Arrange / Act / Assert
        with self.assertRaisesRegex(ValueError, "output exceeds bound"):
            execute(["python3", "-c", "print('x' * 100000)"], self.directory / "bounded", limit=32)
        outputs = list((self.directory / "bounded").glob("*.stdout"))
        self.assertEqual(outputs[0].stat().st_size, 32)
        with self.assertRaisesRegex(ValueError, "timed out"):
            execute(["python3", "-c", "import time; time.sleep(5)"], self.directory / "timeout", timeout=0.1)


if __name__ == "__main__":
    unittest.main()
