#!/usr/bin/env python3
"""Build/check the source-only Summer cliff authority inventory.

This pass deliberately does NOT infer cliff topology or promote historical family
names into runtime semantics. It consolidates exact canonical 32x32 regions from
Terrain/cliff_summer.png with the recovered seasonal crosswalk so the next cliff
pass can classify source geometry without redoing old evidence work.
"""
from __future__ import annotations

import argparse
import json
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
CROSSWALK = ROOT / "content/mapping/recovered/elizawy_all_seasons_split_source_crosswalk_b48r9_v0_1.json"
OUTPUT = ROOT / "content/terrain/recovered/native/cliff_summer_source_authority.v1.json"
ATLAS = "Terrain/cliff_summer.png"


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def build() -> dict:
    runtime = load(RUNTIME)
    crosswalk = load(CROSSWALK)

    image = next((row for row in runtime.get("runtimeSourceImages", []) if row.get("sourcePath") == ATLAS), None)
    if image is None:
        raise ValueError(f"runtime source image missing: {ATLAS}")
    if image.get("imageSizePx") != [512, 448]:
        raise ValueError(f"unexpected Summer cliff atlas size: {image.get('imageSizePx')}")

    # The unified runtime lane may also contain larger reviewed cliff source
    # sections (for example M2D090 cave/showcase templates).  This inventory is
    # specifically the 32x32 semantic-classification grid, so keep those larger
    # exact regions in the unified authority without counting them as cells.
    regions = [
        row for row in runtime.get("canonicalRuntimeRegions", [])
        if row.get("sourcePath") == ATLAS
        and (row.get("sourceRectPx") or [0, 0, 0, 0])[2:] == [32, 32]
    ]
    if len(regions) != 205:
        raise ValueError(f"expected 205 canonical Summer cliff 32x32 cells, got {len(regions)}")

    by_grid: dict[tuple[int, int], dict] = {}
    for row in regions:
        rect = row.get("sourceRectPx") or []
        if len(rect) != 4 or rect[2:] != [32, 32] or rect[0] % 32 or rect[1] % 32:
            raise ValueError(f"cliff runtime region is not a 32x32 grid cell: {rect}")
        if rect[0] < 0 or rect[1] < 0 or rect[0] + 32 > 512 or rect[1] + 32 > 448:
            raise ValueError(f"cliff runtime region escapes source bounds: {rect}")
        key = (rect[0] // 32, rect[1] // 32)
        if key in by_grid:
            raise ValueError(f"duplicate canonical cliff grid cell: {key}")
        by_grid[key] = row

    evidence: dict[tuple[int, int], list[dict]] = defaultdict(list)
    family_reference_counts: Counter[str] = Counter()
    status_counts: Counter[str] = Counter()
    reference_entries = 0
    for entry in crosswalk.get("entries", []):
        target = next((target for target in entry.get("targets", [])
                       if target.get("season") == "summer" and target.get("canonicalAtlas") == ATLAS), None)
        if target is None:
            continue
        reference_entries += 1
        family = str(entry.get("family") or "unclassified")
        family_reference_counts[family] += 1
        status_counts[str(target.get("canonicalMatchStatus") or "unknown")] += 1
        for cell in target.get("exactCanonicalCells") or []:
            if len(cell) != 2:
                continue
            key = (int(cell[0]), int(cell[1]))
            if key not in by_grid:
                raise ValueError(f"crosswalk targets non-canonical Summer cliff cell: {key}")
            evidence[key].append({
                "family": family,
                "referencePath": entry.get("referencePath"),
                "referenceCell": entry.get("referenceCell"),
                "canonicalMatchStatus": target.get("canonicalMatchStatus"),
                "sourceAddressVerified": bool(entry.get("sourceAddressVerified")),
                "semanticRoleVerified": bool(entry.get("semanticRoleVerified")),
            })

    cells = []
    hinted_cells = 0
    for (gx, gy), row in sorted(by_grid.items(), key=lambda item: (item[0][1], item[0][0])):
        refs = evidence.get((gx, gy), [])
        family_hints = sorted(set(row.get("familyHints") or []) | {ref["family"] for ref in refs})
        if family_hints:
            hinted_cells += 1
        cells.append({
            "grid": [gx, gy],
            "sourceRectPx": [gx * 32, gy * 32, 32, 32],
            "canonicalRegionId": row.get("canonicalRegionId"),
            "familyHints": family_hints,
            "historicalReferenceEvidence": refs,
            "semanticStatus": "unclassified_source_cell",
            "runtimeCliffRole": None,
            "collisionRole": None,
            "traversalRole": None,
            "occlusionRole": None,
        })

    return {
        "schema": "havenwild.terrain.cliff_summer_source_authority.v1",
        "version": 1,
        "sourceAtlas": ATLAS,
        "sourceSha256": image.get("sourceSha256"),
        "imageSizePx": image.get("imageSizePx"),
        "tileSizePx": 32,
        "authority": "exact_source_inventory_plus_historical_crosswalk_evidence_only",
        "semanticPromotionAllowed": False,
        "summary": {
            "canonicalCells": len(cells),
            "cellsWithHistoricalFamilyHints": hinted_cells,
            "historicalReferenceEntries": reference_entries,
            "historicalReferenceStatusCounts": dict(sorted(status_counts.items())),
            "historicalFamilyReferenceCounts": dict(sorted(family_reference_counts.items())),
            "runtimeCliffRolesCertified": 0,
            "collisionRolesCertified": 0,
            "traversalRolesCertified": 0,
            "occlusionRolesCertified": 0,
        },
        "havenwildStructuralPolicy": {
            "elevationRange": [0, 30],
            "seaLevel": 0,
            "oneLevelCliffsAreValid": True,
            "sourceArtMutationAllowed": False,
            "generatedCliffArtworkAllowed": False,
            "accessPolicy": "raised regions require route-aware valid access; ramps are not required on every cliff face",
            "coastalAccessPolicy": "water-contact traversal may use water-appropriate connectors such as vines when source geometry and gameplay rules permit",
            "sharedAuthorityTarget": "one structural cliff recipe must drive editor visuals, runtime visuals, collision, navigation/traversal and occlusion",
        },
        "cells": cells,
        "notes": [
            "This inventory intentionally stops before semantic cliff role certification.",
            "Historical mountain_base/mountain_vines/mountain_features/mountain_animated_water/mountain_waterfall_transitions labels are evidence buckets, not runtime cliff topology roles.",
            "The next pass should classify cliff top/face/corner/cap/connector geometry from source and retained Native evidence before enabling automatic elevation rendering.",
            "Waterfall animation remains a separate source-authority task and must not reuse the rejected terrain_summer RepeatableFill-as-animation shortcut.",
        ],
    }


def encoded(data: dict) -> bytes:
    return (json.dumps(data, indent=2) + "\n").encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = build()
    payload = encoded(data)
    if args.check:
        if not OUTPUT.is_file() or OUTPUT.read_bytes() != payload:
            print("M2D082 CLIFF SOURCE AUTHORITY: FAIL / checked-in inventory differs from deterministic build")
            return 2
        summary = data["summary"]
        print(
            "M2D082 CLIFF SOURCE AUTHORITY: PASS / "
            f"{summary['canonicalCells']} canonical cells / "
            f"{summary['historicalReferenceEntries']} historical refs / "
            "0 semantic promotions"
        )
        return 0
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(payload)
    print("WROTE", OUTPUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
