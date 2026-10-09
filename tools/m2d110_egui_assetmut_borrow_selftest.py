#!/usr/bin/env python3
"""M2D110 pre-Cargo guard for Bevy AssetMut<Image> / egui premultiplication.

A source-level check of the exact failure seen in the Windows Full Gate for
source v0.10.29. The actual compiler remains authoritative on Windows.
"""
from __future__ import annotations
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
main = (ROOT / 'src/main.rs').read_text(encoding='utf-8')
assert re.search(r'fn\s+premultiply_elizawy_sheet_texture\s*\(\s*image\s*:\s*&mut\s+Image',main)
assert re.search(r'let\s+Some\s*\(\s*mut\s+image\s*\)\s*=\s*images\.get_mut\s*\(\s*&sheet\.handle\s*\)',main), 'Bevy AssetMut must be a mutable binding'
assert re.search(r'premultiply_elizawy_sheet_texture\s*\(\s*&mut\s+image\s*\)',main), 'Must borrow AssetMut mutably for DerefMut<Image>'
assert not re.search(r'premultiply_elizawy_sheet_texture\s*\(\s*image\s*\)',main), 'Unborrowed AssetMut<Egui> compiler regression'
# Keep the source-only and pixel-data safety boundaries intact.
assert 'premultiply_elizawy_sheet_color(source)' in main
assert 'sheet.premultiplied_for_egui = true' in main
assert 'sheet.texture = if sheet.premultiplied_for_egui' in main
assert re.search(r'hydrate_pending_source\s*,\s*prepare_elizawy_egui_textures\s*,\s*draw_ui\s*,?',main)
print('M2D110 EGUI ASSETMUT BORROW: PASS / correct &mut Image deref coercion; immutable ElizaWy source path retained')
