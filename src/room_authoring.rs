//! Transactional authoring of bounded room scenes. Edits modify only the
//! project-owned room document; source images and overworld files are untouched.
use crate::room_scene::{PortalKind, RoomScene};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomTool {
    Floor,
    EraseFloor,
    MoveExit,
}

pub struct RoomEditor {
    pub scene: RoomScene,
    pub tool: RoomTool,
    pub dirty: bool,
    pub undo: Vec<RoomScene>,
    pub redo: Vec<RoomScene>,
    pub last_cell: Option<[u32; 2]>,
    stroke_active: bool,
    pub zoom: f32,
    pub pan: [f32; 2],
}
impl RoomEditor {
    pub fn open(scene: RoomScene) -> Result<Self, String> {
        scene.validate()?;
        Ok(Self {
            scene,
            tool: RoomTool::Floor,
            dirty: false,
            undo: Vec::new(),
            redo: Vec::new(),
            last_cell: None,
            stroke_active: false,
            zoom: 0.37,
            pan: [0.0, 0.0],
        })
    }
    pub fn stroke_begin(&mut self) {
        self.last_cell = None;
        self.stroke_active = false;
    }
    /// Returns true only if a document edit happened; same-cell drag samples are cheap.
    pub fn paint(&mut self, cell: [u32; 2]) -> Result<bool, String> {
        if self.last_cell == Some(cell) {
            return Ok(false);
        }
        let idx = self.scene.index(cell).ok_or("Room brush outside scene")?;
        let floor = self.scene.cells[idx].floor;
        match self.tool {
            RoomTool::Floor if floor => return Ok(false),
            RoomTool::EraseFloor if !floor => return Ok(false),
            RoomTool::EraseFloor if self.scene.portals.iter().any(|p| p.cell == cell) => {
                return Err("Cannot erase a portal's floor. Move the exit first.".into());
            }
            RoomTool::MoveExit if !floor => return Err("Move the exit onto a floor tile".into()),
            RoomTool::MoveExit
                if self
                    .scene
                    .portals
                    .iter()
                    .any(|p| p.kind == PortalKind::Exit && p.cell == cell) =>
            {
                return Ok(false)
            }
            _ => {}
        }
        let previous = self.scene.clone();
        match self.tool {
            RoomTool::Floor => self.scene.paint_floor(cell, true)?,
            RoomTool::EraseFloor => self.scene.paint_floor(cell, false)?,
            RoomTool::MoveExit => {
                if self
                    .scene
                    .portals
                    .iter()
                    .any(|p| p.kind != PortalKind::Exit && p.cell == cell)
                {
                    return Err("A different portal occupies this floor".into());
                }
                let exit = self
                    .scene
                    .portals
                    .iter_mut()
                    .find(|p| p.kind == PortalKind::Exit)
                    .ok_or("Room has no return portal")?;
                exit.cell = cell;
            }
        }
        if let Err(error) = self.scene.validate() {
            self.scene = previous;
            return Err(error);
        }
        // A continuous mouse stroke is one undo transaction, not one per pixel.
        if !self.stroke_active {
            self.undo.push(previous);
            if self.undo.len() > 64 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
        self.stroke_active = true;
        self.last_cell = Some(cell);
        self.dirty = true;
        Ok(true)
    }
    pub fn resize_chunks(&mut self, chunks: [u8; 2]) -> Result<bool, String> {
        if self.scene.chunks == chunks {
            return Ok(false);
        }
        let previous = self.scene.clone();
        self.scene.resize_chunks(chunks)?;
        self.undo.push(previous);
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.stroke_begin();
        self.dirty = true;
        Ok(true)
    }
    pub fn set_wall_tint(&mut self, rgba: [u8; 4]) -> Result<bool, String> {
        if self.scene.style.wall_tint_rgba == rgba {
            return Ok(false);
        }
        let previous = self.scene.clone();
        self.scene.style.wall_tint_rgba = rgba;
        self.scene.validate()?;
        self.undo.push(previous);
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
        Ok(true)
    }
    pub fn undo(&mut self) -> bool {
        let Some(prior) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.scene, prior));
        self.dirty = true;
        self.stroke_begin();
        true
    }
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.scene, next));
        self.dirty = true;
        self.stroke_begin();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room_scene::RoomScene;
    #[test]
    fn painted_house_floor_auto_updates_walls_and_undoes_without_source_art() {
        let room = RoomScene::starter_house("test", [0, 0]).unwrap();
        let mut edit = RoomEditor::open(room).unwrap();
        assert!(!edit.scene.is_walkable([10, 10]));
        assert!(edit.paint([10, 10]).unwrap());
        assert!(edit.scene.is_walkable([10, 10]));
        assert_eq!(
            edit.scene.cells[edit.scene.index([10, 9]).unwrap()].wall_neighbor_mask,
            4
        );
        assert!(edit.undo());
        assert!(!edit.scene.is_walkable([10, 10]));
        assert!(edit.redo());
        assert!(edit.scene.validate().is_ok());
    }
    #[test]
    fn whole_brush_stroke_is_one_undo_step() {
        let mut edit =
            RoomEditor::open(RoomScene::starter_house("house", [0, 0]).unwrap()).unwrap();
        edit.paint([10, 10]).unwrap();
        edit.paint([11, 10]).unwrap();
        assert_eq!(edit.undo.len(), 1);
        edit.stroke_begin();
        edit.paint([12, 10]).unwrap();
        assert_eq!(edit.undo.len(), 2);
        assert!(edit.undo());
        assert!(!edit.scene.is_walkable([12, 10]));
        assert!(edit.scene.is_walkable([11, 10]));
        assert!(edit.undo());
        assert!(!edit.scene.is_walkable([10, 10]));
        assert!(edit.scene.validate().is_ok());
    }
    #[test]
    fn exit_floor_cannot_be_erased_and_exit_can_move_to_valid_floor() {
        let mut edit = RoomEditor::open(RoomScene::starter_house("test", [0, 0]).unwrap()).unwrap();
        edit.tool = RoomTool::EraseFloor;
        assert!(edit.paint([32, 45]).is_err());
        assert!(edit.scene.validate().is_ok());
        edit.tool = RoomTool::MoveExit;
        edit.stroke_begin();
        assert!(edit.paint([31, 45]).unwrap());
        assert!(edit.scene.exit_near([31.5, 45.5]).is_some());
        edit.stroke_begin();
        edit.tool = RoomTool::EraseFloor;
        assert!(edit.paint([32, 45]).unwrap());
        assert!(edit.scene.validate().is_ok());
    }
}
