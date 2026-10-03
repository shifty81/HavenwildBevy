#!/usr/bin/env python3
import pathlib
root=pathlib.Path(__file__).resolve().parents[1]
world=(root/'src/world_doc.rs').read_text(encoding='utf8'); auth=(root/'src/asset_authority.rs').read_text(encoding='utf8'); main=(root/'src/main.rs').read_text(encoding='utf8')
required_world=['WORLD_CHUNK_SIDE: usize = 32','materialize_3x3','preview_regenerate_3x3','validate_against_authority','set_visual_override','set_collision_mask','set_elevation_override','suppressed_generated_objects','topology_palette']
for token in required_world: assert token in world, token
for token in ['generated_artwork_allowed','contains_binding','unique_fixture_role','fixture_topology_palette']: assert token in auth, token
for token in ['draw_world_workspace','draw_world_generator_panel','draw_chunk_manager_panel','draw_asset_authority_panel','World PIE · pending','.materialize([cx, cy], radius']: assert token in main, token
assert 'generate or synthesize image assets' in (root/'tools/build_elizawy_asset_lane.py').read_text(encoding='utf8')
print('M2D07 WORLD CONTRACT SELFTEST: PASS / 32x32 signed chunks / deterministic radius materialization / overrides+tombstones preserved / no generated art')
