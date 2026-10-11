#!/usr/bin/env python3
"""UA-01/P1 deterministic source-contract test; native Rust build is still required."""
from pathlib import Path
import json,re
ROOT=Path(__file__).resolve().parents[1]
main=(ROOT/'src/main.rs').read_text('utf8')
layout=(ROOT/'src/editor_layout.rs').read_text('utf8')
manifest=json.loads((ROOT/'project/forgepy.project.json').read_text('utf8'))
assert tuple(map(int, manifest['sourceVersion'].split('.'))) >= (0,10,33)
assert f'version = "{manifest["sourceVersion"]}"' in (ROOT/'Cargo.toml').read_text('utf8')
assert f'version = "{manifest["sourceVersion"]}"' in (ROOT/'Cargo.lock').read_text('utf8')
for token in ('authoring_top_height','authoring_middle_height','authoring_context_width','authoring_library_tab','pixel_palette_color'):
    assert token in layout
for token in ('"havenwild.studio.mode_tools"','"havenwild.library.bottom"','fn draw_authoring_tooling','fn draw_authoring_brush_settings','fn draw_authoring_library','fn draw_pixel_studio','fn draw_scene_navigator'):
    assert token in main,token
assert 'egui::Sense::drag()' in main
assert 'fn draw_authoring_atlas' in main
assert '.show_rows(ui, cell_size, rows' in main
assert 'state.selected_extent = [1, 1];' in main
assert 'state.selected_cell = cell;' in main
assert 'choose_brush(state, BrushMode::Tile);' in main
assert 'source_rect[2] == 32' in main
assert 'Original' in main
assert 'Source Browser' in main # advanced browser remains; no destructive removal
assert 'state.pending_source_sheet = Some(path);' in main
assert 'fn zoom_canvas_at(' in main and 'fn zoom_canvas_by(' in main
assert main.count('zoom_canvas_by(state, pointer, canvas, (wheel / 240.0).exp())')==2
assert 'fn draw_canvas_view_controls(' in main
assert 'zoom_canvas_by(state, canvas.center(), canvas, 1.25);' in main
assert 'zoom_canvas_by(state, canvas.center(), canvas, 0.8);' in main
assert 'Key::Num1' in main and 'Key::Num2' in main and 'Key::Num3' in main
assert 'Key::Plus' in main and 'Key::Equals' in main and 'Key::Minus' in main
assert 'mod unified_canvas_zoom_tests' in main
assert 'fn draw_layer_rail(' in main and 'fn draw_inspector_rail(' in main
assert 'fn draw_mapper_surface(' in main # expert workflow remains
assert 'fn premultiply_elizawy_sheet_texture(' in main
assert 'fn world_stroke_apply_segment(' in main
print('M2D112 UNIFIED AUTHORING P1: PASS / persistent tooling and brushes + relocated Library, atlas picker, zoom, shortcuts, immutable source')

# M2D113 native failure prevention: opening the unified dock by default is
# intentional, and the historical unobstructed test must not contradict it.
assert 'tool_context_open: true,' in layout
assert 'assert!(layout.tool_context_open);' in layout
assert 'fn unified_authoring_collapse_and_reopen_are_persistent()' in layout
assert 'assert!(!restored.tool_context_open);' in layout
assert 'assert!(EditorLayout::load(&path).tool_context_open);' in layout
print('M2D126 AUTHORING RECOVERY: PASS / integrated Tooling and Context restored; user collapse/reopen persists')
