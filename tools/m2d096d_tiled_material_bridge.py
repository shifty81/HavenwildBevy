#!/usr/bin/env python3
"""Audit complete Summer Tiled-material accounting without auto-promoting semantics."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
import tempfile
import xml.etree.ElementTree as ET

SCHEMA = "havenwild.tiled_material_bridge.v1"
REGISTRY = Path("content/terrain/evidence/tiled_material_bridge.v1.json")
DEFAULT_TSX = Path("assets/elizawy/_jaidyn_tiled/lpc-tileset-terrain-summer.tsx")
ALLOWED_STATES = {"live_certified", "deferred_explicit", "evidence_blocked"}


def _terrain_materials(tsx: Path) -> list[str]:
    root = ET.parse(tsx).getroot()
    for wangset in root.findall("./wangsets/wangset"):
        name = (wangset.get("name") or "").strip().lower()
        if (wangset.get("type") or "").strip().lower() != "corner":
            continue
        colors = [(c.get("name") or "").strip() for c in wangset.findall("wangcolor")]
        if "Grass" in colors and "Sand" in colors and "Shallow Water" in colors:
            return colors
    raise ValueError(f"No Summer Terrain corner Wang set in {tsx}")


def audit(root: Path, tsx: Path | None = None) -> dict:
    registry_path = root / REGISTRY
    reg = json.loads(registry_path.read_text(encoding="utf-8"))
    if reg.get("schema") != SCHEMA:
        raise ValueError("material bridge schema drift")
    entries = reg.get("tiledMaterials") or []
    names = [e.get("material") for e in entries]
    if len(names) != len(set(names)):
        raise ValueError("duplicate Tiled material ledger entries")
    for entry in entries:
        if entry.get("state") not in ALLOWED_STATES:
            raise ValueError(f"invalid state for {entry.get('material')}: {entry.get('state')}")
        if entry.get("state") == "live_certified" and not entry.get("havenwildRole"):
            raise ValueError(f"live material lacks Havenwild role: {entry.get('material')}")
    roles = reg.get("havenwildRoles") or []
    role_names = [e.get("role") for e in roles]
    if len(role_names) != len(set(role_names)):
        raise ValueError("duplicate Havenwild role ledger entries")
    blocked = {e.get("role") for e in roles if e.get("state") == "evidence_blocked"}
    if not {"WetSand", "PebblePath"}.issubset(blocked):
        raise ValueError("WetSand/PebblePath must remain explicitly evidence-blocked")

    authored = None
    if tsx is not None:
        authored = _terrain_materials(tsx)
        ledger = set(names)
        missing = [name for name in authored if name not in ledger]
        extra = [name for name in names if name not in set(authored)]
        if missing or extra:
            raise ValueError(f"Tiled material ledger mismatch: missing={missing}, extra={extra}")

    counts = {state: sum(1 for e in entries if e.get("state") == state) for state in ALLOWED_STATES}
    return {
        "schema": "havenwild.m2d096d.tiled_material_bridge_audit.v1",
        "status": "PASS_EXPLICIT_COMPLETE_ACCOUNTING",
        "authoredMaterials": authored or names,
        "ledgerMaterialCount": len(entries),
        "stateCounts": counts,
        "liveMappings": {e["havenwildRole"]: e["material"] for e in entries if e.get("state") == "live_certified"},
        "blockedRoles": sorted(blocked),
    }


def selftest() -> int:
    with tempfile.TemporaryDirectory(prefix="havenwild-m2d096d-") as td:
        td = Path(td)
        (td / REGISTRY.parent).mkdir(parents=True)
        source = Path(__file__).resolve().parents[1] / REGISTRY
        (td / REGISTRY).write_bytes(source.read_bytes())
        colors = [
            "Grass", "Sand", "Dirt", "Light Till", "Deep Till", "Shallow Water",
            "Deep Water", "Melted Ice", "Deep Sand", "Mountain", "Grassy Mountain", "Vine"
        ]
        wangcolors = "\n".join(
            f'<wangcolor name="{name}" color="#ffffff" tile="0" probability="1"/>' for name in colors
        )
        tsx = td / "fixture.tsx"
        tsx.write_text(
            f'''<?xml version="1.0" encoding="UTF-8"?>\n<tileset name="fixture" tilewidth="32" tileheight="32" tilecount="1" columns="1">\n<image source="fixture.png" width="32" height="32"/>\n<wangsets><wangset name="Summer Terrain" type="corner" tile="0">{wangcolors}</wangset></wangsets>\n</tileset>''',
            encoding="utf-8",
        )
        result = audit(td, tsx)
    assert result["ledgerMaterialCount"] == 12
    assert result["stateCounts"]["live_certified"] == 5
    assert result["liveMappings"]["RiverWater"] == "Shallow Water"
    assert result["blockedRoles"] == ["PebblePath", "WetSand"]
    print("M2D096D TILED MATERIAL BRIDGE SELFTEST: PASS / 12/12 explicit, 5 live, 7 deferred/blocked")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", type=Path, default=Path.cwd())
    ap.add_argument("--tsx", type=Path)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args(argv)
    if args.selftest:
        return selftest()
    root = args.root.resolve()
    tsx = args.tsx or (root / DEFAULT_TSX)
    if not tsx.is_absolute():
        tsx = root / tsx
    if not tsx.is_file():
        print(f"M2D096D MATERIAL BRIDGE: BLOCKED - TSX missing: {tsx}", file=sys.stderr)
        return 2
    try:
        result = audit(root, tsx)
    except Exception as exc:
        print(f"M2D096D MATERIAL BRIDGE: FAIL - {exc}", file=sys.stderr)
        return 1
    print("M2D096D TILED MATERIAL BRIDGE: PASS")
    print(f"  authored materials : {result['ledgerMaterialCount']}/12 explicitly accounted")
    print(f"  states             : {result['stateCounts']}")
    print(f"  live mappings      : {result['liveMappings']}")
    print(f"  blocked roles      : {result['blockedRoles']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
