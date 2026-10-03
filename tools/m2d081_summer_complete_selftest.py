#!/usr/bin/env python3
from __future__ import annotations
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
DONOR = ROOT / "content/terrain/recovered/native/lpc_terrain_summer_complete_map_32.native.v1.json"
AUTH = ROOT / "src/asset_authority.rs"
WORLD = ROOT / "src/world_doc.rs"
MAIN = ROOT / "src/main.rs"
GEN = ROOT / "tools/build_summer_complete_runtime.py"

def load(p): return json.loads(p.read_text(encoding="utf-8"))
def require(ok, msg):
    if not ok: raise SystemExit(f"M2D081 SELFTEST: FAIL - {msg}")

def main():
    profile, runtime, donor = map(load, (PROFILE, RUNTIME, DONOR))
    require(profile.get("version") == 7, "runtime profile is not v7 land-connectivity/material expansion")
    require(donor.get("summary", {}).get("mappedNonTransparentCells") == 305, "donor is not 305/305")
    groups = profile.get("sourceGroups", {})
    require(len(groups) == 41, f"expected 41 Summer groups, got {len(groups)}")
    require(sum(g.get("cellCount", 0) for g in groups.values()) == 305, "source groups do not cover 305 cells")
    require(runtime.get("counts", {}).get("canonicalRuntimeRegions", 0) >= 2890, "runtime authority does not include M2D090 seasonal/showcase source regions")
    runtime_regions = {r["canonicalRegionId"]: r for r in runtime["canonicalRuntimeRegions"]}
    grouped_ids = [entry["canonicalRegionId"] for group in groups.values() for entry in group.get("cells", [])]
    require(len(grouped_ids) == 305 and len(set(grouped_ids)) == 305, "source groups do not expose 305 unique canonical Summer cells")
    require(all(region_id in runtime_regions for region_id in grouped_ids), "a grouped Summer cell is absent from runtime authority")
    require(all(runtime_regions[region_id]["sourcePath"] == "Terrain/terrain_summer.png" for region_id in grouped_ids), "a grouped Summer cell escaped the source atlas")

    safe = profile.get("safeFill", {})
    direct = profile.get("cornerRecipes", {})
    comps = profile.get("cornerComposites", {})
    core_roles = {"Grass", "MudBank", "RiverWater"}
    core_safe = {k:v for k,v in safe.items() if k in core_roles}
    core_direct = {k:v for k,v in direct.items() if set(k.split("|")) <= core_roles}
    core_comps = {k:v for k,v in comps.items() if set(k.split("|")) <= core_roles}
    require(len(core_safe) == 3 and len(core_direct) == 36 and len(core_comps) == 42, "81-state G/D/W baseline changed")
    require(len(core_safe) + len(core_direct) + len(core_comps) == 81, "Summer G/D/W baseline is not 81/81")
    require(set(safe) >= {"Sand","WetSand","PebblePath","DeepWater"}, "extended source-backed terrain safe fills missing")
    pairs = [frozenset(("Grass","MudBank")), frozenset(("Grass","RiverWater")), frozenset(("MudBank","RiverWater"))]
    for pair in pairs:
        n = sum(1 for k in list(direct) + list(comps) if frozenset(k.split("|")) == pair)
        require(n == 14, f"pair {sorted(pair)} is {n}/14")
    require(sum(1 for k in comps if len(set(k.split("|"))) == 3) == 36, "three-material junction count is not 36")

    variants = profile.get("fillVariants", {})
    require([len(variants.get(r, [])) for r in ("Grass","MudBank","RiverWater")] == [6,6,1], "core fill variant counts changed")
    require([len(variants.get(r, [])) for r in ("Sand","WetSand","PebblePath","DeepWater")] == [3,3,4,1], "extended fill variant counts changed")
    require("RiverWater" not in profile.get("animations", {}), "RepeatableFill water variants must not be promoted into temporal animation")
    water_group = groups.get("summer_water_fill", {})
    require(water_group.get("classification") == "RepeatableFill" and water_group.get("cellCount") == 8, "water source variants must remain cataloged as eight static RepeatableFill cells")
    require(variants["RiverWater"][0].get("sourceRectPx") == [384,512,32,32], "conservative RiverWater base fill changed")

    # Checkerboard regression: Dirt is isolated in Grass; Grass/Dirt are isolated in Water.
    expected_background = {
        frozenset(("Grass","MudBank")): "Grass",
        frozenset(("Grass","RiverWater")): "RiverWater",
        frozenset(("MudBank","RiverWater")): "RiverWater",
    }
    for key, recipe in comps.items():
        corners = key.split("|")
        roles = frozenset(corners)
        if len(roles) != 2 or not (corners[0] == corners[3] and corners[1] == corners[2] and corners[0] != corners[1]):
            continue
        bg = expected_background[roles]
        require(len(recipe.get("parts", [])) == 4, f"checker {key} is not four exact quadrants")
        # Background quadrants must come from the safe-fill canonical parent.
        bg_id = safe[bg]["canonicalRegionId"]
        for i, role in enumerate(corners):
            if role == bg:
                require(recipe["parts"][i]["canonicalRegionId"] == bg_id, f"checker {key} bridges background at quadrant {i}")


    # M2D082C regression: all 36 three-material junctions keep RiverWater as the
    # quadrant background. This is the exact topology family exposed by the user's
    # narrow inlet / opposing grass+dirt bank screenshots.
    water_bg_id = safe["RiverWater"]["canonicalRegionId"]
    tri_count = 0
    for key, recipe in comps.items():
        corners = key.split("|")
        if len(set(corners)) != 3:
            continue
        tri_count += 1
        require("RiverWater" in corners, f"unexpected non-water tri-material state {key}")
        require(len(recipe.get("parts", [])) == 4, f"tri-material {key} is not four exact quadrants")
        for i, role in enumerate(corners):
            if role == "RiverWater":
                require(recipe["parts"][i]["canonicalRegionId"] == water_bg_id, f"tri-material {key} lost water background at quadrant {i}")
    require(tri_count == 36, f"expected 36 water-continuity tri-material states, got {tri_count}")

    auth_text = AUTH.read_text(encoding="utf-8")
    world_text = WORLD.read_text(encoding="utf-8")
    main_text = MAIN.read_text(encoding="utf-8")
    gen_text = GEN.read_text(encoding="utf-8")
    require("summer_flatworld_fill_for_world" in auth_text, "deterministic fill variation API missing")
    require("RepeatableFill variants are static source/detail choices" in auth_text, "water animation evidence gate missing")
    require("contains_source_slice" in auth_text, "canonical source-slice authority API missing")
    require("resolved_visual_parts_with_authority_at" in world_text, "time-aware world resolver missing")
    require("strict Summer flatworld paint must resolve visual parts" in world_text, "flatworld source-sheet regression is not composite-aware")
    require("a.contains_source_slice(&part.source)" in world_text, "flatworld regression does not validate canonical source slices")
    require("strict Summer flatworld paint must resolve\")" not in world_text, "stale single-binding flatworld regression remains")
    require("water_animation_ms" not in main_text, "stale fake water animation clock remains")
    require(re.search(r"Havenwild — Bevy Studio v0\.\d+\.\d+", main_text) is not None, "Studio 0.x marker missing")
    require("land_connectivity_first_no_quadrant_splice" in PROFILE.read_text(encoding="utf-8"), "land-connectivity policy missing from profile")
    require("TRANSITION_PROMOTIONS" in gen_text, "extended terrain transition promotion missing")
    print("M2D082F SUMMER MATERIAL / LAND-CONNECTIVITY SELFTEST: PASS")
    print("  Summer runtime cells : 305/305 across 41 source groups")
    print(f"  runtime authority    : {runtime.get('counts', {}).get('canonicalRuntimeRegions')} exact regions")
    print("  G/D/W grammar        : 81/81 states (14/14 pairwise + 36 tri-material)")
    print("  homogeneous variants : Grass 6 / Dirt 6 / Water 1 / Sand 3 / WetSand 3 / Path 4 / Deep 1")
    print("  water animation      : OFF / RepeatableFill variants are static source/detail cells")
    print("  ambiguous diagonals  : live resolver connects land; historical composites evidence-only")
    print("  extended materials   : Sand / WetSand / DeepWater / PebblePath source-backed")
    return 0

if __name__ == "__main__": raise SystemExit(main())
