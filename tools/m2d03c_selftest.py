#!/usr/bin/env python3
"""M2D03-C cargo-free live-scene workflow assertions (not Windows UI certification)."""
from pathlib import Path
r=Path(__file__).resolve().parents[1]
m=(r/'src/main.rs').read_text(encoding='utf-8')
v=(r/'src/scene_v2.rs').read_text(encoding='utf-8')
l=(r/'src/editor_layout.rs').read_text(encoding='utf-8')
for no_longer_exposed in ('Preview separate v2 scene draft', 'Create separate v2 draft',
                          'migration_preview: Option<SceneV2>', 'CanvasTool::Move',
                          '"MOV"', 'havenwild.canvas.selection.actions'):
    assert no_longer_exposed not in m, no_longer_exposed
assert m.count('"▶ Play Scene"')==1
assert 'start_playtest(&mut state)' in m and 'stop_playtest(&mut state)' in m
assert 'state.playtest_layered = Some(state.layered_scene.clone());' in m
assert 'Scene / Layers' in m and 'Placed objects (' in m
assert 'state.dragging_object = Some((id, grab_offset));' in m
assert 'input.pointer.primary_pressed()' in m and 'input.pointer.primary_released()' in m
assert 'state.layered_scene.can_move_object_to(&id, destination)' in m
assert 'state.drag_preview_origin.unwrap_or(object.anchor)' in m
assert 'SceneCommand::MoveObject { id: id.clone(), to }' in m
assert '"Place selected source at cell"' in m and '"Open Source Browser"' in m
assert '.save_editable_draft(' in m
assert 'state.layered_history.mark_saved()' in m
assert 'pub fn can_move_object_to(' in v
assert 'drag_preview_and_committed_object_move_use_same_scene_bounds' in v
assert 'pub scene_panel_v3_initialized: bool' in l and 'pub canvas_first_v4_initialized: bool' in l
assert 'legacy_base: source.tiles' in v and 'verify_legacy_source' in v
print('M2D03-C SELFTEST: PASS / one PIE button, one live scene, visible layer/object navigator, press-captured drag preview, bounds parity, recovery authority retained')
