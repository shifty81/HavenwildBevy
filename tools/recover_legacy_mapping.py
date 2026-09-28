#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import sys
from typing import Iterable

ROOT = Path(__file__).resolve().parents[1]
LOCAL_ROOT = ROOT / '.forgepy' / 'recovered_mapping'
RAW_ROOT = LOCAL_ROOT / 'raw'
INDEX = LOCAL_ROOT / 'index.json'

KNOWN_RELATIVE = [
    Path('assets/generated/worldgen_v0_1/terrain/lpc_terrain_summer_complete_map_32.json'),
    Path('content/assets/lpc/lpc_seasonal_terrain_topology_v0_1.json'),
    Path('assets/generated/worldgen_v0_1/terrain/lpc_mapped_terrain_v7_32.json'),
    Path('content/terrain/havenwild_terrain_standard_v1.json'),
    Path('content/terrain/havenwild_terrain_authoring_palette_v1.json'),
]

NAME_PATTERNS = [
    '*summer*crosswalk*.json',
    '*source*crosswalk*.json',
    '*seasonal*source*.json',
    '*tuple*catalog*.json',
    '*terrain*topology*.json',
]


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as fh:
        for block in iter(lambda: fh.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def candidates(explicit: str | None) -> list[Path]:
    result: list[Path] = []
    if explicit:
        result.append(Path(explicit).expanduser())
    parent = ROOT.parent
    for name in (
        'Havenwild',
        'Test Havenwild',
        'HavenWild Dev',
        'HavenWild Dev - Copy',
        'Havenwild_CompleteSourceRollup_Pass150H_20260723',
    ):
        result.append(parent / name)
    # ForgePY's remembered source may point at the historical/full Havenwild tree.
    config = ROOT / '.forgepy' / 'config.json'
    if config.is_file():
        try:
            data = json.loads(config.read_text(encoding='utf-8'))
            for key in ('assetSource', 'source', 'sourceRoot'):
                value = data.get(key)
                if value:
                    result.append(Path(value).expanduser())
        except Exception:
            pass
    unique: list[Path] = []
    seen: set[str] = set()
    for path in result:
        try:
            resolved = path.resolve()
        except Exception:
            resolved = path
        key = str(resolved).lower()
        if key not in seen:
            seen.add(key)
            unique.append(resolved)
    return unique


def iter_search_roots(root: Path) -> Iterable[Path]:
    yield root
    for child in ('content', 'assets', 'apps', 'tools', 'DOCS', 'docs'):
        p = root / child
        if p.is_dir():
            yield p


def discover(root: Path) -> list[Path]:
    found: dict[str, Path] = {}
    for rel in KNOWN_RELATIVE:
        p = root / rel
        if p.is_file():
            found[str(p.resolve()).lower()] = p
    # Search only source/metadata lanes; never scan hydrated Characters or build output.
    for search_root in iter_search_roots(root):
        if not search_root.is_dir():
            continue
        for pattern in NAME_PATTERNS:
            try:
                for p in search_root.glob(f'**/{pattern}'):
                    if not p.is_file():
                        continue
                    parts = {part.lower() for part in p.parts}
                    if parts & {'target', '.git', '.pcc', '.forgepy', 'characters'}:
                        continue
                    found[str(p.resolve()).lower()] = p
            except OSError:
                continue
    return sorted(found.values(), key=lambda p: str(p).lower())


def safe_name(source_root: Path, path: Path) -> str:
    try:
        rel = path.resolve().relative_to(source_root.resolve())
        stem = '__'.join(rel.parts)
    except Exception:
        stem = path.name
    return stem.replace(':', '_')


def recover(source_arg: str | None, copy_raw: bool) -> int:
    roots = [p for p in candidates(source_arg) if p.is_dir() and p.resolve() != ROOT.resolve()]
    print('HAVENWILD LEGACY MAPPING RECOVERY')
    if not roots:
        print('No historical Havenwild source root was found automatically.')
        print('Use: PCC.cmd terrain recover --source "C:\\path\\to\\older\\Havenwild"')
        return 2

    records = []
    for source_root in roots:
        matches = discover(source_root)
        if not matches:
            continue
        print(f'SOURCE: {source_root}')
        for path in matches:
            digest = sha256(path)
            rec = {
                'sourceRoot': str(source_root),
                'sourcePath': str(path),
                'relativePath': str(path.relative_to(source_root)) if path.is_relative_to(source_root) else path.name,
                'size': path.stat().st_size,
                'sha256': digest,
                'copiedLocal': False,
            }
            if copy_raw:
                RAW_ROOT.mkdir(parents=True, exist_ok=True)
                dest = RAW_ROOT / safe_name(source_root, path)
                shutil.copy2(path, dest)
                rec['copiedLocal'] = True
                rec['localPath'] = str(dest.relative_to(ROOT))
            records.append(rec)
            print(f"  FOUND {rec['relativePath']} [{rec['size']} bytes] {digest[:12]}…")

    if not records:
        print('Historical roots were found, but none of the known mapping artifacts were present.')
        return 3

    LOCAL_ROOT.mkdir(parents=True, exist_ok=True)
    payload = {
        'schema': 'havenwild.terrain.legacy_mapping_recovery.local.v1',
        'records': records,
        'note': 'Local recovery evidence only. Raw files are not promoted into current recipe authority until reviewed and normalized.',
    }
    INDEX.write_text(json.dumps(payload, indent=2) + '\n', encoding='utf-8')
    print(f'INDEX: {INDEX.relative_to(ROOT)}')
    print(f'ARTIFACTS: {len(records)}')
    if copy_raw:
        print('Raw metadata copied under .forgepy/recovered_mapping/raw (source-only packaging excludes .forgepy).')
    return 0


def status() -> int:
    if not INDEX.is_file():
        print('LEGACY RECOVERY: no local recovery index yet')
        return 0
    try:
        data = json.loads(INDEX.read_text(encoding='utf-8'))
    except Exception as exc:
        print(f'LEGACY RECOVERY: unreadable index: {exc}')
        return 2
    records = data.get('records', [])
    print(f'LEGACY RECOVERY: {len(records)} metadata artifact(s) indexed')
    for rec in records[:20]:
        print(f"  {rec.get('relativePath')} · {str(rec.get('sha256',''))[:12]}…")
    if len(records) > 20:
        print(f'  ... {len(records) - 20} more')
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command')
    rec = sub.add_parser('recover')
    rec.add_argument('--source')
    rec.add_argument('--no-copy', action='store_true')
    sub.add_parser('status')
    args = parser.parse_args(argv)
    if args.command == 'recover':
        return recover(args.source, copy_raw=not args.no_copy)
    return status()


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
