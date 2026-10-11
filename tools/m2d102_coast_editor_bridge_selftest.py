#!/usr/bin/env python3
"""M2D102 source regression contract. Not a substitute for the Windows Cargo Full Gate."""
import sys
from pathlib import Path
import re
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from audit_tiled_shoreline_recipes import audit, TSX

rows, summary = audit(TSX)
for left,right in [('Grass','Shallow Water'),('Sand','Shallow Water'),('Dirt','Shallow Water'),('Shallow Water','Deep Water')]:
    if left=='Shallow Water':
        # Confirm the source-authored shallow/deep 14/16 coverage directly.
        import xml.etree.ElementTree as ET
        w=ET.parse(TSX).getroot().find('.//wangset[@name="Summer Terrain"]')
        colors={c.get('name'):i for i,c in enumerate(w.findall('wangcolor'),1)}
        exact={t.get('wangid') for t in w.findall('wangtile')}
        count=0
        for mask in range(16):
            nw,ne,sw,se=[colors[right] if mask>>i&1 else colors[left] for i in range(4)]
            if ','.join(map(str,(0,ne,0,se,0,sw,0,nw))) in exact: count+=1
        assert count==14, ('shallow/deep source recipe count',count)
    else:
        assert summary[f'{left} / {right}']['authored']==14, (left,right)
for left in ['Grass','Dirt','Sand']:
    assert summary[f'{left} / Deep Water']['authored']==2, left
assert len(rows)==128

world=(ROOT/'src/world_doc.rs').read_text()
main=(ROOT/'src/main.rs').read_text()
layout=(ROOT/'src/editor_layout.rs').read_text()
for item in (
    'if context == "coast" { return "RiverWater".into(); }',
    'pub fn place_authored_bridge(', 'pub fn validate_bridge_span(',
    'pub authored_objects: Vec<WorldObject>', 'fn generated_bridge_contains',
    'pub(crate) fn land_connectivity_projection',
    'fn dragged_bridge_is_exact_source_and_undoable_document_state',
    'fn ocean_coast_uses_shallow_wang_visual_without_mutating_water_semantics',
):
    if re.sub(r"\s+", "", item) not in re.sub(r"\s+", "", world): raise AssertionError(f'World contract missing: {item}')
for item in (
    'fn draw_inspector_rail(', 'fn canvas_layer_rect(', 'world_bridge_tool: bool',
    'world_bridge_drag: Option<([i64; 2], [i64; 2])>',
    'state.world_doc.place_authored_bridge(start, end, &state.asset_authority)',
    'Audit shoreline topology', 'Outline missing coast recipes',
    'snapped_scene_edge(region.min.x, ox', 'fn foliage_variation(',
    'fn draw_foliage_uv_variant(', 'state.editor_layout.metrics_drawer_open',
    'fn draw_canvas_view_controls(',
):
    if re.sub(r"\s+", "", item) not in re.sub(r"\s+", "", main): raise AssertionError(f'Editor contract missing: {item}')
for item in ('pub metrics_drawer_open: bool', 'pub shoreline_audit_overlay: bool', 'metrics_drawer_open: false'):
    if item not in layout: raise AssertionError(f'Layout state missing: {item}')
# M2D105: detect the impossible historical Coast + generated DeepWater
# fixture *before* the expensive Windows Cargo test phase. The worldgen
# planner assigns Coast to shallow RiverWater, while deep water is offshore.
coast_test = world.split('fn ocean_coast_uses_shallow_wang_visual_without_mutating_water_semantics()', 1)[1]
coast_test = coast_test.split('\n    }', 1)[0]
normalized_test = re.sub(r"\s+", "", coast_test)
for item in (
    'worldgen_plan::water_context(seed, cell)',
    'worldgen_plan::terrain_role(seed, coast)',
    'doc.paint_semantic_terrain(coast, "DeepWater")',
    'doc.resolved_role_for_dual_grid(coast)',
):
    assert re.sub(r"\s+", "", item) in normalized_test, f'Coast test missing: {item}'
assert 'materialized seed should include a coastal ocean cell' not in coast_test
planner = (ROOT/'src/worldgen_plan.rs').read_text()
terrain_role = planner.split('pub fn terrain_role(', 1)[1].split('\npub fn ', 1)[0]
assert re.search(r'if\s+score\s*>\s*10_000.*?score\s*<=\s*11_100.*?"RiverWater".*?"DeepWater"', terrain_role, re.S), 'Generated coast water role changed; re-audit runtime assertions'

assert 'egui::Panel::right("havenwild.layer.rail")' not in main
assert 'egui::Window::new("Object properties · on demand")' not in main
assert (ROOT/'tools/audit_tiled_shoreline_recipes.py').is_file()
print('M2D102 SOURCE AND TSX AUTHORITY: PASS')
print('Coast: Grass/Sand/Dirt↔Shallow 14/16; Deep↔Shallow 14/16; land↔Deep 2/16')
print('UI: permanent tool and inspector docks, canvas layer rail, zoom controls, right-dock metrics')
print('World: source-backed bridge drag, persistent authored object, source-preserving foliage variants')
print('Native Rust compilation and runtime shoreline acceptance remain outstanding.')
