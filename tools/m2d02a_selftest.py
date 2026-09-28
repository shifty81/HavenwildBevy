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
 'tool: CanvasTool::Select', 'fn draw_tool_rail(', 'fn draw_layer_guide(',
 'fn save_scene(', 'fn sample_canvas_source(', 'fn draw_alpha_checker(',
 'state.canvas_controls_rect.contains(pointer)',
 'state.canvas_selection_rect.contains(pointer)',
 'state.editor_layout.layers_open', 'state.focus_mode',
 'state.tool = CanvasTool::Paint;',
 'ui.add_enabled(false, egui::Button::new("Regenerate selection / scene"))',
 'ui.add_enabled(false, egui::Button::new("Whole-scene Pixel mode (M2D03)"))',
]
assert all(s in main for s in required), [s for s in required if s not in main]
assert 'edit_mode' not in main and 'Stamp source tile on click' not in main
assert all(s in layout for s in ('pub layers_open: bool', 'pub inspect_alpha: bool', 'terrain_mapper.visible = false', 'previous_layout_json_loads_new_optional_fields_as_false'))
assert '.wants_keyboard_input(' not in main, 'unsupported egui API must not ship'
assert main.count('.egui_wants_keyboard_input()') == 3, 'Delete/Undo/Redo guards missing'
print('M2D02-A SELFTEST: PASS / safe Select default, rail outside canvas, truthful layers/generation, alpha inspect, pinned egui keyboard API')
