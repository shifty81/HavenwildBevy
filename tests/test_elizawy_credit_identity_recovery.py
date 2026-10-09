"""Source-only regression suite for manifest-pinned ElizaWy credits recovery."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "tools/repair_elizawy_credit_identity.py"
spec = importlib.util.spec_from_file_location("repair_elizawy_credit_identity", SCRIPT)
repair = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(repair)


class CreditsRecoveryTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.credits = {
            "Terrain/Credits.txt": b"Artwork by artist\r\nLine two\r\n",
            "Structure/Doors/Credits.txt": b"door credits\r\n",
        }
        entries = []
        for name, original in self.credits.items():
            path = self.root / repair.ART_ROOT / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(original.replace(b"\r\n", b"\n"))
            entries.append({"path": name, "bytes": len(original), "sha256": repair.digest(original)})
        manifest = self.root / repair.MANIFEST
        manifest.parent.mkdir(parents=True, exist_ok=True)
        manifest.write_text(json.dumps({"schema": "havenwild.bevy.asset_manifest.v1", "entries": entries}), encoding="utf-8")

    def test_dry_run_never_modifies_assets_and_reports_full_plan(self):
        before = (self.root / repair.ART_ROOT / "Terrain/Credits.txt").read_bytes()
        result = repair.run(self.root)
        self.assertEqual(result["counts"], {"VERIFIED": 0, "RESTORABLE_CRLF": 2, "CONFLICT": 0})
        self.assertEqual((self.root / repair.ART_ROOT / "Terrain/Credits.txt").read_bytes(), before)
        self.assertFalse((self.root / repair.BACKUP_ROOT).exists())

    def test_apply_restores_exact_bytes_and_keeps_backup(self):
        before = (self.root / repair.ART_ROOT / "Terrain/Credits.txt").read_bytes()
        report = repair.run(self.root, execute=True)
        self.assertEqual(report["status"], "PASS_VERIFIED")
        self.assertEqual(report["repairedCount"], 2)
        for name, original in self.credits.items():
            self.assertEqual((self.root / repair.ART_ROOT / name).read_bytes(), original)
            self.assertEqual((self.root / repair.BACKUP_ROOT / name).read_bytes(), original.replace(b"\r\n", b"\n"))
        self.assertEqual((self.root / repair.BACKUP_ROOT / "Terrain/Credits.txt").read_bytes(), before)
        again = repair.run(self.root, execute=True)
        self.assertEqual(again["repairedCount"], 0)
        self.assertEqual(again["counts"]["VERIFIED"], 2)

    def test_unexpected_content_blocks_all_changes(self):
        (self.root / repair.ART_ROOT / "Structure/Doors/Credits.txt").write_bytes(b"CHANGED CONTENT\n")
        report = repair.run(self.root, execute=True)
        self.assertEqual(report["status"], "BLOCKED_CONFLICT")
        self.assertEqual(report["counts"]["CONFLICT"], 1)
        self.assertEqual((self.root / repair.ART_ROOT / "Terrain/Credits.txt").read_bytes(),
                         self.credits["Terrain/Credits.txt"].replace(b"\r\n", b"\n"))

    def test_manifest_traversal_is_rejected(self):
        self.assertRaises(repair.RecoveryError, repair.safe_credit_path, "../Credits.txt")
        self.assertRaises(repair.RecoveryError, repair.safe_credit_path, "Terrain/Bad.txt")
        self.assertRaises(repair.RecoveryError, repair.safe_credit_path, "C:/Credits.txt")
        self.assertRaises(repair.RecoveryError, repair.safe_credit_path, "Terrain//Credits.txt")

    def test_partial_crlf_drift_fails_closed(self):
        (self.root / repair.ART_ROOT / "Terrain/Credits.txt").write_bytes(b"Artwork by artist\r\nLine two\n")
        result = repair.run(self.root)
        self.assertEqual(result["status"], "BLOCKED_CONFLICT")


if __name__ == "__main__":
    unittest.main()
