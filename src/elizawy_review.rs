//! Read-only, native M2D browser for the workstation-local M2C4 evidence registry.
//! Pixel matches and external Tiled metadata are candidates, never runtime authority.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::{mpsc, Mutex},
    thread,
};

const SCHEMA: &str = "havenwild.elizawy.tiled_review_registry.v1";
const PINNED_ARCHIVE_SHA256: &str =
    "f0236c16b272a7328dad66772e4f971513bc508cb69a56c60d736264a3a5b43a";
const VERIFIED_PIXELS: &str = "original_sha_verified_rgba32_exact_match";
const SEASONS: [&str; 4] = ["summer", "spring", "autumn", "winter"];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewRegistry {
    pub schema: String,
    pub authority: String,
    pub archive_sha256: String,
    pub historical_map_state: String,
    pub certified_mappings_added: usize,
    pub runtime_changed: bool,
    pub source_files_mutated: bool,
    pub seasons: BTreeMap<String, ReviewSeason>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSeason {
    pub original_pixel_state: String,
    pub summary: ReviewSummary,
    pub cells: Vec<ReviewCell>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSummary {
    pub occupied_derivative_cells: usize,
    pub unique_candidate_cells: usize,
    pub ambiguous_candidate_cells: usize,
    pub unmatched_derivative_cells: usize,
    pub candidate_mappings_certified: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCell {
    pub derivative_tile_id: u32,
    pub derivative_cell: [u32; 2],
    pub match_status: String,
    pub original_candidates: Vec<ReviewCandidate>,
    #[serde(default)]
    pub tiled_wang_evidence: BTreeMap<String, serde_json::Value>,
    pub tiled_tile_type: Option<String>,
    pub tiled_has_collision_definition: bool,
    #[serde(default)]
    pub tiled_animation_frames: Vec<serde_json::Value>,
    pub review_state: String,
    pub mapping_certified: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCandidate {
    pub source_cell: [u32; 2],
    pub source_sheet: String,
    pub historical_summer_group_id: Option<String>,
    pub historical_summer_role: Option<String>,
    pub historical_summer_role_is_not_season_certification: bool,
}

impl ReviewRegistry {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA
            || self.authority != "evidence_only_zero_promotions"
            || self.archive_sha256 != PINNED_ARCHIVE_SHA256
            || self.historical_map_state != "sha256_verified"
            || self.certified_mappings_added != 0
            || self.runtime_changed
            || self.source_files_mutated
        {
            return Err("Registry identity, provenance or zero-promotion policy is invalid".into());
        }
        if self.seasons.len() != 4 || SEASONS.iter().any(|s| !self.seasons.contains_key(*s)) {
            return Err("Expected the four pinned seasonal evidence profiles".into());
        }
        for season in SEASONS {
            let profile = &self.seasons[season];
            if profile.original_pixel_state != VERIFIED_PIXELS
                || profile.summary.candidate_mappings_certified != 0
            {
                return Err(format!(
                    "{season}: original pixel proof or certification state invalid"
                ));
            }
            let mut ids = BTreeSet::new();
            let mut counts = [0usize; 3];
            for cell in &profile.cells {
                let id = cell.derivative_tile_id;
                if id >= 4096
                    || !ids.insert(id)
                    || cell.derivative_cell != [id % 64, id / 64]
                    || cell.mapping_certified
                    || cell.review_state != "unreviewed"
                {
                    return Err(format!(
                        "{season}: invalid/duplicate row or unexpected promotion at {id}"
                    ));
                }
                let n = cell.original_candidates.len();
                let index = match cell.match_status.as_str() {
                    "pixel_unique_candidate" if n == 1 => 0,
                    "pixel_ambiguous_candidates" if n > 1 => 1,
                    "no_original_terrain_cell_match" if n == 0 => 2,
                    _ => return Err(format!("{season}: inconsistent match status at {id}")),
                };
                counts[index] += 1;
                for candidate in &cell.original_candidates {
                    if candidate.source_cell[0] >= 16
                        || candidate.source_cell[1] >= 26
                        || candidate.source_sheet != format!("Terrain/terrain_{season}.png")
                        || !candidate.historical_summer_role_is_not_season_certification
                    {
                        return Err(format!("{season}: invalid source provenance at {id}"));
                    }
                }
            }
            let s = &profile.summary;
            if profile.cells.len() != 2526
                || s.occupied_derivative_cells != 2526
                || counts
                    != [
                        s.unique_candidate_cells,
                        s.ambiguous_candidate_cells,
                        s.unmatched_derivative_cells,
                    ]
            {
                return Err(format!("{season}: summary/row coverage mismatch"));
            }
        }
        Ok(())
    }
}

/// The large registry is parsed off the UI thread. No original art or recipe is written.
pub struct ReviewBrowser {
    pub path: PathBuf,
    pub registry: Option<ReviewRegistry>,
    pub error: Option<String>,
    pub loading: bool,
    pub season: String,
    pub match_filter: String,
    pub search: String,
    pub selected_tile_id: Option<u32>,
    pending: Option<Mutex<mpsc::Receiver<Result<ReviewRegistry, String>>>>,
}

impl ReviewBrowser {
    pub fn new(path: PathBuf) -> Self {
        let mut browser = Self {
            path,
            registry: None,
            error: None,
            loading: false,
            season: "summer".into(),
            match_filter: "All".into(),
            search: String::new(),
            selected_tile_id: None,
            pending: None,
        };
        browser.reload();
        browser
    }

    pub fn reload(&mut self) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.error = None;
        self.registry = None;
        self.selected_tile_id = None;
        let path = self.path.clone();
        let (tx, rx) = mpsc::channel();
        self.pending = Some(Mutex::new(rx));
        thread::spawn(move || {
            let result = fs::read_to_string(&path)
                .map_err(|error| {
                    format!(
                        "Cannot open {}: {error}. Generate the M2C4 registry first.",
                        path.display()
                    )
                })
                .and_then(|text| {
                    serde_json::from_str::<ReviewRegistry>(&text)
                        .map_err(|error| format!("Invalid M2C4 registry JSON: {error}"))
                })
                .and_then(|registry| {
                    registry.validate()?;
                    Ok(registry)
                });
            let _ = tx.send(result);
        });
    }

    pub fn poll(&mut self) {
        if !self.loading {
            return;
        }
        let message = self
            .pending
            .as_ref()
            .and_then(|rx| rx.lock().ok())
            .map(|rx| rx.try_recv());
        match message {
            Some(Ok(Ok(registry))) => {
                self.registry = Some(registry);
                self.loading = false;
                self.pending = None;
            }
            Some(Ok(Err(error))) => {
                self.error = Some(error);
                self.loading = false;
                self.pending = None;
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) | None => {
                self.error = Some("ElizaWy background evidence loader stopped unexpectedly".into());
                self.loading = false;
                self.pending = None;
            }
            Some(Err(mpsc::TryRecvError::Empty)) => {}
        }
    }

    pub fn matches_filter(&self, cell: &ReviewCell) -> bool {
        let allowed = match self.match_filter.as_str() {
            "Unique" => cell.match_status == "pixel_unique_candidate",
            "Ambiguous" => cell.match_status == "pixel_ambiguous_candidates",
            "Unmatched" => cell.match_status == "no_original_terrain_cell_match",
            _ => true,
        };
        if !allowed {
            return false;
        }
        let query = self.search.trim().to_ascii_lowercase();
        if query.is_empty() {
            return true;
        }
        cell.derivative_tile_id.to_string().contains(&query)
            || cell
                .tiled_tile_type
                .as_deref()
                .unwrap_or("")
                .to_ascii_lowercase()
                .contains(&query)
            || cell
                .tiled_wang_evidence
                .keys()
                .any(|key| key.to_ascii_lowercase().contains(&query))
            || cell.original_candidates.iter().any(|c| {
                c.historical_summer_group_id
                    .as_deref()
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains(&query)
                    || c.historical_summer_role
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains(&query)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_registry_does_not_become_editor_authority() {
        let invalid = ReviewRegistry {
            schema: SCHEMA.into(),
            authority: "evidence_only_zero_promotions".into(),
            archive_sha256: PINNED_ARCHIVE_SHA256.into(),
            historical_map_state: "sha256_verified".into(),
            certified_mappings_added: 1,
            runtime_changed: false,
            source_files_mutated: false,
            seasons: BTreeMap::new(),
        };
        assert!(invalid.validate().is_err());
    }
    #[test]
    fn status_filter_never_converts_evidence_to_a_mapping() {
        let cell = ReviewCell {
            derivative_tile_id: 0,
            derivative_cell: [0, 0],
            match_status: "pixel_ambiguous_candidates".into(),
            original_candidates: Vec::new(),
            tiled_wang_evidence: BTreeMap::new(),
            tiled_tile_type: None,
            tiled_has_collision_definition: false,
            tiled_animation_frames: Vec::new(),
            review_state: "unreviewed".into(),
            mapping_certified: false,
        };
        let mut browser = ReviewBrowser::new(PathBuf::from("/does/not/exist"));
        browser.match_filter = "Ambiguous".into();
        assert!(browser.matches_filter(&cell));
        browser.match_filter = "Unique".into();
        assert!(!browser.matches_filter(&cell));
    }
}
