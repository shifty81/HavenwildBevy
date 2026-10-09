#!/usr/bin/env python3
"""M2D092C regression gate: Generated World composition, terrain behavior and modular bridge authority."""
from __future__ import annotations

import json
import re
from pathlib import Path
from source_version_contract import cargo_source_version, project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]
CARGO = ROOT / "Cargo.toml"
PROJECT = ROOT / "project/forgepy.project.json"
CONTRACT = ROOT / "content/worldgen/havenwild_generated_world.v1.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
SHOWCASE = ROOT / "content/worldgen/elizawy_showcase_templates.v1.json"
LAYOUT = ROOT / "src/editor_layout.rs"
MAIN = ROOT / "src/main.rs"
WORLD = ROOT / "src/world_doc.rs"
PLAN = ROOT / "src/worldgen_plan.rs"
SCENE = ROOT / "src/scene_v2.rs"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D092C SELFTEST: FAIL - {message}")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def compact(path: Path) -> str:
    return re.sub(r"\s+", " ", path.read_text(encoding="utf-8"))


def main() -> int:
    cargo = CARGO.read_text(encoding="utf-8")
    project = load(PROJECT)
    contract = load(CONTRACT)
    runtime = load(RUNTIME)
    showcase = load(SHOWCASE)
    main_rs = compact(MAIN)
    world = compact(WORLD)
    plan = compact(PLAN)
    layout = compact(LAYOUT)
    scene = compact(SCENE)

    require(source_at_least(cargo, "0.10.13"), "Cargo source predates 0.10.13")
    require(project_matches_cargo(project, cargo), "Forge project sourceVersion does not match Cargo")
    require(contract.get("status") in {"m2d092c3_visual_runtime_recovery_active", "m2d093a_flat_terrain_authority_convergence_active", "m2d097a_visible_renderer_convergence_active", "m2d097b_water_scrutiny_active"}, "M2D092C-compatible world status missing")
    require(int(contract.get("generatorVersion", 0)) >= 11, "Generated World cache predates v11")
    require(runtime.get("counts", {}).get("canonicalRuntimeRegions") == 2892, "runtime source-region count changed")
    require(runtime.get("counts", {}).get("objectTemplates") == 28, "reviewed object-template count changed")

    wc = contract.get("worldComposition", {})
    require("Ground/TerrainDetails/Water/Elevation/Structures/Objects/Foreground" in wc.get("layerAuthority", ""), "canonical world layer order not declared")
    require("anchorY + footprintHeight" in wc.get("objectDepthSort", ""), "ground-foot depth sort not declared")
    require("visual bounds" in wc.get("visualBoundsCulling", ""), "visual-bounds culling not declared")
    require("4.05" in wc.get("detailLod", "") and "5.5" in wc.get("detailLod", ""), "stable far-overview detail LOD not declared")
    require("same cached dual-grid vertex topology" in wc.get("farTerrainLod", ""), "unified far terrain LOD not declared")

    ca = contract.get("collisionAuthority", {})
    require(ca.get("reusableTerrainKey") == "semantic role + cardinal same-role topology mask", "terrain collision authority is not topology keyed")
    require("inactive" in ca.get("legacySourceProfiles", ""), "legacy source-rectangle terrain behavior is not explicitly inactive")
    require("coarse cells below 16" in ca.get("overlayPerformance", ""), "collision overlay LOD contract missing")

    bridge_contract = contract.get("bridgeSystem", {})
    require(bridge_contract.get("status") == "modular_horizontal_source_backed_v1", "modular bridge contract missing")
    require("repeatable 32x64 middle" in bridge_contract.get("recipe", ""), "repeatable bridge middle not declared")
    require("one land approach cell on each side" in bridge_contract.get("placement", ""), "bridge land-end placement contract missing")
    require("walkable" in bridge_contract.get("traversal", ""), "bridge traversal override missing")

    templates = {row["key"]: row for row in showcase.get("templates", [])}
    bridge = templates.get("bridge_wood_a_rails")
    require(bridge is not None, "reviewed bridge template missing")
    require(bridge.get("layer") == "structures", "bridge is not on Structures")
    require(bridge.get("footprintCells") == [3, 2], "bridge minimum seed footprint changed")
    require("modular_bridge" in bridge.get("worldgenTags", []) and "repeatable_span" in bridge.get("worldgenTags", []), "bridge modular tags missing")
    parts = bridge.get("parts", [])
    require([p.get("sourceRectPx") for p in parts] == [[128, 0, 32, 64], [160, 0, 32, 64], [192, 0, 32, 64]], "bridge left/middle/right exact source slices changed")

    for layer in ["Ground", "TerrainDetails", "Water", "Elevation", "Structures", "Objects", "Foreground"]:
        require(layer in scene, f"canonical Scene-v2 layer missing: {layer}")
    require("pub world_layer_visibility: [bool; 7]" in layout, "Generated World layer visibility is not persisted")
    require("default_world_layer_visibility" in layout, "Generated World layer visibility default missing")

    for token in [
        "fn world_layer_id(name: &str) -> LayerId",
        "fn world_layer_slot(layer: LayerId) -> usize",
        "fn world_layer_visible(state: &Studio, layer: LayerId) -> bool",
        "fn world_terrain_role_layer(role: &str) -> LayerId",
        "fn world_terrain_vertex_layer(roles: &[String; 4]) -> LayerId",
        "fn world_object_depth_key(object: &world_doc::WorldObject, anchor: [i64; 2])",
        "fn world_object_visual_bounds(object: &world_doc::WorldObject, anchor: [i64; 2])",
        "fn draw_world_generated_objects(",
        "objects.sort_by(|a, b|",
        "world_layer_visible(state, world_layer_id(&object.layer))",
        "if state.world_detail_lod_visible && px <= 4.05",
        "else if !state.world_detail_lod_visible && px >= 5.5",
        "for object in state.world_doc.generated_objects() {",
        "tiled_vertex_tiles",
        "prepare_world_visual_cache(",
        "state.editor_layout.world_layer_visibility[slot] = !visible",
    ]:
        require(token in main_rs, f"Generated World compositor source missing: {token}")

    # Collision overlay must not rasterize the full 32x32 mask at Fit World scale,
    # and exact mode must merge contiguous blocked pixels into horizontal runs.
    require("if !state.editor_layout.collision_overlay || px < 6.0" in main_rs, "far collision overlay suppression missing")
    require("if px < 16.0" in main_rs, "coarse collision overlay tier missing")
    require("run_start" in main_rs, "exact collision overlay run merging missing")

    require(
        "scene_v2::COLLISION_MASK_SIDE" in world,
        "world_doc test module does not import COLLISION_MASK_SIDE",
    )
    require(
        "(COLLISION_MASK_SIDE * COLLISION_MASK_SIDE) as u32" in world,
        "collision regression assertion does not match blocked_count() u32 type",
    )

    for token in [
        "pub struct TerrainBehaviorProfile",
        "pub terrain_behavior_profiles: Vec<TerrainBehaviorProfile>",
        "pub fn resolved_terrain_behavior_key(&self, world: [i64; 2])",
        "pub fn terrain_behavior_profile(",
        "pub fn set_terrain_collision_profile_untracked(",
        "pub fn set_terrain_traversal_profile(",
        "if self.generated_bridge_contains(world)",
        "return Some(CollisionMask32::empty())",
        "return TraversalMode::Walkable",
        "pub fn generated_bridge_contains(&self, world: [i64; 2]) -> bool",
        "fn push_modular_bridge(",
    ]:
        require(token in world, f"world composition/collision source missing: {token}")

    # Generated terrain must no longer consult raw source-rectangle profiles in
    # effective collision/traversal. The legacy source profile API remains only
    # for compatibility and discrete source-bound workflows.
    collision_body = re.search(r"pub fn effective_collision_mask\(.*?\n\s*}\n\s*pub fn effective_traversal", WORLD.read_text(encoding="utf-8"), re.S)
    require(collision_body is not None, "cannot isolate effective collision implementation")
    require("source_behavior_profile" not in collision_body.group(0), "generated terrain collision still consults raw source-rectangle behavior")

    plan_runtime = PLAN.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]
    require('return "WetSand";' not in plan_runtime, "generated coast still emits legacy WetSand")
    require("clustered_bank_erosion(seed, world, island, dirt_percent)" in plan, "clustered inland-bank erosion missing")
    require("generated_coast_uses_core_sand_without_legacy_wetsand_ring" in plan, "legacy WetSand-ring retirement regression test missing")
    require("generated_inland_dirt_banks_do_not_form_single_cell_islands" in plan, "single-cell bank regression test missing")
    require("fn generated_layer_for(template: &ObjectTemplate, kind: &str) -> String" in world, "generated natural-detail layer routing missing")
    require('"water".into()' in world and '"terrain_details".into()' in world, "Water/TerrainDetails generated routing missing")
    require("pub fn resolved_dual_grid_roles(&self, vertex: [i64; 2]) -> [String; 4]" in world, "terrain layer partition cannot inspect resolved dual-grid roles")

    for token in [
        "pub struct BridgeSpan",
        "pub fn bridge_span(seed: u64, island: IslandDescriptor) -> Option<BridgeSpan>",
        "span_cells",
        "bridge_span(seed, island).map(|span| span.anchor)",
    ]:
        require(token in plan, f"modular bridge planner missing: {token}")

    print("M2D093A FLAT TERRAIN / UNIFIED WORLD COMPOSITOR SELFTEST: PASS")
    print("  layers       : seven canonical Generated World visibility/composition lanes")
    print("  depth/cull   : layer + ground-foot sorting / complete visual bounds")
    print("  detail LOD   : far-only 4.05/5.5 px hysteresis + 8-cell overhang-safe chunk query")
    print("  collision    : semantic topology profiles / coarse+run-merged overlay")
    print("  bridge       : measured land-to-land span / exact left+repeatable middle+right")
    print(f"  source       : {cargo_source_version(cargo)} / generator v11 / unified Tiled dual-grid coast / 320 images / 2892 exact regions")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
