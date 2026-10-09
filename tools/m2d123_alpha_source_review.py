#!/usr/bin/env python3
"""ElizaWy 320-PNG SHA-verified alpha census, never semantic promotion.

--build reads hydrated assets/elizawy or an explicitly provided source ZIP.
Pillow is required only when building this evidence index, not for PCC --check.
--check validates committed evidence using Python standard library; no asset hydration required.
"""
from __future__ import annotations
import argparse, hashlib, io, json, sys, zipfile
from collections import Counter, deque
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
TARGET=ROOT/'content/assets/review/original_alpha_candidates.v1.json'
SCHEMA='havenwild.bevy.original_alpha_candidates.v1'

def manifest():
    original=json.loads((ROOT/'content/catalog/core_source_manifest.json').read_text(encoding='utf-8'))
    return {row['path']:row for row in original['entries'] if row['path'].lower().endswith('.png')}

def scan_groups(image):
    """4-connected groups of occupied 32px *cells*. Conservatively evidence-only."""
    alpha=image.convert('RGBA').getchannel('A')
    w,h=image.size
    # Scan only complete 32px cells; report partial right/bottom strips separately.
    # Wildflowers and one stair sheet have genuine non-32px source dimensions.
    gw,gh=w//32,h//32
    filled=set()
    for y in range(gh):
        for x in range(gw):
            if alpha.crop((x*32,y*32,(x+1)*32,(y+1)*32)).getbbox():filled.add((x,y))
    unseen=set(filled)
    regions=[]; crowded=0
    while unseen:
        start=min(unseen,key=lambda p:(p[1],p[0]))
        q=deque((start,)); unseen.remove(start)
        group=[]
        while q:
            x,y=q.popleft();group.append((x,y))
            for p in ((x+1,y),(x-1,y),(x,y+1),(x,y-1)):
                if p in unseen:unseen.remove(p);q.append(p)
        x0=min(x for x,_ in group);x1=max(x for x,_ in group)
        y0=min(y for _,y in group);y1=max(y for _,y in group)
        sx,sy=x1-x0+1,y1-y0+1
        if sx<=12 and sy<=12 and len(group)<=36:
            regions.append({'rectPx':[x0*32,y0*32,sx*32,sy*32], 'occupiedCells':len(group)})
        else:crowded+=1
    regions.sort(key=lambda p:(p['rectPx'][1],p['rectPx'][0],p['rectPx'][2]))
    return len(filled), regions, crowded

def validate(payload, entries):
    problems=[]
    if payload.get('schema')!=SCHEMA: problems.append('unexpected evidence schema')
    sheets=payload.get('sheets',[])
    if len(sheets)!=len(entries):problems.append(f'source count {len(sheets)} expected {len(entries)}')
    seen=set()
    for row in sheets:
        name=row['sourcePath']; info=entries.get(name)
        if name in seen:problems.append(f'duplicate sheet {name}')
        seen.add(name)
        if info is None:problems.append(f'unrecognized sheet {name}');continue
        size=info['imageSizePx']
        if row['sourceSha256']!=info['sha256'] or row['imageSizePx']!=size:problems.append(f'source identity mismatch {name}')
        if row['unalignedEdgesPx']!=[size[0]%32,size[1]%32]:problems.append(f'partial cell bounds mismatch {name}')
        if row['occupiedCells']<0 or row['occupiedCells']>(size[0]//32)*(size[1]//32):problems.append(f'bad occupancy {name}')
        if row['crowdedGroups']<0:problems.append(f'negative crowded groups {name}')
        visited=set()
        for region in row['isolatedGroups']:
            x,y,w,h=region['rectPx']
            if min(x,y)<0 or min(w,h)<=0 or any(v%32 for v in (x,y,w,h)) or x+w>size[0] or y+h>size[1]:problems.append(f'bad rect {name}: {region}')
            if not (0<region['occupiedCells']<=w*h//1024):problems.append(f'bad group occupancy {name}')
            if (x,y,w,h) in visited:problems.append(f'duplicate group {name}')
            visited.add((x,y,w,h))
        if any(row['isolatedGroups'][i]['rectPx'][1]>row['isolatedGroups'][i+1]['rectPx'][1] for i in range(len(row['isolatedGroups'])-1)):
            problems.append(f'noncanonical group sort {name}')
    if seen!=set(entries):problems.append('source coverage incomplete')
    return problems

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--build',action='store_true')
    ap.add_argument('--check',action='store_true')
    ap.add_argument('--source-zip',type=Path,help='read an original hydrated project ZIP; never writes the original PNG bytes')
    ap.add_argument('--assets-root',type=Path,default=ROOT/'assets/elizawy')
    args=ap.parse_args()
    entries=manifest()
    if args.build:
        try:from PIL import Image
        except ImportError:
            print('Building alpha evidence requires Pillow; PCC --check needs no Pillow',file=sys.stderr);return 2
        rows=[]; z=zipfile.ZipFile(args.source_zip) if args.source_zip else None
        indexed={name:name for name in z.namelist()} if z else {}
        for name,info in sorted(entries.items()):
            if z:
                matches=[v for v in indexed if v.replace('\\','/').endswith('assets/elizawy/'+name)]
                if len(matches)!=1:raise ValueError(f'ZIP original source missing/ambiguous: {name}')
                contents=z.read(matches[0])
            else:contents=(args.assets_root/name).read_bytes()
            if hashlib.sha256(contents).hexdigest()!=info['sha256']:
                raise ValueError(f'Refusing modified/mismatched source art: {name}')
            image=Image.open(io.BytesIO(contents))
            if list(image.size)!=info['imageSizePx']:raise ValueError(f'Image dimensions changed: {name}')
            occ,candidates,crowded=scan_groups(image)
            rows.append({'sourcePath':name,'sourceSha256':info['sha256'],
                         'imageSizePx':info['imageSizePx'],'occupiedCells':occ,
                         'unalignedEdgesPx':[image.width%32,image.height%32],
                         'isolatedGroups':candidates,'crowdedGroups':crowded})
        if z:z.close()
        result={'schema':SCHEMA,'sourceAuthority':'content/catalog/core_source_manifest.json',
                'method':'Four-connected occupied 32px cells; isolated <= 12x12 cells and <=36 total; candidates only, never automatic sprite or role certification',
                'originalPixelsModified':False,'automaticApprovals':0,'sheets':rows}
        errors=validate(result,entries)
        if errors:raise ValueError(errors[:10])
        TARGET.parent.mkdir(parents=True,exist_ok=True)
        TARGET.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    if args.check or args.build:
        p=json.loads(TARGET.read_text(encoding='utf-8'))
        problems=validate(p,entries)
        for error in problems[:20]:print('FAIL:',error)
        if problems:return 1
        n=sum(len(s['isolatedGroups']) for s in p['sheets'])
        dense=sum(s['crowdedGroups'] for s in p['sheets'])
        occupied=sum(s['occupiedCells'] for s in p['sheets'])
        families=Counter(s['sourcePath'].split('/')[0] for s in p['sheets'])
        print(f'M2D123 VERIFIED ORIGINAL-ALPHA CENSUS: PASS / {len(p["sheets"])} SHA-anchored PNGs / {occupied} occupied grid cells / {n} isolated candidates / {dense} crowded groups')
        print('FAMILIES:',dict(sorted(families.items())))
        print('NON-32px ORIGINALS:',sum(any(s['unalignedEdgesPx']) for s in p['sheets']))
        print('Unreviewed alpha groups: NOT approved object templates; no automatic semantic promotions')
        return 0
    ap.print_help();return 2
if __name__=='__main__':raise SystemExit(main())
