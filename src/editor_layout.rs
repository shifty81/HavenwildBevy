//! Local editor layout persistence. Layout state is workstation-local and lives under .forgepy/.

use crate::atomic_file::write_atomic;
use crate::workspace_layout::WorkspaceLayout;
use forge_gui_chrome::{ModularSurfaceState, SurfaceDock};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLayout {
    pub schema: String,
    /// M2D131: editor-only docking/layout authority, never project/world data.
    #[serde(default)]
    pub workspace: WorkspaceLayout,
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
    /// M2D127: restore the integrated three-section authoring workspace once.
    #[serde(default)]
    pub authoring_three_section_v1_initialized: bool,
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
    /// Workstation/editor theme, never part of a game/world document.
    #[serde(default = "default_editor_theme")]
    pub editor_theme: String,
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
    /// M2D131B3: once-only move from the separate icon rail + context pane.
    #[serde(default)]
    pub left_dock_v2_initialized: bool,
    #[serde(default = "default_authoring_library_tab")]
    pub authoring_library_tab: String,
    /// UA-03: one shared asset library, in the bottom dock or a floating window.
    #[serde(default = "default_true")]
    pub library_open: bool,
    #[serde(default)]
    pub library_floating: bool,
    /// One Library authority; dock it in the left workspace or the bottom tray.
    #[serde(default = "default_true")]
    pub library_in_left_workspace: bool,
    /// Explicit floating Library resize geometry, persisted independently of dock height.
    #[serde(default = "default_library_floating_size")]
    pub library_floating_size: [f32; 2],
    #[serde(default = "default_library_height")]
    pub library_height: f32,
    #[serde(default = "default_library_view")]
    pub library_view: String,
    #[serde(default = "default_library_category")]
    pub library_category: String,
    /// M2D131B: migrate legacy Library placement once to the canonical right 50/50 dock.
    #[serde(default)]
    pub canvas_overlay_v1_initialized: bool,
    /// Fraction of usable right dock height reserved for Inspector (remainder Assets).
    #[serde(default = "default_inspector_tab")]
    pub inspector_tab: String,
    #[serde(default = "default_inspector_split_ratio")]
    pub inspector_split_ratio: f32,
    /// M2D128: new unified studio composition runs its migration exactly once.
    #[serde(default)]
    pub studio_convergence_v1_initialized: bool,
    /// One source browser authority, now defaulting to the lower Inspector.
    #[serde(default)]
    pub library_in_inspector: bool,
    #[serde(default = "default_inspector_library_height")]
    pub inspector_library_height: f32,
    /// Always present in editor chrome, collapsed to transport height when closed.
    #[serde(default = "default_true")]
    pub timeline_open: bool,
    #[serde(default = "default_timeline_height")]
    pub timeline_height: f32,
    #[serde(default = "default_true")]
    pub timeline_playing: bool,
    #[serde(default = "default_scene_nav_height")]
    pub scene_nav_height: f32,
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

fn default_editor_theme() -> String {
    "North Star".to_owned()
}
fn default_true() -> bool {
    true
}
fn default_library_height() -> f32 {
    236.0
}
fn default_inspector_library_height() -> f32 {
    240.0
}
fn default_inspector_tab() -> String {
    "Inspect".into()
}
fn default_inspector_split_ratio() -> f32 {
    0.5
}
fn default_timeline_height() -> f32 {
    132.0
}
fn default_scene_nav_height() -> f32 {
    220.0
}
fn default_library_floating_size() -> [f32; 2] {
    [740.0, 480.0]
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
    252.0
}
fn default_authoring_middle_height() -> f32 {
    215.0
}
fn default_authoring_context_width() -> f32 {
    288.0
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
            workspace: WorkspaceLayout::default(),
            terrain_mapper,
            terrain_mapper_tab: default_mapper_tab(),
            canvas_first_v4_initialized: true,
            canvas_hud_v1_initialized: true,
            authoring_recovery_v1_initialized: true,
            authoring_three_section_v1_initialized: true,
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
            editor_theme: default_editor_theme(),
            metrics_drawer_open: false,
            tool_context_open: true,
            authoring_top_height: default_authoring_top_height(),
            authoring_middle_height: default_authoring_middle_height(),
            authoring_context_width: default_authoring_context_width(),
            left_dock_v2_initialized: true,
            authoring_library_tab: default_authoring_library_tab(),
            library_open: true,
            library_floating: false,
            library_in_left_workspace: false,
            library_floating_size: default_library_floating_size(),
            library_height: default_library_height(),
            library_view: default_library_view(),
            library_category: default_library_category(),
            studio_convergence_v1_initialized: true,
            canvas_overlay_v1_initialized: true,
            inspector_tab: default_inspector_tab(),
            inspector_split_ratio: default_inspector_split_ratio(),
            library_in_inspector: true,
            inspector_library_height: default_inspector_library_height(),
            timeline_open: false,
            timeline_height: default_timeline_height(),
            timeline_playing: true,
            scene_nav_height: default_scene_nav_height(),
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
        layout.workspace = layout.workspace.restored_or_default();
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
        if !matches!(
            layout.inspector_tab.as_str(),
            "Inspect" | "World" | "Terrain"
        ) {
            layout.inspector_tab = default_inspector_tab();
        }
        // Legacy window-open state migrates into the Inspector once.
        if layout.structural_open && layout.inspector_tab == "Inspect" {
            layout.inspector_tab = "World".into();
        }
        layout.structural_open = layout.inspector_tab == "World";
        layout.inspector_split_ratio = if layout.inspector_split_ratio.is_finite() {
            layout.inspector_split_ratio.clamp(0.20, 0.80)
        } else {
            default_inspector_split_ratio()
        };
        layout.inspector_library_height = if layout.inspector_library_height.is_finite() {
            layout.inspector_library_height.clamp(145.0, 650.0)
        } else {
            default_inspector_library_height()
        };
        layout.timeline_height = if layout.timeline_height.is_finite() {
            layout.timeline_height.clamp(100.0, 400.0)
        } else {
            default_timeline_height()
        };
        layout.scene_nav_height = if layout.scene_nav_height.is_finite() {
            layout.scene_nav_height.clamp(120.0, 600.0)
        } else {
            default_scene_nav_height()
        };
        layout.library_height = if layout.library_height.is_finite() {
            layout.library_height.clamp(140.0, 700.0)
        } else {
            default_library_height()
        };
        let dimensions = &mut layout.library_floating_size;
        for (i, lo, hi) in [(0, 420.0, 1600.0), (1, 260.0, 1080.0)] {
            let value = dimensions[i];
            dimensions[i] = if value.is_finite() {
                value.clamp(lo, hi)
            } else {
                default_library_floating_size()[i]
            };
        }
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
            layout.authoring_context_width.clamp(210.0, 490.0)
        } else {
            default_authoring_context_width()
        };
        if !matches!(
            layout.authoring_library_tab.as_str(),
            "Terrain" | "Tiles" | "Source" | "Assets" | "Rooms"
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
        if !layout.authoring_three_section_v1_initialized {
            // Restore the approved integrated left Tooling/Context/Library once.
            // Never silently yank an already-undocked user Library back to the left.
            layout.tool_context_open = true;
            // Newer Studio revisions own the Inspector Library default;
            // don't reactivate the retired left Library when old test fields are missing.
            if !layout.studio_convergence_v1_initialized {
                layout.library_in_left_workspace = true;
            }
            layout.library_open = true;
            layout.authoring_three_section_v1_initialized = true;
            let _ = layout.save(path);
        }
        if !layout.studio_convergence_v1_initialized {
            // Preserve deliberate floating Library placement across migration.
            let was_floating = layout.library_floating;
            layout.library_in_inspector = !was_floating;
            layout.library_in_left_workspace = false;
            layout.library_floating = was_floating;
            layout.library_open = true;
            layout.tool_context_open = true;
            // On first convergence, keep the timeline collapsed in the canvas-first
            // workspace. Already-migrated layouts retain the user's saved choice.
            layout.timeline_open = false;
            // One-time allocation for the new wheel + palette rather than carrying
            // the previous cramped Tooling/Context splitter heights forward.
            layout.authoring_top_height = default_authoring_top_height();
            layout.authoring_middle_height = default_authoring_middle_height();
            layout.studio_convergence_v1_initialized = true;
            let _ = layout.save(path);
        }
        if !layout.left_dock_v2_initialized {
            // A legacy untouched 350px context pane PLUS a separate 62px rail
            // becomes one ~288px dock at 1920 desktop width. Preserve custom
            // widths chosen by the user instead of resetting every launch.
            if (layout.authoring_context_width - 350.0).abs() < 0.5 {
                layout.authoring_context_width = default_authoring_context_width();
            }
            layout.left_dock_v2_initialized = true;
            let _ = layout.save(path);
        }
        if !layout.canvas_overlay_v1_initialized {
            // Migrate the old hidden/left/bottom Library into the new permanent
            // Inspector/Assets right split, except explicitly floating windows.
            if !layout.library_floating {
                layout.library_in_inspector = true;
                layout.library_in_left_workspace = false;
                layout.library_open = true;
            }
            layout.inspector_split_ratio = default_inspector_split_ratio();
            layout.canvas_overlay_v1_initialized = true;
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
    fn three_section_workspace_and_floating_size_persist() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d127_layout_{}.json",
            std::process::id()
        ));
        let mut layout = EditorLayout::default();
        assert!(layout.library_in_inspector);
        assert!(layout.tool_context_open);
        assert!(layout.authoring_three_section_v1_initialized);
        layout.library_in_left_workspace = true;
        layout.library_in_inspector = false;
        layout.authoring_top_height = 210.0;
        layout.authoring_middle_height = 260.0;
        layout.library_floating_size = [900.0, 610.0];
        layout.library_floating = true;
        layout.save(&path).unwrap();
        let restored = EditorLayout::load(&path);
        assert_eq!(restored.authoring_top_height, 210.0);
        assert_eq!(restored.authoring_middle_height, 260.0);
        assert_eq!(restored.library_floating_size, [900.0, 610.0]);
        assert!(restored.library_floating);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn three_section_migration_runs_once_and_preserves_user_dock_choice() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d127_migration_{}.json",
            std::process::id()
        ));
        let mut old = serde_json::to_value(EditorLayout::default()).unwrap();
        let fields = old.as_object_mut().unwrap();
        fields.remove("authoringThreeSectionV1Initialized");
        fields.remove("libraryInLeftWorkspace");
        fields.remove("libraryFloatingSize");
        fields.insert("toolContextOpen".into(), serde_json::json!(false));
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let migrated = EditorLayout::load(&path);
        assert!(migrated.authoring_three_section_v1_initialized);
        assert!(migrated.tool_context_open);
        assert!(migrated.library_in_inspector);
        let mut updated = migrated;
        updated.library_in_left_workspace = false;
        updated.library_in_inspector = true;
        updated.library_floating_size = [865.0, 590.0];
        updated.save(&path).unwrap();
        let restored = EditorLayout::load(&path);
        assert!(!restored.library_in_left_workspace);
        assert_eq!(restored.library_floating_size, [865.0, 590.0]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn studio_convergence_layout_migrates_once_and_persists_resize() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d128_layout_{}.json",
            std::process::id()
        ));
        let mut old = serde_json::to_value(EditorLayout::default()).unwrap();
        let fields = old.as_object_mut().unwrap();
        fields.remove("studioConvergenceV1Initialized");
        fields.remove("libraryInInspector");
        fields.remove("timelineHeight");
        fields.insert("libraryInLeftWorkspace".into(), serde_json::json!(true));
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let migrated = EditorLayout::load(&path);
        assert!(migrated.studio_convergence_v1_initialized);
        assert!(migrated.library_in_inspector);
        assert!(!migrated.library_in_left_workspace);
        assert!(!migrated.timeline_open); // M2D131: canvas-toggle default is collapsed.
        let mut chosen = migrated;
        chosen.timeline_height = 215.0;
        chosen.timeline_open = false;
        chosen.library_in_inspector = false;
        chosen.library_floating = true;
        chosen.save(&path).unwrap();
        let loaded = EditorLayout::load(&path);
        assert_eq!(loaded.timeline_height, 215.0);
        assert!(!loaded.timeline_open);
        assert!(!loaded.library_in_inspector);
        assert!(loaded.library_floating);
        // M2D131: reopening Animation explicitly is a persistent user choice.
        let mut reopened = loaded;
        reopened.timeline_open = true;
        reopened.save(&path).unwrap();
        let retained = EditorLayout::load(&path);
        assert!(retained.timeline_open);
        assert!(retained.studio_convergence_v1_initialized);
        let _ = fs::remove_file(path);
    }

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

#[cfg(test)]
mod m2d131_workspace_tests {
    use super::*;
    #[test]
    fn legacy_layout_receives_valid_persisted_docking() {
        let mut raw = serde_json::to_value(EditorLayout::default()).unwrap();
        raw.as_object_mut().unwrap().remove("workspace");
        let restored: EditorLayout = serde_json::from_value(raw).unwrap();
        restored.workspace.validate().unwrap();
        assert!(restored.workspace.right.open);
        assert!(!restored.workspace.bottom.open);
    }
    #[test]
    fn workspace_roundtrip_keeps_existing_brush_and_asset_settings() {
        let mut layout = EditorLayout::default();
        layout.editor_layout_mock_mutate();
        let restored: EditorLayout =
            serde_json::from_str(&serde_json::to_string(&layout).unwrap()).unwrap();
        restored.workspace.validate().unwrap();
        assert_eq!(layout.brush_mode, restored.brush_mode);
        assert_eq!(layout.library_in_inspector, restored.library_in_inspector);
    }
    impl EditorLayout {
        fn editor_layout_mock_mutate(&mut self) {
            self.brush_mode = "tile".into();
            self.workspace.right.extent = 388.0;
            self.library_in_inspector = true;
        }
    }

    #[test]
    fn m2d131b_right_browser_migration_defaults_to_half_and_runs_once() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d131b_overlay_layout_{}.json",
            std::process::id()
        ));
        let mut old = serde_json::to_value(EditorLayout::default()).unwrap();
        let fields = old.as_object_mut().unwrap();
        fields.remove("canvasOverlayV1Initialized");
        fields.remove("inspectorSplitRatio");
        fields.insert("libraryInInspector".into(), serde_json::json!(false));
        fields.insert("libraryOpen".into(), serde_json::json!(false));
        fields.insert("libraryInLeftWorkspace".into(), serde_json::json!(true));
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let migrated = EditorLayout::load(&path);
        assert!(migrated.canvas_overlay_v1_initialized);
        assert!(migrated.library_in_inspector);
        assert!(migrated.library_open);
        assert!(!migrated.library_in_left_workspace);
        assert_eq!(migrated.inspector_split_ratio, 0.5);
        let mut chosen = migrated;
        chosen.inspector_split_ratio = 0.66;
        chosen.save(&path).unwrap();
        assert_eq!(EditorLayout::load(&path).inspector_split_ratio, 0.66);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn m2d131b3_single_dock_and_inspector_tab_migrate_once() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d131b3_left_inspector_{}.json",
            std::process::id()
        ));
        let mut old = serde_json::to_value(EditorLayout::default()).unwrap();
        let fields = old.as_object_mut().unwrap();
        fields.remove("leftDockV2Initialized");
        fields.remove("inspectorTab");
        fields.insert("structuralOpen".into(), serde_json::json!(true));
        fields.insert("authoringContextWidth".into(), serde_json::json!(350.0));
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let migrated = EditorLayout::load(&path);
        assert_eq!(migrated.inspector_tab, "World");
        assert_eq!(migrated.authoring_context_width, 288.0);
        assert!(migrated.left_dock_v2_initialized);
        let mut customized = migrated;
        customized.authoring_context_width = 315.0;
        customized.inspector_tab = "Terrain".into();
        customized.structural_open = false;
        customized.save(&path).unwrap();
        let restored = EditorLayout::load(&path);
        assert_eq!(restored.authoring_context_width, 315.0);
        assert_eq!(restored.inspector_tab, "Terrain");
        assert!(!restored.structural_open);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn m2d131b3_custom_width_is_not_overwritten_by_legacy_upgrade() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d131b3_custom_width_{}.json",
            std::process::id()
        ));
        let mut old = EditorLayout::default();
        old.left_dock_v2_initialized = false;
        old.authoring_context_width = 410.0;
        old.save(&path).unwrap();
        let restored = EditorLayout::load(&path);
        assert_eq!(restored.authoring_context_width, 410.0);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn m2d131b_explicit_floating_library_is_not_silently_redocked() {
        let path = std::env::temp_dir().join(format!(
            "havenwild_m2d131b_floating_layout_{}.json",
            std::process::id()
        ));
        let mut old = serde_json::to_value(EditorLayout::default()).unwrap();
        let fields = old.as_object_mut().unwrap();
        fields.remove("canvasOverlayV1Initialized");
        fields.insert("libraryFloating".into(), serde_json::json!(true));
        fields.insert("libraryInInspector".into(), serde_json::json!(false));
        fs::write(&path, serde_json::to_vec(&old).unwrap()).unwrap();
        let migrated = EditorLayout::load(&path);
        assert!(migrated.library_floating);
        assert!(!migrated.library_in_inspector);
        let _ = fs::remove_file(&path);
    }
}
