#!/usr/bin/env python3
"""Cargo-free wiring guards: these are NOT native Rust or visual certifications."""
from pathlib import Path
import json
root = Path(__file__).resolve().parents[1]
m = (root / 'src/main.rs').read_text(encoding='utf8')
w = (root / 'src/world_pie.rs').read_text(encoding='utf8')
p = (root / 'src/playtest.rs').read_text(encoding='utf8')
assert 'mod world_pie;' in m and 'world_playtest: Option<WorldPlaySession>' in m
assert 'WorldPlaySession::start(&state.world_doc, &state.asset_authority, state.world_selected)' in m
assert 'draw_pie_character(&painter' in m and 'if state.world_playtest.is_none()' in m
assert 'return; // No editor paint, object drag, deletion or other authoring while playing.' in m
assert 'state.world_playtest = Some(world_play);' in m
assert 'ctx.input(|input| input.key_pressed(egui::Key::Escape))' in m
assert 'state.world_playtest.take().is_some()' in m
assert '"World PIE · pending"' not in m
assert 'PLAYER_WALK_SHEET' in m and 'sheet.size != [512, 256]' in m
assert 'draw_pie_character(&painter, state, center, px, play.facing_row, play.elapsed, play.moving)' in m
assert 'pub facing_row: u32' in p and 'pub moving: bool' in p
assert 'pub fn effective_traversal(' in (root / 'src/world_doc.rs').read_text(encoding='utf8')
assert 'world.effective_collision_mask(cell, authority)' in w
assert 'world.generated_cell(cell).is_none()' in w
assert 'solid_bounds' in w and 'world.generated_objects()' in w
assert 'world.clone()' in w and 'self.snapshot' in w
assert 'fn player_facing_has_four_unique_states' in w
assert 'sourceVersion' in (root / 'project/forgepy.project.json').read_text(encoding='utf8')
assert tuple(map(int, json.loads((root / 'project/forgepy.project.json').read_text())['sourceVersion'].split('.'))) >= (0, 10, 34)
print('M2D116 SOURCE CONTRACT: PASS / world snapshot PIE + LPC optional character + collision + no editing + exit')
print('M2D116 SCOPE: original World PIE character/collision contract retained; newer portal and cave capabilities validated separately')
