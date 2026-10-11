#!/usr/bin/env python3
"""M2D114 tools/terrain-workspace consolidation contract (native tests still mandatory)."""
from pathlib import Path
import json
import re
ROOT = Path(__file__).resolve().parents[1]
main = (ROOT/'src/main.rs').read_text(encoding='utf-8')
icons = (ROOT/'src/editor_icons.rs').read_text(encoding='utf-8')
layout = (ROOT/'src/editor_layout.rs').read_text(encoding='utf-8')
assert tuple(map(int, json.loads((ROOT/'project/forgepy.project.json').read_text())['sourceVersion'].split('.'))) >= (0,10,33)
assert 'mod editor_icons;' in main and 'use editor_icons::{rail_icon_button, RailIcon};' in main
assert all(f'    {key},' in icons for key in ('Select','Pixel','Tile','Terrain','Sample','Erase','Bridge','Panel','Eye','EyeOff'))
assert re.search(r'rail_icon_button\s*\(\s*ui\s*,\s*RailIcon::Bridge\s*,\s*state\.world_bridge_tool\b', main), 'Bridge rail action missing'
assert re.search(r'rail_icon_button\s*\(\s*ui\s*,\s*RailIcon::Panel\s*,\s*state\.editor_layout\.tool_context_open\b', main), 'Panel rail action missing'
assert all(f'(BrushMode::{mode}, RailIcon::{mode})' in main for mode in ('Pixel','Tile','Terrain'))
# One authoritative tool-selection strip; lower context shows live properties
# but does not repeat the same Select / Paint / Erase / Bridge actions.
rail=main.split('fn draw_tool_rail(',1)[1].split('fn request_occupied_source_cells(',1)[0]
section=main.split('fn draw_authoring_tooling(',1)[1].split('fn draw_authoring_brush_settings(',1)[0]
assert 'choose_brush(state, mode)' in rail
assert re.search(r'rail_icon_button\s*\(\s*ui\s*,\s*RailIcon::Select\s*,', rail), 'Select rail action missing'
assert 'choose_brush(state, mode)' not in section
assert re.search(r'ui\s*\.\s*button\s*\(\s*"Browse Objects"\s*\)', section)
# M2D128 moved the single shared Library to the Inspector by default; the
# terrain selection still uses the same brush state and Terrain tab.
assert 'Material: {} · choose in Asset Browser' in section
assert 'ui.small_button("Open terrain palette")' in section
assert 'state.editor_layout.authoring_library_tab = "Terrain".into();' in section
assert 'draw_authoring_library(ui, state)' in main.split('fn draw_inspector_rail(', 1)[1]
# Pixel color authority now belongs to Pixel Studio, not Game Tools.
assert 'fn draw_pixel_color_wheel(' in main
assert 'draw_pixel_color_wheel(ui, state)' in main
assert 'Color wheel and palette are in Pixel Studio below' in section
# Atlas/Terrain actions remain in their existing working canvas/library paths.
assert 'fn draw_authoring_atlas' in main and 'fn draw_authoring_library' in main
assert 'fn draw_layer_rail' in main
assert '"Live terrain rules"' in main and '"Original atlas"' in main
assert 'ui.menu_button("Advanced ▾"' in main
assert '"Terrain Brush on canvas"' in main and '"Open Tile Library"' in main
assert 'state.editor_layout.authoring_library_tab = "Source".into();' in main
assert 'fn draw_advanced_recipe_mapper(' in main
assert 'ui.strong("Terrain at selection")' in main
assert 'Locate source in Library' in main
assert 'pending_source_cell: Option<[u32; 2]>' in main
assert 'requested_cell.unwrap_or([0, 0])' in main
assert 'No exact or projected Tiled rule' in main
assert 'state.editor_layout.world_layer_visibility[slot] = !visible;' in main
assert 'tool_context_open: true' in layout
print('M2D114 TOOL RAIL & TERRAIN WORKFLOW: PASS / unique tool actions, consistent vectors, layers isolated, atlas library canonical, advanced diagnostics retained')
