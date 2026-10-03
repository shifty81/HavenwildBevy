#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN = ROOT / "src/main.rs"
WORLD = ROOT / "src/world_doc.rs"
TERRAIN = ROOT / "src/terrain.rs"
CONTRACT = ROOT / "content/worldgen/havenwild_generated_world.v1.json"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D082D ALIGNMENT SELFTEST: FAIL - {message}")


def compact(text: str) -> str:
    return "".join(text.split())


def main() -> int:
    main_rs = MAIN.read_text(encoding="utf-8")
    world = WORLD.read_text(encoding="utf-8")
    terrain = TERRAIN.read_text(encoding="utf-8")
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))

    cmain = compact(main_rs)
    cworld = compact(world)
    cterrain = compact(terrain)

    alignment = contract.get("coordinateAuthority", {})
    require(alignment.get("semanticCell") == "integer_aligned_unit_square", "semantic-cell coordinate contract missing")
    require(alignment.get("dualGridOutput") == "tile_centered_on_integer_vertex", "dual-grid vertex centering contract missing")
    require(alignment.get("directTile") == "semantic_cell_exact_32x32", "direct tile coordinate contract missing")

    require("pubfndual_grid_vertex_tile_min(local_vertex:[usize;2])->[f32;2]" in cterrain,
            "shared vertex-centering helper missing")
    require("local_vertex[0]asf32-0.5" in cterrain and "local_vertex[1]asf32-0.5" in cterrain,
            "vertex output is not half-cell centered")
    require("resolved_dual_grid_visual_parts_with_authority" in world,
            "WorldDocument lacks render-specific dual-grid resolver")
    require("resolved_role_for_dual_grid" in world and "role_for(self.seed,world)" in cworld,
            "outer dual-grid vertices cannot resolve deterministic virtual neighbours")

    require("resolved_dual_grid_visual_parts_with_authority(vertex,&state.asset_authority)" in cmain,
            "Generated World renderer still uses semantic-cell resolver as vertex output")
    require("dual_grid_vertex_tile_min([vx,vy])" in cmain,
            "renderer is not consuming shared half-cell centering")
    require(".min(size[0]+1)" in cmain and ".min(size[1]+1)" in cmain,
            "renderer does not include the extra east/south dual-grid vertex")
    require("has_pending_semantic_edit(world)" in cmain,
            "direct-tile overlay does not respect in-progress semantic paint")
    require("VisualOverrideMode::Replace" in main_rs,
            "direct exact-source replacement is not explicitly separated from dual-grid output")
    require("region.min+egui::vec2(lxasf32*px,lyasf32*px)" in cmain,
            "direct exact-source placement is not semantic-cell aligned")

    print("M2D082D DUAL-GRID ALIGNMENT SELFTEST: PASS")
    print("  semantic cell        : integer-aligned authored square")
    print("  dual-grid output     : centered on integer vertex (-0.5 cell render origin)")
    print("  visible output range : N+1 by M+1 vertices for N by M semantic cells")
    print("  direct source tile   : exact selected semantic cell, no half-cell shift")
    print("  outer boundary       : deterministic virtual generated roles keep seams closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
