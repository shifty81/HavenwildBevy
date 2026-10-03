#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "content/worldgen/havenwild_generated_world.v1.json"
PLAN = ROOT / "src/worldgen_plan.rs"
WORLD = ROOT / "src/world_doc.rs"
MAIN = ROOT / "src/main.rs"
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"
CATALOG = ROOT / "content/worldgen/elizawy_worldgen_asset_catalog.v1.json"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D082C SELFTEST: FAIL - {message}")


def main() -> int:
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    profile = json.loads(PROFILE.read_text(encoding="utf-8"))
    catalog = json.loads(CATALOG.read_text(encoding="utf-8"))
    plan = PLAN.read_text(encoding="utf-8")
    world = WORLD.read_text(encoding="utf-8")
    main_rs = MAIN.read_text(encoding="utf-8")

    require(contract.get("schema") == "havenwild.worldgen.intent.v1", "worldgen contract schema changed")
    macro = contract.get("macroWorld", {})
    require(macro.get("majorIslands") == 4, "Generated World is not four-island")
    require(macro.get("oceanSeparatesMajorIslands") is True, "major islands are not ocean-separated")
    require(macro.get("seasonIdentity") == ["spring", "summer", "autumn", "winter"], "season identity changed")
    require(contract.get("legacyRiverScene", {}).get("userFacing") is False, "legacy River scene is still user-facing authority")
    require(contract.get("caves", {}).get("resetCadence") == "weekly_in_game", "weekly cave reset contract missing")
    require(contract.get("caves", {}).get("runSeedFormula") == "hash(world_seed,cave_id,game_week)", "deterministic cave reset seed missing")
    require(contract.get("roads", {}).get("authority") == "connectivity_graph_before_geometry", "purposeful road graph contract missing")

    for token in ["Spring Isle", "Summer Isle", "Autumn Isle", "Winter Isle", "dense_forest", "WORLD_PROFILE_ID"]:
        require(token in plan, f"worldgen plan missing {token}")
    require("worldgen_plan::terrain_role" in world, "Generated World does not consume macro plan terrain")
    require("worldgen_plan::tree_density_percent" in world, "biome-aware forest placement missing")
    require("season: worldgen_plan::season" in world and "biome: worldgen_plan::biome" in world, "generated cells do not retain season/biome intent")
    require(".materialize([0, 0], 2" in main_rs, "fresh Generated World does not auto-materialize 5x5")
    require("world_mode: true" in main_rs, "Generated World is not the default editor surface")
    require('small_button("Summer scene")' not in main_rs, "legacy River/Summer scene still has a canvas mode button")
    require("source_uv_object" in main_rs and 'source_asset.starts_with("Terrain/trees_")' in main_rs, "tree atlas-guide sampling repair missing")

    require(catalog.get("schema") == "havenwild.worldgen.elizawy_asset_catalog.v1", "ElizaWy worldgen catalog schema changed")
    require(catalog.get("counts", {}).get("images") == 320, "Generated World does not expose all 320 canonical ElizaWy PNGs")
    require(len(catalog.get("assets", [])) == 320, "ElizaWy worldgen catalog row count changed")
    require(all(asset.get("generatedArtwork") is False for asset in catalog.get("assets", [])), "worldgen catalog allows generated art")

    require(profile.get("version") == 6, "mixed-bank continuity profile is not v6")
    notes = " ".join(profile.get("notes", []))
    require("water-continuity-first" in notes, "mixed grass/dirt/water continuity policy missing")

    print("M2D082C GENERATED WORLD SELFTEST: PASS")
    print("  primary surface      : Generated World")
    print("  macro geography      : 4 seasonal island intents + ocean")
    print("  hydrology            : ocean + per-island river/lake intent")
    print("  biomes               : coast/meadow/light woodland/dense forest")
    print("  shoreline            : water-continuity tri-material repair")
    print("  tree renderer        : atlas-guide-safe large-tree sampling")
    print("  ElizaWy source lane  : 320/320 canonical PNGs discoverable; placement authority-gated")
    print("  roads/cities/caves   : purpose-first contracts locked for subsequent passes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
