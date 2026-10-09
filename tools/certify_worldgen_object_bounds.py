#!/usr/bin/env python3
"""Certify exact render crops for promoted worldgen trees and rocks/stones.

Original ElizaWy PNG bytes are immutable. Tree atlas slots are inspected across
Spring/Summer/Autumn/Winter and receive one union-alpha crop so seasonal swaps
keep identical destination anchors. Rock/stone slots are inspected against their
single immutable source sheet. The output only changes render sampling bounds;
logical footprints/collision/depth/interaction remain separate contracts.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
AUTHORITY = ROOT / "content/assets/authority/elizawy_asset_lane.v1.json"
MANIFEST = ROOT / "content/catalog/core_source_manifest.json"
OUTPUT = ROOT / ".forgepy/terrain/worldgen_object_bounds.local.json"
SEASONS = ("spring", "summer", "autumn", "winter")
SUPPORTED_TAGS = ("tree", "rock")

sys.path.insert(0, str(ROOT / "tools"))
from inspect_elizawy_source_alpha import rgba_rows  # noqa: E402


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def verified_image(asset: str, manifest_by_path: dict[str, dict], cache: dict):
    if asset in cache:
        return cache[asset]
    if asset not in manifest_by_path:
        raise ValueError(f"source missing from immutable manifest: {asset}")
    meta = manifest_by_path[asset]
    path = ROOT / "assets/elizawy" / asset
    payload = path.read_bytes()
    if len(payload) != meta["bytes"] or hashlib.sha256(payload).hexdigest() != meta["sha256"]:
        raise ValueError(f"immutable source identity mismatch: {asset}")
    width, height, rows = rgba_rows(payload)
    if [width, height] != meta["imageSizePx"]:
        raise ValueError(f"source dimensions mismatch: {asset}")
    cache[asset] = (width, height, rows)
    return cache[asset]


def _alpha_components(rows, rect):
    """Return 4-connected alpha components local to one declared atlas slot."""
    x, y, w, h = rect
    alpha = {
        (local_x, local_y)
        for local_y in range(h)
        for local_x in range(w)
        if rows[y + local_y][x + local_x][3] != 0
    }
    components = []
    while alpha:
        seed = alpha.pop()
        stack = [seed]
        points = [seed]
        while stack:
            px, py = stack.pop()
            for neighbour in ((px - 1, py), (px + 1, py), (px, py - 1), (px, py + 1)):
                if neighbour in alpha:
                    alpha.remove(neighbour)
                    stack.append(neighbour)
                    points.append(neighbour)
        components.append(points)
    return components


def _component_stats(points, slot_size):
    w, h = slot_size
    xs = [point[0] for point in points]
    ys = [point[1] for point in points]
    left, right = min(xs), max(xs)
    top, bottom = min(ys), max(ys)
    bbox_w = right - left + 1
    bbox_h = bottom - top + 1
    touches_edge = left == 0 or top == 0 or right == w - 1 or bottom == h - 1
    long_vertical = bbox_w <= 2 and bbox_h >= max(16, h // 3)
    long_horizontal = bbox_h <= 2 and bbox_w >= max(16, w // 3)
    return {
        "bbox": [left, top, right + 1, bottom + 1],
        "pixels": len(points),
        "touchesEdge": touches_edge,
        "guideLike": touches_edge and (long_vertical or long_horizontal),
    }


def alpha_bbox(rows, rect):
    """Return artwork bounds while rejecting source-sheet guide lines at slot edges.

    ElizaWy's tree sheet contains real one-pixel atlas guide strokes on some slot
    boundaries. They are provenance pixels, not part of the tree. The previous
    plain alpha bbox promoted them into the runtime crop, which reproduced the
    white vertical line reported in Studio. Only long 1-2px edge components are
    discarded; all other immutable source pixels remain part of the crop.
    """
    _, _, w, h = rect
    components = _alpha_components(rows, rect)
    if not components:
        raise ValueError(f"atlas slot has no visible source pixels: {rect}")

    retained = []
    discarded_guide_pixels = 0
    component_evidence = []
    for points in components:
        stats = _component_stats(points, (w, h))
        component_evidence.append(stats)
        if stats["guideLike"]:
            discarded_guide_pixels += stats["pixels"]
            continue
        retained.extend(points)

    if not retained:
        raise ValueError(f"atlas slot contains only guide-like alpha: {rect}")

    xs = [point[0] for point in retained]
    ys = [point[1] for point in retained]
    bbox = [min(xs), min(ys), max(xs) + 1, max(ys) + 1]
    edge_alpha = sum(
        1
        for local_x, local_y in retained
        if local_x in (0, w - 1) or local_y in (0, h - 1)
    )
    return bbox, len(retained), edge_alpha, discarded_guide_pixels, component_evidence


def crop_from_bbox(slot, bbox, offset):
    left, top, right, bottom = bbox
    return {
        "sourceRectPx": [slot[0] + left, slot[1] + top, right - left, bottom - top],
        "offsetPx": [offset[0] + left, offset[1] + top],
    }


def tree_part(part, manifest_by_path, cache):
    if part.get("sourcePath") != "Terrain/trees_summer.png":
        raise ValueError(f"tree template uses unexpected source: {part.get('sourcePath')}")
    slot = part["sourceRectPx"]
    union = [slot[2], slot[3], -1, -1]
    evidence = []
    for season in SEASONS:
        asset = f"Terrain/trees_{season}.png"
        _, _, rows = verified_image(asset, manifest_by_path, cache)
        bbox, alpha_pixels, edge_alpha, guide_pixels, components = alpha_bbox(rows, slot)
        union[0] = min(union[0], bbox[0])
        union[1] = min(union[1], bbox[1])
        union[2] = max(union[2], bbox[2])
        union[3] = max(union[3], bbox[3])
        evidence.append({
            "season": season,
            "sourcePath": asset,
            "alphaBoundsLocal": bbox,
            "alphaPixels": alpha_pixels,
            "slotEdgeAlphaPixels": edge_alpha,
            "discardedGuidePixels": guide_pixels,
            "alphaComponents": components,
        })
    result = crop_from_bbox(slot, union, part["offsetPx"])
    result.update({
        "sourcePath": part["sourcePath"],
        "slotRectPx": slot,
        "method": "seasonal_union_alpha_bbox_strip_edge_guides",
        "evidence": evidence,
    })
    return result


def rock_part(part, manifest_by_path, cache):
    asset = part.get("sourcePath")
    if asset != "Terrain/Rocks, Grasslands.png":
        raise ValueError(f"rock/stone template uses unexpected source: {asset}")
    slot = part["sourceRectPx"]
    _, _, rows = verified_image(asset, manifest_by_path, cache)
    bbox, alpha_pixels, edge_alpha, guide_pixels, components = alpha_bbox(rows, slot)
    result = crop_from_bbox(slot, bbox, part["offsetPx"])
    result.update({
        "sourcePath": asset,
        "slotRectPx": slot,
        "method": "single_source_alpha_bbox_strip_edge_guides",
        "evidence": [{
            "sourcePath": asset,
            "alphaBoundsLocal": bbox,
            "alphaPixels": alpha_pixels,
            "slotEdgeAlphaPixels": edge_alpha,
            "discardedGuidePixels": guide_pixels,
            "alphaComponents": components,
        }],
    })
    return result


def build():
    lane = load_json(AUTHORITY)
    manifest = load_json(MANIFEST)
    manifest_by_path = {row["path"]: row for row in manifest["entries"]}
    cache = {}
    templates = []
    counts = {"tree": 0, "rock": 0}

    for template in lane.get("objectTemplates", []):
        tags = set(template.get("worldgenTags", []))
        kind = next((tag for tag in SUPPORTED_TAGS if tag in tags), None)
        if kind is None:
            continue
        parts = []
        for part in template.get("parts", []):
            certified = tree_part(part, manifest_by_path, cache) if kind == "tree" else rock_part(part, manifest_by_path, cache)
            parts.append(certified)
        if not parts:
            raise ValueError(f"promoted {kind} template has no render parts: {template['templateId']}")
        counts[kind] += 1
        templates.append({"templateId": template["templateId"], "kind": kind, "parts": parts})

    if counts != {"tree": 7, "rock": 2}:
        raise ValueError(f"expected 7 tree + 2 rock/stone templates, found {counts}")
    return {
        "schema": "havenwild.terrain.worldgen_object_bounds.v1",
        "version": 3,
        "authority": "immutable_elizawy_alpha_render_bounds_edge_guide_safe",
        "sourceMutation": False,
        "certifiedKinds": counts,
        "templates": templates,
    }


def runtime_shape(document):
    return {
        "schema": document["schema"],
        "version": document["version"],
        "authority": document["authority"],
        "sourceMutation": False,
        "certifiedKinds": document["certifiedKinds"],
        "templates": [
            {
                "templateId": item["templateId"],
                "kind": item["kind"],
                "parts": [
                    {
                        "sourcePath": part["sourcePath"],
                        "sourceRectPx": part["sourceRectPx"],
                        "offsetPx": part["offsetPx"],
                    }
                    for part in item["parts"]
                ],
            }
            for item in document["templates"]
        ],
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", action="store_true", help="print detailed alpha evidence JSON")
    args = parser.parse_args(argv)
    try:
        document = build()
        OUTPUT.parent.mkdir(parents=True, exist_ok=True)
        OUTPUT.write_text(json.dumps(runtime_shape(document), indent=2) + "\n", encoding="utf-8")
        counts = document["certifiedKinds"]
        print(
            "WORLDGEN OBJECT RENDER BOUNDS: PASS / "
            f"{counts['tree']} trees + {counts['rock']} rocks/stones / immutable alpha / source mutation OFF"
        )
        print(f"AUTHORITY: {OUTPUT}")
        if args.report:
            print(json.dumps(document, indent=2))
        return 0
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f"WORLDGEN OBJECT RENDER BOUNDS: FAIL / {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
