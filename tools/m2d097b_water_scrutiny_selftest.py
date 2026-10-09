#!/usr/bin/env python3
"""M2D097B: visible water-intent / ecology / prop-sprite regression guard."""
from __future__ import annotations

import json
from pathlib import Path
from source_version_contract import project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]


def req(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D097B SELFTEST: FAIL - {message}")


def main() -> int:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    project = json.loads((ROOT / "project/forgepy.project.json").read_text(encoding="utf-8"))
    plan = (ROOT / "src/worldgen_plan.rs").read_text(encoding="utf-8")
    world = (ROOT / "src/world_doc.rs").read_text(encoding="utf-8")
    tiled = (ROOT / "src/tiled_terrain.rs").read_text(encoding="utf-8")
    main_rs = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    pcc = (ROOT / "ProjectControlCenter.py").read_text(encoding="utf-8")
    contract = json.loads((ROOT / "content/worldgen/havenwild_generated_world.v1.json").read_text(encoding="utf-8"))
    showcase = json.loads((ROOT / "content/worldgen/elizawy_showcase_templates.v1.json").read_text(encoding="utf-8"))
    lane = json.loads((ROOT / "content/assets/authority/elizawy_asset_lane.v1.json").read_text(encoding="utf-8"))
    manifest = json.loads((ROOT / "content/catalog/core_source_manifest.json").read_text(encoding="utf-8"))

    req(source_at_least(cargo, "0.10.19"), "source predates water scrutiny pass")
    req(project_matches_cargo(project, cargo), "project sourceVersion does not match Cargo")
    req(contract.get("status") == "m2d097b_water_scrutiny_active", "world contract status drift")
    req(contract.get("generatorVersion") == 12, "generated world did not advance to v12")

    # Explicit hydrology semantics must exist independently from terrain material roles.
    for marker in (
        "pub enum WaterContext",
        "Ocean",
        "Coast",
        "River",
        "PondLake",
        "supports_lily_pads",
        "lily_pad_density_percent",
        "pub fn water_context",
    ):
        req(marker in plan, f"missing water-context marker: {marker}")
    req("Self::None | Self::Ocean | Self::Coast => 0" in plan, "ocean/coast lily density is not hard-zero")
    req("Self::PondLake => 58" in plan and "Self::River => 14" in plan,
        "lily-pad ecology is not pond-heavy / river-sparse")
    req("WaterContext::PondLake => (base_dirt_percent / 3).max(4)" in plan,
        "pond/lake banks still use full river erosion density")
    req("adjacent_interior_water_context" in plan, "bank erosion cannot distinguish rivers from ponds/lakes")
    req("water_context.lily_pad_density_percent()" in world, "water-detail placement ignores water context")
    req("water_context.supports_lily_pads()" in world, "water-detail placement lacks an explicit ecology gate")
    req('water_context: worldgen_plan::water_context(seed, world).as_str().into()' in world,
        "generated cells do not persist waterContext")
    req("generated_water_context" in world and "water {water_context}" in main_rs,
        "Generated World diagnostics do not expose water context")

    # Chase-light phase gradient must stay retired and live terrain animation must be water-only.
    req("terrain_tile_is_water_boundary" in tiled, "terrain animation is not mixed-water-boundary gated")
    phase_body = tiled.split("pub fn world_animation_phase_ms", 1)[1].split("pub fn canonical_animation_binding_at", 1)[0]
    req("world[0]" not in phase_body and "world[1]" not in phase_body,
        "per-tile animation phase gradient returned")
    req("wrapping_mul(17) % period_ms" in phase_body, "shared seed-stable animation phase missing")
    for marker in (
        "COAST_TERRAIN_ANIMATION_TIME_SCALE: u32 = 4",
        "RIVER_TERRAIN_ANIMATION_TIME_SCALE: u32 = 5",
        "POND_TERRAIN_ANIMATION_TIME_SCALE: u32 = 7",
        "terrain_animation_time_scale_for_vertex",
    ):
        req(marker in world, f"missing context animation cadence marker: {marker}")

    # Original Barrel.png is a 160x64 source sheet. The reviewed first barrel must use
    # the complete 32x64 sprite and rise one tile above its one-cell ground anchor.
    barrel_manifest = next((x for x in manifest.get("files", manifest.get("assets", []))
                            if x.get("path") == "Objects/Furniture/Barrel.png"), None)
    if barrel_manifest is None:
        # current manifest stores entries in a top-level list under a different key in some checkpoints
        def find_manifest(x):
            if isinstance(x, dict):
                if x.get("path") == "Objects/Furniture/Barrel.png":
                    return x
                for v in x.values():
                    got = find_manifest(v)
                    if got: return got
            elif isinstance(x, list):
                for v in x:
                    got = find_manifest(v)
                    if got: return got
            return None
        barrel_manifest = find_manifest(manifest)
    req(barrel_manifest is not None, "Barrel.png missing from source manifest")
    req(barrel_manifest.get("imageSizePx") == [160, 64], "unexpected original Barrel.png dimensions")

    barrel = next((t for t in showcase["templates"] if t.get("key") == "barrel_a"), None)
    req(barrel is not None, "reviewed barrel template missing")
    req(barrel["parts"][0]["sourceRectPx"] == [0, 0, 32, 64], "barrel still crops to top 32px")
    req(barrel["parts"][0]["offsetPx"] == [0, -32], "barrel is not foot-anchored after full-height repair")
    lane_barrel = next((t for t in lane["objectTemplates"] if t.get("label", "").startswith("Barrel")), None)
    req(lane_barrel is not None, "generated asset lane lost barrel template")
    req(lane_barrel["parts"][0]["sourceRectPx"] == [0, 0, 32, 64], "asset lane barrel is stale")
    req(lane_barrel["parts"][0]["offsetPx"] == [0, -32], "asset lane barrel anchor is stale")

    req("m2d097b-water-scrutiny-selftest" in pcc, "PCC does not retain M2D097B regression coverage")

    print("M2D097B WATER SCRUTINY SELFTEST: PASS")
    print("  ecology       : Ocean/Coast lily pads prohibited; PondLake common; River sparse; pond dirt banks reduced")
    print("  water context : persisted on generated cells and exposed in World Generator diagnostics")
    print("  animation     : mixed water boundaries only, synchronized clock, coast 4x / river 5x / pond 7x cadence")
    print("  barrel        : original 160x64 sheet -> reviewed full 32x64 sprite, foot anchored")
    print("  worldgen      : cache generation v12")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
