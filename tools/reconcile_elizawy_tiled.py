#!/usr/bin/env python3
"""M2C3: inspect an external LPC Revised Tiled derivative against ElizaWy evidence.

Read-only: never extracts source files, copies artwork, promotes tiles, or edits source.
The external derivative and native ElizaWy art have distinct provenance and IDs.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import io
import json
import struct
from pathlib import Path
import sys
import zipfile

from inspect_elizawy_tiled import inspect as inspect_archive, read_tileset, read_xml

SEASONS = ("summer", "spring", "autumn", "winter")
KNOWN_SHA = "f0236c16b272a7328dad66772e4f971513bc508cb69a56c60d736264a3a5b43a"
SUMMER_MAP_SHA = "6e6a64725208f6ec18b6f0a203ca25701db87c1facf3941e30077ac1f88c5c3e"
DERIVATIVE = "https://opengameart.org/content/lpc-revised-fully-configured-4-seasons-tilesets-for-tiled-map-editor"
UPSTREAM = "https://github.com/ElizaWy/LPC/tree/f07f7f5892e67c932c68f70bb04472f2c64e46bc"


def sha(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def season_from_name(n: str) -> str | None:
    for s in SEASONS:
        if n in (f"lpc-tileset-terrain-{s}.tsx", f"lpc-tileset-terrain-{s}-old.tsx"):
            return s
    return None


def get_map(root: Path, explicit: Path | None) -> tuple[dict | None, str]:
    if explicit:
        p = explicit
    else:
        idx = root / ".forgepy/recovered_mapping/index.json"
        if not idx.is_file():
            return None, "recovery_index_unavailable"
        records = json.loads(idx.read_text(encoding="utf-8")).get("records", [])
        row = next((r for r in records if r.get("relativePath", "").replace("\\", "/").endswith("/lpc_terrain_summer_complete_map_32.json") and r.get("sha256") == SUMMER_MAP_SHA), None)
        if not row:
            return None, "historical_map_not_indexed"
        p = root / Path(row["localPath"].replace("\\", "/"))
    if not p.is_file():
        return None, "historical_map_file_missing"
    raw = p.read_bytes()
    if sha(raw) != SUMMER_MAP_SHA:
        return None, "historical_map_hash_mismatch"
    doc = json.loads(raw)
    if doc.get("grid") != [16, 26] or doc.get("cellSize") != 32:
        return None, "historical_map_schema_or_dimensions_mismatch"
    return doc, "sha256_verified"


def get_manifest(root: Path) -> dict:
    p = root / "content/catalog/core_source_manifest.json"
    if not p.is_file():
        return {}
    doc = json.loads(p.read_text(encoding="utf-8"))
    # Keep the existing source authority; never infer it from derivative art.
    def recurse(n):
        if isinstance(n, list):
            for x in n: yield from recurse(x)
        elif isinstance(n, dict):
            if n.get("path", "").replace("\\", "/").endswith(".png") and n.get("sha256"):
                yield n
            for k, v in n.items():
                if isinstance(v, (list, dict)): yield from recurse(v)
    return {x["path"].replace("\\", "/"): x for x in recurse(doc)}


def png_size(data: bytes) -> list[int]:
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise ValueError("Expected a PNG with valid IHDR header")
    w, h = struct.unpack(">II", data[16:24])
    return [w, h]


def wang_data(blob: bytes) -> dict:
    rt = read_xml(blob)
    result = {}
    for w in rt.findall("wangsets/wangset"):
        name = w.get("name", "")
        normalized = "Terrain" if name.lower() in {f"{s} terrain" for s in SEASONS} else name
        colors = [x.get("name", "") for x in w.findall("wangcolor")]
        assignments = {}
        for t in w.findall("wangtile"):
            tileid = int(t.attrib["tileid"])
            wangid = tuple(int(x) for x in t.attrib.get("wangid", "").split(","))
            if len(wangid) != 8 or min(wangid) < 0 or max(wangid) > len(colors):
                raise ValueError(f"Invalid Wang signature in {name} tile {tileid}")
            if tileid in assignments:
                raise ValueError(f"Duplicate Wang tile ID in {name}: {tileid}")
            assignments[tileid] = list(wangid)
        if normalized in result:
            raise ValueError(f"Ambiguous duplicate Wang set: {normalized}")
        result[normalized] = {"type": w.get("type"), "colors": colors, "assignments": assignments}
    return result


def tile_definitions(data: bytes) -> dict:
    root = read_xml(data)
    defs = {}
    for t in root.findall("tile"):
        tileid = int(t.attrib["id"])
        anim = t.find("animation")
        group = t.find("objectgroup")
        defs[tileid] = {
            "animated": anim is not None,
            "frames": [[int(f.get("tileid", "-1")), int(f.get("duration", "0"))] for f in anim.findall("frame")] if anim is not None else [],
            "hasCollision": group is not None,
            "objectTypes": sorted(set(x.get("type", "") for x in group.findall("object"))) if group is not None else [],
            "tileType": t.get("type"),
        }
    return defs


def pixel_compare(z: zipfile.ZipFile, roots: Path | None, manifest: dict, source_map: dict | None, warnings: list[str]) -> dict:
    """Optional deterministic RGBA crop concordance. Pillow is an optional analysis dependency.

    A byte/rgba match is address evidence, never a semantic/collision certification.
    """
    if roots is None:
        return {"state": "not_requested", "matches": {}, "certifiedRoles": 0}
    try:
        from PIL import Image
    except ImportError:
        warnings.append("Pixel comparison requested but Pillow unavailable; TSX/Wang metadata analysis remains usable")
        return {"state": "pillow_unavailable", "matches": {}, "certifiedRoles": 0}
    by_coord = {tuple(c["cell"]): c for c in source_map.get("cells", [])} if source_map else {}
    results = {}
    for season in SEASONS:
        original_path = roots / "Terrain" / f"terrain_{season}.png"
        if not original_path.is_file():
            results[season] = {"state": "original_png_missing"}
            continue
        original_bytes = original_path.read_bytes()
        expected = manifest.get(f"Terrain/terrain_{season}.png", {}).get("sha256")
        if not expected:
            results[season] = {"state": "original_manifest_hash_missing"}
            continue
        if sha(original_bytes) != expected:
            results[season] = {"state": "original_source_hash_mismatch", "expectedSha256": expected, "actualSha256": sha(original_bytes)}
            continue
        src = Image.open(io.BytesIO(original_bytes)).convert("RGBA")
        der = Image.open(io.BytesIO(z.read(f"lpc-tileset-terrain-{season}.png"))).convert("RGBA")
        if src.size != (512, 832) or der.size != (2048, 2048):
            results[season] = {"state": "unexpected_png_dimensions", "originalSize": list(src.size), "derivativeSize": list(der.size)}
            continue
        # Skip fully transparent source cells; matching their invisible RGB payload is not useful.
        source_hashes = defaultdict(list)
        for y in range(26):
            for x in range(16):
                sub = src.crop((x*32, y*32, x*32+32, y*32+32))
                if sub.getchannel("A").getbbox():
                    source_hashes[sha(sub.tobytes())].append([x, y])
        counts = Counter()
        examples = []
        matching_pairs = 0
        derivative_addresses_with_matches = 0
        semantic_candidates = 0
        for tid in range(4096):
            x,y=tid%64,tid//64
            sub=der.crop((x*32,y*32,x*32+32,y*32+32))
            if not sub.getchannel("A").getbbox():
                continue
            candidate = source_hashes.get(sha(sub.tobytes()), [])
            if not candidate:
                counts["unmatched"] += 1
                continue
            derivative_addresses_with_matches += 1
            matching_pairs += len(candidate)
            key = "unique" if len(candidate)==1 else "ambiguous"
            counts[key] += 1
            if len(examples)<120:
                roles=[{"sourceCell": q, "groupId": by_coord.get(tuple(q), {}).get("groupId"), "role": by_coord.get(tuple(q), {}).get("role")} for q in candidate]
                examples.append({"derivativeTileId": tid, "derivativeCell": [x,y], "matchType": key, "originalCandidates": roles})
            if any(by_coord.get(tuple(q), {}).get("groupId") for q in candidate): semantic_candidates+=1
        results[season]={"state":"original_sha_verified_rgba32_exact_match", "sourceNontransparentCells":sum(len(v) for v in source_hashes.values()),"derivativeMatchingCells":derivative_addresses_with_matches,"uniqueCandidateCells":counts["unique"],"ambiguousCandidateCells":counts["ambiguous"],"unmatchedDerivativeOccupiedCells":counts["unmatched"],"totalMatchingPairs":matching_pairs,"candidateWithHistoricalSummerRole":semantic_candidates,"sample":examples,"certifiedRoles":0}
    return {"state": "optional_original_comparison", "matches": results, "certifiedRoles": 0}


def reconcile(archive: Path, root: Path, original_root: Path | None = None, summer_map: Path | None = None, strict_archive_sha: bool = False) -> dict:
    base = inspect_archive(archive)
    errors = list(base["errors"])
    warnings = list(base["warnings"])
    actual_sha = sha(archive.read_bytes())
    if strict_archive_sha and actual_sha != KNOWN_SHA:
        errors.append("Supplied archive SHA differs from audited 2024 package; independent review required")
    profile={"schema":"havenwild.elizawy.external_tiled_concordance.v1", "status":"external_derivative_evidence_only", "originalSourceRevision":"f07f7f5892e67c932c68f70bb04472f2c64e46bc", "externalSource":DERIVATIVE, "upstreamSource":UPSTREAM, "archiveSha256":actual_sha, "matchesAuditedUpload":actual_sha==KNOWN_SHA, "seasonProfiles":{}, "buildingProfile":{}, "comparison":{}, "pixelConcordance":{}, "historicalMapState":"", "warnings":warnings,"errors":errors,"certifiedMappingsAdded":0,"runtimeChanged":False,"sourceFilesMutated":False}
    if errors: return profile
    name_by_season={}
    with zipfile.ZipFile(archive) as z:
        names=set(z.namelist())
        for name in names:
            s=season_from_name(name)
            if s:
                if s in name_by_season:
                    errors.append(f"Duplicate seasonal TSX: {s}")
                name_by_season[s]=name
        for s in SEASONS:
            if s not in name_by_season:
                errors.append(f"Missing TSX for season: {s}")
        if errors:return profile
        definitions={}
        for s in SEASONS:
            name=name_by_season[s]
            blob=z.read(name)
            t=read_tileset(name,blob)
            img=t["image"]
            expected=f"lpc-tileset-terrain-{s}.png"
            if img!=expected or img not in names: errors.append(f"TSX/image mismatch or missing image: {name}")
            elif png_size(z.read(img)) != t["imageSize"]:errors.append(f"TSX/image declared dimension mismatch: {name}")
            w=wang_data(blob); definitions[s] = w
            tiles=tile_definitions(blob)
            if any(k < 0 or k >= t["tileCount"] for k in tiles): errors.append(f"Out-of-range tile definition: {name}")
            for setname, setval in w.items():
                if any(k < 0 or k >= t["tileCount"] for k in setval["assignments"]):errors.append(f"Out-of-range Wang assignment: {name}/{setname}")
            profile["seasonProfiles"][s]={"tsxFile":name,"tsxSha256":sha(blob),"imageFile":img,"imageSha256":sha(z.read(img)) if img in names else None,"tileSize":[t["tileWidth"],t["tileHeight"]],"atlasSize":t["imageSize"],"wangSets":{k:{"type":v["type"],"colors":v["colors"],"assignments":len(v["assignments"])} for k,v in w.items()},"animatedTiles":sum(v["animated"] for v in tiles.values()),"collisionObjectGroups":sum(v["hasCollision"] for v in tiles.values()),"tileTypes":dict(Counter(str(v["tileType"] or "unspecified") for v in tiles.values()))}
            if t["imageSize"] != [2048,2048] or t["tileWidth"]!=32 or t["tileHeight"]!=32 or t["tileCount"]!=4096 or t["columns"]!=64:errors.append(f"Unexpected external grid: {name}")
            if name.endswith("-old.tsx"):warnings.append(f"{s}: only legacy '-old.tsx' definition supplied; do not assume current/full seasonal metadata")
        if "lpc-tileset-buildings.tsx" in names:
            name="lpc-tileset-buildings.tsx";blob=z.read(name);t=read_tileset(name,blob)
            if t["image"]!="lpc-tileset-buildings.png" or t["image"] not in names:
                errors.append("Missing/mismatched buildings image reference")
            elif png_size(z.read(t["image"])) != t["imageSize"]:
                errors.append("Building TSX/image dimension mismatch")
            profile["buildingProfile"]={"source":name,"sourceSha256":sha(blob),"imageSha256":sha(z.read(t["image"])) if t["image"] in names else None,"wangSets":{w["name"]:{"type":w["type"],"colors":w["colors"],"assignments":w["assignedTileCount"]} for w in t["wangsets"]},"animatedTiles":t["animatedTiles"],"collisionObjectGroups":t["collisionTiles"],"policy":"architecture_reference_only_not_terrain_authority"}
        for s in ('spring','autumn','winter'):
            group={}
            for setname, b in definitions['summer'].items():
                o=definitions[s].get(setname)
                if o is None:
                    group[setname]={"missingEntireSet":True,"missingIds":sorted(b["assignments"]),"missingCount":len(b["assignments"]),"extraIds":[],"extraCount":0,"changedCount":0}
                    continue
                a,bids=o["assignments"],b["assignments"]
                shared=a.keys() & bids.keys()
                changed=sorted(k for k in shared if a[k]!=bids[k])
                missing=sorted(bids.keys()-a.keys());extra=sorted(a.keys()-bids.keys())
                group[setname]={"missingEntireSet":False,"colorNames":o["colors"],"summerColorNames":b["colors"],"semanticNamesIdentical":o["colors"]==b["colors"],"commonTileIdCount":len(shared),"missingIds":missing,"missingCount":len(missing),"extraIds":extra,"extraCount":len(extra),"changedIds":changed,"changedCount":len(changed)}
            for setname,o in definitions[s].items():
                if setname not in definitions['summer']:
                    group[setname]={"additionalEntireSet":True,"extraCount":len(o['assignments'])}
            profile["comparison"][s]=group
        if profile['comparison']['autumn'].get('Terrain',{}).get('missingCount',0):
            warnings.append("Autumn terrain Wang coverage differs from Summer; do not manufacture missing assignments")
        if profile['comparison']['autumn'].get('Fences',{}).get('missingEntireSet'):
            warnings.append("Autumn fence Wang set missing, although its PNG exists")
        if not profile['comparison']['winter'].get('Terrain',{}).get('semanticNamesIdentical'):
            warnings.append("Winter numeric Wang IDs match Summer, but several material names differ; never infer semantic equality from tile IDs")
        source_map, state=get_map(root, summer_map)
        profile['historicalMapState']=state
        profile['pixelConcordance']=pixel_compare(z,original_root,get_manifest(root),source_map,warnings)
    return profile


def main(argv: list[str] | None=None) -> int:
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('archive',type=Path,help='Original third-party ZIP, kept outside PCC patch')
    ap.add_argument('--root',type=Path,default=Path(__file__).resolve().parents[1])
    ap.add_argument('--original-root',type=Path,help='Verified original LPC source folder containing Terrain/terrain_*.png; enables optional exact RGBA crop matching if Pillow is installed')
    ap.add_argument('--summer-map',type=Path,help='Optional actual recovered Summer map JSON; fixed SHA required')
    ap.add_argument('--output',type=Path,help='Output report; default .forgepy/recovered_mapping/external_tiled_concordance.v1.json')
    ap.add_argument('--strict-archive-sha',action='store_true',help='Require the exact derivative ZIP audited in chat')
    args=ap.parse_args(argv)
    try:
        report=reconcile(args.archive.resolve(),args.root.resolve(),args.original_root.resolve() if args.original_root else None,args.summer_map.resolve() if args.summer_map else None,args.strict_archive_sha)
        out=args.output or args.root/ '.forgepy/recovered_mapping/external_tiled_concordance.v1.json'
        out.parent.mkdir(parents=True,exist_ok=True)
        out.write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
        print(f"ELIZAWY EXTERNAL CONCORDANCE: {'FAIL' if report['errors'] else 'PASS'} / seasons {len(report['seasonProfiles'])}/4 / certified 0 / original art {report['pixelConcordance'].get('state','not_loaded')}")
        print(f"REPORT: {out}")
        return 2 if report['errors'] else 0
    except (ValueError, OSError, KeyError, TypeError, zipfile.BadZipFile, json.JSONDecodeError) as exc:
        print(f'ELIZAWY EXTERNAL CONCORDANCE: FAIL / {exc}',file=sys.stderr)
        return 2

if __name__=='__main__':raise SystemExit(main())
