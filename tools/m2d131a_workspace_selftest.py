#!/usr/bin/env python3
"""M2D131A source-level workspace regression contract (native Windows test still required)."""
from pathlib import Path
import json
import re
root = Path(__file__).resolve().parents[1]
main = (root/'src/main.rs').read_text('utf-8')
layout = (root/'src/editor_layout.rs').read_text('utf-8')
workspace = (root/'src/workspace_layout.rs').read_text('utf-8')
metadata = json.loads((root/'project/forgepy.project.json').read_text('utf-8'))
assert tuple(map(int, metadata['sourceVersion'].split('.'))) >= (0, 10, 49)
assert 'version = "' + metadata['sourceVersion'] + '"' in (root/'Cargo.toml').read_text('utf-8')
for source in ('world_doc.rs','world_pie.rs','asset_authority.rs','tiled_terrain.rs','worldgen_plan.rs'):
    assert (root/'src'/source).is_file(), source
for token in ('pub struct WorkspaceLayout', 'pub enum DockNode', 'pub enum DockPanel',
              'fn validate(&self)', 'fn pie_profile(&self)', 'pub fn tab_with', 'pub fn split_next_to',
              'pub fn float', 'fn nesting_and_tab_moves_are_atomic_and_persisted',
              'fn floating_does_not_duplicate_a_panel'):
    assert token in workspace, token
for token in ('pub workspace: WorkspaceLayout', 'layout.workspace = layout.workspace.restored_or_default();',
              'fn legacy_layout_receives_valid_persisted_docking',
              'fn workspace_roundtrip_keeps_existing_brush_and_asset_settings'):
    assert token in layout, token
for token in ('mod workspace_layout;', 'fn draw_console_dock(', 'fn draw_canvas_animation_toggle(',
              'fn canvas_animation_control_rect(', 'state.canvas_animation_rect.contains(pointer)',
              'fn draw_animation_timeline(', 'draw_layer_rail(ui, &mut state);',
              'draw_canvas_animation_toggle(ui, &mut state);',
              'draw_console_dock(&mut viewport_ui, &mut state);'):
    assert token in main, token
# The Console and timeline must both be reserved AFTER the authoring side rails,
# so neither permanently spans the full screen.
sidebars = main.rindex('draw_inspector_rail(&mut viewport_ui, &mut state);')
console = main.index('draw_console_dock(&mut viewport_ui, &mut state);')
timeline = main.index('draw_animation_timeline(&mut viewport_ui, &mut state);')
central = main.index('egui::CentralPanel::default()')
assert sidebars < console < timeline < central
assert 'rail_icon_button(ui, RailIcon::Apps' not in main
assert 'rail_icon_button(ui, RailIcon::Layers' not in main
assert 'ui.menu_button("Tools", |ui|' in main and 'state.launcher_open = !state.launcher_open' in main
assert '.button("Havenwild Tool Launcher    Ctrl+Space")' in main
assert re.search(r'input\.modifiers\.ctrl\s*&&\s*input\.key_pressed\(egui::Key::Space\)', main)
assert 'draw_app_launcher(ctx, &mut state);' in main
# Migration must default to hidden, without forcing a reset for already migrated layouts.
assert re.search(r'if\s+!layout\.studio_convergence_v1_initialized\s*\{[\s\S]*?layout\.timeline_open\s*=\s*false\s*;', layout), 'stale timeline auto-open migration'
assert 'reopened.timeline_open = true' in layout and 'assert!(retained.timeline_open)' in layout
assert 'state.editor_layout.workspace.right.open' in main
assert 'state.editor_layout.workspace.right.extent' in main
assert '.resizable(true)' in main.split('fn draw_inspector_rail(',1)[1].split('fn canvas_layer_rect(',1)[0]
# PIE retains Console but avoids invoking timeline and authoring panes.
assert re.search(r'if state.playtest.is_none\(\) && state.world_playtest.is_none\(\)\s*\{\s*draw_animation_timeline', main)
assert main.count('state.canvas_animation_rect.contains(pointer)') == 2
assert 'ctx.request_repaint()' in main
print('M2D131A WORKSPACE: PASS / persisted nested-dock model, resizable Inspector, Console+PIE, canvas Animation, input exclusion')
