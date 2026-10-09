#!/usr/bin/env python3
"""Static contract for the canvas-first Havenwild Studio shell. Not a native GUI test."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
commands = (ROOT / 'src/editor_commands.rs').read_text(encoding='utf-8')
layout = (ROOT / 'src/editor_layout.rs').read_text(encoding='utf-8')

required_main = [
    'fn draw_tool_rail(',
    'fn draw_layer_rail(',
    'fn draw_inspector_rail(',
    'fn draw_app_launcher(',
    'egui::Panel::left("havenwild.tool.rail")',
    'egui::Panel::right("havenwild.inspector.rail")',
    'egui::Panel::top("havenwild.command.toolbar")',
    'egui::Window::new("World Properties")',
    'egui::Window::new("Terrain Studio · ElizaWy Authority")',
    'egui::Window::new("Havenwild Tools")',
    'fn draw_world_generator_panel(',
    'Materialized chunks · debug',
    'Pixel Studio · planned',
    'Animation Studio · planned',
    'RailIcon::Eye',
    'world_layer_visibility[slot]',
]
missing = [token for token in required_main if token not in main]
assert not missing, missing

for forbidden in (
    'havenwild.terrain.mapper.left',
    'havenwild.terrain.mapper.right',
    'havenwild.terrain.mapper.bottom',
    'havenwild.structural.editor.right',
    'havenwild.layers.compact',
    'fn draw_layer_panel(',
    'egui::Window::new("Scene / Layers")',
    'fn draw_chunk_manager_panel(',
    'fn draw_asset_authority_panel(',
    'ElizaWy Asset Authority',
    'Source / mapping information',
):
    assert forbidden not in main, f'duplicate/retired editor surface remains: {forbidden}'

for shortcut in ('"Q"', '"B"', '"E"', '"P"', '"Ctrl+S"', '"Ctrl+Z"', '"Ctrl+Y"', '"F6"', '"Ctrl+Space"', '"F11"'):
    assert shortcut in commands, f'missing command shortcut {shortcut}'
assert 'format!("{}\\n\\nShortcut: {}"' in commands

for token in (
    'pub terrain_mapper_compact: bool',
    'pub structural_compact: bool',
    'pub structural_dock: SurfaceDock',
    'pub world_generator_open: bool',
    'pub world_generator_dock: SurfaceDock',
    'pub world_layer_visibility: [bool; 7]',
):
    assert token in layout, f'missing canonical panel/layout state: {token}'
for retired in ('pub layers_open: bool', 'pub scene_layers_compact: bool', 'pub asset_authority_open: bool', 'pub chunk_manager_open: bool'):
    assert retired not in layout, retired

print('M2D06 SHELL SELFTEST: PASS / permanent direct-tool + authoritative layer rails, one launcher, Terrain Studio + World Properties + Generated World as canonical apps, duplicate legacy panels retired')
