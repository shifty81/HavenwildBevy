#!/usr/bin/env python3
"""Expose the complete hydrated ElizaWy image lane to Generated World tooling.

This is a catalog/intent pass, not an auto-placement promotion. Every canonical PNG
becomes discoverable to worldgen, while only already certified object templates or
explicit terrain authorities may place pixels automatically.
"""
from __future__ import annotations
import argparse, json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "content/catalog/core_source_manifest.json"
RUNTIME = ROOT / "content/assets/authority/elizawy_runtime_index.v1.json"
OUT = ROOT / "content/worldgen/elizawy_worldgen_asset_catalog.v1.json"


def classify(path: str) -> tuple[str, list[str]]:
    lower = path.lower()
    tags: list[str] = []
    if path.startswith("Terrain/"):
        domain = "terrain_or_nature"
        if "cliff" in lower: tags += ["elevation", "cliff"]
        if "waterfall" in lower: tags += ["water", "waterfall"]
        if "terrain_" in lower: tags += ["ground", "seasonal_terrain"]
        if "tree" in lower: tags += ["vegetation", "tree"]
        if any(x in lower for x in ("plant", "flower", "mushroom")): tags += ["vegetation", "ground_detail"]
        if "tilled" in lower: tags += ["farming", "soil"]
    elif path.startswith("Structure/"):
        domain = "structure"
        for token, tag in [
            ("bridge", "bridge"), ("door", "door"), ("roof", "roof"),
            ("wall", "wall"), ("floor", "floor"), ("fence", "fence"),
            ("stair", "stairs"), ("window", "window"), ("dock", "dock"),
        ]:
            if token in lower: tags.append(tag)
        tags.append("settlement")
    elif path.startswith("Objects/"):
        domain = "object"
        if "furniture" in lower: tags += ["furniture", "interior"]
        if any(x in lower for x in ("barrel", "crate", "sack", "box")): tags += ["clutter", "storage"]
        if any(x in lower for x in ("sign", "banner")): tags += ["signage"]
        tags += ["prop"]
    elif path.startswith("FX/"):
        domain = "fx"
        tags += ["effect"]
        if "water" in lower or "splash" in lower or "ripple" in lower: tags += ["water"]
    else:
        domain = "other"
    for season in ("spring", "summer", "autumn", "winter"):
        if season in lower: tags.append(season)
    return domain, sorted(set(tags))


def build() -> dict:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    runtime = json.loads(RUNTIME.read_text(encoding="utf-8"))
    template_paths = {
        part["sourcePath"]
        for template in runtime.get("objectTemplates", [])
        for part in template.get("parts", [])
    }
    rows = []
    domains = Counter()
    states = Counter()
    for entry in manifest.get("entries", []):
        path = entry.get("path", "")
        if not path.endswith(".png"):
            continue
        domain, tags = classify(path)
        if path == "Terrain/terrain_summer.png":
            state = "certified_terrain_authority"
        elif path in template_paths:
            state = "certified_template_source_available"
        else:
            state = "source_available_requires_context_mapping"
        domains[domain] += 1
        states[state] += 1
        rows.append({
            "sourcePath": path,
            "sourceAssetId": entry.get("id"),
            "imageSizePx": entry.get("imageSizePx"),
            "domain": domain,
            "contextTags": tags,
            "worldgenState": state,
            "generatedArtwork": False,
            "autoPlacementPolicy": (
                "allowed_through_existing_authority_only"
                if state != "source_available_requires_context_mapping"
                else "disabled_until_mapped"
            ),
        })
    rows.sort(key=lambda row: row["sourcePath"].lower())
    if len(rows) != 320:
        raise ValueError(f"expected all 320 canonical PNGs, got {len(rows)}")
    return {
        "schema": "havenwild.worldgen.elizawy_asset_catalog.v1",
        "purpose": "complete ElizaWy source discovery for Generated World; contextual placement remains evidence-gated",
        "counts": {
            "images": len(rows),
            "domains": dict(sorted(domains.items())),
            "states": dict(sorted(states.items())),
        },
        "assets": rows,
    }


def dump(data: dict) -> bytes:
    return (json.dumps(data, indent=2) + "\n").encode("utf-8")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    data = dump(build())
    if args.check:
        if not OUT.is_file() or OUT.read_bytes() != data:
            raise SystemExit(f"Worldgen ElizaWy catalog is stale: {OUT}")
        print("ELIZAWY WORLDGEN CATALOG: PASS / 320/320 canonical PNGs exposed; automatic placement remains authority-gated")
        return 0
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_bytes(data)
    print(f"WROTE {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
