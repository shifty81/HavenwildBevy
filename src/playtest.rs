//! Source-addressed, in-editor playtest of the current authored scene.
//! This is deliberately NOT a reconstructed demo image, production collision contract,
//! map generator, character art substitute, or separate game runtime.
use crate::document::Scene;
use crate::scene_v2::{CollisionCell, StructuralCell, COLLISION_MASK_SIDE};

/// The only provisional traversal policy backed by the v1 fixture's legacy hints.
/// Unknown roles and erased/unbound artwork fail closed. Scene-v2 structural navigation
/// must replace this preview policy before this can be called gameplay certification.
pub(crate) fn provisional_walkable(
    scene: &Scene,
    structural: &[StructuralCell],
    x: i32,
    y: i32,
) -> bool {
    if x < 0 || y < 0 {
        return false;
    }
    let Some(index) = scene.index(x as usize, y as usize) else {
        return false;
    };
    if let Some(explicit) = structural
        .iter()
        .find(|c| c.index == index)
        .and_then(|c| c.blocks_traversal)
    {
        return !explicit;
    }
    let tile = &scene.tiles[index];
    !tile.is_empty()
        && tile.source_asset == "Terrain/terrain_summer.png"
        && matches!(tile.semantic_role_hint.as_str(), "Grass" | "MudBank")
}

fn point_blocked(
    scene: &Scene,
    structural: &[StructuralCell],
    collision: &[CollisionCell],
    position: [f32; 2],
) -> bool {
    if position[0] < 0.0 || position[1] < 0.0 {
        return true;
    }
    let x = position[0].floor() as i32;
    let y = position[1].floor() as i32;
    let Some(index) = scene.index(x as usize, y as usize) else {
        return true;
    };
    if let Some(mask) = collision
        .iter()
        .find(|cell| cell.index == index)
        .map(|cell| &cell.mask)
    {
        let local_x = ((position[0] - x as f32) * COLLISION_MASK_SIDE as f32)
            .floor()
            .clamp(0.0, (COLLISION_MASK_SIDE - 1) as f32) as usize;
        let local_y = ((position[1] - y as f32) * COLLISION_MASK_SIDE as f32)
            .floor()
            .clamp(0.0, (COLLISION_MASK_SIDE - 1) as f32) as usize;
        return mask.blocked(local_x, local_y);
    }
    !provisional_walkable(scene, structural, x, y)
}

fn body_fits(
    scene: &Scene,
    structural: &[StructuralCell],
    collision: &[CollisionCell],
    position: [f32; 2],
) -> bool {
    const R: f32 = 0.23;
    for dx in [-R, R] {
        for dy in [-R, R] {
            if point_blocked(
                scene,
                structural,
                collision,
                [position[0] + dx, position[1] + dy],
            ) {
                return false;
            }
        }
    }
    true
}

#[derive(Clone, Debug)]
pub struct PlaySession {
    /// Immutable source-addressed scene snapshot. Editor modifications never affect PIE.
    pub snapshot: Scene,
    pub position: [f32; 2], // continuous coordinates in 32px source cells
    pub elapsed: f32,
    pub blocked_steps: u32,
    pub structural_snapshot: Vec<StructuralCell>,
    pub collision_snapshot: Vec<CollisionCell>,
}

impl PlaySession {
    pub fn start(scene: &Scene, preferred: [usize; 2]) -> Result<Self, String> {
        Self::start_with_collision(scene, preferred, &[], &[])
    }
    pub fn start_with_structure(
        scene: &Scene,
        preferred: [usize; 2],
        structural: &[StructuralCell],
    ) -> Result<Self, String> {
        Self::start_with_collision(scene, preferred, structural, &[])
    }
    pub fn start_with_collision(
        scene: &Scene,
        preferred: [usize; 2],
        structural: &[StructuralCell],
        collision: &[CollisionCell],
    ) -> Result<Self, String> {
        scene.validate()?;
        // Deterministic nearest supported spawn; collision masks are authored metadata,
        // never inferred from artist alpha. A partial/empty mask can make a coarse blocked
        // source cell traversable only where the author explicitly cleared pixels.
        let mut locations = Vec::new();
        for y in 0..scene.size[1] {
            for x in 0..scene.size[0] {
                let index = y * scene.size[0] + x;
                let candidate = collision
                    .iter()
                    .find(|cell| cell.index == index)
                    .is_some_and(|cell| cell.mask.blocked_count() < 1024)
                    || provisional_walkable(scene, structural, x as i32, y as i32);
                if candidate {
                    let dx = x.abs_diff(preferred[0]);
                    let dy = y.abs_diff(preferred[1]);
                    locations.push((dx * dx + dy * dy, y, x));
                }
            }
        }
        locations.sort_unstable();
        for (_, y, x) in locations {
            let position = [x as f32 + 0.5, y as f32 + 0.5];
            if body_fits(scene, structural, collision, position) {
                return Ok(Self {
                    snapshot: scene.clone(),
                    position,
                    elapsed: 0.0,
                    blocked_steps: 0,
                    structural_snapshot: structural.to_vec(),
                    collision_snapshot: collision.to_vec(),
                });
            }
        }
        Err("No valid land spawn in this source-addressed scene. No PIE operation started.".into())
    }

    pub fn step(&mut self, direction: [f32; 2], dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        self.elapsed += dt;
        let magnitude = direction[0].hypot(direction[1]);
        if magnitude < 0.001 {
            return;
        }
        const SPEED_CELLS_PER_SECOND: f32 = 4.0;
        let movement = [
            direction[0] / magnitude * SPEED_CELLS_PER_SECOND * dt,
            direction[1] / magnitude * SPEED_CELLS_PER_SECOND * dt,
        ];
        // Segmented axis movement permits sliding while preventing crossing blocked river cells
        // at a frame hitch or diagonal corner. Nothing writes back to the authoring scene.
        let steps = ((movement[0].abs().max(movement[1].abs()) / 0.09).ceil() as usize).max(1);
        for _ in 0..steps {
            let next_x = [
                self.position[0] + movement[0] / steps as f32,
                self.position[1],
            ];
            if body_fits(
                &self.snapshot,
                &self.structural_snapshot,
                &self.collision_snapshot,
                next_x,
            ) {
                self.position = next_x;
            } else if movement[0].abs() > 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
            let next_y = [
                self.position[0],
                self.position[1] + movement[1] / steps as f32,
            ];
            if body_fits(
                &self.snapshot,
                &self.structural_snapshot,
                &self.collision_snapshot,
                next_y,
            ) {
                self.position = next_y;
            } else if movement[1].abs() > 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Tile;
    fn tile(role: &str) -> Tile {
        Tile {
            semantic_role_hint: role.into(),
            source_asset: "Terrain/terrain_summer.png".into(),
            source_rect: [96, 32, 32, 32],
            status: "draft".into(),
        }
    }
    fn map() -> Scene {
        let mut tiles = vec![tile("Grass"); 25];
        for y in 0..5 {
            tiles[y * 5 + 3] = tile("RiverWater");
        }
        Scene {
            schema: "havenwild.elizawy.scene.v1".into(),
            name: "test".into(),
            size: [5, 5],
            tile_size: 32,
            tiles,
            hidden_visual_samples: Vec::new(),
        }
    }
    #[test]
    fn explicitly_authored_collision_overrides_are_snapshot_only() {
        let map = map();
        let blocks = [StructuralCell {
            index: 1 * 5 + 2,
            elevation: Some(1),
            terrain_kind: None,
            water_flow: None,
            blocks_traversal: Some(true),
            connector: None,
        }];
        assert!(!provisional_walkable(&map, &blocks, 2, 1));
        assert!(provisional_walkable(&map, &[], 2, 1));
        let free = [StructuralCell {
            index: 1 * 5 + 3,
            elevation: None,
            terrain_kind: None,
            water_flow: None,
            blocks_traversal: Some(false),
            connector: None,
        }];
        assert!(provisional_walkable(&map, &free, 3, 1));
    }
    #[test]
    fn pixel_collision_mask_overrides_coarse_cell_collision() {
        let map = map();
        let blocks = [StructuralCell {
            index: 1 * 5 + 2,
            elevation: Some(1),
            terrain_kind: None,
            water_flow: None,
            blocks_traversal: Some(true),
            connector: None,
        }];
        let mut mask = crate::scene_v2::CollisionMask32::full();
        for y in 8..24 {
            for x in 8..24 {
                mask.set_blocked(x, y, false);
            }
        }
        let collision = [CollisionCell {
            index: 1 * 5 + 2,
            mask,
        }];
        assert!(!point_blocked(&map, &blocks, &collision, [2.5, 1.5]));
        assert!(point_blocked(&map, &blocks, &collision, [2.05, 1.05]));
    }

    #[test]
    fn pie_snapshots_the_same_scene_without_editing_original_or_source() {
        let mut editor = map();
        let mut play = PlaySession::start(&editor, [1, 2]).unwrap();
        editor.tiles[0].clear_source_binding();
        play.step([1.0, 0.0], 0.05);
        assert!(!play.snapshot.tiles[0].is_empty());
        assert_ne!(play.snapshot.tiles[0], editor.tiles[0]);
        assert_eq!(editor.tiles[1].semantic_role_hint, "Grass");
    }
    #[test]
    fn river_hint_blocks_provisional_crossing_even_under_repeated_input() {
        let mut play = PlaySession::start(&map(), [2, 2]).unwrap();
        for _ in 0..300 {
            play.step([1.0, 0.0], 0.05);
        }
        assert!(play.position[0] < 2.8);
        assert!(play.blocked_steps > 0);
    }
    #[test]
    fn unknown_roles_and_erased_cells_are_not_implicitly_walkable() {
        let mut doc = map();
        doc.tiles[0].semantic_role_hint = "UnverifiedCliff".into();
        doc.tiles[1].clear_source_binding();
        assert!(!provisional_walkable(&doc, &[], 0, 0));
        assert!(!provisional_walkable(&doc, &[], 1, 0));
        assert!(!provisional_walkable(&doc, &[], -1, 0));
        doc.tiles[2].source_asset = "Terrain/cliff_summer.png".into();
        assert!(!provisional_walkable(&doc, &[], 2, 0));
    }
    #[test]
    fn no_walkable_spawn_refuses_to_start() {
        let mut doc = map();
        for tile in &mut doc.tiles {
            tile.semantic_role_hint = "RiverWater".into();
        }
        assert!(PlaySession::start(&doc, [2, 2]).is_err());
    }
    #[test]
    fn diagonal_movement_is_normalized_and_is_not_faster_than_cardinal() {
        let mut straight = PlaySession::start(&map(), [1, 2]).unwrap();
        let mut diagonal = straight.clone();
        straight.step([0.0, 1.0], 0.05);
        diagonal.step([1.0, 1.0], 0.05);
        let a = straight.position[1] - 2.5;
        let b = (diagonal.position[0] - 1.5).hypot(diagonal.position[1] - 2.5);
        assert!((a - b).abs() < 0.0001);
    }
}
