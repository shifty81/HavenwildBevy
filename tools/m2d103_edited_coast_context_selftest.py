#!/usr/bin/env python3
"""M2D103 source/TSX regression. Native Rust build and visual QA still required."""
from pathlib import Path
import re
from collections import Counter
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
world = (ROOT / 'src/world_doc.rs').read_text(encoding='utf-8')
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
for symbol in (
    'fn semantic_role_for_dual_grid(',
    'self.semantic_role_for_dual_grid(neighbor)',
    'self.pending_semantic_edits.get(&world)',
    'if context == "coast" { return "RiverWater".into(); }',
    'fn painted_sand_dirt_and_grass_in_deep_water_create_visual_shallow_fringe()',
):
    assert re.sub(r"\s+", "", symbol) in re.sub(r"\s+", "", world), f'Missing edited-coast source/regression: {symbol}'
for symbol in (
    'response.secondary_clicked()',
    'response.context_menu(|ui|',
    'Select object at cell',
    'Sample terrain into brush',
    'Erase authored terrain here',
    'state.world_doc.paint_semantic_terrain(cell, state.world_terrain_brush.role())',
    'if primary_pressed && response.hovered()',
    'Outline missing coast recipes and land junctions',
):
    assert re.sub(r"\s+", "", symbol) in re.sub(r"\s+", "", main), f'Missing right-click or general terrain-audit source: {symbol}'
assert 'response.dragged_by(egui::PointerButton::Secondary)' not in main.split('fn draw_world_workspace(', 1)[1].split('fn draw_world(', 1)[0], 'Generated World secondary still pans'

def wang(season):
    p = ROOT / 'assets/elizawy/_jaidyn_tiled' / f'lpc-tileset-terrain-{season}.tsx'
    root = ET.parse(p).getroot()
    wang_set = root.find('.//wangset')
    colors = [c.attrib['name'] for c in wang_set.findall('wangcolor')]
    tile_ids = {int(t.attrib['tileid']): t.attrib['wangid'] for t in wang_set.findall('wangtile')}
    return colors, tile_ids
summer_c, summer_tiles = wang('summer')
assert {'Sand','Grass','Dirt','Shallow Water','Deep Water'}.issubset(summer_c)
assert len(summer_tiles) == 331
assert len([tile for tile in summer_tiles.values() if len({int(x) for x in tile.split(',') if int(x)}) >= 3]) == 0, 'Unexpected source three-material support'
for season in ['spring','winter']:
    colors, tiles = wang(season)
    assert len(tiles) == 331 and tiles == summer_tiles, (season, 'changed topology or tile IDs')
_, autumn = wang('autumn-old')
assert len(autumn) == 281 and autumn != summer_tiles
print('M2D103 SOURCE/TSX CHECK: PASS')
print('Manual sand/dirt/grass edits now participate in visual shallow-water fringe; gameplay remains deep water')
print('World canvas: secondary context menu, middle-pan, guarded left-click interactions')
print('Audit: coast plus all other unsupported Wang material junctions')
print('Season: spring/winter topology IDs match summer; older autumn TSX does not')
print('Note: Source check only; native Rust tests, Windows Full Gate and screenshots still required')
