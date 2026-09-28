//! DG-01 source-backed terrain recipe authoring state.
//! This module edits metadata only; it never mutates hydrated source artwork.

use crate::{atomic_file::write_atomic, source_catalog::SourceCatalog};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainRecipeFamily {
    pub schema: String,
    pub id: String,
    pub foreground: String,
    pub background: String,
    pub season: String,
    pub tile_size_px: u32,
    pub source_authority: String,
    pub states: BTreeMap<String, TerrainRecipeState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainRecipeState {
    pub resolution: String,
    /// Manual review state for this exact topology mapping. Source assignment starts
    /// as candidate and must be explicitly promoted before strict certification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<TerrainRecipeSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<TerrainRecipePart>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainRecipeSource {
    pub path: String,
    pub rect: [u32; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainRecipePart {
    pub source: TerrainRecipeSource,
    #[serde(default)]
    pub offset: [i32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummerSourceInventory {
    pub schema: String,
    pub source: SummerSourceDescriptor,
    pub recovery: SummerRecoverySummary,
    pub cells: Vec<SourceCellEvidence>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummerSourceDescriptor {
    pub path: String,
    pub sha256: String,
    pub dimensions: [u32; 2],
    pub tile_size_px: u32,
    pub grid: [u32; 2],
    pub total_cells: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummerRecoverySummary {
    pub historical_non_transparent_mapped: usize,
    pub historical_structural_transparent: usize,
    pub historical_unused_transparent: usize,
    pub current_pixel_replay_non_transparent: usize,
    pub current_pixel_replay_transparent: usize,
    pub structural_transparent_identities_recovered: bool,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCellEvidence {
    pub id: String,
    pub grid: [u32; 2],
    pub rect: [u32; 4],
    pub occupancy: String,
    pub alpha_pixels: usize,
    pub alpha_bbox: Option<[u32; 4]>,
    pub rgba_sha256: String,
    pub recovery_state: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalAuthority {
    pub schema: String,
    pub purpose: String,
    pub canonical_summer: serde_json::Value,
    pub historical_evidence: Vec<HistoricalEvidence>,
    pub confidence_states: Vec<String>,
    pub rules: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalEvidence {
    pub id: String,
    pub date: String,
    pub authority: String,
    pub facts: BTreeMap<String, serde_json::Value>,
    pub current_import_state: String,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TerrainMapper {
    pub path: PathBuf,
    pub family: TerrainRecipeFamily,
    pub selected_mask: u8,
    pub composite_offset: [i32; 2],
    pub dirty: bool,
    pub message: String,
    undo: Vec<TerrainRecipeFamily>,
    redo: Vec<TerrainRecipeFamily>,
    pub historical: HistoricalAuthority,
    pub summer_source: SummerSourceInventory,
}

impl TerrainMapper {
    pub fn load(
        path: impl AsRef<Path>,
        historical_path: impl AsRef<Path>,
        source_inventory_path: impl AsRef<Path>,
    ) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let family: TerrainRecipeFamily = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let historical: HistoricalAuthority =
            serde_json::from_str(&fs::read_to_string(historical_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let summer_source: SummerSourceInventory = serde_json::from_str(
            &fs::read_to_string(source_inventory_path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        for mask in 0u8..16 {
            let key = format!("{mask:04b}");
            if !family.states.contains_key(&key) {
                return Err(format!("Recipe family is missing topology state {key}"));
            }
        }
        Ok(Self {
            path,
            family,
            selected_mask: 0,
            composite_offset: [0, 0],
            dirty: false,
            message: "DG-01 mapper ready; prior Summer source-map evidence recovered separately from recipe certification.".into(),
            undo: Vec::new(),
            redo: Vec::new(),
            historical,
            summer_source,
        })
    }

    pub fn selected_key(&self) -> String {
        format!("{:04b}", self.selected_mask & 0x0f)
    }

    pub fn selected_state(&self) -> Option<&TerrainRecipeState> {
        self.family.states.get(&self.selected_key())
    }

    fn selected_state_mut(&mut self) -> Option<&mut TerrainRecipeState> {
        let key = self.selected_key();
        self.family.states.get_mut(&key)
    }

    fn record_edit(&mut self, before: TerrainRecipeFamily) {
        self.undo.push(before);
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo_edit(&mut self) {
        let Some(previous) = self.undo.pop() else {
            self.message = "Terrain recipe undo history is empty.".into();
            return;
        };
        let current = std::mem::replace(&mut self.family, previous);
        self.redo.push(current);
        if self.redo.len() > 64 {
            self.redo.remove(0);
        }
        self.dirty = true;
        self.message = "Undid terrain recipe edit.".into();
    }

    pub fn redo_edit(&mut self) {
        let Some(next) = self.redo.pop() else {
            self.message = "Terrain recipe redo history is empty.".into();
            return;
        };
        let current = std::mem::replace(&mut self.family, next);
        self.undo.push(current);
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.dirty = true;
        self.message = "Redid terrain recipe edit.".into();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn assign_sprite(&mut self, path: String, rect: [u32; 4]) {
        let key = self.selected_key();
        let before = self.family.clone();
        let changed = if let Some(state) = self.selected_state_mut() {
            state.resolution = "sprite".into();
            state.confidence = Some("candidate".into());
            state.source = Some(TerrainRecipeSource { path, rect });
            state.parts.clear();
            true
        } else {
            false
        };
        if changed {
            self.record_edit(before);
            self.dirty = true;
            self.message =
                format!("Assigned {key} as a single source sprite (draft). Save to persist.");
        }
    }

    pub fn add_composite_part(&mut self, path: String, rect: [u32; 4]) {
        let key = self.selected_key();
        let offset = self.composite_offset;
        let before = self.family.clone();
        let part_count = if let Some(state) = self.selected_state_mut() {
            if state.resolution != "composite" {
                state.resolution = "composite".into();
                state.source = None;
                state.parts.clear();
            }
            state.confidence = Some("candidate".into());
            state.parts.push(TerrainRecipePart {
                source: TerrainRecipeSource { path, rect },
                offset,
            });
            Some(state.parts.len())
        } else {
            None
        };
        if let Some(part_count) = part_count {
            self.record_edit(before);
            self.dirty = true;
            self.message = format!(
                "Added composite part {part_count} for {key} at offset {},{} (draft).",
                offset[0], offset[1]
            );
        }
    }

    pub fn remove_last_composite_part(&mut self) {
        let count = self
            .selected_state()
            .map(|state| state.parts.len())
            .unwrap_or_default();
        if count > 0 {
            self.remove_composite_part(count - 1);
        }
    }

    pub fn remove_composite_part(&mut self, index: usize) {
        let key = self.selected_key();
        let before = self.family.clone();
        let removed = if let Some(state) = self.selected_state_mut() {
            if state.resolution == "composite" && index < state.parts.len() {
                state.parts.remove(index);
                if state.parts.is_empty() {
                    state.resolution = "unmapped".into();
                    state.confidence = None;
                } else {
                    state.confidence = Some("candidate".into());
                }
                true
            } else {
                false
            }
        } else {
            false
        };
        if removed {
            self.record_edit(before);
            self.dirty = true;
            self.message = format!("Removed composite part {} from {key}.", index + 1);
        }
    }

    pub fn replace_composite_part(&mut self, index: usize, path: String, rect: [u32; 4]) {
        let key = self.selected_key();
        let before = self.family.clone();
        let changed = if let Some(state) = self.selected_state_mut() {
            if state.resolution == "composite" && index < state.parts.len() {
                state.parts[index].source = TerrainRecipeSource { path, rect };
                state.confidence = Some("candidate".into());
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            self.record_edit(before);
            self.dirty = true;
            self.message = format!("Replaced composite part {} for {key}.", index + 1);
        }
    }

    pub fn set_composite_part_offset(&mut self, index: usize, offset: [i32; 2]) {
        let key = self.selected_key();
        let before = self.family.clone();
        let changed = if let Some(state) = self.selected_state_mut() {
            if state.resolution == "composite"
                && index < state.parts.len()
                && state.parts[index].offset != offset
            {
                state.parts[index].offset = offset;
                state.confidence = Some("candidate".into());
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            self.record_edit(before);
            self.dirty = true;
            self.message = format!(
                "Updated composite part {} offset for {key} to {},{}.",
                index + 1,
                offset[0],
                offset[1]
            );
        }
    }

    pub fn mark_unsupported(&mut self) {
        let key = self.selected_key();
        let before = self.family.clone();
        let changed = if let Some(state) = self.selected_state_mut() {
            state.resolution = "unsupported".into();
            state.confidence = Some("candidate".into());
            state.source = None;
            state.parts.clear();
            true
        } else {
            false
        };
        if changed {
            self.record_edit(before);
            self.dirty = true;
            self.message =
                format!("Marked {key} unsupported by the current source family (draft).");
        }
    }

    pub fn clear_selected(&mut self) {
        let key = self.selected_key();
        let before = self.family.clone();
        let changed = if let Some(state) = self.selected_state_mut() {
            state.resolution = "unmapped".into();
            state.confidence = None;
            state.source = None;
            state.parts.clear();
            true
        } else {
            false
        };
        if changed {
            self.record_edit(before);
            self.dirty = true;
            self.message = format!("Cleared mapping for {key}.");
        }
    }

    pub fn set_selected_confidence(&mut self, confidence: &str) {
        if !matches!(confidence, "candidate" | "reviewed" | "certified") {
            self.message = format!("Rejected unknown recipe confidence state: {confidence}");
            return;
        }
        let key = self.selected_key();
        let before = self.family.clone();
        let changed = if let Some(state) = self.selected_state_mut() {
            if matches!(
                state.resolution.as_str(),
                "sprite" | "composite" | "unsupported"
            ) {
                state.confidence = Some(confidence.to_owned());
                true
            } else {
                false
            }
        } else {
            false
        };
        if changed {
            self.record_edit(before);
            self.dirty = true;
            self.message = format!("Marked {key} mapping confidence as {confidence}.");
        } else {
            self.message = format!("{key} must be classified before review/certification.");
        }
    }

    pub fn certified_count(&self) -> usize {
        self.family
            .states
            .values()
            .filter(|state| {
                matches!(
                    state.resolution.as_str(),
                    "sprite" | "composite" | "unsupported"
                ) && state.confidence.as_deref() == Some("certified")
            })
            .count()
    }

    pub fn next_unresolved(&mut self) -> Option<u8> {
        for step in 1..=16 {
            let candidate = (self.selected_mask + step) & 0x0f;
            let key = format!("{candidate:04b}");
            let unresolved = self
                .family
                .states
                .get(&key)
                .map(|state| state.resolution == "unmapped")
                .unwrap_or(true);
            if unresolved {
                self.selected_mask = candidate;
                return Some(candidate);
            }
        }
        None
    }

    pub fn counts(&self) -> (usize, usize, usize, usize, usize) {
        let mut classified = 0;
        let mut sprites = 0;
        let mut composites = 0;
        let mut unsupported = 0;
        let mut unmapped = 0;
        for state in self.family.states.values() {
            match state.resolution.as_str() {
                "sprite" => {
                    classified += 1;
                    sprites += 1;
                }
                "composite" => {
                    classified += 1;
                    composites += 1;
                }
                "unsupported" => {
                    classified += 1;
                    unsupported += 1;
                }
                _ => unmapped += 1,
            }
        }
        (classified, sprites, composites, unsupported, unmapped)
    }

    pub fn source_validation_errors(&self, catalog: &SourceCatalog) -> Vec<String> {
        let mut errors = Vec::new();
        for (key, state) in &self.family.states {
            let valid_confidence = matches!(
                state.confidence.as_deref(),
                None | Some("candidate") | Some("reviewed") | Some("certified")
            );
            if !valid_confidence {
                errors.push(format!(
                    "mask {key} has invalid confidence {:?}",
                    state.confidence
                ));
            }
            match state.resolution.as_str() {
                "sprite" => {
                    if !state.parts.is_empty() {
                        errors.push(format!(
                            "mask {key} sprite mapping must not contain composite parts"
                        ));
                    }
                    match &state.source {
                        Some(source) if catalog.contains_rect(&source.path, source.rect) => {}
                        Some(source) => errors.push(format!(
                            "mask {key} sprite source is missing or out of bounds: {} @ {:?}",
                            source.path, source.rect
                        )),
                        None => errors.push(format!("mask {key} sprite mapping has no source")),
                    }
                }
                "composite" => {
                    if state.source.is_some() {
                        errors.push(format!(
                            "mask {key} composite mapping must not also have a sprite source"
                        ));
                    }
                    if state.parts.is_empty() {
                        errors.push(format!("mask {key} composite mapping has no parts"));
                    }
                    for (index, part) in state.parts.iter().enumerate() {
                        if !catalog.contains_rect(&part.source.path, part.source.rect) {
                            errors.push(format!(
                                "mask {key} composite part {} is missing or out of bounds: {} @ {:?}",
                                index + 1,
                                part.source.path,
                                part.source.rect
                            ));
                        }
                    }
                }
                "unsupported" | "unmapped" => {
                    if state.source.is_some() || !state.parts.is_empty() {
                        errors.push(format!(
                            "mask {key} {} mapping must not retain source payloads",
                            state.resolution
                        ));
                    }
                    if state.resolution == "unmapped" && state.confidence.is_some() {
                        errors.push(format!(
                            "mask {key} unmapped state must not carry confidence"
                        ));
                    }
                }
                other => errors.push(format!("mask {key} has unknown resolution {other}")),
            }
        }
        errors
    }

    pub fn source_cell_evidence(
        &self,
        source_path: &str,
        grid: [u32; 2],
    ) -> Option<&SourceCellEvidence> {
        if source_path != self.summer_source.source.path {
            return None;
        }
        self.summer_source
            .cells
            .iter()
            .find(|entry| entry.grid == grid)
    }

    pub fn historical_count(&self, evidence_id: &str, key: &str) -> Option<u64> {
        self.historical
            .historical_evidence
            .iter()
            .find(|entry| entry.id == evidence_id)
            .and_then(|entry| entry.facts.get(key))
            .and_then(serde_json::Value::as_u64)
    }

    pub fn source_recovery_counts(&self) -> (usize, usize, usize) {
        (
            self.summer_source
                .recovery
                .current_pixel_replay_non_transparent,
            self.summer_source
                .recovery
                .historical_non_transparent_mapped,
            self.summer_source.recovery.current_pixel_replay_transparent,
        )
    }

    pub fn save(&mut self) -> Result<(), String> {
        let text = serde_json::to_string_pretty(&self.family).map_err(|e| e.to_string())? + "\n";
        write_atomic(&self.path, text.as_bytes())?;
        self.dirty = false;
        self.message = format!("Saved terrain recipe metadata to {}", self.path.display());
        Ok(())
    }
}

pub fn mask_meaning(mask: u8) -> &'static str {
    match mask & 0x0f {
        0x0 => "background / no foreground corners",
        0x1 => "NW only",
        0x2 => "NE only",
        0x3 => "north half",
        0x4 => "SW only",
        0x5 => "west half",
        0x6 => "split diagonal NE + SW",
        0x7 => "all except SE",
        0x8 => "SE only",
        0x9 => "split diagonal NW + SE",
        0xA => "east half",
        0xB => "all except SW",
        0xC => "south half",
        0xD => "all except NE",
        0xE => "all except NW",
        _ => "full foreground",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_catalog::{SourceCatalog, SourceCatalogEntry};

    fn mapper() -> TerrainMapper {
        let states = (0u8..16)
            .map(|mask| {
                (
                    format!("{mask:04b}"),
                    TerrainRecipeState {
                        resolution: "unmapped".into(),
                        confidence: None,
                        note: None,
                        source: None,
                        parts: Vec::new(),
                    },
                )
            })
            .collect();
        TerrainMapper {
            path: PathBuf::from("test-recipes.json"),
            family: TerrainRecipeFamily {
                schema: "havenwild.terrain.recipe_family.v1".into(),
                id: "test".into(),
                foreground: "Grass".into(),
                background: "Void".into(),
                season: "Summer".into(),
                tile_size_px: 32,
                source_authority: "test".into(),
                states,
            },
            selected_mask: 0b0110,
            composite_offset: [3, -2],
            dirty: false,
            message: String::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            historical: HistoricalAuthority {
                schema: "havenwild.terrain.historical_mapping_authority.v1".into(),
                purpose: "test".into(),
                canonical_summer: serde_json::json!({}),
                historical_evidence: Vec::new(),
                confidence_states: Vec::new(),
                rules: serde_json::json!({}),
            },
            summer_source: SummerSourceInventory {
                schema: "havenwild.terrain.summer_source_inventory.v1".into(),
                source: SummerSourceDescriptor {
                    path: "Terrain/terrain_summer.png".into(),
                    sha256: String::new(),
                    dimensions: [512, 832],
                    tile_size_px: 32,
                    grid: [16, 26],
                    total_cells: 416,
                },
                recovery: SummerRecoverySummary {
                    historical_non_transparent_mapped: 305,
                    historical_structural_transparent: 14,
                    historical_unused_transparent: 97,
                    current_pixel_replay_non_transparent: 305,
                    current_pixel_replay_transparent: 111,
                    structural_transparent_identities_recovered: false,
                    note: String::new(),
                },
                cells: Vec::new(),
            },
        }
    }

    #[test]
    fn composite_edit_releases_state_borrow_before_mapper_status_update() {
        let mut mapper = mapper();
        mapper.add_composite_part("Terrain/terrain_summer.png".into(), [32, 64, 32, 32]);
        let state = mapper.selected_state().expect("selected state");
        assert_eq!(state.resolution, "composite");
        assert_eq!(state.parts.len(), 1);
        assert_eq!(state.parts[0].offset, [3, -2]);
        assert!(mapper.dirty);
        assert!(mapper.message.contains("composite part 1"));
    }

    #[test]
    fn mapper_transitions_between_sprite_composite_unsupported_and_unmapped() {
        let mut mapper = mapper();
        mapper.assign_sprite("Terrain/terrain_summer.png".into(), [0, 0, 32, 32]);
        assert_eq!(mapper.selected_state().unwrap().resolution, "sprite");

        mapper.add_composite_part("Terrain/terrain_summer.png".into(), [32, 0, 32, 32]);
        assert_eq!(mapper.selected_state().unwrap().resolution, "composite");
        assert_eq!(mapper.selected_state().unwrap().parts.len(), 1);

        mapper.mark_unsupported();
        assert_eq!(mapper.selected_state().unwrap().resolution, "unsupported");
        assert!(mapper.selected_state().unwrap().parts.is_empty());

        mapper.clear_selected();
        assert_eq!(mapper.selected_state().unwrap().resolution, "unmapped");
    }

    #[test]
    fn recipe_edits_support_undo_and_redo() {
        let mut mapper = mapper();
        mapper.assign_sprite("Terrain/terrain_summer.png".into(), [0, 0, 32, 32]);
        assert!(mapper.can_undo());
        mapper.undo_edit();
        assert_eq!(mapper.selected_state().unwrap().resolution, "unmapped");
        assert!(mapper.can_redo());
        mapper.redo_edit();
        assert_eq!(mapper.selected_state().unwrap().resolution, "sprite");
    }

    #[test]
    fn editing_a_mapping_demotes_it_until_explicitly_certified() {
        let mut mapper = mapper();
        mapper.assign_sprite("Terrain/terrain_summer.png".into(), [0, 0, 32, 32]);
        assert_eq!(
            mapper.selected_state().unwrap().confidence.as_deref(),
            Some("candidate")
        );
        mapper.set_selected_confidence("certified");
        assert_eq!(mapper.certified_count(), 1);
        mapper.add_composite_part("Terrain/terrain_summer.png".into(), [32, 0, 32, 32]);
        assert_eq!(
            mapper.selected_state().unwrap().confidence.as_deref(),
            Some("candidate")
        );
        assert_eq!(mapper.certified_count(), 0);
    }

    #[test]
    fn composite_parts_can_be_replaced_repositioned_and_removed_by_index() {
        let mut mapper = mapper();
        mapper.add_composite_part("Terrain/a.png".into(), [0, 0, 32, 32]);
        mapper.add_composite_part("Terrain/b.png".into(), [32, 0, 32, 32]);
        mapper.replace_composite_part(0, "Terrain/c.png".into(), [64, 0, 32, 32]);
        mapper.set_composite_part_offset(1, [-4, 6]);
        let state = mapper.selected_state().unwrap();
        assert_eq!(state.parts[0].source.path, "Terrain/c.png");
        assert_eq!(state.parts[1].offset, [-4, 6]);

        mapper.remove_composite_part(0);
        let state = mapper.selected_state().unwrap();
        assert_eq!(state.parts.len(), 1);
        assert_eq!(state.parts[0].source.path, "Terrain/b.png");

        mapper.remove_composite_part(0);
        assert_eq!(mapper.selected_state().unwrap().resolution, "unmapped");
    }
    #[test]
    fn recipe_source_validation_uses_canonical_manifest_bounds() {
        let mut mapper = mapper();
        let catalog = SourceCatalog {
            entries: vec![SourceCatalogEntry {
                path: "Terrain/terrain_summer.png".into(),
                size: [512, 832],
            }],
        };
        mapper.assign_sprite("Terrain/terrain_summer.png".into(), [480, 800, 32, 32]);
        assert!(mapper.source_validation_errors(&catalog).is_empty());
        mapper.assign_sprite("Terrain/terrain_summer.png".into(), [500, 800, 32, 32]);
        let errors = mapper.source_validation_errors(&catalog);
        assert!(errors.iter().any(|error| error.contains("out of bounds")));
    }
}
