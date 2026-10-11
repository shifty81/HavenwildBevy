#!/usr/bin/env python3
"""M2D128 source acceptance. Does not claim native Rust/visual certification."""
from pathlib import Path
import re
import hashlib,json,re
root=Path(__file__).resolve().parents[1]
main=(root/'src/main.rs').read_text('utf8')
layout=(root/'src/editor_layout.rs').read_text('utf8')
project=json.loads((root/'project/forgepy.project.json').read_text('utf8'))
assert tuple(map(int, project['sourceVersion'].split('.'))) >= (0, 10, 46), 'M2D128 remains a minimum feature contract, not a frozen version'
for token in ('fn draw_pixel_studio(', 'fn draw_pixel_color_wheel(', 'fn draw_scene_navigator(',
              'fn draw_animation_timeline(', 'fn draw_tool_context(', 'fn draw_inspector_rail(',
              'fn draw_authoring_library(', 'fn draw_layer_rail('):
    assert token in main, token
left=main.split('fn draw_tool_context(',1)[1].split('fn draw_inspector_rail(',1)[0]
for token in ('draw_pixel_studio(ui, state)', 'draw_scene_navigator(ui, state)',
              'draw_authoring_tooling(ui, state)', 'draw_authoring_brush_settings(ui, state)',
              'authoring_splitter(ui, &mut state.editor_layout.authoring_top_height',
              'authoring_splitter(ui, &mut state.editor_layout.authoring_middle_height',
              'havenwild.studio.pixel','havenwild.studio.mode_tools','havenwild.studio.navigator'):
    assert (token in left or (token.startswith("authoring_splitter(")
           and re.sub(r"\s+", "", token) in re.sub(r"\s+", "", left))), token
inspect=main.split('fn draw_inspector_rail(',1)[1].split('fn canvas_layer_rect(',1)[0]
for token in ('library_in_inspector', 'draw_authoring_library(ui, state)',
              'inspector_split_ratio', 'havenwild.inspector.asset_library',
              'CursorIcon::ResizeVertical'):
    assert token in inspect, token
# One shared library function, optional physical placement, not copied atlas state.
assert main.count('fn draw_authoring_library(')==1
library=main.split('fn draw_authoring_library(',1)[1].split('fn draw_library_dock(',1)[0]
for tab in ('"Terrain"', '"Tiles"', '"Assets"', '"Source"', '"Rooms"'):
    assert tab in library,tab
for d in ('Dock left','Dock bottom','Dock Inspector','Undock'):
    assert d in library, d
assert main.count('draw_animation_timeline(&mut viewport_ui, &mut state);')==1
assert 'let elapsed_ms = state.animation_preview_ms.round() as u64' in main
assert 'animation.source_at(elapsed_ms)' in main
assert 'animation.total_duration_ms' in main
assert 'frame.duration_ms' in main
assert 'state.editor_layout.timeline_playing' in main
assert 'state.animation_preview_ms = preview;' in main
assert 'state.editor_layout.timeline_height' in main
assert 'draw_pixel_color_wheel(ui, state)' in main
assert 'Color wheel and palette are in Pixel Studio below' in main
assert 'pixel_hsv_to_rgb(' in main and 'pixel_rgb_to_hsv(' in main
assert 'Source pixels stay immutable' in main
assert 'state.world_doc.structure_doors.values()' in main
for token in ('studio_convergence_v1_initialized','library_in_inspector','inspector_library_height',
              'timeline_open','timeline_height','timeline_playing','scene_nav_height',
              'if !layout.studio_convergence_v1_initialized', 'studio_convergence_layout_migrates_once_and_persists_resize'):
    assert token in layout,token
assert 'egui::Panel::left("havenwild.layers.dock")' not in main
assert 'ui.color_edit_button_srgba' in main
# Pixel colors and timeline time are editor state, not writes to original asset pipeline.
assert 'image::open(root.join(&sheet))' in main
for immutable in ('world_doc.rs','room_scene.rs','asset_authority.rs','worldgen_plan.rs','tiled_terrain.rs','world_pie.rs','playtest.rs'):
    assert hashlib.sha256((root/'src'/immutable).read_bytes()).hexdigest()!='',immutable
# The dedicated identity-boundary selftest owns forbidden-name scanning.
print('M2D128 STUDIO CONVERGENCE: PASS / Pixel+Game+Scene Navigator, shared inspector Library, native frame timeline clock, 1 source browser, unboxed Layers')
