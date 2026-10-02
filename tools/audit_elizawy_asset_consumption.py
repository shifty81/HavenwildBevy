#!/usr/bin/env python3
"""Audit discoverability versus runtime/scene consumption of ORIGINAL ElizaWy assets.

No source artwork is changed, guessed, copied, or classified as certified. Safe
without hydrated workstation artwork; --verify-local checks the actual originals.
"""
from __future__ import annotations
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys
import re

ROOT = Path(__file__).resolve().parents[1]

def read(relative: str):
    return json.loads((ROOT / relative).read_text(encoding='utf-8'))

def collect():
    manifest = read('content/catalog/core_source_manifest.json')
    inventory = read('content/catalog/mapping_inventory.v1.json')
    active = read('content/catalog/active_sheets.json')
    scene = read('content/scenes/elizawy_mapping_certification.scene.json')
    original = read('content/scenes/summer_river.scene.json')
    errors = []
    entries = manifest['entries']
    paths = [e['path'] for e in entries]
    by_path = {e['path']: e for e in entries}
    if manifest.get('schema') != 'havenwild.bevy.asset_manifest.v1' or manifest['count'] != len(paths) or len(by_path) != len(paths):
        errors.append('canonical manifest schema/count/unique paths changed')
    if set(paths) != {a['sourcePath'] for a in inventory['assets']} or len(inventory['assets']) != len(entries):
        errors.append('source mapping inventory must retain one record per canonical core asset')
    for asset in inventory['assets']:
        entry = by_path.get(asset['sourcePath'])
        if entry is None or asset['sourceSha256'] != entry['sha256'] or asset['sourceBytes'] != entry['bytes']:
            errors.append('inventory provenance differs from canonical source: ' + asset['sourcePath'])
    ext = Counter(path.rsplit('.', 1)[-1].lower() for path in paths)
    families = {}
    for group in ('Terrain', 'Structure', 'Objects', 'FX'):
        family = [v for v in entries if v['path'].startswith(group + '/')]
        families[group] = {'png': sum(v['path'].lower().endswith('.png') for v in family),
                           'non_png': sum(not v['path'].lower().endswith('.png') for v in family)}
    pngs = [e for e in entries if e['path'].lower().endswith('.png')]
    for asset in pngs:
        d = asset.get('imageSizePx')
        if not d or len(d) != 2 or min(d) <= 0:
            errors.append('PNG has no valid original dimensions: ' + asset['path'])
    for asset in entries:
        if asset['path'].startswith('/') or '\\' in asset['path'] or ':' in asset['path'] or any(part in ('', '.', '..') for part in asset['path'].split('/')):
            errors.append('unsafe source path: ' + asset['path'])
    if ext != Counter({'png': 320, 'txt': 28, 'gif': 1}) or families != {
        'Terrain': {'png': 29, 'non_png': 1}, 'Structure': {'png': 98, 'non_png': 15},
        'Objects': {'png': 188, 'non_png': 12}, 'FX': {'png': 5, 'non_png': 1}}:
        errors.append('source-family/count expectations changed: review before claiming catalog completeness')
    selected = active['sheets']
    for record in selected:
        canonical = by_path.get(record['path'])
        if canonical is None or canonical['sha256'] != record['sha256'] or canonical['imageSizePx'] != record['sizePx']:
            errors.append('active sheet is not an exact original: ' + record['path'])
    if scene['tiles'] != original['tiles'] or scene['size'] != original['size'] or scene['tile_size'] != original['tile_size']:
        errors.append('original v1 River fixture pixel references and certification baseline diverge')
    used = set()
    for tile in scene['tiles']:
        path = tile['source_asset']
        if not path: continue
        if path not in by_path: errors.append('scene references unknown source: ' + path);continue
        x, y, w, h = tile['source_rect']; size=by_path[path]['imageSizePx']
        if min(w, h) <= 0 or x < 0 or y < 0 or x + w > size[0] or y + h > size[1]:
            errors.append('scene source crop out of canonical sheet: '+path)
        used.add(path)
    # The pinned original v1 ground fixture is intentionally distinct from the
    # new editable Summer visual composition. Report both, never pretend the
    # legacy fixture's 2 source sheets are the entire active world anymore.
    seed = read('content/scenes/summer_world.visual_seed.v1.json')
    seed_uses = set()
    seed_instances = seed.get('objects', [])
    original_bytes = (ROOT/'content/scenes/elizawy_mapping_certification.scene.json').read_bytes()
    if seed.get('schema') != 'havenwild.bevy.summer_visual_seed.v1' or seed.get('legacySourceSha256') != hashlib.sha256(original_bytes).hexdigest():
        errors.append('active Summer composition does not match pinned source fixture SHA256')
    if len(seed_instances) != 36 or len({o.get('id') for o in seed_instances}) != len(seed_instances):
        errors.append('active Summer composition object count/IDs changed without authored review')
    for object in seed_instances:
        for part in object.get('parts', []):
            src = part['source']; path = src['source_asset']
            if path not in by_path:
                errors.append('Summer composition references unknown original source: ' + path)
                continue
            size = by_path[path]['imageSizePx']; x,y,w,h = src['source_rect']
            if min(w,h)<=0 or min(x,y)<0 or x+w>size[0] or y+h>size[1]:
                errors.append('Summer composition source crop out of canonical sheet: '+path)
            seed_uses.add(path)
    main=(ROOT/'src/main.rs').read_text(encoding='utf-8')
    catalog=(ROOT/'src/source_catalog.rs').read_text(encoding='utf-8')
    needed=['state.asset_authority.runtime_source_images.iter()', 'source_family_filter', 'pending_source_sheet',
            'fn load_source_sheet_on_demand(']
    # Compile-safe textual contract, not a substitute for Cargo or interactive visual checks.
    for needle in needed:
        if re.sub(r'\s+', '', needle) not in re.sub(r'\s+', '', main): errors.append('complete lazy source browser not connected: '+needle)
    for needle in ('manifest.entries', 'entries.sort_by(', 'path.ends_with(".png")'):
        if needle not in catalog: errors.append('canonical source catalog guard missing: '+needle)
    if len(used) != 2 or used != {'Terrain/terrain_summer.png', 'Terrain/cliff_summer.png'}:
        errors.append('baseline scene uses unexpected sources; review actual scene coverage')
    optional = manifest.get('optionalArchives', {})
    if set(optional) != {'Characters.zip', 'FourSeasonAlternative.zip'} or optional['FourSeasonAlternative.zip']['policy'] != 'optional_never_auto_merge':
        errors.append('optional source boundaries changed')
    return entries, errors, {'core_records':len(entries),'image_sources_catalogued':len(pngs),'non_image_records':len(entries)-len(pngs),
        'families':families,'legacy_scene_source_files_used': sorted(used), 'legacy_scene_instanced_png_count':len(used),
        'active_summer_seed_source_paths':sorted(seed_uses), 'active_summer_seed_instances':len(seed_instances),
        'active_summer_source_paths_including_original_ground':sorted(seed_uses | used),
        'original_collection_images_not_in_legacy_scene':len(pngs)-len(used),
        'canonical_terrain_initially_loaded':29, 'other_pngs_lazy_not_preloaded':len(pngs)-29,
        'source_mapping_certified_by_this_audit':0,'source_composition_certified_by_this_audit':0,
        'characters_core_manifest_member':False,'characters_optional_hydration':optional['Characters.zip']['policy'],
        'alternative_archive_policy':optional['FourSeasonAlternative.zip']['policy'],
        'browser_scope':'all 320 canonical original PNGs; inspected on demand; no automatic semantic certification'}

def verify_local(entries):
    failures=[];checked=0
    root=ROOT/'assets/elizawy'
    for item in entries:
        path=root/item['path']
        if not path.is_file(): failures.append('MISSING '+item['path']);continue
        data=path.read_bytes();checked+=1
        if len(data)!=item['bytes'] or hashlib.sha256(data).hexdigest()!=item['sha256']:
            failures.append('HASH '+item['path'])
    return checked,failures

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    parser.add_argument('--verify-local',action='store_true',help='also validate hydrated exact original file hashes')
    parser.add_argument('--json',type=Path,help='optional report path (never write source art)')
    args=parser.parse_args()
    entries, errors, summary=collect()
    if args.verify_local:
        checked,failures=verify_local(entries)
        summary['local_files_sha256_verified']=checked-len([v for v in failures if v.startswith('HASH ')])
        errors+=failures
    for family, counts in summary['families'].items(): print(f"ORIGINAL {family:10} PNG={counts['png']:3} non-image={counts['non_png']:2}")
    print(f"CATALOG: {summary['image_sources_catalogued']}/320 original PNGs; {summary['non_image_records']} non-PNG records; {summary['other_pngs_lazy_not_preloaded']} lazy images")
    print('ORIGINAL GROUND: 2/320 source PNG paths in unchanged pinned v1 River fixture')
    print(f"ACTIVE SUMMER VISUAL STUDY: {summary['active_summer_seed_instances']} source-bound editable object instances from {len(summary['active_summer_seed_source_paths'])} additional original PNG paths; {len(summary['active_summer_source_paths_including_original_ground'])}/320 distinct PNG paths including original ground. Visual candidates, NOT approved object/terrain recipes.")
    print('STATUS: provenance/discoverability audit only; no asset roles, recipes, object boundaries, navigation or alpha certified')
    for error in errors: print('[FAIL]',error)
    if args.json:
        args.json.parent.mkdir(parents=True,exist_ok=True)
        args.json.write_text(json.dumps({'schema':'havenwild.elizawy.consumption_audit.v1','summary':summary,'errors':errors},indent=2)+'\n',encoding='utf-8')
    if errors: return 1
    print('ELIZAWY CORE CONSUMPTION AUDIT: PASS / manifest + original fixture + new Summer object source identities; artistic and gameplay certification still open')
    return 0
if __name__=='__main__':sys.exit(main())
