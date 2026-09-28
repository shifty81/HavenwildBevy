#!/usr/bin/env python3
"""M2D02-B2 static integration checks. Native Rust tests still require Windows Cargo."""
from pathlib import Path
import re
ROOT = Path(__file__).resolve().parents[1]
main=(ROOT/'src/main.rs').read_text(encoding='utf8')
layout=(ROOT/'src/editor_layout.rs').read_text(encoding='utf8')
rulers=(ROOT/'src/canvas_rulers.rs').read_text(encoding='utf8')
assert 'mod canvas_rulers;' in main
assert 'state.scene_rect = scene;' in main
assert 'canvas_rulers::draw(&painter, canvas, scene, px' in main
assert 'canvas_rulers::blocks_scene_edit(state.canvas_rect, pointer,' in main
assert 'canvas.min + ruler_inset' in main
assert 'canvas.top() + ruler_inset.y' in main
assert 'fn major_tile_step(' in rulers
assert 'fn blocks_scene_edit(' in rulers
assert 'tile_screen_px >= 12.0' in rulers
assert 'rulers_visible: true' in layout
assert 'default_rulers_visible' in layout
assert '.pccpatch.zip' not in rulers
assert not any(token in rulers for token in ('fs::write(', 'write_atomic(', 'state.scene.tiles'))
assert 'SceneV2::preview_import(source)' in main
print('M2D02-B2 RULERS: PASS / canvas-only overlay, adaptive tiles, input guard, v2 migration intact')
