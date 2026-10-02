//! Local editor layout persistence. Layout state is workstation-local and lives under .forgepy/.

use crate::atomic_file::write_atomic;
use forge_gui_chrome::{ModularSurfaceState, SurfaceDock};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLayout {
    pub schema: String,
    pub terrain_mapper: ModularSurfaceState,
    #[serde(default = "default_mapper_tab")]
    pub terrain_mapper_tab: String,
    #[serde(default = "default_layers_open")]
    pub layers_open: bool,
    #[serde(default)]
    pub scene_panel_v3_initialized: bool,
    /// One-time migration from the old always-visible two-sidebar layout.
    #[serde(default)]
    pub canvas_first_v4_initialized: bool,
    #[serde(default)]
    pub inspect_alpha: bool,
    #[serde(default = "default_rulers_visible")]
    pub rulers_visible: bool,
    #[serde(default = "default_mapping_status_visible")]
    pub mapping_status_visible: bool,
    #[serde(default = "default_chunk_grid_visible")]
    pub chunk_grid_visible: bool,
    #[serde(default)]
    pub tile_grid_visible: bool,
    #[serde(default = "default_chunk_size")]
    pub chunk_size_tiles: usize,
    #[serde(default)]
    pub structural_open: bool,
    #[serde(default)]
    pub structural_overlay: bool,
    #[serde(default)]
    pub collision_overlay: bool,
    #[serde(default)]
    pub terrain_mapper_compact: bool,
    #[serde(default)]
    pub scene_layers_compact: bool,
    #[serde(default)]
    pub structural_compact: bool,
    #[serde(default = "default_scene_layers_dock")]
    pub scene_layers_dock: SurfaceDock,
    #[serde(default = "default_structural_dock")]
    pub structural_dock: SurfaceDock,
    #[serde(default)]
    pub world_generator_open: bool,
    #[serde(default)]
    pub world_generator_compact: bool,
    #[serde(default = "default_world_generator_dock")]
    pub world_generator_dock: SurfaceDock,
    #[serde(default)]
    pub chunk_manager_open: bool,
    #[serde(default)]
    pub chunk_manager_compact: bool,
    #[serde(default = "default_chunk_manager_dock")]
    pub chunk_manager_dock: SurfaceDock,
    #[serde(default)]
    pub asset_authority_open: bool,
    #[serde(default)]
    pub asset_authority_compact: bool,
    #[serde(default = "default_asset_authority_dock")]
    pub asset_authority_dock: SurfaceDock,
}

fn default_mapper_tab() -> String {
    "source".into()
}
fn default_layers_open() -> bool {
    false
}

fn default_rulers_visible() -> bool {
    true
}
fn default_mapping_status_visible() -> bool {
    true
}

fn default_chunk_grid_visible() -> bool {
    true
}
fn default_chunk_size() -> usize {
    32
}
fn default_scene_layers_dock() -> SurfaceDock {
    SurfaceDock::Right
}
fn default_structural_dock() -> SurfaceDock {
    SurfaceDock::Right
}
fn default_world_generator_dock() -> SurfaceDock {
    SurfaceDock::Right
}
fn default_chunk_manager_dock() -> SurfaceDock {
    SurfaceDock::Right
}
fn default_asset_authority_dock() -> SurfaceDock {
    SurfaceDock::Right
}

impl Default for EditorLayout {
    fn default() -> Self {
        let mut terrain_mapper = ModularSurfaceState::new(
            "havenwild.terrain_mapper",
            "Terrain Mapper",
            SurfaceDock::Floating,
        );
        terrain_mapper.preferred_size = [430.0, 540.0];
        terrain_mapper.visible = false;
        Self {
            schema: "havenwild.bevy.editor_layout.local.v1".into(),
            terrain_mapper,
            terrain_mapper_tab: default_mapper_tab(),
            layers_open: false,
            scene_panel_v3_initialized: true,
            canvas_first_v4_initialized: true,
            inspect_alpha: false,
            rulers_visible: true,
            mapping_status_visible: true,
            chunk_grid_visible: true,
            tile_grid_visible: false,
            chunk_size_tiles: 32,
            structural_open: false,
            structural_overlay: false,
            collision_overlay: false,
            terrain_mapper_compact: false,
            scene_layers_compact: false,
            structural_compact: false,
            scene_layers_dock: SurfaceDock::Right,
            structural_dock: SurfaceDock::Right,
            world_generator_open: false,
            world_generator_compact: false,
            world_generator_dock: SurfaceDock::Right,
            chunk_manager_open: false,
            chunk_manager_compact: false,
            chunk_manager_dock: SurfaceDock::Right,
            asset_authority_open: false,
            asset_authority_compact: false,
            asset_authority_dock: SurfaceDock::Right,
        }
    }
}

impl EditorLayout {
    pub fn load(path: &Path) -> Self {
        let mut layout = fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<Self>(&text).ok())
            .filter(|layout| layout.schema == "havenwild.bevy.editor_layout.local.v1")
            .unwrap_or_default();
        if !crate::world_chunks::ALLOWED_CHUNK_TILES.contains(&layout.chunk_size_tiles) {
            layout.chunk_size_tiles = crate::world_chunks::DEFAULT_CHUNK_TILES;
        }
        // One-time normalization of old persistent layouts which force-opened a wide
        // left Scene panel and docked the advanced mapper on the right. Subsequent
        // user toggles and dock decisions persist; this is not a per-launch reset.
        if !layout.canvas_first_v4_initialized {
            layout.layers_open = false;
            layout.terrain_mapper.visible = false;
            layout.terrain_mapper.dock = SurfaceDock::Floating;
            layout.terrain_mapper_tab = "source".into();
            layout.scene_panel_v3_initialized = true;
            layout.canvas_first_v4_initialized = true;
            let _ = layout.save(path);
        }
        layout
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())? + "\n";
        write_atomic(path, text.as_bytes())
    }
}

#[cfg(test)]
mod scene_first_layout_tests {
    use super::*;

    #[test]
    fn fresh_studio_starts_unobstructed() {
        let layout = EditorLayout::default();
        assert!(!layout.terrain_mapper.visible);
        assert_eq!(layout.terrain_mapper.dock, SurfaceDock::Floating);
        assert!(!layout.layers_open);
        assert!(layout.scene_panel_v3_initialized);
        assert!(layout.canvas_first_v4_initialized);
        assert!(!layout.inspect_alpha);
        assert!(layout.rulers_visible);
        assert!(layout.mapping_status_visible);
        assert!(layout.chunk_grid_visible);
        assert!(!layout.tile_grid_visible);
        assert_eq!(layout.chunk_size_tiles, 32);
        assert!(!layout.structural_open);
        assert!(!layout.structural_overlay);
        assert!(!layout.collision_overlay);
        assert!(!layout.terrain_mapper_compact);
        assert!(!layout.scene_layers_compact);
        assert!(!layout.structural_compact);
        assert_eq!(layout.scene_layers_dock, SurfaceDock::Right);
        assert_eq!(layout.structural_dock, SurfaceDock::Right);
        assert!(!layout.world_generator_open);
        assert!(!layout.chunk_manager_open);
        assert!(!layout.asset_authority_open);
        assert_eq!(layout.world_generator_dock, SurfaceDock::Right);
        assert_eq!(layout.chunk_manager_dock, SurfaceDock::Right);
        assert_eq!(layout.asset_authority_dock, SurfaceDock::Right);
    }

    #[test]
    fn previous_layout_json_loads_new_optional_fields_safely() {
        let mut json = serde_json::to_value(EditorLayout::default()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.remove("layersOpen");
        object.remove("scenePanelV3Initialized");
        object.remove("canvasFirstV4Initialized");
        object.remove("inspectAlpha");
        object.remove("rulersVisible");
        object.remove("mappingStatusVisible");
        object.remove("chunkGridVisible");
        object.remove("tileGridVisible");
        object.remove("chunkSizeTiles");
        object.remove("structuralOpen");
        object.remove("structuralOverlay");
        object.remove("collisionOverlay");
        object.remove("terrainMapperCompact");
        object.remove("sceneLayersCompact");
        object.remove("structuralCompact");
        object.remove("sceneLayersDock");
        object.remove("structuralDock");
        object.remove("worldGeneratorOpen");
        object.remove("worldGeneratorCompact");
        object.remove("worldGeneratorDock");
        object.remove("chunkManagerOpen");
        object.remove("chunkManagerCompact");
        object.remove("chunkManagerDock");
        object.remove("assetAuthorityOpen");
        object.remove("assetAuthorityCompact");
        object.remove("assetAuthorityDock");
        let legacy: EditorLayout = serde_json::from_value(json).unwrap();
        assert!(!legacy.layers_open);
        assert!(!legacy.scene_panel_v3_initialized);
        assert!(!legacy.canvas_first_v4_initialized);
        assert!(!legacy.inspect_alpha);
        assert!(legacy.rulers_visible);
        assert!(legacy.mapping_status_visible);
        assert!(legacy.chunk_grid_visible);
        assert!(!legacy.tile_grid_visible);
        assert_eq!(legacy.chunk_size_tiles, 32);
        assert!(!legacy.terrain_mapper_compact);
        assert!(!legacy.scene_layers_compact);
        assert!(!legacy.structural_compact);
        assert_eq!(legacy.scene_layers_dock, SurfaceDock::Right);
        assert_eq!(legacy.structural_dock, SurfaceDock::Right);
        assert!(!legacy.world_generator_open);
        assert!(!legacy.chunk_manager_open);
        assert!(!legacy.asset_authority_open);
    }
    #[test]
    fn old_workspace_migrates_once_and_respects_future_panel_choices() {
        let unique = std::env::temp_dir().join(format!(
            "hw_canvas_first_{}_layout.json",
            std::process::id()
        ));
        let mut old = EditorLayout::default();
        old.canvas_first_v4_initialized = false;
        old.layers_open = true;
        old.terrain_mapper.visible = true;
        old.terrain_mapper.dock = SurfaceDock::Right;
        old.save(&unique).unwrap();
        let mut normalized = EditorLayout::load(&unique);
        assert!(!normalized.layers_open);
        assert!(!normalized.terrain_mapper.visible);
        assert_eq!(normalized.terrain_mapper.dock, SurfaceDock::Floating);
        assert_eq!(normalized.terrain_mapper_tab, "source");
        normalized.layers_open = true;
        normalized.terrain_mapper.visible = true;
        normalized.terrain_mapper.dock = SurfaceDock::Bottom;
        normalized.save(&unique).unwrap();
        let restored = EditorLayout::load(&unique);
        assert!(restored.layers_open);
        assert!(restored.terrain_mapper.visible);
        assert_eq!(restored.terrain_mapper.dock, SurfaceDock::Bottom);
        let _ = fs::remove_file(unique);
    }
}
