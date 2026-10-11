#!/usr/bin/env python3
"""M2D124 regression against reinstating the unwanted Layers sidebar/tooling slab.

This protects source contracts; actual Rust build and DPI/visual input tests remain mandatory.
"""
from pathlib import Path
import re
root=Path(__file__).resolve().parents[1]
main=(root/'src/main.rs').read_text(encoding='utf-8')
layout=(root/'src/editor_layout.rs').read_text(encoding='utf-8')
icons=(root/'src/editor_icons.rs').read_text(encoding='utf-8')
assert 'fn canvas_layer_rect(' in main and 'fn draw_layer_rail(canvas_ui: &mut egui::Ui' in main
assert 'egui::Panel::left("havenwild.layers.dock")' not in main
assert 'egui::Area::new(egui::Id::new("havenwild.canvas.layer.rail"))' not in main
assert 'egui::Order::Foreground' not in main.split('fn draw_layer_rail(',1)[1].split('fn docked_overlay_window(',1)[0]
assert main.index('draw_layer_rail(ui, &mut state);') > main.index('egui::CentralPanel::default()')
assert 'canvas_ui.scope_builder(' in main and 'egui::Frame::NONE.fill(' not in main.split('fn draw_layer_rail(',1)[1].split('fn docked_overlay_window(',1)[0]
assert main.count('state.editor_layout.layers_dock_open,') >= 2 and 'fn canvas_layer_rect(canvas: egui::Rect, expanded: bool, rulers: bool)' in main
assert main.count('state.canvas_layers_rect.contains(pointer)') >= 2
assert 'state.editor_layout.world_layer_visibility[slot] = !visible;' in main
# Shift-isolate is reversible and lives in the session-only helper, not an inline
# visibility reset that permanently discards previously hidden layer preferences.
layer=(root/'src/layer_overlay.rs').read_text(encoding='utf-8')
assert 'state.layer_isolate.toggle' in main
assert '*visibility = [false; 7];' in layer and '*visibility = saved;' in layer
assert 'fn draw_library_dock(' in main and 'fn draw_floating_library(' in main
assert re.search(r'if\s+!state\.editor_layout\.tool_context_open\s*\{\s*return;\s*\}', main)
section=main.split('fn draw_tool_context(',1)[1].split('fn draw_inspector_rail(',1)[0]
assert '"havenwild.studio.mode_tools"' in section
assert '"havenwild.studio.pixel"' in section
assert '"havenwild.studio.navigator"' in section
assert '"havenwild.authoring.brush"' not in section
assert 'draw_authoring_tooling(ui, state)' in section and 'draw_authoring_brush_settings(ui, state)' in section
assert 'tool_context_open: true,' in layout and 'canvas_hud_v1_initialized: true,' in layout
assert 'if !layout.canvas_hud_v1_initialized {' in layout and 'layout.tool_context_open = false;' in layout
assert 'RailIcon::Select' in main and 'RailIcon::Pixel' in main and 'RailIcon::Tile' in main and 'RailIcon::Terrain' in main
assert all(v in icons for v in ('Select,','Pixel,','Tile,','Terrain,','Layers,','EyeOff,'))
# Explicitly report the unavailable derived-image write backend; this must
# never accidentally route pixel tools into world terrain painting.
assert 'Derived-image painting remains disabled' in main
assert 'Image-pixel strokes require a derived-image document' in main
assert 'Never route a Pixel stroke to the world semantic' in main
print('M2D124+M2D126 GUI RECOVERY: PASS / unboxed canvas layers, useful integrated Tooling and Tool Context, preserved library/editor source')
