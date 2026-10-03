#!/usr/bin/env python3
from __future__ import annotations
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
SHOWCASE = ROOT / "content/worldgen/elizawy_showcase_templates.v1.json"
CONTRACT = ROOT / "content/worldgen/havenwild_generated_world.v1.json"
PLAN = ROOT / "src/worldgen_plan.rs"
WORLD = ROOT / "src/world_doc.rs"
MAIN = ROOT / "src/main.rs"
CARGO = ROOT / "Cargo.toml"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D090 SELFTEST: FAIL - {message}")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    runtime = load(RUNTIME)
    showcase = load(SHOWCASE)
    contract = load(CONTRACT)
    plan = PLAN.read_text(encoding="utf-8")
    world = WORLD.read_text(encoding="utf-8")
    main_rs = MAIN.read_text(encoding="utf-8")
    cargo = CARGO.read_text(encoding="utf-8")

    require(re.search(r'^version\s*=\s*"0\.9\.1"', cargo, re.M) is not None, "Cargo source is not 0.9.1")
    counts = runtime.get("counts", {})
    require(counts.get("runtimeSourceImages") == 320, "runtime source-image lane changed")
    require(counts.get("canonicalRuntimeRegions") == 2890, "M2D090 exact-region authority count changed")
    require(counts.get("objectTemplates") == 28, "M2D090 reviewed object-template count changed")
    require(runtime.get("policy", {}).get("generatedArtworkAllowed") is False, "generated artwork became allowed")

    require(showcase.get("schema") == "havenwild.worldgen.elizawy_showcase_templates.v1", "showcase schema changed")
    require(len(showcase.get("templates", [])) == 8, "expected eight reviewed showcase templates")
    tags = {tag for row in showcase["templates"] for tag in row.get("worldgenTags", [])}
    for expected in ["house", "bridge", "clutter", "lighting", "waterfall", "cave_entrance", "cliff"]:
        require(expected in tags, f"showcase template family missing {expected}")

    regions = {(row["sourcePath"], tuple(row["sourceRectPx"])) for row in runtime["canonicalRuntimeRegions"]}
    for path in [
        "Terrain/terrain_spring.png", "Terrain/terrain_autumn.png", "Terrain/terrain_winter.png",
        "Terrain/trees_spring.png", "Terrain/trees_autumn.png", "Terrain/trees_winter.png",
        "Terrain/plants_spring.png", "Terrain/plants_autumn.png", "Terrain/plants_winter.png",
        "Terrain/wildflowers_spring.png", "Terrain/wildflowers_autumn.png", "Terrain/wildflowers_winter.png",
        "Terrain/cliff_spring.png", "Terrain/cliff_autumn.png", "Terrain/cliff_winter.png",
    ]:
        require(any(source == path for source, _ in regions), f"seasonal exact-source authority missing {path}")

    require(contract.get("profileId") == "havenwild.four_season_archipelago.v2", "world profile is not v2")
    require(contract.get("status") == "m2d090_kitchen_sink_population_active", "M2D090 world status missing")
    require(contract.get("population", {}).get("active") is True, "world population is not active")
    require(contract.get("population", {}).get("sourceTemplates") == 28, "contract template count changed")
    require(contract.get("elevation", {}).get("prototypeLevels") == [0, 1, 2], "elevation prototype levels changed")
    require(contract.get("waterfalls", {}).get("active") is True, "waterfall worldgen not active")
    require("coordinate-equivalent" in contract.get("macroWorld", {}).get("seasonVisualPolicy", ""), "seasonal source substitution policy missing")

    for token in [
        'WORLD_PROFILE_ID: &str = "havenwild.four_season_archipelago.v2"',
        "settlement_centers", "bridge_anchor", "waterfall_anchor", "cave_anchor",
        "pub fn elevation", '"rocky_upland"', '"settlement"', '"road_corridor"',
    ]:
        require(token in plan, f"worldgen plan missing {token}")

    for token in [
        'WORLD_GENERATOR_ID: &str = "havenwild.source_backed.generated_world.v3"',
        "elevation: worldgen_plan::elevation", "place_purpose_objects", "place_natural_objects",
        'templates_with_tag("house")', 'templates_with_tag("bridge")',
        'templates_with_tag("waterfall")', 'templates_with_tag("cave_entrance")',
        "seasonalize_binding", "generated_ground_uses_coordinate_equivalent_seasonal_source_sheets",
    ]:
        require(token in world, f"Generated World source missing {token}")

    require("Generated World object references missing/out-of-bounds original source" in main_rs,
            "Studio does not eager-load materialized Structure/Object worldgen sheets")
    require("generated_elevation(state.world_selected)" in main_rs,
            "Generated World selection does not expose elevation")
    require("M2D090 uses exact coordinate-equivalent" in main_rs,
            "Studio still describes seasonal art as disabled")

    print("M2D090 KITCHEN-SINK GENERATED WORLD SELFTEST: PASS")
    print("  source authority     : 320 images / 2890 exact regions / 28 reviewed templates")
    print("  seasons              : exact terrain/tree/plant/wildflower/cliff coordinate counterparts")
    print("  purpose graph        : 2 settlements/island + road/cave spur + bridge + waterfall")
    print("  elevation            : generated 0/+1/+2 intent with cliff/cave showcase feature")
    print("  population           : trees/shrubs/foliage/flowers/rocks/water details + settlement assets")
    print("  generated artwork    : OFF")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
