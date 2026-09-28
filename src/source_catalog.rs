//! Read-only catalog of hydrated ElizaWy source images.
//! The manifest is source-controlled metadata; the image bytes remain local-only.

use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoreSourceManifest {
    schema: String,
    count: usize,
    entries: Vec<CoreSourceEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoreSourceEntry {
    path: String,
    image_size_px: Option<[u32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCatalogEntry {
    pub path: String,
    pub size: [u32; 2],
}

#[derive(Clone, Debug, Default)]
pub struct SourceCatalog {
    pub entries: Vec<SourceCatalogEntry>,
}

impl SourceCatalog {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let manifest: CoreSourceManifest =
            serde_json::from_str(&text).map_err(|error| error.to_string())?;
        if manifest.schema != "havenwild.bevy.asset_manifest.v1"
            || manifest.count != manifest.entries.len()
        {
            return Err("Canonical ElizaWy manifest identity/count mismatch".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut entries = Vec::new();
        for entry in manifest.entries {
            let path = entry.path.as_str();
            if !seen.insert(entry.path.clone())
                || path.is_empty()
                || path.starts_with('/')
                || path.contains('\\')
                || path.contains(':')
                || path
                    .split('/')
                    .any(|part| part.is_empty() || part == "." || part == "..")
            {
                return Err(format!("Unsafe or duplicated canonical asset path: {path}"));
            }
            if path.ends_with(".png") {
                let size = entry
                    .image_size_px
                    .ok_or_else(|| format!("PNG dimensions missing: {path}"))?;
                if size.contains(&0) {
                    return Err(format!("Invalid PNG dimensions: {path}"));
                }
                entries.push(SourceCatalogEntry {
                    path: entry.path,
                    size,
                });
            } else if entry.image_size_px.is_some() {
                return Err(format!("Non-PNG record contains PNG dimensions: {path}"));
            }
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(Self { entries })
    }

    pub fn entry(&self, path: &str) -> Option<&SourceCatalogEntry> {
        self.entries.iter().find(|entry| entry.path == path)
    }

    pub fn contains_rect(&self, path: &str, rect: [u32; 4]) -> bool {
        let Some(entry) = self.entry(path) else {
            return false;
        };
        let [x, y, width, height] = rect;
        width > 0
            && height > 0
            && x.checked_add(width)
                .zip(y.checked_add(height))
                .map(|(right, bottom)| right <= entry.size[0] && bottom <= entry.size[1])
                .unwrap_or(false)
    }

    pub fn terrain_entries(&self) -> impl Iterator<Item = &SourceCatalogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.path.starts_with("Terrain/"))
    }

    pub fn image_count(&self) -> usize {
        self.entries.len()
    }

    pub fn terrain_count(&self) -> usize {
        self.terrain_entries().count()
    }
    pub fn family_count(&self, family: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.path.split('/').next() == Some(family))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_original_png_family_coverage_is_complete_without_loading_image_bytes() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("content/catalog/core_source_manifest.json");
        let catalog = SourceCatalog::load(&path).expect("valid exact original source manifest");
        assert_eq!(catalog.image_count(), 320);
        assert_eq!(catalog.terrain_count(), 29);
        assert_eq!(catalog.family_count("Structure"), 98);
        assert_eq!(catalog.family_count("Objects"), 188);
        assert_eq!(catalog.family_count("FX"), 5);
    }

    #[test]
    fn terrain_filter_is_path_scoped() {
        let catalog = SourceCatalog {
            entries: vec![
                SourceCatalogEntry {
                    path: "Objects/chest.png".into(),
                    size: [32, 32],
                },
                SourceCatalogEntry {
                    path: "Terrain/terrain_summer.png".into(),
                    size: [512, 832],
                },
            ],
        };
        assert_eq!(catalog.image_count(), 2);
        assert_eq!(catalog.terrain_count(), 1);
        assert!(catalog.contains_rect("Terrain/terrain_summer.png", [480, 800, 32, 32]));
        assert!(!catalog.contains_rect("Terrain/terrain_summer.png", [500, 800, 32, 32]));
        assert!(!catalog.contains_rect("Terrain/missing.png", [0, 0, 32, 32]));
    }
}
