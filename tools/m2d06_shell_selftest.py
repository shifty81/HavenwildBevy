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
    'fn draw_layer_panel(',
    'fn draw_app_launcher(',
    'egui::Panel::left("havenwild.tool.rail")',
    'egui::Panel::right("havenwild.layer.rail")',
    'egui::Panel::top("havenwild.command.toolbar")',
    'egui::Window::new("Scene / Layers")',
    'egui::Window::new("World Properties")',
    'egui::Window::new("ElizaWy · Source / Terrain / Evidence")',
    'egui::Window::new("Havenwild Tools")',
    'fn draw_world_generator_panel(',
    'fn draw_chunk_manager_panel(',
    'fn draw_asset_authority_panel(',
    'Pixel Studio · planned',
    'Animation Studio · planned',
]
missing = [token for token in required_main if token not in main]
assert not missing, missing

for forbidden in (
    'havenwild.terrain.mapper.left',
    'havenwild.terrain.mapper.right',
    'havenwild.terrain.mapper.bottom',
    'havenwild.structural.editor.right',
    'havenwild.layers.compact',
):
    assert forbidden not in main, f'feature panel still reserves canvas: {forbidden}'

for shortcut in ('"Q"', '"B"', '"E"', '"P"', '"Ctrl+S"', '"Ctrl+Z"', '"Ctrl+Y"', '"F6"', '"Ctrl+Space"', '"F11"'):
    assert shortcut in commands, f'missing command shortcut {shortcut}'
assert 'format!("{}\\n\\nShortcut: {}"' in commands

for token in (
    'pub terrain_mapper_compact: bool',
    'pub scene_layers_compact: bool',
    'pub structural_compact: bool',
    'pub scene_layers_dock: SurfaceDock',
    'pub structural_dock: SurfaceDock',
):
    assert token in layout, f'missing panel layout state: {token}'

print('M2D06 SHELL SELFTEST: PASS / permanent tool+layer rails, launcher, command tooltips, overlay dock panels, real world/chunk/asset-authority apps + truthful planned Pixel/Animation apps')
