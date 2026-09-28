#!/usr/bin/env python3
"""Build a deterministic, source-only asset mapping inventory. Never promotes mappings."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = Path("content/catalog/core_source_manifest.json")
SUMMER = Path("content/terrain/recovery/summer_source_cells.v1.json")
HISTORY = Path("content/terrain/recovery/historical_authority.v1.json")
OUTPUT = Path("content/catalog/mapping_inventory.v1.json")

def load(root: Path, rel: Path) -> dict:
    return json.loads((root / rel).read_text(encoding="utf-8"))

def build(root: Path) -> dict:
    manifest, summer, history = (load(root, p) for p in (MANIFEST, SUMMER, HISTORY))
    entries = manifest["entries"]
    if len(entries) != manifest["count"]:
        raise ValueError("manifest count does not match entries")
    ids, paths = set(), set()
    assets = []
    for entry in entries:
        aid, path = entry["id"], entry["path"]
        if aid in ids or path in paths:
            raise ValueError(f"duplicate source identity or path: {aid} / {path}")
        ids.add(aid); paths.add(path)
        assets.append({
            "id": aid, "sourcePath": path, "sourceSha256": entry["sha256"],
            "sourceBytes": entry.get("bytes"), "imageSizePx": entry.get("imageSizePx"),
            "pack": entry.get("pack"), "sourceState": entry.get("stage", "source_only"),
            "mappingState": "unreviewed", "certification": "not_certified",
            "semanticRoles": [], "sourceManifest": str(MANIFEST)
        })
    source = summer["source"]
    matching = [a for a in assets if a["sourcePath"] == source["path"]]
    if len(matching) != 1 or matching[0]["sourceSha256"] != source["sha256"]:
        raise ValueError("Summer recovery source does not match canonical manifest hash")
    cells = summer["cells"]
    if len(cells) != source["totalCells"]:
        raise ValueError("Summer cell count mismatch")
    seen = set()
    recovered = []
    for cell in cells:
        xy = tuple(cell["grid"])
        if xy in seen:
            raise ValueError(f"duplicate Summer coordinate: {xy}")
        seen.add(xy)
        recovered.append({
            "id": f'{matching[0]["id"]}#cell:{xy[0]},{xy[1]}',
            "parentAssetId": matching[0]["id"], "grid": list(xy),
            "sourceRectPx": cell["rect"], "rgbaSha256": cell.get("rgbaSha256"),
            "occupancy": cell["occupancy"], "sourceRecoveryState": cell["recoveryState"],
            "semanticRoles": [], "mappingState": "source_recovered_semantics_unreviewed",
            "certification": "not_certified", "sourceEvidence": str(SUMMER)
        })
    nontransparent = sum(c["occupancy"] == "nontransparent" for c in cells)
    if nontransparent != summer["recovery"]["currentPixelReplayNonTransparent"]:
        raise ValueError("Summer recovered non-transparent count mismatch")
    return {
        "schema": "havenwild.asset_mapping.inventory.v1",
        "authority": "derived_source_inventory_not_mapping_certification",
        "inputs": [str(MANIFEST), str(SUMMER), str(HISTORY)],
        "assets": sorted(assets, key=lambda a:a["id"]),
        "recoveredSourceCells": sorted(recovered, key=lambda c:(c["grid"][1],c["grid"][0])),
        "historicalEvidence": history["historicalEvidence"],
        "summary": {
            "canonicalAssets": len(assets), "summerCells": len(cells),
            "summerNonTransparentRecovered": nontransparent,
            "summerTransparentIdentityUnresolved": summer["recovery"]["currentPixelReplayTransparent"],
            "semanticMappingsPromoted": 0, "certifiedMappings": 0,
            "historicalEvidenceRecords": len(history["historicalEvidence"])
        }
    }

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=ROOT)
    ap.add_argument("--check", action="store_true", help="Compare generated inventory without writing")
    args = ap.parse_args()
    try:
        data = build(args.root)
        payload = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
        target = args.root / OUTPUT
        if args.check:
            if not target.is_file() or target.read_text(encoding="utf-8") != payload:
                print("MAPPING INVENTORY: FAIL / missing or stale generated inventory")
                return 1
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(payload, encoding="utf-8")
        print("MAPPING INVENTORY: PASS", data["summary"])
        return 0
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print("MAPPING INVENTORY: FAIL", exc)
        return 2

if __name__ == "__main__":
    raise SystemExit(main())
