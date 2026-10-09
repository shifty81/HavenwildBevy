#!/usr/bin/env python3
"""Interpret M2D096B shoreline evidence without changing runtime semantics.

This tool deliberately cannot promote a Havenwild role. It summarizes what the
hydrated Jaidyn TSX actually says about Sand / Deep Sand / Shallow Water / Deep
Water animation ownership, then leaves WetSand/PebblePath blocked until explicit
source/demo evidence is reviewed.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

SCHEMA = "havenwild.m2d096c.shoreline_promotion_readiness.v1"
INPUT_SCHEMA = "havenwild.m2d096b.jaidyn_shoreline_animation.v1"
DEFAULT_INPUT = Path("artifacts/terrain-certification/M2D096B_JaidynShorelineAnimation.json")
DEFAULT_OUTPUT = Path("artifacts/terrain-certification/M2D096C_ShorelinePromotionReadiness.json")
TARGETS = ("Sand", "Deep Sand", "Shallow Water", "Deep Water")


def _count_pair(material: dict, partner: str) -> int:
    return int((material.get("animatedPartnerAssignmentCounts") or {}).get(partner, 0))


def build_report(source: dict) -> dict:
    if source.get("schema") != INPUT_SCHEMA:
        raise ValueError(f"Expected {INPUT_SCHEMA}, got {source.get('schema')!r}")
    materials = source.get("materials") or {}
    missing = [name for name in TARGETS if name not in materials]
    if missing:
        raise ValueError(f"Missing shoreline materials in evidence report: {missing}")

    deep = materials["Deep Sand"]
    shallow = materials["Shallow Water"]
    sand = materials["Sand"]
    deep_water = materials["Deep Water"]

    deep_pure_animated = list(deep.get("animatedPureFillTileIds") or [])
    shallow_pure_animated = list(shallow.get("animatedPureFillTileIds") or [])
    sand_pure_animated = list(sand.get("animatedPureFillTileIds") or [])
    deep_water_pure_animated = list(deep_water.get("animatedPureFillTileIds") or [])
    deep_shallow_animated = max(
        _count_pair(deep, "Shallow Water"),
        _count_pair(shallow, "Deep Sand"),
    )
    sand_shallow_animated = max(
        _count_pair(sand, "Shallow Water"),
        _count_pair(shallow, "Sand"),
    )

    if deep_pure_animated:
        shoreline_hypothesis = "DEEP_SAND_PURE_FILL_HAS_AUTHORED_ANIMATION"
    elif deep_shallow_animated:
        shoreline_hypothesis = "DEEP_SAND_SHALLOW_WATER_TRANSITION_HAS_AUTHORED_ANIMATION"
    elif int(shallow.get("animatedAssignmentCount") or 0) > 0:
        shoreline_hypothesis = "ANIMATED_WATER_OVER_STATIC_SHORE_REMAINS_PLAUSIBLE"
    else:
        shoreline_hypothesis = "NO_AUTHORED_SHORE_ANIMATION_PROVEN_FOR_TARGET_SET"

    return {
        "schema": SCHEMA,
        "status": "EVIDENCE_CLASSIFIED_NO_RUNTIME_PROMOTION",
        "sourceEvidenceSchema": INPUT_SCHEMA,
        "sourceTsx": source.get("tsx"),
        "terrainWangSet": source.get("terrainWangSet"),
        "classification": {
            "shorelineHypothesis": shoreline_hypothesis,
            "deepSandPureAnimatedTileIds": deep_pure_animated,
            "shallowWaterPureAnimatedTileIds": shallow_pure_animated,
            "sandPureAnimatedTileIds": sand_pure_animated,
            "deepWaterPureAnimatedTileIds": deep_water_pure_animated,
            "deepSandShallowWaterAnimatedAssignments": deep_shallow_animated,
            "sandShallowWaterAnimatedAssignments": sand_shallow_animated,
            "deepSandDistinctTerrainMaterial": bool((source.get("interpretation") or {}).get("deepSandIsDistinctTerrainMaterial")),
        },
        "runtimePolicy": {
            "WetSand": "BLOCKED_UNPROVEN_DO_NOT_ALIAS_TO_DEEP_SAND",
            "PebblePath": "BLOCKED_UNPROVEN_DO_NOT_USE_AS_TERRAIN_TRANSITION",
            "animationPresentation": "PREPARED_BUT_DISABLED_PENDING_CANONICAL_FRAME_COVERAGE_AND_VISUAL_ACCEPTANCE",
            "configuredDerivativePixels": "NEVER_RUNTIME_ART_AUTHORITY",
            "canonicalElizaWyPixels": "REQUIRED_FOR_EVERY_PROMOTED_FRAME"
        },
        "nextEvidence": [
            "Compare hydrated Summer TSX assignments with pinned ElizaWy Summer test-scene composition.",
            "Require all animation frames for any promoted tile to exact-match canonical ElizaWy source pixels.",
            "Enable presentation only after Vulkan visual acceptance shows no topology shift, flashing, or shoreline desynchronization."
        ]
    }


def _fixture() -> dict:
    def material(animated=0, pure=None, partners=None):
        return {
            "animatedAssignmentCount": animated,
            "animatedPureFillTileIds": pure or [],
            "animatedPartnerAssignmentCounts": partners or {},
        }
    return {
        "schema": INPUT_SCHEMA,
        "tsx": "fixture.tsx",
        "terrainWangSet": "Summer Terrain",
        "materials": {
            "Sand": material(animated=1, partners={"Shallow Water": 1}),
            "Deep Sand": material(animated=2, partners={"Shallow Water": 2}),
            "Shallow Water": material(animated=4, pure=[12], partners={"Sand": 1, "Deep Sand": 2}),
            "Deep Water": material(animated=1, pure=[33]),
        },
        "interpretation": {"deepSandIsDistinctTerrainMaterial": True},
    }


def selftest() -> int:
    result = build_report(_fixture())
    c = result["classification"]
    assert c["shorelineHypothesis"] == "DEEP_SAND_SHALLOW_WATER_TRANSITION_HAS_AUTHORED_ANIMATION"
    assert c["deepSandShallowWaterAnimatedAssignments"] == 2
    assert c["sandShallowWaterAnimatedAssignments"] == 1
    assert result["runtimePolicy"]["WetSand"].startswith("BLOCKED_")
    assert result["runtimePolicy"]["PebblePath"].startswith("BLOCKED_")
    print("M2D096C SHORELINE PROMOTION READINESS SELFTEST: PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path.cwd())
    ap.add_argument("--input", type=Path)
    ap.add_argument("--output", type=Path)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args(argv)
    if args.selftest:
        return selftest()

    root = args.root.resolve()
    input_path = args.input or (root / DEFAULT_INPUT)
    if not input_path.is_absolute():
        input_path = root / input_path
    if not input_path.is_file():
        print(f"M2D096C SHORELINE PROMOTION READINESS: BLOCKED - missing evidence report: {input_path}", file=sys.stderr)
        return 2
    try:
        source = json.loads(input_path.read_text(encoding="utf-8"))
        report = build_report(source)
    except Exception as exc:
        print(f"M2D096C SHORELINE PROMOTION READINESS: FAIL - {exc}", file=sys.stderr)
        return 1

    output_path = args.output or (root / DEFAULT_OUTPUT)
    if not output_path.is_absolute():
        output_path = root / output_path
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    c = report["classification"]
    print("M2D096C SHORELINE PROMOTION READINESS")
    print(f"Hypothesis: {c['shorelineHypothesis']}")
    print(f"Deep Sand pure animated tiles: {c['deepSandPureAnimatedTileIds']}")
    print(f"Deep Sand <-> Shallow Water animated assignments: {c['deepSandShallowWaterAnimatedAssignments']}")
    print(f"Sand <-> Shallow Water animated assignments: {c['sandShallowWaterAnimatedAssignments']}")
    print("WetSand: BLOCKED / no automatic alias")
    print("PebblePath: BLOCKED / no terrain-transition use")
    print(f"REPORT: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
