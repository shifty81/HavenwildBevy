#!/usr/bin/env python3
from __future__ import annotations
import json
import re
from pathlib import Path
from source_version_contract import cargo_source_version, project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]
CARGO = ROOT / "Cargo.toml"
SHOWCASE = ROOT / "content/worldgen/elizawy_showcase_templates.v1.json"
CONTRACT = ROOT / "content/worldgen/havenwild_generated_world.v1.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
PLAN = ROOT / "src/worldgen_plan.rs"
WORLD = ROOT / "src/world_doc.rs"
MAIN = ROOT / "src/main.rs"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D091 SELFTEST: FAIL - {message}")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    cargo = CARGO.read_text(encoding="utf-8")
    showcase = load(SHOWCASE)
    contract = load(CONTRACT)
    runtime = load(RUNTIME)
    plan = PLAN.read_text(encoding="utf-8")
    world = WORLD.read_text(encoding="utf-8")
    world_runtime = world.split("#[cfg(test)]", 1)[0]
    main_rs = MAIN.read_text(encoding="utf-8")
    # Source-contract assertions must survive rustfmt.  The Full Gate is allowed
    # to normalize Rust formatting, so semantic checks compare whitespace-
    # collapsed source instead of pinning one formatter layout.
    plan_compact = re.sub(r"\s+", " ", plan)
    world_compact = re.sub(r"\s+", " ", world)
    world_runtime_compact = re.sub(r"\s+", " ", world_runtime)
    main_compact = re.sub(r"\s+", " ", main_rs)

    require(source_at_least(cargo, "0.10.13"), "Cargo source predates 0.10.13")
    require(contract.get("profileId") == "havenwild.four_season_archipelago.v3", "world profile is not v3")
    require(contract.get("status") in {"m2d091_visual_authority_repair_active", "m2d092a_shared_cliff_topology_active", "m2d092b_shoreline_owner_authority_recovered", "m2d092c_unified_world_compositor_active", "m2d092c3_visual_runtime_recovery_active", "m2d093a_flat_terrain_authority_convergence_active", "m2d097a_visible_renderer_convergence_active", "m2d097b_water_scrutiny_active"}, "visual-authority repair status missing")
    require(runtime.get("counts", {}).get("runtimeSourceImages") == 320, "runtime source-image lane changed")
    require(runtime.get("counts", {}).get("canonicalRuntimeRegions") == 2892, "exact-region count changed")
    require(runtime.get("counts", {}).get("objectTemplates") == 28, "reviewed template count changed")

    templates = {row["key"]: row for row in showcase.get("templates", [])}
    fall = templates.get("waterfall_front_a_static")
    cave = templates.get("cave_cliff_feature_a")
    bridge = templates.get("bridge_wood_a_rails")
    require(fall is not None and fall["parts"][0]["sourceRectPx"] == [0, 32, 96, 128],
            "south-facing waterfall still includes the north cap")
    require(fall.get("footprintCells") == [3, 4], "waterfall footprint does not match reviewed crop")
    require(cave is not None and cave["parts"][0]["sourceRectPx"] == [224, 288, 160, 96],
            "cave template is not the complete wide entrance crop")
    require(cave.get("footprintCells") == [5, 3], "cave footprint does not match complete crop")
    require(bridge is not None and bridge.get("footprintCells") == [3, 2], "bridge seed footprint changed")
    require(len(bridge.get("parts", [])) == 3, "bridge modular source recipe must expose left/middle/right")
    require([part["sourceRectPx"] for part in bridge["parts"]] == [[128, 0, 32, 64], [160, 0, 32, 64], [192, 0, 32, 64]], "bridge modular source slices changed")
    require("repeatable_span" in bridge.get("worldgenTags", []), "bridge middle span is not declared repeatable")

    for token in [
        'WORLD_PROFILE_ID: &str = "havenwild.four_season_archipelago.v3"',
        '"spring_isle" => {',
        '"summer_isle" => {',
        '"autumn_isle" => {',
        'one semantic cell wide',
        'clustered_bank_erosion(seed, world, island, dirt_percent)',
        'if score >= 9_050',
        'generated_coast_uses_core_sand_without_legacy_wetsand_ring',
        'return "MudBank";',
        'waterfall_anchor(seed: u64, island: IslandDescriptor) -> Option<[i64; 2]>',
        'bridge_anchor(seed: u64, island: IslandDescriptor) -> Option<[i64; 2]>',
        'seasonal_islands_have_distinct_hydrology_signatures',
        'pub const CLIFF_NORTH: u8 = 0b0001',
        'pub const CLIFF_EAST: u8 = 0b0010',
        'pub const CLIFF_SOUTH: u8 = 0b0100',
        'pub const CLIFF_WEST: u8 = 0b1000',
        'pub fn cliff_mask(seed: u64, world: [i64; 2]) -> u8',
    ]:
        require(token in plan_compact, f"worldgen repair source missing {token}")

    for token in [
        '"roadside lighting"',
        'standalone feature sprites',
        'matches!(settlement.island_id, "summer_isle" | "autumn_isle")',
        'settlement_house_kind(settlement.class)',
        'find_buildable_anchor_near(seed, primary_center, primary.footprint_cells, 7)',
        'cliff_mask: worldgen_plan::cliff_mask(seed, world)',
    ]:
        require(token in world_compact, f"world object placement repair missing {token}")
    require(re.search(r'WORLD_GENERATOR_ID: &str = "havenwild\.source_backed\.generated_world\.v(\d+)"', world_compact) is not None,
            "world generator identity missing")
    generator_match = re.search(r'generated_world\.v(\d+)', world_compact)
    require(generator_match is not None and int(generator_match.group(1)) >= 11,
            "world generator predates v11 visual-authority repair")
    require('templates_with_tag("waterfall")' not in world_runtime_compact, "waterfall still auto-places before cliff authority")
    require('templates_with_tag("cave_entrance")' not in world_runtime_compact, "cave entrance still auto-places before cliff authority")
    require('"Worldgen waterfall",' not in world_compact, "stale Rust test still requires gated waterfall placement")
    require('"Worldgen cliff cave entrance",' not in world_compact, "stale Rust test still requires gated cave placement")

    require('source_uv_object(_source_asset: &str, rect: [u32; 4], sheet: [u32; 2])' in main_rs,
            "object render path no longer exposes source-bound sampling")
    require('source_uv(rect, sheet)' in main_compact,
            "object render path no longer uses normal half-pixel atlas sampling")
    require('tree-specific UV inset' in main_rs,
            "tree-specific UV-inset retirement contract missing")

    visual = contract.get("visualRepair", {})
    for key in ["treeAtlasSampling", "waterfallCrop", "caveCrop", "bridgeAnchor", "bankPolicy", "sandPolicy"]:
        require(key in visual, f"world contract missing visual repair rule {key}")
    cliff_features = contract.get("cliffDependentFeatures", {})
    require(cliff_features.get("automaticWaterfalls") == "gated_until_continuous_cliff_faces", "waterfall cliff gate missing")
    require(cliff_features.get("automaticCaveEntrances") == "gated_until_continuous_cliff_faces", "cave cliff gate missing")
    cliff_topology = contract.get("cliffTopology", {})
    require(cliff_topology.get("active") is True, "shared cliff topology is not active")
    require(cliff_topology.get("authority") == "elevation_neighbor_drop_mask", "shared cliff topology authority changed")
    require(cliff_topology.get("bits") == {"north": 1, "east": 2, "south": 4, "west": 8}, "cliff bit contract changed")
    require(cliff_topology.get("persistedOnGeneratedCell") is True, "cliff topology is not persisted on generated cells")

    print("M2D091 VISUAL AUTHORITY REPAIR SELFTEST: PASS")
    print("  islands      : season-specific silhouettes/hydrology/biome density")
    print("  shoreline    : clustered inland erosion + core Sand/Shallow/Deep Tiled topology; legacy WetSand ring retired")
    print("  roads        : one-cell source-backed dirt road; PebblePath no longer worldgen default")
    print("  objects      : exact hydrated alpha render bounds replace tree/rock atlas-slot hacks")
    print("  bridge       : modular exact-source left/middle/right recipe on measured road/water crossing")
    print("  waterfall    : corrected south-facing crop retained; auto placement gated on cliffs")
    print("  cave         : complete 160x96 recipe retained; auto placement gated on cliffs")
    print("  lighting     : roadside placement separated from house frontage")
    print("  settlements  : class-correct house labels + deterministic parcel fallback")
    print("  cliff topo   : shared cardinal elevation-drop mask persisted per generated cell")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
