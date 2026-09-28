#!/usr/bin/env python3
"""Read-only inspection of a derivative LPC Revised Tiled ZIP or TSX/TSJ directory.

No extraction, map promotion, source-file mutation, collision authority or art copying.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import sys
import xml.etree.ElementTree as ET
import zipfile

MAX_MEMBER = 12 * 1024 * 1024
MAX_TOTAL = 128 * 1024 * 1024
MAX_FILES = 2000
EXTS = {".tsx", ".tsj", ".tmx", ".tmj", ".png", ".txt", ".md", ".json"}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def bad_path(name: str) -> bool:
    p = PurePosixPath(name.replace("\\", "/"))
    return p.is_absolute() or any(seg in ("..", "") for seg in p.parts) or ":" in name or "\x00" in name


def read_xml(data: bytes) -> ET.Element:
    # ElementTree does not fetch external DTDs, but reject DTD/entity input anyway.
    if b"<!DOCTYPE" in data.upper() or b"<!ENTITY" in data.upper():
        raise ValueError("DTD/entity markup not permitted")
    return ET.fromstring(data)


def get_text(node: ET.Element | None, key: str, fallback: str = "") -> str:
    return node.attrib.get(key, fallback) if node is not None else fallback


def read_tileset(name: str, data: bytes) -> dict:
    result: dict = {"path": name, "sha256": sha(data), "parseState": "parsed"}
    if name.lower().endswith(".tsx"):
        root = read_xml(data)
        if root.tag != "tileset":
            raise ValueError("TSX root must be <tileset>")
        image = root.find("image")
        result.update({"name": root.get("name"), "tileWidth": int(root.get("tilewidth", "0")), "tileHeight": int(root.get("tileheight", "0")), "tileCount": int(root.get("tilecount", "0")), "columns": int(root.get("columns", "0")), "spacing": int(root.get("spacing", "0")), "margin": int(root.get("margin", "0")), "image": get_text(image, "source"), "imageSize": [int(get_text(image, "width", "0")), int(get_text(image, "height", "0"))], "transformations": root.find("transformations").attrib if root.find("transformations") is not None else {}, "tileOffset": root.find("tileoffset").attrib if root.find("tileoffset") is not None else {}, "wangsets": [], "legacyTerrainSets": 0, "animatedTiles": 0, "collisionTiles": 0})
        for wangset in root.findall("wangsets/wangset"):
            tiles = wangset.findall("wangtile")
            result["wangsets"].append({"name": wangset.get("name"), "type": wangset.get("type", "legacy-unspecified"), "colors": [x.get("name", "") for x in wangset.findall("wangcolor")], "assignedTileCount": len(tiles), "tileIds": [int(x.get("tileid", "-1")) for x in tiles], "wangIds": [x.get("wangid", "") for x in tiles]})
        result["legacyTerrainSets"] = len(root.findall("terraintypes/terrain"))
        tiles = root.findall("tile")
        result["animatedTiles"] = sum(1 for t in tiles if t.find("animation") is not None)
        result["collisionTiles"] = sum(1 for t in tiles if t.find("objectgroup") is not None)
    else:
        doc = json.loads(data.decode("utf-8-sig"))
        if doc.get("type") != "tileset":
            raise ValueError("TSJ root type must be tileset")
        image = doc.get("image", "")
        result.update({"name": doc.get("name"), "tileWidth": doc.get("tilewidth", 0), "tileHeight": doc.get("tileheight", 0), "tileCount": doc.get("tilecount", 0), "columns": doc.get("columns", 0), "spacing": doc.get("spacing", 0), "margin": doc.get("margin", 0), "image": image, "imageSize": [doc.get("imagewidth", 0), doc.get("imageheight", 0)], "transformations": doc.get("transformations", {}), "tileOffset": doc.get("tileoffset", {}), "wangsets": [], "legacyTerrainSets": len(doc.get("terrains", [])), "animatedTiles": 0, "collisionTiles": 0})
        for w in doc.get("wangsets", []):
            tiles = w.get("wangtiles", [])
            result["wangsets"].append({"name": w.get("name"), "type": w.get("type", "legacy-unspecified"), "colors": [x.get("name", "") for x in w.get("colors", [])], "assignedTileCount": len(tiles), "tileIds": [x.get("tileid") for x in tiles], "wangIds": [x.get("wangid", []) for x in tiles]})
        for t in doc.get("tiles", []):
            result["animatedTiles"] += bool(t.get("animation"))
            result["collisionTiles"] += bool(t.get("objectgroup"))
    return result


def inspect(path: Path) -> dict:
    errors, warnings, file_rows, tilesets, maps = [], [], [], [], []
    if path.is_file() and path.suffix.lower() == ".zip":
        with zipfile.ZipFile(path) as z:
            members = z.infolist()
            if len(members) > MAX_FILES:
                raise ValueError("ZIP exceeds safe file count limit")
            if sum(m.file_size for m in members) > MAX_TOTAL:
                raise ValueError("ZIP exceeds safe uncompressed size limit")
            candidates = []
            for member in members:
                if member.is_dir():
                    continue
                if bad_path(member.filename) or member.file_size > MAX_MEMBER or member.flag_bits & 1 or ((member.external_attr >> 16) & 0o170000) == 0o120000:
                    errors.append(f"Unsafe or unsupported ZIP member: {member.filename}")
                    continue
                if Path(member.filename).suffix.lower() in EXTS:
                    candidates.append((member.filename, z.read(member)))
    elif path.is_dir():
        candidates = []
        for f in path.rglob("*"):
            if not f.is_file() or f.is_symlink() or f.suffix.lower() not in EXTS:
                continue
            if f.stat().st_size > MAX_MEMBER or len(candidates) > MAX_FILES:
                raise ValueError("Input tree exceeds size/member limits")
            candidates.append((f.relative_to(path).as_posix(), f.read_bytes()))
    else:
        raise ValueError("Provide a ZIP or directory containing actual TSX/TSJ files")
    for name, data in candidates:
        ext = Path(name).suffix.lower()
        file_rows.append({"path": name, "bytes": len(data), "sha256": sha(data), "type": ext})
        if ext in (".tsx", ".tsj"):
            try:
                tilesets.append(read_tileset(name, data))
            except (ValueError, ET.ParseError, TypeError, KeyError, UnicodeError, json.JSONDecodeError) as exc:
                errors.append(f"{name}: {exc}")
        elif ext in (".tmx", ".tmj"):
            try:
                if ext == ".tmx":
                    node = read_xml(data)
                    if node.tag != "map":
                        raise ValueError("TMX root must be <map>")
                    refs = [{"source": t.get("source", ""), "firstgid": int(t.get("firstgid", "0")), "name": t.get("name", "")} for t in node.findall("tileset")]
                else:
                    doc = json.loads(data.decode("utf-8-sig"))
                    if doc.get("type") != "map":
                        raise ValueError("TMJ root type must be map")
                    refs = [{"source": t.get("source", ""), "firstgid": t.get("firstgid", 0), "name": t.get("name", "")} for t in doc.get("tilesets", [])]
                maps.append({"path": name, "tilesetReferences": refs, "note": "firstgid is map-specific and must not be used as a source cell or native tile ID"})
            except (ValueError, ET.ParseError, TypeError, KeyError, UnicodeError, json.JSONDecodeError) as exc:
                errors.append(f"{name}: {exc}")
    if not tilesets:
        errors.append("No parsed TSX/TSJ files: cannot claim derivative terrain mapping evidence")
    for t in tilesets:
        if (t["tileWidth"], t["tileHeight"]) != (32, 32):
            warnings.append(f"Non-32px tileset {t['path']}: {t['tileWidth']}x{t['tileHeight']}")
        if not t["wangsets"] and t["legacyTerrainSets"] == 0:
            warnings.append(f"No terrain/Wang metadata on {t['path']}")
    return {"schema": "havenwild.elizawy.tiled_derivative_inspection.v1", "authority": "external_derivative_evidence_only_zero_promotions", "input": str(path), "inputSha256": hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None, "files": file_rows, "tilesets": tilesets, "maps": maps, "summary": {"mapFiles": len(maps), "fileCount": len(file_rows), "pngFiles": sum(r["type"] == ".png" for r in file_rows), "tilesetFiles": len(tilesets), "wangSets": sum(len(t["wangsets"]) for t in tilesets), "wangTileAssignments": sum(sum(w["assignedTileCount"] for w in t["wangsets"]) for t in tilesets), "legacyTerrainTypes": sum(t["legacyTerrainSets"] for t in tilesets), "animatedTileDefinitions": sum(t["animatedTiles"] for t in tilesets), "collisionTileDefinitions": sum(t["collisionTiles"] for t in tilesets)}, "warnings": warnings, "errors": errors, "certifiedMappingsAdded": 0}


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("input", type=Path, help="Derivative package ZIP or unpacked directory")
    p.add_argument("--output", type=Path, help="JSON report (default adjacent to supplied archive)")
    a = p.parse_args(argv)
    try:
        report = inspect(a.input.resolve())
        output = a.output or a.input.with_name(a.input.stem + ".tiled_inspection.v1.json")
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        print(f"TILED EVIDENCE: {'PASS' if not report['errors'] else 'FAIL'} / {report['summary']['tilesetFiles']} tilesets / {report['summary']['wangSets']} Wang sets / {report['summary']['wangTileAssignments']} assignments / 0 promoted")
        print(f"REPORT: {output}")
        return 0 if not report["errors"] else 2
    except (ValueError, OSError, zipfile.BadZipFile, RuntimeError) as exc:
        print(f"TILED EVIDENCE: FAIL / {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
