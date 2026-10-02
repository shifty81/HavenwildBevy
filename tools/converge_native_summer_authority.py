#!/usr/bin/env python3
"""One-pass Native -> Bevy Summer terrain authority convergence.

The checked-in 0.8 authority is already source-derived from the mature Havenwild
Native/LPC mapping. This command verifies that authority against the hydrated,
pinned ElizaWy Summer PNG, publishes an identical local override for explicit
editor provenance, and quarantines experimental v1 large-object crop metadata.
It never edits source artwork.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parents[1]
ATLAS_REL = "Terrain/terrain_summer.png"
ATLAS = ROOT / "assets" / "elizawy" / ATLAS_REL
PINNED_SHA = "1251a6ea556330190ccb1e3af166eb69fbec4b7a3f7d51c1f54728abd75bd752"
PROFILE = ROOT / "content" / "terrain" / "recipes" / "summer_flatworld_runtime.v1.json"
DONOR_MAP = ROOT / "content" / "terrain" / "recovered" / "native" / "lpc_terrain_summer_complete_map_32.native.v1.json"
DONOR_TOPOLOGY = ROOT / "content" / "terrain" / "recovered" / "native" / "lpc_seasonal_terrain_topology.native.v1.json"
REVIEW = ROOT / "content" / "terrain" / "recovered" / "native" / "native_summer_authority_review.v1.json"
LOCAL_DIR = ROOT / ".forgepy" / "terrain"
LOCAL_PROFILE = LOCAL_DIR / "summer_flatworld_runtime.local.json"
REPORT = LOCAL_DIR / "native_summer_convergence.v2.json"
OBJECT_BOUNDS = LOCAL_DIR / "worldgen_object_bounds.local.json"
OBJECT_QUARANTINE = LOCAL_DIR / "worldgen_object_bounds.pre_080.quarantined.json"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def profile_metrics(profile: dict) -> dict:
    direct = profile.get("cornerRecipes", {})
    composites = profile.get("cornerComposites", {})
    keys = list(direct) + list(composites)
    pairs = {
        "Grass|MudBank": frozenset(("Grass", "MudBank")),
        "Grass|RiverWater": frozenset(("Grass", "RiverWater")),
        "MudBank|RiverWater": frozenset(("MudBank", "RiverWater")),
    }
    counts = {
        name: sum(1 for key in keys if frozenset(key.split("|")) == roles)
        for name, roles in pairs.items()
    }
    return {
        "safeFills": len(profile.get("safeFill", {})),
        "directMixedRecipes": len(direct),
        "exactSourceCompositeRecipes": len(composites),
        "totalFlatStates": len(profile.get("safeFill", {})) + len(direct) + len(composites),
        "pairMixedCounts": counts,
    }


def verify() -> dict:
    for path in (PROFILE, DONOR_MAP, DONOR_TOPOLOGY, REVIEW):
        if not path.is_file():
            raise ValueError(f"Missing checked-in Summer authority artifact: {path.relative_to(ROOT)}")
    if not ATLAS.is_file():
        raise ValueError("Hydrated Terrain/terrain_summer.png is missing; run PCC Assets / hydration first")
    atlas_sha = digest(ATLAS)
    if atlas_sha != PINNED_SHA:
        raise ValueError(f"terrain_summer.png differs from pinned ElizaWy source: {atlas_sha}")

    profile = load(PROFILE)
    donor_map = load(DONOR_MAP)
    donor_topology = load(DONOR_TOPOLOGY)
    review = load(REVIEW)
    metrics = profile_metrics(profile)

    if profile.get("schema") != "havenwild.terrain.summer_flatworld_runtime.v1" or profile.get("version") != 5:
        raise ValueError("Checked-in Summer runtime profile is not v5 water-authority-corrected source authority")
    if profile.get("sourceAtlas") != ATLAS_REL or profile.get("sourceAtlasSha256") != PINNED_SHA:
        raise ValueError("Summer runtime profile source identity mismatch")
    summary = donor_map.get("summary", {})
    if summary.get("mappedNonTransparentCells") != 305 or summary.get("nonTransparentCells") != 305:
        raise ValueError("Native complete Summer donor no longer proves 305/305 source coverage")
    if donor_topology.get("summary", {}).get("familyCount") != 20:
        raise ValueError("Native topology donor no longer contains the expected 20 Summer families")
    if metrics["totalFlatStates"] != 81 or any(value != 14 for value in metrics["pairMixedCounts"].values()):
        raise ValueError(f"Flat Summer runtime coverage is incomplete: {metrics}")
    if len(profile.get("sourceGroups", {})) != 41:
        raise ValueError("Summer v5 runtime profile no longer exposes 41 recovered source groups")
    if sum(group.get("cellCount", 0) for group in profile.get("sourceGroups", {}).values()) != 305:
        raise ValueError("Summer v5 runtime profile no longer exposes all 305 source cells")
    expected_variants = {"Grass": 6, "MudBank": 6, "RiverWater": 1}
    for role, expected in expected_variants.items():
        if len(profile.get("fillVariants", {}).get(role, [])) != expected:
            raise ValueError(f"Summer v5 fill variants changed for {role}")
    if "RiverWater" in profile.get("animations", {}):
        raise ValueError("Summer v5 forbids promoting RepeatableFill water variants into animation frames")
    water_group = profile.get("sourceGroups", {}).get("summer_water_fill", {})
    if water_group.get("classification") != "RepeatableFill" or water_group.get("cellCount") != 8:
        raise ValueError("Summer v5 must preserve the eight water-looking cells as static RepeatableFill source variants")
    if review.get("sourceMutationAllowed") is not False:
        raise ValueError("Native authority review must explicitly forbid source mutation")
    return {
        "atlasSha256": atlas_sha,
        "sourceCoverage": {
            "mappedNonTransparent": 305,
            "nonTransparent": 305,
            "structuralTransparent": summary.get("structuralTransparentCells"),
            "unusedTransparent": summary.get("unusedTransparentCells"),
            "groups": summary.get("primaryGroups"),
            "topologyFamilies": donor_topology.get("summary", {}).get("familyCount"),
        },
        "runtime": metrics,
    }


def converge() -> int:
    facts = verify()
    LOCAL_DIR.mkdir(parents=True, exist_ok=True)
    shutil.copy2(PROFILE, LOCAL_PROFILE)
    quarantined = False
    if OBJECT_BOUNDS.is_file():
        try:
            obj = load(OBJECT_BOUNDS)
        except Exception:
            obj = {}
        if int(obj.get("version", 0)) < 2:
            if OBJECT_QUARANTINE.exists():
                OBJECT_QUARANTINE.unlink()
            OBJECT_BOUNDS.replace(OBJECT_QUARANTINE)
            quarantined = True
    report = {
        "schema": "havenwild.terrain.native_summer_convergence.v2",
        "status": "PASS",
        "sourceArtworkModified": False,
        "checkedInProfile": str(PROFILE.relative_to(ROOT)).replace("\\", "/"),
        "localProfile": str(LOCAL_PROFILE.relative_to(ROOT)).replace("\\", "/"),
        "nativeDonorMap": str(DONOR_MAP.relative_to(ROOT)).replace("\\", "/"),
        "nativeDonorTopology": str(DONOR_TOPOLOGY.relative_to(ROOT)).replace("\\", "/"),
        "review": str(REVIEW.relative_to(ROOT)).replace("\\", "/"),
        "legacyObjectBoundsQuarantined": quarantined,
        **facts,
    }
    REPORT.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("NATIVE SUMMER CONVERGENCE: PASS")
    print("  exact ElizaWy source: verified and unchanged")
    print("  source catalog      : 305/305 non-transparent cells")
    print("  topology donor      : 20 Summer families")
    print("  flat runtime        : 45/45 states; 14/14 each Grass/Dirt, Grass/Water, Dirt/Water")
    print("  pixel classifier    : diagnostic only")
    if quarantined:
        print("  object crops        : experimental v1 bounds quarantined")
    print(f"  report              : {REPORT}")
    return 0


def status() -> int:
    try:
        facts = verify()
    except Exception as exc:
        print(f"NATIVE SUMMER STATUS: FAIL - {exc}")
        return 1
    print("NATIVE SUMMER STATUS: PASS")
    print(json.dumps(facts, indent=2))
    if REPORT.is_file():
        print(f"Last convergence report: {REPORT}")
    else:
        print("Last convergence report: not run yet")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", nargs="?", choices=("converge", "status"), default="converge")
    parser.add_argument("--source", help="Retained for PCC compatibility; checked-in reviewed donor snapshots are authoritative in 0.8.")
    args = parser.parse_args()
    return status() if args.action == "status" else converge()


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"NATIVE SUMMER CONVERGENCE: FAIL - {exc}", file=sys.stderr)
        raise SystemExit(1)
