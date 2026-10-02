#!/usr/bin/env python3
"""Source-only preflight for M2D02-B. Native Rust tests execute during PCC Full Gate."""
from pathlib import Path
import hashlib, json
ROOT = Path(__file__).resolve().parents[1]
main = (ROOT/'src/main.rs').read_text(encoding='utf-8')
v1_src = (ROOT/'src/document.rs').read_text(encoding='utf-8')
v2_src = (ROOT/'src/scene_v2.rs').read_text(encoding='utf-8')
atomic = (ROOT/'src/atomic_file.rs').read_text(encoding='utf-8')
scene_path = ROOT/'content/scenes/summer_river.scene.json'
original = scene_path.read_bytes()
scene = json.loads(original)
assert scene['schema'] == 'havenwild.elizawy.scene.v1'
assert scene['size'] == [40,28] and len(scene['tiles']) == 1120
assert all('role' in cell and 'source_asset' in cell and 'source_rect' in cell for cell in scene['tiles'])
assert 'mod scene_v2;' in main and 'SceneV2::preview_import(&legacy_base_path)' in main
assert '.save_editable_draft(' in main
assert 'Preview separate v2 scene draft' not in main and 'Create separate v2 draft' not in main
assert 'migration_preview: Option<SceneV2>' not in main
assert 'verify_legacy_source' in v2_src and 'pub fn save_new_draft(' in v2_src
assert 'let source_scene_path' in main and 'summer_world.layered.draft.json' in main and 'summer_seed::populate_new_scene' in main
assert 'read_draft(&layered_path)' in main and 'verify_legacy_source(&legacy_base_path)' in main
assert 'write_atomic_new(draft_path, &bytes)' in v2_src
assert 'fs::hard_link(&staged, path)' in atomic and '.create_new(true)' in atomic
assert 'pub fn validate(&self)' in v1_src and 'pub fn validate(&self)' in v2_src
for snippet in ['legacy_base: source.tiles', 'structural_cells: Vec::new()', 'pub structural_cells: Vec<StructuralCell>',
                'LayerId::ORDERED', 'pub removed_generated_objects: Vec<RemovalTombstone>',
                'pub protected_regions: Vec<ProtectedRegion>', 'pub enum SceneCommand',
                'pub struct SceneHistory', 'fn whole_stroke_is_one_sparse_transaction_and_semantics_survive_undo',
                'fn canonical_v1_fixture_import_roundtrip_preserves_all_tiles_and_roles',
                'fn generated_removal_creates_restorable_tombstone_manual_removal_does_not',
                'fn sha256_vectors_and_source_fingerprint', 'fn failed_input_and_protected_destination_never_replace_v1_or_draft']:
    assert snippet in v2_src, snippet
draft_path = ROOT/'content/scenes/summer_river.scene.v2.draft.json'
if draft_path.exists():
    draft = json.loads(draft_path.read_bytes())
    assert draft['schema'] == 'havenwild.elizawy.scene.v2.draft'
    assert draft['size'] == [40, 28] and len(draft['legacy_base']) == 1120
    assert len(draft['visual_layers']) == 7
    assert draft['legacy_origin']['source_bytes'] > 0
    if hashlib.sha256(original).hexdigest() != draft['legacy_origin']['source_sha256']:
        print('NOTE: saved v2 draft has a different v1 origin revision; its existing contents are retained, not silently rebased.')
assert scene_path.read_bytes() == original, 'read-only source fixture contract'
print('M2D02-B SELFTEST: PASS / v1 fixture 1120 intact; v2 source migration internal and one active live scene; sparse layer/object ops; structural metadata not invented; atomic create-new only; SHA256 fixture =', hashlib.sha256(original).hexdigest()[:12])
print('NOTE: Source-only test; native cargo test/build and UI validation are separate.')
