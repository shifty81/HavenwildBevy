"""Small deterministic fixtures for the complete Characters.zip rollup authority."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import sys
import zipfile
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import ForgePY


class CharacterArchiveIntegrityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.archive = self.root / "Characters.zip"
        self.manifest = self.root / "core_manifest.json"
        with zipfile.ZipFile(self.archive, "w", compression=zipfile.ZIP_DEFLATED) as z:
            z.writestr("Characters/Body/Idle.png", b"original sprite bytes")
        self.write_manifest()

    def tearDown(self):
        self.temp.cleanup()

    def write_manifest(self):
        self.manifest.write_text(json.dumps({"optionalArchives": {"Characters.zip": {
            "bytes": self.archive.stat().st_size,
            "sha256": hashlib.sha256(self.archive.read_bytes()).hexdigest(),
        }}}), encoding="utf-8")

    def test_pinned_source_archive_is_accepted(self):
        with mock.patch.object(ForgePY, "CORE_MANIFEST", self.manifest):
            ok, result = ForgePY.verify_character_archive(self.archive)
        self.assertTrue(ok, result)

    def test_tampered_archive_with_same_byte_count_is_rejected(self):
        with self.archive.open("r+b") as f:
            f.seek(40)
            old = f.read(1)
            f.seek(40)
            f.write(bytes([old[0] ^ 1]))
        with mock.patch.object(ForgePY, "CORE_MANIFEST", self.manifest):
            ok, result = ForgePY.verify_character_archive(self.archive)
        self.assertFalse(ok)
        self.assertIn("SHA256", result)

    def test_wrong_archive_size_is_rejected(self):
        self.archive.write_bytes(self.archive.read_bytes() + b"x")
        with mock.patch.object(ForgePY, "CORE_MANIFEST", self.manifest):
            ok, result = ForgePY.verify_character_archive(self.archive)
        self.assertFalse(ok)
        self.assertIn("byte count", result)

    def test_crc32_rejects_same_size_corruption(self):
        target = self.root / "Idle.png"
        target.write_bytes(b"original sprite bytes")
        good = zlib.crc32(target.read_bytes())
        self.assertTrue(ForgePY._character_file_matches_crc32(target, target.stat().st_size, good))
        target.write_bytes(b"corrupt! sprite bytes")
        self.assertFalse(ForgePY._character_file_matches_crc32(target, target.stat().st_size, good))

    def test_missing_manifest_denies_unpinned_archive(self):
        self.manifest.write_text('{}', encoding='utf-8')
        with mock.patch.object(ForgePY, "CORE_MANIFEST", self.manifest):
            ok, result = ForgePY.verify_character_archive(self.archive)
        self.assertFalse(ok)
        self.assertIn("authority missing", result)


if __name__ == "__main__":
    unittest.main()
