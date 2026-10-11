#!/usr/bin/env python3
"""M2D120 static/document contract tests; cannot replace a native Windows Cargo build."""
from __future__ import annotations
import json
import re

def rust_compact(text: str) -> str:
    return re.sub(r"\s+", "", text)

from pathlib import Path
root=Path(__file__).resolve().parents[1]
main=(root/'src/main.rs').read_text(encoding='utf8')
author=(root/'src/room_authoring.rs').read_text(encoding='utf8')
world=(root/'src/world_pie.rs').read_text(encoding='utf8')
room=json.loads((root/'content/rooms/starter_house.room.json').read_text())
project=json.loads((root/'project/forgepy.project.json').read_text())
assert tuple(map(int,project['sourceVersion'].split('.'))) >= (0,10,38)
assert f'version = "{project["sourceVersion"]}"' in (root/'Cargo.toml').read_text()
assert f'version = "{project["sourceVersion"]}"' in (root/'Cargo.lock').read_text()
assert len(room['cells']) == 4096 and room['chunks'] == [2,2]
for y in range(64):
 for x in range(64):
  idx=y*64+x; tile=room['cells'][idx]
  if tile['floor']:
   assert tile['wallNeighborMask']==0
  else:
   mask=0
   for dx,dy,bit in [(0,-1,1),(1,0,2),(0,1,4),(-1,0,8)]:
    xx,yy=x+dx,y+dy
    if 0 <= xx < 64 and 0 <= yy < 64 and room['cells'][yy*64+xx]['floor']:mask|=bit
   assert tile['wallNeighborMask']==mask,(x,y)
assert all(room['cells'][p['cell'][1]*64+p['cell'][0]]['floor'] for p in room['portals'])
for needle in (
 'mod room_authoring;', 'room_authoring: Option<room_authoring::RoomEditor>',
 'fn draw_room_authoring_workspace(', 'fn draw_room_editor_controls(',
 'fn open_house_room_editor_for(', 'fn save_house_room_editor(',
 'save_house_room_editor(state); return;', 'RoomScene::load_template(&path)',
 'write_atomic(&path, &bytes)', 'draw_room_authoring_workspace(ui, state); return;',
 'if state.room_authoring.is_some() {', '"Rooms"',
 'state.editor_layout.authoring_library_tab = "Rooms".into();',
 '"Move exit"', '2×2', '3×3', '"Paint floor"', '"Erase floor"',
 'ui.color_edit_button_srgba(&mut tint)', 'draw_room_editor_controls(ui, state);',
 'edit.pan[0] = edit.pan[0] * ratio',
 'let stones = state.world_doc.generated_objects().filter(',
 'room wall tint',
):
 if rust_compact(needle) not in rust_compact(main):
  raise AssertionError(f'Room canvas missing {needle}')
for needle in (
 'pub enum RoomTool', 'pub struct RoomEditor', 'pub fn stroke_begin(',
 'pub fn paint(', 'pub fn undo(', 'pub fn redo(',
 'pub fn set_wall_tint(', 'pub fn resize_chunks(',
 'Room brush outside scene', 'Cannot erase a portal',
 'self.scene.validate()', 'self.stroke_active = true',
 'fn whole_brush_stroke_is_one_undo_step()',
 'fn exit_floor_cannot_be_erased_and_exit_can_move_to_valid_floor()',
):assert needle in author,needle
assert 'pub fn resize_chunks(' in (root/'src/room_scene.rs').read_text()
assert 'RoomScene::load_template(&path)' in world
assert 'fn interact(' in world and 'pub fn enter_test_cave(' in world
print('M2D120 ROOM CANVAS: PASS / 2x2 source-independent house, floor painting, derived wall topology, tint, edit tools, atomic save, per-stroke undo, world PIE consumption')
print('M2D120 LIMIT: no native Rust compilation or runtime GUI test in this environment; original ElizaWy room art still unbound')
