//! Havenwild-native sparse Corner Wang solver.
//!
//! This module deliberately contains no Bevy dependency. It consumes the Wang
//! assignments already imported from Jaidyn's TSX and derives a temporary visual
//! vertex grid from Havenwild semantic vertices. When a two-material diagonal
//! checkerboard has no authored tile, the visual grid is repaired deterministically
//! until every cell maps to an authored exact Wang signature. Gameplay/world
//! semantics are never mutated by this solver.

use super::{stable_choice_hash, WangAuthoritySet, WangKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeCornerSolve {
    pub width: u32,
    pub height: u32,
    pub tile_ids: Vec<u32>,
    /// One-based Tiled Wang material IDs on the derived visual vertex grid.
    pub visual_vertex_ids: Vec<u8>,
    pub initial_unresolved_cells: usize,
    pub repaired_vertices: usize,
    pub repair_passes: usize,
}

fn corner_wang(corners: [u8; 4]) -> [u8; 8] {
    // Input: NW, NE, SW, SE.
    // Tiled: top, top-right, right, bottom-right, bottom,
    // bottom-left, left, top-left.
    [0, corners[1], 0, corners[3], 0, corners[2], 0, corners[0]]
}

fn vertex_index(vertex_width: u32, x: u32, y: u32) -> usize {
    (y as usize) * (vertex_width as usize) + x as usize
}

fn cell_corners(vertices: &[u8], vertex_width: u32, x: u32, y: u32) -> [u8; 4] {
    [
        vertices[vertex_index(vertex_width, x, y)],
        vertices[vertex_index(vertex_width, x + 1, y)],
        vertices[vertex_index(vertex_width, x, y + 1)],
        vertices[vertex_index(vertex_width, x + 1, y + 1)],
    ]
}

fn is_pair_checkerboard(corners: [u8; 4]) -> bool {
    corners[0] != 0
        && corners[1] != 0
        && corners[0] == corners[3]
        && corners[1] == corners[2]
        && corners[0] != corners[1]
}

fn unresolved_count(set: &WangAuthoritySet, vertices: &[u8], vertex_size: [u32; 2]) -> usize {
    let width = vertex_size[0] - 1;
    let height = vertex_size[1] - 1;
    let mut unresolved = 0usize;
    for y in 0..height {
        for x in 0..width {
            let wang = corner_wang(cell_corners(vertices, vertex_size[0], x, y));
            if set.exact_candidates(wang).is_empty() {
                unresolved += 1;
            }
        }
    }
    unresolved
}

/// Resolve a semantic vertex grid into an exact authored Corner Wang tile grid.
///
/// All authored exact cells remain exact. The only automatic visual repair this
/// lane permits is an unresolved two-material diagonal checkerboard. Such a cell
/// has no exact tile in Jaidyn's configured Terrain set (14/16 pair coverage), so
/// one visual vertex is deterministically reassigned to the opposite material.
/// The shared derived visual grid guarantees neighboring tiles remain seamless.
/// Any other missing topology fails closed for review instead of inventing art.
pub fn solve_corner_grid(
    set: &WangAuthoritySet,
    vertex_size: [u32; 2],
    semantic_vertex_ids: &[u8],
    seed: u64,
) -> Result<NativeCornerSolve, String> {
    solve_corner_grid_with_origin(set, vertex_size, semantic_vertex_ids, seed, [0, 0])
}

/// Same solver as [`solve_corner_grid`], but hashes repair/candidate choices in
/// absolute world-cell coordinates so solving a materialized window is stable
/// across camera position and chunk-window changes.
pub fn solve_corner_grid_with_origin(
    set: &WangAuthoritySet,
    vertex_size: [u32; 2],
    semantic_vertex_ids: &[u8],
    seed: u64,
    output_origin: [i32; 2],
) -> Result<NativeCornerSolve, String> {
    if set.kind != WangKind::Corner {
        return Err(format!("Wang set {:?} is not a Corner set", set.name));
    }
    if vertex_size[0] < 2 || vertex_size[1] < 2 {
        return Err("Semantic vertex grid must be at least 2x2".into());
    }
    let expected = usize::try_from(vertex_size[0])
        .ok()
        .and_then(|w| {
            usize::try_from(vertex_size[1])
                .ok()
                .and_then(|h| w.checked_mul(h))
        })
        .ok_or_else(|| "Semantic vertex grid dimensions overflow".to_owned())?;
    if semantic_vertex_ids.len() != expected {
        return Err(format!(
            "Semantic vertex grid contains {} values; expected {expected}",
            semantic_vertex_ids.len()
        ));
    }
    if semantic_vertex_ids.iter().any(|id| *id == 0) {
        return Err("Semantic vertex grid contains unset Wang material ID 0".into());
    }
    let max_material = u8::try_from(set.materials.len())
        .map_err(|_| "Wang set contains more than 255 terrain materials".to_owned())?;
    if semantic_vertex_ids.iter().any(|id| *id > max_material) {
        return Err(format!(
            "Semantic vertex grid references material outside set {:?}",
            set.name
        ));
    }

    let width = vertex_size[0] - 1;
    let height = vertex_size[1] - 1;
    let cell_count = usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
        .ok_or_else(|| "Solved tile grid dimensions overflow".to_owned())?;

    let mut visual = semantic_vertex_ids.to_vec();
    let initial_unresolved_cells = unresolved_count(set, &visual, vertex_size);
    let mut repaired_vertices = 0usize;
    let mut repair_passes = 0usize;
    let max_passes = cell_count.saturating_mul(4).max(8);

    while unresolved_count(set, &visual, vertex_size) > 0 {
        if repair_passes >= max_passes {
            return Err(format!(
                "Native Wang repair exceeded {max_passes} passes with {} unresolved cell(s)",
                unresolved_count(set, &visual, vertex_size)
            ));
        }
        let mut changed = false;

        for y in 0..height {
            for x in 0..width {
                let corners = cell_corners(&visual, vertex_size[0], x, y);
                if !set.exact_candidates(corner_wang(corners)).is_empty() {
                    continue;
                }
                if !is_pair_checkerboard(corners) {
                    return Err(format!(
                        "Unconfigured non-checkerboard Corner topology at cell ({x},{y}): {:?}",
                        corners
                    ));
                }

                // Pick one of the four shared visual vertices deterministically.
                // Re-evaluating in raster passes converges while avoiding one
                // permanent directional bias across the whole world.
                let selector = (stable_choice_hash(
                    seed,
                    [
                        output_origin[0].saturating_add(x as i32),
                        output_origin[1].saturating_add(y as i32),
                    ],
                    corner_wang(corners),
                ) & 3) as usize;
                let (vx, vy) = match selector {
                    0 => (x, y),
                    1 => (x + 1, y),
                    2 => (x, y + 1),
                    _ => (x + 1, y + 1),
                };
                let index = vertex_index(vertex_size[0], vx, vy);
                let before = visual[index];
                visual[index] = if before == corners[0] {
                    corners[1]
                } else {
                    corners[0]
                };
                if visual[index] != before {
                    repaired_vertices += 1;
                    changed = true;
                }
            }
        }

        repair_passes += 1;
        if !changed {
            return Err(format!(
                "Native Wang repair stalled with {} unresolved cell(s)",
                unresolved_count(set, &visual, vertex_size)
            ));
        }
    }

    let mut tile_ids = Vec::with_capacity(cell_count);
    for y in 0..height {
        for x in 0..width {
            let wang = corner_wang(cell_corners(&visual, vertex_size[0], x, y));
            let candidates = set.exact_candidates(wang);
            if candidates.is_empty() {
                return Err(format!(
                    "Native Wang solve left cell ({x},{y}) without an exact authored tile"
                ));
            }
            let hash = stable_choice_hash(
                seed,
                [
                    output_origin[0].saturating_add(x as i32),
                    output_origin[1].saturating_add(y as i32),
                ],
                wang,
            );
            tile_ids.push(candidates[(hash as usize) % candidates.len()]);
        }
    }

    Ok(NativeCornerSolve {
        width,
        height,
        tile_ids,
        visual_vertex_ids: visual,
        initial_unresolved_cells,
        repaired_vertices,
        repair_passes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiled_terrain::{WangAssignment, WangAuthoritySet};
    use std::collections::HashMap;

    fn binary_fourteen_set() -> WangAuthoritySet {
        let mut assignments = Vec::new();
        let mut exact: HashMap<[u8; 8], Vec<u32>> = HashMap::new();
        let mut tile_id = 0u32;
        for mask in 0u8..16 {
            let corners = [
                if mask & 1 != 0 { 2 } else { 1 },
                if mask & 2 != 0 { 2 } else { 1 },
                if mask & 4 != 0 { 2 } else { 1 },
                if mask & 8 != 0 { 2 } else { 1 },
            ];
            if is_pair_checkerboard(corners) {
                continue;
            }
            let wang = corner_wang(corners);
            assignments.push(WangAssignment {
                tile_id,
                wang_id: wang,
            });
            exact.entry(wang).or_default().push(tile_id);
            tile_id += 1;
        }
        WangAuthoritySet {
            name: "Terrain".into(),
            source_name: "Summer Terrain".into(),
            kind: WangKind::Corner,
            materials: vec!["Grass".into(), "Sand".into()],
            assignments,
            exact,
        }
    }

    #[test]
    fn exact_stripe_needs_no_visual_repair() {
        let set = binary_fourteen_set();
        let semantic = vec![1, 1, 2, 1, 1, 2, 1, 1, 2];
        let result = solve_corner_grid(&set, [3, 3], &semantic, 7).unwrap();
        assert_eq!(result.initial_unresolved_cells, 0);
        assert_eq!(result.visual_vertex_ids, semantic);
        assert_eq!(result.tile_ids.len(), 4);
    }

    #[test]
    fn checkerboards_become_seamless_exact_visual_topology() {
        let set = binary_fourteen_set();
        let semantic = vec![1, 2, 1, 2, 2, 1, 2, 1, 1, 2, 1, 2, 2, 1, 2, 1];
        let result = solve_corner_grid(&set, [4, 4], &semantic, 99).unwrap();
        assert!(result.initial_unresolved_cells > 0);
        assert!(result.repaired_vertices > 0);
        assert_eq!(result.tile_ids.len(), 9);
        assert_eq!(unresolved_count(&set, &result.visual_vertex_ids, [4, 4]), 0);
        assert_ne!(result.visual_vertex_ids, semantic);
    }

    #[test]
    fn native_solver_is_deterministic() {
        let set = binary_fourteen_set();
        let semantic = vec![1, 2, 1, 2, 2, 1, 2, 1, 1, 2, 1, 2, 2, 1, 2, 1];
        let a = solve_corner_grid(&set, [4, 4], &semantic, 12345).unwrap();
        let b = solve_corner_grid(&set, [4, 4], &semantic, 12345).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn unknown_sparse_topology_fails_closed() {
        let set = binary_fourteen_set();
        let semantic = vec![1, 2, 1, 2];
        // This fixture's checkerboard is supported by visual repair, so create
        // a different sparse set with no authored transitions at all.
        let mut sparse = set;
        sparse
            .assignments
            .retain(|entry| entry.wang_id == [0, 1, 0, 1, 0, 1, 0, 1]);
        sparse
            .exact
            .retain(|wang, _| *wang == [0, 1, 0, 1, 0, 1, 0, 1]);
        assert!(solve_corner_grid(&sparse, [2, 2], &semantic, 1).is_err());
    }
}
