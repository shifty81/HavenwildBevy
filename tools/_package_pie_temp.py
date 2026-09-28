from pathlib import Path
import hashlib,zipfile,json,shutil,subprocess,sys,importlib.util
root=Path(__file__).resolve().parents[1]
out=Path('/mnt/data')
patch=out/'Havenwild_Bevy_M2D02F1_Source_Scene_PIE_Proof.pccpatch.zip'
rollup=out/'Havenwild_Bevy_M2D02F1_COMPLETE_SOURCE_ONLY.zip'
paths=[
 'src/main.rs', 'src/playtest.rs', 'tools/rust_source_audit.py',
 'tools/pie_source_contract_selftest.py',
 'docs/patch/M2D02F1_PIE_SOURCE_SCENE.md',
 'docs/handoff/M2D02F1_SOURCE_CHECKPOINT.json',
]
checks=lambda f: hashlib.sha256(f.read_bytes()).hexdigest()
manifest={'schema':'havenwild.bevy.pcc.patch.v1',
 'id':'HW-BEVY-M2D02-F1-SOURCE-SCENE-PIE-20260928',
 'description':'Source-addressed Summer River Play-in-Editor proof: immutable live scene snapshot, debug player marker, preview-only v1 traversal, camera follow, reversible editor state, tests; zero guessed object art and zero scene/asset mutations. Requires installed M2D02-E.',
 'from':{'source':'0.5.6','forgepy':'0.4.6','pcc':'1.2.6'},
 'to':{'source':'0.5.6','forgepy':'0.4.6','pcc':'1.2.6'},
 'files':[{'path':name,'bytes':(root/name).stat().st_size,'sha256':checks(root/name)} for name in paths]}
with zipfile.ZipFile(patch,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
 z.writestr('pcc_patch.json',json.dumps(manifest,indent=2)+'\n')
 for name in paths:z.write(root/name,name)
# Invoke the one source-only packaging implementation and copy produced bytes to stable handoff name.
sys.path.insert(0,str(root))
import ForgePY
assert ForgePY.package_source()==0
source=sorted((root/'artifacts').glob('Havenwild_Bevy_Standalone_SOURCE_ONLY_*.zip'),key=lambda x:x.stat().st_mtime)[-1]
shutil.copy2(source,rollup)
# Verify identities, mutually consistent hashes and source exclusions.
with zipfile.ZipFile(rollup) as z:
 names=z.namelist()
 assert 'src/playtest.rs' in names and 'tools/pie_source_contract_selftest.py' in names
 assert not any(name.startswith('assets/elizawy/') or name.startswith('content/scenes/derived/') or name.startswith('reference/') for name in names)
 assert all(z.read(name)==(root/name).read_bytes() for name in paths)
with zipfile.ZipFile(patch) as z:
 assert set(z.namelist())==set(paths)|{'pcc_patch.json'}
 for entry in manifest['files']:
  assert len(z.read(entry['path']))==entry['bytes']
  assert hashlib.sha256(z.read(entry['path'])).hexdigest()==entry['sha256']
print('PATCH',patch.name,patch.stat().st_size,'bytes',len(paths),'files sha256',checks(patch))
print('ROLLUP',rollup.name,rollup.stat().st_size,'bytes',len(names),'entries sha256',checks(rollup))
print('EXCLUSIONS: hydrated ElizaWy assets, original reference art, local derived drafts absent')
