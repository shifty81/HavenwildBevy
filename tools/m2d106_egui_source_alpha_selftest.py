#!/usr/bin/env python3
"""Source-backed guard for egui's premultiplied-alpha ElizaWy texture path.

A fully transparent RGB triplet is not necessarily black in the original PNG;
that RGB must not be submitted straight to a premultiplied-alpha renderer.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAIN = (ROOT / 'src/main.rs').read_text(encoding='utf8')


def verify():
    assert 'fn premultiply_elizawy_sheet_color' in MAIN
    assert 'fn premultiply_elizawy_sheet_texture' in MAIN
    assert 'fn prepare_elizawy_egui_textures' in MAIN
    assert '(hydrate_pending_source, prepare_elizawy_egui_textures, draw_ui).chain()' in MAIN
    assert 'sheet.premultiplied_for_egui' in MAIN
    assert 'sheet.texture = if sheet.premultiplied_for_egui' in MAIN
    assert 'Color::LinearRgba(linear)' in MAIN
    from inspect_elizawy_source_alpha import rgba_rows
    source = ROOT / 'assets/elizawy/Terrain/trees_summer.png'
    meta = json.loads((ROOT / 'content/catalog/core_source_manifest.json').read_text())
    expected = next(entry for entry in meta['entries'] if entry['path'] == 'Terrain/trees_summer.png')
    payload = source.read_bytes()
    assert hashlib.sha256(payload).hexdigest() == expected['sha256']
    _, _, rows = rgba_rows(payload)
    # The exact transparent pixels visible as green boxes/white lines in Studio.
    assert rows[16][130] == (0, 48, 35, 0), rows[16][130]
    assert rows[16][129] == (255, 255, 255, 0), rows[16][129]
    print('M2D106 PREMULTIPLIED-ALPHA CONTRACT: PASS / source transparent RGB proven, original PNG hash pinned, UI preparation ordered before rendering')


if __name__ == '__main__':
    verify()
