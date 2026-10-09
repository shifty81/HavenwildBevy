from __future__ import annotations
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

DOCTOR = Path(__file__).resolve().parents[1] / "tools" / "hw_barrel_gate_doctor.py"
spec = importlib.util.spec_from_file_location("hw_barrel_gate_doctor", DOCTOR)
doctor = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(doctor)


def write_json(root, rel, document):
    out = root / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(document) + "\n", encoding="utf-8")


class BarrelDoctorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        (self.root / "Cargo.toml").write_text('[package]\nname="havenwild_bevy"\nversion="0.10.19"\n', encoding="utf-8")
        manifest = {"entries": [{"path": doctor.BARREL, "id": "elizawy:"+doctor.BARREL,
                                 "sha256": "a"*64, "bytes": 5, "imageSizePx": [160,64]}]}
        write_json(self.root, doctor.MANIFEST, manifest)
        self.runtime = {"runtimeSourceImages": [{"sourcePath": doctor.BARREL,
                         "sourceSha256": "a"*64, "imageSizePx": [160,64]}],
                        "canonicalRuntimeRegions": [], "objectTemplates": [],
                        "counts": {"objectTemplates": 0}}
        self.lane = {"objectTemplates": []}
        write_json(self.root, doctor.SEED, {"objects": []})
        self.flush()

    def tearDown(self):
        self.temp.cleanup()

    def flush(self):
        write_json(self.root, doctor.LANE, self.lane)
        self.runtime["laneSha256"] = doctor.sha256((self.root / doctor.LANE).read_bytes())
        write_json(self.root, doctor.RUNTIME, self.runtime)

    def test_missing_template_is_reported(self):
        report = doctor.diagnosis(self.root)
        self.assertEqual(report["status"], "MISSING_TEMPLATE")
        self.assertFalse(report["checks"]["reviewed_barrel_template_exists"])

    def test_full_height_template_is_recognized(self):
        rect = [64, 0, 32, 64]
        region = {"canonicalRegionId": "full", "sourcePath": doctor.BARREL, "sourceRectPx": rect}
        part = {"sourcePath": doctor.BARREL, "sourceRectPx": rect, "canonicalRegionId": "full"}
        template = {"templateId": "barrel.one", "label": "Reviewed Barrel", "parts": [part]}
        self.runtime["canonicalRuntimeRegions"] = [region]
        self.runtime["objectTemplates"] = [template]
        self.runtime["counts"]["objectTemplates"] = 1
        self.lane["objectTemplates"] = [template]
        self.flush()
        report = doctor.diagnosis(self.root)
        self.assertEqual(report["status"], "METADATA_LOOKS_VALID_REVIEW_RUST_TEST")
        self.assertTrue(report["checks"]["reviewed_template_canonical_region_matches"])

    def test_half_height_barrel_is_rejected(self):
        template = {"templateId": "bad", "label": "Barrel", "parts": [
            {"sourcePath": doctor.BARREL, "sourceRectPx": [32,0,32,32], "canonicalRegionId": "half"}]}
        self.runtime["objectTemplates"] = [template]
        self.runtime["counts"]["objectTemplates"] = 1
        self.lane["objectTemplates"] = [template]
        self.flush()
        report = doctor.diagnosis(self.root)
        self.assertEqual(report["status"], "BROKEN_TEMPLATE_CROP_OR_REGION")

    def test_stale_runtime_hash_rejected(self):
        self.runtime["laneSha256"] = "f"*64
        write_json(self.root, doctor.RUNTIME, self.runtime)
        report = doctor.diagnosis(self.root)
        self.assertFalse(report["checks"]["runtime_lane_hash_consistent"])
        self.assertEqual(report["status"], "MISSING_TEMPLATE")

    def test_showcase_builder_is_not_misreported_as_seed_only(self):
        builder = self.root / doctor.BUILDER
        builder.parent.mkdir(parents=True, exist_ok=True)
        builder.write_text(
            'for obj in seed["objects"]:\n    pass\n'
            'for obj in showcase["templates"]:\n    pass\n', encoding="utf-8"
        )
        report = doctor.diagnosis(self.root)
        self.assertTrue(report["evidence"]["builderReferencesOtherObjectTemplateSource"])
        self.assertFalse(report["checks"]["builder_is_seed_only_for_templates"])
        self.assertFalse(any("Legacy seed-only" in action for action in report["nextActions"]))

    def test_bundle_excludes_artwork_and_git(self):
        (self.root / "assets/elizawy/Objects/Furniture").mkdir(parents=True)
        (self.root / "assets/elizawy/Objects/Furniture/Barrel.png").write_bytes(b"fake")
        (self.root / ".git").mkdir()
        out = self.root / "debug.zip"
        doctor.write_bundle(self.root, doctor.diagnosis(self.root), out)
        import zipfile
        with zipfile.ZipFile(out) as z:
            names = z.namelist()
            self.assertIn("audit/barrel_gate_report.json", names)
            self.assertIn("source/Cargo.toml", names)
            self.assertFalse(any(".git" in n or n.endswith(".png") for n in names))


if __name__ == "__main__":
    unittest.main()
