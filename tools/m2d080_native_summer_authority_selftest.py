#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"
DONOR_MAP = ROOT / "content/terrain/recovered/native/lpc_terrain_summer_complete_map_32.native.v1.json"
DONOR_TOPOLOGY = ROOT / "content/terrain/recovered/native/lpc_seasonal_terrain_topology.native.v1.json"
REVIEW = ROOT / "content/terrain/recovered/native/native_summer_authority_review.v1.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
MAIN = ROOT / "src/main.rs"
WORLD = ROOT / "src/world_doc.rs"
AUTHORITY = ROOT / "src/asset_authority.rs"
PCC = ROOT / "ProjectControlCenter.py"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def check(condition: bool, message: str):
    if not condition:
        raise SystemExit(f"M2D080 SELFTEST: FAIL - {message}")


def main() -> int:
    profile = load(PROFILE)
    donor = load(DONOR_MAP)
    topology = load(DONOR_TOPOLOGY)
    review = load(REVIEW)
    runtime = load(RUNTIME)

    check(profile.get("version") in {3, 4, 5, 6, 7}, "Summer profile must be v3-v7 source authority")
    check(profile.get("sourceAtlas") == "Terrain/terrain_summer.png", "wrong source atlas")
    check(profile.get("sourceAtlasSha256") == "1251a6ea556330190ccb1e3af166eb69fbec4b7a3f7d51c1f54728abd75bd752", "wrong pinned source hash")
    check(donor.get("summary", {}).get("mappedNonTransparentCells") == 305, "donor does not prove 305 mapped cells")
    check(donor.get("summary", {}).get("nonTransparentCells") == 305, "donor non-transparent count changed")
    check(donor.get("summary", {}).get("structuralTransparentCells") == 14, "structural transparent count changed")
    check(donor.get("summary", {}).get("unusedTransparentCells") == 97, "unused transparent count changed")
    check(topology.get("summary", {}).get("familyCount") == 20, "Native topology must retain 20 families")
    check(review.get("sourceMutationAllowed") is False, "review must forbid source mutation")

    safe = profile.get("safeFill", {})
    direct = profile.get("cornerRecipes", {})
    comps = profile.get("cornerComposites", {})
    core_roles = {"Grass", "MudBank", "RiverWater"}
    core_safe = {k:v for k,v in safe.items() if k in core_roles}
    core_direct = {k:v for k,v in direct.items() if set(k.split("|")) <= core_roles}
    core_comps = {k:v for k,v in comps.items() if set(k.split("|")) <= core_roles}
    check(set(core_safe) == core_roles, "core flat safe fills changed")
    check(len(core_direct) == 36, f"expected 36 core direct mixed recipes, got {len(core_direct)}")
    check(len(core_comps) == 42, f"expected 42 core evidence composites, got {len(core_comps)}")
    check(len(core_safe) + len(core_direct) + len(core_comps) == 81, "core Summer G/D/W authority is not 81/81")
    check(set(safe) >= {"Sand", "WetSand", "PebblePath", "DeepWater"}, "extended Summer materials missing")
    for pair in [
        frozenset(("Grass", "MudBank")),
        frozenset(("Grass", "RiverWater")),
        frozenset(("MudBank", "RiverWater")),
    ]:
        count = sum(1 for key in list(direct) + list(comps) if frozenset(key.split("|")) == pair)
        check(count == 14, f"pair {sorted(pair)} has {count}/14 mixed states")

    regions = {row["canonicalRegionId"]: row for row in runtime["canonicalRuntimeRegions"]}
    for entry in list(safe.values()) + list(direct.values()):
        check(entry["sourcePath"] == "Terrain/terrain_summer.png", "direct recipe left Summer source")
        row = regions.get(entry["canonicalRegionId"])
        check(row is not None, "direct recipe canonical region missing")
        check(row["sourcePath"] == entry["sourcePath"] and row["sourceRectPx"] == entry["sourceRectPx"], "direct recipe is not exact canonical source region")
    for recipe in comps.values():
        check(len(recipe.get("parts", [])) == 4, "checkerboard composite must have four exact source quadrants")
        destinations = set()
        for part in recipe["parts"]:
            check(part["sourcePath"] == "Terrain/terrain_summer.png", "composite escaped Summer source")
            check(part["sourceRectPx"][2:] == [16, 16], "composite source part is not a 16x16 exact quadrant")
            check(part["destinationRectPx"][2:] == [16, 16], "composite destination is not a 16x16 quadrant")
            destinations.add(tuple(part["destinationRectPx"][:2]))
            parent = regions.get(part["canonicalRegionId"])
            check(parent is not None and parent["sourcePath"] == part["sourcePath"], "composite parent canonical region missing")
            sx, sy, sw, sh = part["sourceRectPx"]
            px, py, pw, ph = parent["sourceRectPx"]
            check(px <= sx and py <= sy and sx + sw <= px + pw and sy + sh <= py + ph, "composite source slice leaves canonical parent")
        check(destinations == {(0, 0), (16, 0), (0, 16), (16, 16)}, "composite does not cover all output quadrants")

    main_text = MAIN.read_text(encoding="utf-8")
    world_text = WORLD.read_text(encoding="utf-8")
    authority_text = AUTHORITY.read_text(encoding="utf-8")
    pcc_text = PCC.read_text(encoding="utf-8")
    check(re.search(r"Havenwild — Bevy Studio v0\.\d+\.\d+", main_text) is not None, "Studio 0.x version marker missing")
    check("World Properties" in main_text and "Paint on canvas" in main_text, "friendly world-properties collision workflow missing")
    check("TraversalMode::Wadeable" in main_text and "TraversalMode::Swimmable" in main_text, "traversal workflow incomplete")
    check("clamp(0.125, 16.0)" in main_text, "16x world zoom missing")
    check("resolved_visual_parts_with_authority" in world_text, "shared exact-source visual resolver missing")
    check("worldgen_plan::terrain_role(seed, world)" in world_text, "purpose-first generated terrain-role wiring missing")
    check("corner_composites" in authority_text and "1..=7" in authority_text, "v7 extended authority is not accepted by runtime")
    check("ONE-PASS Native -> Bevy Summer authority convergence" in pcc_text, "PCC one-pass command missing")

    print("M2D080 NATIVE SUMMER AUTHORITY SELFTEST: PASS")
    print("  full source sheet: 305/305 non-transparent cells cataloged")
    print("  donor topology   : 20 source-authored families retained/reviewed")
    print(f"  flat paint       : 81/81 G/D/W baseline + {len(safe)-3} promoted material fills + extended source transitions")
    print("  source mutation  : OFF")
    print("  generated art    : not runtime authority")
    print("  traversal/collision local + reusable metadata workflow: present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
