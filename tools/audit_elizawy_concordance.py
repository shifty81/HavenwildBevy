#!/usr/bin/env python3
"""Read-only ElizaWy/LPC Revised evidence concordance; never promotes mapping authority.

Uses PCC terrain recover's local, immutable copies, and optionally ForgePY-hydrated
source art. Does not fetch art, change recipes, import V7 tuples, or require Pillow.
"""
from __future__ import annotations

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
PIN = "f07f7f5892e67c932c68f70bb04472f2c64e46bc"
INDEX = Path(".forgepy/recovered_mapping/index.json")
OUTPUT = Path(".forgepy/recovered_mapping/elizawy_concordance.v1.json")
MANIFEST = Path("content/catalog/core_source_manifest.json")
REQUIRED = {
    "summer": "lpc_terrain_summer_complete_map_32.json",
    "topology": "lpc_seasonal_terrain_topology_v0_1.json",
    "provenance": "lpc_seasonal_terrain_topology_v0_1.json.provenance.json",
    "summerCrosswalk": "elizawy_summer_atlas_source_crosswalk_b48r7_v0_1.json",
    "seasonalCrosswalk": "elizawy_all_seasons_split_source_crosswalk_b48r9_v0_1.json",
}
SEASONS = ("spring", "summer", "autumn", "winter", "winter_ice")
ASSETS = tuple(f"Terrain/{family}_{season}.png" for family in ("terrain", "cliff") for season in SEASONS) + ("Terrain/Waterfall.png", "Terrain/ice-shallows.png", "Terrain/Credits.txt")
SCHEMAS = {
    "summer": "havenwild.lpc_terrain_summer_complete_map.generated.v0_1",
    "topology": "havenwild.lpc_seasonal_terrain_topology.v0.1",
    "provenance": "havenwild.generated.provenance.v1",
    "summerCrosswalk": "havenwild.elizawy.source_cell_crosswalk.b48r7",
    "seasonalCrosswalk": "havenwild.elizawy.reference_split_seasonal_source_crosswalk.b48r9",
}


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def load(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(data, dict):
        raise ValueError(f"JSON root must be object: {path}")
    return data


def png_dimensions(path: Path) -> tuple[int, int] | None:
    with path.open("rb") as f:
        header = f.read(24)
    if len(header) < 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
        return None
    return struct.unpack(">II", header[16:24])


def inspect(root: Path, asset_root: Path | None = None) -> dict:
    errors: list[str] = []
    review: list[str] = []
    index_path = root / INDEX
    if not index_path.is_file():
        raise ValueError("Recovery index unavailable; run PCC.cmd terrain recover first")
    index = load(index_path)
    if index.get("schema") != "havenwild.terrain.legacy_mapping_recovery.local.v1":
        raise ValueError("Unsupported recovery index schema")
    raw_base = (root / ".forgepy/recovered_mapping/raw").resolve()
    records = index.get("records", [])
    if not isinstance(records, list):
        raise ValueError("Recovery index records are not a list")
    found: dict[str, dict] = {}
    for key, filename in REQUIRED.items():
        candidates = [r for r in records if str(r.get("relativePath", "")).replace("\\", "/").endswith("/"+filename) or str(r.get("relativePath", "")).replace("\\", "/")==filename]
        resolved = []
        for rec in candidates:
            local = rec.get("localPath")
            if not isinstance(local, str) or not local.strip():
                continue
            path = (root / local.replace("\\", "/")).resolve()
            if not path.is_relative_to(raw_base):
                errors.append(f"{key}: unsafe recovered localPath")
                continue
            if not path.is_file():
                continue
            if sha(path).lower() != str(rec.get("sha256", "")).lower():
                errors.append(f"{key}: local copy failed indexed SHA-256")
                continue
            resolved.append((path, rec["sha256"]))
        unique_hashes = {h for _, h in resolved}
        if len(unique_hashes) > 1:
            errors.append(f"{key}: multiple distinct recovered versions; selection is ambiguous")
            continue
        if not resolved:
            errors.append(f"{key}: no index-backed, SHA-verified raw copy")
            continue
        path, hashed = resolved[0]
        data = load(path)
        if data.get("schema") != SCHEMAS[key]:
            errors.append(f"{key}: unexpected schema {data.get('schema')!r}")
        found[key] = {"path": str(path.relative_to(root)), "sha256": hashed, "candidateCopies": len(resolved), "document": data}
    report = {
        "schema": "havenwild.elizawy.concordance.readonly.v1",
        "authority": "evidence_only_zero_promotions",
        "sourceLockExpected": PIN,
        "recoveredIndexRecords": len(records),
        "verifiedDocuments": {k: {x: v[x] for x in ("path", "sha256", "candidateCopies")} for k, v in found.items()},
        "errors": errors,
        "reviewFlags": review,
        "certifiedMappingsAdded": 0,
    }
    if len(found) == len(REQUIRED):
        summer = found["summer"]["document"]
        topology = found["topology"]["document"]
        b7 = found["summerCrosswalk"]["document"]
        b9 = found["seasonalCrosswalk"]["document"]
        prov = found["provenance"]["document"]
        grid = summer.get("grid", [])
        cells = summer.get("cells", [])
        if grid != [16, 26] or summer.get("cellSize") != 32 or not isinstance(cells, list):
            errors.append("Summer grid must be 16x26 at 32px")
        coordinates = []
        for cell in cells:
            pos = cell.get("cell") if isinstance(cell, dict) else None
            if not isinstance(pos, list) or len(pos) != 2 or not all(isinstance(x, int) for x in pos) or not (0 <= pos[0] < 16 and 0 <= pos[1] < 26):
                errors.append("Summer cell has invalid coordinate")
                continue
            coordinates.append(tuple(pos))
            if cell.get("pixelRect") != [pos[0] * 32, pos[1] * 32, 32, 32]:
                errors.append(f"Summer pixel rect does not match cell {pos}")
        if len(cells) != 416 or len(set(coordinates)) != 416:
            errors.append(f"Summer grid coverage is {len(cells)} cells / {len(set(coordinates))} unique instead of 416")
        occupied = sum(c.get("nonTransparent") is True for c in cells if isinstance(c, dict))
        transparent = sum(c.get("nonTransparent") is False for c in cells if isinstance(c, dict))
        if (occupied, transparent) != (305, 111):
            errors.append(f"Summer occupancy {occupied}/{transparent} differs from historical 305/111")
        summary = summer.get("summary", {})
        if summary.get("mappedNonTransparentCells") != occupied or summary.get("primaryGroups") != 41:
            errors.append("Summer summary does not reconcile with mapped cells/groups")
        group_counts = Counter(c.get("groupId") for c in cells if isinstance(c, dict) and c.get("groupId"))
        if len(group_counts) != 41:
            errors.append(f"Summer primary group IDs: {len(group_counts)} instead of 41")
        if topology.get("grid") != grid or topology.get("cellSize") != 32:
            errors.append("Topology source dimensions disagree with Summer map")
        families = topology.get("families", [])
        if len(families) != 20 or len({f.get('id') for f in families}) != 20:
            errors.append("Topology family count/identities disagree with 20 expected")
        outer_keys = {"northWest": "outer_north_west", "north": "outer_north", "northEast": "outer_north_east", "west": "outer_west", "center": "center", "east": "outer_east", "southWest": "outer_south_west", "south": "outer_south", "southEast": "outer_south_east"}
        inner_keys = {"northWest": "inner_north_west", "northEast": "inner_north_east", "southWest": "inner_south_west", "southEast": "inner_south_east"}
        topology_role_disagreements = []
        role_refs = 0
        invalid_role_refs = []
        season_binding_counts = Counter()
        for family in families:
            for season_data in family.get("seasonalBindings", []):
                season = season_data.get("season")
                season_binding_counts[season] += 1
                for topology_key, binding_key, source_key, binding_section in ((outer_keys, "outerRoles", "outerTopology", "outer"), (inner_keys, "innerRoles", "innerTopology", "inner")):
                    for top_role, source_role in topology_key.items():
                        declared = family.get(source_key, {}).get(top_role)
                        bound = season_data.get(binding_key, {}).get(source_role)
                        if declared is not None and declared != bound:
                            topology_role_disagreements.append([family.get("id"), season, binding_section, top_role])
                for section in ("outerRoles", "innerRoles"):
                    for role, pos in season_data.get(section, {}).items():
                        role_refs += 1
                        if not isinstance(pos, list) or len(pos) != 2 or not all(isinstance(x,int) for x in pos) or not (0 <= pos[0] < 16 and 0 <= pos[1] < 26):
                            invalid_role_refs.append([family.get("id"), season, role, pos])
        if topology_role_disagreements:
            errors.append(f"Topology family/season roles disagree at {len(topology_role_disagreements)} coordinates")
        if invalid_role_refs:
            errors.append(f"Topology has {len(invalid_role_refs)} out-of-grid role references")
        if set(season_binding_counts) != set(SEASONS) or any(season_binding_counts[s] != 20 for s in SEASONS):
            errors.append(f"Seasonal topology family bindings incomplete: {dict(season_binding_counts)}")
        if len(b7.get("entries", [])) != 290 or len(b9.get("entries", [])) != 1781:
            errors.append("Recovered crosswalk entry counts disagree with recorded 290/1781")
        b7_counts = dict(Counter(e.get("status") for e in b7.get("entries", [])))
        crosswalk_invalid_target_cells = 0
        for entry in b7.get("entries", []):
            for pos in entry.get("exactMainAtlasCells", []):
                if not isinstance(pos, list) or len(pos) != 2 or not all(isinstance(n,int) for n in pos) or not (0 <= pos[0] < 16 and 0 <= pos[1] < 26):
                    crosswalk_invalid_target_cells += 1
        b9_season_targets = Counter()
        for entry in b9.get("entries", []):
            for target in entry.get("targets", []):
                b9_season_targets[target.get("season")] += 1
                for pos in target.get("exactCanonicalCells", []):
                    if not isinstance(pos, list) or len(pos) != 2 or not all(isinstance(n,int) for n in pos) or not (0 <= pos[0] < 16 and 0 <= pos[1] < 26):
                        crosswalk_invalid_target_cells += 1
        if crosswalk_invalid_target_cells:
            errors.append(f"Crosswalk references {crosswalk_invalid_target_cells} invalid canonical coordinates")
        if any(e.get("roleCertified") is not False for e in b7.get("entries", [])):
            errors.append("B48R7 unexpectedly claims certified roles")
        if b9.get("mappingAuthority") != "pixel_identical_addressing_only" or b9.get("productionEnabled") is not False:
            errors.append("B48R9 has incompatible production/authority flags")
        for e in b9.get("entries", []):
            if any(e.get(flag) is not False for flag in ("semanticRoleVerified", "editorRuntimeBindingVerified", "collisionVerified")):
                errors.append("B48R9 unexpectedly claims verified semantics/runtime/collision")
                break
        actual_topo_sha = found["topology"]["sha256"]
        expected_topo_sha = prov.get("output_sha256", "")
        provenance_ok = expected_topo_sha == actual_topo_sha
        if not provenance_ok:
            review.append("Topology provenance output_sha256 does not match recovered topology; investigate revision lineage, do not rewrite either historical source")
        if prov.get("source_lock") != PIN:
            errors.append("Provenance source revision does not match pinned ElizaWy commit")
        if "v7" in json.dumps(b9.get("sourceArchives", {})).lower():
            review.append("Check sourceArchive references; V7 must not become a terrain renderer authority")
        report["historical"] = {
            "summer": {"grid": grid, "cellSize": 32, "occupied": occupied, "transparent": transparent, "primaryGroups": len(group_counts), "summerSourceSha256": summer.get("sourceSha256")},
            "topology": {"families": len(families), "roleReferences": role_refs, "seasonRoleCoordinateDisagreements": len(topology_role_disagreements), "seasonBindingCounts": dict(season_binding_counts), "recoveredSha256": actual_topo_sha, "provenanceOutputSha256": expected_topo_sha, "provenanceMatches": provenance_ok},
            "crosswalk": {"summerEntries": len(b7.get("entries", [])), "summerStates": b7_counts, "seasonalEntries": len(b9.get("entries", [])), "seasonalSheets": len(b9.get("sourceSheets", [])), "seasonTargetEntries": dict(b9_season_targets), "invalidCanonicalCells": crosswalk_invalid_target_cells, "semanticPromotions": 0},
        }
    manifest_path = root / MANIFEST
    if not manifest_path.is_file():
        errors.append("Core source manifest unavailable")
        entries = {}
    else:
        m = load(manifest_path)
        entries = {e.get("path"): e for e in m.get("entries", []) if isinstance(e, dict)}
    candidate_roots = [asset_root] if asset_root else [root / "assets/elizawy"]
    asset_rows = []
    for asset in ASSETS:
        entry = entries.get(asset)
        expected = entry.get("sha256") if entry else None
        expected_dim = entry.get("imageSizePx") if entry else None
        if not entry:
            errors.append(f"Asset not recorded in core manifest: {asset}")
        file_path = next((base / asset for base in candidate_roots if base is not None and (base / asset).is_file()), None)
        if file_path is None:
            asset_rows.append({"path": asset, "status": "not_hydrated", "manifestSha256": expected, "manifestDimensions": expected_dim})
            continue
        actual = sha(file_path)
        dim = png_dimensions(file_path) if asset.lower().endswith(".png") else None
        status = "source_byte_verified" if actual == expected and (expected_dim is None or dim == tuple(expected_dim)) else "hash_or_dimension_mismatch"
        if status != "source_byte_verified":
            errors.append(f"Hydrated asset does not match core manifest: {asset}")
        asset_rows.append({"path": asset, "status": status, "manifestSha256": expected, "actualSha256": actual, "dimensions": dim, "manifestDimensions": expected_dim})
    report["sourceArt"] = {"assetRoot": str(candidate_roots[0]), "files": asset_rows, "verified": sum(x["status"] == "source_byte_verified" for x in asset_rows), "available": sum(x["status"] != "not_hydrated" for x in asset_rows), "required": len(asset_rows)}
    if "historical" in report:
        manifest_summer = entries.get("Terrain/terrain_summer.png", {}).get("sha256")
        if report["historical"]["summer"]["summerSourceSha256"] != manifest_summer:
            errors.append("Historical Summer source SHA disagrees with current ForgePY source manifest")
    report["evidenceReady"] = len(found) == len(REQUIRED) and not errors
    report["pixelSourceVerified"] = report["sourceArt"]["verified"] == report["sourceArt"]["required"] and not errors
    return report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--assets-root", type=Path, help="Directory containing Terrain/*.png; default: PROJECT/assets/elizawy")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    try:
        report = inspect(args.root.resolve(), args.assets_root.resolve() if args.assets_root else None)
        path = args.output or args.root / OUTPUT
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"ELIZAWY CONCORDANCE: {'PASS' if report['evidenceReady'] else 'FAIL'} / evidence {len(report['verifiedDocuments'])}/{len(REQUIRED)} / source art {report['sourceArt']['verified']}/{report['sourceArt']['required']} / 0 promoted")
        print(f"REVIEW FLAGS: {len(report['reviewFlags'])} / ERRORS: {len(report['errors'])}")
        print(f"REPORT: {path}")
        return 0 if report["evidenceReady"] else 2
    except (ValueError, KeyError, TypeError, OSError, json.JSONDecodeError) as exc:
        print(f"ELIZAWY CONCORDANCE: FAIL / {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
