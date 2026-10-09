#!/usr/bin/env python3
"""Non-mutating source-linked diagnosis for Havenwild Bevy barrel gate failures.

Works on a *current* local source tree; never changes project source or hydrated art.
Outputs a machine-readable report and an optional minimal source-only handoff ZIP.
Python 3.11+, no external dependencies.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import tomllib
import zipfile
from pathlib import Path
from datetime import datetime, timezone

BARREL = "Objects/Furniture/Barrel.png"
MANIFEST = "content/catalog/core_source_manifest.json"
LANE = "content/assets/authority/elizawy_asset_lane.v1.json"
RUNTIME = "content/assets/authority/elizawy_runtime_index.v1.json"
SEED = "content/scenes/summer_world.visual_seed.v1.json"
BUILDER = "tools/build_elizawy_asset_lane.py"
WORLD_DOC = "src/world_doc.rs"
SNAPSHOT = ["Cargo.toml", MANIFEST, LANE, RUNTIME, SEED, BUILDER,
            WORLD_DOC, "src/asset_authority.rs", "src/worldgen_plan.rs",
            "tools/m2d097b_water_scrutiny_selftest.py"]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(root: Path, relative: str):
    f = root / relative
    if not f.is_file():
        return None
    return json.loads(f.read_text(encoding="utf-8-sig"))


def toplevel_barrel_templates(authority: dict) -> list[dict]:
    return [t for t in authority.get("objectTemplates", [])
            if "barrel" in str(t.get("label", "")).lower()
            or any(p.get("sourcePath") == BARREL for p in t.get("parts", []))]


def diagnosis(root: Path) -> dict:
    root = root.resolve(strict=True)
    if not (root / "Cargo.toml").is_file():
        raise ValueError(f"No Cargo.toml in project root {root}")
    cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8-sig"))
    version = cargo.get("package", {}).get("version")
    m = read_json(root, MANIFEST)
    lane = read_json(root, LANE)
    runtime = read_json(root, RUNTIME)
    seed = read_json(root, SEED)
    if not isinstance(m, dict):
        raise ValueError(f"Missing/unreadable {MANIFEST}")
    if not isinstance(runtime, dict):
        raise ValueError(f"Missing/unreadable {RUNTIME}")
    report = {
        "schema": "havenwild.bevy.barrel_gate_diagnosis.v1",
        "generatedUtc": datetime.now(timezone.utc).isoformat(),
        "root": str(root),
        "sourceVersion": version,
        "status": "UNKNOWN",
        "checks": {},
        "evidence": {},
        "nextActions": [],
    }
    c, e, actions = report["checks"], report["evidence"], report["nextActions"]
    entries = [row for row in m.get("entries", []) if row.get("path") == BARREL]
    c["manifest_exact_one_barrel"] = len(entries) == 1
    if entries:
        e["sourceManifest"] = {k: entries[0].get(k) for k in ("id", "path", "sha256", "bytes", "imageSizePx")}
    e["sourceRevision"] = version
    hydrated = root / "assets/elizawy" / BARREL
    e["localHydratedImagePresent"] = hydrated.is_file()
    if hydrated.is_file() and entries:
        raw = hydrated.read_bytes()
        e["localHydratedSha256"] = sha256(raw)
        c["hydrated_image_matches_manifest"] = e["localHydratedSha256"] == entries[0].get("sha256")
        c["hydrated_image_bytecount_matches_manifest"] = len(raw) == entries[0].get("bytes")
    else:
        c["hydrated_image_matches_manifest"] = None
        c["hydrated_image_bytecount_matches_manifest"] = None

    runtime_images = [r for r in runtime.get("runtimeSourceImages", []) if r.get("sourcePath") == BARREL]
    c["runtime_image_registered_once"] = len(runtime_images) == 1
    if runtime_images:
        e["runtimeImage"] = {k: runtime_images[0].get(k) for k in ("sourcePath", "sourceSha256", "imageSizePx")}
        c["manifest_runtime_image_identity_matches"] = bool(entries) and (
            runtime_images[0].get("sourceSha256") == entries[0].get("sha256") and
            runtime_images[0].get("imageSizePx") == entries[0].get("imageSizePx"))
    else:
        c["manifest_runtime_image_identity_matches"] = False

    barrel_regions = [r for r in runtime.get("canonicalRuntimeRegions", [])
                      if r.get("sourcePath") == BARREL]
    e["canonicalBarrelRegionRects"] = [r.get("sourceRectPx") for r in barrel_regions]
    c["canonical_full_height_region_exists"] = any((r.get("sourceRectPx") or [0,0,0,0])[3] == 64 for r in barrel_regions)

    templates = toplevel_barrel_templates(runtime)
    e["runtimeBarrelTemplates"] = templates
    c["reviewed_barrel_template_exists"] = bool(templates)
    valid_templates = []
    for t in templates:
        parts = [p for p in t.get("parts", []) if p.get("sourcePath") == BARREL]
        valid = bool(parts) and all(isinstance(p.get("sourceRectPx"), list) and
                                    len(p["sourceRectPx"]) == 4 and
                                    p["sourceRectPx"][2] == 32 and p["sourceRectPx"][3] == 64
                                    and p["sourceRectPx"][0] in (0,32,64,96,128)
                                    and p["sourceRectPx"][1] == 0 for p in parts)
        valid_templates.append(valid)
    c["reviewed_template_full_32x64_sprite"] = any(valid_templates)
    c["reviewed_template_canonical_region_matches"] = any(
        all(any(p.get("canonicalRegionId") == r.get("canonicalRegionId") and
                    p.get("sourceRectPx") == r.get("sourceRectPx")
                    for r in barrel_regions) for p in t.get("parts", []) if p.get("sourcePath") == BARREL)
        and any(p.get("sourcePath") == BARREL for p in t.get("parts", [])) for t in templates)

    if isinstance(lane, dict):
        lane_templates = toplevel_barrel_templates(lane)
        c["lane_runtime_barrel_template_consistent"] = (
            sorted(json.dumps(t, sort_keys=True) for t in lane_templates) ==
            sorted(json.dumps(t, sort_keys=True) for t in templates))
        actual = sha256((root / LANE).read_bytes())
        e["laneActualSha256"] = actual
        e["runtimeDeclaredLaneSha256"] = runtime.get("laneSha256")
        c["runtime_lane_hash_consistent"] = actual == runtime.get("laneSha256")
        e["laneBarrelTemplates"] = lane_templates
    else:
        c["lane_runtime_barrel_template_consistent"] = None
        c["runtime_lane_hash_consistent"] = None

    c["object_template_count_consistent"] = (len(runtime.get("objectTemplates", [])) ==
                                              runtime.get("counts", {}).get("objectTemplates"))
    e["runtimeTemplateCount"] = len(runtime.get("objectTemplates", []))
    e["declaredRuntimeTemplateCount"] = runtime.get("counts", {}).get("objectTemplates")

    if isinstance(seed, dict):
        seed_barrels = [o for o in seed.get("objects", [])
                        if "barrel" in str(o.get("label", "")).lower()
                        or any(p.get("source", {}).get("source_asset") == BARREL
                               for p in o.get("parts", []))]
        e["summerSeedBarrelObjects"] = seed_barrels
        c["summer_seed_contains_barrel"] = bool(seed_barrels)
    else:
        c["summer_seed_contains_barrel"] = None

    builder = root / BUILDER
    if builder.is_file():
        source = builder.read_text(encoding="utf-8-sig")
        e["builderHasTemplateSeedLoop"] = bool(re.search(r'for\s+obj\s+in\s+seed\s*\[\s*["\']objects["\']\s*\]', source))
        e["builderReferencesOtherObjectTemplateSource"] = (
            'showcase["templates"]' in source and
            'for obj in showcase["templates"]' in source
        )
        c["builder_is_seed_only_for_templates"] = (
            e["builderHasTemplateSeedLoop"] and
            not e["builderReferencesOtherObjectTemplateSource"]
        )
    else:
        c["builder_is_seed_only_for_templates"] = None

    world_src = root / WORLD_DOC
    if world_src.is_file():
        data = world_src.read_text(encoding="utf-8-sig")
        mtest = re.search(r'(?m)^\s*fn\s+reviewed_barrel_template_uses_full_height_source_sprite\s*\(', data)
        if mtest:
            start = mtest.start()
            depth, opened, end = 0, False, min(start + 7000, len(data))
            for i in range(mtest.end(), min(start + 7000, len(data))):
                if data[i] == '{':
                    opened = True
                    depth += 1
                elif data[i] == '}' and opened:
                    depth -= 1
                    if depth == 0:
                        end = i + 1
                        break
            e["failingRustTestSource"] = data[start:end].strip()
            c["failing_test_present"] = True
        else:
            c["failing_test_present"] = False
    else:
        c["failing_test_present"] = None

    if not c["reviewed_barrel_template_exists"]:
        report["status"] = "MISSING_TEMPLATE"
        actions.append("Register one reviewed barrel template in the builder's canonical inputs; do not manually edit generated runtime index alone.")
        actions.append("Use a source-verified complete 32x64 column of the original 160x64 barrel sheet; do not guess a variant or split it into 32x32 halves.")
    elif not c["reviewed_template_full_32x64_sprite"] or not c["reviewed_template_canonical_region_matches"]:
        report["status"] = "BROKEN_TEMPLATE_CROP_OR_REGION"
        actions.append("Repair the reviewed template's sourceRectPx AND matching canonical region; use validated original pixels and preserve the bottom foot anchor.")
    elif not c["runtime_lane_hash_consistent"] or not c["object_template_count_consistent"]:
        report["status"] = "STALE_OR_INCONSISTENT_AUTHORITY"
        actions.append("Run current-source asset-lane generator and --check; ensure runtime lane SHA and template count match exactly.")
    else:
        report["status"] = "METADATA_LOOKS_VALID_REVIEW_RUST_TEST"
        actions.append("Inspect failingRustTestSource: its label, lookup filter, footprint, and source crop expectations may differ from the registered template.")
    if c.get("builder_is_seed_only_for_templates"):
        actions.append("Legacy seed-only builder pattern detected. Add a separate source-reviewed object-template registry rather than creating a bogus placed seed object.")
    if c.get("hydrated_image_matches_manifest") is False:
        actions.append("Local hydrated image hash differs from the pinned source manifest; repair asset hydration before changing any crop authority.")
    actions.append("After a source-backed fix: regenerate both authority JSONs; run builder --check, targeted cargo test, cargo fmt --check, cargo test --all-targets, then the PCC full gate.")
    return report


def write_bundle(root: Path, report: dict, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(dest, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        z.writestr("audit/barrel_gate_report.json", json.dumps(report, indent=2) + "\n")
        for rel in SNAPSHOT:
            f = root / rel
            if f.is_file() and not f.is_symlink():
                z.write(f, "source/" + rel)
        z.writestr("audit/README.txt", "Havenwild source-only evidence bundle. Original PNGs, workstation settings, .git, tokens, logs and caches excluded. Does not apply a patch.\n")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd(), help="Current Havenwild Bevy source root")
    parser.add_argument("--output", type=Path, help="Write standalone JSON report")
    parser.add_argument("--bundle", type=Path, help="Write source-only ZIP with failing test and builder inputs")
    args = parser.parse_args(argv)
    try:
        root = args.root.resolve(strict=True)
        result = diagnosis(root)
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        if args.bundle:
            write_bundle(root, result, args.bundle)
        print("HAVENWILD BARREL GATE DOCTOR:", result["status"])
        print("SOURCE:", result["sourceVersion"])
        for key, value in result["checks"].items():
            print(f"  {key}: {value}")
        print("NEXT:")
        for a in result["nextActions"]:
            print(" -", a)
        return 0 if result["status"] == "METADATA_LOOKS_VALID_REVIEW_RUST_TEST" else 2
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as exc:
        print("DOCTOR ERROR:", exc, file=sys.stderr)
        return 3


if __name__ == "__main__":
    raise SystemExit(main())
