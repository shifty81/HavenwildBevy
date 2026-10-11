#!/usr/bin/env python3
"""M2D131B3 guard: live UI ownership and no destructive temporary views.
Native Rust compile and actual mouse/PIE screenshots are separate gates.
"""
from pathlib import Path
import re, json
root=Path(__file__).resolve().parents[1]
main=(root/'src/main.rs').read_text(encoding='utf-8')
layout=(root/'src/editor_layout.rs').read_text(encoding='utf-8')
layer=(root/'src/layer_overlay.rs').read_text(encoding='utf-8')
project=json.loads((root/'project/forgepy.project.json').read_text())
assert project['sourceVersion']=='0.10.57'
assert 'persisted_visibility(state.editor_layout.world_layer_visibility)' in main
assert 'disk_layout.save(&state.layout_path)' in main
assert 'fn isolating_then_saving_an_unrelated_setting_preserves_disk_visibility' in layer
assert 'fn m2d131b3_single_dock_and_inspector_tab_migrate_once' in layout
assert 'fn m2d131b3_custom_width_is_not_overwritten_by_legacy_upgrade' in layout
# One left 15% dock surface; preserved tools are now inside its three sections.
assert 'egui::Panel::left("havenwild.tool.rail")' not in main
left=main.split('fn draw_tool_context(',1)[1].split('fn draw_inspector_rail(',1)[0]
for token in ('egui::Panel::left("havenwild.tool.context")', 'draw_tool_rail(ui, state)',
              'draw_authoring_tooling(ui, state)', 'draw_authoring_brush_settings(ui, state)',
              'draw_pixel_studio(ui, state)', 'draw_scene_navigator(ui, state)',
              '"MODE & TOOLS"',
              'authoring_top_height', 'authoring_middle_height'):
    assert token in left,token
assert '"PIXEL STUDIO"' in main.split('fn draw_pixel_studio(',1)[1].split('fn draw_scene_navigator(',1)[0]
assert '"SCENE NAVIGATOR"' in main.split('fn draw_scene_navigator(',1)[1].split('fn draw_console_dock(',1)[0]
assert 'draw_tool_rail(&mut viewport_ui, &mut state)' not in main
# Single inspector tabs, no floating legacy World Properties.
inspector=main.split('fn draw_inspector_rail(',1)[1].split('fn canvas_layer_rect(',1)[0]
for token in ('"Inspect", "World", "Terrain"', 'draw_world_properties_contents(ui, state)',
              'draw_authoring_library(ui, state)', 'state.editor_layout.inspector_split_ratio'):
    assert token in inspector,token
assert 'egui::Window::new("World Properties")' not in main
assert 'draw_structural_panel(&mut viewport_ui, &mut state)' not in main
assert 'fn draw_world_properties_contents(' in main
assert 'SceneCommand::SetStructuralCell' in main
assert 'draw_world_structural_contents(ui, state)' in main
assert 'pub inspector_tab: String' in layout
# Right-click is positional but not an implicit Select command.
ctx=main.split('if response.secondary_clicked() {',1)[1].split('response.context_menu(|ui| {',1)[0]
assert 'state.world_context_cell = Some(world);' in ctx
assert 'state.world_selected = world;' not in ctx
menu=main.split('response.context_menu(|ui| {',1)[1].split('let visible = canvas.intersect(region);',1)[0]
assert 'state.world_context_cell.unwrap_or(state.world_selected)' in menu
assert 'state.world_selected = cell;' in menu
assert 'checkpoint_world(state)' in menu
assert 'world_object_at(state, cell)' in menu
print('M2D131B3 GUI CONVERGENCE: PASS / one left dock, World Inspector tab, persistent-isolation guard, non-destructive right click, right Asset Browser')
