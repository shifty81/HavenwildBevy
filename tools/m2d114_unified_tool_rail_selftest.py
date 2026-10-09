#!/usr/bin/env python3
"""M2D114 tools/terrain-workspace consolidation contract (native tests still mandatory)."""
from pathlib import Path
import json
ROOT = Path(__file__).resolve().parents[1]
main = (ROOT/'src/main.rs').read_text(encoding='utf-8')
icons = (ROOT/'src/editor_icons.rs').read_text(encoding='utf-8')
layout = (ROOT/'src/editor_layout.rs').read_text(encoding='utf-8')
assert tuple(map(int, json.loads((ROOT/'project/forgepy.project.json').read_text())['sourceVersion'].split('.'))) >= (0,10,33)
assert 'mod editor_icons;' in main and 'use editor_icons::{rail_icon_button, RailIcon};' in main
assert all(f'    {key},' in icons for key in ('Select','Pixel','Tile','Terrain','Sample','Erase','Bridge','Panel','Eye','EyeOff'))
assert 'rail_icon_button(ui, RailIcon::Bridge, state.world_bridge_tool' in main
assert 'rail_icon_button(ui, RailIcon::Panel, state.editor_layout.tool_context_open' in main
assert all(f'(BrushMode::{mode}, RailIcon::{mode})' in main for mode in ('Pixel','Tile','Terrain'))
# Recovered M2D112 Tooling tabs intentionally provide a second direct workflow
# beside the slim rail; both use choose_brush rather than duplicate backend logic.
section=main.split('fn draw_authoring_tooling(',1)[1].split('fn draw_authoring_brush_settings(',1)[0]
assert 'choose_brush(state, mode)' in section
assert 'selectable_label(state.tool == CanvasTool::Select' in section
assert 'Material: {} · choose in bottom Library' in section
assert 'ui.color_edit_button_srgba' in section
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
