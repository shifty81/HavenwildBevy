//! Bevy-owned read-only historical ElizaWy evidence. This is not M2C4's
//! derivative registry, terrain-role authority, approval, or live collision.
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRegistry {
    pub schema: String,
    pub source_count: usize,
    pub runtime_enabled: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audit {
    pub source_sheets: usize,
    pub source_regions: usize,
    pub semantic_mappings_promoted: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterMember {
    pub source_path: String,
    pub source_rect_px: [u32; 4],
    pub family_hint: String,
    pub source_cell: [u32; 2],
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCluster {
    pub cluster_id: String,
    pub season: String,
    pub triage_category: String,
    pub canonical_atlas: Option<String>,
    pub canonical_match_status: String,
    pub member_count: usize,
    pub membership: Vec<ClusterMember>,
    pub approved: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewTriage {
    pub schema: String,
    pub authority: String,
    pub cluster_count: usize,
    pub row_count: usize,
    pub clusters: Vec<ReviewCluster>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSource {
    pub schema: String,
    pub region_count: usize,
    pub runtime_enabled: bool,
    pub entries: Vec<RegionEntry>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionEntry {
    pub region_id: String,
    pub source_path: String,
    pub source_rect_px: [u32; 4],
    pub source_pixel_sha256: String,
    pub family_hint: String,
    pub transparent: bool,
    pub semantic_review: String,
    pub seasonal_candidates: Vec<SeasonCandidate>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonCandidate {
    pub season: String,
    pub canonical_atlas: Option<String>,
    pub canonical_match_status: String,
    pub exact_canonical_cells: Vec<[u32; 2]>,
}
// An absent canonical atlas is a meaningful historical no-match record, not
// a missing original source sprite. Never rewrite the signed source evidence.
fn canonical_atlas_valid(status: &str, atlas: Option<&str>) -> bool {
    match (status, atlas) {
        ("none", None) => true,
        ("none" | "one" | "multiple", Some(path)) => !path.trim().is_empty(),
        _ => false,
    }
}

pub fn canonical_atlas_label(atlas: Option<&str>) -> &str {
    atlas.unwrap_or("No canonical atlas (original source preserved)")
}

#[derive(Debug)]
pub struct HistoricalEvidence {
    pub audit: Audit,
    pub triage: ReviewTriage,
    pub regions: RegionSource,
    pub view: String,
    pub selected_region: Option<usize>,
    pub filter: String,
    pub season: String,
    pub selected_cluster: Option<usize>,
}
impl HistoricalEvidence {
    pub fn load(root: &Path) -> Result<Self, String> {
        let base = root.join("content/mapping/recovered");
        fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
            let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
            serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
        }
        let source: SourceRegistry = read(&base.join("source_registry.v1.json"))?;
        let audit: Audit = read(&base.join("consolidation_audit.v1.json"))?;
        let triage: ReviewTriage = read(&base.join("review_triage.v3.json"))?;
        let regions: RegionSource = read(&base.join("region_candidates.v1.json"))?;
        if source.schema != "havenwild.bevy.elizawy.source_registry.candidate.v1"
            || source.source_count != 40
            || source.runtime_enabled
            || audit.source_sheets != source.source_count
            || audit.source_regions != 1781
            || audit.semantic_mappings_promoted != 0
            || regions.schema != "havenwild.bevy.elizawy.region_candidates.v1"
            || regions.runtime_enabled
            || regions.region_count != audit.source_regions
            || regions.entries.len() != regions.region_count
            || regions.entries.iter().any(|r| {
                r.region_id.is_empty()
                    || r.source_path.is_empty()
                    || r.source_rect_px[2] != 32
                    || r.source_rect_px[3] != 32
                    || r.semantic_review != "not_promoted"
                    || r.source_pixel_sha256.len() != 64
                    || r.seasonal_candidates.iter().any(|c| {
                        !canonical_atlas_valid(
                            &c.canonical_match_status,
                            c.canonical_atlas.as_deref(),
                        ) || match c.canonical_match_status.as_str() {
                            "none" => !c.exact_canonical_cells.is_empty(),
                            "one" => c.exact_canonical_cells.len() != 1,
                            "multiple" => c.exact_canonical_cells.len() < 2,
                            _ => true,
                        }
                    })
            })
            || triage.schema != "havenwild.bevy.elizawy.review_triage.v3"
            || triage.authority != "historical_evidence_only"
            || triage.cluster_count != triage.clusters.len()
            || triage.row_count != 787
            || triage.clusters.iter().any(|c| {
                c.approved
                    || c.member_count != c.membership.len()
                    || c.membership.is_empty()
                    || c.cluster_id.is_empty()
                    || !canonical_atlas_valid(
                        &c.canonical_match_status,
                        c.canonical_atlas.as_deref(),
                    )
            })
        {
            return Err("Historical source-evidence identity/count or no-promotion contract differs; no automatic imports".into());
        }
        Ok(Self {
            audit,
            triage,
            regions,
            view: "regions".into(),
            selected_region: None,
            filter: String::new(),
            season: "all".into(),
            selected_cluster: None,
        })
    }
    pub fn filtered_regions(&self) -> Vec<(usize, String)> {
        let term = self.filter.trim().to_lowercase();
        self.regions
            .entries
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                (self.season == "all"
                    || r.seasonal_candidates
                        .iter()
                        .any(|s| s.season == self.season))
                    && (term.is_empty()
                        || r.source_path.to_lowercase().contains(&term)
                        || r.family_hint.to_lowercase().contains(&term)
                        || r.region_id.to_lowercase().contains(&term))
            })
            .map(|(i, r)| {
                (
                    i,
                    format!(
                        "{} · {} @ {},{} {}×{}",
                        r.family_hint,
                        r.source_path,
                        r.source_rect_px[0],
                        r.source_rect_px[1],
                        r.source_rect_px[2],
                        r.source_rect_px[3]
                    ),
                )
            })
            .collect()
    }
    pub fn filtered_rows(&self) -> Vec<(usize, String)> {
        let term = self.filter.trim().to_lowercase();
        self.triage
            .clusters
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                (self.season == "all" || self.season == c.season)
                    && (term.is_empty()
                        || c.triage_category.to_lowercase().contains(&term)
                        || c.canonical_atlas
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&term)
                        || c.membership.iter().any(|m| {
                            m.source_path.to_lowercase().contains(&term)
                                || m.family_hint.to_lowercase().contains(&term)
                        }))
            })
            .map(|(i, c)| {
                (
                    i,
                    format!(
                        "{} · {} · {} · {} source",
                        c.season,
                        c.triage_category.replace('_', " "),
                        canonical_atlas_label(c.canonical_atlas.as_deref()),
                        c.member_count
                    ),
                )
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_historical_review_ledger_is_valid_and_remains_unpromoted() {
        let evidence = HistoricalEvidence::load(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert_eq!(evidence.audit.source_regions, 1781);
        assert_eq!(evidence.regions.entries.len(), 1781);
        assert_eq!(evidence.view, "regions");
        assert_eq!(evidence.triage.cluster_count, 284);
        assert_eq!(evidence.triage.row_count, 787);
        assert_eq!(evidence.audit.semantic_mappings_promoted, 0);
        // The current evidence includes real "no canonical atlas" cases. Null is
        // evidence of no target, not a malformed original-source record.
        assert_eq!(
            evidence
                .triage
                .clusters
                .iter()
                .filter(|c| c.canonical_atlas.is_none())
                .count(),
            21
        );
        assert_eq!(
            evidence
                .regions
                .entries
                .iter()
                .flat_map(|r| &r.seasonal_candidates)
                .filter(|c| c.canonical_atlas.is_none())
                .count(),
            21
        );
        assert!(evidence.triage.clusters.iter().all(|c| !c.approved));
    }

    #[test]
    fn null_canonical_atlas_is_valid_only_for_an_unmatched_target() {
        assert!(canonical_atlas_valid("none", None));
        assert!(canonical_atlas_valid(
            "none",
            Some("Terrain/terrain_summer.png")
        ));
        assert!(canonical_atlas_valid(
            "one",
            Some("Terrain/terrain_summer.png")
        ));
        assert!(canonical_atlas_valid(
            "multiple",
            Some("Terrain/terrain_summer.png")
        ));
        assert!(!canonical_atlas_valid("one", None));
        assert!(!canonical_atlas_valid("multiple", None));
        assert!(!canonical_atlas_valid("none", Some("")));
        assert!(!canonical_atlas_valid(
            "unknown",
            Some("Terrain/terrain_summer.png")
        ));
        assert_eq!(
            canonical_atlas_label(None),
            "No canonical atlas (original source preserved)"
        );
    }
}
