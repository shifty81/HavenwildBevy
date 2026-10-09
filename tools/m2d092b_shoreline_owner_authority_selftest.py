#!/usr/bin/env python3
"""M2D092B regression gate for recovered Summer shoreline owner authority."""
from __future__ import annotations

import json
import re
from pathlib import Path
from source_version_contract import cargo_source_version, project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"
TOPOLOGY = ROOT / "content/terrain/recovered/native/lpc_seasonal_terrain_topology.native.v1.json"
DONOR = ROOT / "content/terrain/recovered/native/lpc_terrain_summer_complete_map_32.native.v1.json"
CARGO = ROOT / "Cargo.toml"
BUILD = ROOT / "tools/build_summer_complete_runtime.py"
PLAN = ROOT / "src/worldgen_plan.rs"
WORLD = ROOT / "src/world_doc.rs"
PROJECT = ROOT / "project/forgepy.project.json"
CONTRACT = ROOT / "content/worldgen/havenwild_generated_world.v1.json"

ROLE_TO_CORNERS = {
    "outer_north_west": (0, 0, 0, 1),
    "outer_north": (0, 0, 1, 1),
    "outer_north_east": (0, 0, 1, 0),
    "outer_west": (0, 1, 0, 1),
    "outer_east": (1, 0, 1, 0),
    "outer_south_west": (0, 1, 0, 0),
    "outer_south": (1, 1, 0, 0),
    "outer_south_east": (1, 0, 0, 0),
    "inner_south_east": (1, 1, 1, 0),
    "inner_south_west": (1, 1, 0, 1),
    "inner_north_east": (1, 0, 1, 1),
    "inner_north_west": (0, 1, 1, 1),
}

EXPECTED = {
    "summer_grass_over_dirt": ("Grass", "MudBank", 8, 4),
    "summer_grass_over_sand": ("Grass", "Sand", 8, 4),
    "summer_pebble_path_over_dirt": ("PebblePath", "MudBank", 8, 0),
    "summer_grass_bank_pond_a": ("RiverWater", "Grass", 8, 4),
    "summer_dirt_bank_pond_a": ("RiverWater", "MudBank", 8, 4),
    "summer_sand_bank_pond": ("RiverWater", "Sand", 8, 4),
    "summer_sand_over_wet_sand": ("WetSand", "Sand", 8, 4),
    "summer_deep_water_basin": ("DeepWater", "RiverWater", 8, 4),
}

CRITICAL_RECTS = {
    ("summer_sand_over_wet_sand", "outer_north_west"): [288, 160, 32, 32],
    ("summer_sand_over_wet_sand", "inner_south_east"): [288, 256, 32, 32],
    ("summer_sand_bank_pond", "outer_north_west"): [0, 640, 32, 32],
    ("summer_sand_bank_pond", "inner_south_east"): [96, 640, 32, 32],
    ("summer_deep_water_basin", "outer_north_west"): [0, 736, 32, 32],
    ("summer_deep_water_basin", "inner_south_east"): [96, 736, 32, 32],
}

PALE_SHORE_RECTS = {
    "outer_north_west": [320, 640, 32, 32],
    "outer_north": [352, 640, 32, 32],
    "outer_north_east": [384, 640, 32, 32],
    "outer_west": [320, 672, 32, 32],
    "outer_east": [384, 672, 32, 32],
    "outer_south_west": [320, 704, 32, 32],
    "outer_south": [352, 704, 32, 32],
    "outer_south_east": [384, 704, 32, 32],
    "inner_south_east": [416, 640, 32, 32],
    "inner_south_west": [448, 640, 32, 32],
    "inner_north_east": [416, 672, 32, 32],
    "inner_north_west": [448, 672, 32, 32],
}


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D092B SELFTEST: FAIL - {message}")


def main() -> int:
    profile = load(PROFILE)
    topology = load(TOPOLOGY)
    donor = load(DONOR)
    project = load(PROJECT)
    contract = load(CONTRACT)
    cargo = CARGO.read_text(encoding="utf-8")
    build = BUILD.read_text(encoding="utf-8")
    plan = PLAN.read_text(encoding="utf-8")
    world = WORLD.read_text(encoding="utf-8")

    require(source_at_least(cargo, "0.10.13"), "Cargo source predates 0.10.13")
    require(project_matches_cargo(project, cargo), "Forge project sourceVersion does not match Cargo")
    require(profile.get("version") == 8, "Summer runtime profile is not v8")
    require(topology.get("summary", {}).get("familyCount") == 20, "Native topology no longer proves 20 Summer families")
    require(donor.get("summary", {}).get("mappedNonTransparentCells") == 305, "Native Summer map no longer proves 305/305")
    require(contract.get("status") in {"m2d092b_shoreline_owner_authority_recovered", "m2d092c_unified_world_compositor_active", "m2d092c3_visual_runtime_recovery_active", "m2d093a_flat_terrain_authority_convergence_active", "m2d097a_visible_renderer_convergence_active", "m2d097b_water_scrutiny_active"}, "world contract is not a compatible shoreline-authority milestone")

    families = profile.get("ownerSideTransitions", {})
    require(set(families) == set(EXPECTED), "active owner-side family set changed")
    corner_recipes = profile.get("cornerRecipes", {})
    for family_id, (owner, neighbor, outer_count, inner_count) in EXPECTED.items():
        family = families[family_id]
        require(family.get("topologyRule") == "shared_8_neighbor_owner_side", f"{family_id} lost owner-side rule")
        require(family.get("owner") == owner and family.get("neighbor") == neighbor, f"{family_id} owner/neighbour reversed")
        require(family.get("outerRoleCount") == outer_count, f"{family_id} outer role count changed")
        require(family.get("innerRoleCount") == inner_count, f"{family_id} inner role count changed")
        roles = family.get("roles", {})
        require(len(roles) == outer_count + inner_count, f"{family_id} role inventory changed")
        for source_role, bits in ROLE_TO_CORNERS.items():
            entry = roles.get(source_role)
            if entry is None:
                continue
            key = "|".join(owner if bit else neighbor for bit in bits)
            live = corner_recipes.get(key)
            require(live is not None, f"{family_id}/{source_role} missing live corner recipe {key}")
            require(live.get("canonicalRegionId") == entry.get("canonicalRegionId"), f"{family_id}/{source_role} live recipe does not point at exact source role")
            require(live.get("sourceRectPx") == entry.get("sourceRectPx"), f"{family_id}/{source_role} source rectangle changed")

    for (family_id, role), rect in CRITICAL_RECTS.items():
        require(families[family_id]["roles"][role]["sourceRectPx"] == rect, f"critical source rectangle changed: {family_id}/{role}")

    overlays = profile.get("shorelineOverlayFamilies", {})
    pale = overlays.get("summer_pale_shore_ring_overlay")
    require(pale is not None, "pale-shore family is not certified/addressable")
    require(pale.get("topologyRule") == "shared_8_neighbor_owner_side", "pale shore lost owner-side rule")
    require(pale.get("owner") == "RiverWater" and pale.get("neighbor") == "pale_shore", "pale shore ownership changed")
    require(pale.get("outerRoleCount") == 8 and pale.get("innerRoleCount") == 4, "pale shore is not the exact 12-role ring")
    require(pale.get("runtimeUse") in {"certified_source_visual_family_not_automatic_semantic_fill", "certified_source_visual_bridge_for_wetsand"}, "historical pale-shore evidence contract changed unexpectedly")
    for role, rect in PALE_SHORE_RECTS.items():
        require(pale["roles"][role]["sourceRectPx"] == rect, f"pale-shore source rectangle changed: {role}")

    require("ACTIVE_OWNER_SIDE_FAMILIES" in build, "builder no longer consumes explicit owner-side families")
    require("native_shared_8_neighbor_owner_side" in build, "builder no longer declares Native topology authority")
    require("TRANSITION_PROMOTIONS" not in build, "old guessed foreground/background promotion returned")
    require("smooth_coarse_noise" in plan and "smoothstep_fixed" in plan, "smooth island-edge noise is missing")
    require("let edge = coarse_hash(seed, world, island.salt, 5) % 1601" not in plan, "old blocky 5x5 coast score returned")
    require("island_edge_noise_is_smooth_across_lattice_boundaries" in plan, "smooth coast regression test missing")
    generator_match = re.search(r'generated_world\.v(\d+)', world)
    require(generator_match is not None and int(generator_match.group(1)) >= 11,
            "generator predates v11 cache invalidation")
    world_runtime = world.split("#[cfg(test)]", 1)[0]
    require("summer_shoreline_overlay_for_pair(" not in world_runtime, "retired pale-shore repaint returned to live resolver")

    visual = contract.get("visualRepair", {})
    for key in ("ownerSideTerrainTransitions", "coastNoise", "paleShore"):
        require(key in visual, f"world contract missing {key}")
    require(contract.get("cliffTopology", {}).get("active") is True, "M2D092A shared cliff topology was lost")
    require(contract.get("cliffDependentFeatures", {}).get("automaticWaterfalls") == "gated_until_continuous_cliff_faces", "waterfall cliff gate changed")

    print("M2D092B SHORELINE OWNER AUTHORITY SELFTEST: PASS")
    print(f"  source       : {cargo_source_version(cargo)} / generator v11 / Summer runtime profile v8")
    print("  owner-side   : 8 active Native topology families verified against live corner recipes")
    print("  coast repair : historical owner-side source families preserved; core live coast now resolves through Tiled/native-Wang")
    print("  pale shore   : exact 12-role source family retained as evidence/reference; second live repaint retired")
    print("  island edge  : smooth deterministic coarse interpolation; blocky 5x5 score cells removed")
    print("  preserved    : shared cliff topology active; waterfall/cave automatic placement still gated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
