#!/usr/bin/env python3
"""Audit configured Jaidyn/LPC shoreline animation evidence without promoting runtime semantics.

This tool is intentionally evidence-only. It reads the hydrated Summer TSX and
reports which Terrain Wang assignments involving Sand, Deep Sand, Shallow Water,
and Deep Water own authored Tiled animations, including exact frame tile IDs and
durations. It does not mutate source art, mapping data, or runtime roles.
"""
from __future__ import annotations

import argparse
import json
import sys
import tempfile
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from pathlib import Path

TARGETS = ("Sand", "Deep Sand", "Shallow Water", "Deep Water")
DEFAULT_TSX = Path("assets/elizawy/_jaidyn_tiled/lpc-tileset-terrain-summer.tsx")
DEFAULT_REPORT = Path("artifacts/terrain-certification/M2D096B_JaidynShorelineAnimation.json")
CORNER_SLOTS = (1, 3, 5, 7)


def _parse_wang_id(raw: str) -> tuple[int, ...]:
    parts = [p.strip() for p in raw.split(",")]
    if len(parts) != 8:
        raise ValueError(f"Wang ID must contain 8 entries, got {len(parts)}: {raw!r}")
    out = tuple(int(p or "0", 0) for p in parts)
    if any(v < 0 or v > 255 for v in out):
        raise ValueError(f"Wang ID entry outside u8 range: {raw!r}")
    return out


def _terrain_wang_set(root: ET.Element) -> ET.Element:
    wangsets = root.find("wangsets")
    if wangsets is None:
        raise ValueError("TSX has no <wangsets>")
    candidates = list(wangsets.findall("wangset"))
    for wangset in candidates:
        name = (wangset.get("name") or "").strip().lower()
        if name in {"terrain", "summer terrain"}:
            return wangset
    for wangset in candidates:
        if (wangset.get("type") or "").strip().lower() == "corner":
            names = {(c.get("name") or "").strip() for c in wangset.findall("wangcolor")}
            if {"Sand", "Shallow Water", "Deep Water"}.issubset(names):
                return wangset
    raise ValueError("No configured Summer Terrain Wang set found")


def audit_tsx(tsx_path: Path) -> dict:
    tree = ET.parse(tsx_path)
    root = tree.getroot()
    tile_w = int(root.get("tilewidth") or 0)
    tile_h = int(root.get("tileheight") or 0)
    if (tile_w, tile_h) != (32, 32):
        raise ValueError(f"Expected 32x32 configured LPC tiles, got {tile_w}x{tile_h}")

    wangset = _terrain_wang_set(root)
    colors: dict[int, str] = {}
    for index, color in enumerate(wangset.findall("wangcolor"), start=1):
        colors[index] = (color.get("name") or "").strip()
    name_to_id = {name: ident for ident, name in colors.items()}

    animations: dict[int, list[dict]] = {}
    tile_types: dict[int, str] = {}
    for tile in root.findall("tile"):
        tile_id = int(tile.get("id") or 0)
        tile_types[tile_id] = (tile.get("type") or tile.get("class") or "").strip()
        animation = tile.find("animation")
        if animation is None:
            continue
        frames = []
        for frame in animation.findall("frame"):
            frames.append({
                "tileId": int(frame.get("tileid") or 0),
                "durationMs": int(frame.get("duration") or 0),
            })
        if frames:
            animations[tile_id] = frames

    assignments = []
    for wt in wangset.findall("wangtile"):
        tile_id = int(wt.get("tileid") or 0)
        wang = _parse_wang_id(wt.get("wangid") or "")
        corner_ids = tuple(wang[i] for i in CORNER_SLOTS)
        corner_names = tuple(colors.get(i, "") if i else "" for i in corner_ids)
        used = sorted({name for name in corner_names if name})
        frames = animations.get(tile_id, [])
        assignments.append({
            "tileId": tile_id,
            "wangId": list(wang),
            "cornerMaterials": list(corner_names),
            "materials": used,
            "animated": bool(frames),
            "animation": frames,
            "tileType": tile_types.get(tile_id, ""),
        })

    material_reports = {}
    for material in TARGETS:
        material_id = name_to_id.get(material)
        involving = [a for a in assignments if material in a["materials"]]
        animated = [a for a in involving if a["animated"]]
        pure = [
            a for a in involving
            if a["cornerMaterials"] == [material, material, material, material]
        ]
        pure_animated = [a for a in pure if a["animated"]]
        partners = Counter()
        animated_partners = Counter()
        for assignment in involving:
            others = [m for m in assignment["materials"] if m != material]
            if not others:
                partners["<pure fill>"] += 1
                if assignment["animated"]:
                    animated_partners["<pure fill>"] += 1
            else:
                for other in others:
                    partners[other] += 1
                    if assignment["animated"]:
                        animated_partners[other] += 1
        durations = Counter()
        frame_ids = set()
        for assignment in animated:
            for frame in assignment["animation"]:
                durations[frame["durationMs"]] += 1
                frame_ids.add(frame["tileId"])
        material_reports[material] = {
            "wangColorId": material_id,
            "assignmentCount": len(involving),
            "animatedAssignmentCount": len(animated),
            "pureFillTileIds": [a["tileId"] for a in pure],
            "animatedPureFillTileIds": [a["tileId"] for a in pure_animated],
            "partnerAssignmentCounts": dict(sorted(partners.items())),
            "animatedPartnerAssignmentCounts": dict(sorted(animated_partners.items())),
            "animationFrameIds": sorted(frame_ids),
            "durationHistogramMs": {str(k): v for k, v in sorted(durations.items())},
            "animatedAssignments": animated,
        }

    relevant = [a for a in assignments if any(t in a["materials"] for t in TARGETS)]
    report = {
        "schema": "havenwild.m2d096b.jaidyn_shoreline_animation.v1",
        "status": "EVIDENCE_ONLY_NO_RUNTIME_PROMOTION",
        "tsx": str(tsx_path),
        "tileset": (root.get("name") or "").strip(),
        "tileSize": [tile_w, tile_h],
        "terrainWangSet": (wangset.get("name") or "").strip(),
        "terrainMaterials": [colors[i] for i in sorted(colors)],
        "animatedTileCountAll": len(animations),
        "terrainAssignmentCount": len(assignments),
        "shoreRelevantAssignmentCount": len(relevant),
        "shoreRelevantAnimatedAssignmentCount": sum(1 for a in relevant if a["animated"]),
        "materials": material_reports,
        "interpretation": {
            "deepSandIsDistinctTerrainMaterial": "Deep Sand" in name_to_id,
            "deepSandHasAnimatedWangAssignments": bool(material_reports["Deep Sand"]["animatedAssignmentCount"]),
            "shallowWaterHasAnimatedWangAssignments": bool(material_reports["Shallow Water"]["animatedAssignmentCount"]),
            "sandHasAnimatedWangAssignments": bool(material_reports["Sand"]["animatedAssignmentCount"]),
            "note": "Animation ownership is authored TSX evidence. It does not by itself prove Havenwild WetSand == Deep Sand.",
        },
    }
    return report


def print_summary(report: dict) -> None:
    print("JAIDYN SHORELINE / ANIMATION AUTHORITY AUDIT")
    print(f"TSX: {report['tsx']}")
    print(f"Terrain set: {report['terrainWangSet']} / {report['terrainAssignmentCount']} assignments")
    print(f"All animated tiles in TSX: {report['animatedTileCountAll']}")
    print(
        "Shore-relevant: "
        f"{report['shoreRelevantAssignmentCount']} assignments / "
        f"{report['shoreRelevantAnimatedAssignmentCount']} animated"
    )
    for material in TARGETS:
        item = report["materials"][material]
        print(
            f"  {material}: wang={item['wangColorId']} "
            f"assignments={item['assignmentCount']} "
            f"animated={item['animatedAssignmentCount']} "
            f"pure={item['pureFillTileIds']} "
            f"animated-pure={item['animatedPureFillTileIds']}"
        )
        if item["animatedPartnerAssignmentCounts"]:
            print(f"    animated partners: {item['animatedPartnerAssignmentCounts']}")
        if item["durationHistogramMs"]:
            print(f"    frame durations: {item['durationHistogramMs']}")
    print("RESULT: PASS / evidence report only; no terrain role was promoted")


def _selftest() -> int:
    xml = '''<?xml version="1.0" encoding="UTF-8"?>
<tileset name="fixture" tilewidth="32" tileheight="32" tilecount="16" columns="4">
 <image source="fixture.png" width="128" height="128"/>
 <wangsets>
  <wangset name="Summer Terrain" type="corner" tile="0">
   <wangcolor name="Grass" color="#00ff00" tile="0" probability="1"/>
   <wangcolor name="Sand" color="#ffff00" tile="1" probability="1"/>
   <wangcolor name="Shallow Water" color="#00ffff" tile="2" probability="1"/>
   <wangcolor name="Deep Water" color="#0000ff" tile="3" probability="1"/>
   <wangcolor name="Deep Sand" color="#ff9900" tile="4" probability="1"/>
   <wangtile tileid="1" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="2" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="3" wangid="0,5,0,5,0,5,0,5"/>
   <wangtile tileid="4" wangid="0,3,0,5,0,5,0,3"/>
  </wangset>
 </wangsets>
 <tile id="2"><animation><frame tileid="2" duration="120"/><frame tileid="6" duration="120"/></animation></tile>
 <tile id="4"><animation><frame tileid="4" duration="80"/><frame tileid="7" duration="100"/></animation></tile>
</tileset>'''
    with tempfile.TemporaryDirectory(prefix="havenwild-m2d096b-") as td:
        path = Path(td) / "fixture.tsx"
        path.write_text(xml, encoding="utf-8")
        report = audit_tsx(path)
    assert report["terrainAssignmentCount"] == 4
    assert report["materials"]["Sand"]["animatedAssignmentCount"] == 0
    assert report["materials"]["Shallow Water"]["animatedAssignmentCount"] == 2
    assert report["materials"]["Deep Sand"]["animatedAssignmentCount"] == 1
    assert report["materials"]["Deep Sand"]["animatedPureFillTileIds"] == []
    assert report["interpretation"]["deepSandIsDistinctTerrainMaterial"] is True
    assert report["interpretation"]["deepSandHasAnimatedWangAssignments"] is True
    print("M2D096B SHORELINE ANIMATION AUDITOR SELFTEST: PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--tsx", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--selftest", action="store_true")
    args = parser.parse_args(argv)
    if args.selftest:
        return _selftest()

    root = args.root.resolve()
    tsx = args.tsx or (root / DEFAULT_TSX)
    if not tsx.is_absolute():
        tsx = root / tsx
    if not tsx.is_file():
        print(f"M2D096B SHORELINE ANIMATION AUDIT: BLOCKED - TSX not found: {tsx}", file=sys.stderr)
        print("Run PCC -> Full Gate or asset hydration first; source-only ZIPs intentionally omit third-party hydrated art.", file=sys.stderr)
        return 2
    try:
        report = audit_tsx(tsx)
    except Exception as exc:
        print(f"M2D096B SHORELINE ANIMATION AUDIT: FAIL - {exc}", file=sys.stderr)
        return 1

    report_path = args.report or (root / DEFAULT_REPORT)
    if not report_path.is_absolute():
        report_path = root / report_path
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print_summary(report)
    print(f"REPORT: {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
