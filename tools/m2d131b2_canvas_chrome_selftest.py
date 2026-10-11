#!/usr/bin/env python3
"""M2D131B2 regression: opaque canvas groups, shared views, state+input authority.

This is a *structural* source check, not a native Rust GUI or pixel screenshot test.
"""
import json
from pathlib import Path
import re
from source_version_contract import project_matches_cargo

ROOT = Path(__file__).resolve().parents[1]
main = (ROOT/'src/main.rs').read_text(encoding='utf-8')
chrome = (ROOT/'src/canvas_chrome.rs').read_text(encoding='utf-8')
layout = (ROOT/'src/editor_layout.rs').read_text(encoding='utf-8')
cargo = (ROOT/'Cargo.toml').read_text(encoding='utf-8')
project = json.loads((ROOT/'project/forgepy.project.json').read_text())
assert project_matches_cargo(project, cargo)
assert project['sourceVersion'] == '0.10.57'
assert 'mod canvas_chrome;' in main
assert 'fn draw_canvas_view_controls(' in main
assert len(re.findall(r'canvas_chrome::overlay_geometry\s*\(\s*canvas\s*,', main)) == 3, 'both canvases reserve geometry and renderer uses it'
assert main.count('state.canvas_grid_rect.contains(pointer)') == 2, 'scene/world hit tests exclude Grid and Chunks'
assert main.count('state.canvas_controls_rect.contains(pointer)') == 2, 'scene/world hit tests exclude Zoom'
assert 'state.canvas_animation_rect.contains(pointer)' in main and 'state.canvas_layers_rect.contains(pointer)' in main
# View toggles are canvas-owned, not duplicated as top command actions.
command = main.split('egui::Panel::top("havenwild.command.toolbar")',1)[1].split('egui::Panel::bottom("havenwild.status.chrome")',1)[0]
assert '.selectable_label(state.editor_layout.chunk_grid_visible, "Chunks")' not in command
assert '.selectable_label(state.editor_layout.tile_grid_visible, "Tiles")' not in command
controls = main.split('fn draw_canvas_view_controls(',1)[1].split('fn canvas_animation_control_rect(',1)[0]
for token in ('"Grid"','"Chunks"','"Rulers"','"Fit"','"−"','"+"',
              'state.editor_layout.tile_grid_visible = !state.editor_layout.tile_grid_visible',
              'state.editor_layout.chunk_grid_visible = !state.editor_layout.chunk_grid_visible',
              'zoom_canvas_by(state, canvas.center()', 'save_editor_layout(state)'):
    assert token in controls, token
assert 'state.display_fps' not in controls, 'FPS must be bottom status only'
worldpart = main.split('fn draw_world_workspace(',1)[1].split('fn open_house_room_editor_for(',1)[0]
scenepart = main.split('fn draw_world(ui:',1)[1].split('fn draw_mask_preview(',1)[0]
assert 'havenwild.world.view.controls' not in worldpart
assert 'havenwild.canvas.view.controls' not in scenepart
status = main.split('egui::Panel::bottom("havenwild.status.chrome")',1)[1].split('egui::CentralPanel::default()',1)[0]
assert '"{:.0} FPS"' in status, 'live FPS must live on existing bottom status bar'
assert 'pub editor_theme: String' in layout
assert '"North Star".to_owned()' in layout
assert 'editor_theme: default_editor_theme()' in layout
for theme in ('North Star','Slate','High Contrast'):
    assert theme in chrome
assert 'ForgeTheme' in main and 'apply_creator_visuals(ctx, &state.theme)' in main
assert 'canvas_chrome::apply_theme(ctx, &state.editor_layout.editor_theme)' in main
assert 'ui.menu_button("Theme",' in main
assert 'state.creator_visuals_applied = false' in main
# Every overlay color is opaque RGB by construction, not translucent RGBA.
assert 'from_rgba_unmultiplied' not in chrome
assert 'fn themes_and_overlay_chrome_are_fully_opaque' in chrome
assert 'fn grid_hides_at_intermediate_width_without_hitting_zoom' in chrome
assert 'fn overlay_zones_do_not_overlap_or_escape_a_desktop_canvas' in chrome
assert 'let theme = state.editor_layout.editor_theme.clone();' in main
assert 'canvas_chrome::tool_group(ui, &theme, |ui|' in main, 'Animation chrome must clone theme before layout mutation'
assert 'canvas_chrome::toggle(ui, if state.editor_layout.timeline_playing' in main
assert 'draw_canvas_view_controls(ui, &mut state)' in main
print('M2D131B2 CANVAS CHROME: PASS / opaque themed buttons, shared world+scene zoom, canvas Grid/Chunks, status FPS, hit test, mode-safe input')
