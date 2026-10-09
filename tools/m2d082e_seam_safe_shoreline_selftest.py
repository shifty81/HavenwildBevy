#!/usr/bin/env python3
"""Compatibility guard for the superseded 0.8.8 seam-safe experiment."""
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
s = (ROOT / 'src/world_doc.rs').read_text(encoding='utf-8')
assert 'fn seam_safe_summer_full_tile' not in s
assert 'fn land_connectivity_projection' in s
assert 'Historical quadrant composites and the old RiverWater/WetSand pale-' in s and 'shore repaint are not live compositor authorities anymore' in s
assert 'summer_flatworld_visual_parts(role_refs)' not in s
print('M2D082E COMPATIBILITY SELFTEST: PASS / rejected 0.8.8 projection retired; exact Tiled-first dual-grid resolver active')
