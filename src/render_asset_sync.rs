//! Render-resource dependencies for a materialized Havenwild world.
//! This is NOT semantic certification: an original PNG address can be known
//! and renderable without being an approved terrain/prop gameplay recipe.
use crate::world_doc::WorldDocument;
use std::collections::{BTreeSet, HashMap};

/// The renderer's *source dependency* set, not a new authoritative asset list.
/// Include generated terrain, manual visual overrides, authored bridges, and
/// every visible object's exact atlas source, even after world regeneration.
pub fn required_world_source_paths(world: &WorldDocument) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for chunk in &world.chunks {
        for cell in &chunk.generated_cells {
            paths.insert(cell.source.source_asset.clone());
        }
    }
    for object in world.generated_objects() {
        for part in &object.parts {
            paths.insert(part.source.source_asset.clone());
        }
    }
    for item in &world.cell_overrides {
        if let Some(source) = &item.source {
            paths.insert(source.source_asset.clone());
        }
    }
    paths
}

/// Missing texture registrations are requested lazily, in reproducible order.
/// An asset is not considered render-ready merely because it exists in JSON.
pub fn missing_world_source_paths(
    world: &WorldDocument,
    registered: &HashMap<String, usize>,
) -> Vec<String> {
    required_world_source_paths(world)
        .into_iter()
        .filter(|path| !registered.contains_key(path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene_v2::SourceBinding;
    use crate::world_doc::{WorldCellOverride, WorldObject, WorldObjectPart};

    fn binding(path: &str) -> SourceBinding {
        SourceBinding {
            source_asset: path.into(),
            source_rect: [0, 0, 32, 32],
        }
    }

    #[test]
    fn materialized_world_asset_dependencies_cover_objects_overrides_and_deduplicate() {
        let mut world = WorldDocument::new(2026);
        world.authored_objects.push(WorldObject {
            id: "test:object".into(),
            template_id: "test:template".into(),
            label: "Source object".into(),
            layer: "objects".into(),
            anchor_world: [0, 0],
            footprint_cells: [1, 1],
            generator_id: "test".into(),
            parts: vec![WorldObjectPart {
                offset_px: [0, 0],
                source: binding("Objects/test.png"),
            }],
        });
        world.cell_overrides.push(WorldCellOverride {
            world: [1, 0],
            visual_mode: None,
            source: Some(binding("Terrain/test.png")),
            semantic: None,
            elevation: None,
            collision: None,
            traversal: None,
        });
        world.cell_overrides.push(WorldCellOverride {
            world: [2, 0],
            visual_mode: None,
            source: Some(binding("Objects/test.png")),
            semantic: None,
            elevation: None,
            collision: None,
            traversal: None,
        });
        let required = required_world_source_paths(&world);
        assert_eq!(required.len(), 2);
        assert!(required.contains("Objects/test.png"));
        assert!(required.contains("Terrain/test.png"));
        let registered = HashMap::from([("Terrain/test.png".into(), 1usize)]);
        assert_eq!(
            missing_world_source_paths(&world, &registered),
            vec!["Objects/test.png"]
        );
    }
}
