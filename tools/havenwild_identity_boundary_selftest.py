#!/usr/bin/env python3
"""Fail-closed standalone project identity guard for checked-in source files."""
from __future__ import annotations
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import ForgePY  # noqa: E402

# Assemble signatures at runtime to keep this guard from triggering itself.
TOKENS = tuple(token.casefold() for token in (
    "null" + "harbor", "null_" + "harbor", "null-" + "harbor",
    "null " + "harbor", "sub" + "space", "ship" + "yard",
))
TEXT_EXT = {".py", ".rs", ".md", ".json", ".toml", ".cmd", ".ps1", ".txt", ".yaml", ".yml", ".csv", ".gitignore", ".gitattributes"}

def validate(root: Path = ROOT) -> list[str]:
    problems: list[str] = []
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(root)
        if rel.parts[0] in {".pcc", ".forgepy", ".git", "target", "artifacts", "__pycache__", "reference", "updates"}:
            continue
        if rel.parts[0] == "assets" and rel.as_posix() != "assets/README.md":
            continue
        name = rel.as_posix().casefold()
        if rel.parts[0] in {"legacy", "rust"} or (len(rel.parts) > 1 and rel.parts[:2] in {("tools", "rust"), ("content", "architecture")}):
            problems.append(f"foreign rooted source: {rel}")
        if any(t in name for t in TOKENS):
            problems.append(f"foreign source path: {rel}")
        if not ForgePY._source_package_include(rel):
            continue
        if p.suffix.casefold() not in TEXT_EXT and p.name not in {".gitignore", ".gitattributes"}:
            continue
        if p.stat().st_size > 25 * 1024 * 1024:
            problems.append(f"text audit skipped oversized file: {rel}")
            continue
        data = p.read_bytes()
        if b"\x00" in data[:8192]:
            problems.append(f"binary data in text file: {rel}")
            continue
        content = data.decode("utf-8", "replace").casefold()
        if any(t in content for t in TOKENS):
            problems.append(f"foreign project identity in: {rel}")
    for rel in ("Cargo.toml", "src/main.rs", "ProjectControlCenter.py", "ForgePY.py", "project/forgepy.project.json"):
        if not (root / rel).is_file():
            problems.append(f"missing primary Havenwild source: {rel}")
    return problems


def main() -> int:
    problems = validate()
    if problems:
        print(f"HAVENWILD IDENTITY BOUNDARY: FAIL ({len(problems)} issue(s))")
        for problem in problems[:35]: print(" -", problem)
        return 2
    print("HAVENWILD IDENTITY BOUNDARY: PASS / deployable files contain no foreign identity; main game retained")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
