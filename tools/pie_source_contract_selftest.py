#!/usr/bin/env python3
"""Fail closed on PIE falsely treating the original screenshot as authored scene bytes."""
from pathlib import Path
import json
import re
r=Path(__file__).resolve().parents[1]
main=(r/'src/main.rs').read_text(encoding='utf-8')
play=(r/'src/playtest.rs').read_text(encoding='utf-8')
scene=json.loads((r/'content/scenes/elizawy_mapping_certification.scene.json').read_text(encoding='utf-8'))
ref=json.loads((r/'docs/research/ELIZAWY_ORIGINAL_DEMO_REFERENCES.json').read_text(encoding='utf-8'))
assert scene['schema']=='havenwild.elizawy.scene.v1' and scene['size']==[40,28]
assert scene['tiles'] and set(t['source_asset'] for t in scene['tiles']) == {'Terrain/terrain_summer.png', 'Terrain/cliff_summer.png'}
assert ref['authority']=='visual_reference_only_not_tile_source_or_editable_scene'
for s in ('mod playtest;', 'playtest: Option<PlaySession>', 'PlaySession::start_with_collision(&source_exact_snapshot', 'fn stop_playtest(', 'playtest.step(', 'pub fn step('):
    assert re.sub(r'\s+', '', s) in re.sub(r'\s+', '', main+play),s
assert 'include_bytes!("hw_reference_demo_summer.png")' not in main
assert 'legacy' in play and 'RiverWater' in play and 'snapshot: scene.clone()' in play
assert 'state.playtest_layered = Some(state.layered_scene.clone());' in main
assert 'draw_source_exact_visual_layers(' in main and re.search(r'playtest_layered\s*\.as_ref\(\)\s*\.unwrap_or\(&state\.layered_scene\)', main)
assert 'structural_snapshot: structural.to_vec()' in play
assert 'state.layered_scene.structural_cells' in main
print('PIE SOURCE CONTRACT: PASS / exact authored-scene snapshot, explicit structural override, original demo visual-only')
