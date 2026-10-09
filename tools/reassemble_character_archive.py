#!/usr/bin/env python3
"""Restore the byte-exact LPC Characters.zip from M2D100 delivery parts.

Never mutates canonical art or a matching existing archive, never silently
removes parts, and fails closed before replacing a local source archive.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import os
import sys

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "optional" / "Characters.zip"
PART_MANIFEST = ROOT / "content" / "catalog" / "character_rollup_parts.v1.json"
CORE_MANIFEST = ROOT / "content" / "catalog" / "core_source_manifest.json"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def run() -> int:
    expected = json.loads(CORE_MANIFEST.read_text(encoding='utf-8'))['optionalArchives']['Characters.zip']
    if OUT.is_file():
        if OUT.stat().st_size == expected['bytes'] and sha256(OUT) == expected['sha256']:
            print('CHARACTERS ARCHIVE: already present and pinned; nothing to do')
            return 0
        print('ERROR: existing Characters.zip has a different identity; refusing to overwrite it.')
        return 3
    manifest = json.loads(PART_MANIFEST.read_text(encoding='utf-8'))
    parts = manifest['parts']
    checked = []
    for row in parts:
        p = ROOT / row['path']
        if not p.is_file() or p.stat().st_size != row['bytes'] or sha256(p) != row['sha256']:
            print(f'ERROR: missing/corrupt part: {row["path"]}')
            return 3
        checked.append(p)
        print(f'PART VERIFIED: {p.name} ({row["bytes"]} bytes)', flush=True)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    tmp = OUT.with_name('Characters.zip.assembling')
    if tmp.exists():
        print('ERROR: stale Characters.zip.assembling exists. Resolve it explicitly before retrying.')
        return 3
    try:
        with tmp.open('xb') as target:
            for p in checked:
                with p.open('rb') as source:
                    for block in iter(lambda: source.read(1024 * 1024), b''):
                        target.write(block)
        if tmp.stat().st_size != expected['bytes'] or sha256(tmp) != expected['sha256']:
            print('ERROR: reassembled bytes did not match canonical source; not installed')
            return 3
        os.replace(tmp, OUT)
        print(f'CHARACTERS ARCHIVE: PASS / original {expected["bytes"]} bytes / pinned SHA256')
        print('Next: PCC.cmd assets hydrate-characters')
        return 0
    finally:
        if tmp.exists():
            tmp.unlink()


if __name__ == '__main__':
    raise SystemExit(run())
