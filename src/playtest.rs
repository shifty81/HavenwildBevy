//! Source-addressed, in-editor playtest of the current authored scene.
//! This is deliberately NOT a reconstructed demo image, production collision contract,
//! map generator, character art substitute, or separate game runtime.
use crate::document::Scene;

/// The only provisional traversal policy backed by the v1 fixture's legacy hints.
/// Unknown roles and erased/unbound artwork fail closed. Scene-v2 structural navigation
/// must replace this preview policy before this can be called gameplay certification.
fn provisional_walkable(scene: &Scene, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 {
        return false;
    }
    let Some(index) = scene.index(x as usize, y as usize) else {
        return false;
    };
    let tile = &scene.tiles[index];
    !tile.is_empty()
        && tile.source_asset == "Terrain/terrain_summer.png"
        && matches!(tile.semantic_role_hint.as_str(), "Grass" | "MudBank")
}

fn body_fits(scene: &Scene, position: [f32; 2]) -> bool {
    const R: f32 = 0.23;
    for dx in [-R, R] {
        for dy in [-R, R] {
            let x = (position[0] + dx).floor() as i32;
            let y = (position[1] + dy).floor() as i32;
            if !provisional_walkable(scene, x, y) {
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
}

impl PlaySession {
    pub fn start(scene: &Scene, preferred: [usize; 2]) -> Result<Self, String> {
        scene.validate()?;
        // Deterministic nearest supported spawn; never guess a collision shape from sprite alpha.
        let mut locations = Vec::new();
        for y in 0..scene.size[1] {
            for x in 0..scene.size[0] {
                if provisional_walkable(scene, x as i32, y as i32) {
                    let dx = x.abs_diff(preferred[0]);
                    let dy = y.abs_diff(preferred[1]);
                    locations.push((dx * dx + dy * dy, y, x));
                }
            }
        }
        locations.sort_unstable();
        for (_, y, x) in locations {
            let position = [x as f32 + 0.5, y as f32 + 0.5];
            if body_fits(scene, position) {
                return Ok(Self {
                    snapshot: scene.clone(),
                    position,
                    elapsed: 0.0,
                    blocked_steps: 0,
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
            if body_fits(&self.snapshot, next_x) {
                self.position = next_x;
            } else if movement[0].abs() > 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
            let next_y = [
                self.position[0],
                self.position[1] + movement[1] / steps as f32,
            ];
            if body_fits(&self.snapshot, next_y) {
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
        assert!(!provisional_walkable(&doc, 0, 0));
        assert!(!provisional_walkable(&doc, 1, 0));
        assert!(!provisional_walkable(&doc, -1, 0));
        doc.tiles[2].source_asset = "Terrain/cliff_summer.png".into();
        assert!(!provisional_walkable(&doc, 2, 0));
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
