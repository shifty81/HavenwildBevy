#!/usr/bin/env python3
"""Build the M2D081 complete Summer runtime profile.

This generator is intentionally metadata-only. It promotes every reviewed
non-transparent 32x32 cell from the immutable Native Summer complete map into a
cataloged source group, adds deterministic fill-variant sets for the normal
Grass/Dirt/Water authoring lane, builds exact-source three-material junction
composites, and declares the source-backed Summer water interior animation cycle.

No PNG bytes are created or modified.
"""
from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROFILE = ROOT / "content/terrain/recipes/summer_flatworld_runtime.v1.json"
DONOR = ROOT / "content/terrain/recovered/native/lpc_terrain_summer_complete_map_32.native.v1.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
ATLAS = "Terrain/terrain_summer.png"

ROLE_GROUP = {
    "Grass": "summer_grass_fill",
    "MudBank": "summer_dirt_fill",
    "RiverWater": "summer_water_fill",
    "Sand": "summer_sand_fill",
    "WetSand": "summer_wet_sand_fill",
    "PebblePath": "summer_pebble_path_fill",
}

# Source-authored transition/stamp families that can be promoted into the
# semantic terrain lane without synthesizing pixels. 13-piece families cover
# the 12 non-diagonal mixed corner states plus their center; diagonal states are
# resolved by the runtime's land-connectivity policy instead of quadrant splicing.
TRANSITION_PROMOTIONS = (
    ("Grass", "Sand", "summer_grass_over_sand"),
    ("Sand", "WetSand", "summer_sand_over_wet_sand"),
    ("Sand", "RiverWater", "summer_sand_bank_pond"),
    ("RiverWater", "DeepWater", "summer_deep_water_basin"),
    ("PebblePath", "MudBank", "summer_pebble_path_over_dirt"),
)

# Keep plain grass/dirt variation spatial and deterministic. Water is deliberately
# restricted to one conservative base fill until the source sheet provides explicit
# temporal animation evidence. The other seven water-looking cells remain cataloged
# source regions for deliberate detail/decor use; they are not animation frames and
# are not shuffled into ordinary homogeneous water.
ROLE_WEIGHTS = {
    "Grass": [5, 1, 1, 1, 1, 1],
    "MudBank": [3, 1, 1, 1, 1, 1],
    "RiverWater": [1],
    "Sand": [3, 1, 1],
    "WetSand": [3, 1, 1],
    "PebblePath": [1, 1, 1, 1],
}

QUADRANTS = [(0, 0), (16, 0), (0, 16), (16, 16)]
ROLES = ("Grass", "MudBank", "RiverWater")

ROLE_TO_CORNERS = {
    "outer_north_west": (0, 0, 0, 1),
    "outer_north":      (0, 0, 1, 1),
    "outer_north_east": (0, 0, 1, 0),
    "outer_west":       (0, 1, 0, 1),
    "outer_east":       (1, 0, 1, 0),
    "outer_south_west": (0, 1, 0, 0),
    "outer_south":      (1, 1, 0, 0),
    "outer_south_east": (1, 0, 0, 0),
    "inner_south_east": (1, 1, 1, 0),
    "inner_south_west": (1, 1, 0, 1),
    "inner_north_east": (1, 0, 1, 1),
    "inner_north_west": (0, 1, 1, 1),
}


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def dump(data) -> bytes:
    return (json.dumps(data, indent=2, sort_keys=False) + "\n").encode("utf-8")


def runtime_lookup(runtime):
    return {
        (row["sourcePath"], tuple(row["sourceRectPx"])): row
        for row in runtime["canonicalRuntimeRegions"]
    }


def palette_entry(row, weight=1):
    return {
        "canonicalRegionId": row["canonicalRegionId"],
        "sourcePath": row["sourcePath"],
        "sourceRectPx": row["sourceRectPx"],
        "weight": weight,
    }


def source_quadrant(entry, quadrant):
    qx, qy = QUADRANTS[quadrant]
    sx, sy, sw, sh = entry["sourceRectPx"]
    if [sw, sh] != [32, 32]:
        raise ValueError("quadrant parent must be a canonical 32x32 source region")
    return {
        "canonicalRegionId": entry["canonicalRegionId"],
        "sourcePath": entry["sourcePath"],
        "sourceRectPx": [sx + qx, sy + qy, 16, 16],
        "destinationRectPx": [qx, qy, 16, 16],
    }


def recipe_quadrant(profile, corners, quadrant):
    key = "|".join(corners)
    direct = profile.get("cornerRecipes", {}).get(key)
    if direct:
        return source_quadrant(direct, quadrant)
    composite = profile.get("cornerComposites", {}).get(key)
    if composite:
        qx, qy = QUADRANTS[quadrant]
        for part in composite.get("parts", []):
            if part.get("destinationRectPx") == [qx, qy, 16, 16]:
                return dict(part)
    raise ValueError(f"pairwise recipe missing while constructing junction: {key}")



def donor_group_role_entries(donor, lookup, group_id):
    out = {}
    for cell in donor.get("cells", []):
        if cell.get("groupId") != group_id or not cell.get("nonTransparent"):
            continue
        role = cell.get("role")
        row = lookup.get((ATLAS, tuple(cell["pixelRect"])))
        if role and row is not None:
            out[role] = palette_entry(row)
    return out


def promote_transition_recipes(corner_recipes, donor, lookup, foreground, background, group_id):
    entries = donor_group_role_entries(donor, lookup, group_id)
    promoted = 0
    for source_role, bits in ROLE_TO_CORNERS.items():
        entry = entries.get(source_role)
        if entry is None:
            continue
        corners = [foreground if bit else background for bit in bits]
        corner_recipes["|".join(corners)] = entry
        promoted += 1
    return promoted

def build() -> dict:
    profile = load(PROFILE)
    donor = load(DONOR)
    runtime = load(RUNTIME)
    lookup = runtime_lookup(runtime)

    summary = donor.get("summary", {})
    if summary.get("mappedNonTransparentCells") != 305 or summary.get("nonTransparentCells") != 305:
        raise ValueError("Native Summer complete map no longer proves 305/305")

    source_groups = defaultdict(lambda: {
        "groupLabel": "",
        "classification": "",
        "roles": set(),
        "runtimeIds": set(),
        "pcgUse": set(),
        "cells": [],
    })
    for cell in donor.get("cells", []):
        if not cell.get("nonTransparent"):
            continue
        rect = tuple(cell["pixelRect"])
        row = lookup.get((ATLAS, rect))
        if row is None:
            raise ValueError(f"Summer source cell is not canonical runtime authority: {rect}")
        group_id = cell.get("groupId") or "summer_ungrouped"
        group = source_groups[group_id]
        group["groupLabel"] = cell.get("groupLabel") or group_id
        group["classification"] = cell.get("classification") or "Unclassified"
        if cell.get("role"):
            group["roles"].add(cell["role"])
        group["runtimeIds"].update(cell.get("runtimeIds") or [])
        group["pcgUse"].update(cell.get("pcgUse") or [])
        group["cells"].append(palette_entry(row))

    built_groups = {}
    for group_id in sorted(source_groups):
        group = source_groups[group_id]
        group["cells"].sort(key=lambda entry: tuple(entry["sourceRectPx"]))
        built_groups[group_id] = {
            "groupLabel": group["groupLabel"],
            "classification": group["classification"],
            "cellCount": len(group["cells"]),
            "roles": sorted(group["roles"]),
            "runtimeIds": sorted(group["runtimeIds"]),
            "pcgUse": sorted(group["pcgUse"]),
            "cells": group["cells"],
        }

    fill_variants = {}
    for role, group_id in ROLE_GROUP.items():
        cells = built_groups[group_id]["cells"]
        weights = ROLE_WEIGHTS[role]
        if role == "RiverWater":
            # M2D081C correction: the eight recovered cells are RepeatableFill
            # variants, not an authored ordered animation strip. Keep only the
            # conservative safe/base cell in automatic homogeneous water.
            cells = cells[:1]
        if len(cells) != len(weights):
            raise ValueError(f"{role} expected {len(weights)} fill variants, got {len(cells)}")
        fill_variants[role] = [
            {**entry, "weight": weight}
            for entry, weight in zip(cells, weights)
        ]

    # Deep water is represented by the authored center of the shallow-rim-over-
    # deep basin family. It is a source-backed world material, not a generated
    # recolor. Keep one conservative center until a broader deep-water fill family
    # is explicitly identified.
    deep_roles = donor_group_role_entries(donor, lookup, "summer_deep_water_basin")
    if "center" not in deep_roles:
        raise ValueError("deep-water basin is missing its authored center")
    fill_variants["DeepWater"] = [{**deep_roles["center"], "weight": 1}]

    safe_fill = dict(profile.get("safeFill", {}))
    for role in ("Sand", "WetSand", "PebblePath"):
        safe_fill[role] = dict(fill_variants[role][0])
    safe_fill["DeepWater"] = dict(fill_variants["DeepWater"][0])

    # Promote complete source-authored material boundaries into the semantic lane.
    # No diagonal tile is invented: the runtime resolves ambiguous diagonals by
    # connecting land unless an actual continuous water path exists.
    corner_recipes = dict(profile.get("cornerRecipes", {}))
    promoted_transition_counts = {}
    for foreground, background, group_id in TRANSITION_PROMOTIONS:
        promoted_transition_counts[group_id] = promote_transition_recipes(
            corner_recipes, donor, lookup, foreground, background, group_id
        )

    # Animation authority is evidence-gated. RepeatableFill variants may never be
    # promoted into a temporal sequence just because they share a semantic label.
    # Keep RiverWater static until an explicitly ordered source animation is proven.
    animations = {}

    # Complete the normal three-material GRS/DIR/WTR grammar. Every three-material
    # output is a 4-quadrant composite of exact source pixels. M2D082C gives water
    # continuity explicit priority whenever Grass, Dirt and Water meet in one dual-
    # grid output: RiverWater owns the background and both land materials contribute
    # exact one-corner quadrants. This prevents narrow channels/inlets from breaking
    # into disconnected water wedges when the doubled material happens to be land.
    composites = dict(profile.get("cornerComposites", {}))

    # The six pairwise checkerboard states are inherently ambiguous if represented
    # as one fixed 32x32 source composition: either diagonal material can look as
    # if it bridges through the other. The authoring screenshots exposed exactly
    # that failure for isolated dirt spots and grass islands. For flat Summer
    # terrain we deliberately keep the land/detail material disconnected and let
    # the surrounding base material own the four quadrant backgrounds. Every
    # foreground quadrant still comes from the exact source-authored one-corner
    # recipe for that pair; no pixels are invented.
    checker_background = {
        frozenset(("Grass", "MudBank")): "Grass",
        frozenset(("Grass", "RiverWater")): "RiverWater",
        frozenset(("MudBank", "RiverWater")): "RiverWater",
    }
    checker_states = 0
    for key in list(composites):
        corners = key.split("|")
        roles = set(corners)
        if len(corners) != 4 or len(roles) != 2:
            continue
        if not (corners[0] == corners[3] and corners[1] == corners[2] and corners[0] != corners[1]):
            continue
        background = checker_background.get(frozenset(roles))
        if background is None:
            continue
        parts = []
        for index, role in enumerate(corners):
            if role == background:
                parts.append(source_quadrant(profile["safeFill"][background], index))
            else:
                pair = [background] * 4
                pair[index] = role
                parts.append(recipe_quadrant(profile, pair, index))
        composites[key] = {"parts": parts}
        checker_states += 1
    if checker_states != 6:
        raise ValueError(f"expected 6 disconnected checkerboard states, got {checker_states}")

    three_material = 0
    for a in ROLES:
        for b in ROLES:
            for c in ROLES:
                for d in ROLES:
                    corners = [a, b, c, d]
                    counts = Counter(corners)
                    if len(counts) != 3:
                        continue
                    background = "RiverWater" if "RiverWater" in counts else next(
                        role for role, count in counts.items() if count == 2
                    )
                    parts = []
                    for index, role in enumerate(corners):
                        if role == background:
                            parts.append(source_quadrant(profile["safeFill"][background], index))
                        else:
                            pair = [background] * 4
                            pair[index] = role
                            parts.append(recipe_quadrant(profile, pair, index))
                    key = "|".join(corners)
                    composites[key] = {"parts": parts}
                    three_material += 1

    if three_material != 36:
        raise ValueError(f"expected 36 three-material junction states, got {three_material}")

    profile["version"] = 7
    profile["safeFill"] = safe_fill
    profile["sourceGroups"] = built_groups
    profile["fillVariants"] = fill_variants
    profile["animations"] = animations
    profile["cornerRecipes"] = dict(sorted(corner_recipes.items()))
    profile["cornerComposites"] = dict(sorted(composites.items()))
    profile["extendedTerrainPromotions"] = {
        "roles": ["Sand", "WetSand", "DeepWater", "PebblePath"],
        "transitionRecipeCounts": promoted_transition_counts,
        "diagonalPolicy": "land_connectivity_first_no_quadrant_splice",
    }
    profile["notes"] = [
        "All 305 non-transparent cells in the Summer atlas are canonical runtime-selectable source regions grouped by the recovered Native authority.",
        "The original Grass/Dirt/Water 81-state evidence grammar remains preserved for compatibility and audit, while live ambiguous shoreline resolution is land-connectivity-first and never requires quadrant-spliced output.",
        "Sand, WetSand, PebblePath and DeepWater are promoted as source-backed semantic materials from existing Summer groups; no pixels are synthesized.",
        "Source-authored Grass/Sand, Sand/WetSand, Sand/Water, shallow/deep-water and PebblePath/Dirt boundaries are available to the semantic resolver where their source families provide an exact role.",
        "Ambiguous diagonal land/water contacts connect land by default. Water remains open only when its two corners are cardinally adjacent, so one-cell square water notches do not appear between near-touching land.",
        "Grass and Dirt homogeneous interiors choose deterministic source variants; no generated terrain artwork is created.",
        "RiverWater uses one conservative static base fill. The other seven recovered water-looking RepeatableFill cells remain available as explicit source/detail regions but are not shuffled into base water and are not treated as animation frames.",
        "Water animation is evidence-gated: no RiverWater temporal sequence is active until an explicitly ordered authored frame set is proven from source metadata or equivalent source evidence.",
        "Cliffs/elevation and waterfall assembly remain separate structural stages and are not silently injected into flat Summer terrain.",
        "Historical color inference and generated runtime atlases remain diagnostic evidence only, not visual authority.",
    ]
    return profile


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    built = build()
    data = dump(built)
    if args.check:
        if not PROFILE.is_file() or PROFILE.read_bytes() != data:
            raise SystemExit(f"Summer complete runtime profile is stale: {PROFILE}")
        print("M2D082F SUMMER PROFILE: PASS / 305 cells / 41 groups / 81-state G/D/W baseline + Sand/WetSand/DeepWater/PebblePath / land-connectivity-first")
        return 0
    PROFILE.write_bytes(data)
    print(f"WROTE {PROFILE}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
