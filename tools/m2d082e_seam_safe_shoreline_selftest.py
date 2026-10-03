#!/usr/bin/env python3
"""Compatibility guard for the superseded 0.8.8 seam-safe experiment."""
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
s = (ROOT / 'src/world_doc.rs').read_text(encoding='utf-8')
assert 'fn seam_safe_summer_full_tile' not in s
assert 'fn land_connectivity_projection' in s
assert 'Historical quadrant composites remain evidence/tooling records only' in s
assert 'summer_flatworld_visual_parts(role_refs)' not in s
print('M2D082E COMPATIBILITY SELFTEST: PASS / rejected 0.8.8 projection retired; M2D082F land-connectivity resolver active')
