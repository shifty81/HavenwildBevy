#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUTH = ROOT / "content/terrain/recovered/native/cliff_summer_source_authority.v1.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
WORLD = ROOT / "src/world_doc.rs"
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D082 CLIFF SOURCE SELFTEST: FAIL - {message}")


def main() -> int:
    authority = json.loads(AUTH.read_text(encoding="utf-8"))
    runtime = json.loads(RUNTIME.read_text(encoding="utf-8"))
    profile = json.loads(PROFILE.read_text(encoding="utf-8"))
    require(authority.get("schema") == "havenwild.terrain.cliff_summer_source_authority.v1", "wrong schema")
    require(authority.get("sourceAtlas") == "Terrain/cliff_summer.png", "wrong source atlas")
    require(authority.get("sourceSha256") == "94bd2ddb2c51677498453486092ef28e57e0c1880f4334359189229590298147", "wrong pinned cliff source hash")
    require(authority.get("imageSizePx") == [512, 448], "wrong source size")
    summary = authority.get("summary", {})
    require(summary.get("canonicalCells") == 205, "expected 205 canonical cliff cells")
    require(summary.get("historicalReferenceEntries") == 212, "expected 212 retained historical reference entries")
    require(summary.get("runtimeCliffRolesCertified") == 0, "source inventory must not invent runtime cliff roles")
    require(authority.get("semanticPromotionAllowed") is False, "historical cliff evidence must remain non-promoting")
    cells = authority.get("cells", [])
    require(len(cells) == 205 and len({tuple(cell["grid"]) for cell in cells}) == 205, "cliff source cells are not unique")
    runtime_ids = {
        row["canonicalRegionId"] for row in runtime.get("canonicalRuntimeRegions", [])
        if row.get("sourcePath") == "Terrain/cliff_summer.png"
    }
    require(len(runtime_ids) == 205, "runtime authority does not contain 205 cliff regions")
    require(all(cell.get("canonicalRegionId") in runtime_ids for cell in cells), "cliff inventory escaped canonical runtime source authority")
    families = set(summary.get("historicalFamilyReferenceCounts", {}))
    require(families == {"mountain_base", "mountain_animated_water", "mountain_features", "mountain_vines", "mountain_waterfall_transitions"}, "historical cliff family buckets changed")
    policy = authority.get("havenwildStructuralPolicy", {})
    require(policy.get("elevationRange") == [0, 30] and policy.get("seaLevel") == 0, "Havenwild 0..30 elevation contract missing")
    require(policy.get("oneLevelCliffsAreValid") is True, "+1 cliffs must remain valid")
    require("route-aware" in policy.get("accessPolicy", ""), "route-aware cliff access policy missing")
    require(profile.get("animations", {}).get("RiverWater") is None, "fake flat-water animation leaked into cliff baseline")
    world_text = WORLD.read_text(encoding="utf-8")
    require("World elevation override must be 0..30" in world_text, "world elevation bounds changed")
    print("M2D082 CLIFF SOURCE SELFTEST: PASS")
    print("  cliff source cells    : 205 canonical 32x32 regions")
    print("  historical evidence   : 212 reference entries across 5 non-promoting families")
    print("  runtime cliff roles   : 0 certified (classification is next; no guessing)")
    print("  elevation contract    : sea 0 / +1..+30 valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
