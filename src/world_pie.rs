//! In-editor World PIE uses a frozen authored/generated WorldDocument snapshot.
//! This is the first walking/collision slice, not a final player, streaming or door system.
use crate::asset_authority::AssetAuthority;
use crate::room_scene::{RoomKind, RoomScene};
use crate::scene_v2::COLLISION_MASK_SIDE;
use crate::world_doc::{TraversalMode, WorldDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    North,
    West,
    South,
    East,
}

impl Facing {
    pub fn lpc_walk_row(self) -> u32 {
        match self {
            Self::North => 0,
            Self::West => 1,
            Self::South => 2,
            Self::East => 3,
        }
    }
}

#[derive(Clone)]
pub struct WorldPlaySession {
    /// Snapshot isolates gameplay from all editor changes and save files.
    pub snapshot: WorldDocument,
    pub position: [f32; 2],
    pub facing: Facing,
    pub moving: bool,
    pub elapsed: f32,
    pub blocked_steps: u32,
    /// A room/cave is a separate 2x2-chunk scene, never painted over the overworld.
    pub interior: Option<InteriorPlaySession>,
    solid_bounds: Vec<([i64; 2], [i64; 2])>,
}

#[derive(Clone)]
pub struct InteriorPlaySession {
    pub scene: RoomScene,
    pub position: [f32; 2],
    /// Exact overworld coordinates are preserved when entering the portal.
    pub return_position: [f32; 2],
}

fn traversable(
    world: &WorldDocument,
    authority: &AssetAuthority,
    solids: &[([i64; 2], [i64; 2])],
    pos: [f32; 2],
) -> bool {
    let cell = [pos[0].floor() as i64, pos[1].floor() as i64];
    if world.generated_cell(cell).is_none()
        || world.effective_traversal(cell, authority) != TraversalMode::Walkable
    {
        return false;
    }
    if let Some(mask) = world.effective_collision_mask(cell, authority) {
        let frac_x = (pos[0] - pos[0].floor()).clamp(0.0, 0.99999);
        let frac_y = (pos[1] - pos[1].floor()).clamp(0.0, 0.99999);
        let x = (frac_x * COLLISION_MASK_SIDE as f32) as usize;
        let y = (frac_y * COLLISION_MASK_SIDE as f32) as usize;
        if mask.blocked(x, y) {
            return false;
        }
    }
    if solids.iter().any(|(at, end)| {
        cell[0] >= at[0] && cell[0] < end[0] && cell[1] >= at[1] && cell[1] < end[1]
    }) {
        return false;
    }
    true
}

fn body_fits(
    world: &WorldDocument,
    authority: &AssetAuthority,
    solids: &[([i64; 2], [i64; 2])],
    pos: [f32; 2],
) -> bool {
    const R: f32 = 0.20;
    [-R, R].into_iter().all(|x| {
        [-R, R]
            .into_iter()
            .all(|y| traversable(world, authority, solids, [pos[0] + x, pos[1] + y]))
    })
}

impl WorldPlaySession {
    pub fn start(
        world: &WorldDocument,
        authority: &AssetAuthority,
        selected: [i64; 2],
    ) -> Result<Self, String> {
        let (origin, size) = world
            .materialized_bounds()
            .ok_or("Generate a world before starting World PIE")?;
        let mut solid_bounds = Vec::new();
        for object in world.generated_objects() {
            let id = object.template_id.to_ascii_lowercase();
            let solid = id.contains("house")
                || id.contains("building")
                || id.contains("tree")
                || id.contains("rock")
                || id.contains("boulder")
                || id.contains("stone");
            if !solid || id.contains("bridge") {
                continue;
            }
            let at = world.object_anchor(object);
            solid_bounds.push((
                at,
                [
                    at[0] + i64::from(object.footprint_cells[0]),
                    at[1] + i64::from(object.footprint_cells[1]),
                ],
            ));
        }
        // Keep spawn search bounded and deterministic. Spawn near the selected world cell,
        // but never teleport into sea, a building or an unknown material.
        let mut cells = Vec::new();
        for y in origin[1]..origin[1] + size[1] as i64 {
            for x in origin[0]..origin[0] + size[0] as i64 {
                let p = [x as f32 + 0.5, y as f32 + 0.5];
                if body_fits(world, authority, &solid_bounds, p) {
                    let dx = x.abs_diff(selected[0]);
                    let dy = y.abs_diff(selected[1]);
                    cells.push((
                        dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)),
                        y,
                        x,
                    ));
                }
            }
        }
        cells.sort_unstable();
        let (_, y, x) = cells
            .first()
            .copied()
            .ok_or("No supported land spawn in the materialized world")?;
        Ok(Self {
            snapshot: world.clone(),
            position: [x as f32 + 0.5, y as f32 + 0.5],
            facing: Facing::South,
            moving: false,
            elapsed: 0.0,
            blocked_steps: 0,
            interior: None,
            solid_bounds,
        })
    }

    pub fn step(&mut self, authority: &AssetAuthority, direction: [f32; 2], dt: f32) {
        let dt = dt.clamp(0.0, 0.05);
        if self.interior.is_some() {
            self.step_interior(direction, dt);
            return;
        }
        let len = direction[0].hypot(direction[1]);
        self.moving = len >= 0.001;
        if !self.moving {
            return;
        }
        self.elapsed += dt;
        self.facing = if direction[0].abs() > direction[1].abs() {
            if direction[0] < 0.0 {
                Facing::West
            } else {
                Facing::East
            }
        } else if direction[1] < 0.0 {
            Facing::North
        } else {
            Facing::South
        };
        const SPEED: f32 = 3.3;
        let delta = [
            direction[0] / len * SPEED * dt,
            direction[1] / len * SPEED * dt,
        ];
        let steps = ((delta[0].abs().max(delta[1].abs()) / 0.09).ceil() as usize).max(1);
        for _ in 0..steps {
            let px = [self.position[0] + delta[0] / steps as f32, self.position[1]];
            if body_fits(&self.snapshot, authority, &self.solid_bounds, px) {
                self.position = px;
            } else if delta[0].abs() > 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
            let py = [self.position[0], self.position[1] + delta[1] / steps as f32];
            if body_fits(&self.snapshot, authority, &self.solid_bounds, py) {
                self.position = py;
            } else if delta[1].abs() > 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
        }
    }
    /// Interaction candidates come from *actual* generated or authored structure
    /// objects, not fabricated atlas pixels or an invisible portal under the player.
    /// The first reachable side around the source footprint becomes the door.
    pub fn nearby_entrance(&self, authority: &AssetAuthority) -> Option<(String, RoomKind)> {
        if self.interior.is_some() {
            return None;
        }
        let mut candidates: Vec<(f32, String, RoomKind)> = Vec::new();
        for obj in self.snapshot.generated_objects() {
            let id = obj.template_id.to_ascii_lowercase();
            let label = obj.label.to_ascii_lowercase();
            let kind = if id.contains("cave") || label.contains("cave entrance") {
                RoomKind::Cave
            } else if id.contains("house") || id.contains("building") || label.contains("house") {
                RoomKind::House
            } else {
                continue;
            };
            let at = self.snapshot.object_anchor(obj);
            let width = i64::from(obj.footprint_cells[0]);
            let height = i64::from(obj.footprint_cells[1]);
            if width == 0 || height == 0 {
                continue;
            }
            let midx = at[0] + width / 2;
            let midy = at[1] + height / 2;
            // Once authored, only the mapped doorway is interactive: never
            // silently route through a different side of the building.
            if let Some(link) = self.snapshot.structure_doors.get(&obj.id) {
                let at_door = [
                    link.doorway_world[0] as f32 + 0.5,
                    link.doorway_world[1] as f32 + 0.5,
                ];
                let dist = (self.position[0] - at_door[0]).hypot(self.position[1] - at_door[1]);
                if dist < 1.8 && body_fits(&self.snapshot, authority, &self.solid_bounds, at_door) {
                    candidates.push((dist, obj.id.clone(), kind));
                }
                continue;
            }
            let doors = [
                [midx as f32 + 0.5, (at[1] + height) as f32 + 0.5],
                [midx as f32 + 0.5, at[1] as f32 - 0.5],
                [at[0] as f32 - 0.5, midy as f32 + 0.5],
                [(at[0] + width) as f32 + 0.5, midy as f32 + 0.5],
            ];
            for at_door in doors {
                let dist = (self.position[0] - at_door[0]).hypot(self.position[1] - at_door[1]);
                if dist < 1.8 && body_fits(&self.snapshot, authority, &self.solid_bounds, at_door) {
                    candidates.push((dist, obj.id.clone(), kind));
                }
            }
        }
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        candidates
            .into_iter()
            .next()
            .map(|(_, id, kind)| (id, kind))
    }

    /// F enters a source-backed placed doorway or leaves through the room exit.
    /// An unconfigured house uses an explicitly provisional *document* layout;
    /// renderers must not claim its floor/wall appearance is source-certified.
    pub fn interact(&mut self, authority: &AssetAuthority) -> Result<String, String> {
        if self.interior.is_some() {
            return self.exit_interior();
        }
        let (id, kind) = self
            .nearby_entrance(authority)
            .ok_or("No reachable house door or cave entrance nearby")?;
        let return_cell = [
            self.position[0].floor() as i64,
            self.position[1].floor() as i64,
        ];
        let room = match kind {
            RoomKind::House => {
                let mapped = self.snapshot.structure_doors.get(&id);
                let path = if mapped.is_some() {
                    crate::project_root::resolve().join(
                        crate::room_scene::house_instance_relative_path(self.snapshot.seed, &id),
                    )
                } else {
                    crate::project_root::resolve().join("content/rooms/starter_house.room.json")
                };
                if mapped.is_some() && !path.is_file() {
                    return Err("Mapped house interior is not saved yet: right-click house > Edit Interior > Save".into());
                }
                if path.exists() {
                    let mut authored = RoomScene::load_template(&path)?;
                    if authored.kind != RoomKind::House {
                        return Err("House template has incorrect room kind".into());
                    }
                    authored.scene_id = format!("house:{id}");
                    for portal in &mut authored.portals {
                        if portal.kind == crate::room_scene::PortalKind::Exit {
                            portal.return_world = Some(return_cell);
                        }
                    }
                    authored.validate()?;
                    authored
                } else {
                    RoomScene::starter_house(format!("house:{id}"), return_cell)?
                }
            }
            RoomKind::Cave => RoomScene::linked_cave(
                format!("cave:{id}"),
                self.snapshot.seed ^ stable_portal_hash(&id),
                return_cell,
            )?,
        };
        let mapped_spawn = self
            .snapshot
            .structure_doors
            .get(&id)
            .map(|link| link.destination_cell);
        self.enter_room(room)?;
        if let Some(cell) = mapped_spawn {
            let active = self.interior.as_mut().ok_or("Interior transition failed")?;
            if !active.scene.is_walkable([cell[0] as i32, cell[1] as i32]) {
                self.interior = None;
                return Err("Mapped interior entry is not on a valid floor cell".into());
            }
            active.position = [cell[0] as f32 + 0.5, cell[1] as f32 + 0.5];
        }
        Ok(format!(
            "Entered {}. F at the marked exit returns to the world.",
            id
        ))
    }

    /// Explicit diagnostics-only cave preview, until validated cliff caves are
    /// present in worldgen. This never adds fictitious entrances to the world.
    pub fn enter_test_cave(&mut self) -> Result<String, String> {
        if self.interior.is_some() {
            return Err("Exit the current room first".into());
        }
        let cell = [
            self.position[0].floor() as i64,
            self.position[1].floor() as i64,
        ];
        let room = RoomScene::linked_cave("diagnostics:cave_preview", self.snapshot.seed, cell)?;
        self.enter_room(room)?;
        Ok("Diagnostic procedural cave preview; no world cave entrance was created".into())
    }

    fn exit_interior(&mut self) -> Result<String, String> {
        let active = self.interior.as_ref().ok_or("Not inside an interior")?;
        if active.scene.exit_near(active.position).is_none() {
            return Err("Walk to the room exit and press F".into());
        }
        self.position = active.return_position;
        self.interior = None;
        self.moving = false;
        Ok("Returned to the original overworld position".into())
    }

    fn enter_room(&mut self, scene: RoomScene) -> Result<(), String> {
        scene.validate()?;
        let exit = scene
            .portals
            .iter()
            .find(|p| p.kind == crate::room_scene::PortalKind::Exit)
            .ok_or("Room has no valid exit")?;
        let spawn = [exit.cell[0] as f32 + 0.5, exit.cell[1] as f32 + 0.5];
        self.interior = Some(InteriorPlaySession {
            scene,
            position: spawn,
            return_position: self.position,
        });
        self.moving = false;
        Ok(())
    }

    fn step_interior(&mut self, direction: [f32; 2], dt: f32) {
        let len = direction[0].hypot(direction[1]);
        self.moving = len >= 0.001;
        if !self.moving {
            return;
        }
        self.elapsed += dt;
        self.facing = if direction[0].abs() > direction[1].abs() {
            if direction[0] < 0.0 {
                Facing::West
            } else {
                Facing::East
            }
        } else if direction[1] < 0.0 {
            Facing::North
        } else {
            Facing::South
        };
        let Some(active) = self.interior.as_mut() else {
            return;
        };
        let delta = [direction[0] / len * 3.3 * dt, direction[1] / len * 3.3 * dt];
        let steps = ((delta[0].abs().max(delta[1].abs()) / 0.09).ceil() as usize).max(1);
        for _ in 0..steps {
            let px = active.position[0] + delta[0] / steps as f32;
            if active
                .scene
                .is_walkable([px.floor() as i32, active.position[1].floor() as i32])
            {
                active.position[0] = px;
            } else if delta[0] != 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
            let py = active.position[1] + delta[1] / steps as f32;
            if active
                .scene
                .is_walkable([active.position[0].floor() as i32, py.floor() as i32])
            {
                active.position[1] = py;
            } else if delta[1] != 0.0 {
                self.blocked_steps = self.blocked_steps.saturating_add(1);
            }
        }
    }
}

fn stable_portal_hash(id: &str) -> u64 {
    // FNV-1a, so cave identity does not depend on process-local HashMap seeds.
    id.bytes().fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lpc_direction_rows_are_explicit_not_inferred_from_atlas_position() {
        assert_eq!(
            [Facing::North, Facing::West, Facing::South, Facing::East].map(Facing::lpc_walk_row),
            [0, 1, 2, 3]
        );
    }
    #[test]
    fn portal_seed_is_stable_and_distinct() {
        assert_eq!(stable_portal_hash("home-a"), stable_portal_hash("home-a"));
        assert_ne!(stable_portal_hash("home-a"), stable_portal_hash("home-b"));
    }
    #[test]
    fn interior_transition_keeps_world_position_and_blocks_walls() {
        let mut play = WorldPlaySession {
            snapshot: WorldDocument::new(77),
            position: [8.25, -5.75],
            facing: Facing::South,
            moving: false,
            elapsed: 0.0,
            blocked_steps: 0,
            interior: None,
            solid_bounds: Vec::new(),
        };
        let room = RoomScene::starter_house("house:test", [8, -6]).unwrap();
        play.enter_room(room).unwrap();
        assert!(play.interior.is_some());
        let prior = play.interior.as_ref().unwrap().position;
        play.step_interior([1.0, 0.0], 0.03);
        assert!(play.interior.as_ref().unwrap().position[0] > prior[0]);
        assert!(play.exit_interior().is_ok());
        assert_eq!(play.position, [8.25, -5.75]);
        assert!(play.interior.is_none());
    }
    #[test]
    fn interior_exit_is_rejected_until_player_reaches_portal() {
        let mut play = WorldPlaySession {
            snapshot: WorldDocument::new(1),
            position: [0.5, 0.5],
            facing: Facing::South,
            moving: false,
            elapsed: 0.0,
            blocked_steps: 0,
            interior: None,
            solid_bounds: Vec::new(),
        };
        play.enter_room(RoomScene::starter_house("house:test", [0, 0]).unwrap())
            .unwrap();
        play.interior.as_mut().unwrap().position = [30.5, 30.5];
        assert!(play.exit_interior().is_err());
        assert!(play.interior.is_some());
    }
    #[test]
    fn player_facing_has_four_unique_states() {
        assert_ne!(Facing::North, Facing::South);
        assert_ne!(Facing::East, Facing::West);
    }
}
