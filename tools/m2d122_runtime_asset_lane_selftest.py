#!/usr/bin/env python3
"""Asset render synchronization and honest coverage regression (source-only)."""
from pathlib import Path
import json
r=Path(__file__).resolve().parents[1]
s=(r/'src/main.rs').read_text(encoding='utf-8')
module=(r/'src/render_asset_sync.rs').read_text(encoding='utf-8')
contract=json.loads((r/'project/forgepy.project.json').read_text())
assert tuple(map(int,contract['sourceVersion'].split('.'))) >= (0,10,40)
for token in ('mod render_asset_sync;', 'runtime_asset_sync_revision: Option<u64>',
    'runtime_asset_sync_queue: VecDeque<String>', 'runtime_asset_sync_failures: BTreeMap<String, String>',
    'render_asset_sync::missing_world_source_paths(',
    'if state.runtime_asset_sync_revision != Some(state.world_doc.revision)',
    'state.runtime_asset_sync_queue.pop_front()', 'for _ in 0..4',
    'load_source_sheet_on_demand(', 'if state.sheet_lookup.contains_key(&path)',
    '"Rescan world asset bindings"', '"Missing world render assets"',
    '"Reviewed complete source sprites"', 'state.selected_extent = [rect[2] / 32, rect[3] / 32]',
    'state.sheets.iter().filter(|sheet| sheet.texture.is_none()).count()'):
    assert token in s, token
for token in ('pub fn required_world_source_paths(', 'pub fn missing_world_source_paths(',
    'for chunk in &world.chunks', 'for cell in &chunk.generated_cells',
    'for object in world.generated_objects()', 'for item in &world.cell_overrides',
    'materialized_world_asset_dependencies_cover_objects_overrides_and_deduplicate'):
    assert token in module, token
assert 'world.chunks.clear()' not in module
assert 'world.source_behavior_profiles' not in module
import sys
sys.path.insert(0, str(r/'tools'))
from audit_asset_render_coverage import collect
report=collect()
sum_=report['summary']
assert not report['issues'],report['issues']
assert sum_['cataloguedPngSheets']==320
assert sum_['sheetsWithCanonicalRegions']+sum_['sheetsWithoutCanonicalRegions']==320
assert sum_['sheetsWithCanonicalRegions']==30 and sum_['sheetsWithoutCanonicalRegions']==290
assert sum_['canonicalRuntimeRegionAddresses']==2892 and sum_['reviewedObjectTemplates']==28
assert sum_['sheetsUsedByReviewedTemplates']==13
assert sum_['byFamily']['FX']['hasCanonicalRegions']==0
assert sum_['rolePromotions']==0
print('M2D122 RENDER LANE: PASS / 320 manifest PNGs, 30 source-region images, 290 yet unenumerated, 28 reviewed templates')
print('M2D122 NEW: materialized world dependency collection, post-regeneration lazy hydration, failed image reporting; native Cargo and screenshot acceptance outstanding')
