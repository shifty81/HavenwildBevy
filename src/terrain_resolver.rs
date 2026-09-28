//! Shared semantic -> dual-grid -> recipe resolver contract.
//!
//! This module is intentionally UI-agnostic. The Studio, future world renderer,
//! and tests can consume the same resolved vertex records instead of reimplementing
//! topology interpretation in separate surfaces.

use crate::{
    semantic_lab::SemanticTerrainLab,
    terrain::{affected_vertices, CornerMask, TerrainVertexCoord, WorldCellCoord},
    terrain_mapper::TerrainMapper,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTerrainVertex {
    pub coord: TerrainVertexCoord,
    pub mask: CornerMask,
    pub resolution: String,
    pub confidence: Option<String>,
}

impl ResolvedTerrainVertex {
    pub fn recipe_key(&self) -> String {
        self.mask.key()
    }

    pub fn is_classified(&self) -> bool {
        matches!(
            self.resolution.as_str(),
            "sprite" | "composite" | "unsupported"
        )
    }

    pub fn is_certified(&self) -> bool {
        self.is_classified() && self.confidence.as_deref() == Some("certified")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainResolveGrid {
    pub size: [usize; 2],
    pub vertices: Vec<ResolvedTerrainVertex>,
}

impl TerrainResolveGrid {
    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        let width = self.size[0].saturating_add(1);
        let height = self.size[1].saturating_add(1);
        if x <= self.size[0] && y <= self.size[1] && y < height {
            Some(y * width + x)
        } else {
            None
        }
    }

    pub fn vertex(&self, x: usize, y: usize) -> Option<&ResolvedTerrainVertex> {
        self.index(x, y).and_then(|index| self.vertices.get(index))
    }

    pub fn classified_count(&self) -> usize {
        self.vertices
            .iter()
            .filter(|vertex| vertex.is_classified())
            .count()
    }

    pub fn certified_count(&self) -> usize {
        self.vertices
            .iter()
            .filter(|vertex| vertex.is_certified())
            .count()
    }

    pub fn mask_coverage(&self) -> [bool; 16] {
        let mut covered = [false; 16];
        for vertex in &self.vertices {
            covered[vertex.mask.0 as usize] = true;
        }
        covered
    }
}

pub fn resolve_semantic_lab(
    lab: &SemanticTerrainLab,
    mapper: &TerrainMapper,
) -> TerrainResolveGrid {
    let width = lab.size[0].saturating_add(1);
    let height = lab.size[1].saturating_add(1);
    let mut vertices = Vec::with_capacity(width.saturating_mul(height));
    for y in 0..height {
        for x in 0..width {
            let mask = CornerMask::new(lab.mask_at_vertex(x, y));
            let state = mapper.family.states.get(&mask.key());
            vertices.push(ResolvedTerrainVertex {
                coord: TerrainVertexCoord {
                    x: x as i32,
                    y: y as i32,
                },
                mask,
                resolution: state
                    .map(|state| state.resolution.clone())
                    .unwrap_or_else(|| "missing".into()),
                confidence: state.and_then(|state| state.confidence.clone()),
            });
        }
    }
    TerrainResolveGrid {
        size: lab.size,
        vertices,
    }
}

pub fn dirty_vertices_for_cell(x: usize, y: usize) -> [TerrainVertexCoord; 4] {
    affected_vertices(WorldCellCoord {
        x: x as i32,
        y: y as i32,
    })
}

pub fn all_vertex_coords(size: [usize; 2]) -> Vec<TerrainVertexCoord> {
    let mut result = Vec::with_capacity(
        size[0]
            .saturating_add(1)
            .saturating_mul(size[1].saturating_add(1)),
    );
    for y in 0..=size[1] {
        for x in 0..=size[0] {
            result.push(TerrainVertexCoord {
                x: x as i32,
                y: y as i32,
            });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain_mapper::{
        HistoricalAuthority, SummerRecoverySummary, SummerSourceDescriptor, SummerSourceInventory,
        TerrainRecipeFamily, TerrainRecipeState,
    };
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(1);

    fn test_mapper() -> TerrainMapper {
        let states = (0u8..16)
            .map(|mask| {
                (
                    format!("{mask:04b}"),
                    TerrainRecipeState {
                        resolution: if mask == 15 {
                            "sprite".into()
                        } else {
                            "unmapped".into()
                        },
                        confidence: (mask == 15).then(|| "certified".into()),
                        note: None,
                        source: None,
                        parts: Vec::new(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let family = TerrainRecipeFamily {
            schema: "havenwild.terrain.recipe_family.v1".into(),
            id: "test".into(),
            foreground: "Grass".into(),
            background: "Void".into(),
            season: "Summer".into(),
            tile_size_px: 32,
            source_authority: "test".into(),
            states,
        };
        let history = HistoricalAuthority {
            schema: "test".into(),
            purpose: "test".into(),
            canonical_summer: serde_json::json!({}),
            historical_evidence: Vec::new(),
            confidence_states: Vec::new(),
            rules: serde_json::json!({}),
        };
        let inventory = SummerSourceInventory {
            schema: "test".into(),
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
        };
        let root = std::env::temp_dir().join(format!(
            "havenwild-terrain-resolver-test-{}-{}",
            std::process::id(),
            NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("test directory");
        let family_path: PathBuf = root.join("family.json");
        let history_path = root.join("history.json");
        let inventory_path = root.join("inventory.json");
        fs::write(
            &family_path,
            serde_json::to_vec(&family).expect("serialize family"),
        )
        .expect("write family");
        fs::write(
            &history_path,
            serde_json::to_vec(&history).expect("serialize history"),
        )
        .expect("write history");
        fs::write(
            &inventory_path,
            serde_json::to_vec(&inventory).expect("serialize inventory"),
        )
        .expect("write inventory");
        TerrainMapper::load(family_path, history_path, inventory_path).expect("load mapper")
    }

    fn lab() -> SemanticTerrainLab {
        SemanticTerrainLab {
            schema: "havenwild.terrain.semantic_lab.v1".into(),
            id: "test".into(),
            foreground: "Grass".into(),
            background: "Void".into(),
            size: [5, 5],
            cells: vec![
                false, true, true, false, true, false, false, false, true, false, false, true,
                false, true, true, false, true, true, true, true, true, false, false, true, false,
            ],
        }
    }

    #[test]
    fn resolver_grid_uses_every_semantic_vertex() {
        let mapper = test_mapper();
        let grid = resolve_semantic_lab(&lab(), &mapper);
        assert_eq!(grid.vertices.len(), 36);
        assert!(grid.mask_coverage().into_iter().all(|covered| covered));
        assert_eq!(
            grid.vertex(0, 0).unwrap().coord,
            TerrainVertexCoord { x: 0, y: 0 }
        );
        assert_eq!(
            grid.vertex(5, 5).unwrap().coord,
            TerrainVertexCoord { x: 5, y: 5 }
        );
    }

    #[test]
    fn semantic_cell_dirties_exactly_four_vertices() {
        assert_eq!(
            dirty_vertices_for_cell(2, 3),
            [
                TerrainVertexCoord { x: 2, y: 3 },
                TerrainVertexCoord { x: 3, y: 3 },
                TerrainVertexCoord { x: 2, y: 4 },
                TerrainVertexCoord { x: 3, y: 4 },
            ]
        );
    }

    #[test]
    fn mapper_review_state_flows_into_runtime_resolution() {
        let mapper = test_mapper();
        let grid = resolve_semantic_lab(&lab(), &mapper);
        let full = grid
            .vertices
            .iter()
            .find(|vertex| vertex.mask.0 == 15)
            .expect("seed includes full foreground mask");
        assert_eq!(full.resolution, "sprite");
        assert_eq!(full.confidence.as_deref(), Some("certified"));
        assert!(full.is_certified());
    }
}
