//! Authored rooms and connected procedural caves for the 2x2/3x3 scene lane.
//! This is pure document logic. Rendering, GUI painting, and PIE portal activation
//! must integrate this contract in a subsequent native-verified pass.
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const ROOM_CHUNK_SIDE: u32 = 32;
pub const DEFAULT_ROOM_CHUNKS: [u8; 2] = [2, 2];
pub const MAX_ROOM_CHUNKS: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomKind {
    House,
    Cave,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomStyle {
    /// Canonical asset path or a project-owned derivative; empty means unresolved.
    pub floor_asset: String,
    pub wall_asset: String,
    /// Runtime presentation choice; source pixels are never changed.
    pub wall_tint_rgba: [u8; 4],
}

impl Default for RoomStyle {
    fn default() -> Self {
        Self {
            floor_asset: String::new(),
            wall_asset: String::new(),
            wall_tint_rgba: [255; 4],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomTile {
    pub floor: bool,
    /// 1=N, 2=E, 4=S, 8=W; all connected sides combine deterministically.
    pub wall_neighbor_mask: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortalKind {
    HouseDoor,
    CaveEntrance,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalLink {
    pub id: String,
    pub kind: PortalKind,
    pub cell: [u32; 2],
    pub destination_scene: String,
    pub destination_cell: [u32; 2],
    /// Only for world return links; a matching link in the target scene is required.
    pub return_world: Option<[i64; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomScene {
    pub schema: String,
    pub scene_id: String,
    pub kind: RoomKind,
    pub chunks: [u8; 2],
    pub style: RoomStyle,
    pub cells: Vec<RoomTile>,
    pub portals: Vec<PortalLink>,
    /// Cave only; authored rooms keep 0 and are never regenerated accidentally.
    pub generation_seed: u64,
}

/// A house's room is project-owned and keyed by stable world seed + object identity.
/// Do not use arbitrary object IDs as paths or overwrite the shared starter template.
pub fn house_instance_relative_path(seed: u64, object_id: &str) -> String {
    let hash = object_id.bytes().fold(0xcbf2_9ce4_8422_2325u64, |v, byte| {
        (v ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
    });
    format!("content/rooms/houses/house_{seed:016x}_{hash:016x}.room.json")
}

impl RoomScene {
    pub fn new(
        scene_id: impl Into<String>,
        kind: RoomKind,
        chunks: [u8; 2],
        style: RoomStyle,
    ) -> Result<Self, String> {
        if !chunks.iter().all(|&n| (2..=MAX_ROOM_CHUNKS).contains(&n)) {
            return Err("Room scenes must occupy 2x2 through 3x3 chunks".into());
        }
        let width = usize::from(chunks[0]) * ROOM_CHUNK_SIDE as usize;
        let height = usize::from(chunks[1]) * ROOM_CHUNK_SIDE as usize;
        let id = scene_id.into();
        if id.is_empty() {
            return Err("Room scene identity cannot be empty".into());
        }
        Ok(Self {
            schema: "havenwild.bevy.room_scene.v1".into(),
            scene_id: id,
            kind,
            chunks,
            style,
            cells: vec![RoomTile::default(); width * height],
            portals: Vec::new(),
            generation_seed: 0,
        })
    }
    pub fn dimensions(&self) -> [u32; 2] {
        [
            u32::from(self.chunks[0]) * ROOM_CHUNK_SIDE,
            u32::from(self.chunks[1]) * ROOM_CHUNK_SIDE,
        ]
    }
    /// Resize only within the 2–3 chunk budget; preserve existing floor and
    /// portal coordinates and rebuild derived walls. Never silently clip a door.
    pub fn resize_chunks(&mut self, chunks: [u8; 2]) -> Result<(), String> {
        if !chunks.iter().all(|&n| (2..=MAX_ROOM_CHUNKS).contains(&n)) {
            return Err("Room size must remain between 2x2 and 3x3 chunks".into());
        }
        if self.chunks == chunks {
            return Ok(());
        }
        let old_size = self.dimensions();
        let new_size = [
            u32::from(chunks[0]) * ROOM_CHUNK_SIDE,
            u32::from(chunks[1]) * ROOM_CHUNK_SIDE,
        ];
        if self
            .portals
            .iter()
            .any(|p| p.cell[0] >= new_size[0] || p.cell[1] >= new_size[1])
        {
            return Err("Resize would discard a room portal; move it first".into());
        }
        let mut cells = vec![RoomTile::default(); (new_size[0] * new_size[1]) as usize];
        for y in 0..old_size[1].min(new_size[1]) {
            for x in 0..old_size[0].min(new_size[0]) {
                cells[(y * new_size[0] + x) as usize].floor =
                    self.cells[(y * old_size[0] + x) as usize].floor;
            }
        }
        let previous_cells = std::mem::replace(&mut self.cells, cells);
        let previous_chunks = std::mem::replace(&mut self.chunks, chunks);
        self.rebuild_walls();
        if let Err(error) = self.validate() {
            self.cells = previous_cells;
            self.chunks = previous_chunks;
            return Err(error);
        }
        Ok(())
    }
    pub fn index(&self, cell: [u32; 2]) -> Option<usize> {
        let size = self.dimensions();
        if cell[0] < size[0] && cell[1] < size[1] {
            Some(cell[1] as usize * size[0] as usize + cell[0] as usize)
        } else {
            None
        }
    }
    /// Paint one floor cell. The surrounding walls are derived, never manually guessed.
    pub fn paint_floor(&mut self, cell: [u32; 2], value: bool) -> Result<(), String> {
        let index = self
            .index(cell)
            .ok_or("Floor paint is outside the room scene")?;
        self.cells[index].floor = value;
        self.rebuild_walls();
        Ok(())
    }
    /// One transaction for a room-sized floor footprint, useful for drag rectangles.
    pub fn paint_floor_rect(
        &mut self,
        min: [u32; 2],
        max_exclusive: [u32; 2],
        value: bool,
    ) -> Result<(), String> {
        let dimensions = self.dimensions();
        if min[0] >= max_exclusive[0]
            || min[1] >= max_exclusive[1]
            || max_exclusive[0] > dimensions[0]
            || max_exclusive[1] > dimensions[1]
        {
            return Err("Floor rectangle is empty or outside the scene".into());
        }
        for y in min[1]..max_exclusive[1] {
            for x in min[0]..max_exclusive[0] {
                let index = self.index([x, y]).ok_or("Unexpected room index")?;
                self.cells[index].floor = value;
            }
        }
        self.rebuild_walls();
        Ok(())
    }
    pub fn rebuild_walls(&mut self) {
        let [width, height] = self.dimensions();
        let old = self.cells.iter().map(|tile| tile.floor).collect::<Vec<_>>();
        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                if old[index] {
                    self.cells[index].wall_neighbor_mask = 0;
                    continue;
                }
                let mut mask = 0u8;
                for (dx, dy, bit) in [(0i32, -1i32, 1u8), (1, 0, 2), (0, 1, 4), (-1, 0, 8)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && nx < width as i32
                        && ny < height as i32
                        && old[(ny as u32 * width + nx as u32) as usize]
                    {
                        mask |= bit;
                    }
                }
                self.cells[index].wall_neighbor_mask = mask;
            }
        }
    }
    pub fn add_portal(&mut self, link: PortalLink) -> Result<(), String> {
        if link.id.trim().is_empty() || link.destination_scene.trim().is_empty() {
            return Err("Portal identity and destination must be specified".into());
        }
        let index = self.index(link.cell).ok_or("Portal is outside the scene")?;
        if !self.cells[index].floor {
            return Err("Portal must be placed on a walkable floor cell".into());
        }
        if self
            .portals
            .iter()
            .any(|other| other.id == link.id || other.cell == link.cell)
        {
            return Err("Portal identity or entrance position is already used".into());
        }
        self.portals.push(link);
        Ok(())
    }
    /// Carve a guaranteed connected cave floor from entrance to interior using a local seed.
    /// This is procedural *layout data*, not source-verified cliff or cave wall artwork.
    pub fn generate_cave(
        scene_id: impl Into<String>,
        chunks: [u8; 2],
        seed: u64,
        style: RoomStyle,
    ) -> Result<Self, String> {
        let mut scene = Self::new(scene_id, RoomKind::Cave, chunks, style)?;
        scene.generation_seed = seed;
        let [width, height] = scene.dimensions();
        let mut x = width / 2;
        let mut y = 1u32;
        let mut rng = seed ^ 0x9e37_79b9_7f4a_7c15;
        // Start with a guaranteed entrance-to-center passage.
        while y < height / 2 {
            scene.cells[(y * width + x) as usize].floor = true;
            y += 1;
        }
        let steps = (width * height * 2 / 3) as usize;
        for _ in 0..steps {
            scene.cells[(y * width + x) as usize].floor = true;
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            match rng % 4 {
                0 if x > 2 => x -= 1,
                1 if x + 3 < width => x += 1,
                2 if y > 2 => y -= 1,
                _ if y + 3 < height => y += 1,
                _ => {}
            }
        }
        // Preserve the authored entrance corridor after random walk.
        for entry_y in 1..=height / 2 {
            scene.cells[(entry_y * width + width / 2) as usize].floor = true;
        }
        scene.rebuild_walls();
        Ok(scene)
    }
    /// Load a hand-authored scene document from the project's own content tree.
    /// Invalid existing content fails visibly instead of silently becoming a test room.
    pub fn load_template(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path)
            .map_err(|e| format!("Cannot read room template {}: {e}", path.display()))?;
        let room: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Cannot parse room template {}: {e}", path.display()))?;
        room.validate()?;
        Ok(room)
    }

    /// Small hand-authored first-interior layout. The footprint is 2x2 chunks,
    /// although only its actual walkable floor is painted. The wall topology is
    /// calculated from the floor and must never be placed as free-floating art.
    pub fn starter_house(
        scene_id: impl Into<String>,
        return_world: [i64; 2],
    ) -> Result<Self, String> {
        let mut room = Self::new(
            scene_id,
            RoomKind::House,
            DEFAULT_ROOM_CHUNKS,
            RoomStyle::default(),
        )?;
        room.paint_floor_rect([16, 18], [48, 46], true)?;
        room.add_portal(PortalLink {
            id: "front_door_exit".into(),
            kind: PortalKind::Exit,
            cell: [32, 45],
            destination_scene: "world".into(),
            destination_cell: [0, 0],
            return_world: Some(return_world),
        })?;
        room.validate()?;
        Ok(room)
    }

    /// The procedural cave owns a walkable entrance at the upper middle of its
    /// local grid. Entrance and exit share the same validated portal contract.
    pub fn linked_cave(
        scene_id: impl Into<String>,
        seed: u64,
        return_world: [i64; 2],
    ) -> Result<Self, String> {
        let mut room =
            Self::generate_cave(scene_id, DEFAULT_ROOM_CHUNKS, seed, RoomStyle::default())?;
        room.add_portal(PortalLink {
            id: "cave_exit".into(),
            kind: PortalKind::Exit,
            cell: [room.dimensions()[0] / 2, 1],
            destination_scene: "world".into(),
            destination_cell: [0, 0],
            return_world: Some(return_world),
        })?;
        room.validate()?;
        Ok(room)
    }

    pub fn is_walkable(&self, cell: [i32; 2]) -> bool {
        if cell[0] < 0 || cell[1] < 0 {
            return false;
        }
        self.index([cell[0] as u32, cell[1] as u32])
            .is_some_and(|i| self.cells[i].floor)
    }

    pub fn exit_near(&self, at: [f32; 2]) -> Option<&PortalLink> {
        self.portals.iter().find(|p| {
            p.kind == PortalKind::Exit
                && (at[0] - (p.cell[0] as f32 + 0.5)).abs() < 1.5
                && (at[1] - (p.cell[1] as f32 + 0.5)).abs() < 1.5
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != "havenwild.bevy.room_scene.v1" || self.scene_id.trim().is_empty() {
            return Err("Invalid room scene identity or schema".into());
        }
        let dims = self.dimensions();
        if !self
            .chunks
            .iter()
            .all(|&n| (2..=MAX_ROOM_CHUNKS).contains(&n))
            || self.cells.len() != (dims[0] * dims[1]) as usize
        {
            return Err("Invalid room dimensions".into());
        }
        let mut expected = self.clone();
        expected.rebuild_walls();
        if expected.cells != self.cells {
            return Err("Room walls disagree with floor topology".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut occupied = std::collections::BTreeSet::new();
        for portal in &self.portals {
            if portal.id.trim().is_empty()
                || portal.destination_scene.trim().is_empty()
                || !ids.insert(&portal.id)
                || !occupied.insert(portal.cell)
            {
                return Err("Duplicate or invalid portal identity, destination, or cell".into());
            }
            let i = self
                .index(portal.cell)
                .ok_or("Portal is outside the room")?;
            if !self.cells[i].floor {
                return Err("Portal is not on floor".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn room_dimensions_are_bounded_to_two_or_three_chunks() {
        for size in [[2, 2], [3, 2], [3, 3]] {
            let r = RoomScene::new("room", RoomKind::House, size, RoomStyle::default()).unwrap();
            assert_eq!(
                r.dimensions(),
                [u32::from(size[0]) * 32, u32::from(size[1]) * 32]
            );
            assert!(r.validate().is_ok());
        }
        assert!(RoomScene::new("bad", RoomKind::House, [1, 2], RoomStyle::default()).is_err());
        assert!(RoomScene::new("bad", RoomKind::House, [4, 2], RoomStyle::default()).is_err());
    }
    #[test]
    fn resize_retains_floor_and_portals_and_recomputes_boundary_masks() {
        let mut room = RoomScene::starter_house("test", [0, 0]).unwrap();
        let previous_exit = room.portals[0].cell;
        room.resize_chunks([3, 3]).unwrap();
        assert_eq!(room.dimensions(), [96, 96]);
        assert_eq!(room.portals[0].cell, previous_exit);
        assert!(room.is_walkable([32, 25]));
        assert!(room.validate().is_ok());
        room.resize_chunks([2, 2]).unwrap();
        assert_eq!(room.dimensions(), [64, 64]);
        assert!(room.validate().is_ok());
        assert!(room.resize_chunks([4, 4]).is_err());
        room.resize_chunks([3, 3]).unwrap();
        room.paint_floor([80, 80], true).unwrap();
        room.add_portal(PortalLink {
            id: "bonus".into(),
            kind: PortalKind::HouseDoor,
            cell: [80, 80],
            destination_scene: "somewhere".into(),
            destination_cell: [4, 4],
            return_world: None,
        })
        .unwrap();
        assert!(room.resize_chunks([2, 2]).is_err());
        assert_eq!(room.dimensions(), [96, 96]);
    }
    #[test]
    fn painted_floor_automatically_builds_and_updates_wall_neighbors() {
        let mut room =
            RoomScene::new("house", RoomKind::House, [2, 2], RoomStyle::default()).unwrap();
        room.paint_floor([20, 20], true).unwrap();
        assert_eq!(
            room.cells[room.index([20, 19]).unwrap()].wall_neighbor_mask,
            4
        );
        assert_eq!(
            room.cells[room.index([21, 20]).unwrap()].wall_neighbor_mask,
            8
        );
        room.paint_floor([21, 20], true).unwrap();
        assert_eq!(
            room.cells[room.index([21, 20]).unwrap()].wall_neighbor_mask,
            0
        );
        assert_eq!(
            room.cells[room.index([22, 20]).unwrap()].wall_neighbor_mask,
            8
        );
        assert!(room.validate().is_ok());
    }
    #[test]
    fn portal_requires_floor_and_unique_destination_link() {
        let mut room =
            RoomScene::new("house", RoomKind::House, [2, 2], RoomStyle::default()).unwrap();
        let portal = PortalLink {
            id: "exit".into(),
            kind: PortalKind::Exit,
            cell: [3, 3],
            destination_scene: "world".into(),
            destination_cell: [10, 10],
            return_world: Some([10, 10]),
        };
        assert!(room.add_portal(portal.clone()).is_err());
        room.paint_floor([3, 3], true).unwrap();
        assert!(room.add_portal(portal.clone()).is_ok());
        assert!(room.add_portal(portal).is_err());
        assert!(room.validate().is_ok());
    }
    #[test]
    fn caves_are_deterministic_connected_and_saveable() {
        let a = RoomScene::generate_cave("cave", [2, 2], 12345, RoomStyle::default()).unwrap();
        let b = RoomScene::generate_cave("cave", [2, 2], 12345, RoomStyle::default()).unwrap();
        assert_eq!(a, b);
        assert!(a.cells.iter().any(|tile| tile.floor));
        assert!(a.validate().is_ok());
        let restored: RoomScene =
            serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(restored, a);
    }
    #[test]
    fn starter_house_has_walkable_exit_and_wall_topology() {
        let room = RoomScene::starter_house("house:test", [18, 33]).unwrap();
        assert_eq!(room.dimensions(), [64, 64]);
        assert!(room.is_walkable([32, 45]));
        assert!(!room.is_walkable([15, 45]));
        assert_eq!(
            room.exit_near([32.5, 45.5]).unwrap().return_world,
            Some([18, 33])
        );
        assert!(room.exit_near([24.5, 28.5]).is_none());
        assert!(room.validate().is_ok());
    }
    #[test]
    fn linked_cave_has_return_and_is_deterministic() {
        let a = RoomScene::linked_cave("cave:test", 876, [-4, 19]).unwrap();
        let b = RoomScene::linked_cave("cave:test", 876, [-4, 19]).unwrap();
        assert_eq!(a, b);
        assert!(a.is_walkable([32, 1]));
        assert_eq!(
            a.exit_near([32.5, 1.5]).unwrap().return_world,
            Some([-4, 19])
        );
        assert!(a.validate().is_ok());
    }
}

#[cfg(test)]
mod house_instance_path_tests {
    use super::*;
    #[test]
    fn different_houses_do_not_share_rooms_or_escape_content_root() {
        let first = house_instance_relative_path(101, "town:house:A");
        assert_eq!(first, house_instance_relative_path(101, "town:house:A"));
        assert_ne!(first, house_instance_relative_path(101, "town:house:B"));
        assert_ne!(first, house_instance_relative_path(102, "town:house:A"));
        assert_eq!(first.split('/').count(), 4);
        assert!(first.starts_with("content/rooms/houses/house_"));
        assert!(!first.contains(".."));
        assert!(!house_instance_relative_path(1, "../../secret").contains(".."));
    }
}
