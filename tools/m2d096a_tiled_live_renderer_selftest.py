#!/usr/bin/env python3
"""M2D096A: certify the in-project Tiled -> canonical ElizaWy renderer cutover."""
from __future__ import annotations

import json
import re
from pathlib import Path
from source_version_contract import project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]


def req(ok: bool, msg: str) -> None:
    if not ok:
        raise SystemExit(f"M2D096A SELFTEST: FAIL - {msg}")


def main() -> int:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    lock = (ROOT / "Cargo.lock").read_text(encoding="utf-8")
    project = json.loads((ROOT / "project/forgepy.project.json").read_text(encoding="utf-8"))
    tiled = (ROOT / "src/tiled_terrain.rs").read_text(encoding="utf-8")
    world = (ROOT / "src/world_doc.rs").read_text(encoding="utf-8")
    main_rs = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    pcc = (ROOT / "ProjectControlCenter.py").read_text(encoding="utf-8")

    req(source_at_least(cargo, "0.10.16"), "Cargo source predates live Tiled renderer cutover")
    req(project_matches_cargo(project, cargo), "project sourceVersion does not match Cargo")
    req('image = { version = "=0.25.10", default-features = false, features = ["png"] }' in cargo,
        "canonical pixel-concordance PNG reader dependency missing")
    req('"image",' in lock, "Cargo.lock root package does not include image dependency")
    req('bevy_map_autotile' not in cargo and 'bevy_map_autotile' not in lock,
        "external Bevy 0.18 autotile dependency returned")

    for token in [
        "pub struct CanonicalTiledTerrainAuthority",
        "Jaidyn owns Wang topology/animation",
        "canonical ElizaWy Terrain catalog",
        "canonical_by_tile",
        '"Grass" => Some("Grass")',
        '"MudBank" => Some("Dirt")',
        '"Sand" => Some("Sand")',
        '"RiverWater" => Some("Shallow Water")',
        '"DeepWater" => Some("Deep Water")',
        '"WetSand" | "PebblePath" => None',
        "animation_frames",
        "resolve_havenwild_corner",
        "solve_havenwild_corner_grid",
        "solve_corner_material_grid_with_origin",
    ]:
        req(token in tiled, f"Tiled canonical bridge missing {token!r}")

    generator_match = re.search(r'generated_world\.v(\d+)', world)
    req(generator_match is not None and int(generator_match.group(1)) >= 11,
        "world cache predates the live Tiled source-binding generation")
    req("resolved_dual_grid_visual_parts_with_authorities" in world,
        "near renderer has no Tiled-aware resolver entry point")
    req("resolved_tiled_materialized_vertex_tiles" in world and "solve_havenwild_corner_grid" in world,
        "materialized world does not use the shared native-Wang/Tiled solve")
    req("resolve_havenwild_corner_tile_id(role_refs, self.seed, vertex)" in world,
        "per-vertex exact Tiled fallback is missing")
    req("baseline_source_for" in world and "tiled: Option<&CanonicalTiledTerrainAuthority>" in world,
        "generated/far source binding path does not accept Tiled authority")
    req("materialize_with_tiled" in world and "preview_regenerate_with_tiled" in world,
        "worldgen does not share the Tiled-aware materialization path")

    req("tiled_terrain: Option<CanonicalTiledTerrainAuthority>" in main_rs,
        "Studio does not own the live Tiled authority")
    req("CanonicalTiledTerrainAuthority::load_summer(&root, &source_catalog)" in main_rs,
        "Studio startup does not build canonical Tiled concordance")
    req("resolved_dual_grid_visual_parts_with_authorities" in main_rs,
        "world canvas fallback is not routed through Tiled authority")
    req("tiled_vertex_tiles" in main_rs and "resolved_tiled_visual_part_for_tile_id" in main_rs,
        "world canvas cache does not consume the shared solved Tiled tile grid")
    req("materialize_with_tiled" in main_rs and "preview_regenerate_with_tiled" in main_rs,
        "World Generator is not routed through Tiled authority")
    req("resolved_source_with_authorities" in main_rs,
        "Paint/sample/source inspection is not routed through the same Tiled authority")

    req("Jaidyn configured Tiled terrain availability" in pcc,
        "Full Gate does not verify configured Tiled terrain before runtime certification")
    req("Jaidyn configured Tiled terrain hydration" in pcc,
        "Full Gate cannot hydrate the pinned configured Tiled terrain on a source-only checkout")
    req("m2d096a-tiled-live-renderer-selftest" in pcc,
        "Full Gate does not retain M2D096A regression coverage")

    # The derivative atlas may be read for topology/pixel concordance, but the live
    # visual result must be a canonical source binding, never the derivative path.
    req("source_asset: binding.source_asset" in world and "source_rect: binding.source_rect" in world,
        "renderer does not consume canonical source binding returned by concordance")
    req("tileset_asset_path" not in world,
        "world renderer directly references the configured derivative atlas")

    print("M2D096A TILED LIVE RENDERER SELFTEST: PASS")
    print("  topology      : Jaidyn Summer Terrain Wang set")
    print("  art authority : exact canonical ElizaWy RGBA source regions only")
    print("  live surfaces : one shared near/far dual-grid renderer + generated bindings + Paint/sample")
    print("  core mapping  : Grass/Dirt/Sand/Shallow Water/Deep Water")
    print("  fail closed   : WetSand + PebblePath retain legacy source-backed resolver pending proof")
    print("  cache         : Generated World v11 invalidates pre-cutover source bindings")
    print("  animation     : complete canonical TSX clips retained for live cached presentation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
