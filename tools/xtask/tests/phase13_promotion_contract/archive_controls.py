"""Exercise real ZIP bytes against the production archive validation helper."""

import hashlib
import importlib.util
import pathlib
import stat
import struct
import tempfile
import unittest
from unittest import mock
import warnings
import zipfile

REPOSITORY = pathlib.Path(__file__).resolve().parents[4]
SPEC = importlib.util.spec_from_file_location("archive_validator", REPOSITORY / "scripts/phase13-evidence-archive.py")
VALIDATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(VALIDATOR)


class ArchiveControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = pathlib.Path(self.temporary.name).resolve()
        self.bundle = self.root / "bundle"
        self.bundle.mkdir()
        (self.bundle / "record.json").write_bytes(b"genuine bundle bytes")
        self.archive = self.root / "archive.zip"
        self.write_archive([("record.json", b"genuine bundle bytes")])

    def write_archive(self, members):
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            with zipfile.ZipFile(self.archive, "w") as archive:
                for name, content in members:
                    archive.writestr(name, content)

    def validate(self, digest=None, expires="2999-12-14T00:00:00Z"):
        if digest is None:
            digest = "sha256:" + hashlib.sha256(self.archive.read_bytes()).hexdigest()
        return VALIDATOR.validate(self.archive, self.bundle, digest, expires)

    def test_matching_provider_archive_and_bundle_pass(self):
        self.assertEqual(set(self.validate()["files"]), {"record.json"})

    def test_wrong_provider_digest_fails(self):
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            self.validate("sha256:" + "0" * 64)

    def test_mutated_extracted_bytes_fail(self):
        (self.bundle / "record.json").write_bytes(b"changed bundle bytes")
        with self.assertRaisesRegex(ValueError, "bytes mismatch"):
            self.validate()

    def test_extra_extracted_file_fails(self):
        (self.bundle / "extra").write_bytes(b"extra")
        with self.assertRaisesRegex(ValueError, "file sets differ"):
            self.validate()

    def test_missing_extracted_file_fails(self):
        (self.bundle / "record.json").unlink()
        with self.assertRaisesRegex(ValueError, "not a regular file"):
            self.validate()

    def test_duplicate_archive_member_fails(self):
        self.write_archive([("record.json", b"genuine bundle bytes")] * 2)
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.validate()

    def test_traversal_archive_member_fails(self):
        self.write_archive([("../record.json", b"bad")])
        with self.assertRaisesRegex(ValueError, "unsafe archive path"):
            self.validate()

    def test_windows_drive_stream_and_alias_members_fail_before_file_access(self):
        for name in ["D:escape", "record.json:stream", "NUL.txt", "COM1", "folder./record", "folder /record"]:
            with self.subTest(name=name):
                self.write_archive([(name, b"bad")])
                with mock.patch.object(VALIDATOR, "regular_path", wraps=VALIDATOR.regular_path) as checked:
                    with self.assertRaisesRegex(ValueError, "unsafe archive path"):
                        self.validate()
                self.assertEqual(checked.call_count, 1)

    def test_symlink_archive_member_fails(self):
        member = zipfile.ZipInfo("record.json")
        member.external_attr = (stat.S_IFLNK | 0o777) << 16
        self.write_archive([(member, b"other")])
        with self.assertRaisesRegex(ValueError, "non-regular"):
            self.validate()

    def test_extracted_symlink_fails(self):
        (self.bundle / "record.json").unlink()
        (self.bundle / "record.json").symlink_to(self.archive)
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.validate()

    def test_expired_provider_time_fails(self):
        with self.assertRaisesRegex(ValueError, "expired"):
            self.validate(expires="2000-01-01T00:00:00Z")

    def test_encrypted_member_fails_explicitly(self):
        content = bytearray(self.archive.read_bytes())
        central = content.index(b"PK\x01\x02")
        struct.pack_into("<H", content, 6, 1)
        struct.pack_into("<H", content, central + 8, 1)
        self.archive.write_bytes(content)
        with self.assertRaisesRegex(ValueError, "encrypted"):
            self.validate()

    def test_member_size_bound_is_enforced(self):
        with mock.patch.object(VALIDATOR, "MAX_MEMBER", 1):
            with self.assertRaisesRegex(ValueError, "member exceeds"):
                self.validate()

    def test_total_expanded_size_bound_is_enforced(self):
        with mock.patch.object(VALIDATOR, "MAX_TOTAL", 1):
            with self.assertRaisesRegex(ValueError, "expanded size"):
                self.validate()

    def test_raw_archive_size_bound_is_enforced(self):
        with mock.patch.object(VALIDATOR, "MAX_ARCHIVE", 1):
            with self.assertRaisesRegex(ValueError, "archive exceeds"):
                self.validate()


if __name__ == "__main__":
    unittest.main()
