"""Behavioral fake-provider tests for bounded Phase 13 metadata acquisition."""

import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[4]
SPEC = importlib.util.spec_from_file_location("phase13_query", ROOT / "scripts/phase13-evidence-query.py")
QUERY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(QUERY)
FAKE = Path(__file__).with_name("fake_gh.py")
COMMON = sys.modules["common"]


class FailingOutput:
    def __init__(self, stream, error):
        self.stream = stream
        self.error = error

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        self.stream.close()

    def write(self, content):
        self.stream.write(content[:3])
        raise self.error


class QueryControls(unittest.TestCase):
    def setUp(self):
        owned = ROOT / "target/phase15-query-controls"
        owned.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=owned)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.retained = self.root / "run.json"
        self.pid = self.root / "pid"

    def run_query(self, mode, timeout=0.2):
        return QUERY.query("repos/example/run", self.retained,
                           [sys.executable, str(FAKE), mode, str(self.pid)], timeout)

    def record(self):
        records = list((self.root / "run-query").glob("*.command.json"))
        self.assertEqual(len(records), 1)
        return json.loads(records[0].read_text())

    def assert_reaped(self):
        with self.assertRaises(ProcessLookupError):
            os.kill(int(self.pid.read_text()), 0)

    def test_normal_success_retains_metadata_and_status(self):
        self.run_query("success", timeout=2)
        self.assertEqual(json.loads(self.retained.read_text()), {"id": 99})
        self.assertEqual(self.record()["exit_code"], 0)
        self.assertIsNone(self.record()["failure"])
        self.assert_reaped()

    def test_sleeping_process_without_eof_times_out(self):
        start = time.monotonic()
        with self.assertRaisesRegex(ValueError, "timed out"):
            self.run_query("no-eof")
        self.assertLess(time.monotonic() - start, 3)
        self.assertEqual(self.record()["failure"], "command timed out")
        self.assert_reaped()

    def test_process_running_after_pipe_eof_times_out(self):
        start = time.monotonic()
        with self.assertRaisesRegex(ValueError, "timed out"):
            self.run_query("closed-pipes")
        self.assertLess(time.monotonic() - start, 3)
        self.assertEqual(self.record()["failure"], "command timed out")
        self.assert_reaped()

    def test_stderr_overflow_is_bounded_and_reaped(self):
        with self.assertRaisesRegex(ValueError, "output exceeds bound"):
            self.run_query("stderr-overflow", timeout=10)
        record = self.record()
        self.assertEqual(record["failure"], "command output exceeds bound")
        self.assertEqual((ROOT / record["stderr"]).stat().st_size, 64 * 1024 * 1024)
        self.assert_reaped()

    def test_nonzero_provider_status_is_retained(self):
        with self.assertRaisesRegex(ValueError, "command failed"):
            self.run_query("failed", timeout=2)
        self.assertEqual(self.record()["exit_code"], 7)
        self.assert_reaped()

    def test_finite_stdout_overflow_retains_failure_after_producer_exit(self):
        with self.assertRaisesRegex(ValueError, "output exceeds bound"):
            self.run_query("finite-overflow", timeout=2)
        self.assertEqual(self.record()["failure"], "command output exceeds bound")
        self.assertEqual(self.retained.stat().st_size, 1024 * 1024)
        self.assert_reaped()

    def test_destination_outside_target_is_rejected(self):
        with self.assertRaises(ValueError):
            QUERY.query("endpoint", ROOT / "unowned.json")

    def test_symlink_destination_parent_is_rejected(self):
        link = self.root / "alias"
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symbolic link"):
            QUERY.query("endpoint", link / "run.json")

    def failing_output(self, failure):
        original_open = Path.open

        def open_file(path, *args, **kwargs):
            stream = original_open(path, *args, **kwargs)
            if path == self.retained and args[0] == "xb":
                return FailingOutput(stream, failure)
            return stream

        return mock.patch.object(Path, "open", open_file)

    def test_output_write_failure_reaps_child_and_preserves_original_error(self):
        failure = OSError("injected disk full during capture")
        with self.failing_output(failure):
            with self.assertRaises(OSError) as raised:
                self.run_query("no-eof", timeout=2)
        self.assertIs(raised.exception, failure)
        self.assert_reaped()
        self.assertEqual(self.retained.read_bytes(), b'{"i')
        self.assertEqual(self.record()["failure"], "capture_error")
        self.assertEqual(self.record()["error_type"], "OSError")

    def test_unwritable_failure_record_does_not_replace_capture_error(self):
        failure = OSError("original capture failure")
        with self.failing_output(failure):
            with mock.patch.object(COMMON, "write_json", side_effect=OSError("record disk full")):
                with self.assertRaises(OSError) as raised:
                    self.run_query("no-eof", timeout=2)
        self.assertIs(raised.exception, failure)
        self.assert_reaped()
        self.assertTrue(any("could not be retained" in note for note in failure.__notes__))
        self.assertEqual(list((self.root / "run-query").glob("*.command.json")), [])


if __name__ == "__main__":
    unittest.main()
