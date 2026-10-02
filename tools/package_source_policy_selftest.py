#!/usr/bin/env python3
"""Regression checks for the source-only package boundary."""
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

import ForgePY


def main() -> int:
    allowed = [
        Path(".gitattributes"),
        Path("Cargo.toml"),
        Path("PCC.cmd"),
        Path("src/main.rs"),
        Path("docs/PROJECT_CONTROL_CENTER.md"),
        Path("assets/README.md"),
        Path("updates/README.md"),
    ]
    blocked = [
        Path("old-handoff.zip"),
        Path("old-update.patch"),
        Path("something.pccpatch.zip"),
        Path("docs/archive/old-build.zip"),
        Path("tools/fixture.tar.gz"),
        Path("Havenwild_M2D080A_Guarded_Compile_Repair.ps1"),
        Path("tools/_package_pie_temp.py"),
        Path("artifacts/source.zip"),
        Path(".git/config"),
        Path(".pcc/logs/session.log"),
        Path(".forgepy/local.json"),
        Path("__pycache__/local.pyc"),
        Path("assets/elizawy/Terrain/example.png"),
        Path("reference/donor.zip"),
        Path("updates/inbox/new.pccpatch.zip"),
        Path("content/scenes/derived/local-draft.json"),
    ]
    errors = []
    for rel in allowed:
        if not ForgePY._source_package_include(rel):
            errors.append(f"expected included: {rel.as_posix()}")
    for rel in blocked:
        if ForgePY._source_package_include(rel):
            errors.append(f"expected excluded: {rel.as_posix()}")
    if errors:
        print("SOURCE PACKAGE POLICY: FAIL")
        for error in errors:
            print(" -", error)
        return 2
    print("SOURCE PACKAGE POLICY: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
