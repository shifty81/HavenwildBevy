//! One visually authored Summer composition over immutable original River terrain.
//! Only exact source-addressed PNG regions are placed. This is NOT DG certification,
//! procedural worldgen, collision authority, verified structure assembly, or finished
//! waterfalls/bridge connections. Local editable Summer drafts own all later changes.
use crate::scene_v2::{ObjectOrigin, PlacedObject, SceneV2};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};

const SCHEMA: &str = "havenwild.bevy.summer_visual_seed.v1";
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Seed {
    schema: String,
    legacy_source_sha256: String,
    generator_id: String,
    objects: Vec<PlacedObject>,
}

/// Invoked only when creating the new Summer working document; never reseed an
/// existing local Summer draft. Earlier River drafts are never opened for writing.
pub fn populate_new_scene(doc: &mut SceneV2, path: &Path) -> Result<usize, String> {
    let raw = fs::read(path).map_err(|e| format!("Cannot read Summer source composition: {e}"))?;
    let seed: Seed = serde_json::from_slice(&raw)
        .map_err(|e| format!("Cannot parse Summer source composition: {e}"))?;
    if seed.schema != SCHEMA
        || seed.generator_id.is_empty()
        || seed.legacy_source_sha256 != doc.legacy_origin.source_sha256
        || !doc.objects.is_empty()
        || seed.objects.len() < 30
    {
        return Err("Summer composition mismatches the original River source fingerprint or current document; nothing substituted".into());
    }
    let mut ids = BTreeSet::new();
    for object in &seed.objects {
        if !object.id.starts_with("summer.")
            || !ids.insert(object.id.as_str())
            || !matches!(&object.origin, ObjectOrigin::Generated {generator_id} if generator_id == &seed.generator_id)
        {
            return Err(
                "Summer composition has duplicate/untrusted object identity or provenance".into(),
            );
        }
    }
    let count = seed.objects.len();
    doc.objects = seed.objects;
    doc.name = "Summer World — original-source composition study".into();
    doc.validate()?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn summer_sample_imports_from_exact_original_without_modifying_it() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source = root.join("content/scenes/elizawy_mapping_certification.scene.json");
        let bytes = fs::read(&source).unwrap();
        let mut scene = SceneV2::preview_import(&source).unwrap();
        let added = populate_new_scene(
            &mut scene,
            &root.join("content/scenes/summer_world.visual_seed.v1.json"),
        )
        .unwrap();
        assert!(added >= 30);
        assert_eq!(scene.objects.len(), added);
        assert!(scene
            .objects
            .iter()
            .any(|o| o.label.contains("brick house")));
        assert!(scene
            .objects
            .iter()
            .any(|o| o.label.contains("Summer tree")));
        assert!(scene.objects.iter().any(|o| o.label.contains("Water-edge")));
        assert!(
            scene.structural_cells.is_empty(),
            "visual composition cannot invent structural authority"
        );
        assert_eq!(
            fs::read(&source).unwrap(),
            bytes,
            "original terrain must remain unmodified"
        );
    }
}
