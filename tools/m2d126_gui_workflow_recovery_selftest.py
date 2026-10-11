#!/usr/bin/env python3
"""M2D126 source-contract gate; never substitutes native Rust compile/UI testing."""
from pathlib import Path
import json,re

def rust_compact(text: str) -> str:
    return re.sub(r"\s+", "", text)

root=Path(__file__).resolve().parents[1]
s=(root/'src/main.rs').read_text('utf8')
l=(root/'src/editor_layout.rs').read_text('utf8')
p=json.loads((root/'project/forgepy.project.json').read_text('utf8'))
assert tuple(map(int,p['sourceVersion'].split('.'))) >= (0,10,44)
assert all(x in s for x in ('fn draw_authoring_tooling(', 'fn draw_authoring_brush_settings(', 'fn draw_tool_context(', 'fn authoring_splitter('))
a=s.split('fn draw_authoring_tooling(',1)[1].split('fn draw_authoring_brush_settings(',1)[0]
rail=s.split('fn draw_tool_rail(',1)[1].split('fn request_occupied_source_cells(',1)[0]
for name in ('RailIcon::Select','RailIcon::Pixel','RailIcon::Tile','RailIcon::Terrain',
             'RailIcon::Erase','RailIcon::Sample','RailIcon::Bridge'):
    assert name in rail, name
assert 'choose_brush(state, mode);' in rail
assert '"ACTIVE TOOL"' in a
assert 'choose_brush(state, mode);' not in a
assert 'ui.button("Browse Objects")' in a
assert 'state.editor_layout.authoring_library_tab = "Assets".into();' in a
assert 'Color wheel and palette are in Pixel Studio below' in a
assert 'ui.color_edit_button_srgba(&mut color)' in s
assert 'Derived-image painting remains disabled' in a
c=s.split('fn draw_tool_context(',1)[1].split('fn draw_inspector_rail(',1)[0]
assert 'draw_authoring_tooling(ui, state)' in c
assert 'draw_authoring_brush_settings(ui, state)' in c
assert 'authoring_splitter(ui,&mutstate.editor_layout.authoring_top_height' in rust_compact(c)
assert 'draw_pixel_studio(ui, state)' in c
assert 'draw_scene_navigator(ui, state)' in c
assert '"havenwild.studio.mode_tools"' in c and '"havenwild.studio.pixel"' in c
b=s.split('fn draw_library_dock(',1)[1].split('fn draw_floating_library(',1)[0]
for token in ('egui::Panel::bottom("havenwild.library.bottom")','egui::Sense::click_and_drag()','egui::CursorIcon::ResizeVertical','response.drag_delta().y', 'response.drag_stopped()', 'state.editor_layout.library_open = !state.editor_layout.library_open;', 'draw_authoring_library(ui, state);'):
    assert token in b,token
assert b.index('response.drag_delta().y') < b.index('response.drag_stopped()')
assert 'if !state.editor_layout.library_open || state.editor_layout.library_floating { return; }' not in b
assert 'state.editor_layout.library_height = next;' in b
assert 'state.editor_layout.library_floating = true;' in b
assert s.count('draw_library_dock(&mut viewport_ui, &mut state);')==2
assert 'egui::Panel::left("havenwild.layers.dock")' not in s
assert 'draw_layer_rail(ui, &mut state);' in s
assert 'authoring_recovery_v1_initialized' in l and 'if !layout.authoring_recovery_v1_initialized {' in l
assert 'layout.tool_context_open = true;' in l
assert 'recovered_authoring_workspace_migrates_once_and_preserves_user_collapse' in l
assert 'tool_context_open: true,' in l
assert 'max_height' in b and 'preferred' in b
print('M2D126 GUI RECOVERY: PASS / upgraded integrated Pixel+Game+Navigator, unboxed Layers, shared resizable Library, no art/PIE changes')
