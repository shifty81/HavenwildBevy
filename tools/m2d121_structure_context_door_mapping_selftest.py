#!/usr/bin/env python3
"""Source contract checks for per-structure interior and doorway interactions.
These do NOT replace native Cargo/PIE visual verification."""
from pathlib import Path
import json
import re

def rust_compact(text: str) -> str:
    return re.sub(r",(?=\))", "", re.sub(r"\s+", "", text))

r=Path(__file__).resolve().parents[1]
main=(r/'src/main.rs').read_text()
world=(r/'src/world_doc.rs').read_text()
pie=(r/'src/world_pie.rs').read_text()
room=(r/'src/room_scene.rs').read_text()
assert tuple(map(int,json.loads((r/'project/forgepy.project.json').read_text())['sourceVersion'].split('.'))) >= (0,10,39)
for s in ('ui.button("Edit Interior")', 'ui.button("Map Doorway → Scene")',
          'open_house_room_editor_for(state, &object_id)', 'world_door_mapping: Option<String>',
          'world_door_mapping.take()', 'map_structure_door(&object_id, world, destination, &state.asset_authority)',
          'house_instance_relative_path(state.world_doc.seed, object_id)',
          'state.room_authoring_object = Some(object_id.to_owned())',
          'state.world_doc.structure_doors.values()', 'state.world_dirty = true;',
          'room_scene::RoomScene::load_template(source_path)'):
 assert rust_compact(s) in rust_compact(main),s
assert 'ui.button("Edit House Room")' not in main
for s in ('pub struct StructureDoorLink', 'pub structure_doors: BTreeMap<String, StructureDoorLink>',
    '#[serde(default)]', 'pub fn doorway_borders_footprint(', 'pub fn map_structure_door(',
    'self.effective_traversal(world_cell, authority) != TraversalMode::Walkable',
    'self.structure_doors.insert(object_id.to_owned(), link)',
    'Choose a cardinally adjacent cell OUTSIDE the house footprint'):
 assert s in world,s
for s in ('self.snapshot.structure_doors.get(&obj.id)', 'if mapped.is_some() && !path.is_file()',
    'let mapped_spawn', 'active.position = [cell[0] as f32 + 0.5, cell[1] as f32 + 0.5]'):
 assert s in pie,s
assert 'pub fn house_instance_relative_path' in room
print('M2D121 STRUCTURE CONTEXT / DOORWAY: PASS / house-specific rooms, right-click authoring, mapped exterior cell, project-owned scene, dynamic room spawn, cancel, world persistence')
print('M2D121 NATIVE CARGO/GUI: pending Windows Full Gate')
