#!/usr/bin/env python3
import hashlib,json,pathlib
root=pathlib.Path(__file__).resolve().parents[1]
lane_p=root/'content/assets/authority/elizawy_asset_lane.v1.json'; runtime_p=root/'content/assets/authority/elizawy_runtime_index.v1.json'
lane=json.load(open(lane_p,encoding='utf8')); runtime=json.load(open(runtime_p,encoding='utf8'))
assert lane['schema']=='havenwild.bevy.elizawy.asset_lane.v1'
assert runtime['schema']=='havenwild.bevy.elizawy.runtime_index.v1'
assert runtime['lane']=='content/assets/authority/elizawy_asset_lane.v1.json'
assert runtime['laneSha256']==hashlib.sha256(lane_p.read_bytes()).hexdigest()
assert lane['policy']['generatedArtworkAllowed'] is False and runtime['policy']['generatedArtworkAllowed'] is False
assert lane['policy']['sourcePixelsImmutable'] is True
assert lane['counts']==runtime['counts']
assert lane['counts']['runtimeSourceImages']==320
assert lane['counts']['historicalSourceRegions']==1781
assert lane['counts']['canonicalRuntimeRegions']==2890
assert lane['counts']['fixtureRoleRegions']==64
assert lane['counts']['objectTemplates']==28
assert set(runtime['fixtureRolePalette'])=={'Grass','MudBank','RiverWater'}
assert len(runtime['fixtureTopologyPalette'])>=20
assert all(not r['generatedArtwork'] for r in runtime['canonicalRuntimeRegions'])
paths={x['sourcePath'] for x in runtime['runtimeSourceImages']}; assert len(paths)==320
for rows in runtime['fixtureRolePalette'].values(): assert rows and all(x['sourcePath'] in paths and x['weight']>0 for x in rows)
for rows in runtime['fixtureTopologyPalette'].values(): assert rows and all(x['sourcePath'] in paths and x['weight']>0 for x in rows)
regions={r['canonicalRegionId']:r for r in runtime['canonicalRuntimeRegions']}
for t in runtime['objectTemplates']:
    assert t['parts'] and t['worldgenEligible'] is True
    for p in t['parts']:
        assert p['sourcePath'] in paths
        assert p['canonicalRegionId'] in regions
        canonical=regions[p['canonicalRegionId']]
        assert canonical['sourcePath']==p['sourcePath']
        assert canonical['sourceRectPx']==p['sourceRectPx']
print('M2D07 ASSET LANE SELFTEST: PASS / 320 images / 1781 historical regions / 2890 runtime regions / 28 template regions canonical / topology evidence / generated artwork OFF')
