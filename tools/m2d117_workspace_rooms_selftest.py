#!/usr/bin/env python3
"""M2D117 source-contract checks; native Rust compilation and visual PIE acceptance remain required."""
from pathlib import Path
import json
root=Path(__file__).resolve().parents[1]
main=(root/'src/main.rs').read_text(encoding='utf8')
layout=(root/'src/editor_layout.rs').read_text(encoding='utf8')
room=(root/'src/room_scene.rs').read_text(encoding='utf8')
world=(root/'src/world_doc.rs').read_text(encoding='utf8')
icons=(root/'src/editor_icons.rs').read_text(encoding='utf8')
project=json.loads((root/'project/forgepy.project.json').read_text())
assert tuple(map(int, project['sourceVersion'].split('.'))) >= (0, 10, 35)
for item in ('library_open', 'library_floating', 'library_height', 'library_view', 'library_category', 'layers_dock_open'):
    assert item in layout, item
for item in ('fn draw_library_dock(', 'fn draw_floating_library(', 'egui::Panel::bottom("havenwild.library.bottom")',
             'fn canvas_layer_rect(', 'draw_layer_rail(ui, &mut state);',
             'draw_library_dock(&mut viewport_ui, &mut state);','draw_floating_library(ctx, &mut state);',
             'source_asset', 'fn draw_authoring_atlas', 'fn request_occupied_source_cells(',
             'std::thread::spawn(move ||', 'get_pixel(x * 32 + dx, y * 32 + dy)[3] != 0',
             'let raw_atlas =', 'ui.selectable_label(state.editor_layout.library_view == view, view)',
             'if list_mode', 'choose_brush(state, BrushMode::Tile)',
             'state.canvas_layers_rect = egui::Rect::NOTHING;'):
    assert item in main, item
layers=main.split('fn draw_layer_rail(',1)[1].split('fn docked_overlay_window(',1)[0]
assert 'Order::Foreground' not in layers and 'egui::Area' not in layers
assert 'canvas_layer_rect' in main
assert main.index('draw_layer_rail(ui, &mut state);') > main.index('egui::CentralPanel::default()')
assert 'RailIcon::Library' in main and 'RailIcon::Layers' in main
assert 'Library,' in icons and 'Layers,' in icons
assert 'fn library_and_canvas_layers_have_persistent_state()' in layout
assert 'mod room_scene;' in main
for token in ('pub fn paint_floor(', 'pub fn paint_floor_rect(', 'pub fn rebuild_walls(', 'pub fn add_portal(',
              'pub fn generate_cave(', 'pub fn validate(', 'pub const DEFAULT_ROOM_CHUNKS',
              'pub const MAX_ROOM_CHUNKS: u8 = 3;', 'enum PortalKind', 'HouseDoor', 'CaveEntrance',
              'fn caves_are_deterministic_connected_and_saveable()',
              'fn painted_floor_automatically_builds_and_updates_wall_neighbors()'):
    assert token in room, token
assert '"rocky_upland" => 80' in world
assert '"meadow" => 42' in world
assert '"light_woodland" => 28' in world
assert world.index('let stone_chance = match biome') < world.index('let tree_density = worldgen_plan::tree_density_percent')
assert '"natural stone"' in world
print('M2D117 SOURCE CONTRACT: PASS / canvas-integrated Layers controls, bottom/floating Library, alpha-backed compact list/grid, original Atlas, room/cave documents, expanded natural stone')
print('M2D117 SCOPE: room document generation stays source-only; newer PIE portal integration has its own regression')
