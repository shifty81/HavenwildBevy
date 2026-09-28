//! Studio-owned draft scene. No Havenwild runtime, PCC, donor paths, or approval flags.
use crate::atomic_file::write_atomic;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tile {
    /// Legacy semantic hint retained for backward compatibility with the v1 acceptance scene.
    /// Source stamping never mutates this value; semantic terrain authoring lives in its own
    /// atlas-independent document/resolver path.
    #[serde(rename = "role", default = "void_role")]
    pub semantic_role_hint: String,
    pub source_asset: String,
    /// Exact pixel rectangle on the untouched source sheet: x, y, width, height.
    pub source_rect: [u32; 4],
    #[serde(default = "draft")]
    pub status: String,
}

fn draft() -> String {
    "draft".into()
}

fn void_role() -> String {
    "Void".into()
}

impl Tile {
    pub fn empty() -> Self {
        Self {
            semantic_role_hint: "Void".into(),
            source_asset: String::new(),
            source_rect: [0, 0, 0, 0],
            status: "draft".into(),
        }
    }

    pub fn clear_source_binding(&mut self) {
        // Source-address editing must not erase the legacy semantic hint. The hint is
        // non-authoritative, but retaining it preserves recovery/provenance evidence.
        self.source_asset.clear();
        self.source_rect = [0, 0, 0, 0];
        self.status = "draft".into();
    }

    pub fn is_empty(&self) -> bool {
        self.source_asset.is_empty() || self.source_rect[2] == 0 || self.source_rect[3] == 0
    }
}

#[cfg(test)]
mod tile_tests {
    use super::*;

    #[test]
    fn clearing_source_binding_preserves_legacy_semantic_hint() {
        let mut tile = Tile {
            semantic_role_hint: "RiverWater".into(),
            source_asset: "Terrain/terrain_summer.png".into(),
            source_rect: [64, 96, 32, 32],
            status: "draft".into(),
        };
        tile.clear_source_binding();
        assert_eq!(tile.semantic_role_hint, "RiverWater");
        assert!(tile.is_empty());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub schema: String,
    pub name: String,
    pub size: [usize; 2],
    pub tile_size: u32,
    pub tiles: Vec<Tile>,
    /// Hidden built-in visual integration samples; these IDs are instance removals,
    /// not changes to source artwork or certified generator rules.
    #[serde(default)]
    pub hidden_visual_samples: Vec<String>,
}

impl Scene {
    pub fn load(path: &Path) -> Result<Self, String> {
        let scene: Scene = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        scene.validate()?;
        Ok(scene)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "havenwild.elizawy.scene.v1"
            || self.tile_size != 32
            || self.size[0] == 0
            || self.size[1] == 0
            || self.size[0].checked_mul(self.size[1]) != Some(self.tiles.len())
        {
            return Err("Unsupported or incomplete scene document; original file untouched".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        if self.hidden_visual_samples.iter().any(|id| {
            id.is_empty()
                || id.len() > 96
                || !id.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-' || byte == b'_'
                })
                || !ids.insert(id.as_str())
        }) {
            return Err("Invalid or duplicate visual-sample removal ID".into());
        }
        Ok(())
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        // Save to a draft document only; no modification of images or original fixtures.
        write_atomic(path, &bytes)
    }
    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.size[0] && y < self.size[1] {
            Some(y * self.size[0] + x)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod assembled_scene_tests {
    use super::*;
    #[test]
    fn derived_sample_removals_are_bounded_distinct_and_preserve_original_tiles() {
        let original = Tile {
            semantic_role_hint: "Grass".into(),
            source_asset: "Terrain/terrain_summer.png".into(),
            source_rect: [96, 32, 32, 32],
            status: "draft".into(),
        };
        let mut scene = Scene {
            schema: "havenwild.elizawy.scene.v1".into(),
            name: "sample".into(),
            size: [1, 1],
            tile_size: 32,
            tiles: vec![original.clone()],
            hidden_visual_samples: Vec::new(),
        };
        scene.validate().unwrap();
        scene.hidden_visual_samples.push("woods.tree.00".into());
        scene.validate().unwrap();
        let restored: Scene = serde_json::from_slice(&serde_json::to_vec(&scene).unwrap()).unwrap();
        assert_eq!(restored.hidden_visual_samples, vec!["woods.tree.00"]);
        assert_eq!(restored.tiles[0], original);
        scene.hidden_visual_samples.push("woods.tree.00".into());
        assert!(scene.validate().is_err());
        scene.hidden_visual_samples[1] = "../bad".into();
        assert!(scene.validate().is_err());
    }
}
