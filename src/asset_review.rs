//! Read-only original-alpha evidence. A candidate is NEVER an approved region.
//! It cannot be used as an object template, collision rule, or semantic mapping.
use crate::asset_authority::AssetAuthority;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaGroup {
    pub rect_px: [u32; 4],
    pub occupied_cells: u32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlphaSheet {
    pub source_path: String,
    pub source_sha256: String,
    pub image_size_px: [u32; 2],
    pub occupied_cells: u32,
    pub isolated_groups: Vec<AlphaGroup>,
    pub crowded_groups: u32,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlphaInventory {
    schema: String,
    sheets: Vec<AlphaSheet>,
}
#[derive(Clone, Debug, Default)]
pub struct AlphaCandidateIndex {
    sheets: BTreeMap<String, AlphaSheet>,
}
impl AlphaCandidateIndex {
    pub fn load(path: &Path, authority: &AssetAuthority) -> Result<Self, String> {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let decoded: AlphaInventory = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        if decoded.schema != "havenwild.bevy.original_alpha_candidates.v1" {
            return Err("Unexpected original-alpha evidence schema".into());
        }
        let expected: BTreeMap<&str, (&str, [u32; 2])> = authority
            .runtime_source_images
            .iter()
            .map(|row| {
                (
                    row.source_path.as_str(),
                    (row.source_sha256.as_str(), row.image_size_px),
                )
            })
            .collect();
        let mut sheets = BTreeMap::new();
        for sheet in decoded.sheets {
            let Some(&(hash, size)) = expected.get(sheet.source_path.as_str()) else {
                return Err(format!(
                    "Unrecognized candidate source: {}",
                    sheet.source_path
                ));
            };
            if sheet.source_sha256 != hash || sheet.image_size_px != size || size.contains(&0) {
                return Err(format!(
                    "Original-alpha identity mismatch: {}",
                    sheet.source_path
                ));
            }
            let mut seen = BTreeSet::new();
            for group in &sheet.isolated_groups {
                let [x, y, w, h] = group.rect_px;
                if w == 0
                    || h == 0
                    || group.occupied_cells == 0
                    || [x, y, w, h].iter().any(|v| v % 32 != 0)
                    || x.checked_add(w).is_none_or(|v| v > size[0])
                    || y.checked_add(h).is_none_or(|v| v > size[1])
                    || !seen.insert(group.rect_px)
                {
                    return Err(format!(
                        "Invalid candidate rectangle: {} {:?}",
                        sheet.source_path, group.rect_px
                    ));
                }
            }
            if sheets.insert(sheet.source_path.clone(), sheet).is_some() {
                return Err("Duplicate original-alpha source entry".into());
            }
        }
        if sheets.len() != expected.len() {
            return Err(format!(
                "Original-alpha census incomplete: {} of {}",
                sheets.len(),
                expected.len()
            ));
        }
        Ok(Self { sheets })
    }
    pub fn sheet(&self, path: &str) -> Option<&AlphaSheet> {
        self.sheets.get(path)
    }
}
