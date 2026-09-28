//! Semantic terrain/topology contracts for the standalone Havenwild Bevy lane.
//! Rendering code must consume resolved output rather than own terrain meaning.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainKind {
    #[default]
    Void,
    Grass,
    Dirt,
    Sand,
    Stone,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterKind {
    #[default]
    None,
    Shallow,
    Deep,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainCell {
    pub terrain: TerrainKind,
    pub elevation: i16,
    pub water: WaterKind,
    pub variation_seed: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldCellCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerrainVertexCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CornerMask(pub u8);

impl CornerMask {
    pub const NW: u8 = 0b0001;
    pub const NE: u8 = 0b0010;
    pub const SW: u8 = 0b0100;
    pub const SE: u8 = 0b1000;

    pub const fn new(value: u8) -> Self {
        Self(value & 0x0f)
    }

    pub fn key(self) -> String {
        format!("{:04b}", self.0)
    }
}

/// The four semantic cells sampled by one dual-grid terrain vertex.
/// Outside-world handling belongs to the caller (normally background/void).
pub const fn surrounding_cells(vertex: TerrainVertexCoord) -> [WorldCellCoord; 4] {
    [
        WorldCellCoord {
            x: vertex.x - 1,
            y: vertex.y - 1,
        },
        WorldCellCoord {
            x: vertex.x,
            y: vertex.y - 1,
        },
        WorldCellCoord {
            x: vertex.x - 1,
            y: vertex.y,
        },
        WorldCellCoord {
            x: vertex.x,
            y: vertex.y,
        },
    ]
}

/// Editing one semantic cell can only change these four dual-grid outputs.
pub const fn affected_vertices(cell: WorldCellCoord) -> [TerrainVertexCoord; 4] {
    [
        TerrainVertexCoord {
            x: cell.x,
            y: cell.y,
        },
        TerrainVertexCoord {
            x: cell.x + 1,
            y: cell.y,
        },
        TerrainVertexCoord {
            x: cell.x,
            y: cell.y + 1,
        },
        TerrainVertexCoord {
            x: cell.x + 1,
            y: cell.y + 1,
        },
    ]
}

/// Resolve the four semantic samples around a dual-grid vertex into the contract mask.
/// The caller owns world-boundary behavior through `is_foreground`; out-of-bounds
/// cells normally resolve as background/void.
pub fn corner_mask_for_vertex(
    vertex: TerrainVertexCoord,
    mut is_foreground: impl FnMut(WorldCellCoord) -> bool,
) -> CornerMask {
    let [nw, ne, sw, se] = surrounding_cells(vertex);
    let mut mask = 0u8;
    if is_foreground(nw) {
        mask |= CornerMask::NW;
    }
    if is_foreground(ne) {
        mask |= CornerMask::NE;
    }
    if is_foreground(sw) {
        mask |= CornerMask::SW;
    }
    if is_foreground(se) {
        mask |= CornerMask::SE;
    }
    CornerMask::new(mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_cell_targets_exact_four_vertices() {
        let result = affected_vertices(WorldCellCoord { x: 4, y: 7 });
        assert_eq!(
            result,
            [
                TerrainVertexCoord { x: 4, y: 7 },
                TerrainVertexCoord { x: 5, y: 7 },
                TerrainVertexCoord { x: 4, y: 8 },
                TerrainVertexCoord { x: 5, y: 8 },
            ]
        );
    }

    #[test]
    fn mask_key_uses_contract_bit_order() {
        assert_eq!(
            CornerMask::new(CornerMask::NW | CornerMask::SE).key(),
            "1001"
        );
    }

    #[test]
    fn resolver_samples_nw_ne_sw_se_in_contract_order() {
        let foreground = [WorldCellCoord { x: 3, y: 6 }, WorldCellCoord { x: 4, y: 7 }];
        let mask = corner_mask_for_vertex(TerrainVertexCoord { x: 4, y: 7 }, |coord| {
            foreground.contains(&coord)
        });
        assert_eq!(mask.0, CornerMask::NW | CornerMask::SE);
        assert_eq!(mask.key(), "1001");
    }

    #[test]
    fn one_foreground_cell_has_expected_mask_from_all_four_surrounding_vertices() {
        let cell = WorldCellCoord { x: 2, y: 3 };
        let cases = [
            (TerrainVertexCoord { x: 2, y: 3 }, CornerMask::SE),
            (TerrainVertexCoord { x: 3, y: 3 }, CornerMask::SW),
            (TerrainVertexCoord { x: 2, y: 4 }, CornerMask::NE),
            (TerrainVertexCoord { x: 3, y: 4 }, CornerMask::NW),
        ];
        for (vertex, expected) in cases {
            let mask = corner_mask_for_vertex(vertex, |coord| coord == cell);
            assert_eq!(mask.0, expected, "unexpected corner bit at {vertex:?}");
        }
    }
}
