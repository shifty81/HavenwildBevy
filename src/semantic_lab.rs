//! Small persisted semantic-only terrain lab used to exercise dual-grid resolution.
//! It deliberately contains no atlas paths or source rectangles.

use crate::{
    atomic_file::write_atomic,
    terrain::{corner_mask_for_vertex, TerrainVertexCoord, WorldCellCoord},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SemanticTerrainLab {
    pub schema: String,
    pub id: String,
    pub foreground: String,
    pub background: String,
    pub size: [usize; 2],
    pub cells: Vec<bool>,
}

impl SemanticTerrainLab {
    pub fn load(path: &Path) -> Result<Self, String> {
        let lab: Self = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if lab.schema != "havenwild.terrain.semantic_lab.v1"
            || lab.size[0] == 0
            || lab.size[1] == 0
            || lab.cells.len() != lab.size[0].saturating_mul(lab.size[1])
        {
            return Err("Unsupported or incomplete semantic terrain lab".into());
        }
        Ok(lab)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        write_atomic(path, &bytes)
    }

    pub fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.size[0] && y < self.size[1] {
            Some(y * self.size[0] + x)
        } else {
            None
        }
    }

    pub fn get(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        self.index(x as usize, y as usize)
            .and_then(|index| self.cells.get(index).copied())
            .unwrap_or(false)
    }

    pub fn toggle(&mut self, x: usize, y: usize) -> bool {
        let Some(index) = self.index(x, y) else {
            return false;
        };
        self.cells[index] = !self.cells[index];
        true
    }

    pub fn mask_at_vertex(&self, x: usize, y: usize) -> u8 {
        corner_mask_for_vertex(
            TerrainVertexCoord {
                x: x as i32,
                y: y as i32,
            },
            |coord: WorldCellCoord| self.get(coord.x, coord.y),
        )
        .0
    }

    pub fn topology_coverage(&self) -> [bool; 16] {
        let mut covered = [false; 16];
        for y in 0..=self.size[1] {
            for x in 0..=self.size[0] {
                covered[self.mask_at_vertex(x, y) as usize] = true;
            }
        }
        covered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_pattern_can_cover_all_sixteen_masks() {
        let lab = SemanticTerrainLab {
            schema: "havenwild.terrain.semantic_lab.v1".into(),
            id: "test".into(),
            foreground: "Grass".into(),
            background: "Void".into(),
            size: [5, 5],
            cells: vec![
                false, true, true, false, true, false, false, false, true, false, false, true,
                false, true, true, false, true, true, true, true, true, false, false, true, false,
            ],
        };
        assert!(lab.topology_coverage().into_iter().all(|covered| covered));
    }
}
