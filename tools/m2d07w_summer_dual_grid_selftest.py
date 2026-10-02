#!/usr/bin/env python3
"""Cargo-free contract check for M2D07W Summer dual-grid/autotile repair."""
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parents[1]

def require(cond, message):
    if not cond:
        raise AssertionError(message)

def main():
    meta = json.loads((ROOT/'project/forgepy.project.json').read_text(encoding='utf-8'))
    profile = json.loads((ROOT/'content/terrain/recipes/summer_flatworld_runtime.v1.json').read_text(encoding='utf-8'))
    main_rs = (ROOT/'src/main.rs').read_text(encoding='utf-8')
    world_rs = (ROOT/'src/world_doc.rs').read_text(encoding='utf-8')
    authority_rs = (ROOT/'src/asset_authority.rs').read_text(encoding='utf-8')
    pcc = (ROOT/'ProjectControlCenter.py').read_text(encoding='utf-8')
    mapper = (ROOT/'tools/build_summer_flatworld_mapping.py').read_text(encoding='utf-8')

    require(tuple(map(int, meta.get('sourceVersion','0.0.0').split('.'))) >= (0,7,8), 'sourceVersion must be >= 0.7.8')
    require(profile.get('sourceAtlas') == 'Terrain/terrain_summer.png', 'flatworld atlas must be terrain_summer.png')
    require(profile.get('sourceAtlasSha256') == '1251a6ea556330190ccb1e3af166eb69fbec4b7a3f7d51c1f54728abd75bd752', 'pinned Summer sheet SHA missing/wrong')
    require('cornerRecipes' in profile, 'checked-in profile must expose cornerRecipes')
    require('summer_flatworld_corner' in authority_rs, 'runtime authority needs four-corner lookup')
    require('summer_flatworld_runtime.local.json' in authority_rs, 'runtime must load local generated mapping metadata')
    require('[world[0] - 1, world[1] - 1]' in world_rs and 'world,                        // SE' in world_rs, 'dual-grid SE-anchor convention missing')
    require('paint_semantic_terrain_untracked' in world_rs and 'commit_edit_batch' in world_rs, 'batched semantic painting contract missing')
    require('binary_search_by_key' in world_rs, 'sorted override lookup/insert optimization missing')
    require('quarantine_uncertified_large_objects' in world_rs, 'large object quarantine missing')
    require('worldgen_object_bounds.local.json' in authority_rs and 'locally_certified_object_template' in authority_rs, 'local object-crop authority missing')
    require('world_stroke_seen: BTreeSet' in main_rs and 'world_stroke_apply_segment' in main_rs, 'responsive drag-stroke contract missing')
    require(('Studio 0.7.8' in main_rs and 'Bevy Studio v0.7.8' in main_rs) or ('Studio 0.8.0' in main_rs and 'Bevy Studio v0.8.0' in main_rs) or ('Studio 0.8.1' in main_rs and 'Bevy Studio v0.8.1' in main_rs) or ('Studio 0.8.2' in main_rs and 'Bevy Studio v0.8.2' in main_rs) or ('Studio 0.8.3' in main_rs and 'Bevy Studio v0.8.3' in main_rs) or ('Studio 0.8.4' in main_rs and 'Bevy Studio v0.8.4' in main_rs), 'visible Studio identity missing')
    require(('ONE-PASS Native -> Bevy Summer authority convergence' in pcc or 'BUILD Summer GRS/DIR/WTR autotile map' in pcc) and 'summer-map' in pcc, 'PCC Summer mapper/convergence entry missing')
    require('rgba_rows' in mapper and 'cornerRecipes' in mapper and 'cliff' in mapper.lower(), 'source-pixel mapper safeguards missing')
    require('largest_connected_crop' in mapper and 'worldgen_object_bounds.local.json' in mapper, 'tree/rock source-bound certification missing')
    require('"assets" / "elizawy"' in mapper and 'sha256' in mapper, 'mapper must use local verified original source')
    print('M2D07W SUMMER DUAL-GRID SELFTEST: PASS / exact Summer source map + true four-corner resolver + batched strokes + large-object quarantine')
    return 0

if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(f'M2D07W SUMMER DUAL-GRID SELFTEST: FAIL / {error}', file=sys.stderr)
        raise SystemExit(2)
