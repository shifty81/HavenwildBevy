#!/usr/bin/env python3
"""M2D131D: world/scene rulers are opaque canvas chrome, not a settings-only overlay.
Source contracts only: native Rust rendering and GUI behavior need a Windows Full Gate.
"""
from pathlib import Path
from source_version_contract import project_matches_cargo
import json
import re

root = Path(__file__).resolve().parents[1]
main = (root / "src/main.rs").read_text(encoding="utf-8")
rulers = (root / "src/canvas_rulers.rs").read_text(encoding="utf-8")
chrome = (root / "src/canvas_chrome.rs").read_text(encoding="utf-8")
layout = (root / "src/editor_layout.rs").read_text(encoding="utf-8")
cargo = (root / "Cargo.toml").read_text(encoding="utf-8")
meta = json.loads((root / "project/forgepy.project.json").read_text(encoding="utf-8"))
assert meta['sourceVersion'] == '0.10.57' and project_matches_cargo(meta, cargo)
world = main.split('fn draw_world_workspace(', 1)[1].split('fn open_house_room_editor_for(', 1)[0]
scene = main.split('fn draw_world(ui:', 1)[1].split('fn draw_mask_preview(', 1)[0]
controls = main.split('fn draw_canvas_view_controls(', 1)[1].split('fn canvas_animation_control_rect(', 1)[0]
assert 'canvas_rulers::draw(' in world, 'Generated World must draw rulers, not only scenes'
assert 'canvas_rulers::draw(' in scene, 'Scene rulers must remain intact'
assert 'state.selection_active.then_some(state.world_selected)' in world
assert 'state.selection_active.then_some(state.selected_world.map(|v| v as i64))' in scene
assert 'origin,' in world.split('canvas_rulers::draw(', 1)[1].split(');', 1)[0]
assert '[0, 0],' in scene.split('canvas_rulers::draw(', 1)[1].split(');', 1)[0]
assert 'canvas_rulers::blocks_scene_edit(' in main.split('fn world_pointer_cell(', 1)[1].split('fn ', 1)[0], 'world ruler strips must not paint terrain'
assert 'canvas_rulers::blocks_scene_edit(' in main.split('fn world_at(', 1)[1].split('fn ', 1)[0], 'scene ruler strips must not paint terrain'
assert 'state.editor_layout.rulers_visible = !state.editor_layout.rulers_visible' in controls
assert all(f'"{name}"' in controls for name in ('Grid', 'Chunks', 'Rulers', 'Fit'))
assert 'save_editor_layout(state)' in controls
assert 'if canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode)' in world
assert 'if canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode)' in controls
assert 'pub const TOP: f32 = 24.0' in rulers and 'pub const LEFT: f32 = 48.0' in rulers
assert 'crate::canvas_chrome::panel_fill(theme)' in rulers
assert 'from_rgba_unmultiplied' not in rulers, 'ruler backing must be fully opaque'
assert 'x_tile.saturating_add(origin[0]).to_string()' in rulers
assert 'y_tile.saturating_add(origin[1]).to_string()' in rulers
assert 'fn signed_world_labels_keep_major_ticks_aligned_to_global_zero' in rulers
assert 'fn ruler_theme_backing_is_solid' in rulers
assert 'pub fn overlay_geometry' in chrome and 'canvas.center().x - 105.0' in chrome
assert 'rulers_visible: true' in layout
assert 'fn canvas_layer_rect(canvas: egui::Rect, expanded: bool, rulers: bool)' in main
assert 'fn canvas_animation_control_rect(canvas: egui::Rect, rulers: bool)' in main
assert 'pub fn visible(preference: bool, focus_mode: bool)' in rulers
assert not any(t in rulers for t in ('fs::write(', 'write_atomic(', 'state.scene.tiles'))
print('M2D131D CANVAS RULERS: PASS / world+scene, opaque theme, signed world ticks, toggle, hit guard, authoring unchanged')
