#!/usr/bin/env python3
"""Source-contract gate for incremental UI01/BR01. Cargo and live mouse testing remain required."""
from pathlib import Path
import re
import json

ROOT = Path(__file__).resolve().parents[1]
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
layout = (ROOT / 'src/editor_layout.rs').read_text(encoding='utf-8')
alpha = (ROOT / 'tools/m2d106_egui_source_alpha_selftest.py').read_text(encoding='utf-8')
project = json.loads((ROOT / 'project/forgepy.project.json').read_text(encoding='utf-8'))
assert tuple(map(int, project['sourceVersion'].split('.'))) >= (0, 10, 28)
assert 'fn draw_tool_rail(ui: &mut egui::Ui, state: &mut Studio)' in main
assert 'draw_tool_rail(ui, state);' in main
assert 'egui::Panel::left("havenwild.tool.context")' in main
assert 'state.editor_layout.tool_context_open' in main
assert 'fn draw_tool_context(' in main
assert 'draw_tool_context(&mut viewport_ui, &mut state);' in main
assert 'enum BrushMode' in main
for variant in ('Pixel', 'Tile', 'Terrain'):
    assert f'{variant},' in main
    assert f'BrushMode::{variant}' in main
assert 'choose_brush(state, mode)' in main
assert 'world_semantic_stroke_begin(state, world, false)' in main
assert 'world_stroke_begin(state, world, Some(binding), false, true)' in main
assert 'if state.brush_mode != BrushMode::Tile' in main
assert 'world_stroke_last_center' in main
assert 'fn world_stroke_apply_segment(' in main
assert 'fn world_stroke_apply_single_cell(' in main
assert 'state.world_stroke_seen.insert(world)' in main
assert 'world_stroke_apply_segment(state, world)' in main
assert 'local visual override' in main.lower()
assert 'Pixel Brush cannot write to world tiles' in main
assert 'Scene painting requires Tile Brush' in main
assert 'Pixel erasing requires a project-owned derived image document' in main
assert 'if state.brush_mode == BrushMode::Tile' in main
assert 'source_rect[2] == 32' in main and 'source_rect[3] == 32' in main
for f in ('brush_mode', 'terrain_brush_role', 'tile_brush_radius_tiles', 'terrain_brush_radius_tiles', 'pixel_brush_size_px'):
    assert f in layout
assert 'layout.terrain_brush_radius_tiles.clamp(1, 4)' in layout
assert 'layout.tile_brush_radius_tiles.clamp(1, 4)' in layout
assert 'layout.pixel_brush_size_px.clamp(1, 32)' in layout
assert 'save_editor_layout(state);' in main
assert 'WorldTerrainBrush::from_role(&editor_layout.terrain_brush_role)' in main
assert 'premultiply_elizawy_sheet_texture' in main
assert 'PREMULTIPLIED-ALPHA CONTRACT' in alpha
assert main.count('egui::Panel::right("havenwild.inspector.rail")') == 1
assert 'fn canvas_layer_rect(' in main and 'draw_layer_rail(ui, &mut state);' in main
print('M2D107 CANVAS/BRUSH CONTRACT: PASS / one resizable three-section rail; integrated context; typed Pixel/Tile/Terrain; tile visual-only; terrain semantic; guarded Pixel; shared stroke; M2D106 alpha retained')
