#!/usr/bin/env python3
"""Cargo-free M2D119 document+code contracts; NOT a native Rust/visual test."""
from __future__ import annotations
import json
import sys
from pathlib import Path
root = Path(__file__).resolve().parents[1]
main = (root / 'src/main.rs').read_text(encoding='utf-8')
world_pie = (root / 'src/world_pie.rs').read_text(encoding='utf-8')
room_source = (root / 'src/room_scene.rs').read_text(encoding='utf-8')
room = json.loads((root / 'content/rooms/starter_house.room.json').read_text(encoding='utf-8'))
assert room['schema'] == 'havenwild.bevy.room_scene.v1'
assert room['kind'] == 'house' and room['chunks'] == [2,2]
assert room['sceneId'] == 'starter_house_template'
assert room['style']['floorAsset'] == '' and room['style']['wallAsset'] == '', 'do not misrepresent source certification'
assert len(room['cells']) == 64*64
floor = lambda x,y: 0 <= x < 64 and 0 <= y < 64 and room['cells'][y*64+x]['floor']
for y in range(64):
    for x in range(64):
        cell=room['cells'][y*64+x]
        mask = 0 if floor(x,y) else ((1 if floor(x,y-1) else 0)
            | (2 if floor(x+1,y) else 0)
            | (4 if floor(x,y+1) else 0)
            | (8 if floor(x-1,y) else 0))
        assert cell['wallNeighborMask'] == mask, (x,y,mask)
assert sum(c['floor'] for c in room['cells']) == 32*28
assert len(room['portals']) == 1
p=room['portals'][0]
assert p['id'] == 'front_door_exit' and p['kind'] == 'exit' and p['cell'] == [32,45]
assert floor(*p['cell']) and p['destinationScene'] == 'world'
for needle in [
  'pub fn starter_house(', 'pub fn linked_cave(', 'pub fn load_template(',
  'pub fn exit_near(', 'pub fn is_walkable(', 'Duplicate or invalid portal identity',
  'fn starter_house_has_walkable_exit_and_wall_topology',
  'fn linked_cave_has_return_and_is_deterministic',
]: assert needle in room_source, needle
for needle in [
  'pub interior: Option<InteriorPlaySession>', 'pub fn nearby_entrance(',
  'pub fn interact(', 'pub fn enter_test_cave(', 'fn step_interior(',
  'fn exit_interior(', 'RoomScene::load_template(&path)',
  'fn interior_transition_keeps_world_position_and_blocks_walls',
  'fn interior_exit_is_rejected_until_player_reaches_portal',
]: assert needle in world_pie, needle
for needle in [
  'fn draw_world_pie_room(', 'GEOMETRY PREVIEW ONLY',
  'world_play.interact(&state.asset_authority)', 'world_play.enter_test_cave()',
  'input.key_pressed(egui::Key::F)', 'input.key_pressed(egui::Key::F8)',
  'draw_world_pie_room(ui, state, canvas)',
]: assert needle in main, needle
assert 'if world_play.interior.is_none()' in main
project=json.loads((root/'project/forgepy.project.json').read_text())
assert tuple(map(int, project['sourceVersion'].split('.'))) >= (0,10,37)
assert (root/'Cargo.toml').read_text().split('version = "')[1].split('"')[0] == project['sourceVersion']
print('M2D119 AUTHORING/PIE SOURCE CONTRACT: PASS / 4096 room tiles, 896 floor cells, derived walls, unique exit, F house + F8 diagnostic cave, collision, world return and native Rust tests staged')
print('M2D119 LIMIT: Cargo/Rust native compilation and runtime visual acceptance not available in this environment')
