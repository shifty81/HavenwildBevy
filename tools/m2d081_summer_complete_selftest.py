#!/usr/bin/env python3
from __future__ import annotations
import json
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
    require(profile.get("version") == 4, "runtime profile is not v4")
    require(donor.get("summary", {}).get("mappedNonTransparentCells") == 305, "donor is not 305/305")
    groups = profile.get("sourceGroups", {})
    require(len(groups) == 41, f"expected 41 Summer groups, got {len(groups)}")
    require(sum(g.get("cellCount", 0) for g in groups.values()) == 305, "source groups do not cover 305 cells")
    require(runtime.get("counts", {}).get("canonicalRuntimeRegions") == 2316, "runtime authority is not 2316 regions")
    runtime_regions = {r["canonicalRegionId"]: r for r in runtime["canonicalRuntimeRegions"]}
    grouped_ids = [entry["canonicalRegionId"] for group in groups.values() for entry in group.get("cells", [])]
    require(len(grouped_ids) == 305 and len(set(grouped_ids)) == 305, "source groups do not expose 305 unique canonical Summer cells")
    require(all(region_id in runtime_regions for region_id in grouped_ids), "a grouped Summer cell is absent from runtime authority")
    require(all(runtime_regions[region_id]["sourcePath"] == "Terrain/terrain_summer.png" for region_id in grouped_ids), "a grouped Summer cell escaped the source atlas")

    safe = profile.get("safeFill", {})
    direct = profile.get("cornerRecipes", {})
    comps = profile.get("cornerComposites", {})
    require(len(safe) == 3 and len(direct) == 36 and len(comps) == 42, "81-state grammar counts changed")
    require(len(safe) + len(direct) + len(comps) == 81, "Summer G/D/W grammar is not 81/81")
    pairs = [frozenset(("Grass","MudBank")), frozenset(("Grass","RiverWater")), frozenset(("MudBank","RiverWater"))]
    for pair in pairs:
        n = sum(1 for k in list(direct) + list(comps) if frozenset(k.split("|")) == pair)
        require(n == 14, f"pair {sorted(pair)} is {n}/14")
    require(sum(1 for k in comps if len(set(k.split("|"))) == 3) == 36, "three-material junction count is not 36")

    variants = profile.get("fillVariants", {})
    require([len(variants.get(r, [])) for r in ("Grass","MudBank","RiverWater")] == [6,6,8], "fill variant counts changed")
    water = profile.get("animations", {}).get("RiverWater", {})
    require(water.get("frameDurationMs") == 220 and len(water.get("frames", [])) == 8, "water cycle must be 8 source-backed phases @ 220 ms")
    require(water.get("visualCertification") == "runtime_candidate_requires_user_visual_review", "water candidate review marker missing")

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

    auth_text = AUTH.read_text(encoding="utf-8")
    world_text = WORLD.read_text(encoding="utf-8")
    main_text = MAIN.read_text(encoding="utf-8")
    gen_text = GEN.read_text(encoding="utf-8")
    require("summer_flatworld_fill_for_world" in auth_text, "deterministic fill variation API missing")
    require("summer_flatworld_animation_frame" in auth_text, "water animation API missing")
    require("contains_source_slice" in auth_text, "canonical source-slice authority API missing")
    require("resolved_visual_parts_with_authority_at" in world_text, "time-aware world resolver missing")
    require("strict Summer flatworld paint must resolve visual parts" in world_text, "flatworld source-sheet regression is not composite-aware")
    require("a.contains_source_slice(&part.source)" in world_text, "flatworld regression does not validate canonical source slices")
    require("strict Summer flatworld paint must resolve\")" not in world_text, "stale single-binding flatworld regression remains")
    require("Some(state.water_animation_ms)" in main_text, "world renderer is not passing animation time")
    require("Havenwild — Bevy Studio v0.8.1" in main_text, "Studio 0.8.1 marker missing")
    require("checker_background" in gen_text, "disconnected checkerboard generation missing")
    print("M2D081 SUMMER COMPLETE SELFTEST: PASS")
    print("  Summer runtime cells : 305/305 across 41 source groups")
    print("  runtime authority    : 2316 exact regions")
    print("  G/D/W grammar        : 81/81 states (14/14 pairwise + 36 tri-material)")
    print("  homogeneous variants : Grass 6 / Dirt 6 / Water 8")
    print("  water cycle          : 8 source-backed phases @ 220 ms (visual review candidate)")
    print("  checkerboards        : disconnected foreground policy enabled")
    return 0

if __name__ == "__main__": raise SystemExit(main())
