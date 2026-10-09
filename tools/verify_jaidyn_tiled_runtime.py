#!/usr/bin/env python3
"""Acceptance verifier for the hydrated JaidynReiman Tiled runtime authority.

This is intentionally independent of the Rust implementation. It verifies the pinned
third-party bytes, atlas geometry, Wang topology, season concordance, animation and
collision counts, and detects two Tiled behaviors the first exact resolver does not yet
implement: tile probability weighting and flip/rotation synthesis.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import struct
import xml.etree.ElementTree as ET

from hydrate_jaidyn_tiled import ARCHIVE_SHA256, MEMBERS, TARGET_REL

EXPECTED = {
    "summer": {
        "tsx": "lpc-tileset-terrain-summer.tsx",
        "png": "lpc-tileset-terrain-summer.png",
        "animated": 325,
        "collision": 1461,
        "sets": {
            "Terrain": ("corner", ["Grass", "Sand", "Dirt", "Light Till", "Deep Till", "Shallow Water", "Deep Water", "Melted Ice", "Deep Sand", "Mountain", "Grassy Mountain", "Vine"], 331),
            "Walls": ("corner", ["Smithy Walls", "Stone Platform"], 24),
            "Fences": ("edge", ["Wood Fence", "Metal Fence", "Fancy Fence", "Fancy Fence Base"], 60),
        },
    },
    "spring": {
        "tsx": "lpc-tileset-terrain-spring.tsx",
        "png": "lpc-tileset-terrain-spring.png",
        "animated": 325,
        "collision": 1461,
        "sets": {
            "Terrain": ("corner", ["Grass", "Sand", "Dirt", "Light Till", "Deep Till", "Shallow Water", "Deep Water", "Melted Ice", "Deep Sand", "Mountain", "Grassy Mountain", "Vine"], 331),
            "Walls": ("corner", ["Smithy Walls", "Stone Platform"], 24),
            "Fences": ("edge", ["Wood Fence", "Metal Fence", "Fancy Fence", "Fancy Fence Base"], 60),
        },
    },
    "autumn": {
        "tsx": "lpc-tileset-terrain-autumn-old.tsx",
        "png": "lpc-tileset-terrain-autumn.png",
        "animated": 214,
        "collision": 1365,
        "sets": {
            "Terrain": ("corner", ["Grass", "Sand", "Dirt", "Light Till", "Deep Till", "Shallow Water", "Deep Water", "Melted Ice", "Deep Sand"], 281),
            "Walls": ("corner", ["Smithy Walls", "Stone Platform"], 24),
        },
    },
    "winter": {
        "tsx": "lpc-tileset-terrain-winter.tsx",
        "png": "lpc-tileset-terrain-winter.png",
        "animated": 325,
        "collision": 1461,
        "sets": {
            "Terrain": ("corner", ["Grass", "Sand", "Snow", "Light Till", "Deep Till", "Shallow Water", "Deep Water", "Ice", "Deep Sand", "Mountain", "Grassy Mountain", "Vine"], 331),
            "Walls": ("corner", ["Smithy Walls", "Stone Platform"], 24),
            "Fences": ("edge", ["Wood Fence", "Metal Fence", "Fancy Fence", "Fancy Fence Base"], 60),
        },
    },
}

BUILDINGS = {
    "tsx": "lpc-tileset-buildings.tsx",
    "png": "lpc-tileset-buildings.png",
    "animated": 0,
    "collision": 1117,
    "sets": {
        "Brick Walls": ("corner", 24, 216),
        "Flat Roofs": ("mixed", 9, 417),
        "Angled Roof": ("mixed", 6, 54),
        "Adobe Walls": ("mixed", 4, 45),
    },
}

SEASONS = ("summer", "spring", "autumn", "winter")

CORE_PURE_FILLS = ["Grass", "Sand", "Dirt", "Shallow Water", "Deep Water"]
# These are the adjacency families Havenwild must be able to express before an
# exact-only resolver can replace the live terrain path. Missing signatures do
# not invalidate the Jaidyn source; they prove we need Tiled-compatible fuzzy/
# propagation behavior (e.g. WangFiller) instead of pretending exact lookup is
# full Terrain Brush parity.
REQUIRED_EXACT_PAIR_COVERAGE = [
    ("Grass", "Sand"),
    ("Grass", "Dirt"),
    ("Grass", "Shallow Water"),
    ("Sand", "Shallow Water"),
    ("Dirt", "Shallow Water"),
    ("Shallow Water", "Deep Water"),
]
LEGACY_UNCERTIFIED_NAMES = ["PebblePath", "Pebble / Stone Path", "Dry Sand", "Wet/Tidal Sand"]


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def png_size(path: Path) -> tuple[int, int]:
    raw = path.read_bytes()[:24]
    if len(raw) < 24 or raw[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"Not a valid PNG header: {path}")
    return struct.unpack(">II", raw[16:24])


def bool_attr(value: str | None) -> bool:
    return (value or "0").strip().lower() in {"1", "true", "yes"}


def canonical_wang_set_name(name: str) -> str:
    trimmed = name.strip()
    if trimmed.lower() in {f"{season} terrain" for season in SEASONS}:
        return "Terrain"
    return trimmed


def parse_tsx(path: Path) -> dict:
    raw = path.read_bytes()
    upper = raw.upper()
    if b"<!DOCTYPE" in upper or b"<!ENTITY" in upper:
        raise ValueError(f"DTD/entity declarations are forbidden in {path}")
    root = ET.fromstring(raw)
    if root.tag != "tileset":
        raise ValueError(f"Expected <tileset> root in {path}")

    tw = int(root.get("tilewidth", "0"))
    th = int(root.get("tileheight", "0"))
    columns = int(root.get("columns", "0"))
    tilecount = int(root.get("tilecount", "0"))
    margin = int(root.get("margin", "0"))
    spacing = int(root.get("spacing", "0"))
    if tw != 32 or th != 32:
        raise ValueError(f"Unexpected tile size {tw}x{th} in {path.name}")
    if columns <= 0:
        raise ValueError(f"Invalid columns={columns} in {path.name}")

    image = root.find("image")
    if image is None:
        raise ValueError(f"Regular atlas image missing in {path.name}")
    image_source = Path(image.get("source", "")).name
    iw = int(image.get("width", "0"))
    ih = int(image.get("height", "0"))
    if iw <= 0 or ih <= 0:
        raise ValueError(f"Invalid atlas dimensions in {path.name}")
    stride_x = tw + spacing
    stride_y = th + spacing
    if stride_x <= 0 or stride_y <= 0 or iw < margin * 2 + tw or ih < margin * 2 + th:
        raise ValueError(f"Invalid tile geometry for atlas dimensions in {path.name}")
    usable_w = iw - 2 * margin + spacing
    usable_h = ih - 2 * margin + spacing
    if usable_w % stride_x != 0 or usable_h % stride_y != 0:
        raise ValueError(f"Atlas dimensions do not align to margin/spacing/tile geometry in {path.name}")
    geometric_columns = usable_w // stride_x
    geometric_rows = usable_h // stride_y
    if columns != geometric_columns:
        raise ValueError(
            f"TSX columns={columns} disagrees with image geometry columns={geometric_columns} in {path.name}"
        )

    transform = root.find("transformations")
    transformations = {
        "hflip": bool_attr(transform.get("hflip") if transform is not None else None),
        "vflip": bool_attr(transform.get("vflip") if transform is not None else None),
        "rotate": bool_attr(transform.get("rotate") if transform is not None else None),
        "preferUntransformed": bool_attr(transform.get("preferuntransformed") if transform is not None else None),
    }

    tile_meta: dict[int, dict] = {}
    animated = 0
    collision = 0
    for tile in root.findall("tile"):
        tid = int(tile.get("id", "-1"))
        probability = float(tile.get("probability", "1"))
        if probability < 0:
            raise ValueError(f"Negative probability on tile {tid} in {path.name}")
        animation = tile.find("animation")
        frames = []
        if animation is not None:
            animated += 1
            for frame in animation.findall("frame"):
                fid = int(frame.get("tileid", "-1"))
                duration = int(frame.get("duration", "0"))
                if duration <= 0:
                    raise ValueError(f"Non-positive animation duration on tile {tid} in {path.name}")
                frames.append((fid, duration))
        has_collision = tile.find("objectgroup") is not None
        if has_collision:
            collision += 1
        tile_meta[tid] = {"probability": probability, "frames": frames, "collision": has_collision}

    def rect_for(tid: int) -> tuple[int, int, int, int]:
        stride_x = tw + spacing
        stride_y = th + spacing
        x = margin + (tid % columns) * stride_x
        y = margin + (tid // columns) * stride_y
        return x, y, tw, th

    sets = {}
    for ws in root.findall("wangsets/wangset"):
        source_name = ws.get("name", "")
        name = canonical_wang_set_name(source_name)
        kind = ws.get("type", "")
        colors = [x.get("name", "") for x in ws.findall("wangcolor")]
        assignments = []
        sig_candidates: dict[tuple[int, ...], list[int]] = defaultdict(list)
        for wt in ws.findall("wangtile"):
            tid = int(wt.get("tileid", "-1"))
            wang = tuple(int(x) for x in wt.get("wangid", "").split(","))
            if len(wang) != 8:
                raise ValueError(f"Wang ID on tile {tid} in {path.name}/{name} has {len(wang)} entries")
            if any(v < 0 or v > len(colors) for v in wang):
                raise ValueError(f"Wang ID on tile {tid} in {path.name}/{name} references invalid color")
            x, y, w, h = rect_for(tid)
            if x < 0 or y < 0 or x + w > iw or y + h > ih:
                raise ValueError(f"Wang tile {tid} in {path.name}/{name} is outside atlas bounds")
            assignments.append((tid, wang))
            sig_candidates[wang].append(tid)
        assignments.sort()

        probability_sensitive = []
        for sig, tids in sig_candidates.items():
            if len(tids) < 2:
                continue
            probs = [tile_meta.get(tid, {}).get("probability", 1.0) for tid in tids]
            if len(set(probs)) > 1 or any(p == 0 for p in probs):
                probability_sensitive.append({"wangId": list(sig), "tileIds": tids, "probabilities": probs})

        if name in sets:
            raise ValueError(
                f"Ambiguous duplicate canonical Wang set {name!r} in {path.name}; source labels include {sets[name]['sourceName']!r} and {source_name!r}"
            )
        sets[name] = {
            "sourceName": source_name,
            "type": kind,
            "colors": colors,
            "assignments": assignments,
            "assignmentCount": len(assignments),
            "probabilitySensitiveGroups": probability_sensitive,
        }

    # Validate animation frame IDs against actual atlas bounds, not tilecount alone.
    for tid, meta in tile_meta.items():
        for fid, _duration in meta["frames"]:
            x, y, w, h = rect_for(fid)
            if x < 0 or y < 0 or x + w > iw or y + h > ih:
                raise ValueError(f"Animation frame {fid} referenced by tile {tid} is outside {path.name}")

    return {
        "name": root.get("name", ""),
        "tileSize": [tw, th],
        "tileCount": tilecount,
        "columns": columns,
        "geometricRows": geometric_rows,
        "margin": margin,
        "spacing": spacing,
        "image": image_source,
        "imageSize": [iw, ih],
        "transformations": transformations,
        "sets": sets,
        "animatedTiles": animated,
        "collisionTiles": collision,
        "tileMeta": tile_meta,
    }


def topology(profile: dict, set_name: str) -> list[tuple[int, tuple[int, ...]]]:
    return profile["sets"].get(set_name, {}).get("assignments", [])


def verify(root: Path, json_out: Path | None = None) -> dict:
    root = root.resolve()
    target = root / TARGET_REL
    errors: list[str] = []
    warnings: list[str] = []

    manifest_path = target / "havenwild.external.jaidyn_tiled_runtime.v2.json"
    if not manifest_path.is_file():
        errors.append(f"Hydration manifest missing: {manifest_path}")
        manifest = {}
    else:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if manifest.get("schema") != "havenwild.external.jaidyn_tiled_runtime.v2":
            errors.append("Hydration manifest schema is not havenwild.external.jaidyn_tiled_runtime.v2")
        if manifest.get("archiveSha256") != ARCHIVE_SHA256:
            errors.append("Hydration manifest archive SHA does not match pinned authority")
        if manifest.get("files") != {name: MEMBERS[name] for name in sorted(MEMBERS)}:
            errors.append("Hydration manifest member hashes do not exactly match the pinned authority")
    if not (target / "ATTRIBUTION.txt").is_file():
        errors.append("Hydrated Jaidyn lane is missing ATTRIBUTION.txt")

    for name, expected_hash in MEMBERS.items():
        p = target / name
        if not p.is_file():
            errors.append(f"Missing pinned hydrated file: {name}")
            continue
        actual = sha256_file(p)
        if actual != expected_hash:
            errors.append(f"SHA mismatch for {name}: {actual}")

    profiles = {}
    for season, spec in EXPECTED.items():
        tsx = target / spec["tsx"]
        png = target / spec["png"]
        if not tsx.is_file() or not png.is_file():
            continue
        try:
            parsed = parse_tsx(tsx)
            profiles[season] = parsed
            if png_size(png) != tuple(parsed["imageSize"]):
                errors.append(f"{season}: PNG IHDR dimensions disagree with TSX")
            if parsed["image"] != spec["png"]:
                errors.append(f"{season}: TSX image points to {parsed['image']} not {spec['png']}")
            if parsed["animatedTiles"] != spec["animated"]:
                errors.append(f"{season}: animated tile count {parsed['animatedTiles']} != {spec['animated']}")
            if parsed["collisionTiles"] != spec["collision"]:
                errors.append(f"{season}: collision tile count {parsed['collisionTiles']} != {spec['collision']}")
            if set(parsed["sets"]) != set(spec["sets"]):
                errors.append(f"{season}: Wang set names {sorted(parsed['sets'])} != {sorted(spec['sets'])}")
            for set_name, (kind, colors, count) in spec["sets"].items():
                actual_set = parsed["sets"].get(set_name)
                if not actual_set:
                    continue
                if actual_set["type"] != kind:
                    errors.append(f"{season}/{set_name}: type {actual_set['type']} != {kind}")
                if actual_set["colors"] != colors:
                    errors.append(f"{season}/{set_name}: color/material names differ from audited M2C3 evidence")
                if actual_set["assignmentCount"] != count:
                    errors.append(f"{season}/{set_name}: assignments {actual_set['assignmentCount']} != {count}")
        except Exception as exc:
            errors.append(f"{season}: {exc}")

    # Buildings are verified as structured evidence but remain outside terrain runtime authority.
    btsx = target / BUILDINGS["tsx"]
    bpng = target / BUILDINGS["png"]
    buildings = None
    if btsx.is_file() and bpng.is_file():
        try:
            buildings = parse_tsx(btsx)
            if png_size(bpng) != tuple(buildings["imageSize"]):
                errors.append("buildings: PNG IHDR dimensions disagree with TSX")
            if buildings["animatedTiles"] != BUILDINGS["animated"]:
                errors.append("buildings: animated tile count differs from audited evidence")
            if buildings["collisionTiles"] != BUILDINGS["collision"]:
                errors.append("buildings: collision tile count differs from audited evidence")
            for set_name, (kind, color_count, assignment_count) in BUILDINGS["sets"].items():
                s = buildings["sets"].get(set_name)
                if not s:
                    errors.append(f"buildings: missing Wang set {set_name}")
                    continue
                if s["type"] != kind or len(s["colors"]) != color_count or s["assignmentCount"] != assignment_count:
                    errors.append(f"buildings/{set_name}: audited profile mismatch")
        except Exception as exc:
            errors.append(f"buildings: {exc}")

    if all(x in profiles for x in ("summer", "spring", "winter")):
        for set_name in ("Terrain", "Walls", "Fences"):
            if topology(profiles["spring"], set_name) != topology(profiles["summer"], set_name):
                errors.append(f"spring/{set_name}: numeric Wang topology differs from Summer")
            if topology(profiles["winter"], set_name) != topology(profiles["summer"], set_name):
                errors.append(f"winter/{set_name}: numeric Wang topology differs from Summer")
        winter_colors = profiles["winter"]["sets"]["Terrain"]["colors"]
        summer_colors = profiles["summer"]["sets"]["Terrain"]["colors"]
        if summer_colors[2] != "Dirt" or winter_colors[2] != "Snow" or summer_colors[7] != "Melted Ice" or winter_colors[7] != "Ice":
            errors.append("Winter semantic remap Dirt->Snow / Melted Ice->Ice is not in expected positions")

    if "autumn" in profiles:
        if "Fences" in profiles["autumn"]["sets"]:
            errors.append("Autumn unexpectedly gained a Fences Wang set; re-audit upstream before promotion")
        warnings.append("Autumn remains partial/old: no fence Wang authority and reduced Terrain coverage.")

    runtime_blockers = []
    for season, profile in profiles.items():
        tr = profile["transformations"]
        if tr["hflip"] or tr["vflip"] or tr["rotate"]:
            runtime_blockers.append(f"{season}: TSX enables tile transforms {tr}; exact resolver cannot synthesize transformed Wang matches yet")
        for set_name, s in profile["sets"].items():
            if s["probabilitySensitiveGroups"]:
                runtime_blockers.append(
                    f"{season}/{set_name}: {len(s['probabilitySensitiveGroups'])} exact Wang signature group(s) depend on non-uniform/zero tile probability"
                )

    pure_fill = {}
    exact_pair_coverage = {}
    if "summer" in profiles:
        terrain = profiles["summer"]["sets"].get("Terrain")
        if terrain:
            colors = terrain["colors"]
            assignment_sigs = {sig for _tid, sig in terrain["assignments"]}
            for material in CORE_PURE_FILLS:
                try:
                    mid = colors.index(material) + 1
                except ValueError:
                    pure_fill[material] = False
                    errors.append(f"Summer Terrain lacks required core material {material}")
                    continue
                sig = (0, mid, 0, mid, 0, mid, 0, mid)
                pure_fill[material] = sig in assignment_sigs
                if not pure_fill[material]:
                    errors.append(f"Summer Terrain has no authored pure-fill Wang tile for {material}")

            # For a corner Wang set there are sixteen binary corner patterns for
            # any two terrain colors. Tiled can still paint when some exact
            # signatures are absent by using its matching/propagation behavior;
            # therefore missing patterns are a *live-promotion blocker*, not a
            # claim that the source TSX is wrong.
            for left, right in REQUIRED_EXACT_PAIR_COVERAGE:
                key = f"{left}<->{right}"
                if left not in colors or right not in colors:
                    exact_pair_coverage[key] = {"present": 0, "required": 16, "missing": "material_absent"}
                    runtime_blockers.append(f"summer/Terrain: required transition family {key} is not represented by both Wang colors")
                    continue
                a = colors.index(left) + 1
                b = colors.index(right) + 1
                missing = []
                present = 0
                for bits in range(16):
                    nw = b if bits & 1 else a
                    ne = b if bits & 2 else a
                    sw = b if bits & 4 else a
                    se = b if bits & 8 else a
                    sig = (0, ne, 0, se, 0, sw, 0, nw)
                    if sig in assignment_sigs:
                        present += 1
                    else:
                        missing.append(list(sig))
                exact_pair_coverage[key] = {"present": present, "required": 16, "missing": missing}
                if missing:
                    runtime_blockers.append(
                        f"summer/Terrain: exact-only resolver covers {present}/16 corner patterns for {key}; Tiled-compatible Wang matching/propagation is required"
                    )

            for unsafe in LEGACY_UNCERTIFIED_NAMES:
                if unsafe in colors:
                    warnings.append(f"Legacy uncertified material name unexpectedly exists in Jaidyn Terrain: {unsafe}")

    evidence_path = root / "docs/patch/evidence/M2C3_JAIDYNREIMAN_TILED_CONCORDANCE.v1.json"
    if evidence_path.is_file():
        evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
        if evidence.get("archiveSha256") != ARCHIVE_SHA256:
            errors.append("Local M2C3 evidence archive hash disagrees with M2D094B pin")
        if evidence.get("status") != "external_derivative_evidence_only":
            errors.append(
                f"Local M2C3 evidence has unexpected status {evidence.get('status')!r}; expected external_derivative_evidence_only"
            )
        if evidence.get("matchesAuditedUpload") is not True:
            errors.append("Local M2C3 evidence does not confirm the audited Jaidyn upload")
        if evidence.get("errors"):
            errors.append(f"Local M2C3 evidence contains errors: {evidence.get('errors')}")
    else:
        warnings.append("M2C3 concordance evidence file is absent; hydrated bytes are still hash-verified.")

    if errors:
        status = "FAIL_FOUNDATION"
    elif runtime_blockers:
        status = "PASS_FOUNDATION_LIVE_PROMOTION_BLOCKED"
    else:
        status = "PASS_EXACT_RUNTIME_READY"
    result = {
        "schema": "havenwild.m2d094b.jaidyn_tiled_acceptance.v2",
        "status": status,
        "foundationPass": not errors,
        "livePromotionReady": not errors and not runtime_blockers,
        "root": str(root),
        "target": str(target),
        "archiveSha256": ARCHIVE_SHA256,
        "profiles": {
            season: {
                "imageSize": p["imageSize"],
                "tileSize": p["tileSize"],
                "sets": {name: {"sourceName": s["sourceName"], "type": s["type"], "colors": s["colors"], "assignments": s["assignmentCount"], "probabilitySensitiveGroups": len(s["probabilitySensitiveGroups"])} for name, s in p["sets"].items()},
                "animatedTiles": p["animatedTiles"],
                "collisionTiles": p["collisionTiles"],
                "transformations": p["transformations"],
            }
            for season, p in profiles.items()
        },
        "summerCorePureFill": pure_fill,
        "summerExactPairCoverage": exact_pair_coverage,
        "runtimePromotionBlockers": runtime_blockers,
        "warnings": warnings,
        "errors": errors,
        "policy": {
            "tiledWorldAuthority": False,
            "havenwildWorldAuthority": True,
            "exactWangOnlyUntilTransformAndProbabilityParityCertified": True,
            "autumnFailClosed": True,
            "collisionProvisional": True,
            "legacyUncertifiedNames": LEGACY_UNCERTIFIED_NAMES,
        },
    }
    if json_out:
        json_out.parent.mkdir(parents=True, exist_ok=True)
        json_out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return result


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path.cwd())
    ap.add_argument("--json-out", type=Path)
    ap.add_argument(
        "--require-runtime-ready",
        action="store_true",
        help="Return non-zero when the source foundation passes but exact-only live promotion is still blocked.",
    )
    args = ap.parse_args()
    try:
        result = verify(args.root, args.json_out)
    except Exception as exc:
        result = {
            "schema": "havenwild.m2d094b.jaidyn_tiled_acceptance.v2",
            "status": "FAIL_VERIFIER_EXCEPTION",
            "foundationPass": False,
            "livePromotionReady": False,
            "root": str(args.root.resolve()),
            "runtimePromotionBlockers": [],
            "warnings": [],
            "errors": [f"Verifier exception: {type(exc).__name__}: {exc}"],
        }
        if args.json_out:
            args.json_out.parent.mkdir(parents=True, exist_ok=True)
            args.json_out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(result, indent=2))
    if not result["foundationPass"]:
        return 2
    if args.require_runtime_ready and not result["livePromotionReady"]:
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
