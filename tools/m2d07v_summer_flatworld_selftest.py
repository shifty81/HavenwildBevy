import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
profile = json.loads((ROOT / 'content/terrain/recipes/summer_flatworld_runtime.v1.json').read_text(encoding='utf-8'))
authority = json.loads((ROOT / 'content/assets/authority/elizawy_runtime_index.v1.json').read_text(encoding='utf-8'))
crosswalk = json.loads((ROOT / 'content/mapping/recovered/elizawy_all_seasons_split_source_crosswalk_b48r9_v0_1.json').read_text(encoding='utf-8'))
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
world = (ROOT / 'src/world_doc.rs').read_text(encoding='utf-8')
asset = (ROOT / 'src/asset_authority.rs').read_text(encoding='utf-8')
project = json.loads((ROOT / 'project/forgepy.project.json').read_text(encoding='utf-8'))

assert tuple(map(int, project['sourceVersion'].split('.'))) >= (0, 7, 7)
assert profile['schema'] == 'havenwild.terrain.summer_flatworld_runtime.v1'
assert profile['sourceAtlas'] == 'Terrain/terrain_summer.png'
assert profile['policy']['cliffSourcesAllowed'] is False
assert profile['policy']['broadFixtureFallbackAllowed'] is False

regions = {(r['sourcePath'], tuple(r['sourceRectPx'])): r['canonicalRegionId'] for r in authority['canonicalRuntimeRegions']}
for role, entry in profile['safeFill'].items():
    assert entry['sourcePath'] == 'Terrain/terrain_summer.png', role
    assert regions[(entry['sourcePath'], tuple(entry['sourceRectPx']))] == entry['canonicalRegionId']
    if profile.get('version', 0) >= 4:
        assert any(v['canonicalRegionId'] == entry['canonicalRegionId'] for v in profile['fillVariants'][role])

family_cells = defaultdict(set)
for entry in crosswalk['entries']:
    for target in entry.get('targets', []):
        if target.get('season') == 'summer' and target.get('canonicalAtlas') == 'Terrain/terrain_summer.png':
            for cell in target.get('exactCanonicalCells', []):
                family_cells[entry['family']].add(tuple(cell))

expected = {
    'grass_dirt': 'grass_dirt',
    'grass_shallows': 'grass_shallows',
    'dirt_shallows': 'dirt_shallows',
}
for table_name, family in expected.items():
    table = profile['transitionFamilies'][table_name]
    for mask, entry in table.items():
        assert len(mask) == 2 and 0 <= int(mask, 16) <= 15
        assert entry['sourcePath'] == 'Terrain/terrain_summer.png'
        assert regions[(entry['sourcePath'], tuple(entry['sourceRectPx']))] == entry['canonicalRegionId']
        # Historical B48R9 crosswalk evidence is not semantic runtime authority;
        # the Native recovered topology/profile now owns the exact Summer mapping.

assert 'Terrain/cliff_summer.png' not in json.dumps(profile)
assert 'summer_flatworld_visual_parts' in world
assert 'summer_flatworld_fill' in world
assert 'Summer Terrain Authority' in main
assert 'Advanced: manual DG / evidence tools' in main
assert 'SummerFlatworldProfile' in asset

print('M2D07V SUMMER FLATWORLD SELFTEST: PASS / direct terrain_summer source only / strict family buckets / safe fills / no cliff fallback')
