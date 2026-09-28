#!/usr/bin/env python3
"""M2D02-E: reject the invented atlas-crop scene; preserve source/legacy drafts."""
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
fixture_path=ROOT/'content/scenes/summer_river.scene.json'
fixture_bytes=fixture_path.read_bytes()
scene_path=ROOT/'content/scenes/elizawy_mapping_certification.scene.json'
index_path=ROOT/'content/scenes/elizawy_mapping_certification.index.json'
old_archive=ROOT/'docs/handoff/retired/M2D02D_guessed_assembled_scene.archive.json'
old_index=ROOT/'docs/handoff/retired/M2D02D_guessed_assembled_index.archive.json'
original=ROOT/'content/catalog/core_source_manifest.json'
original_hash=sha(original)
run=subprocess.run([sys.executable,str(ROOT/'tools/build_mapping_certification_scene.py'),'--check'],cwd=ROOT,capture_output=True,text=True)
assert run.returncode==0,(run.stdout,run.stderr)
scene=json.loads(scene_path.read_bytes());fixture=json.loads(fixture_bytes);index=json.loads(index_path.read_bytes())
assert scene['tiles']==fixture['tiles'] and scene['size']==fixture['size']==[40,28]
assert scene['hidden_visual_samples']==[] and len(scene['tiles'])==1120
assert index['visualSamples']==[] and len(index['retiredGuessedVisualSampleIds'])==18
assert index['certifiedRecipeMasks']==0 and index['summerRecoveredSourceCells']==305
assert index['authority']=='pinned_authored_fixture_no_guessed_compositions_zero_promotions'
assert index['fixtureZone']['sourceFixtureSha256']==sha(fixture_path)
assert json.loads(old_archive.read_bytes())['size']==[72,28]
assert len(json.loads(old_index.read_bytes())['visualSamples'])==18
assert index['retiredLocalDraftPreserved'].endswith('.assembled.draft.json')
assert index['activeDerivedDraft'].endswith('.source_exact.draft.json')
assert fixture_path.read_bytes()==fixture_bytes and sha(original)==original_hash
main=(ROOT/'src/main.rs').read_text('utf-8')
assert '.source_exact.draft.json' in main and '.assembled.draft.json' not in main
assert 'visual_samples.len() != 0' in main
assert 'fn snapped_scene_edge(' in main
assert 'draw_assembled_objects(&painter, state, scene, px, canvas);' in main
assert 'pinned source-authored River' in main
print('M2D02-E SOURCE EXACT: PASS / 1120 authored cells / zero guessed objects / old 72x28 evidence archived / separate user draft / snapped shared tile edges')
