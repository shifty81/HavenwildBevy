#!/usr/bin/env python3
"""Offline structural and synthetic tests for M2D094B."""
from __future__ import annotations

import ast
import importlib.util
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / "src/tiled_terrain.rs"
HYDRATE = ROOT / "tools/hydrate_jaidyn_tiled.py"
VERIFY = ROOT / "tools/verify_jaidyn_tiled_runtime.py"


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def main() -> int:
    errors: list[str] = []
    for path in (RUST, HYDRATE, VERIFY):
        if not path.is_file():
            errors.append(f"missing {path}")
            continue
        if path.suffix == ".py":
            try:
                ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            except Exception as exc:
                errors.append(f"syntax error {path}: {exc}")

    if HYDRATE.is_file():
        text = HYDRATE.read_text(encoding="utf-8")
        for token in (
            "f0236c16b272a7328dad66772e4f971513bc508cb69a56c60d736264a3a5b43a",
            "assets/elizawy/_jaidyn_tiled",
            "MAX_ARCHIVE_BYTES",
            "User-Agent",
            "havenwild.external.jaidyn_tiled_runtime.v2",
            "Original LPC Revised contributors",
        ):
            if token not in text:
                errors.append(f"hydrator missing contract token {token}")

    if RUST.is_file():
        text = RUST.read_text(encoding="utf-8")
        # Rust raw strings using one # terminate on the byte sequence "#; XML color
        # attributes such as color="#00ff00" therefore require at least r##"..."##.
        if 'let xml = r#"' in text:
            errors.append('Rust fixture uses r# raw string; XML color attributes can terminate it early. Use r## delimiters.')
        if text.count('let xml = r##"') < 2:
            errors.append('Rust parser fixtures are not using the required r## raw-string delimiters')

        for token in (
            "use tiled::{Loader, WangSetType};",
            "load_tsx_tileset",
            "wang.wang_id.0",
            "resolve_corner_materials",
            "[0, ne, 0, se, 0, sw, 0, nw]",
            "choose_exact_uniform",
            "validate_assignment_bounds",
            "real_rs_tiled_parser_path_is_exercised",
            "out_of_atlas_wang_assignment_is_rejected",
            "canonical_wang_set_name",
            "source_name",
        ):
            if token not in text:
                errors.append(f"Rust authority missing contract token {token}")

    if VERIFY.is_file():
        text = VERIFY.read_text(encoding="utf-8")
        for token in (
            "probabilitySensitiveGroups",
            "transformations",
            "Summer Terrain has no authored pure-fill Wang tile",
            "numeric Wang topology differs from Summer",
            "Autumn remains partial/old",
            "M2C3_JAIDYNREIMAN_TILED_CONCORDANCE.v1.json",
            "external_derivative_evidence_only",
            "canonical_wang_set_name",
        ):
            if token not in text:
                errors.append(f"acceptance verifier missing contract token {token}")

        try:
            verify = load_module("m2d094b_verify", VERIFY)
            with tempfile.TemporaryDirectory(prefix="m2d094b-selftest-") as td:
                good = Path(td) / "good.tsx"
                good.write_text(
                    '''<?xml version="1.0"?>
<tileset name="fixture" tilewidth="32" tileheight="32" tilecount="4" columns="2">
 <image source="fixture.png" width="64" height="64"/>
 <wangsets><wangset name="Winter Terrain" type="corner">
  <wangcolor name="Grass"/><wangcolor name="Water"/>
  <wangtile tileid="1" wangid="0,1,0,2,0,1,0,1"/>
 </wangset></wangsets>
 <tile id="1" probability="0.25"><animation><frame tileid="1" duration="100"/></animation><objectgroup><object id="1" x="0" y="0" width="32" height="32"/></objectgroup></tile>
</tileset>''',
                    encoding="utf-8",
                )
                parsed = verify.parse_tsx(good)
                assert parsed["sets"]["Terrain"]["assignmentCount"] == 1
                assert parsed["sets"]["Terrain"]["sourceName"] == "Winter Terrain"
                assert parsed["animatedTiles"] == 1
                assert parsed["collisionTiles"] == 1
                assert parsed["transformations"]["rotate"] is False

                bad = Path(td) / "bad.tsx"
                bad.write_text(
                    '''<?xml version="1.0"?>
<tileset name="fixture" tilewidth="32" tileheight="32" tilecount="4" columns="2">
 <image source="fixture.png" width="64" height="64"/>
 <wangsets><wangset name="Terrain" type="corner"><wangcolor name="Grass"/><wangtile tileid="99" wangid="0,1,0,1,0,1,0,1"/></wangset></wangsets>
</tileset>''',
                    encoding="utf-8",
                )
                try:
                    verify.parse_tsx(bad)
                    errors.append("synthetic out-of-atlas Wang assignment was not rejected")
                except ValueError:
                    pass
        except Exception as exc:
            errors.append(f"acceptance verifier synthetic test failed: {exc}")


    cargo_toml = ROOT / "Cargo.toml"
    if cargo_toml.is_file():
        cargo_text = cargo_toml.read_text(encoding="utf-8")
        if 'tiled = { version = "=0.16.0", default-features = false }' not in cargo_text:
            errors.append("installed Cargo.toml is missing the pinned no-default-features tiled 0.16 dependency")

    src_root = ROOT / "src"
    module_owners = []
    if src_root.is_dir():
        for rs in src_root.rglob("*.rs"):
            try:
                if "mod tiled_terrain;" in rs.read_text(encoding="utf-8"):
                    module_owners.append(rs)
            except OSError:
                pass
    # In payload/preflight mode there is intentionally no Cargo.toml/module owner yet.
    # Once installed into a project, Cargo.toml exists and the module declaration is mandatory.
    if cargo_toml.is_file() and not module_owners:
        errors.append("installed Rust source tree has no mod tiled_terrain declaration")

    evidence = ROOT / "docs/patch/evidence/M2C3_JAIDYNREIMAN_TILED_CONCORDANCE.v1.json"
    if evidence.is_file():
        import json
        try:
            data = json.loads(evidence.read_text(encoding="utf-8"))
            if data.get("archiveSha256") != "f0236c16b272a7328dad66772e4f971513bc508cb69a56c60d736264a3a5b43a":
                errors.append("local M2C3 evidence archive SHA does not match M2D094B")
            if data.get("status") != "external_derivative_evidence_only":
                errors.append("local M2C3 evidence status contract changed unexpectedly")
            verify = load_module("m2d094b_verify_evidence", VERIFY)
            for season, expected in verify.EXPECTED.items():
                actual = data.get("seasonProfiles", {}).get(season, {})
                if actual.get("tsxFile") != expected["tsx"] or actual.get("imageFile") != expected["png"]:
                    errors.append(f"local M2C3 {season} filenames disagree with M2D094B")
                if actual.get("animatedTiles") != expected["animated"] or actual.get("collisionObjectGroups") != expected["collision"]:
                    errors.append(f"local M2C3 {season} animation/collision counts disagree with M2D094B")
                for set_name, (kind, colors, count) in expected["sets"].items():
                    aset = actual.get("wangSets", {}).get(set_name, {})
                    if aset.get("type") != kind or aset.get("colors") != colors or aset.get("assignments") != count:
                        errors.append(f"local M2C3 {season}/{set_name} profile disagrees with M2D094B")
            bp = data.get("buildingProfile", {})
            if bp.get("animatedTiles") != verify.BUILDINGS["animated"] or bp.get("collisionObjectGroups") != verify.BUILDINGS["collision"]:
                errors.append("local M2C3 building animation/collision profile disagrees with M2D094B")
            for set_name, (kind, color_count, assignment_count) in verify.BUILDINGS["sets"].items():
                aset = bp.get("wangSets", {}).get(set_name, {})
                if aset.get("type") != kind or len(aset.get("colors", [])) != color_count or aset.get("assignments") != assignment_count:
                    errors.append(f"local M2C3 buildings/{set_name} profile disagrees with M2D094B")
        except Exception as exc:
            errors.append(f"local M2C3 evidence parse failed: {exc}")

    if errors:
        print("M2D094B SELFTEST: FAIL")
        for error in errors:
            print(" -", error)
        return 2
    print("M2D094B SELFTEST: PASS / parser boundary, hydration integrity, real-TSX acceptance gates, atlas bounds")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
