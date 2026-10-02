#!/usr/bin/env python3
"""Read-only source-contract test. Not a Rust compiler or GUI interaction test."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
layout = (ROOT / 'src/editor_layout.rs').read_text(encoding='utf-8')
scene = json.loads((ROOT / 'content/scenes/summer_river.scene.json').read_text(encoding='utf-8'))
assert scene['schema'] == 'havenwild.elizawy.scene.v1'
assert scene['size'] == [40,28] and len(scene['tiles']) == 1120
required = [
 'tool: CanvasTool::Select', 'fn draw_tool_rail(', 'fn draw_layer_rail(', 'fn draw_layer_panel(',
 'fn save_scene(', 'fn sample_canvas_source(', 'fn draw_alpha_checker(',
 'state.canvas_controls_rect.contains(pointer)',
 'state.editor_layout.layers_open', 'state.focus_mode',
 'state.tool = CanvasTool::Paint;',
 'fn draw_app_launcher(', 'Pixel Studio · planned', 'World Generator · next milestone',
]
assert all(s in main for s in required), [s for s in required if s not in main]
assert 'havenwild.canvas.selection.actions' not in main, 'selection controls must not block the canvas'
assert 'edit_mode' not in main and 'Stamp source tile on click' not in main
assert all(s in layout for s in ('pub layers_open: bool', 'pub inspect_alpha: bool', 'terrain_mapper.visible = false', 'previous_layout_json_loads_new_optional_fields_safely'))
assert '.wants_keyboard_input(' not in main, 'unsupported egui API must not ship'
assert main.count('.egui_wants_keyboard_input()') >= 3, 'Delete/Undo/Redo guards missing'
print('M2D02-A SELFTEST: PASS / safe Select default, permanent tool+layer rails, overlay applications, truthful generation status, alpha inspect, pinned egui keyboard API')
