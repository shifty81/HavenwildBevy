#!/usr/bin/env python3
"""M2D101 source checks. These do not replace Cargo or interactive acceptance."""
from pathlib import Path
import re
root = Path(__file__).resolve().parents[1]
main = (root / 'src/main.rs').read_text(encoding='utf-8')
world = (root / 'src/world_doc.rs').read_text(encoding='utf-8')
for token in (
    'fn draw_inspector_rail(', 'egui::Panel::right("havenwild.inspector.rail")',
    'fn draw_layer_rail(', 'fn canvas_layer_rect(',
    'egui::Panel::left("havenwild.tool.rail")',
    'const RAIL_WIDTH: f32 = 62.0',
    'fn draw_tool_context(',
    'state.canvas_layers_rect.contains(pointer)',
    'world_dragging_object: Option<(String, [i64; 2])>',
    'world_drag_preview_anchor: Option<[i64; 2]>',
    'state.world_doc.can_move_generated_object_to(&id, to)',
    'state.world_doc.move_generated_object(&id, to)',
    'checkpoint_world(state)', 'world_object_visual_bounds(object, anchor)',
    'state.world_doc.object_anchor(object)',
    'format!("{:.0}%  ·  {:.0} FPS", state.zoom * 100.0, state.display_fps)',
):
    assert re.sub(r"\s+", "", token) in re.sub(r"\s+", "", main), f'M2D101 missing main source contract: {token}'
assert 'egui::Panel::right("havenwild.layer.rail")' not in main
assert 'egui::Window::new("Object properties · on demand")' not in main
for token in (
    '#[serde(default)]\n    pub moved_generated_objects: BTreeMap<String, [i64; 2]>',
    'pub fn can_move_generated_object_to(',
    'pub fn move_generated_object(',
    'pub fn object_anchor(',
    'self.moved_generated_objects.remove(id);',
    'moved_objects_keep_original_procedural_anchor_and_serialize',
    'world_move_rejects_outside_and_suppressed_objects',
    'prior_world_documents_default_missing_move_metadata',
):
    assert re.sub(r"\s+", "", token) in re.sub(r"\s+", "", world), f'M2D101 missing world model contract: {token}'

# M2D104 native compiler regression guards: live generated-object iterator must
# be dropped before inspector actions mutate Studio/WorldDocument. Cargo remains
# the definitive compiler check; these assertions are only an early warning.
compact_main = re.sub(r"\s+", "", main)
inspector = compact_main.split('fndraw_inspector_rail(', 1)[1].split('fndraw_layer_rail(', 1)[0]
assert 'letselected_object=state.world_doc.generated_objects().find(|object|object.id==id).cloned();' in inspector, 'inspector must close immutable object lookup before mutation (E0502)'
assert 'ifletSome(object)=selected_object{' in inspector, 'inspector object snapshot not used'
assert 'ifletSome(object)=state.world_doc.generated_objects()' not in inspector, 'inspector holds live generator iterator across mutations'
status = compact_main.split('egui::Panel::bottom("havenwild.status.chrome")', 1)[1].split('draw_tool_rail(&mutviewport_ui', 1)[0]
assert 'save_editor_layout(&mutstate)' in status, 'status Metrics must pass mutable Studio (E0308)'

print('M2D101 SOURCE CONTRACT: PASS / 62px icon rail, canvas-integrated layers, object overrides, source-preserving drag')
