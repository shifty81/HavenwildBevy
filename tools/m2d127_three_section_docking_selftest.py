#!/usr/bin/env python3
"""Source contract only; the native Windows Bevy/egui gate is still required."""
import re
import hashlib
from pathlib import Path
root=Path(__file__).resolve().parents[1]
s=(root/'src/main.rs').read_text('utf8')
l=(root/'src/editor_layout.rs').read_text('utf8')
assert 'fn draw_authoring_tooling(' in s
assert 'fn draw_authoring_brush_settings(' in s
assert 'fn draw_authoring_library(' in s
section=s.split('fn draw_tool_context(',1)[1].split('fn draw_inspector_rail(',1)[0]
for phrase in ('"LIBRARY / PALETTE"',
    'draw_pixel_studio(ui, state)',
    'draw_scene_navigator(ui, state)',
    'draw_authoring_tooling(ui, state)',
    'draw_authoring_brush_settings(ui, state)',
    'draw_authoring_library(ui, state)',
    'authoring_splitter(ui, &mut state.editor_layout.authoring_top_height',
    'authoring_splitter(ui, &mut state.editor_layout.authoring_middle_height',
    'havenwild.studio.navigator'):
    assert (phrase in section or (phrase.startswith("authoring_splitter(")
           and re.sub(r"\s+", "", phrase) in re.sub(r"\s+", "", section))), phrase
library=s.split('fn draw_authoring_library(',1)[1].split('fn draw_library_dock(',1)[0]
for label in ('"Dock left"','"Dock bottom"','"Dock Inspector"','"Undock"','"Terrain"','"Tiles"','"Assets"','"Source"','"Rooms"'):
    assert label in library, label
floating=s.split('fn draw_floating_library(',1)[1].split('fn authoring_splitter(',1)[0]
for phrase in ('fixed_size(', 'egui::Sense::drag()', 'drag_delta()',
    'CursorIcon::ResizeNwSe', 'library_floating_size[0]', 'library_floating_size[1]',
    'drag_stopped()', 'save_editor_layout(state)'):
    assert phrase in floating, phrase
assert '.resizable(false)' in floating, 'Only the explicit resize grip owns floating resize'
assert re.search(r'if\s+state\.editor_layout\.library_floating\s*\|\|', s)
assert 'state.editor_layout.library_in_left_workspace' in s
assert 'authoring_three_section_v1_initialized' in l
assert 'layout.library_floating_size' in l
assert 'three_section_workspace_and_floating_size_persist' in l
assert 'three_section_migration_runs_once_and_preserves_user_dock_choice' in l
for fname in ('world_doc.rs','room_scene.rs','asset_authority.rs','tiled_terrain.rs','worldgen_plan.rs','playtest.rs'):
    assert (root/'src'/fname).is_file(), fname
assert 'draw_animation_timeline(' in s
assert 'library_in_inspector' in l
print('M2D127 DOCK CONTRACT: PASS / one shared Library with left, bottom, floating, Inspector destinations; corner resize retained')
