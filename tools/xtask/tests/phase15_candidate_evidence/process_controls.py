"""Actual native process controls, run directly on the Windows Oracle runner."""

from pathlib import Path
import ctypes
from ctypes import wintypes
import json
import os
import subprocess
import sys
import time
import unittest
from unittest.mock import patch
import uuid

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "scripts/phase15-candidate-evidence"))
import common


def running(pid):
    if os.name == "nt":
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        kernel.OpenProcess.restype = wintypes.HANDLE
        kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
        kernel.WaitForSingleObject.restype = wintypes.DWORD
        kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        kernel.CloseHandle.restype = wintypes.BOOL
        handle = kernel.OpenProcess(0x100000, False, pid)  # SYNCHRONIZE only; no process mutation.
        if not handle:
            error = ctypes.get_last_error()
            if error != 87:  # ERROR_INVALID_PARAMETER: this PID no longer exists.
                raise ctypes.WinError(error)
            return False
        try:
            return kernel.WaitForSingleObject(handle, 0) == 0x102
        finally:
            if not kernel.CloseHandle(handle):
                raise ctypes.WinError(ctypes.get_last_error())
    result = subprocess.run(["ps", "-p", str(pid), "-o", "stat="], capture_output=True, text=True, timeout=5)
    return bool(result.stdout.strip()) and not result.stdout.strip().startswith("Z")


class NativeProcessControls(unittest.TestCase):
    def setUp(self):
        self.root = ROOT / "target/phase15-process-controls" / uuid.uuid4().hex
        self.root.mkdir(parents=True)

    def command(self, mode, **options):
        return common.execute([sys.executable, str(Path(__file__).with_name("process_child.py")),
                               mode, str(self.root / "pid")], self.root, **options)

    def assert_stopped(self):
        pid = int((self.root / "pid").read_text())
        deadline = time.monotonic() + 5
        while running(pid) and time.monotonic() < deadline:
            time.sleep(0.05)
        self.assertFalse(running(pid), f"owned descendant {pid} remains active")

    def test_normal_output_is_retained(self):
        # Arrange / Act / Assert
        self.assertEqual(self.command("normal").read_text().strip(), "complete")

    def test_nonzero_status_is_retained(self):
        # Arrange / Act / Assert
        with self.assertRaisesRegex(ValueError, "command failed \\(7\\)"):
            self.command("error")
        record = json.loads(next(self.root.glob("*.command.json")).read_text())
        self.assertEqual(record["exit_code"], 7)

    def test_stdout_overflow_stays_bounded(self):
        # Arrange / Act / Assert
        with self.assertRaisesRegex(ValueError, "output exceeds bound"):
            self.command("overflow", limit=32)
        self.assertEqual(next(self.root.glob("*.stdout")).stat().st_size, 32)

    def test_stderr_overflow_stays_bounded(self):
        # Arrange / Act / Assert
        with patch.object(common, "MAX_FILE", 32), self.assertRaisesRegex(ValueError, "output exceeds bound"):
            self.command("stderr-overflow")
        self.assertEqual(next(self.root.glob("*.stderr")).stat().st_size, 32)

    def test_timeout_terminates_owned_descendant(self):
        # Arrange / Act / Assert
        with self.assertRaisesRegex(ValueError, "timed out"):
            self.command("tree", timeout=2)
        self.assert_stopped()

    def test_parent_exit_does_not_leave_an_owned_orphan(self):
        # Arrange / Act / Assert
        try:
            self.command("orphan", timeout=2)
        except ValueError as error:
            self.assertIn("timed out", str(error))
        self.assert_stopped()

    def test_capture_exception_cleans_up_owned_process(self):
        # Arrange / Act / Assert
        with patch.object(common, "write_capture", side_effect=OSError("controlled sink failure")):
            with self.assertRaisesRegex(OSError, "controlled sink failure"):
                self.command("pid-overflow")
        self.assert_stopped()
        record = json.loads(next(self.root.glob("*.command.json")).read_text())
        self.assertEqual(record["failure"], "capture_error")

    def test_logical_paths_stay_posix_and_reject_windows_aliases(self):
        # Arrange / Act / Assert
        self.assertEqual(str(common.normalized("target/retained/artifacts/item.json")), "target/retained/artifacts/item.json")
        for value in ("target/C:escape", "target/NUL.txt", "target/COM¹.log", "target/LPT²", "target/PRN .txt",
                      "target/item.", "target/item ", "target\\item"):
            with self.subTest(value=value), self.assertRaises(ValueError):
                common.normalized(value)
        self.assertEqual(common.confined(str(self.root)), self.root)

    def test_zip_windows_aliases_fail_before_any_member_is_written(self):
        # Arrange / Act / Assert
        import zipfile
        from payloads import unpack
        for index, name in enumerate(("D:evil", "nested/file:stream", "NUL.txt", "COM¹.log", "nested/dir./payload")):
            archive = self.root / f"hostile-{index}.zip"
            destination = self.root / f"extracted-{index}"
            with zipfile.ZipFile(archive, "w") as bundle:
                bundle.writestr("ordinary.json", "not yet written")
                bundle.writestr(name, "invalid")
            with self.subTest(name=name), self.assertRaises(ValueError):
                unpack(archive, destination)
            self.assertFalse((destination / "ordinary.json").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
