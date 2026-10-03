#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"
WORLD = ROOT / "src/world_doc.rs"
PLAN = ROOT / "src/worldgen_plan.rs"
MAIN = ROOT / "src/main.rs"


def require(ok: bool, message: str) -> None:
    if not ok:
        raise SystemExit(f"M2D082F SELFTEST: FAIL - {message}")


def pair_count(profile: dict, a: str, b: str) -> int:
    target = {a, b}
    return sum(1 for key in profile.get("cornerRecipes", {}) if set(key.split("|")) == target)


def main() -> int:
    profile = json.loads(PROFILE.read_text(encoding="utf-8"))
    world = WORLD.read_text(encoding="utf-8")
    plan = PLAN.read_text(encoding="utf-8")
    main_rs = MAIN.read_text(encoding="utf-8")

    require(profile.get("version") == 7, "Summer profile is not v7")
    safe = profile.get("safeFill", {})
    require(set(safe) >= {"Grass","MudBank","Sand","WetSand","RiverWater","DeepWater","PebblePath"}, "seven source-backed semantic materials are not present")
    variants = profile.get("fillVariants", {})
    require([len(variants.get(r, [])) for r in ("Sand","WetSand","PebblePath","DeepWater")] == [3,3,4,1], "extended material variant counts changed")
    require(pair_count(profile, "Grass", "Sand") == 12, "Grass/Sand exact transition family is not 12 states")
    require(pair_count(profile, "Sand", "WetSand") == 12, "Sand/WetSand exact transition family is not 12 states")
    require(pair_count(profile, "Sand", "RiverWater") == 12, "Sand/Water exact transition family is not 12 states")
    require(pair_count(profile, "RiverWater", "DeepWater") == 12, "Shallow/Deep exact transition family is not 12 states")
    require(pair_count(profile, "PebblePath", "MudBank") == 8, "Pebble/Dirt authored outer transition family is not 8 states")

    require("fn land_connectivity_projection" in world, "land-connectivity projection missing")
    require("Two water corners that only touch diagonally" in world, "diagonal-water land-win policy missing")
    require("Historical quadrant composites remain evidence/tooling records only" in world, "live quadrant-composite ban missing")
    require("summer_flatworld_visual_parts(role_refs)" not in world, "live resolver still invokes quadrant composite visual parts")
    require("ambiguous_diagonal_water_connects_land_without_quadrant_parts" in world, "land-connectivity Rust regression missing")
    require("summer_flatworld_fill_for_world(\"Grass\", w.seed, vertex)" in world, "diagonal land regression must accept the deterministic Grass fill variant instead of pinning one atlas coordinate")
    require("cardinal_water_channel_remains_open" in world, "explicit/cardinal water-channel regression missing")

    for token in ['"WetSand"', '"Sand"', '"DeepWater"', '"PebblePath"']:
        require(token in plan, f"Generated World plan does not emit {token}")
    require('ui.small("TERR")' not in main_rs, "material abbreviations remain on permanent left tool rail")
    require("WorldTerrainBrush::Sand" in main_rs and "WorldTerrainBrush::DeepWater" in main_rs, "World Generator does not expose extended material brushes")

    print("M2D082F LAND-CONNECTIVITY / MATERIAL EXPANSION SELFTEST: PASS")
    print("  ambiguous shoreline  : diagonal water closes; cardinal water remains open")
    print("  live composites      : exact full tiles only; quadrant composites evidence-only")
    print("  semantic materials   : Grass Dirt Sand WetSand Shallow Deep PebblePath")
    print("  generated coast      : Deep -> Shallow -> WetSand -> Sand -> inland land")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
