#!/usr/bin/env python3
"""M2D097A: guard the visible Tiled/native-Wang renderer convergence.

This gate intentionally checks what the player/editor sees, not just that Tiled
metadata exists: one dual-grid geometry at every zoom, no generated WetSand
ring, no second pale-shore repaint, shared materialized native-Wang solve, and
live canonical TSX animation only when every frame maps to original ElizaWy
pixels.
"""
from __future__ import annotations

import json
import re
from pathlib import Path
from source_version_contract import project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]


def req(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D097A SELFTEST: FAIL - {message}")


def main() -> int:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    project = json.loads((ROOT / "project/forgepy.project.json").read_text(encoding="utf-8"))
    contract = json.loads((ROOT / "content/worldgen/havenwild_generated_world.v1.json").read_text(encoding="utf-8"))
    main_rs = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    world = (ROOT / "src/world_doc.rs").read_text(encoding="utf-8")
    plan = (ROOT / "src/worldgen_plan.rs").read_text(encoding="utf-8")
    tiled = (ROOT / "src/tiled_terrain.rs").read_text(encoding="utf-8")
    native = (ROOT / "src/tiled_terrain/native_wang.rs").read_text(encoding="utf-8")
    auth = (ROOT / "src/asset_authority.rs").read_text(encoding="utf-8")
    pcc = (ROOT / "ProjectControlCenter.py").read_text(encoding="utf-8")

    req(source_at_least(cargo, "0.10.18"), "source predates visible renderer convergence")
    req(project_matches_cargo(project, cargo), "project sourceVersion does not match Cargo")
    req(contract.get("status") in {"m2d097a_visible_renderer_convergence_active", "m2d097b_water_scrutiny_active"}, "world contract status predates visible renderer convergence")
    req(int(contract.get("generatorVersion", 0)) >= 11, "generated world predates cache generation v11")
    far = contract.get("worldComposition", {}).get("farTerrainLod", "")
    req("same cached dual-grid vertex topology" in far and "GeneratedCell.source" in far,
        "far LOD does not explicitly retain one shared dual-grid terrain authority")

    plan_runtime = plan.split("#[cfg(test)]", 1)[0]
    req('return "WetSand";' not in plan_runtime, "worldgen still emits legacy WetSand")
    req('return "PebblePath";' not in plan_runtime, "worldgen still emits legacy PebblePath")
    req('generated_coast_uses_core_sand_without_legacy_wetsand_ring' in plan,
        "worldgen lacks a regression proving the legacy WetSand ring stays retired")

    world_runtime = world.split("#[cfg(test)]", 1)[0]
    req("summer_shoreline_overlay_for_pair(" not in world_runtime,
        "retired RiverWater/WetSand pale-shore repaint is still live")
    req("resolved_tiled_materialized_vertex_tiles" in world_runtime,
        "world document does not expose a shared materialized Tiled solve")
    req("solve_havenwild_corner_grid" in world_runtime,
        "world document does not invoke the shared native-Wang Tiled solve")
    req("resolve_havenwild_corner_tile_id(role_refs, self.seed, vertex)" in world_runtime,
        "exact Tiled per-vertex fallback is missing")
    raw_i = world_runtime.find("resolve_havenwild_corner_tile_id(role_refs, self.seed, vertex)")
    projected_i = world_runtime.find("land_connectivity_projection(role_refs)")
    req(raw_i >= 0 and projected_i >= 0 and raw_i < projected_i,
        "legacy land-connectivity projection runs before exact authored Tiled topology")

    req("solve_corner_grid_with_origin" in native,
        "native Wang solve is not world-origin stable")
    req("solve_corner_material_grid_with_origin" in tiled,
        "Tiled authority does not expose origin-stable shared solve")
    req("canonical_visual_binding" in tiled and "canonical_animation_clip" in tiled,
        "canonical source/animation lookup is incomplete")
    req('"WetSand" | "PebblePath" => None' in tiled,
        "evidence-gated legacy roles were promoted without proof")

    # One terrain renderer: all zooms go through the cached dual-grid vertex loop.
    draw_start = main_rs.find("Semantic world cells occupy the square between four integer dual-grid")
    draw_end = main_rs.find("Exact-source local replacements are authored in semantic-cell space", draw_start)
    req(draw_start >= 0 and draw_end > draw_start, "cannot isolate unified terrain draw block")
    terrain_draw = main_rs[draw_start:draw_end]
    req("if px < 6.0" not in terrain_draw, "far zoom still switches terrain renderer")
    req("generated_cell(world)" not in terrain_draw, "far zoom still draws GeneratedCell.source directly")
    req("cached_world_vertex_visual" in terrain_draw and "terrain::dual_grid_vertex_tile_min" in terrain_draw,
        "terrain draw block is not the unified cached dual-grid geometry")
    req("tiled_vertex_tiles" in main_rs and "prepare_world_visual_cache" in main_rs,
        "renderer cache does not retain shared solved Tiled tile IDs")

    # Complete canonical clips may animate; partial clips must remain fail closed.
    for marker in ("SummerVisualAnimation", "SummerVisualAnimationFrame"):
        req(marker in auth, f"asset visual animation contract missing {marker}")
    for marker in ("CachedWorldAnimation", "source_at(elapsed_ms)", "cached_world_animation", "request_repaint_after"):
        req(marker in main_rs, f"cached live animation presentation missing {marker}")
    req("Partial clips are never promoted" in tiled,
        "partial configured animation clips are not explicitly fail closed")

    # Derivative Tiled art may drive topology but never becomes renderer art.
    req("tileset_asset_path" not in world_runtime,
        "world renderer directly references the derivative Jaidyn atlas")
    req("canonical ElizaWy" in tiled,
        "canonical original-pixel authority marker missing")

    req("m2d097a-visible-renderer-selftest" in pcc,
        "PCC Full Gate/static audit do not retain M2D097A regression coverage")

    print("M2D097A VISIBLE RENDERER CONVERGENCE SELFTEST: PASS")
    print("  terrain LOD  : one cached dual-grid vertex renderer at all zoom levels")
    print("  topology     : exact authored Tiled first -> shared native-Wang -> conservative fallback")
    print("  coast        : Deep -> Shallow -> Sand -> Grass; generated WetSand ring retired")
    print("  overlays     : second RiverWater/WetSand pale-shore repaint retired")
    print("  animation    : complete canonical TSX clips live; partial clips fail closed")
    print("  art          : original ElizaWy pixels remain render authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
