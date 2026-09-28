#!/usr/bin/env python3
"""Source-exact River certification baseline. Never guess object/terrain composites.

The M2D02-D random rock/tree samples and synthetic meadow have been retired. Only
reproduce the SHA-pinned original authored River fixture. The original LPC demo
images are REFERENCE ONLY, not a replacement for editable, source-addressed art.
"""
import hashlib, json, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FIXTURE=ROOT/'content/scenes/summer_river.scene.json'
SCENE=ROOT/'content/scenes/elizawy_mapping_certification.scene.json'
INDEX=ROOT/'content/scenes/elizawy_mapping_certification.index.json'
CAT=ROOT/'content/catalog/core_source_manifest.json'
SUMMER=ROOT/'content/terrain/recovery/summer_source_cells.v1.json'
PINNED_FIXTURE_SHA256='9e5ade01d388c6184984c61c8bf97cd3dafcb436a4c9c060b8db9e3c842b09e7'
RETIRED=['woods.tree.00', 'woods.tree.01', 'woods.tree.02', 'woods.tree.03', 'woods.tree.04', 'woods.tree.05', 'woods.tree.06', 'woods.tree.07', 'woods.tree.08', 'woods.tree.09', 'woods.tree.10', 'woods.tree.11', 'scree.boulder.00', 'scree.boulder.01', 'scree.boulder.02', 'scree.boulder.03', 'scree.boulder.04', 'scree.boulder.05']

def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def build():
    if sha(FIXTURE)!=PINNED_FIXTURE_SHA256:
        raise ValueError('Original River fixture differs from SHA-pinned authority; refusing synthetic reconstruction')
    fixture=json.loads(FIXTURE.read_text('utf-8'))
    assert fixture['schema']=='havenwild.elizawy.scene.v1' and fixture['size']==[40,28]
    assert fixture['tile_size']==32 and len(fixture['tiles'])==1120
    catalog=json.loads(CAT.read_text('utf-8'))
    catalog_art={entry['path']:entry for entry in catalog['entries'] if entry['path'].startswith('Terrain/') and entry['path'].endswith('.png')}
    assert len(catalog_art)==29
    summer=json.loads(SUMMER.read_text('utf-8'))
    assert sum(c['occupancy']=='nontransparent' for c in summer['cells'])==305
    for tile in fixture['tiles']:
        if not tile['source_asset']: continue
        entry=catalog_art[tile['source_asset']]
        x,y,w,h=tile['source_rect']; W,H=entry['imageSizePx']
        assert w>0 and h>0 and x>=0 and y>=0 and x+w<=W and y+h<=H
    scene=dict(fixture)
    scene['name']='ElizaWy Mapping Review — original Summer River fixture'
    scene['hidden_visual_samples']=[]
    used=sorted({tuple(t['source_rect'][:2]) for t in fixture['tiles'] if
       t['source_asset']=='Terrain/terrain_summer.png' and t['source_rect'][2:]==[32,32]})
    index={
       'schema':'havenwild.elizawy.assembled_certification_scene.v1',
       'authority':'pinned_authored_fixture_no_guessed_compositions_zero_promotions',
       'generatedSceneSize':fixture['size'],
       'catalogSha256':sha(CAT),'summerInventorySha256':sha(SUMMER),
       'totalCoreAssets':catalog['count'],'terrainSourceSheetsAvailableInBrowser':29,
       'summerRecoveredSourceCells':305,'summerTransparentSourceCells':111,
       'summerSourceCellsActuallyUsed':len(used),'certifiedRecipeMasks':0,
       'fixtureZone':{'origin':[0,0],'sizeCells':fixture['size'],
            'sourceFixture':'content/scenes/summer_river.scene.json','sourceFixtureSha256':sha(FIXTURE)},
       'testZone':{'origin':[0,0],'sizeCells':fixture['size']},
       'visualSamples':[],
       'retiredGuessedVisualSampleIds':RETIRED,
       'retiredGeneratedDocuments':[
         'docs/handoff/retired/M2D02D_guessed_assembled_scene.archive.json',
         'docs/handoff/retired/M2D02D_guessed_assembled_index.archive.json'],
       'retiredLocalDraftPreserved':'content/scenes/derived/elizawy_mapping_certification.assembled.draft.json',
       'activeDerivedDraft':'content/scenes/derived/elizawy_mapping_certification.source_exact.draft.json',
       'originalSourcesUnchanged':True,'sampleFixtureUnchanged':True,
       'instruction':'Source-addressed authoring fixture only. No guessed rock/tree crops, extra grass fill, or fake certified scenery. Originals and existing M2D02-D derived drafts are not modified. Exact source demo images are comparison evidence only.'}
    return scene,index

def main(argv):
    scene,index=build()
    expected={SCENE:json.dumps(scene,ensure_ascii=False,separators=(',',':'))+'\n',
              INDEX:json.dumps(index,indent=2,ensure_ascii=False)+'\n'}
    if argv==['--check']:
        for path,data in expected.items():
            if not path.is_file() or path.read_text('utf-8')!=data:
                print('SOURCE EXACT LAB FAIL:',path.relative_to(ROOT));return 1
        print(f'SOURCE EXACT LAB PASS: {scene["size"]} / {len(scene["tiles"])} fixture tiles / {len(index["visualSamples"])} guessed objects / {index["summerSourceCellsActuallyUsed"]} source addresses used / no promotions')
        return 0
    if argv: print('Use --check or no arguments'); return 2
    for path,data in expected.items():
        path.parent.mkdir(parents=True,exist_ok=True);path.write_text(data,encoding='utf-8')
    print('SOURCE EXACT LAB: restored pinned authoring fixture; guessed scene archived; local derived drafts untouched')
    return 0
if __name__=='__main__': raise SystemExit(main(sys.argv[1:]))
