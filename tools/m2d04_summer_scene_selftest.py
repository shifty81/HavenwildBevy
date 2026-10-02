#!/usr/bin/env python3
"""Deterministic source-only check for M2D04's genuinely populated visual scene.
Does not assert a native build, exact sprite alpha, visual approval or gameplay collision.
"""
from pathlib import Path
from hashlib import sha256
from collections import Counter
import json
ROOT=Path(__file__).resolve().parents[1]
seed_path=ROOT/'content/scenes/summer_world.visual_seed.v1.json'
source_path=ROOT/'content/scenes/elizawy_mapping_certification.scene.json'
manifest_path=ROOT/'content/catalog/core_source_manifest.json'
seed=json.loads(seed_path.read_bytes()); source_bytes=source_path.read_bytes(); base=json.loads(source_bytes)
manifest=json.loads(manifest_path.read_bytes())
entries={item['path']:item for item in manifest['entries'] if item['path'].endswith('.png')}
objects=seed['objects']; by_id={o['id']:o for o in objects}
assert seed['schema']=='havenwild.bevy.summer_visual_seed.v1'
assert seed['legacySourceSha256']==sha256(source_bytes).hexdigest()
assert seed['authority'].endswith('not_terrain_or_collision_certification')
assert seed['baseScene']==source_path.name
assert seed['generatorId']=='m2d04.summer.composition.visual-study.v1'
assert base['size']==[40,28] and len(base['tiles'])==1120
assert len(objects)>=35 and len(by_id)==len(objects), 'must have genuinely populated, unique instances'
assert all(o['id'].startswith('summer.') and o['origin']=={'kind':'generated','generator_id':seed['generatorId']} for o in objects)
assert all(len(o['parts'])==1 and o['parts'][0]['offset']==[0,0] for o in objects)
paths=Counter(); layers=Counter()
for object in objects:
    x,y=object['anchor']; w,h=object['footprint']; p=object['parts'][0]['source']; rx,ry,rw,rh=p['source_rect']
    entry=entries[p['source_asset']]; W,H=entry['imageSizePx']
    assert 0<=x and 0<=y and x+w<=40 and y+h<=28,(object['id'],'scene bounds')
    assert rx>=0 and ry>=0 and rw>0 and rh>0 and rx+rw<=W and ry+rh<=H,(object['id'],'source bounds')
    assert [w,h]==[(rw+31)//32,(rh+31)//32],object['id']
    assert not p['source_asset'].startswith(('archive/','derived/'))
    assert entry['stage']=='source_only',object['id']
    foot_x=x+(w-1)//2;foot_y=y+h-1
    foot_role=base['tiles'][foot_y*40+foot_x]['role']
    if '.reed.' in object['id'] or '.lily.' in object['id']:
        assert foot_role=='RiverWater',(object['id'],foot_role)
    elif object['layer'] in ('structures','terrain_details') or object['id'].startswith(('summer.broadleaf','summer.conifer','summer.fern','summer.shrub','summer.rock')):
        assert foot_role=='Grass',(object['id'],foot_role)
    paths[p['source_asset']]+=1;layers[object['layer']]+=1
assert len(paths)>=5 and {'objects','structures','terrain_details'}<=set(layers)
assert by_id['summer.house.east']['parts'][0]['source']['source_rect']==[0,0,192,192]
assert len([o for o in objects if o['id'].startswith(('summer.broadleaf','summer.conifer'))])>=9
assert len([o for o in objects if '.reed.' in o['id']])>=4
assert len([o for o in objects if '.lily.' in o['id']])>=2
assert not any('flattened' in o['parts'][0]['source']['source_asset'].lower() or 'demo' in o['parts'][0]['source']['source_asset'].lower() for o in objects)
assert all(o['parts'][0]['source']['source_asset'] not in ('Terrain/cliff_summer.png','Terrain/Waterfall.png') for o in objects), 'no fabricated structural source'
# Independently placed decorative objects must be selectable without hiding each other.
for i,a in enumerate(objects):
    ax,ay=a['anchor'];aw,ah=a['footprint']
    for b in objects[i+1:]:
        bx,by=b['anchor'];bw,bh=b['footprint']
        assert not (ax<bx+bw and bx<ax+aw and ay<by+bh and by<ay+ah),(a['id'],b['id'],'overlapping selection boxes')
main=(ROOT/'src/main.rs').read_text('utf-8');layout=(ROOT/'src/editor_layout.rs').read_text('utf-8');v2=(ROOT/'src/scene_v2.rs').read_text('utf-8');seed_rs=(ROOT/'src/summer_seed.rs').read_text('utf-8')
assert 'mod summer_seed;' in main and 'summer_seed::populate_new_scene(' in main
assert 'let layered_path = root.join("content/scenes/derived/summer_world.layered.draft.json")' in main
assert 'SceneV2::read_draft(&layered_path)' in main and 'state.scene_needs_initial_save = false;' in main
assert 'state.playtest_layered = Some(state.layered_scene.clone());' in main
assert main.count('"▶ Play Scene"')==1 and 'start_playtest(&mut state)' in main and 'stop_playtest(&mut state)' in main
assert 'properties_open: false' in main and 'egui::Window::new("Object properties · on demand")' in main
assert 'scene_panel_v3_initialized: true' in layout and 'canvas_first_v4_initialized: true' in layout
assert 'layers_open: false' in layout and 'terrain_mapper.visible = false' in layout
assert 'if !layout.canvas_first_v4_initialized' in layout
assert 'summer_world.layered.draft.json' in v2
assert 'doc.validate()?' in seed_rs and 'source_sha256' in seed_rs
# A hydrated live checkout is expected to have source art and private drafts.
# Enforce their exclusion in the authoritative SOURCE ZIP packager instead of
# incorrectly rejecting a real installed game just because those files exist.
package_source=(ROOT/'ForgePY.py').read_text('utf-8')
assert '"assets", "reference"' in package_source
assert 'rel.parts[:3] == ("content", "scenes", "derived")' in package_source
assert '"target", "artifacts"' in package_source and '".forgepy", ".pcc"' in package_source
print('M2D04 SOURCE SELFTEST: PASS /',len(objects),'independent original-addressed placements;',len(paths),'source PNGs;',dict(layers),'; SHA256 exact ground fixture; one PIE toggle; canvas-first migrated layout; prior drafts excluded')
print('NOTE: original PNG pixel hashes and Windows native/visual verification still require the hydrated user machine.')
