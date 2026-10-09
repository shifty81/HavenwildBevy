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
    /// One-time migration from the old always-visible two-sidebar layout.
    #[serde(default)]
    pub canvas_first_v4_initialized: bool,
    /// M2D124: one-time migration from the accidental default-open empty slab.
    #[serde(default)]
    pub canvas_hud_v1_initialized: bool,
    /// M2D126: restore user-approved Tooling/Context workspace only once.
    #[serde(default)]
    pub authoring_recovery_v1_initialized: bool,
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
    #[serde(default = "default_world_layer_visibility")]
    pub world_layer_visibility: [bool; 7],
    #[serde(default)]
    pub metrics_drawer_open: bool,
    #[serde(default = "default_tool_context_open")]
    pub tool_context_open: bool,
    /// Unified authoring dock: independently resizable height allocation.
    #[serde(default = "default_authoring_top_height")]
    pub authoring_top_height: f32,
    #[serde(default = "default_authoring_middle_height")]
    pub authoring_middle_height: f32,
    #[serde(default = "default_authoring_context_width")]
    pub authoring_context_width: f32,
    #[serde(default = "default_authoring_library_tab")]
    pub authoring_library_tab: String,
    /// UA-03: one shared asset library, in the bottom dock or a floating window.
    #[serde(default = "default_true")]
    pub library_open: bool,
    #[serde(default)]
    pub library_floating: bool,
    #[serde(default = "default_library_height")]
    pub library_height: f32,
    #[serde(default = "default_library_view")]
    pub library_view: String,
    #[serde(default = "default_library_category")]
    pub library_category: String,
    #[serde(default = "default_true")]
    pub layers_dock_open: bool,
    #[serde(default = "default_pixel_palette_color")]
    pub pixel_palette_color: [u8; 4],
    #[serde(default = "default_brush_mode")]
    pub brush_mode: String,
    #[serde(default = "default_terrain_brush_role")]
    pub terrain_brush_role: String,
    #[serde(default = "default_brush_footprint")]
    pub terrain_brush_radius_tiles: u8,
    #[serde(default = "default_brush_footprint")]
    pub tile_brush_radius_tiles: u8,
    #[serde(default = "default_pixel_brush_size")]
    pub pixel_brush_size_px: u8,
    #[serde(default)]
    pub shoreline_audit_overlay: bool,
    #[serde(default)]
    pub terrain_mapper_compact: bool,
    #[serde(default)]
    pub structural_compact: bool,
    #[serde(default = "default_structural_dock")]
    pub structural_dock: SurfaceDock,
    #[serde(default)]
    pub world_generator_open: bool,
    #[serde(default)]
    pub world_generator_compact: bool,
    #[serde(default = "default_world_generator_dock")]
    pub world_generator_dock: SurfaceDock,
}

fn default_true() -> bool {
    true
}
fn default_library_height() -> f32 {
    236.0
}
fn default_library_view() -> String {
    "Grid".into()
}
fn default_library_category() -> String {
    "All".into()
}
fn default_tool_context_open() -> bool {
    true
}
fn default_authoring_top_height() -> f32 {
    145.0
}
fn default_authoring_middle_height() -> f32 {
    205.0
}
fn default_authoring_context_width() -> f32 {
    274.0
}
fn default_authoring_library_tab() -> String {
    "Terrain".into()
}
fn default_pixel_palette_color() -> [u8; 4] {
    [255, 255, 255, 255]
}
fn default_mapper_tab() -> String {
    "patterns".into()
}
fn default_brush_mode() -> String {
    "terrain".into()
}
fn default_terrain_brush_role() -> String {
    "Grass".into()
}
fn default_brush_footprint() -> u8 {
    1
}
fn default_pixel_brush_size() -> u8 {
    1
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
fn default_world_layer_visibility() -> [bool; 7] {
    [true; 7]
}
fn default_structural_dock() -> SurfaceDock {
    SurfaceDock::Right
}
fn default_world_generator_dock() -> SurfaceDock {
    SurfaceDock::Right
}

impl Default for EditorLayout {
    fn default() -> Self {
        let mut terrain_mapper = ModularSurfaceState::new(
            "havenwild.terrain_mapper",
            "Terrain Studio",
            SurfaceDock::Floating,
        );
        terrain_mapper.preferred_size = [430.0, 540.0];
        terrain_mapper.visible = false;
        Self {
            schema: "havenwild.bevy.editor_layout.local.v1".into(),
            terrain_mapper,
            terrain_mapper_tab: default_mapper_tab(),
            canvas_first_v4_initialized: true,
            canvas_hud_v1_initialized: true,
            authoring_recovery_v1_initialized: true,
            inspect_alpha: false,
            rulers_visible: true,
            mapping_status_visible: true,
            chunk_grid_visible: true,
            tile_grid_visible: false,
            chunk_size_tiles: 32,
            structural_open: false,
            structural_overlay: false,
            collision_overlay: false,
            world_layer_visibility: default_world_layer_visibility(),
            metrics_drawer_open: false,
            tool_context_open: true,
            authoring_top_height: default_authoring_top_height(),
            authoring_middle_height: default_authoring_middle_height(),
            authoring_context_width: default_authoring_context_width(),
            authoring_library_tab: default_authoring_library_tab(),
            library_open: true,
            library_floating: false,
            library_height: default_library_height(),
            library_view: default_library_view(),
            library_category: default_library_category(),
            layers_dock_open: true,
            pixel_palette_color: default_pixel_palette_color(),
            brush_mode: default_brush_mode(),
            terrain_brush_role: default_terrain_brush_role(),
            terrain_brush_radius_tiles: 1,
            tile_brush_radius_tiles: 1,
            pixel_brush_size_px: 1,
            shoreline_audit_overlay: false,
            terrain_mapper_compact: false,
            structural_compact: false,
            structural_dock: SurfaceDock::Right,
            world_generator_open: false,
            world_generator_compact: false,
            world_generator_dock: SurfaceDock::Right,
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
        if !matches!(layout.brush_mode.as_str(), "pixel" | "tile" | "terrain") {
            layout.brush_mode = default_brush_mode();
        }
        if !matches!(
            layout.terrain_brush_role.as_str(),
            "Grass" | "MudBank" | "Sand" | "WetSand" | "RiverWater" | "DeepWater" | "PebblePath"
        ) {
            layout.terrain_brush_role = default_terrain_brush_role();
        }
        layout.terrain_brush_radius_tiles = layout.terrain_brush_radius_tiles.clamp(1, 4);
        layout.tile_brush_radius_tiles = layout.tile_brush_radius_tiles.clamp(1, 4);
        layout.pixel_brush_size_px = layout.pixel_brush_size_px.clamp(1, 32);
        layout.authoring_top_height = if layout.authoring_top_height.is_finite() {
            layout.authoring_top_height.clamp(76.0, 600.0)
        } else {
            default_authoring_top_height()
        };
        layout.authoring_middle_height = if layout.authoring_middle_height.is_finite() {
            layout.authoring_middle_height.clamp(85.0, 600.0)
        } else {
            default_authoring_middle_height()
        };
        layout.library_height = if layout.library_height.is_finite() {
            layout.library_height.clamp(140.0, 700.0)
        } else {
            default_library_height()
        };
        if !matches!(layout.library_view.as_str(), "Grid" | "List" | "Atlas") {
            layout.library_view = default_library_view();
        }
        if !matches!(
            layout.library_category.as_str(),
            "All" | "Terrain" | "Objects" | "Structure" | "Vegetation" | "FX"
        ) {
            layout.library_category = default_library_category();
        }
        layout.authoring_context_width = if layout.authoring_context_width.is_finite() {
            layout.authoring_context_width.clamp(240.0, 352.0)
        } else {
            default_authoring_context_width()
        };
        if !matches!(
            layout.authoring_library_tab.as_str(),
            "Terrain" | "Tiles" | "Source" | "Assets"
        ) {
            layout.authoring_library_tab = default_authoring_library_tab();
        }

        if !crate::world_chunks::ALLOWED_CHUNK_TILES.contains(&layout.chunk_size_tiles) {
            layout.chunk_size_tiles = crate::world_chunks::DEFAULT_CHUNK_TILES;
        }
        // One-time normalization of old persistent layouts which force-opened a wide
        // left Scene panel and docked the advanced mapper on the right. Subsequent
        // user toggles and dock decisions persist; this is not a per-launch reset.
        if !layout.canvas_first_v4_initialized {
            layout.terrain_mapper.visible = false;
            layout.terrain_mapper.dock = SurfaceDock::Floating;
            layout.terrain_mapper_tab = "patterns".into();
            layout.canvas_first_v4_initialized = true;
            let _ = layout.save(path);
        }
        if !layout.canvas_hud_v1_initialized {
            // Restore canvas space once for legacy layouts. The user may reopen
            // settings thereafter; saved choices are not reset on each launch.
            layout.tool_context_open = false;
            layout.canvas_hud_v1_initialized = true;
            let _ = layout.save(path);
        }
        if !layout.authoring_recovery_v1_initialized {
            // Earlier GUI migrations collapsed the authoring surface and
            // persisted that wrong layout. Restore once, not on every boot.
            layout.tool_context_open = true;
            layout.library_open = true;
            layout.library_floating = false;
            layout.authoring_recovery_v1_initialized = true;
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
    fn library_and_canvas_layers_have_persistent_state() {
        let layout = EditorLayout::default();
        assert!(layout.library_open);
        assert!(!layout.library_floating);
        assert!(layout.layers_dock_open);
        assert_eq!(layout.library_view, "Grid");
        assert_eq!(layout.library_category, "All");
        assert!(layout.library_height >= 120.0);
    }

    #[test]
    fn fresh_studio_starts_with_unified_authoring_and_no_floating_studios() {
        let layout = EditorLayout::default();
        assert!(!layout.terrain_mapper.visible);
        assert_eq!(layout.terrain_mapper.dock, SurfaceDock::Floating);
        assert!(layout.canvas_first_v4_initialized);
        // Canvas-first: no unused left context slab at startup; active brush
        // options remain available on demand and are persisted locally.
        assert!(layout.tool_context_open);
        assert_eq!(layout.brush_mode, "terrain");
        assert_eq!(layout.terrain_brush_role, "Grass");
        assert_eq!(layout.terrain_brush_radius_tiles, 1);
        assert_eq!(layout.tile_brush_radius_tiles, 1);
        assert_eq!(layout.pixel_brush_size_px, 1);
        assert!(!layout.inspect_alpha);
        assert!(layout.rulers_visible);
        assert!(layout.mapping_status_visible);
        assert!(layout.chunk_grid_visible);
        assert!(!layout.tile_grid_visible);
        assert_eq!(layout.chunk_size_tiles, 32);
        assert!(!layout.structural_open);
        assert!(!layout.structural_overlay);
        assert!(!layout.collision_overlay);
        assert_eq!(layout.world_layer_visibility, [true; 7]);
        assert!(!layout.terrain_mapper_compact);
        assert!(!layout.structural_compact);
        assert_eq!(layout.structural_dock, SurfaceDock::Right);
        assert!(!layout.world_generator_open);
        assert_eq!(layout.world_generator_dock, SurfaceDock::Right);
    }

    #[test]
    fn unified_authoring_collapse_and_reopen_are_persistent() {
        let path = std::env::temp_dir().join(format!(
            "hw_unified_authoring_{}_layout.json",
            std::process::id()
        ));
        let mut layout = EditorLayout::default();
        assert!(layout.tool_context_open);
        layout.tool_context_open = false;
        layout.save(&path).unwrap();
        let mut restored = EditorLayout::load(&path);
        assert!(!restored.tool_context_open);
        restored.tool_context_open = true;
        restored.save(&path).unwrap();
        assert!(EditorLayout::load(&path).tool_context_open);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn legacy_wide_tooling_migrates_once_and_keeps_user_reopen() {
        let path =
            std::env::temp_dir().join(format!("hw_gui124_{}_layout.json", std::process::id()));
        let mut json = serde_json::to_value(EditorLayout::default()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.remove("canvasHudV1Initialized");
        object.insert("toolContextOpen".into(), serde_json::json!(true));
        fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
        let mut migrated = EditorLayout::load(&path);
        assert!(migrated.canvas_hud_v1_initialized);
        assert!(!migrated.tool_context_open);
        migrated.tool_context_open = true;
        migrated.save(&path).unwrap();
        assert!(EditorLayout::load(&path).tool_context_open);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn recovered_authoring_workspace_migrates_once_and_preserves_user_collapse() {
        let path =
            std::env::temp_dir().join(format!("hw_gui126_{}_layout.json", std::process::id()));
        let mut old = serde_json::to_value(EditorLayout::default()).unwrap();
        let fields = old.as_object_mut().unwrap();
        fields.remove("authoringRecoveryV1Initialized");
        fields.insert("toolContextOpen".into(), serde_json::json!(false));
        fields.insert("libraryOpen".into(), serde_json::json!(false));
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let mut recovered = EditorLayout::load(&path);
        assert!(recovered.authoring_recovery_v1_initialized);
        assert!(recovered.tool_context_open);
        assert!(recovered.library_open);
        recovered.tool_context_open = false;
        recovered.library_height = 390.0;
        recovered.save(&path).unwrap();
        let loaded = EditorLayout::load(&path);
        assert!(!loaded.tool_context_open);
        assert_eq!(loaded.library_height, 390.0);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn previous_layout_json_ignores_retired_duplicate_surfaces() {
        let mut json = serde_json::to_value(EditorLayout::default()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.insert("layersOpen".into(), serde_json::json!(true));
        object.insert("sceneLayersCompact".into(), serde_json::json!(true));
        object.insert("sceneLayersDock".into(), serde_json::json!("right"));
        object.insert("assetAuthorityOpen".into(), serde_json::json!(true));
        object.insert("assetAuthorityCompact".into(), serde_json::json!(true));
        object.insert("assetAuthorityDock".into(), serde_json::json!("right"));
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
        object.remove("worldLayerVisibility");
        object.remove("terrainMapperCompact");
        object.remove("structuralCompact");
        object.remove("structuralDock");
        object.remove("worldGeneratorOpen");
        object.remove("worldGeneratorCompact");
        object.remove("worldGeneratorDock");
        let legacy: EditorLayout = serde_json::from_value(json).unwrap();
        assert!(!legacy.canvas_first_v4_initialized);
        assert!(!legacy.inspect_alpha);
        assert!(legacy.rulers_visible);
        assert!(legacy.mapping_status_visible);
        assert!(legacy.chunk_grid_visible);
        assert!(!legacy.tile_grid_visible);
        assert_eq!(legacy.chunk_size_tiles, 32);
        assert_eq!(legacy.world_layer_visibility, [true; 7]);
        assert!(!legacy.terrain_mapper_compact);
        assert!(!legacy.structural_compact);
        assert_eq!(legacy.structural_dock, SurfaceDock::Right);
        assert!(!legacy.world_generator_open);
    }

    #[test]
    fn old_workspace_migrates_once_and_respects_future_tool_surfaces() {
        let unique = std::env::temp_dir().join(format!(
            "hw_canvas_first_{}_layout.json",
            std::process::id()
        ));
        let mut old = EditorLayout::default();
        old.canvas_first_v4_initialized = false;
        old.terrain_mapper.visible = true;
        old.terrain_mapper.dock = SurfaceDock::Right;
        old.save(&unique).unwrap();
        let mut normalized = EditorLayout::load(&unique);
        assert!(!normalized.terrain_mapper.visible);
        assert_eq!(normalized.terrain_mapper.dock, SurfaceDock::Floating);
        assert_eq!(normalized.terrain_mapper_tab, "patterns");
        normalized.terrain_mapper.visible = true;
        normalized.terrain_mapper.dock = SurfaceDock::Bottom;
        normalized.save(&unique).unwrap();
        let restored = EditorLayout::load(&unique);
        assert!(restored.terrain_mapper.visible);
        assert_eq!(restored.terrain_mapper.dock, SurfaceDock::Bottom);
        let _ = fs::remove_file(unique);
    }
}
