#!/usr/bin/env python3
"""No-network, no-source-mutation unit checks for the M2C2 read-only evidence tools."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import zipfile

HERE = Path(__file__).resolve().parent


def load(name: str):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + ".py"))
    if spec is None or spec.loader is None:
        raise RuntimeError(name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    eliza = load("audit_elizawy_concordance")
    tiled = load("inspect_elizawy_tiled")
    tsx = b'''<?xml version="1.0"?><tileset version="1.10" tiledversion="1.10" name="summer" tilewidth="32" tileheight="32" tilecount="4" columns="2" margin="0" spacing="0"><image source="summer.png" width="64" height="64"/><transformations hflip="1" vflip="0" rotate="0" preferuntransformed="1"/><wangsets><wangset name="Grass-Dirt" type="corner" tile="0"><wangcolor name="Grass" color="#00ff00" tile="0"/><wangcolor name="Dirt" color="#aa5500" tile="1"/><wangtile tileid="1" wangid="0,1,0,2,0,1,0,2"/></wangset></wangsets><tile id="1"><animation><frame tileid="1" duration="120"/></animation><objectgroup><object id="1" x="0" y="0" width="32" height="32"/></objectgroup></tile></tileset>'''
    tsj = {"type": "tileset", "name": "winter", "tilewidth": 32, "tileheight": 32, "tilecount": 4, "columns": 2, "image": "winter.png", "imagewidth": 64, "imageheight": 64, "wangsets": [{"name": "Ice", "type": "edge", "colors": [{"name": "Ice"}], "wangtiles": [{"tileid": 0, "wangid": [1, 0, 1, 0, 1, 0, 1, 0]}]}], "tiles": [{"id": 0, "animation": [{"tileid": 0, "duration": 100}], "objectgroup": {"objects": []}}]}
    assert eliza.png_dimensions is not None
    assert tiled.read_tileset("source.tsx", tsx)["wangsets"][0]["assignedTileCount"] == 1
    assert tiled.read_tileset("source.tsx", tsx)["animatedTiles"] == 1
    assert tiled.read_tileset("source.tsx", tsx)["collisionTiles"] == 1
    assert tiled.read_tileset("source.tsj", json.dumps(tsj).encode())["wangsets"][0]["type"] == "edge"
    assert tiled.bad_path("../escape.tsx") and tiled.bad_path("C:/evil.tsx")
    try:
        tiled.read_xml(b'<!DOCTYPE foo [<!ENTITY test SYSTEM "file:///x">]><tileset/>')
        raise AssertionError("DTD should have been rejected")
    except ValueError:
        pass
    with tempfile.TemporaryDirectory(prefix="elizawy-m2c2-") as d:
        path = Path(d) / "derivative.zip"
        with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
            z.writestr("season/summer.tsx", tsx)
            z.writestr("season/winter.tsj", json.dumps(tsj))
            z.writestr("season/summer.png", b"PNG fake input kept only as bytes")
            z.writestr("sample.tmx", b'<map version="1.10"><tileset firstgid="27" source="season/summer.tsx"/></map>')
        report = tiled.inspect(path)
        assert not report["errors"] and report["summary"]["tilesetFiles"] == 2
        assert report["summary"]["wangSets"] == 2 and report["summary"]["wangTileAssignments"] == 2
        assert report["summary"]["mapFiles"] == 1 and report["maps"][0]["tilesetReferences"][0]["firstgid"] == 27
        assert report["summary"]["animatedTileDefinitions"] == 2
        assert report["summary"]["collisionTileDefinitions"] == 2
        assert report["certifiedMappingsAdded"] == 0
        hostile = Path(d) / "hostile.zip"
        with zipfile.ZipFile(hostile, "w") as z:
            z.writestr("../escape.tsx", tsx)
        bad = tiled.inspect(hostile)
        assert bad["errors"] and not (Path(d).parent / "escape.tsx").exists()
    print("M2C2 SELFTEST: PASS / TSX, TSJ, Wang sets, animations, collision, traversal/DTD rejection, zero promotions")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
