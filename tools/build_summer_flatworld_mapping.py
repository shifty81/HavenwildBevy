#!/usr/bin/env python3
"""Build Havenwild's flat Summer dual-grid mapping from the hydrated ElizaWy sheet.

The tool never changes source artwork.  It reads the exact, hash-verified
Terrain/terrain_summer.png, classifies only canonical 32x32 source regions whose
recovered family hints are Grass/Dirt/Water transition families, and writes local
mapping metadata under .forgepy/terrain/.

Every emitted recipe points back to an existing canonical region in
content/assets/authority/elizawy_runtime_index.v1.json.  Missing/ambiguous states
remain unmapped and fail closed in the editor instead of borrowing cliff artwork.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import sys

from inspect_elizawy_source_alpha import rgba_rows

ROOT = Path(__file__).resolve().parents[1]
ATLAS = "Terrain/terrain_summer.png"
OUT = ROOT / ".forgepy" / "terrain" / "summer_flatworld_runtime.local.json"
OBJECT_OUT = ROOT / ".forgepy" / "terrain" / "worldgen_object_bounds.local.json"
REPORT = ROOT / ".forgepy" / "terrain" / "summer_flatworld_mapping_report.json"
PROFILE = ROOT / "content" / "terrain" / "recipes" / "summer_flatworld_runtime.v1.json"
INDEX = ROOT / "content" / "assets" / "authority" / "elizawy_runtime_index.v1.json"
MANIFEST = ROOT / "content" / "catalog" / "core_source_manifest.json"
ASSET = ROOT / "assets" / "elizawy" / ATLAS
ROLES = ("Grass", "MudBank", "RiverWater")
PAIR_HINT = {
    frozenset(("Grass", "MudBank")): "grass_dirt",
    frozenset(("Grass", "RiverWater")): "grass_shallows",
    frozenset(("MudBank", "RiverWater")): "dirt_shallows",
}


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def canonical_entry(rect, regions):
    for region in regions:
        if region.get("sourcePath") == ATLAS and region.get("sourceRectPx") == list(rect):
            return region
    return None


def patch_stats(rows, x0: int, y0: int, size: int = 7):
    pixels = []
    for y in range(y0, y0 + size):
        for x in range(x0, x0 + size):
            r, g, b, a = rows[y][x]
            if a >= 224:
                pixels.append((r, g, b))
    coverage = len(pixels) / float(size * size)
    if not pixels:
        return coverage, None
    return coverage, tuple(sum(p[i] for p in pixels) / len(pixels) for i in range(3))


def tile_mean(rows, rect):
    x, y, w, h = rect
    values = []
    for row in rows[y:y+h]:
        for r, g, b, a in row[x:x+w]:
            if a >= 224:
                values.append((r, g, b))
    if not values:
        raise ValueError(f"Prototype has no opaque pixels: {rect}")
    return tuple(sum(p[i] for p in values) / len(values) for i in range(3))


def distance(a, b):
    return math.sqrt(sum((a[i] - b[i]) ** 2 for i in range(3)))


def classify(mean, prototypes):
    ranked = sorted((distance(mean, rgb), role) for role, rgb in prototypes.items())
    best_distance, best_role = ranked[0]
    second_distance = ranked[1][0]
    separation = second_distance - best_distance
    return best_role, best_distance, separation


def classify_cell(rows, rect, prototypes):
    x, y, w, h = rect
    if (w, h) != (32, 32):
        return None
    # Samples stay inside the source cell so neighbouring atlas pixels can never
    # contaminate classification.  Seven pixels gives enough material signal while
    # remaining clear of most antialiased transition boundaries.
    samples = [
        (x + 1, y + 1),
        (x + 24, y + 1),
        (x + 1, y + 24),
        (x + 24, y + 24),
    ]
    roles = []
    confidence = []
    for sx, sy in samples:
        coverage, mean = patch_stats(rows, sx, sy)
        if mean is None or coverage < 0.72:
            return None
        role, best, separation = classify(mean, prototypes)
        # Broad enough for palette variation, strict enough to reject alpha/object
        # cells and unrelated sand/hole artwork.  Pair-family hints provide a second
        # independent guard below.
        if best > 105.0 or separation < 8.0:
            return None
        roles.append(role)
        confidence.append((coverage * 100.0) + separation - best * 0.15)
    return tuple(roles), min(confidence)



def verified_png(asset: str, manifest_entries):
    meta = next((entry for entry in manifest_entries if entry.get("path") == asset), None)
    if meta is None:
        raise ValueError(f"Canonical manifest has no {asset}")
    path = ROOT / "assets" / "elizawy" / asset
    if not path.is_file():
        raise ValueError(f"Hydrated source is missing: {path.relative_to(ROOT)}; run PCC Assets / hydration first")
    payload = path.read_bytes()
    digest = hashlib.sha256(payload).hexdigest()
    if digest != meta.get("sha256") or len(payload) != meta.get("bytes"):
        raise ValueError(f"{asset} hash/size differs from pinned ElizaWy source; refusing to map it")
    width, height, rows = rgba_rows(payload)
    if [width, height] != meta.get("imageSizePx"):
        raise ValueError(f"{asset} dimensions differ from canonical manifest")
    return meta, digest, width, height, rows


def largest_connected_crop(rows, source_size, rect, *, padding=18):
    width, height = source_size
    x, y, w, h = rect
    left = max(0, x - padding)
    top = max(0, y - padding)
    right = min(width, x + w + padding)
    bottom = min(height, y + h + padding)
    points = set()
    for py in range(top, bottom):
        for px in range(left, right):
            if rows[py][px][3] > 0:
                points.add((px, py))
    components = []
    while points:
        seed = points.pop()
        stack = [seed]
        component = [seed]
        while stack:
            px, py = stack.pop()
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    if not dx and not dy:
                        continue
                    q = (px + dx, py + dy)
                    if q in points:
                        points.remove(q)
                        stack.append(q)
                        component.append(q)
        components.append(component)
    if not components:
        return None
    def score(component):
        overlap = sum(1 for px, py in component if x <= px < x + w and y <= py < y + h)
        return overlap, len(component)
    component = max(components, key=score)
    overlap, total = score(component)
    if overlap < 24 or total < 48:
        return None
    min_x = min(px for px, _ in component)
    max_x = max(px for px, _ in component)
    min_y = min(py for _, py in component)
    max_y = max(py for _, py in component)
    return [min_x, min_y, max_x - min_x + 1, max_y - min_y + 1]


def build_object_bounds(manifest_entries, runtime_index):
    decoded = {}
    output = []
    for template in runtime_index.get("objectTemplates", []):
        label = str(template.get("label", "")).lower()
        if not any(token in label for token in ("tree", "rock", "stone")):
            continue
        parts_out = []
        ok = True
        for part in template.get("parts", []):
            asset = part.get("sourcePath")
            rect = part.get("sourceRectPx")
            if asset not in ("Terrain/trees_summer.png", "Terrain/Rocks, Grasslands.png") or not isinstance(rect, list):
                ok = False
                break
            if asset not in decoded:
                _, digest, width, height, rows = verified_png(asset, manifest_entries)
                decoded[asset] = (digest, width, height, rows)
            digest, width, height, rows = decoded[asset]
            cropped = largest_connected_crop(rows, (width, height), rect)
            if cropped is None:
                ok = False
                break
            dx = cropped[0] - rect[0]
            dy = cropped[1] - rect[1]
            old_offset = part.get("offsetPx", [0, 0])
            parts_out.append({
                "sourcePath": asset,
                "sourceRectPx": cropped,
                "offsetPx": [old_offset[0] + dx, old_offset[1] + dy],
                "sourceSha256": digest,
                "originalSourceRectPx": rect,
            })
        if ok and parts_out:
            output.append({"templateId": template["templateId"], "parts": parts_out})
    return {
        "schema": "havenwild.terrain.worldgen_object_bounds.v1",
        "version": 1,
        "policy": "largest-alpha-connected-component-from-hash-verified-original-source",
        "templates": output,
    }

def build(*, dry_run: bool = False):
    manifest = load_json(MANIFEST)
    manifest_entries = manifest.get("entries", [])
    source_meta, digest, width, height, rows = verified_png(ATLAS, manifest_entries)
    if width % 32 or height % 32:
        raise ValueError("terrain_summer.png is not aligned to 32x32 source cells")

    base = load_json(PROFILE)
    index = load_json(INDEX)
    object_bounds = build_object_bounds(manifest_entries, index)
    regions = index.get("canonicalRuntimeRegions", [])
    safe = base["safeFill"]
    prototypes = {
        role: tile_mean(rows, entry["sourceRectPx"])
        for role, entry in safe.items()
    }

    eligible = []
    for region in regions:
        rect = region.get("sourceRectPx")
        if region.get("sourcePath") != ATLAS or not isinstance(rect, list) or rect[2:] != [32, 32]:
            continue
        hints = set(region.get("familyHints") or [])
        if not hints.intersection({"grass", "dirt", "grass_dirt", "grass_shallows", "dirt_shallows"}):
            continue
        eligible.append(region)

    recipes = {}
    candidates = []
    rejected = 0
    for region in eligible:
        rect = region["sourceRectPx"]
        classified = classify_cell(rows, rect, prototypes)
        if classified is None:
            rejected += 1
            continue
        corners, score = classified
        unique = frozenset(corners)
        if len(unique) == 1:
            # Homogeneous material comes from the explicit safe-fill contract.
            continue
        if len(unique) != 2 or unique not in PAIR_HINT:
            rejected += 1
            continue
        required_hint = PAIR_HINT[unique]
        hints = set(region.get("familyHints") or [])
        if required_hint not in hints:
            rejected += 1
            continue
        key = "|".join(corners)
        entry = {
            "canonicalRegionId": region["canonicalRegionId"],
            "sourcePath": ATLAS,
            "sourceRectPx": rect,
            "weight": 1,
        }
        candidates.append({"key": key, "score": round(score, 3), "rect": rect, "family": required_hint})
        previous = recipes.get(key)
        if previous is None or score > previous[0]:
            recipes[key] = (score, entry)

    # Homogeneous tuples are explicit and deterministic.
    for role in ROLES:
        recipes["|".join((role,) * 4)] = (10_000.0, safe[role])

    corner_recipes = {key: pair[1] for key, pair in sorted(recipes.items())}
    expected_binary = 45  # 3 homogeneous + 14 masks for each of three material pairs.
    pair_counts = {}
    for pair, hint in PAIR_HINT.items():
        pair_counts[hint] = sum(
            1
            for key in corner_recipes
            if frozenset(key.split("|")) == pair and len(set(key.split("|"))) == 2
        )

    output = {
        "schema": "havenwild.terrain.summer_flatworld_runtime.v1",
        "version": 2,
        "sourceAtlas": ATLAS,
        "sourceAtlasSha256": digest,
        "policy": {
            "cliffSourcesAllowed": False,
            "broadFixtureFallbackAllowed": False,
            "unmappedTransitionFallback": "safe_fill",
            "artworkAuthority": "exact_elizawy_source_region",
            "mappingAuthority": "local_source_pixel_corner_classifier",
        },
        "safeFill": safe,
        "transitionFamilies": {},
        "cornerRecipes": corner_recipes,
        "notes": [
            "Generated locally from the exact hash-verified ElizaWy terrain_summer.png; source artwork is never modified.",
            "Only canonical 32x32 regions with matching recovered Grass/Dirt/Water family hints may become recipes.",
            "Ambiguous or missing corner tuples remain unmapped and fail closed to safe fill; cliff sources are never eligible.",
        ],
    }
    report = {
        "schema": "havenwild.terrain.summer_flatworld_mapping_report.v1",
        "sourceAtlas": ATLAS,
        "sourceAtlasSha256": digest,
        "sourceCells": [width // 32, height // 32],
        "eligibleCanonicalCells": len(eligible),
        "rejectedOrAmbiguousCells": rejected,
        "cornerRecipeCount": len(corner_recipes),
        "expectedBinaryMvpRecipeCount": expected_binary,
        "pairTransitionCounts": pair_counts,
        "coveragePercent": round(100.0 * len(corner_recipes) / expected_binary, 1),
        "topCandidates": sorted(candidates, key=lambda item: item["score"], reverse=True)[:96],
        "output": str(OUT.relative_to(ROOT)),
        "objectBoundsOutput": str(OBJECT_OUT.relative_to(ROOT)),
        "certifiedObjectTemplateCount": len(object_bounds["templates"]),
    }
    if not dry_run:
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
        OBJECT_OUT.write_text(json.dumps(object_bounds, indent=2) + "\n", encoding="utf-8")
        REPORT.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def status():
    if not OUT.is_file():
        return {"status": "MISSING", "path": str(OUT.relative_to(ROOT)), "cornerRecipeCount": 0}
    data = load_json(OUT)
    object_data = load_json(OBJECT_OUT) if OBJECT_OUT.is_file() else {"templates": []}
    return {
        "status": "READY",
        "path": str(OUT.relative_to(ROOT)),
        "sourceAtlas": data.get("sourceAtlas"),
        "sourceAtlasSha256": data.get("sourceAtlasSha256"),
        "cornerRecipeCount": len(data.get("cornerRecipes") or {}),
        "certifiedObjectTemplateCount": len(object_data.get("templates") or []),
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("build", "status"), nargs="?", default="build")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)
    try:
        result = status() if args.action == "status" else build(dry_run=args.dry_run)
        prefix = "SUMMER FLATWORLD MAP"
        print(f"{prefix}: {result.get('status', 'PASS')}")
        print(json.dumps(result, indent=2))
        if args.action == "build" and not args.dry_run:
            print("Restart Studio after rebuilding mapping metadata so AssetAuthority reloads it.")
        return 0
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f"SUMMER FLATWORLD MAP: FAIL / {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
