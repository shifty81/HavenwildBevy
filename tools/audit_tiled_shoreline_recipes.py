#!/usr/bin/env python3
"""Read-only exact TSX Wang audit. Never invents tiles or changes LPC source files.

Reports coverage of the 16 possible 2-material corner configurations for each
land/water pair; generated-world fallback frequencies are audited inside Studio.
"""
from __future__ import annotations
import argparse
import csv
import json
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
TSX = ROOT / 'assets/elizawy/_jaidyn_tiled/lpc-tileset-terrain-summer.tsx'
LAND = ('Grass', 'Dirt', 'Sand', 'Deep Sand')
WATER = ('Shallow Water', 'Deep Water')


def audit(path: Path):
    root = ET.parse(path).getroot()
    wang = next((w for w in root.findall('.//wangset') if w.get('name') == 'Summer Terrain'), None)
    if wang is None or wang.get('type') != 'corner':
        raise ValueError('Required ElizaWy Summer Terrain corner-Wang set missing')
    colors = {c.get('name'): i for i,c in enumerate(wang.findall('wangcolor'), 1)}
    exact = {}
    for tile in wang.findall('wangtile'):
        bits = tuple(int(x) for x in tile.get('wangid', '').split(','))
        if len(bits) != 8:
            raise ValueError(f'Incomplete Wang ID for tile {tile.get("tileid")}')
        exact.setdefault(bits, []).append(int(tile.get('tileid')))
    rows = []
    for land in LAND:
        for water in WATER:
            for mask in range(16):
                # NW NE SW SE mask, Tiled corner positions TR BR BL TL.
                ids = [colors[water] if mask & (1 << bit) else colors[land] for bit in range(4)]
                nw,ne,sw,se = ids
                signature = (0,ne,0,se,0,sw,0,nw)
                ids_found = sorted(exact.get(signature, []))
                rows.append({'land': land, 'water': water, 'water_mask': f'{mask:04b}',
                             'wang_id': ','.join(map(str, signature)),
                             'status': 'authored' if ids_found else 'missing',
                             'tile_ids': ','.join(map(str, ids_found))})
    summary = {}
    for land in LAND:
        for water in WATER:
            records = [row for row in rows if row['land'] == land and row['water'] == water]
            summary[f'{land} / {water}'] = {
                'authored': sum(row['status']=='authored' for row in records), 'total': 16,
                'missing_masks': [row['water_mask'] for row in records if row['status'] == 'missing'],
            }
    return rows, summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tsx',type=Path,default=TSX)
    parser.add_argument('--output',type=Path,default=ROOT/'artifacts'/'shoreline_authority'/'M2D102_source_wang_coverage.json')
    args=parser.parse_args()
    rows, summary = audit(args.tsx)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps({'schema':'havenwild.shoreline.authority.audit.v1',
            'source':str(args.tsx.relative_to(ROOT)) if args.tsx.is_relative_to(ROOT) else str(args.tsx),
            'non_destructive':True,'coverage':summary},indent=2)+'\n',encoding='utf-8')
    with args.output.with_suffix('.csv').open('w',newline='',encoding='utf-8') as f:
        writer=csv.DictWriter(f,fieldnames=rows[0].keys());writer.writeheader();writer.writerows(rows)
    for name, counts in summary.items():
        print(f'{name:26} {counts["authored"]:2}/16 exact recipes')
    print(f'Report: {args.output}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
