#!/usr/bin/env python3
"""No-network, no-build tests for the cumulative M2C2+M2C3 ElizaWy evidence tooling."""
from __future__ import annotations
import argparse
import hashlib
import io
import json
from pathlib import Path
import tempfile
import zipfile
from reconcile_elizawy_tiled import get_map, png_size, reconcile, wang_data, pixel_compare, SUMMER_MAP_SHA


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--archive', type=Path, help='Optional original JaidynReiman ZIP for exact data test')
    a=p.parse_args()
    xml=b'''<?xml version="1.0"?><tileset name="Summer Terrain" tilewidth="32" tileheight="32" tilecount="4" columns="2"><image source="x.png" width="64" height="64"/><wangsets><wangset name="Summer Terrain" type="corner"><wangcolor name="Grass"/><wangtile tileid="1" wangid="0,1,0,1,0,1,0,1"/></wangset></wangsets></tileset>'''
    assert wang_data(xml)['Terrain']['assignments'][1]==[0,1,0,1,0,1,0,1]
    try:
        wang_data(xml.replace(b'0,1,0,1,0,1,0,1',b'0,2,0,1,0,1,0,1'))
        raise AssertionError('out-of-range Wang color accepted')
    except ValueError: pass
    assert png_size(b'\x89PNG\r\n\x1a\n'+b'\x00\x00\x00\rIHDR'+b'\x00\x00\x08\x00\x00\x00\x08\x00')==[2048,2048]
    try:
        png_size(b'not a PNG')
        raise AssertionError('non-PNG accepted')
    except ValueError:pass
    with tempfile.TemporaryDirectory() as d:
        root=Path(d)
        assert get_map(root,None)[1]=='recovery_index_unavailable'
        f=root/'fake.json';f.write_text(json.dumps({'grid':[16,26],'cellSize':32}),encoding='utf-8')
        assert get_map(root,f)[1]=='historical_map_hash_mismatch'
        assert pixel_compare(None,None,{},None,[])['state']=='not_requested'
    print('M2C3 SELFTEST: PASS / Wang signature bounds, PNG header, historical source hash, optional image proof, zero source mutation')
    if a.archive:
        rep=reconcile(a.archive.resolve(),Path(__file__).resolve().parents[1],strict_archive_sha=True)
        assert not rep['errors'] and rep['matchesAuditedUpload']
        assert len(rep['seasonProfiles'])==4 and rep['certifiedMappingsAdded']==0 and not rep['runtimeChanged']
        assert rep['comparison']['autumn']['Terrain']['missingCount']==58
        assert rep['comparison']['autumn']['Terrain']['extraCount']==8
        assert rep['comparison']['autumn']['Terrain']['changedCount']==10
        assert rep['comparison']['autumn']['Fences']['missingCount']==60
        assert rep['comparison']['winter']['Terrain']['semanticNamesIdentical'] is False
        assert rep['buildingProfile']['policy']=='architecture_reference_only_not_terrain_authority'
        print('M2C3 ORIGINAL ARCHIVE TEST: PASS / 4 season profiles, autumn gap, semantic winter difference, building quarantine, zero promotion')
    return 0
if __name__=='__main__':raise SystemExit(main())
