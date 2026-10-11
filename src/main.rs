//! Fresh independent ElizaWy-first world canvas / mapping seed.
//! One primary Bevy camera and one original-pixel atlas renderer. No PCC gates,
//! historical Bevy validation probes, alternative world renderers, or game claims.
mod asset_authority;
mod asset_review;
mod atomic_file;
mod canvas_chrome;
mod canvas_rulers;
mod document;
mod editor_commands;
mod editor_icons;
mod editor_layout;
mod elizawy_review;
mod historical_evidence;
mod layer_overlay;
mod playtest;
mod project_root;
mod render_asset_sync;
mod room_authoring;
mod room_scene;
mod scene_v2;
mod semantic_lab;
mod source_catalog;
mod summer_seed;
mod terrain;
mod terrain_mapper;
mod terrain_resolver;
mod tiled_terrain;
mod workspace_layout;
mod world_chunks;
mod world_doc;
mod world_pie;
mod worldgen_plan;

use asset_authority::AssetAuthority;
use bevy::{image::ImagePlugin, prelude::*};
use bevy_egui::{
    egui, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, EguiTextureHandle,
    EguiUserTextures, PrimaryEguiContext,
};
use document::Scene;
use editor_commands::{
    spec as command_spec, tool_command, tooltip as command_tooltip, StudioCommand,
};
use editor_icons::{rail_icon_button, RailIcon};
use editor_layout::EditorLayout;
use elizawy_review::{ReviewBrowser, ReviewCandidate};
use forge_gui_chrome::{apply_creator_visuals, SurfaceDock};
use forge_gui_theme::{ForgeTheme, ForgeThemePreset};
use historical_evidence::HistoricalEvidence;
use playtest::PlaySession;
use scene_v2::{
    CollisionMask32, LayerId, PaintCell, PlacedObject, SceneCommand, SceneHistory, SceneV2,
    SourceBinding, StructuralCell, COLLISION_MASK_SIDE,
};
use semantic_lab::SemanticTerrainLab;
use source_catalog::SourceCatalog;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use terrain_mapper::{mask_meaning, TerrainMapper, TerrainRecipeSource, TerrainRecipeState};
use terrain_resolver::{all_vertex_coords, dirty_vertices_for_cell, resolve_semantic_lab};
use tiled_terrain::CanonicalTiledTerrainAuthority;
use world_doc::{TraversalMode, VisualOverrideMode, WorldDocument};
use world_pie::WorldPlaySession;

// Hydrated LPC character art is optional; no character PNG is vendored in PCC patches.
const PLAYER_WALK_SHEET: &str = "Characters/Body/Body 01 - Feminine, Thin/Peach/Walk.png";

const INITIAL_SHEETS: &[&str] = &[
    "Terrain/terrain_summer.png",
    "Terrain/cliff_summer.png",
    "Terrain/Waterfall.png",
    "Terrain/plants_summer.png",
];

#[derive(Clone)]
struct SemanticHistoryEntry {
    cells: Vec<bool>,
    dirty_vertices: Vec<terrain::TerrainVertexCoord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CanvasTool {
    Select,
    Paint,
    Erase,
    Sample,
}

impl CanvasTool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Paint => "Paint terrain",
            Self::Erase => "Erase authored terrain",
            Self::Sample => "Pick terrain/source",
        }
    }
}

// Painting identity is independent from the active source sheet and terrain material.
// Never route a Pixel stroke to the world semantic or tile-override document.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BrushMode {
    Pixel,
    Tile,
    Terrain,
}
impl BrushMode {
    fn label(self) -> &'static str {
        match self {
            Self::Pixel => "Pixel Brush",
            Self::Tile => "Tile Brush",
            Self::Terrain => "Terrain Brush",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldTerrainBrush {
    Grass,
    DirtBank,
    Sand,
    WetSand,
    Water,
    DeepWater,
    PebblePath,
}

impl WorldTerrainBrush {
    fn role(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::DirtBank => "MudBank",
            Self::Sand => "Sand",
            Self::WetSand => "WetSand",
            Self::Water => "RiverWater",
            Self::DeepWater => "DeepWater",
            Self::PebblePath => "PebblePath",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::DirtBank => "Dirt / river bank",
            Self::Sand => "Sand",
            Self::WetSand => "Legacy wet/tidal sand (manual)",
            Self::Water => "Shallow / river water",
            Self::DeepWater => "Deep water",
            Self::PebblePath => "Legacy pebble/stone path (manual)",
        }
    }

    fn from_role(role: &str) -> Option<Self> {
        match role {
            "Grass" => Some(Self::Grass),
            "MudBank" => Some(Self::DirtBank),
            "Sand" => Some(Self::Sand),
            "WetSand" => Some(Self::WetSand),
            "RiverWater" => Some(Self::Water),
            "DeepWater" => Some(Self::DeepWater),
            "PebblePath" => Some(Self::PebblePath),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldBehaviorScope {
    TerrainTopology,
    LocalArea,
}

impl WorldBehaviorScope {
    fn label(self) -> &'static str {
        match self {
            Self::TerrainTopology => "Terrain topology",
            Self::LocalArea => "This area only",
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MappingLabTestZone {
    origin: [usize; 2],
    size_cells: [usize; 2],
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MappingLabVisualSample {
    id: String,
    label: String,
    source_asset: String,
    source_rect: [u32; 4],
    origin: [usize; 2],
    size_cells: [usize; 2],
    purpose: String,
    review_state: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MappingLabIndex {
    schema: String,
    generated_scene_size: [usize; 2],
    test_zone: MappingLabTestZone,
    visual_samples: Vec<MappingLabVisualSample>,
    summer_recovered_source_cells: usize,
    summer_source_cells_actually_used: usize,
    certified_recipe_masks: usize,
}
impl MappingLabIndex {
    fn load(path: &Path, scene: &Scene) -> Result<Self, String> {
        let index: Self = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if index.schema != "havenwild.elizawy.assembled_certification_scene.v1"
            || index.generated_scene_size != scene.size
            || index.test_zone.origin != [0, 0]
            || index.test_zone.size_cells != scene.size
            || index.summer_recovered_source_cells != 305
            || index.summer_source_cells_actually_used == 0
            || index.certified_recipe_masks != 0
            || index.visual_samples.len() != 0
        {
            return Err("Assembled mapping scene/index mismatch; original source untouched".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for item in &index.visual_samples {
            if item.id.is_empty()
                || !ids.insert(item.id.as_str())
                || item.review_state != "visual_integration_draft_not_certified"
                || item.source_asset.is_empty()
                || item.source_rect[2] == 0
                || item.source_rect[3] == 0
                || item.size_cells.iter().zip(item.origin).zip(scene.size).any(
                    |((&len, start), bound)| {
                        len == 0 || start.checked_add(len).is_none_or(|end| end > bound)
                    },
                )
            {
                return Err(format!("Invalid source-bound sample {}", item.id));
            }
        }
        if scene
            .hidden_visual_samples
            .iter()
            .any(|id| !ids.contains(id.as_str()))
        {
            return Err("Derived draft contains an unknown visual-sample removal".into());
        }
        Ok(index)
    }
    fn sample_zone_contains(&self, pos: [usize; 2]) -> bool {
        (0..2).all(|axis| {
            pos[axis] >= self.test_zone.origin[axis]
                && pos[axis] - self.test_zone.origin[axis] < self.test_zone.size_cells[axis]
        })
    }
    fn visual_at<'a>(
        &'a self,
        pos: [usize; 2],
        scene: &Scene,
    ) -> Option<&'a MappingLabVisualSample> {
        self.visual_samples.iter().rev().find(|item| {
            !scene.hidden_visual_samples.contains(&item.id)
                && (0..2).all(|axis| {
                    pos[axis] >= item.origin[axis]
                        && pos[axis] - item.origin[axis] < item.size_cells[axis]
                })
        })
    }
}

struct Sheet {
    path: String,
    size: [u32; 2],
    handle: Handle<Image>,
    texture: Option<egui::TextureId>,
    // PNGs have straight alpha but egui composites premultiplied textures.
    // Never reveal an unprepared source sheet in the canvas or source browser.
    premultiplied_for_egui: bool,
}

#[derive(Clone)]
struct CachedWorldAnimationFrame {
    sheet_index: usize,
    source_rect: [u32; 4],
    duration_ms: u32,
}

#[derive(Clone)]
struct CachedWorldAnimation {
    frames: Vec<CachedWorldAnimationFrame>,
    total_duration_ms: u64,
    phase_ms: u64,
}

impl CachedWorldAnimation {
    fn source_at(&self, elapsed_ms: u64) -> Option<(usize, [u32; 4])> {
        if self.frames.is_empty() || self.total_duration_ms == 0 {
            return None;
        }
        let mut cursor = elapsed_ms
            .wrapping_add(self.phase_ms)
            .rem_euclid(self.total_duration_ms);
        for frame in &self.frames {
            let duration = u64::from(frame.duration_ms);
            if cursor < duration {
                return Some((frame.sheet_index, frame.source_rect));
            }
            cursor = cursor.saturating_sub(duration);
        }
        self.frames
            .last()
            .map(|frame| (frame.sheet_index, frame.source_rect))
    }
}

#[derive(Clone)]
struct CachedWorldVisualPart {
    sheet_index: usize,
    source_rect: [u32; 4],
    destination_rect_px: [u32; 4],
    animation: Option<CachedWorldAnimation>,
}

#[derive(Clone)]
struct CachedWorldVertexVisual {
    layer: LayerId,
    parts: Vec<CachedWorldVisualPart>,
}

#[derive(Default)]
struct WorldVisualCache {
    revision: u64,
    pending_edit_count: usize,
    sheet_count: usize,
    tiled_vertex_tiles: HashMap<[i64; 2], u32>,
    tiled_solve_error: Option<String>,
    vertices: HashMap<[i64; 2], Option<CachedWorldVertexVisual>>,
}

#[derive(Resource)]
struct Studio {
    scene: Scene, // imported v1 evidence baseline; editing occurs in sparse live v2 layers
    layered_scene: SceneV2,
    layered_history: SceneHistory,
    layered_path: PathBuf,
    scene_needs_initial_save: bool,
    legacy_base_path: PathBuf,
    active_layer: LayerId,
    layer_isolate: layer_overlay::LayerIsolateState,
    playtest_layered: Option<SceneV2>,
    playtest: Option<PlaySession>,
    world_playtest: Option<WorldPlaySession>,
    room_authoring: Option<room_authoring::RoomEditor>,
    room_authoring_object: Option<String>,
    world_door_mapping: Option<String>,
    saved_editor_view: Option<(f32, egui::Vec2)>,
    mapping_lab: MappingLabIndex,
    sheets: Vec<Sheet>,
    sheet_lookup: HashMap<String, usize>,
    active_sheet: usize,
    selected_cell: [u32; 2],
    selected_extent: [u32; 2], // explicit user-chosen source-region size in 32px cells
    selected_world: [usize; 2],
    selection_active: bool,
    selected_object_id: Option<String>,
    dragging_object: Option<(String, [i32; 2])>,
    drag_preview_origin: Option<[i32; 2]>,
    source_filter: String,
    source_family_filter: String,
    // Compact asset thumbnails use background alpha occupancy, keyed by exact source PNG.
    // The CPU decode never blocks egui or modifies the original ElizaWy art.
    occupied_source_cells: Arc<Mutex<HashMap<String, Result<Vec<[u32; 2]>, String>>>>,
    occupancy_requests: BTreeSet<String>,
    source_catalog: SourceCatalog,
    alpha_candidates: asset_review::AlphaCandidateIndex,
    world_template_placement: Option<String>,
    asset_authority: AssetAuthority,
    tiled_terrain: Option<CanonicalTiledTerrainAuthority>,
    world_doc: WorldDocument,
    world_path: PathBuf,
    world_mode: bool,
    world_dirty: bool,
    world_selected: [i64; 2],
    /// Position captured by the context menu without changing canvas selection.
    world_context_cell: Option<[i64; 2]>,
    world_selected_object: Option<String>,
    world_dragging_object: Option<(String, [i64; 2])>,
    world_drag_preview_anchor: Option<[i64; 2]>,
    world_bridge_tool: bool,
    world_bridge_drag: Option<([i64; 2], [i64; 2])>,
    shoreline_audit_summary: String,
    world_undo: Vec<WorldDocument>,
    world_redo: Vec<WorldDocument>,
    // A world paint stroke is authored locally while the pointer is down, then
    // topology-safe corrections are promoted to generator knowledge once on release.
    // This keeps Paint responsive and avoids regenerating every materialized chunk
    // for every mouse-move event.
    world_stroke_cells: Vec<[i64; 2]>,
    world_stroke_last_center: Option<[i64; 2]>,
    world_stroke_seen: BTreeSet<[i64; 2]>,
    world_stroke_binding: Option<SourceBinding>,
    world_stroke_role: Option<String>,
    world_terrain_brush: WorldTerrainBrush,
    brush_mode: BrushMode,
    world_stroke_erase: bool,
    world_stroke_local_only: bool,
    pending_source_sheet: Option<String>,
    pending_source_cell: Option<[u32; 2]>,
    // World revision invalidation prevents newly generated source-backed props
    // (especially stones, vegetation and houses) from vanishing on a cold sheet.
    runtime_asset_sync_revision: Option<u64>,
    runtime_asset_sync_queue: VecDeque<String>,
    runtime_asset_sync_failures: BTreeMap<String, String>,
    semantic_lab: SemanticTerrainLab,
    semantic_lab_path: PathBuf,
    semantic_lab_seed_path: PathBuf,
    semantic_lab_dirty: bool,
    semantic_undo: Vec<SemanticHistoryEntry>,
    semantic_redo: Vec<SemanticHistoryEntry>,
    last_dirty_vertices: Vec<terrain::TerrainVertexCoord>,
    selected_lab_vertex: [usize; 2],
    atlas_zoom: f32,
    launcher_open: bool,
    collision_world_pixel_edit: bool,
    collision_brush_blocked: bool,
    collision_brush_radius: usize,
    collision_stroke_active: bool,
    world_behavior_scope: WorldBehaviorScope,
    world_detail_lod_visible: bool,
    world_visual_cache: WorldVisualCache,
    tool: CanvasTool,
    focus_mode: bool,
    zoom: f32,
    pan: egui::Vec2,
    canvas_rect: egui::Rect,
    scene_rect: egui::Rect,
    canvas_controls_rect: egui::Rect,
    canvas_grid_rect: egui::Rect,
    canvas_layers_rect: egui::Rect,
    canvas_animation_rect: egui::Rect,
    /// Editor-only elapsed animation clock; no gameplay clock or authored frames modified.
    animation_preview_ms: f64,
    display_fps: f32,
    atlas_rect: egui::Rect,
    structural_rect: egui::Rect,
    world_generator_rect: egui::Rect,
    launcher_rect: egui::Rect,
    dragging: Option<(usize, [u32; 4])>,
    undo: Vec<Scene>,
    redo: Vec<Scene>,
    dirty: bool,
    message: String,
    /// Session-only diagnostics; intentionally never serialized with a world or scene.
    console_history: VecDeque<String>,
    console_last_message: String,
    console_tab: String,
    theme: ForgeTheme,
    creator_visuals_applied: bool,
    terrain_mapper: TerrainMapper,
    review_browser: ReviewBrowser,
    historical_evidence: Result<HistoricalEvidence, String>,
    editor_layout: EditorLayout,
    layout_path: PathBuf,
}

// Source Browser uses the entire canonical PNG catalog. Only selected sheets and
// sheets referenced by an active scene are actually requested from the asset server.
// A source-bound record is never silently replaced by a manufactured fallback image.
fn load_source_sheet_on_demand(
    state: &mut Studio,
    assets: &AssetServer,
    textures: &mut EguiUserTextures,
    relative: &str,
) -> Result<usize, String> {
    if let Some(index) = state.sheets.iter().position(|sheet| sheet.path == relative) {
        return Ok(index);
    }
    let Some(entry) = state.source_catalog.entry(relative) else {
        return Err(format!(
            "Unknown source PNG in the canonical manifest: {relative}"
        ));
    };
    let expected = entry.size;
    let path = project_root::resolve()
        .join("assets/elizawy")
        .join(relative);
    let actual = image_size(&path)
        .ok_or_else(|| format!("Source image missing or invalid: {}", path.display()))?;
    if actual != expected {
        return Err(format!("Original image dimensions differ from canonical manifest for {relative}: expected {expected:?}, found {actual:?}"));
    }
    let handle: Handle<Image> = assets.load(relative.to_owned());
    textures.add_image(EguiTextureHandle::Strong(handle.clone()));
    state.sheets.push(Sheet {
        path: relative.into(),
        size: expected,
        handle,
        texture: None,
        premultiplied_for_egui: false,
    });
    let index = state.sheets.len() - 1;
    state.sheet_lookup.insert(relative.to_owned(), index);
    Ok(index)
}

fn image_size(path: &Path) -> Option<[u32; 2]> {
    // PNG dimensions live in the fixed IHDR header. Do not read every hydrated
    // texture into CPU memory merely to validate catalog dimensions at startup.
    let mut bytes = [0u8; 24];
    fs::File::open(path).ok()?.read_exact(&mut bytes).ok()?;
    if &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    Some([
        u32::from_be_bytes(bytes[16..20].try_into().ok()?),
        u32::from_be_bytes(bytes[20..24].try_into().ok()?),
    ])
}

fn assert_current_binary_matches_source(root: &Path) {
    let project_path = root.join("project/forgepy.project.json");
    let Ok(raw) = fs::read_to_string(&project_path) else {
        return;
    };
    let Ok(document) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    let Some(source_version) = document
        .get("sourceVersion")
        .and_then(|value| value.as_str())
    else {
        return;
    };
    let compiled = env!("CARGO_PKG_VERSION");
    assert_eq!(
        source_version, compiled,
        "STALE HAVENWILD STUDIO BINARY: executable is v{compiled}, but this project source is v{source_version}. Launch through Studio.cmd / PCC so Cargo rebuilds the current source before visual acceptance."
    );
}

fn startup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut textures: ResMut<EguiUserTextures>,
    mut settings: ResMut<EguiGlobalSettings>,
) {
    settings.auto_create_primary_context = false;
    commands.spawn((Camera2d, PrimaryEguiContext));
    let root = project_root::resolve();
    assert_current_binary_matches_source(&root);
    // A new, separately saved Summer authoring document starts from a pinned,
    // source-fingerprinted River fixture. Previous C-era local v1/v2 draft files
    // are preserved as recovery material and never silently overwritten.
    let source_scene_path = root.join("content/scenes/elizawy_mapping_certification.scene.json");
    let scene = Scene::load(&source_scene_path)
        .unwrap_or_else(|error| panic!("Cannot open immutable Summer source terrain: {error}"));
    let legacy_base_path = source_scene_path.clone();
    let layered_path = root.join("content/scenes/derived/summer_world.layered.draft.json");
    let scene_needs_initial_save = !layered_path.is_file();
    let layered_scene = if !scene_needs_initial_save {
        let doc = SceneV2::read_draft(&layered_path).unwrap_or_else(|error| {
            panic!("Cannot read existing Summer authored scene without replacing it: {error}")
        });
        doc.verify_legacy_source(&legacy_base_path)
            .unwrap_or_else(|error| panic!("Existing Summer scene refers to different original terrain bytes: {error}. Nothing was overwritten."));
        doc
    } else {
        let mut doc = SceneV2::preview_import(&legacy_base_path)
            .unwrap_or_else(|error| panic!("Cannot initialize source-exact Summer world: {error}"));
        summer_seed::populate_new_scene(
            &mut doc,
            &root.join("content/scenes/summer_world.visual_seed.v1.json"),
        )
        .unwrap_or_else(|error| {
            panic!("Cannot assemble original-art Summer opening scene: {error}")
        });
        doc
    };
    let mapping_lab = MappingLabIndex::load(
        &root.join("content/scenes/elizawy_mapping_certification.index.json"),
        &scene,
    )
    .unwrap_or_else(|error| panic!("Cannot open mapping certification layout: {error}"));
    let terrain_recipe_path = root.join("content/terrain/recipes/summer_grass_void.v1.json");
    let historical_path = root.join("content/terrain/recovery/historical_authority.v1.json");
    let source_inventory_path = root.join("content/terrain/recovery/summer_source_cells.v1.json");
    let terrain_mapper = TerrainMapper::load(
        &terrain_recipe_path,
        &historical_path,
        &source_inventory_path,
    )
    .unwrap_or_else(|error| panic!("Cannot open terrain recipe/recovery metadata: {error}"));
    let layout_path = root.join(".forgepy/editor_layout.local.json");
    let editor_layout = EditorLayout::load(&layout_path);
    let catalog_path = root.join("content/catalog/core_source_manifest.json");
    let source_catalog = SourceCatalog::load(&catalog_path)
        .unwrap_or_else(|error| panic!("Cannot open ElizaWy source catalog: {error}"));
    let source_catalog_images = source_catalog.image_count();
    let source_catalog_terrain = source_catalog.terrain_count();
    let asset_authority = AssetAuthority::load(
        &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
        &source_catalog,
    )
    .unwrap_or_else(|error| panic!("Cannot open unified ElizaWy asset authority: {error}"));
    let alpha_candidates = asset_review::AlphaCandidateIndex::load(
        &root.join("content/assets/review/original_alpha_candidates.v1.json"),
        &asset_authority,
    )
    .unwrap_or_else(|error| panic!("Cannot open original-alpha census: {error}"));
    let tiled_terrain = match CanonicalTiledTerrainAuthority::load_summer(&root, &source_catalog) {
        Ok(authority) => {
            let (mapped, total) = authority.mapped_fraction();
            bevy::log::info!(
                "Tiled live terrain bridge: {mapped}/{total} Terrain Wang assignments map to exact canonical ElizaWy source pixels; animation frames {}/{} canonical; clips {} complete / {} incomplete",
                authority.canonical_animation_frame_count,
                authority.animation_frame_count,
                authority.canonical_animation_clip_count,
                authority.incomplete_animation_clip_count
            );
            Some(authority)
        }
        Err(error) => {
            bevy::log::warn!(
                "Tiled live terrain bridge unavailable; retaining legacy source-backed resolver: {error}"
            );
            None
        }
    };
    let world_path = root.join(".forgepy/world/havenwild_world.local.json");
    let mut world_doc = WorldDocument::read_or_new(&world_path, 0x484156454e57494c)
        .unwrap_or_else(|error| panic!("Cannot open local Havenwild world composition: {error}"));
    world_doc
        .validate_against_authority(&asset_authority)
        .unwrap_or_else(|error| {
            panic!("Local world composition violates unified ElizaWy authority: {error}")
        });
    let repaired_world_rules = world_doc
        .repair_invalid_generation_corrections(&asset_authority)
        .unwrap_or_else(|error| {
            panic!("Cannot repair unsafe world-generation corrections: {error}")
        });
    let quarantined_world_objects =
        world_doc.quarantine_uncertified_large_objects(&asset_authority);
    let generated_world_initialized = if world_doc.chunks.is_empty() {
        world_doc
            .materialize_with_tiled([0, 0], 2, &asset_authority, tiled_terrain.as_ref())
            .unwrap_or_else(|error| panic!("Cannot initialize Generated World: {error}"));
        true
    } else {
        false
    };
    // Exact bounds and source identity are checked for every saved v2 overlay/object.
    // Never replace missing material with a black rectangle or invented sprite.
    let mut layered_references = std::collections::BTreeSet::new();
    for layer in &layered_scene.visual_layers {
        for edit in &layer.edits {
            if let Some(source) = &edit.source {
                if !source_catalog.contains_rect(&source.source_asset, source.source_rect) {
                    panic!(
                        "Layered draft references missing/out-of-bounds original source: {} {:?}",
                        source.source_asset, source.source_rect
                    );
                }
                layered_references.insert(source.source_asset.clone());
            }
        }
    }
    for object in &layered_scene.objects {
        for part in &object.parts {
            if !source_catalog.contains_rect(&part.source.source_asset, part.source.source_rect) {
                panic!("Layered draft object references missing/out-of-bounds original source: {} {:?}", part.source.source_asset, part.source.source_rect);
            }
            layered_references.insert(part.source.source_asset.clone());
        }
    }
    // Generated World purpose/nature objects can use Structure/Object sheets that are
    // intentionally not part of the eager Terrain-only set. Promote every currently
    // materialized source sheet into the same exact-source texture load set.
    for chunk in &world_doc.chunks {
        for object in &chunk.generated_objects {
            for part in &object.parts {
                if !source_catalog.contains_rect(&part.source.source_asset, part.source.source_rect)
                {
                    panic!(
                        "Generated World object references missing/out-of-bounds original source: {} {:?}",
                        part.source.source_asset, part.source.source_rect
                    );
                }
                layered_references.insert(part.source.source_asset.clone());
            }
        }
    }
    let recipe_source_errors = terrain_mapper.source_validation_errors(&source_catalog);
    if !recipe_source_errors.is_empty() {
        panic!(
            "Terrain recipe source validation failed:\n{}",
            recipe_source_errors.join("\n")
        );
    }

    // Eager-load only 29 known Terrain source sheets for existing recipe preview.
    // The picker lists all 320 canonical PNGs and loads other families on demand.
    let mut sheet_paths = INITIAL_SHEETS
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<Vec<_>>();
    for entry in source_catalog.terrain_entries() {
        if !sheet_paths.iter().any(|path| path == &entry.path) {
            sheet_paths.push(entry.path.clone());
        }
    }
    let required_scene_paths = layered_references.clone();
    for relative in layered_references {
        if !sheet_paths.contains(&relative) {
            sheet_paths.push(relative);
        }
    }
    // Optional, hydrated LPC body walk sheet. The asset server already roots at
    // assets/elizawy; loading this does not modify or copy any original source PNG.
    if root
        .join("assets/elizawy")
        .join(PLAYER_WALK_SHEET)
        .is_file()
    {
        sheet_paths.push(PLAYER_WALK_SHEET.to_owned());
    }
    let mut sheets = Vec::new();
    for relative in sheet_paths {
        let path = root.join("assets/elizawy").join(&relative);
        let Some(size) = image_size(&path) else {
            if INITIAL_SHEETS
                .iter()
                .any(|required| *required == relative.as_str())
                || required_scene_paths.contains(&relative)
            {
                panic!(
                    "Missing required ElizaWy source sheet: {}. Run `ForgePY.cmd assets sync --source <path>` first.",
                    path.display()
                );
            }
            continue;
        };
        if let Some(expected) = source_catalog
            .entries
            .iter()
            .find(|entry| entry.path == relative)
            .map(|entry| entry.size)
        {
            if expected != size {
                panic!(
                    "Hydrated source dimensions differ from the canonical manifest for {}: expected {:?}, got {:?}",
                    relative, expected, size
                );
            }
        }
        let handle: Handle<Image> = assets.load(relative.clone());
        textures.add_image(EguiTextureHandle::Strong(handle.clone()));
        sheets.push(Sheet {
            path: relative,
            size,
            handle,
            texture: None,
            premultiplied_for_egui: false,
        });
    }
    let semantic_lab_seed_path =
        root.join("content/terrain/previews/dg01_grass_void.semantic_lab.v1.json");
    let semantic_lab_path = root.join(".forgepy/semantic_lab.local.json");
    let semantic_lab = if semantic_lab_path.is_file() {
        match SemanticTerrainLab::load(&semantic_lab_path) {
            Ok(lab) => lab,
            Err(error) => {
                bevy::log::warn!(
                    "Ignoring invalid local DG semantic lab {}: {}; falling back to checked-in seed",
                    semantic_lab_path.display(),
                    error
                );
                SemanticTerrainLab::load(&semantic_lab_seed_path).unwrap_or_else(|seed_error| {
                    panic!("Cannot open DG semantic resolver seed: {seed_error}")
                })
            }
        }
    } else {
        SemanticTerrainLab::load(&semantic_lab_seed_path)
            .unwrap_or_else(|error| panic!("Cannot open DG semantic resolver seed: {error}"))
    };
    if semantic_lab.foreground != terrain_mapper.family.foreground
        || semantic_lab.background != terrain_mapper.family.background
    {
        panic!(
            "Semantic resolver lab family {}/{} does not match recipe family {}/{}",
            semantic_lab.foreground,
            semantic_lab.background,
            terrain_mapper.family.foreground,
            terrain_mapper.family.background
        );
    }
    let resolver_coverage = semantic_lab
        .topology_coverage()
        .into_iter()
        .filter(|covered| *covered)
        .count();
    let hydrated_terrain_sheets = sheets
        .iter()
        .filter(|sheet| sheet.path.starts_with("Terrain/"))
        .count();
    bevy::log::info!(
        "Terrain authoring startup: {} / {} Terrain manifest sheets hydrated; {} total image entries; semantic resolver lab {}/16 masks",
        hydrated_terrain_sheets,
        source_catalog_terrain,
        source_catalog_images,
        resolver_coverage
    );
    // The pinned source-authored River supplies immutable ground UNDER the opening Summer
    // object composition. Guessed M2D02-D visual samples stay retired.
    let initial_zoom = 0.20;
    let initial_pan = egui::Vec2::ZERO;
    let summer_object_count = layered_scene.objects.len();
    let sheet_lookup = sheets
        .iter()
        .enumerate()
        .map(|(index, sheet)| (sheet.path.clone(), index))
        .collect::<HashMap<_, _>>();
    commands.insert_resource(Studio {
        scene,
        layered_scene,
        layered_history: SceneHistory::default(),
        layered_path,
        scene_needs_initial_save,
        legacy_base_path,
        active_layer: LayerId::Objects,
        layer_isolate: layer_overlay::LayerIsolateState::default(),
        playtest_layered: None,
        playtest: None,
        world_playtest: None,
        room_authoring: None,
        room_authoring_object: None,
        world_door_mapping: None,
        saved_editor_view: None,
        mapping_lab,
        sheets,
        sheet_lookup,
        active_sheet: 0,
        selected_cell: [0, 0],
        selected_extent: [1, 1],
        selected_world: [0, 0],
        selection_active: false,
        source_filter: String::new(),
        source_family_filter: "All".into(),
        occupied_source_cells: Arc::new(Mutex::new(HashMap::new())),
        occupancy_requests: BTreeSet::new(),
        source_catalog,
        alpha_candidates,
        world_template_placement: None,
        asset_authority,
        tiled_terrain,
        world_doc,
        world_path,
        world_mode: true,
        world_dirty: generated_world_initialized || repaired_world_rules > 0 || quarantined_world_objects > 0,
        world_selected: [0, 0],
        world_context_cell: None,
        world_selected_object: None,
        world_dragging_object: None,
        world_drag_preview_anchor: None,
        world_bridge_tool: false,
        world_bridge_drag: None,
        shoreline_audit_summary: String::new(),
        world_undo: Vec::new(),
        world_redo: Vec::new(),
        world_stroke_cells: Vec::new(),
        world_stroke_last_center: None,
        world_stroke_seen: BTreeSet::new(),
        world_stroke_binding: None,
        world_stroke_role: None,
        world_terrain_brush: WorldTerrainBrush::from_role(&editor_layout.terrain_brush_role)
            .unwrap_or(WorldTerrainBrush::Grass),
        brush_mode: match editor_layout.brush_mode.as_str() {
            "pixel" => BrushMode::Pixel,
            "tile" => BrushMode::Tile,
            _ => BrushMode::Terrain,
        },
        world_stroke_erase: false,
        world_stroke_local_only: false,
        pending_source_sheet: None,
        pending_source_cell: None,
        runtime_asset_sync_revision: None,
        runtime_asset_sync_queue: VecDeque::new(),
        runtime_asset_sync_failures: BTreeMap::new(),
        semantic_lab,
        semantic_lab_path,
        semantic_lab_seed_path,
        semantic_lab_dirty: false,
        semantic_undo: Vec::new(),
        semantic_redo: Vec::new(),
        last_dirty_vertices: Vec::new(),
        selected_lab_vertex: [0, 0],
        atlas_zoom: 0.75,
        launcher_open: false,
        collision_world_pixel_edit: false,
        collision_brush_blocked: true,
        collision_brush_radius: 1,
        collision_stroke_active: false,
        world_behavior_scope: WorldBehaviorScope::LocalArea,
        world_detail_lod_visible: true,
        world_visual_cache: WorldVisualCache::default(),
        tool: CanvasTool::Select,
        focus_mode: false,
        zoom: initial_zoom,
        pan: initial_pan,
        canvas_rect: egui::Rect::NOTHING,
        scene_rect: egui::Rect::NOTHING,
        canvas_controls_rect: egui::Rect::NOTHING,
        canvas_grid_rect: egui::Rect::NOTHING,
        canvas_layers_rect: egui::Rect::NOTHING,
        canvas_animation_rect: egui::Rect::NOTHING,
        display_fps: 0.0,
        animation_preview_ms: 0.0,
        atlas_rect: egui::Rect::NOTHING,
        structural_rect: egui::Rect::NOTHING,
        world_generator_rect: egui::Rect::NOTHING,
        launcher_rect: egui::Rect::NOTHING,
        dragging: None,
        selected_object_id: None,
        dragging_object: None,
        drag_preview_origin: None,
        undo: Vec::new(),
        redo: Vec::new(),
        dirty: false,
        console_history: VecDeque::new(),
        console_last_message: String::new(),
        console_tab: "Console".into(),
        message: if repaired_world_rules > 0 || quarantined_world_objects > 0 {
            format!(
                "Generated World safety repair: {repaired_world_rules} unsafe learned rule(s) repaired; {quarantined_world_objects} uncertified large generated object(s) quarantined. The four-season archipelago is now the primary authoring map."
            )
        } else if generated_world_initialized {
            "Generated World initialized: four season-specific island archetypes, ocean separation, interior water, roads, settlements and biome intent are materialized as the primary map. Ground rendering uses exact coordinate-equivalent seasonal ElizaWy terrain source regions; source pixels remain immutable.".into()
        } else {
            format!("Generated World ready: {} legacy source-exact scene instances remain available only as regression evidence; normal authoring now stays on the generated map.", summer_object_count)
        },
        theme: ForgeTheme::from_preset(ForgeThemePreset::MidnightMint),
        creator_visuals_applied: false,
        terrain_mapper,
        review_browser: ReviewBrowser::new(root.join(".forgepy/recovered_mapping/elizawy_tiled_review_registry.v1.json")),
        historical_evidence: HistoricalEvidence::load(&root),
        editor_layout,
        layout_path,
    });
}

fn source_uv(rect: [u32; 4], sheet: [u32; 2]) -> egui::Rect {
    let inset_x = if rect[2] > 1 { 0.5 } else { 0.0 };
    let inset_y = if rect[3] > 1 { 0.5 } else { 0.0 };
    egui::Rect::from_min_max(
        egui::pos2(
            (rect[0] as f32 + inset_x) / sheet[0] as f32,
            (rect[1] as f32 + inset_y) / sheet[1] as f32,
        ),
        egui::pos2(
            (rect[0] as f32 + rect[2] as f32 - inset_x) / sheet[0] as f32,
            (rect[1] as f32 + rect[3] as f32 - inset_y) / sheet[1] as f32,
        ),
    )
}

fn source_uv_object(_source_asset: &str, rect: [u32; 4], sheet: [u32; 2]) -> egui::Rect {
    // Object render bounds are now certified from immutable source alpha instead
    // of being hidden with a tree-specific UV inset. The normal half-pixel source
    // sampling keeps nearest-neighbour atlas reads stable without stretching
    // transparent padding or guide-edge pixels across the destination rectangle.
    source_uv(rect, sheet)
}

fn world_layer_id(name: &str) -> LayerId {
    if name.eq_ignore_ascii_case("ground") {
        LayerId::Ground
    } else if name.eq_ignore_ascii_case("terrain_details")
        || name.eq_ignore_ascii_case("terrain-details")
        || name.eq_ignore_ascii_case("details")
    {
        LayerId::TerrainDetails
    } else if name.eq_ignore_ascii_case("water") {
        LayerId::Water
    } else if name.eq_ignore_ascii_case("elevation") {
        LayerId::Elevation
    } else if name.eq_ignore_ascii_case("structures") || name.eq_ignore_ascii_case("structure") {
        LayerId::Structures
    } else if name.eq_ignore_ascii_case("foreground") {
        LayerId::Foreground
    } else {
        LayerId::Objects
    }
}

fn world_terrain_role_layer(role: &str) -> LayerId {
    match role {
        "RiverWater" | "DeepWater" => LayerId::Water,
        "MudBank" | "Sand" | "WetSand" | "PebblePath" => LayerId::TerrainDetails,
        _ => LayerId::Ground,
    }
}

fn world_terrain_vertex_layer(roles: &[String; 4]) -> LayerId {
    if roles
        .iter()
        .any(|role| matches!(role.as_str(), "RiverWater" | "DeepWater"))
    {
        LayerId::Water
    } else if roles
        .iter()
        .any(|role| matches!(role.as_str(), "MudBank" | "Sand" | "WetSand" | "PebblePath"))
    {
        LayerId::TerrainDetails
    } else {
        LayerId::Ground
    }
}

fn world_layer_slot(layer: LayerId) -> usize {
    match layer {
        LayerId::Ground => 0,
        LayerId::TerrainDetails => 1,
        LayerId::Water => 2,
        LayerId::Elevation => 3,
        LayerId::Structures => 4,
        LayerId::Objects => 5,
        LayerId::Foreground => 6,
    }
}

fn world_layer_visible(state: &Studio, layer: LayerId) -> bool {
    state.editor_layout.world_layer_visibility[world_layer_slot(layer)]
}

fn prepare_world_visual_cache(state: &mut Studio) {
    let revision = state.world_doc.revision;
    let pending_edit_count = state.world_doc.pending_semantic_edit_count();
    let sheet_count = state.sheets.len();
    if state.world_visual_cache.revision != revision
        || state.world_visual_cache.pending_edit_count != pending_edit_count
        || state.world_visual_cache.sheet_count != sheet_count
    {
        let tiled_solve = state.tiled_terrain.as_ref().map(|tiled| {
            state
                .world_doc
                .resolved_tiled_materialized_vertex_tiles(tiled)
        });
        state.world_visual_cache.revision = revision;
        state.world_visual_cache.pending_edit_count = pending_edit_count;
        state.world_visual_cache.sheet_count = sheet_count;
        state.world_visual_cache.vertices.clear();
        state.world_visual_cache.tiled_vertex_tiles.clear();
        state.world_visual_cache.tiled_solve_error = None;
        if let Some(result) = tiled_solve {
            match result {
                Ok(tiles) => {
                    state.world_visual_cache.tiled_vertex_tiles = tiles.into_iter().collect();
                }
                Err(error) => state.world_visual_cache.tiled_solve_error = Some(error),
            }
        }
    }
}

fn cached_world_animation(
    state: &Studio,
    animation: &asset_authority::SummerVisualAnimation,
) -> Option<CachedWorldAnimation> {
    if animation.frames.is_empty() || animation.total_duration_ms == 0 {
        return None;
    }
    let frames = animation
        .frames
        .iter()
        .map(|frame| {
            state
                .sheet_lookup
                .get(frame.source.source_asset.as_str())
                .copied()
                .map(|sheet_index| CachedWorldAnimationFrame {
                    sheet_index,
                    source_rect: frame.source.source_rect,
                    duration_ms: frame.duration_ms,
                })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(CachedWorldAnimation {
        frames,
        total_duration_ms: animation.total_duration_ms,
        phase_ms: animation.phase_ms,
    })
}

fn cached_world_vertex_visual(
    state: &mut Studio,
    vertex: [i64; 2],
) -> Option<CachedWorldVertexVisual> {
    if let Some(cached) = state.world_visual_cache.vertices.get(&vertex) {
        return cached.clone();
    }
    let roles = state.world_doc.resolved_dual_grid_roles(vertex);
    let layer = world_terrain_vertex_layer(&roles);
    let tiled_part = state
        .world_visual_cache
        .tiled_vertex_tiles
        .get(&vertex)
        .copied()
        .and_then(|tile_id| {
            let tiled = state.tiled_terrain.as_ref()?;
            state.world_doc.resolved_tiled_visual_part_for_tile_id(
                vertex,
                &state.asset_authority,
                tiled,
                tile_id,
            )
        });
    let parts = tiled_part.map(|part| vec![part]).or_else(|| {
        state
            .world_doc
            .resolved_dual_grid_visual_parts_with_authorities(
                vertex,
                &state.asset_authority,
                state.tiled_terrain.as_ref(),
            )
    });
    let cached = parts.map(|parts| CachedWorldVertexVisual {
        layer,
        parts: parts
            .into_iter()
            .filter_map(|part| {
                state
                    .sheet_lookup
                    .get(part.source.source_asset.as_str())
                    .copied()
                    .map(|sheet_index| CachedWorldVisualPart {
                        sheet_index,
                        source_rect: part.source.source_rect,
                        destination_rect_px: part.destination_rect_px,
                        animation: part
                            .animation
                            .as_ref()
                            .and_then(|animation| cached_world_animation(state, animation)),
                    })
            })
            .collect(),
    });
    state
        .world_visual_cache
        .vertices
        .insert(vertex, cached.clone());
    cached
}

fn world_object_depth_key(object: &world_doc::WorldObject, anchor: [i64; 2]) -> (i64, i64, &str) {
    (
        anchor[1] + i64::from(object.footprint_cells[1]),
        anchor[0] + i64::from(object.footprint_cells[0] / 2),
        object.id.as_str(),
    )
}

fn world_object_visual_bounds(object: &world_doc::WorldObject, anchor: [i64; 2]) -> [f32; 4] {
    let mut min_x = anchor[0] as f32;
    let mut min_y = anchor[1] as f32;
    let mut max_x = min_x + object.footprint_cells[0] as f32;
    let mut max_y = min_y + object.footprint_cells[1] as f32;
    for part in &object.parts {
        let x = anchor[0] as f32 + part.offset_px[0] as f32 / 32.0;
        let y = anchor[1] as f32 + part.offset_px[1] as f32 / 32.0;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + part.source.source_rect[2] as f32 / 32.0);
        max_y = max_y.max(y + part.source.source_rect[3] as f32 / 32.0);
    }
    [min_x, min_y, max_x, max_y]
}

fn world_object_is_minor(object: &world_doc::WorldObject) -> bool {
    let label = object.label.to_ascii_lowercase();
    label.contains("detail")
        || label.contains("wildflower")
        || label.contains("foliage")
        || label.contains("shrub")
        || label.contains("clutter")
}

fn world_at(state: &Studio, pointer: egui::Pos2) -> Option<[usize; 2]> {
    if !state.canvas_rect.contains(pointer)
        || !state.scene_rect.contains(pointer)
        || canvas_rulers::blocks_scene_edit(
            state.canvas_rect,
            pointer,
            canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
        )
        || state.canvas_controls_rect.contains(pointer)
        || state.canvas_grid_rect.contains(pointer)
        || state.canvas_layers_rect.contains(pointer)
        || state.canvas_animation_rect.contains(pointer)
        || (state.editor_layout.terrain_mapper.visible && state.atlas_rect.contains(pointer))
        || (state.editor_layout.structural_open && state.structural_rect.contains(pointer))
        || (state.launcher_open && state.launcher_rect.contains(pointer))
    {
        return None;
    }
    let scale = state.scene_rect.width() / state.scene.size[0] as f32;
    if scale <= 0.0 {
        return None;
    }
    let x = ((pointer.x - state.scene_rect.left()) / scale).floor() as usize;
    let y = ((pointer.y - state.scene_rect.top()) / scale).floor() as usize;
    state.scene.index(x, y).map(|_| [x, y])
}

fn is_complete_object_layer(layer: LayerId) -> bool {
    matches!(
        layer,
        LayerId::Elevation | LayerId::Structures | LayerId::Objects | LayerId::Foreground
    )
}

fn selected_object(state: &Studio) -> Option<&PlacedObject> {
    let id = state.selected_object_id.as_deref()?;
    state
        .layered_scene
        .objects
        .iter()
        .find(|object| object.id == id)
}

fn select_object_at(state: &mut Studio, pos: [usize; 2]) -> bool {
    let Some(object) = state
        .layered_scene
        .objects
        .iter()
        .filter(|object| world_layer_visible(state, object.layer) && object.contains_cell(pos))
        .max_by_key(|object| {
            (
                world_layer_slot(object.layer),
                object.anchor[1],
                object.anchor[0],
            )
        })
    else {
        state.selected_object_id = None;
        return false;
    };
    let id = object.id.clone();
    let label = object.label.clone();
    let layer = object.layer;
    state.selected_object_id = Some(id);
    state.active_layer = layer;
    state.selected_world = pos;
    state.selection_active = true;
    state.message = format!("Selected {label} on {layer:?}; drag with Select; properties are in the right Inspector. Bounds are visual, not collision.");
    true
}

fn remove_selected_object(state: &mut Studio) -> bool {
    let Some(id) = state.selected_object_id.clone() else {
        return false;
    };
    match state.layered_history.execute(
        &mut state.layered_scene,
        SceneCommand::RemoveObject { id: id.clone() },
    ) {
        Ok(()) => {
            state.selected_object_id = None;
            state.dragging_object = None;
            state.drag_preview_origin = None;
            state.message = format!("Removed placed object {id}. Underlying ground and other layers are untouched. Undo restores it.");
            true
        }
        Err(error) => {
            state.message = format!("Object removal refused: {error}");
            true
        }
    }
}

fn move_selected_object_to_origin(state: &mut Studio, id: String, to: [i32; 2]) {
    match state.layered_history.execute(
        &mut state.layered_scene,
        SceneCommand::MoveObject { id: id.clone(), to },
    ) {
        Ok(()) => {
            if let Some(object) = selected_object(state) {
                let focus = [
                    to[0] + (object.footprint[0] as i32 - 1) / 2,
                    to[1] + (object.footprint[1] as i32 - 1),
                ];
                state.selected_world = [focus[0] as usize, focus[1] as usize];
            }
            state.message = format!("Moved complete object {id}; other scene layers and original artwork unchanged. Undo available.");
        }
        Err(error) => state.message = format!("Object move refused: {error}"),
    }
}

// Paint is now a v2 source-bound transaction on an explicitly chosen visual layer.
// A foreground object/reed cannot replace the v1 grass or river beneath it.
fn apply_source(state: &mut Studio, pos: [usize; 2], sheet_index: usize, source: [u32; 4]) {
    if !state.mapping_lab.sample_zone_contains(pos) {
        return;
    }
    let Some(index) = state.scene.index(pos[0], pos[1]) else {
        return;
    };
    let sheet = &state.sheets[sheet_index];
    if !state.source_catalog.contains_rect(&sheet.path, source) {
        state.message = "Source selection exceeds original PNG boundaries; nothing changed.".into();
        return;
    }
    if state.active_layer == LayerId::Ground && !sheet.path.starts_with("Terrain/") {
        state.message = "Original Object/Structure/FX art cannot replace the ground. Choose Objects, Structures or Foreground in Layers.".into();
        return;
    }
    let binding = SourceBinding {
        source_asset: sheet.path.clone(),
        source_rect: source,
    };
    if is_complete_object_layer(state.active_layer) {
        let footprint = [source[2].div_ceil(32), source[3].div_ceil(32)];
        let Some(anchor) = PlacedObject::origin_at_foot(pos, footprint) else {
            state.message = "Invalid complete-object footprint; placement canceled.".into();
            return;
        };
        let object = PlacedObject::manual_region(
            state.layered_scene.next_object_id(),
            state.active_layer,
            anchor,
            binding,
        );
        let id = object.id.clone();
        match state
            .layered_history
            .execute(&mut state.layered_scene, SceneCommand::PlaceObject(object))
        {
            Ok(()) => {
                state.selected_world = pos;
                state.selection_active = true;
                state.selected_object_id = Some(id);
                state.message = format!("Placed one source-exact complete object on {:?} at foot ({}, {}). Use Select to drag, Object Properties to relocate, or Delete to remove. No asset boundary or collision certified.", state.active_layer, pos[0], pos[1]);
            }
            Err(error) => state.message = format!("Complete-object placement refused: {error}"),
        }
        return;
    }
    let operation = SceneCommand::PaintCells {
        layer: state.active_layer,
        cells: vec![PaintCell {
            index,
            source: Some(binding),
        }],
    };
    match state
        .layered_history
        .execute(&mut state.layered_scene, operation)
    {
        Ok(()) => {
            state.selected_object_id = None;
            state.selected_world = pos;
            state.selection_active = true;
            state.message = format!("Source-exact draft on {:?} at ({}, {}). Underlying original scene remains intact; save derived v2 draft.", state.active_layer, pos[0], pos[1]);
        }
        Err(error) => state.message = format!("Layer edit refused: {error}"),
    }
}

fn erase_source(state: &mut Studio, pos: [usize; 2]) {
    if !state.mapping_lab.sample_zone_contains(pos) {
        return;
    }
    // An object on this layer is its own instance; erasing it never masks the ground.
    if is_complete_object_layer(state.active_layer) {
        if let Some(object) = state
            .layered_scene
            .object_at(pos)
            .filter(|object| object.layer == state.active_layer)
        {
            state.selected_object_id = Some(object.id.clone());
            remove_selected_object(state);
            return;
        }
    }
    let Some(index) = state.scene.index(pos[0], pos[1]) else {
        return;
    };
    let command = SceneCommand::PaintCells {
        layer: state.active_layer,
        cells: vec![PaintCell {
            index,
            source: None,
        }],
    };
    match state
        .layered_history
        .execute(&mut state.layered_scene, command)
    {
        Ok(()) => {
            state.selected_object_id = None;
            state.selected_world = pos;
            state.selection_active = true;
            state.message = format!("Cleared {:?} visual layer only at ({}, {}). Original source and other layers preserved.", state.active_layer, pos[0], pos[1]);
        }
        Err(error) => state.message = format!("Layer erase refused: {error}"),
    }
}

fn selected_scene_source(state: &Studio, pos: [usize; 2]) -> Option<(String, [u32; 4])> {
    if let Some(object) = selected_object(state) {
        if object.layer == state.active_layer && object.contains_cell(pos) {
            let source = &object.parts.first()?.source;
            return Some((source.source_asset.clone(), source.source_rect));
        }
    }
    let index = state.scene.index(pos[0], pos[1])?;
    if state.active_layer != LayerId::Ground {
        let layer = state
            .layered_scene
            .visual_layers
            .iter()
            .find(|layer| layer.id == state.active_layer)?;
        return layer
            .edits
            .iter()
            .find(|edit| edit.index == index)
            .and_then(|edit| edit.source.as_ref())
            .map(|source| (source.source_asset.clone(), source.source_rect));
    }
    let tile = state.layered_scene.resolved_legacy_tile(index)?;
    (!tile.is_empty()).then_some((tile.source_asset, tile.source_rect))
}

fn sample_canvas_source(state: &mut Studio, pos: [usize; 2]) {
    let Some((path, rect)) = selected_scene_source(state, pos) else {
        state.message = "No source artwork at this layer/cell.".into();
        return;
    };
    if rect[0] % 32 != 0 || rect[1] % 32 != 0 {
        state.message = "This source region is not 32px aligned; inspect the stored exact pixel rectangle instead of guessing a new crop.".into();
        return;
    }
    if let Some(sheet) = state.sheets.iter().position(|item| item.path == path) {
        state.active_sheet = sheet;
        state.selected_cell = [rect[0] / 32, rect[1] / 32];
        state.selected_extent = [rect[2].div_ceil(32), rect[3].div_ceil(32)];
        state.message = format!(
            "Selected exact original source {path} region {rect:?} at ({}, {}).",
            pos[0], pos[1]
        );
    } else {
        state.pending_source_sheet = Some(path);
        state.message = "Loading selected original image on demand. Re-select its source cell to inspect the exact address.".into();
    }
}

fn checkpoint_world(state: &mut Studio) {
    state.world_undo.push(state.world_doc.clone());
    if state.world_undo.len() > 32 {
        state.world_undo.remove(0);
    }
    state.world_redo.clear();
}

fn materialize_world_with_current_authorities(
    state: &mut Studio,
    center: [i64; 2],
    radius: i32,
) -> Result<(), String> {
    let (world_doc, asset_authority, tiled_terrain) = (
        &mut state.world_doc,
        &state.asset_authority,
        state.tiled_terrain.as_ref(),
    );
    world_doc.materialize_with_tiled(center, radius, asset_authority, tiled_terrain)
}

fn save_world_document(state: &mut Studio) {
    if state.world_doc.has_pending_semantic_edits() {
        state.world_doc.commit_edit_batch();
    }
    if let Err(error) = state
        .world_doc
        .validate_against_authority(&state.asset_authority)
    {
        state.message = format!("Save world refused by unified ElizaWy authority: {error}");
        return;
    }
    state.message = match state.world_doc.save(&state.world_path) {
        Ok(()) => {
            state.world_dirty = false;
            format!(
                "Saved world composition revision {}. ElizaWy source artwork unchanged.",
                state.world_doc.revision
            )
        }
        Err(error) => format!("Save world composition failed; previous file preserved: {error}"),
    };
}

fn save_scene(state: &mut Studio) {
    if state.room_authoring.is_some() {
        save_house_room_editor(state);
        return;
    }
    if state.world_mode {
        if !state.world_dirty {
            state.message =
                "World composition already saved; ElizaWy source artwork unchanged.".into();
            return;
        }
        save_world_document(state);
        return;
    }
    if !state.layered_history.is_dirty() && !state.scene_needs_initial_save {
        state.message =
            "Current Summer scene already saved; original ElizaWy assets unchanged.".into();
        return;
    }
    state.message = match state
        .layered_scene
        .save_editable_draft(&state.legacy_base_path, &state.layered_path)
    {
        Ok(()) => {
            state.layered_history.mark_saved();
            state.scene_needs_initial_save = false;
            "Saved source-exact layered Summer scene corrections. Original v1 fixture, source art and recipe authority remain unchanged.".into()
        }
        Err(error) => format!("Save layered draft failed; previous file preserved: {error}"),
    };
}

fn undo_scene_edit(state: &mut Studio) {
    if state.world_mode {
        if let Some(previous) = state.world_undo.pop() {
            state.world_redo.push(state.world_doc.clone());
            state.world_doc = previous;
            state.world_dirty = true;
            state.message = "Undid world composition transaction".into();
        }
        return;
    }
    if state.layered_history.can_undo() {
        state.layered_history.undo(&mut state.layered_scene);
        state.message = "Undid layered scene transaction".into();
    }
}

fn redo_scene_edit(state: &mut Studio) {
    if state.world_mode {
        if let Some(next) = state.world_redo.pop() {
            state.world_undo.push(state.world_doc.clone());
            state.world_doc = next;
            state.world_dirty = true;
            state.message = "Redid world composition transaction".into();
        }
        return;
    }
    if state.layered_history.can_redo() {
        state.layered_history.redo(&mut state.layered_scene);
        state.message = "Redid layered scene transaction".into();
    }
}

fn draw_pie_character(
    painter: &egui::Painter,
    state: &Studio,
    center: egui::Pos2,
    px: f32,
    direction_row: u32,
    elapsed: f32,
    moving: bool,
) {
    let Some(&sheet_index) = state.sheet_lookup.get(PLAYER_WALK_SHEET) else {
        return;
    };
    let sheet = &state.sheets[sheet_index];
    let Some(texture) = sheet.texture else {
        return;
    };
    if sheet.size != [512, 256] {
        return;
    }
    let frame = if moving {
        ((elapsed * 8.0) as u32) % 8
    } else {
        0
    };
    let target = egui::Rect::from_min_size(
        egui::pos2(center.x - px, center.y - 1.75 * px),
        egui::vec2(2.0 * px, 2.0 * px),
    );
    painter.image(
        texture,
        target,
        source_uv([frame * 64, direction_row * 64, 64, 64], sheet.size),
        egui::Color32::WHITE,
    );
}

fn start_playtest(state: &mut Studio) {
    if state.room_authoring.is_some() {
        state.message = "Save & Return to World before starting PIE; unsaved room edits are never played implicitly.".into();
        return;
    }
    if state.playtest.is_some() || state.world_playtest.is_some() {
        return;
    }
    if state.world_mode {
        match WorldPlaySession::start(
            &state.world_doc,
            &state.asset_authority,
            state.world_selected,
        ) {
            Ok(play) => {
                state.saved_editor_view = Some((state.zoom, state.pan));
                state.zoom = state.zoom.max(0.9).min(1.7);
                state.world_playtest = Some(play);
                state.message = "World PIE active: source-backed terrain, authored traversability, collision and world objects frozen in a snapshot. Character uses hydrated LPC walking source if available. W/A/S/D or arrows; Esc to stop. F enters reachable house/cave objects; F8 opens a diagnostic cave. Room visuals are collision previews, not authored final art. Streaming remains pending.".into();
            }
            Err(error) => state.message = format!("World PIE not started: {error}"),
        }
        return;
    }
    let suggested = if state.selection_active {
        state.selected_world
    } else {
        [state.scene.size[0] / 3, state.scene.size[1] / 2]
    };
    let mut source_exact_snapshot = state.scene.clone();
    for index in 0..source_exact_snapshot.tiles.len() {
        if let Some(tile) = state.layered_scene.resolved_legacy_tile(index) {
            source_exact_snapshot.tiles[index] = tile;
        }
    }
    match PlaySession::start_with_collision(
        &source_exact_snapshot,
        suggested,
        &state.layered_scene.structural_cells,
        &state.layered_scene.collision_cells,
    ) {
        Ok(play) => {
            state.playtest_layered = Some(state.layered_scene.clone());
            state.saved_editor_view = Some((state.zoom, state.pan));
            state.zoom = state.zoom.max(0.8).min(1.25);
            state.dragging = None;
            state.dragging_object = None;
            state.drag_preview_origin = None;
            state.playtest = Some(play);
            state.message = "PIE: active Summer scene snapshot, including current object visuals. W/A/S/D or arrows, Esc/Stop exits. Explicit authored traversal overrides are now honored in PIE; all other terrain-role collision is provisional. Tree/house footprints are not certified.".into();
        }
        Err(error) => state.message = format!("PIE not started: {error}"),
    }
}

fn stop_playtest(state: &mut Studio) {
    if state.world_playtest.take().is_some() {
        if let Some((zoom, pan)) = state.saved_editor_view.take() {
            state.zoom = zoom;
            state.pan = pan;
        }
        state.message = "World PIE ended. World and authoring changes remain untouched.".into();
        return;
    }
    if state.playtest.take().is_some() {
        state.playtest_layered = None;
        if let Some((zoom, pan)) = state.saved_editor_view.take() {
            state.zoom = zoom;
            state.pan = pan;
        }
        state.message = "PIE stopped. Editor scene and unsaved corrections retained unchanged; no gameplay state was written to the source or derived draft.".into();
    }
}

fn draw_alpha_checker(
    painter: &egui::Painter,
    state: &Studio,
    scene: egui::Rect,
    px: f32,
    canvas: egui::Rect,
) {
    if !state.editor_layout.inspect_alpha || state.playtest.is_some() {
        return;
    }
    let [x, y] = state.selected_world;
    if state.scene.index(x, y).is_none() {
        return;
    }
    let start = scene.min + egui::vec2(x as f32 * px, y as f32 * px);
    let half = px * 0.5;
    for row in 0..2 {
        for col in 0..2 {
            let rect = egui::Rect::from_min_size(
                start + egui::vec2(col as f32 * half, row as f32 * half),
                egui::Vec2::splat(half),
            );
            if rect.intersects(canvas) {
                let shade = if (row + col) % 2 == 0 { 58 } else { 35 };
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(shade));
            }
        }
    }
}

fn focus_mapping_zone(state: &mut Studio, origin: [usize; 2], size: [usize; 2]) {
    let canvas = state.canvas_rect;
    let available = if canvas.width() > 0.0 && canvas.height() > 0.0 {
        canvas.size()
    } else {
        egui::vec2(1000.0, 750.0)
    };
    let pad = egui::vec2(72.0, 90.0);
    let zoom = ((available.x - pad.x).max(200.0) / (size[0] as f32 * 32.0))
        .min((available.y - pad.y).max(160.0) / (size[1] as f32 * 32.0))
        .clamp(0.125, 3.0);
    let center = [
        origin[0] as f32 + size[0] as f32 * 0.5,
        origin[1] as f32 + size[1] as f32 * 0.5,
    ];
    state.zoom = zoom;
    state.pan = egui::vec2(
        (state.scene.size[0] as f32 * 0.5 - center[0]) * 32.0 * zoom,
        (state.scene.size[1] as f32 * 0.5 - center[1]) * 32.0 * zoom,
    );
    state.selection_active = false;
}

fn draw_assembled_objects(
    painter: &egui::Painter,
    state: &Studio,
    scene_rect: egui::Rect,
    px: f32,
    canvas: egui::Rect,
) {
    for visual in &state.mapping_lab.visual_samples {
        if state.scene.hidden_visual_samples.contains(&visual.id) {
            continue;
        }
        let rect = egui::Rect::from_min_size(
            scene_rect.min + egui::vec2(visual.origin[0] as f32 * px, visual.origin[1] as f32 * px),
            egui::vec2(
                visual.source_rect[2] as f32 * px / 32.0,
                visual.source_rect[3] as f32 * px / 32.0,
            ),
        );
        if !rect.intersects(canvas) {
            continue;
        }
        let Some(sheet) = state.sheets.iter().find(|s| s.path == visual.source_asset) else {
            continue;
        };
        if let Some(texture) = sheet.texture {
            if visual.source_rect[0] + visual.source_rect[2] <= sheet.size[0]
                && visual.source_rect[1] + visual.source_rect[3] <= sheet.size[1]
            {
                painter.image(
                    texture,
                    rect,
                    source_uv(visual.source_rect, sheet.size),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}

fn hide_selected_visual_sample(state: &mut Studio, pos: [usize; 2]) -> bool {
    let Some((id, label)) = state
        .mapping_lab
        .visual_at(pos, &state.scene)
        .map(|sample| (sample.id.clone(), sample.label.clone()))
    else {
        return false;
    };
    state.undo.push(state.scene.clone());
    if state.undo.len() > 64 {
        state.undo.remove(0);
    }
    state.redo.clear();
    state.scene.hidden_visual_samples.push(id);
    state.dirty = true;
    state.message = format!("Removed {label} instance from scene. Undo restores it; Save retains removal. Original source art unchanged.");
    true
}

fn locate_selected_visual_sample(state: &mut Studio, pos: [usize; 2]) -> bool {
    let Some((path, rect, label)) = state.mapping_lab.visual_at(pos, &state.scene).map(|item| {
        (
            item.source_asset.clone(),
            item.source_rect,
            item.label.clone(),
        )
    }) else {
        return false;
    };
    if let Some(sheet) = state.sheets.iter().position(|s| s.path == path) {
        state.active_sheet = sheet;
        state.selected_cell = [rect[0] / 32, rect[1] / 32];

        state.editor_layout.terrain_mapper_tab = "source".into();
        state.editor_layout.terrain_mapper.visible = true;
        state.message = format!(
            "{label}: original {path} @ {},{} size {}x{}. Visual sample only; no recipe certified.",
            rect[0], rect[1], rect[2], rect[3]
        );
        return true;
    }
    false
}

// Each adjoining cell must use the same quantized edge at the current display scale.
// Snapping every TILE WIDTH individually causes accumulated hairline seams at fractional zoom.
fn snapped_scene_edge(
    scene_origin: f32,
    cell_coordinate: f32,
    tile_screen_size: f32,
    pixels_per_point: f32,
) -> f32 {
    ((scene_origin + cell_coordinate * tile_screen_size) * pixels_per_point).round()
        / pixels_per_point
}

fn draw_source_exact_visual_layers(
    painter: &egui::Painter,
    state: &Studio,
    scene_rect: egui::Rect,
    px: f32,
    canvas: egui::Rect,
    ppp: f32,
) {
    let doc = state
        .playtest_layered
        .as_ref()
        .unwrap_or(&state.layered_scene);
    for layer in &doc.visual_layers {
        if layer.id == LayerId::Ground {
            continue;
        } // already resolved into base ground
        if !world_layer_visible(state, layer.id) {
            continue;
        }
        for edit in &layer.edits {
            let Some(source) = &edit.source else {
                continue;
            };
            let Some(sheet_index) = state
                .sheet_lookup
                .get(source.source_asset.as_str())
                .copied()
            else {
                continue;
            };
            let sheet = &state.sheets[sheet_index];
            let Some(texture) = sheet.texture else {
                continue;
            };
            let x = edit.index % doc.size[0];
            let y = edit.index / doc.size[0];
            let [_, _, w, h] = source.source_rect;
            let target = egui::Rect::from_min_max(
                egui::pos2(
                    snapped_scene_edge(scene_rect.min.x, x as f32, px, ppp),
                    snapped_scene_edge(scene_rect.min.y, y as f32, px, ppp),
                ),
                egui::pos2(
                    snapped_scene_edge(scene_rect.min.x, x as f32 + w as f32 / 32.0, px, ppp),
                    snapped_scene_edge(scene_rect.min.y, y as f32 + h as f32 / 32.0, px, ppp),
                ),
            );
            if target.intersects(canvas) {
                painter.image(
                    texture,
                    target,
                    source_uv(source.source_rect, sheet.size),
                    egui::Color32::WHITE,
                );
            }
        }
        // Complete v2 object instances are explicit manual source addresses and
        // whole-object transforms. No source region, alpha mask or collision is invented.
        for object in doc.objects.iter().filter(|item| item.layer == layer.id) {
            for part in &object.parts {
                let source = &part.source;
                let Some(sheet_index) = state
                    .sheet_lookup
                    .get(source.source_asset.as_str())
                    .copied()
                else {
                    continue;
                };
                let sheet = &state.sheets[sheet_index];
                let Some(texture) = sheet.texture else {
                    continue;
                };
                let preview = if state.playtest.is_none()
                    && state
                        .dragging_object
                        .as_ref()
                        .is_some_and(|(id, _)| id == &object.id)
                {
                    state.drag_preview_origin.unwrap_or(object.anchor)
                } else {
                    object.anchor
                };
                let x = preview[0] as f32 + part.offset[0] as f32 / 32.0;
                let y = preview[1] as f32 + part.offset[1] as f32 / 32.0;
                let [_, _, w, h] = source.source_rect;
                let target = egui::Rect::from_min_max(
                    egui::pos2(
                        snapped_scene_edge(scene_rect.min.x, x, px, ppp),
                        snapped_scene_edge(scene_rect.min.y, y, px, ppp),
                    ),
                    egui::pos2(
                        snapped_scene_edge(scene_rect.min.x, x + w as f32 / 32.0, px, ppp),
                        snapped_scene_edge(scene_rect.min.y, y + h as f32 / 32.0, px, ppp),
                    ),
                );
                if target.intersects(canvas) {
                    painter.image(
                        texture,
                        target,
                        source_uv_object(&source.source_asset, source.source_rect, sheet.size),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
    }
}

fn world_object_at(state: &Studio, world: [i64; 2]) -> Option<String> {
    let point = [world[0] as f32 + 0.5, world[1] as f32 + 0.5];
    let mut candidates = state
        .world_doc
        .generated_objects()
        .filter(|object| world_layer_visible(state, world_layer_id(&object.layer)))
        .filter(|object| {
            let anchor = state.world_doc.object_anchor(object);
            let [min_x, min_y, max_x, max_y] = world_object_visual_bounds(object, anchor);
            point[0] >= min_x && point[0] < max_x && point[1] >= min_y && point[1] < max_y
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        let layer_a = world_layer_slot(world_layer_id(&a.layer));
        let layer_b = world_layer_slot(world_layer_id(&b.layer));
        (
            layer_a,
            world_object_depth_key(a, state.world_doc.object_anchor(a)),
        )
            .cmp(&(
                layer_b,
                world_object_depth_key(b, state.world_doc.object_anchor(b)),
            ))
    });
    candidates.last().map(|object| object.id.clone())
}

fn world_automatic_blocked(state: &Studio, world: [i64; 2]) -> bool {
    matches!(
        state
            .world_doc
            .effective_traversal(world, &state.asset_authority),
        TraversalMode::Blocked
    )
}

fn world_pointer_cell(
    state: &Studio,
    pointer: egui::Pos2,
    region: egui::Rect,
    origin: [i64; 2],
    size: [usize; 2],
    px: f32,
) -> Option<([i64; 2], [usize; 2])> {
    if !state.canvas_rect.contains(pointer)
        || !region.contains(pointer)
        || canvas_rulers::blocks_scene_edit(
            state.canvas_rect,
            pointer,
            canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
        )
        || state.canvas_controls_rect.contains(pointer)
        || state.canvas_grid_rect.contains(pointer)
        || state.canvas_layers_rect.contains(pointer)
        || state.canvas_animation_rect.contains(pointer)
        || (state.editor_layout.terrain_mapper.visible && state.atlas_rect.contains(pointer))
        || (state.editor_layout.structural_open && state.structural_rect.contains(pointer))
        || (state.editor_layout.world_generator_open
            && state.world_generator_rect.contains(pointer))
        || (state.launcher_open && state.launcher_rect.contains(pointer))
        || px <= 0.0
    {
        return None;
    }
    let lx = ((pointer.x - region.left()) / px).floor() as isize;
    let ly = ((pointer.y - region.top()) / px).floor() as isize;
    if lx < 0 || ly < 0 || lx as usize >= size[0] || ly as usize >= size[1] {
        return None;
    }
    let fx = ((pointer.x - (region.left() + lx as f32 * px)) / px).clamp(0.0, 0.99999);
    let fy = ((pointer.y - (region.top() + ly as f32 * px)) / px).clamp(0.0, 0.99999);
    Some((
        [origin[0] + lx as i64, origin[1] + ly as i64],
        [
            (fx * COLLISION_MASK_SIDE as f32).floor() as usize,
            (fy * COLLISION_MASK_SIDE as f32).floor() as usize,
        ],
    ))
}

fn selected_world_binding(state: &Studio) -> Result<SourceBinding, String> {
    let sheet = state
        .sheets
        .get(state.active_sheet)
        .ok_or_else(|| "No active source sheet is loaded".to_owned())?;
    let binding = SourceBinding {
        source_asset: sheet.path.clone(),
        source_rect: selected_source_rect(state),
    };
    if !sheet.path.starts_with("Terrain/") {
        return Err("World paint currently accepts Terrain source regions only.".into());
    }
    if !state.asset_authority.contains_binding(&binding) {
        return Err(
            "World paint requires an exact canonical region from the unified ElizaWy lane.".into(),
        );
    }
    Ok(binding)
}

fn paint_world_local_preview(state: &mut Studio, world: [i64; 2], binding: &SourceBinding) {
    state.world_doc.set_visual_override(world, binding.clone());
    // Tile Brush is a visual placement, not a terrain semantic operation.
    // Keep any existing generated/painted terrain meaning and navigation intact.
    if state.brush_mode != BrushMode::Tile {
        let semantic = state
            .asset_authority
            .unique_fixture_role(binding)
            .map(str::to_owned);
        state.world_doc.set_semantic_override(world, semantic);
    }
    state.world_dirty = true;
}

fn world_stroke_begin(
    state: &mut Studio,
    world: [i64; 2],
    binding: Option<SourceBinding>,
    erase: bool,
    local_only: bool,
) {
    checkpoint_world(state);
    state.world_stroke_cells.clear();
    state.world_stroke_seen.clear();
    state.world_stroke_last_center = None;
    state.world_stroke_binding = binding;
    state.world_stroke_role = None;
    state.world_stroke_erase = erase;
    state.world_stroke_local_only = local_only;
    world_stroke_apply_cell(state, world);
}

fn world_semantic_stroke_begin(state: &mut Studio, world: [i64; 2], erase: bool) {
    checkpoint_world(state);
    state.world_stroke_cells.clear();
    state.world_stroke_seen.clear();
    state.world_stroke_last_center = None;
    state.world_stroke_binding = None;
    state.world_stroke_role = (!erase).then(|| state.world_terrain_brush.role().to_owned());
    state.world_stroke_erase = erase;
    state.world_stroke_local_only = true;
    world_stroke_apply_cell(state, world);
}

fn world_stroke_apply_cell(state: &mut Studio, world: [i64; 2]) {
    // One stroke, one transaction; brush radius expands each interpolated center
    // without changing the source grid, terrain recipe or selection identity.
    let radius = match state.brush_mode {
        BrushMode::Terrain => state.editor_layout.terrain_brush_radius_tiles,
        BrushMode::Tile => state.editor_layout.tile_brush_radius_tiles,
        BrushMode::Pixel => 1,
    }
    .clamp(1, 4) as i64
        - 1;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if let (Some(x), Some(y)) = (world[0].checked_add(dx), world[1].checked_add(dy)) {
                world_stroke_apply_single_cell(state, [x, y]);
            }
        }
    }
    state.world_stroke_last_center = Some(world);
    state.world_selected = world;
    state.selection_active = true;
}

fn world_stroke_apply_single_cell(state: &mut Studio, world: [i64; 2]) {
    if state.world_doc.generated_cell(world).is_none() || !state.world_stroke_seen.insert(world) {
        return;
    }
    if state.world_stroke_erase {
        state.world_doc.clear_authored_terrain_untracked(world);
        state.world_dirty = true;
    } else if let Some(role) = state.world_stroke_role.clone() {
        state
            .world_doc
            .paint_semantic_terrain_untracked(world, &role);
        state.world_dirty = true;
    } else if let Some(binding) = state.world_stroke_binding.clone() {
        // Exact-source drag/drop remains available as the advanced correction lane.
        // Normal Paint is semantic terrain paint and does not require Source Browser.
        paint_world_local_preview(state, world, &binding);
    }
    state.world_stroke_cells.push(world);
}

fn world_stroke_apply_segment(state: &mut Studio, to: [i64; 2]) {
    let Some(from) = state.world_stroke_last_center else {
        world_stroke_apply_cell(state, to);
        return;
    };
    let mut x0 = from[0];
    let mut y0 = from[1];
    let x1 = to[0];
    let y1 = to[1];
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        world_stroke_apply_cell(state, [x0, y0]);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let doubled = error * 2;
        if doubled >= dy {
            error += dy;
            x0 += sx;
        }
        if doubled <= dx {
            error += dx;
            y0 += sy;
        }
    }
}

fn world_stroke_finish(state: &mut Studio) {
    if state.world_stroke_cells.is_empty() {
        state.world_stroke_last_center = None;
        state.world_stroke_seen.clear();
        state.world_stroke_binding = None;
        state.world_stroke_role = None;
        return;
    }
    let cells = std::mem::take(&mut state.world_stroke_cells);
    state.world_stroke_last_center = None;
    let local_only = state.world_stroke_local_only;
    let erase = state.world_stroke_erase;
    let role = state.world_stroke_role.take();
    let binding = state.world_stroke_binding.take();
    state.world_stroke_erase = false;
    state.world_stroke_local_only = false;

    // Semantic world paint is the normal editor workflow. It persists meaning
    // (Grass / Dirt-Bank / Water), while the visible tile is resolved from
    // exact ElizaWy source regions using the local topology every frame.
    if erase || role.is_some() {
        state.world_doc.commit_edit_batch();
        state.world_stroke_seen.clear();
        state.world_dirty = true;
        state.message = if erase {
            format!(
                "Reverted {} authored terrain cell(s) to generated terrain; neighbouring autotiles updated immediately.",
                cells.len()
            )
        } else {
            format!(
                "Painted {} {} semantic terrain cell(s); exact-source autotiling updated the edited edge neighbourhood. Save World to persist.",
                cells.len(),
                role.as_deref().unwrap_or("terrain")
            )
        };
        return;
    }

    state.world_stroke_seen.clear();

    // Direct source placement is an advanced correction path. Preserve the guarded
    // PCG-learning behaviour here so a manually selected source can teach only a
    // topology-safe generator mapping. Shift keeps it location-specific.
    if local_only {
        state.message = format!(
            "Applied {} location-specific exact-source correction cell(s).",
            cells.len()
        );
        return;
    }

    let Some(binding) = binding else {
        state.world_dirty = true;
        state.message = format!("World stroke updated {} cell(s).", cells.len());
        return;
    };

    let mut learned = 0usize;
    let mut staged_local = 0usize;
    let mut last_reason = None;
    for world in &cells {
        match state.world_doc.learn_generation_correction(
            *world,
            binding.clone(),
            &state.asset_authority,
        ) {
            Ok(_) => learned += 1,
            Err(error) => {
                staged_local += 1;
                last_reason = Some(error);
            }
        }
    }

    if learned > 0 {
        match state
            .world_doc
            .refresh_materialized_generation(&state.asset_authority)
        {
            Ok(changed) => {
                state.world_dirty = true;
                state.message = if staged_local > 0 {
                    format!(
                        "Exact-source correction: {learned} topology-safe generator mapping(s), {staged_local} local correction(s), {changed} generated cell(s) refreshed. {}",
                        last_reason.as_deref().unwrap_or("Unsafe mappings stayed local.")
                    )
                } else {
                    format!(
                        "Exact-source correction learned {learned} topology-safe mapping(s); {changed} generated cell(s) refreshed."
                    )
                };
            }
            Err(error) => {
                if let Some(previous) = state.world_undo.pop() {
                    state.world_doc = previous;
                }
                state.world_dirty = true;
                state.message = format!(
                    "World correction regeneration failed and the stroke was rolled back: {error}"
                );
            }
        }
    } else {
        state.world_dirty = true;
        state.message = format!(
            "Painted {} exact-source local correction cell(s). {}",
            staged_local.max(cells.len()),
            last_reason.unwrap_or_else(|| "No unsafe generator rule was created.".into())
        );
    }
}

fn world_pointer_from_screen(state: &Studio, pointer: egui::Pos2) -> Option<[i64; 2]> {
    let (origin, size) = state.world_doc.materialized_bounds()?;
    let px = 32.0 * state.zoom;
    world_pointer_cell(state, pointer, state.scene_rect, origin, size, px).map(|(world, _)| world)
}

fn draw_world_collision_overlay(
    painter: &egui::Painter,
    state: &Studio,
    region: egui::Rect,
    canvas: egui::Rect,
    origin: [i64; 2],
    size: [usize; 2],
    px: f32,
) {
    if !state.editor_layout.collision_overlay || px < 6.0 {
        // Pixel collision remains authoritative at every zoom, but the editor does
        // not rasterize thousands of 32x32 masks in Fit World/far overview mode.
        return;
    }
    let visible = canvas.intersect(region);
    if visible.width() <= 0.0 || visible.height() <= 0.0 {
        return;
    }
    let x0 = (((visible.left() - region.left()) / px).floor() as isize).max(0) as usize;
    let y0 = (((visible.top() - region.top()) / px).floor() as isize).max(0) as usize;
    let x1 = ((((visible.right() - region.left()) / px).ceil() as usize) + 1).min(size[0]);
    let y1 = ((((visible.bottom() - region.top()) / px).ceil() as usize) + 1).min(size[1]);
    let p = painter.with_clip_rect(canvas);
    let coarse_red = egui::Color32::from_rgba_unmultiplied(112, 18, 24, 76);
    let exact_red = egui::Color32::from_rgba_unmultiplied(104, 14, 22, 112);
    for ly in y0..y1 {
        for lx in x0..x1 {
            let world = [origin[0] + lx as i64, origin[1] + ly as i64];
            let cell = egui::Rect::from_min_size(
                region.min + egui::vec2(lx as f32 * px, ly as f32 * px),
                egui::Vec2::splat(px),
            );
            if let Some(mask) = state
                .world_doc
                .effective_collision_mask(world, &state.asset_authority)
            {
                if mask.blocked_count() == 0 {
                    continue;
                }
                if px < 16.0 {
                    p.rect_filled(cell, 0.0, coarse_red);
                } else {
                    // Merge contiguous blocked pixels into horizontal runs. The old
                    // path emitted up to 1,024 rects per cell every frame.
                    let unit = px / COLLISION_MASK_SIDE as f32;
                    for row in 0..COLLISION_MASK_SIDE {
                        let mut run_start: Option<usize> = None;
                        for col in 0..=COLLISION_MASK_SIDE {
                            let blocked = col < COLLISION_MASK_SIDE && mask.blocked(col, row);
                            match (run_start, blocked) {
                                (None, true) => run_start = Some(col),
                                (Some(start), false) => {
                                    p.rect_filled(
                                        egui::Rect::from_min_max(
                                            cell.min
                                                + egui::vec2(
                                                    start as f32 * unit,
                                                    row as f32 * unit,
                                                ),
                                            cell.min
                                                + egui::vec2(
                                                    col as f32 * unit,
                                                    (row + 1) as f32 * unit,
                                                ),
                                        ),
                                        0.0,
                                        exact_red,
                                    );
                                    run_start = None;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } else if world_automatic_blocked(state, world) {
                p.rect_filled(
                    cell,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(92, 20, 26, 42),
                );
            }
        }
    }
}

/// Deterministic variants never change authored PNGs, templates or collision geometry.
/// Large trees may mirror left/right but never rotate upside-down. Only isolated
/// square ground-level foliage is eligible for quarter turns.
fn foliage_variation(object: &world_doc::WorldObject) -> (bool, usize) {
    let label = object.label.to_ascii_lowercase();
    let foliage = [
        "tree", "shrub", "foliage", "flower", "palm", "fern", "bush", "mushroom", "grass",
        "clutter",
    ]
    .iter()
    .any(|word| label.contains(word));
    if !foliage || object.label.contains("river bridge") {
        return (false, 0);
    }
    let mut hash = 14695981039346656037u64;
    for byte in object.id.as_bytes() {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(1099511628211);
    }
    let mirror = hash & 1 != 0;
    let square_low_foliage = object.parts.len() == 1
        && object.parts[0].source.source_rect[2] == 32
        && object.parts[0].source.source_rect[3] == 32
        && ["flower", "grass", "clutter", "fern", "shrub", "mushroom"]
            .iter()
            .any(|word| label.contains(word));
    let quarter_turns = if square_low_foliage {
        ((hash >> 8) & 3) as usize
    } else {
        0
    };
    (mirror, quarter_turns)
}

fn draw_foliage_uv_variant(
    painter: &egui::Painter,
    texture: egui::TextureId,
    target: egui::Rect,
    uv: egui::Rect,
    mirror: bool,
    turns: usize,
) {
    if !mirror && turns == 0 {
        painter.image(texture, target, uv, egui::Color32::WHITE);
        return;
    }
    let source = [
        uv.min,
        egui::pos2(uv.max.x, uv.min.y),
        uv.max,
        egui::pos2(uv.min.x, uv.max.y),
    ];
    let corners = [
        target.min,
        egui::pos2(target.max.x, target.min.y),
        target.max,
        egui::pos2(target.min.x, target.max.y),
    ];
    let mut mesh = egui::Mesh::with_texture(texture);
    for i in 0..4 {
        let flipped_index = if mirror { [1, 0, 3, 2][i] } else { i };
        let uv_index = (flipped_index + 4 - (turns & 3)) % 4;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: corners[i],
            uv: source[uv_index],
            color: egui::Color32::WHITE,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_world_generated_objects(
    painter: &egui::Painter,
    state: &Studio,
    canvas: egui::Rect,
    region: egui::Rect,
    origin: [i64; 2],
    visible_world_min: [i64; 2],
    visible_world_max: [i64; 2],
    px: f32,
) {
    // Authored position overrides can move a generator-owned instance across a
    // chunk boundary. Cull by effective visual position instead of the original
    // owning chunk; otherwise the object vanishes after drag or camera pan.
    let mut objects = Vec::new();
    for object in state.world_doc.generated_objects() {
        let layer = world_layer_id(&object.layer);
        if !world_layer_visible(state, layer) {
            continue;
        }
        if !state.world_detail_lod_visible && world_object_is_minor(object) {
            continue;
        }
        let anchor = if state
            .world_dragging_object
            .as_ref()
            .is_some_and(|(id, _)| id == &object.id)
        {
            state
                .world_drag_preview_anchor
                .unwrap_or(state.world_doc.object_anchor(object))
        } else {
            state.world_doc.object_anchor(object)
        };
        let [min_x, min_y, max_x, max_y] = world_object_visual_bounds(object, anchor);
        if max_x < visible_world_min[0] as f32 - 1.0
            || max_y < visible_world_min[1] as f32 - 1.0
            || min_x > visible_world_max[0] as f32 + 1.0
            || min_y > visible_world_max[1] as f32 + 1.0
        {
            continue;
        }
        objects.push(object);
    }
    objects.sort_by(|a, b| {
        let layer_a = world_layer_slot(world_layer_id(&a.layer));
        let layer_b = world_layer_slot(world_layer_id(&b.layer));
        (
            layer_a,
            world_object_depth_key(a, state.world_doc.object_anchor(a)),
        )
            .cmp(&(
                layer_b,
                world_object_depth_key(b, state.world_doc.object_anchor(b)),
            ))
    });

    let pixels_per_point = painter.ctx().pixels_per_point().max(1.0);
    let snap_point = |value: f32| (value * pixels_per_point).round() / pixels_per_point;
    for object in objects {
        let (mirror, turns) = foliage_variation(object);
        let effective_anchor = if state
            .world_dragging_object
            .as_ref()
            .is_some_and(|(id, _)| id == &object.id)
        {
            state
                .world_drag_preview_anchor
                .unwrap_or(state.world_doc.object_anchor(object))
        } else {
            state.world_doc.object_anchor(object)
        };
        let [visual_left, _, visual_right, _] =
            world_object_visual_bounds(object, effective_anchor);
        for part in &object.parts {
            let Some(sheet_index) = state
                .sheet_lookup
                .get(part.source.source_asset.as_str())
                .copied()
            else {
                continue;
            };
            let sheet = &state.sheets[sheet_index];
            let Some(texture) = sheet.texture else {
                continue;
            };
            let anchor = if state
                .world_dragging_object
                .as_ref()
                .is_some_and(|(id, _)| id == &object.id)
            {
                state
                    .world_drag_preview_anchor
                    .unwrap_or(state.world_doc.object_anchor(object))
            } else {
                state.world_doc.object_anchor(object)
            };
            let local_x = anchor[0] as f32 + part.offset_px[0] as f32 / 32.0;
            let width_cells = part.source.source_rect[2] as f32 / 32.0;
            let x = (if mirror {
                visual_left + visual_right - local_x - width_cells
            } else {
                local_x
            }) - origin[0] as f32;
            let y = (anchor[1] - origin[1]) as f32 + part.offset_px[1] as f32 / 32.0;
            let min = region.min + egui::vec2(x * px, y * px);
            let size_px = egui::vec2(
                part.source.source_rect[2] as f32 * px / 32.0,
                part.source.source_rect[3] as f32 * px / 32.0,
            );
            let target = egui::Rect::from_min_max(
                egui::pos2(snap_point(min.x), snap_point(min.y)),
                egui::pos2(snap_point(min.x + size_px.x), snap_point(min.y + size_px.y)),
            );
            if target.intersects(canvas) {
                draw_foliage_uv_variant(
                    &painter,
                    texture,
                    target,
                    source_uv_object(
                        &part.source.source_asset,
                        part.source.source_rect,
                        sheet.size,
                    ),
                    mirror,
                    turns,
                );
            }
        }
    }
}

#[cfg(test)]
mod unified_canvas_zoom_tests {
    use super::*;
    #[test]
    fn world_and_scene_zoom_anchor_math_preserves_source_position() {
        let rect = egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(720.0, 480.0));
        let pointer = egui::pos2(290.0, 180.0);
        let center = rect.center();
        let pan = egui::vec2(78.0, -36.0);
        let old_zoom: f32 = 0.5;
        let next_zoom: f32 = 1.25;
        for anchor in [pointer, center] {
            let old_local = (anchor - center - pan) / old_zoom;
            let ratio = next_zoom / old_zoom;
            let new_pan = pan * ratio + (anchor - center) * (1.0 - ratio);
            let new_local = (anchor - center - new_pan) / next_zoom;
            assert!((new_local - old_local).length() < 0.001);
        }
    }
}

/// Preserve the world coordinate below `anchor` across zoom changes.
/// The canvas-center button anchor and cursor wheel anchor intentionally differ.
fn zoom_canvas_at(state: &mut Studio, anchor: egui::Pos2, canvas: egui::Rect, requested: f32) {
    let old_zoom = state.zoom;
    let next_zoom = requested.clamp(0.125, 16.0);
    if old_zoom <= 0.0 || (next_zoom - old_zoom).abs() <= f32::EPSILON {
        return;
    }
    let ratio = next_zoom / old_zoom;
    state.pan = state.pan * ratio + (anchor - canvas.center()) * (1.0 - ratio);
    state.zoom = next_zoom;
}

fn zoom_canvas_by(state: &mut Studio, anchor: egui::Pos2, canvas: egui::Rect, factor: f32) {
    let requested = state.zoom * factor;
    zoom_canvas_at(state, anchor, canvas, requested);
}

/// In-PIE room viewport. This is deliberately a topology/collision proof view:
/// neutral diagnostics are not presented as certified ElizaWy floors or walls.
/// The authoring system must bind original floor and wall art before production.
fn draw_world_pie_room(ui: &mut egui::Ui, state: &Studio, canvas: egui::Rect) {
    let Some(play) = state.world_playtest.as_ref() else {
        return;
    };
    let Some(active) = play.interior.as_ref() else {
        return;
    };
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(15, 20, 22));
    let px = 32.0 * state.zoom.clamp(0.65, 2.0);
    let origin = canvas.center() - egui::vec2(active.position[0] * px, active.position[1] * px);
    let dimensions = active.scene.dimensions();
    let min_x = (((canvas.left() - origin.x) / px).floor() as i32).max(0);
    let min_y = (((canvas.top() - origin.y) / px).floor() as i32).max(0);
    let max_x = (((canvas.right() - origin.x) / px).ceil() as i32).min(dimensions[0] as i32);
    let max_y = (((canvas.bottom() - origin.y) / px).ceil() as i32).min(dimensions[1] as i32);
    for y in min_y..max_y {
        for x in min_x..max_x {
            let Some(index) = active.scene.index([x as u32, y as u32]) else {
                continue;
            };
            let cell = active.scene.cells[index];
            let tile_rect = egui::Rect::from_min_size(
                origin + egui::vec2(x as f32 * px, y as f32 * px),
                egui::Vec2::splat(px),
            );
            if cell.floor {
                let tone = if (x + y) % 2 == 0 { 66 } else { 71 };
                painter.rect_filled(
                    tile_rect,
                    0.0,
                    egui::Color32::from_rgb(tone, tone - 5, tone - 11),
                );
            } else if cell.wall_neighbor_mask != 0 {
                let tint = active.scene.style.wall_tint_rgba;
                painter.rect_filled(
                    tile_rect,
                    0.0,
                    egui::Color32::from_rgb(
                        (u16::from(tint[0]) / 3 + 25) as u8,
                        (u16::from(tint[1]) / 3 + 25) as u8,
                        (u16::from(tint[2]) / 3 + 25) as u8,
                    ),
                );
            }
        }
    }
    for portal in &active.scene.portals {
        let point = origin
            + egui::vec2(
                (portal.cell[0] as f32 + 0.5) * px,
                (portal.cell[1] as f32 + 0.5) * px,
            );
        if canvas.contains(point) {
            painter.circle_stroke(
                point,
                px * 0.33,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(245, 197, 80)),
            );
        }
    }
    draw_pie_character(
        &painter,
        state,
        canvas.center(),
        px,
        play.facing.lpc_walk_row(),
        play.elapsed,
        play.moving,
    );
    if !state.sheet_lookup.contains_key(PLAYER_WALK_SHEET) {
        painter.circle_stroke(
            canvas.center(),
            px * 0.25,
            egui::Stroke::new(2.0, egui::Color32::YELLOW),
        );
    }
    let kind = match active.scene.kind {
        room_scene::RoomKind::House => "HOUSE",
        room_scene::RoomKind::Cave => "CAVE",
    };
    let text = format!(
        "{kind} PIE · {} · {}x{} chunks · F at marked exit · Esc stop",
        active.scene.scene_id, active.scene.chunks[0], active.scene.chunks[1]
    );
    painter.text(
        canvas.min + egui::vec2(14.0, 18.0),
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::proportional(15.0),
        egui::Color32::WHITE,
    );
    painter.text(
        canvas.min + egui::vec2(14.0, 39.0),
        egui::Align2::LEFT_TOP,
        "GEOMETRY PREVIEW ONLY · room wall tint reflects the saved room · source art pending",
        egui::FontId::proportional(12.0),
        egui::Color32::LIGHT_GRAY,
    );
    ui.ctx().request_repaint();
}

fn draw_world_workspace(ui: &mut egui::Ui, state: &mut Studio) {
    let available = ui.available_size().max(egui::vec2(200.0, 160.0));
    let (canvas, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
    state.canvas_rect = canvas;
    state.canvas_animation_rect =
        if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some() {
            egui::Rect::NOTHING
        } else {
            canvas_animation_control_rect(
                canvas,
                canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
            )
        };
    state.canvas_layers_rect =
        if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some() {
            egui::Rect::NOTHING
        } else {
            canvas_layer_rect(
                canvas,
                state.editor_layout.layers_dock_open,
                canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
            )
        };
    let overlays = if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some()
    {
        canvas_chrome::OverlayGeometry {
            grid: egui::Rect::NOTHING,
            zoom: egui::Rect::NOTHING,
        }
    } else {
        canvas_chrome::overlay_geometry(
            canvas,
            if canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode) {
                canvas_rulers::TOP
            } else {
                0.0
            },
        )
    };
    state.canvas_controls_rect = overlays.zoom;
    state.canvas_grid_rect = overlays.grid;
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(20, 29, 27));
    if state
        .world_playtest
        .as_ref()
        .is_some_and(|play| play.interior.is_some())
    {
        draw_world_pie_room(ui, state, canvas);
        return;
    }
    let Some((origin, size)) = state.world_doc.materialized_bounds() else {
        painter.text(
            canvas.center(),
            egui::Align2::CENTER_CENTER,
            "No world chunks yet. HW → World Generator → Generate 3×3",
            egui::FontId::proportional(18.0),
            egui::Color32::LIGHT_GRAY,
        );
        return;
    };
    if response.hovered() {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if wheel.abs() > 0.01 {
            if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                zoom_canvas_by(state, pointer, canvas, (wheel / 240.0).exp());
            }
        }
    }
    // Right click is reserved for canvas context actions; only middle-drag pans.
    // Secondary-drag no longer steals object/terrain editing gestures.
    if state.world_playtest.is_none() && response.dragged_by(egui::PointerButton::Middle) {
        state.pan += response.drag_delta();
    }
    let px = 32.0 * state.zoom;
    // Stable detail LOD: do not flicker shrubs/foliage on a single zoom boundary.
    // Hide only at far-overview scale and require a wider threshold to restore.
    if state.world_detail_lod_visible && px <= 4.05 {
        state.world_detail_lod_visible = false;
    } else if !state.world_detail_lod_visible && px >= 5.5 {
        state.world_detail_lod_visible = true;
    }
    let region = egui::Rect::from_center_size(
        canvas.center() + state.pan,
        egui::vec2(size[0] as f32 * px, size[1] as f32 * px),
    );
    state.scene_rect = region;
    // Keep right-click placement coordinates independent of the popup cursor.
    if response.secondary_clicked() {
        if let Some((world, _)) = ui
            .ctx()
            .pointer_hover_pos()
            .and_then(|pointer| world_pointer_cell(state, pointer, region, origin, size, px))
        {
            // Opening a context menu must not mutate selection, active layer,
            // brush, or the current object's transform. A chosen action owns
            // any subsequent selection or world edit.
            state.world_context_cell = Some(world);
        }
    }
    response.context_menu(|ui| {
        let cell = state.world_context_cell.unwrap_or(state.world_selected);
        ui.strong(format!("World cell {}, {}", cell[0], cell[1]));
        if let Some(role) = state.world_doc.resolved_role(cell) {
            ui.small(format!("Terrain: {role}"));
        }
        if let Some(object_id) = world_object_at(state, cell) {
            let is_house = state.world_doc.generated_objects().any(|obj| obj.id == object_id && {
                let template = obj.template_id.to_ascii_lowercase();
                template.contains("house") || template.contains("building") || obj.label.to_ascii_lowercase().contains("house")
            });
            if is_house {
                ui.separator();
                ui.strong("House actions");
                if ui.button("Edit Interior").clicked() {
                    open_house_room_editor_for(state, &object_id);
                    ui.close();
                }
                if ui.button("Map Doorway → Scene").clicked() {
                    state.world_door_mapping = Some(object_id.clone());
                    state.tool = CanvasTool::Select;
                    state.message = "Click a walkable cell immediately outside the house edge to map its door. Escape cancels.".into();
                    ui.close();
                }
                if let Some(link) = state.world_doc.structure_doors.get(&object_id) {
                    ui.small(format!("Door: {},{} → {}", link.doorway_world[0], link.doorway_world[1], link.destination_scene));
                }
            }
        }
        ui.separator();
        if ui.button("Select object at cell").clicked() {
            state.world_selected = cell;
            state.selection_active = true;
            state.world_selected_object = world_object_at(state, cell);
            state.tool = CanvasTool::Select;
            ui.close();
        }
        if ui.button("Sample terrain into brush").clicked() {
            if let Some(role) = state.world_doc.resolved_role(cell) {
                if let Some(brush) = WorldTerrainBrush::from_role(&role) {
                    state.world_terrain_brush = brush;
                    state.active_layer = world_terrain_role_layer(brush.role());
                    state.tool = CanvasTool::Paint;
                    state.brush_mode = BrushMode::Terrain;
                }
            }
            ui.close();
        }
        if ui.button(format!("Paint {} here", state.world_terrain_brush.label())).clicked() {
            checkpoint_world(state);
            state.world_doc.paint_semantic_terrain(cell, state.world_terrain_brush.role());
            state.world_dirty = true;
            state.active_layer = world_terrain_role_layer(state.world_terrain_brush.role());
            state.message = "Painted one semantic terrain cell; neighboring source Wang tiles will re-resolve.".into();
            ui.close();
        }
        if ui.button("Erase authored terrain here").clicked() {
            checkpoint_world(state);
            state.world_doc.clear_authored_terrain(cell);
            state.world_dirty = true;
            state.message = "Cleared authored terrain override (original generated material restored).".into();
            ui.close();
        }
        ui.separator();
        ui.small("Middle-drag: pan · Wheel: zoom · Left-drag: active tool");
    });
    let visible = canvas.intersect(region);
    if visible.width() > 0.0 {
        // Semantic world cells occupy the square between four integer dual-grid
        // vertices. Render each resolved vertex tile centered on that vertex, not
        // with the vertex as its top-left corner. The previous top-left placement
        // shifted authored terrain down/right by half a cell and exposed square
        // ownership artifacts at tight shorelines.
        let local_left = (visible.left() - region.left()) / px;
        let local_top = (visible.top() - region.top()) / px;
        let local_right = (visible.right() - region.left()) / px;
        let local_bottom = (visible.bottom() - region.top()) / px;
        let x0 = (local_left.floor() as isize).max(0) as usize;
        let y0 = (local_top.floor() as isize).max(0) as usize;
        let x1 = (((local_right.ceil() as usize) + 1).min(size[0])).max(x0);
        let y1 = (((local_bottom.ceil() as usize) + 1).min(size[1])).max(y0);

        {
            let mut terrain_mesh: Option<(egui::TextureId, egui::Mesh)> = None;
            // Shared timeline preview drives the existing Tiled/source frame resolver.
            let elapsed_ms = state.animation_preview_ms.round() as u64;
            let mut animated_visible = false;
            let mut shoreline_fallback_markers: Vec<(egui::Rect, bool)> = Vec::new();
            prepare_world_visual_cache(state);
            let vx0 = ((local_left - 0.5).floor() as isize).max(0) as usize;
            let vy0 = ((local_top - 0.5).floor() as isize).max(0) as usize;
            let vx1 = (((local_right + 0.5).ceil() as usize) + 1).min(size[0] + 1);
            let vy1 = (((local_bottom + 0.5).ceil() as usize) + 1).min(size[1] + 1);
            for vy in vy0..vy1 {
                for vx in vx0..vx1 {
                    let vertex = [origin[0] + vx as i64, origin[1] + vy as i64];
                    let Some(cached) = cached_world_vertex_visual(state, vertex) else {
                        continue;
                    };
                    if !world_layer_visible(state, cached.layer) {
                        continue;
                    }
                    let vertex_tile_min_units = terrain::dual_grid_vertex_tile_min([vx, vy]);
                    if state.editor_layout.shoreline_audit_overlay && px >= 8.0 {
                        if let Some(tiled) = state.tiled_terrain.as_ref() {
                            let roles = state.world_doc.resolved_dual_grid_roles(vertex);
                            let rs = [
                                roles[0].as_str(),
                                roles[1].as_str(),
                                roles[2].as_str(),
                                roles[3].as_str(),
                            ];
                            if tiled
                                .resolve_havenwild_corner_tile_id(rs, state.world_doc.seed, vertex)
                                .is_none()
                            {
                                let projected =
                                    world_doc::WorldDocument::land_connectivity_projection(rs);
                                let can_project = tiled
                                    .resolve_havenwild_corner_tile_id(
                                        projected,
                                        state.world_doc.seed,
                                        vertex,
                                    )
                                    .is_some();
                                let min = egui::pos2(
                                    snapped_scene_edge(
                                        region.min.x,
                                        vertex_tile_min_units[0],
                                        px,
                                        painter.ctx().pixels_per_point(),
                                    ),
                                    snapped_scene_edge(
                                        region.min.y,
                                        vertex_tile_min_units[1],
                                        px,
                                        painter.ctx().pixels_per_point(),
                                    ),
                                );
                                shoreline_fallback_markers.push((
                                    egui::Rect::from_min_size(min, egui::Vec2::splat(px)),
                                    can_project,
                                ));
                            }
                        }
                    }
                    for part in cached.parts {
                        let (sheet_index, source_rect) = part
                            .animation
                            .as_ref()
                            .and_then(|animation| {
                                animated_visible = true;
                                animation.source_at(elapsed_ms)
                            })
                            .unwrap_or((part.sheet_index, part.source_rect));
                        let sheet = &state.sheets[sheet_index];
                        let Some(texture) = sheet.texture else {
                            continue;
                        };
                        let [dx, dy, dw, dh] = part.destination_rect_px;
                        // Adjacent dual-grid tiles must share identical rounded physical
                        // pixel boundaries; independent fractional-size rounding opened
                        // water/sand hairline seams during scroll/zoom.
                        let ox = vertex_tile_min_units[0] + dx as f32 / 32.0;
                        let oy = vertex_tile_min_units[1] + dy as f32 / 32.0;
                        let target = egui::Rect::from_min_max(
                            egui::pos2(
                                snapped_scene_edge(
                                    region.min.x,
                                    ox,
                                    px,
                                    painter.ctx().pixels_per_point(),
                                ),
                                snapped_scene_edge(
                                    region.min.y,
                                    oy,
                                    px,
                                    painter.ctx().pixels_per_point(),
                                ),
                            ),
                            egui::pos2(
                                snapped_scene_edge(
                                    region.min.x,
                                    ox + dw as f32 / 32.0,
                                    px,
                                    painter.ctx().pixels_per_point(),
                                ),
                                snapped_scene_edge(
                                    region.min.y,
                                    oy + dh as f32 / 32.0,
                                    px,
                                    painter.ctx().pixels_per_point(),
                                ),
                            ),
                        );
                        let needs_new_mesh =
                            terrain_mesh.as_ref().is_none_or(|(id, _)| *id != texture);
                        if needs_new_mesh {
                            if let Some((_, mesh)) = terrain_mesh.take() {
                                painter.add(egui::Shape::mesh(mesh));
                            }
                            terrain_mesh = Some((texture, egui::Mesh::with_texture(texture)));
                        }
                        if let Some((_, mesh)) = terrain_mesh.as_mut() {
                            mesh.add_rect_with_uv(
                                target,
                                source_uv(source_rect, sheet.size),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            }
            if let Some((_, mesh)) = terrain_mesh.take() {
                painter.add(egui::Shape::mesh(mesh));
            }
            for (rect, has_projected_recipe) in shoreline_fallback_markers {
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(
                        1.25,
                        if has_projected_recipe {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::LIGHT_RED
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
            }
            if animated_visible {
                // Authored TSX frame durations remain the timing authority; this
                // only guarantees that egui wakes often enough to display them.
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(33));
            }
        }

        // Exact-source local replacements are authored in semantic-cell space,
        // so draw them directly inside the selected 32x32 cell after the shared
        // dual-grid terrain. This keeps Direct Tile corrections pixel-aligned
        // without reintroducing the old vertex/cell coordinate conflation.
        {
            for ly in y0..y1 {
                for lx in x0..x1 {
                    let world = [origin[0] + lx as i64, origin[1] + ly as i64];
                    if state.world_doc.has_pending_semantic_edit(world) {
                        continue;
                    }
                    let Some(item) = state.world_doc.cell_override(world) else {
                        continue;
                    };
                    let role = item
                        .semantic
                        .as_deref()
                        .or_else(|| {
                            state
                                .world_doc
                                .generated_cell(world)
                                .map(|cell| cell.role.as_str())
                        })
                        .unwrap_or("Grass");
                    if !world_layer_visible(state, world_terrain_role_layer(role)) {
                        continue;
                    }
                    if !matches!(item.visual_mode, Some(VisualOverrideMode::Replace)) {
                        continue;
                    }
                    let Some(source) = item.source.as_ref() else {
                        continue;
                    };
                    let Some(sheet_index) = state
                        .sheet_lookup
                        .get(source.source_asset.as_str())
                        .copied()
                    else {
                        continue;
                    };
                    let sheet = &state.sheets[sheet_index];
                    let Some(texture) = sheet.texture else {
                        continue;
                    };
                    let target = egui::Rect::from_min_size(
                        region.min + egui::vec2(lx as f32 * px, ly as f32 * px),
                        egui::Vec2::splat(px),
                    );
                    painter.image(
                        texture,
                        target,
                        source_uv(source.source_rect, sheet.size),
                        egui::Color32::WHITE,
                    );
                }
            }
        }

        let visible_world_min = [origin[0] + x0 as i64, origin[1] + y0 as i64];
        let visible_world_max = [origin[0] + x1 as i64, origin[1] + y1 as i64];
        draw_world_generated_objects(
            &painter,
            state,
            canvas,
            region,
            origin,
            visible_world_min,
            visible_world_max,
            px,
        );
    }
    draw_world_collision_overlay(&painter, state, region, canvas, origin, size, px);
    // Mapped doorways are visible in the editor and PIE independently of
    // uncertified door artwork. These are non-picking geometry markers.
    for link in state.world_doc.structure_doors.values() {
        let center = region.min
            + egui::vec2(
                (link.doorway_world[0] as f32 - origin[0] as f32 + 0.5) * px,
                (link.doorway_world[1] as f32 - origin[1] as f32 + 0.5) * px,
            );
        if canvas.contains(center) {
            painter.rect_filled(
                egui::Rect::from_center_size(center, egui::Vec2::splat((px * 0.42).max(5.0))),
                2.0,
                egui::Color32::from_rgb(53, 181, 137),
            );
            if px > 13.0 {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    "D",
                    egui::FontId::proportional((px * 0.42).min(18.0)),
                    egui::Color32::BLACK,
                );
            }
        }
    }
    if let Some(play) = state.world_playtest.as_ref() {
        let center = region.min
            + egui::vec2(
                (play.position[0] - origin[0] as f32) * px,
                (play.position[1] - origin[1] as f32) * px,
            );
        draw_pie_character(
            &painter,
            state,
            center,
            px,
            play.facing.lpc_walk_row(),
            play.elapsed,
            play.moving,
        );
        egui::Area::new(egui::Id::new("havenwild.world.pie.status"))
            .fixed_pos(canvas.min + egui::vec2(12.0, 10.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::NONE.fill(egui::Color32::from_black_alpha(210)).show(ui, |ui| {
                    ui.strong("WORLD PIE · WASD / arrows · F = Enter door · F8 = Test cave · Esc = Stop");
                    if let Some((id, kind)) = play.nearby_entrance(&state.asset_authority) {
                        ui.colored_label(egui::Color32::LIGHT_GREEN,
                            format!("F: Enter {} ({:?})", id, kind));
                    }
                    ui.small(format!("World {:.1}, {:.1} · {} blocked steps · local snapshot", play.position[0], play.position[1], play.blocked_steps));
                    if !state.sheet_lookup.contains_key(PLAYER_WALK_SHEET) { ui.colored_label(egui::Color32::YELLOW, "LPC walking art not hydrated; character sprite unavailable"); }
                });
            });
        ui.ctx().request_repaint();
        return; // No editor paint, object drag, deletion or other authoring while playing.
    }

    if state.world_door_mapping.is_some() {
        painter.text(
            canvas.min + egui::vec2(14.0, 14.0),
            egui::Align2::LEFT_TOP,
            "MAP DOORWAY · Click a walkable tile bordering the selected house · Esc cancels",
            egui::FontId::proportional(14.0),
            egui::Color32::LIGHT_GREEN,
        );
    }
    world_chunks::draw_materialized(
        &painter,
        canvas,
        region,
        px,
        origin,
        size,
        state.world_doc.chunk_side,
        state.editor_layout.chunk_grid_visible,
        state.editor_layout.tile_grid_visible && px >= 12.0,
    );
    if state.selection_active {
        let local = [
            state.world_selected[0] - origin[0],
            state.world_selected[1] - origin[1],
        ];
        if local[0] >= 0 && local[1] >= 0 && local[0] < size[0] as i64 && local[1] < size[1] as i64
        {
            painter.rect_stroke(
                egui::Rect::from_min_size(
                    region.min + egui::vec2(local[0] as f32 * px, local[1] as f32 * px),
                    egui::Vec2::splat(px),
                ),
                0.0,
                egui::Stroke::new(1.5, egui::Color32::YELLOW),
                egui::StrokeKind::Inside,
            );
        }
    }
    let primary_pressed = ui.input(|input| input.pointer.primary_pressed());
    let primary_down = ui.input(|input| input.pointer.primary_down());
    let primary_released = ui.input(|input| input.pointer.primary_released());
    let pointer = ui.ctx().pointer_hover_pos();

    if primary_pressed && response.hovered() {
        if let Some(pointer) = pointer {
            if let Some((world, pixel)) =
                world_pointer_cell(state, pointer, region, origin, size, px)
            {
                state.world_selected = world;
                state.selection_active = true;
                if let Some(object_id) = state.world_door_mapping.clone() {
                    // Load only the target house's template/instance. The destination
                    // is the room's actual authored exit, not a guessed center pixel.
                    let path = project_root::resolve().join(
                        room_scene::house_instance_relative_path(state.world_doc.seed, &object_id),
                    );
                    let template_path =
                        project_root::resolve().join("content/rooms/starter_house.room.json");
                    let source_path = if path.is_file() {
                        &path
                    } else {
                        &template_path
                    };
                    let entry =
                        room_scene::RoomScene::load_template(source_path).and_then(|room| {
                            room.portals
                                .iter()
                                .find(|portal| portal.kind == room_scene::PortalKind::Exit)
                                .map(|portal| portal.cell)
                                .ok_or_else(|| "Destination room has no exit/entry portal".into())
                        });
                    match entry {
                        Ok(destination) => {
                            checkpoint_world(state);
                            match state.world_doc.map_structure_door(
                                &object_id,
                                world,
                                destination,
                                &state.asset_authority,
                            ) {
                                Ok(()) => {
                                    state.world_door_mapping = None;
                                    state.world_dirty = true;
                                    state.message = "Doorway mapped to this house's interior. Ctrl+S saves the world; right-click house → Edit Interior → Save Room before PIE.".into();
                                }
                                Err(error) => {
                                    let _ = state.world_undo.pop();
                                    state.message = format!("Doorway not mapped: {error}. Try another adjacent cell or Esc.");
                                }
                            }
                        }
                        Err(error) => state.message = format!("Doorway target is invalid: {error}"),
                    }
                } else if let Some(template_id) = state.world_template_placement.clone() {
                    checkpoint_world(state);
                    match state.world_doc.place_reviewed_template(
                        &template_id,
                        world,
                        &state.asset_authority,
                    ) {
                        Ok(id) => {
                            state.world_template_placement = None;
                            state.world_selected_object = Some(id);
                            state.world_dirty = true;
                            state.message = "Reviewed original sprite placed as an undoable visual object; collision/gameplay meaning remains unassigned. Ctrl+S saves World.".into();
                        }
                        Err(error) => {
                            let _ = state.world_undo.pop();
                            state.message = format!("Reviewed object placement refused: {error}");
                        }
                    }
                } else if state.world_bridge_tool && state.tool == CanvasTool::Select {
                    state.world_bridge_drag = Some((world, world));
                    state.world_selected_object = None;
                    state.world_dragging_object = None;
                    state.world_drag_preview_anchor = None;
                } else if state.collision_world_pixel_edit && state.editor_layout.structural_open {
                    checkpoint_world(state);
                    state.collision_stroke_active = true;
                    paint_world_collision_brush(state, world, pixel);
                } else {
                    match state.tool {
                        CanvasTool::Select => {
                            state.world_selected_object = world_object_at(state, world);
                            state.world_dragging_object = state.world_selected_object.as_ref().and_then(|id| {
                                let object = state.world_doc.generated_objects().find(|object| &object.id == id)?;
                                let anchor = state.world_doc.object_anchor(object);
                                Some((id.clone(), [world[0] - anchor[0], world[1] - anchor[1]]))
                            });
                            state.world_drag_preview_anchor = state.world_selected_object.as_ref().and_then(|id| {
                                state.world_doc.generated_objects().find(|object| &object.id == id)
                                    .map(|object| state.world_doc.object_anchor(object))
                            });
                        }
                        CanvasTool::Paint => {
                            match state.brush_mode {
                                BrushMode::Terrain => world_semantic_stroke_begin(state, world, false),
                                BrushMode::Tile => match selected_world_binding(state) {
                                    Ok(binding) if binding.source_rect[2] == 32 && binding.source_rect[3] == 32 =>
                                        world_stroke_begin(state, world, Some(binding), false, true),
                                    Ok(_) => state.message = "World Tile Brush requires one exact 32×32 source cell; multi-cell sources belong on an object layer.".into(),
                                    Err(error) => state.message = format!("Tile Brush: {error}"),
                                },
                                BrushMode::Pixel => state.message = "Pixel Brush cannot write to world tiles. Open a project-owned derived image document before editing pixels.".into(),
                            }
                        }
                        CanvasTool::Erase => {
                            if state.brush_mode == BrushMode::Pixel {
                                state.message = "Pixel erasing requires a project-owned derived image document; no world terrain changed.".into();
                            } else {
                                world_semantic_stroke_begin(state, world, true);
                            }
                        }
                        CanvasTool::Sample => {
                            if let Some(role) = state.world_doc.resolved_role(world) {
                                if let Some(brush) = WorldTerrainBrush::from_role(&role) {
                                    state.world_terrain_brush = brush;
                                    state.active_layer = world_terrain_role_layer(brush.role());
                                }
                            }
                            if let Some(source) = state.world_doc.resolved_source_with_authorities(
                                world,
                                &state.asset_authority,
                                state.tiled_terrain.as_ref(),
                            ) {
                                if let Some(index) = state
                                    .sheets
                                    .iter()
                                    .position(|sheet| sheet.path == source.source_asset)
                                {
                                    state.active_sheet = index;
                                    state.selected_cell =
                                        [source.source_rect[0] / 32, source.source_rect[1] / 32];
                                    state.selected_extent = [
                                        source.source_rect[2].div_ceil(32),
                                        source.source_rect[3].div_ceil(32),
                                    ];
                                } else {
                                    state.pending_source_sheet = Some(source.source_asset);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Generated-world object drag: anchored on initial click, previews at
    // integer cell positions and commits exactly one undoable scene correction.
    if state.tool == CanvasTool::Select && !state.collision_world_pixel_edit {
        if let Some((id, grab)) = state.world_dragging_object.clone() {
            if primary_down {
                if let Some((world, _)) =
                    pointer.and_then(|p| world_pointer_cell(state, p, region, origin, size, px))
                {
                    let to = [world[0] - grab[0], world[1] - grab[1]];
                    if state.world_doc.can_move_generated_object_to(&id, to) {
                        state.world_drag_preview_anchor = Some(to);
                    }
                }
                ui.ctx().request_repaint();
            }
            if primary_released {
                let destination = pointer
                    .and_then(|p| world_pointer_cell(state, p, region, origin, size, px))
                    .map(|(world, _)| [world[0] - grab[0], world[1] - grab[1]]);
                if let Some(to) = destination {
                    if state.world_doc.can_move_generated_object_to(&id, to) {
                        let previous = state
                            .world_doc
                            .generated_objects()
                            .find(|object| object.id == id)
                            .map(|object| state.world_doc.object_anchor(object));
                        if previous.is_some_and(|from| from != to) {
                            checkpoint_world(state);
                            match state.world_doc.move_generated_object(&id, to) {
                                Ok(true) => {
                                    state.world_dirty = true;
                                    state.message = format!("Moved generated object to {},{} (one undo step; source art untouched)", to[0], to[1]);
                                }
                                Ok(false) => {
                                    let _ = state.world_undo.pop();
                                }
                                Err(error) => {
                                    let _ = state.world_undo.pop();
                                    state.message = format!("Object move refused: {error}");
                                }
                            }
                        }
                    } else {
                        state.message = "Object stays in its original position: destination outside materialized world.".into();
                    }
                }
                state.world_dragging_object = None;
                state.world_drag_preview_anchor = None;
            }
        }
    }
    if let Some((start, _)) = state.world_bridge_drag {
        if let Some(pointer) = pointer {
            if let Some((end, _)) = world_pointer_cell(state, pointer, region, origin, size, px) {
                state.world_bridge_drag = Some((start, end));
            }
        }
        if let Some((start, end)) = state.world_bridge_drag {
            let valid = state.world_doc.validate_bridge_span(start, end).is_ok();
            let from = region.min
                + egui::vec2(
                    (start[0] - origin[0]) as f32 * px + px * 0.5,
                    (start[1] - origin[1]) as f32 * px + px * 0.5,
                );
            let to = region.min
                + egui::vec2(
                    (end[0] - origin[0]) as f32 * px + px * 0.5,
                    (end[1] - origin[1]) as f32 * px + px * 0.5,
                );
            painter.line_segment(
                [from, to],
                egui::Stroke::new(
                    3.0,
                    if valid {
                        egui::Color32::LIGHT_GREEN
                    } else {
                        egui::Color32::LIGHT_RED
                    },
                ),
            );
            if primary_released {
                if valid {
                    checkpoint_world(state);
                    match state
                        .world_doc
                        .place_authored_bridge(start, end, &state.asset_authority)
                    {
                        Ok(id) => {
                            state.world_selected_object = Some(id);
                            state.world_dirty = true;
                            state.message =
                                "Authored bridge committed as one undoable, source-backed object."
                                    .into();
                        }
                        Err(error) => {
                            let _ = state.world_undo.pop();
                            state.message = error;
                        }
                    }
                } else {
                    state.message = state
                        .world_doc
                        .validate_bridge_span(start, end)
                        .unwrap_err();
                }
                state.world_bridge_drag = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
    }
    if primary_down && state.collision_stroke_active {
        if let Some(pointer) = pointer {
            if let Some((world, pixel)) =
                world_pointer_cell(state, pointer, region, origin, size, px)
            {
                paint_world_collision_brush(state, world, pixel);
            }
        }
        ui.ctx().request_repaint();
    }
    if primary_released && state.collision_stroke_active {
        state.collision_stroke_active = false;
        state.world_doc.commit_edit_batch();
        state.world_dirty = true;
        state.message = format!(
            "Collision stroke committed ({}) · brush {}px. Source artwork unchanged.",
            state.world_behavior_scope.label(),
            state.collision_brush_radius
        );
    }

    // Paint/erase continuously while the primary button is held. A cell is only
    // processed once per stroke, so slow drags do not spam revisions or regeneration.
    if primary_down && !state.world_stroke_cells.is_empty() {
        if let Some(pointer) = pointer {
            if let Some((world, _)) = world_pointer_cell(state, pointer, region, origin, size, px) {
                world_stroke_apply_segment(state, world);
            }
        }
        ui.ctx().request_repaint();
    }
    if primary_released && !state.world_stroke_cells.is_empty() {
        world_stroke_finish(state);
    }
    if state.selection_active
        && response.hovered()
        && !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|i| i.key_pressed(egui::Key::Delete))
    {
        if let Some(id) = state.world_selected_object.clone() {
            checkpoint_world(state);
            if state.world_doc.suppress_generated_object(&id) {
                state.world_dirty = true;
                state.world_selected_object = None;
            }
        }
    }
    // Paint over generated-world textures/objects using signed *world* coordinates.
    // The ruler occupies only the canvas border and never changes the document.
    if state.world_playtest.is_none()
        && canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode)
    {
        canvas_rulers::draw(
            &painter,
            canvas,
            region,
            px,
            ui.ctx().pointer_hover_pos(),
            state.selection_active.then_some(state.world_selected),
            origin,
            &state.editor_layout.editor_theme,
        );
    }
}

/// Room document editing lives in the same Studio canvas and bottom Library,
/// not in a duplicate floating Terrain Studio or a hidden PIE-only tool.
fn open_house_room_editor_for(state: &mut Studio, object_id: &str) {
    let object_is_house = state.world_doc.generated_objects().any(|obj| {
        obj.id == object_id && {
            let template = obj.template_id.to_ascii_lowercase();
            template.contains("house")
                || template.contains("building")
                || obj.label.to_ascii_lowercase().contains("house")
        }
    });
    if !object_is_house {
        state.message =
            "Select and right-click a house/building object to edit its interior".into();
        return;
    }
    if state.room_authoring.as_ref().is_some_and(|edit| edit.dirty) {
        state.message = "Save or leave the current room before opening another house".into();
        return;
    }
    let path = project_root::resolve().join(room_scene::house_instance_relative_path(
        state.world_doc.seed,
        object_id,
    ));
    let room = if path.is_file() {
        room_scene::RoomScene::load_template(&path)
    } else {
        let template_path = project_root::resolve().join("content/rooms/starter_house.room.json");
        room_scene::RoomScene::load_template(&template_path).map(|mut room| {
            room.scene_id = format!("house:{object_id}");
            room
        })
    };
    match room.and_then(room_authoring::RoomEditor::open) {
        Ok(edit) => {
            state.room_authoring = Some(edit);
            state.room_authoring_object = Some(object_id.to_owned());
            state.editor_layout.authoring_library_tab = "Rooms".into();
            state.editor_layout.library_open = true;
            save_editor_layout(state);
            state.message = "Editing this house's own room. Paint floor/walls, move exit, Ctrl+S to save. Map Doorway from the house context menu for PIE.".into();
        }
        Err(error) => state.message = format!("Room editing unavailable: {error}"),
    }
}

fn save_house_room_editor(state: &mut Studio) -> bool {
    let Some(edit) = state.room_authoring.as_mut() else {
        return false;
    };
    let Some(object_id) = state.room_authoring_object.as_ref() else {
        state.message = "Cannot save an unbound room; re-open it from a house object".into();
        return false;
    };
    let path = project_root::resolve().join(room_scene::house_instance_relative_path(
        state.world_doc.seed,
        object_id,
    ));
    let result = edit
        .scene
        .validate()
        .and_then(|_| serde_json::to_vec_pretty(&edit.scene).map_err(|e| e.to_string()))
        .and_then(|bytes| {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            crate::atomic_file::write_atomic(&path, &bytes)
        });
    match result {
        Ok(()) => {
            edit.dirty = false;
            if let Some(link) = state.world_doc.structure_doors.get_mut(object_id) {
                if let Some(exit) = edit
                    .scene
                    .portals
                    .iter()
                    .find(|p| p.kind == room_scene::PortalKind::Exit)
                {
                    if link.destination_cell != exit.cell {
                        link.destination_cell = exit.cell;
                        state.world_dirty = true;
                    }
                }
            }
            state.message = "House-specific interior saved. If a mapped entry changed, save World with Ctrl+S after returning.".into();
            true
        }
        Err(error) => {
            state.message = format!("Room save refused; existing room preserved: {error}");
            false
        }
    }
}

fn draw_room_editor_controls(ui: &mut egui::Ui, state: &mut Studio) {
    ui.strong("Room authoring · house template");
    let Some(edit) = state.room_authoring.as_mut() else {
        ui.small("Select a house in the World canvas, right-click → Edit Interior.");
        return;
    };
    ui.horizontal_wrapped(|ui| {
        for (tool, label) in [
            (room_authoring::RoomTool::Floor, "Paint floor"),
            (room_authoring::RoomTool::EraseFloor, "Erase floor"),
            (room_authoring::RoomTool::MoveExit, "Move exit"),
        ] {
            if ui.selectable_label(edit.tool == tool, label).clicked() {
                edit.tool = tool;
                edit.stroke_begin();
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.label("Room size");
        for (dims, label) in [
            ([2, 2], "2×2"),
            ([3, 2], "3×2"),
            ([2, 3], "2×3"),
            ([3, 3], "3×3"),
        ] {
            if ui
                .selectable_label(edit.scene.chunks == dims, label)
                .clicked()
            {
                if let Err(error) = edit.resize_chunks(dims) {
                    state.message = error;
                }
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Wall color");
        let mut tint = egui::Color32::from_rgba_unmultiplied(
            edit.scene.style.wall_tint_rgba[0],
            edit.scene.style.wall_tint_rgba[1],
            edit.scene.style.wall_tint_rgba[2],
            edit.scene.style.wall_tint_rgba[3],
        );
        if ui.color_edit_button_srgba(&mut tint).changed() {
            if let Err(error) = edit.set_wall_tint(tint.to_array()) {
                state.message = error;
            }
        }
    });
    let floor_count = edit.scene.cells.iter().filter(|tile| tile.floor).count();
    ui.small(format!(
        "{}×{} chunks · {} painted floor cells · {} exit(s)",
        edit.scene.chunks[0],
        edit.scene.chunks[1],
        floor_count,
        edit.scene
            .portals
            .iter()
            .filter(|p| p.kind == room_scene::PortalKind::Exit)
            .count()
    ));
    ui.small("Source artwork binding is pending. This is a true saved topology/collision editor, not a claim of finished tile art.");
}

fn draw_room_authoring_workspace(ui: &mut egui::Ui, state: &mut Studio) {
    let size = ui.available_size().max(egui::vec2(200.0, 150.0));
    let (canvas, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
    state.canvas_rect = canvas;
    state.canvas_animation_rect =
        if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some() {
            egui::Rect::NOTHING
        } else {
            canvas_animation_control_rect(
                canvas,
                canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
            )
        };
    state.canvas_layers_rect = egui::Rect::NOTHING;
    state.canvas_animation_rect = egui::Rect::NOTHING;
    state.canvas_controls_rect = egui::Rect::NOTHING;
    state.canvas_grid_rect = egui::Rect::NOTHING;
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(18, 24, 27));
    let Some(edit) = state.room_authoring.as_mut() else {
        return;
    };
    let dimensions = edit.scene.dimensions();
    let px = 32.0 * edit.zoom;
    if response.hovered() {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if wheel.abs() > 0.01 {
            let old = edit.zoom;
            edit.zoom = (old * (wheel / 240.0).exp()).clamp(0.12, 3.0);
            let ratio = edit.zoom / old;
            let anchor = ui
                .input(|i| i.pointer.hover_pos())
                .unwrap_or(canvas.center());
            edit.pan[0] = edit.pan[0] * ratio + (anchor.x - canvas.center().x) * (1.0 - ratio);
            edit.pan[1] = edit.pan[1] * ratio + (anchor.y - canvas.center().y) * (1.0 - ratio);
        }
    }
    if response.dragged_by(egui::PointerButton::Middle) {
        // Pointer delta is per-frame; Response::drag_delta is cumulative and
        // would accelerate panning every frame of the same drag.
        let delta = ui.input(|input| input.pointer.delta());
        edit.pan[0] += delta.x;
        edit.pan[1] += delta.y;
    }
    let px = 32.0 * edit.zoom;
    let origin = canvas.center() + egui::vec2(edit.pan[0], edit.pan[1])
        - egui::vec2(
            dimensions[0] as f32 * px * 0.5,
            dimensions[1] as f32 * px * 0.5,
        );
    let min_x = (((canvas.left() - origin.x) / px).floor() as i32).max(0);
    let min_y = (((canvas.top() - origin.y) / px).floor() as i32).max(0);
    let max_x = (((canvas.right() - origin.x) / px).ceil() as i32).min(dimensions[0] as i32);
    let max_y = (((canvas.bottom() - origin.y) / px).ceil() as i32).min(dimensions[1] as i32);
    let tint = edit.scene.style.wall_tint_rgba;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let Some(idx) = edit.scene.index([x as u32, y as u32]) else {
                continue;
            };
            let cell = edit.scene.cells[idx];
            let rect = egui::Rect::from_min_size(
                origin + egui::vec2(x as f32 * px, y as f32 * px),
                egui::Vec2::splat(px),
            );
            if cell.floor {
                painter.rect_filled(
                    rect,
                    0.0,
                    if (x + y) % 2 == 0 {
                        egui::Color32::from_rgb(75, 72, 68)
                    } else {
                        egui::Color32::from_rgb(80, 77, 72)
                    },
                );
            } else if cell.wall_neighbor_mask != 0 {
                painter.rect_filled(
                    rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(tint[0], tint[1], tint[2], tint[3]),
                );
                if px >= 8.0 {
                    painter.rect_stroke(
                        rect.shrink(0.5),
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::from_gray(70)),
                        egui::StrokeKind::Inside,
                    );
                }
            }
            if px >= 13.0 {
                painter.rect_stroke(
                    rect,
                    0.0,
                    egui::Stroke::new(0.5, egui::Color32::from_gray(47)),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
    for portal in &edit.scene.portals {
        let center = origin
            + egui::vec2(
                (portal.cell[0] as f32 + 0.5) * px,
                (portal.cell[1] as f32 + 0.5) * px,
            );
        if canvas.contains(center) {
            painter.circle_filled(
                center,
                (px * 0.28).max(3.0),
                egui::Color32::from_rgb(240, 188, 65),
            );
        }
    }
    if !ui.input(|i| i.pointer.primary_down()) {
        edit.stroke_begin();
    }
    if response.hovered() && ui.input(|i| i.pointer.primary_down()) {
        if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
            let local = pointer - origin;
            let x = (local.x / px).floor() as i32;
            let y = (local.y / px).floor() as i32;
            if x >= 0 && y >= 0 && x < dimensions[0] as i32 && y < dimensions[1] as i32 {
                if let Err(error) = edit.paint([x as u32, y as u32]) {
                    state.message = error;
                }
            }
        }
    }
    painter.text(
        canvas.min + egui::vec2(12.0, 12.0),
        egui::Align2::LEFT_TOP,
        format!(
            "ROOM EDITOR · {} · {}x{} tiles{}",
            edit.scene.scene_id,
            dimensions[0],
            dimensions[1],
            if edit.dirty { " · UNSAVED" } else { "" }
        ),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );
    painter.text(canvas.min + egui::vec2(12.0,32.0), egui::Align2::LEFT_TOP,
        "Left drag = paint · Middle drag = pan · Wheel = zoom · Ctrl+S = save · Return through World PIE",
        egui::FontId::proportional(12.0), egui::Color32::LIGHT_GRAY);
}

fn draw_world(ui: &mut egui::Ui, state: &mut Studio) {
    if state.room_authoring.is_some() {
        draw_room_authoring_workspace(ui, state);
        return;
    }
    if state.world_mode {
        draw_world_workspace(ui, state);
        return;
    }
    let available = ui.available_size().max(egui::vec2(200.0, 160.0));
    let (canvas, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
    state.canvas_rect = canvas;
    state.canvas_animation_rect =
        if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some() {
            egui::Rect::NOTHING
        } else {
            canvas_animation_control_rect(
                canvas,
                canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
            )
        };
    state.canvas_layers_rect =
        if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some() {
            egui::Rect::NOTHING
        } else {
            canvas_layer_rect(
                canvas,
                state.editor_layout.layers_dock_open,
                canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
            )
        };
    // Rulers are painted inside the allocated canvas; no dock, layout shrink or extra scene layer.
    let ruler_inset =
        if canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode) {
            egui::vec2(canvas_rulers::LEFT, canvas_rulers::TOP)
        } else {
            egui::Vec2::ZERO
        };
    let overlays = if state.focus_mode || state.playtest.is_some() || state.world_playtest.is_some()
    {
        canvas_chrome::OverlayGeometry {
            grid: egui::Rect::NOTHING,
            zoom: egui::Rect::NOTHING,
        }
    } else {
        canvas_chrome::overlay_geometry(canvas, ruler_inset.y)
    };
    state.canvas_controls_rect = overlays.zoom;
    state.canvas_grid_rect = overlays.grid;
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(20, 29, 27));
    if state.playtest.is_none() && state.world_playtest.is_none() && response.hovered() {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if wheel.abs() > 0.01 {
            if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                zoom_canvas_by(state, pointer, canvas, (wheel / 240.0).exp());
            }
        }
    }
    if state.playtest.is_none()
        && (response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary))
    {
        state.pan += response.drag_delta();
    }
    let px = state.scene.tile_size as f32 * state.zoom;
    let dimensions = egui::vec2(
        state.scene.size[0] as f32 * px,
        state.scene.size[1] as f32 * px,
    );
    let scene = egui::Rect::from_center_size(canvas.center() + state.pan, dimensions);
    state.scene_rect = scene;
    // Draw neutral checkerboard only under the selected cell: transparent source
    // pixels remain inspectable without changing or recoloring original art.
    draw_alpha_checker(&painter, state, scene, px, canvas);
    // Visible-cell culling keeps the assembled scene responsive while preserving
    // exact source PNG addresses and transparent boundaries.
    let visible = canvas.intersect(scene);
    if visible.width() > 0.0 && visible.height() > 0.0 {
        let x0 = (((visible.left() - scene.left()) / px).floor() as isize).max(0) as usize;
        let y0 = (((visible.top() - scene.top()) / px).floor() as isize).max(0) as usize;
        let x1 =
            (((visible.right() - scene.left()) / px).ceil() as usize + 1).min(state.scene.size[0]);
        let y1 =
            (((visible.bottom() - scene.top()) / px).ceil() as usize + 1).min(state.scene.size[1]);
        if world_layer_visible(state, LayerId::Ground) {
            for y in y0..y1 {
                for x in x0..x1 {
                    let index = y * state.scene.size[0] + x;
                    let tile = if let Some(play) = &state.playtest {
                        play.snapshot.tiles[index].clone()
                    } else {
                        state
                            .layered_scene
                            .resolved_legacy_tile(index)
                            .expect("valid base scene index")
                    };
                    if tile.is_empty() {
                        continue;
                    }
                    // Partial right/bottom edge cells (for example 144px-wide wildflowers)
                    // keep their true pixel footprint, instead of stretching to a full 32px tile.
                    let pixels_per_point = ui.ctx().pixels_per_point();
                    let target = egui::Rect::from_min_max(
                        egui::pos2(
                            snapped_scene_edge(scene.min.x, x as f32, px, pixels_per_point),
                            snapped_scene_edge(scene.min.y, y as f32, px, pixels_per_point),
                        ),
                        egui::pos2(
                            snapped_scene_edge(
                                scene.min.x,
                                x as f32 + tile.source_rect[2] as f32 / 32.0,
                                px,
                                pixels_per_point,
                            ),
                            snapped_scene_edge(
                                scene.min.y,
                                y as f32 + tile.source_rect[3] as f32 / 32.0,
                                px,
                                pixels_per_point,
                            ),
                        ),
                    );
                    if !target.intersects(canvas) {
                        continue;
                    }
                    let Some(sheet_index) =
                        state.sheet_lookup.get(tile.source_asset.as_str()).copied()
                    else {
                        continue;
                    };
                    let sheet = &state.sheets[sheet_index];
                    if let Some(texture) = sheet.texture {
                        if tile.source_rect[0] + tile.source_rect[2] <= sheet.size[0]
                            && tile.source_rect[1] + tile.source_rect[3] <= sheet.size[1]
                        {
                            painter.image(
                                texture,
                                target,
                                source_uv(tile.source_rect, sheet.size),
                                egui::Color32::WHITE,
                            );
                            // Source-evidence markers are small and outside the artist's alpha content.
                            // Green = recovered source identity, never automatic DG certification.
                            if state.editor_layout.mapping_status_visible
                                && px >= 15.0
                                && tile.status == "source_mapped_not_recipe_certified"
                            {
                                painter.rect_filled(
                                    egui::Rect::from_min_size(target.min, egui::vec2(3.0, 3.0)),
                                    0.0,
                                    egui::Color32::from_rgb(124, 210, 155),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // The exact same sparse v2 visual layers are composited above v1 ground in
    // authoring and the frozen PIE snapshot. Alpha reveals underlying ground/water.
    draw_source_exact_visual_layers(
        &painter,
        state,
        scene,
        px,
        canvas,
        ui.ctx().pixels_per_point(),
    );
    // Hydrated LPC sprite is source artwork; a diagnostic marker remains only if character art is absent.
    if let Some(play) = &state.playtest {
        let center = scene.min + egui::vec2(play.position[0] * px, play.position[1] * px);
        if canvas.contains(center) {
            if state.sheet_lookup.contains_key(PLAYER_WALK_SHEET) {
                draw_pie_character(
                    &painter,
                    state,
                    center,
                    px,
                    play.facing_row,
                    play.elapsed,
                    play.moving,
                );
            } else {
                let radius = (px * 0.25).clamp(4.0, 12.0);
                painter.circle_filled(
                    center + egui::vec2(1.0, 2.0),
                    radius + 2.0,
                    egui::Color32::from_black_alpha(155),
                );
                painter.circle_filled(center, radius, egui::Color32::from_rgb(253, 223, 113));
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(1.5, egui::Color32::from_rgb(29, 34, 35)),
                );
                painter.line_segment(
                    [center, center - egui::vec2(0.0, radius * 0.72)],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(24, 39, 48)),
                );
            }
        }
    } else {
        draw_assembled_objects(&painter, state, scene, px, canvas);
    }
    if state.playtest.is_none() {
        draw_collision_overlay(&painter, state, scene, px, canvas);
        world_chunks::draw(
            &painter,
            canvas,
            scene,
            px,
            state.scene.size,
            state.editor_layout.chunk_size_tiles,
            state.editor_layout.chunk_grid_visible,
            state.editor_layout.tile_grid_visible,
        );
        draw_structural_overlay(&painter, state, scene, px, canvas);
    }
    if state.playtest.is_none() && state.selection_active {
        if let Some(selected) = state
            .scene
            .index(state.selected_world[0], state.selected_world[1])
        {
            let x = selected % state.scene.size[0];
            let y = selected / state.scene.size[0];
            let rect = if let Some(object) = selected_object(state) {
                let anchor = if state
                    .dragging_object
                    .as_ref()
                    .is_some_and(|(id, _)| id == &object.id)
                {
                    state.drag_preview_origin.unwrap_or(object.anchor)
                } else {
                    object.anchor
                };
                egui::Rect::from_min_size(
                    scene.min + egui::vec2(anchor[0] as f32 * px, anchor[1] as f32 * px),
                    egui::vec2(
                        object.footprint[0] as f32 * px,
                        object.footprint[1] as f32 * px,
                    ),
                )
            } else if let Some(object) = state
                .mapping_lab
                .visual_at(state.selected_world, &state.scene)
            {
                egui::Rect::from_min_size(
                    scene.min
                        + egui::vec2(object.origin[0] as f32 * px, object.origin[1] as f32 * px),
                    egui::vec2(
                        object.source_rect[2] as f32 * px / 32.0,
                        object.source_rect[3] as f32 * px / 32.0,
                    ),
                )
            } else {
                egui::Rect::from_min_size(
                    scene.min + egui::vec2(x as f32 * px, y as f32 * px),
                    egui::Vec2::splat(px),
                )
            };
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.5, egui::Color32::YELLOW),
                egui::StrokeKind::Inside,
            );
        }
    }
    if state.playtest.is_none()
        && canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode)
    {
        canvas_rulers::draw(
            &painter,
            canvas,
            scene,
            px,
            ui.ctx().pointer_hover_pos(),
            state
                .selection_active
                .then_some(state.selected_world.map(|v| v as i64)),
            [0, 0],
            &state.editor_layout.editor_theme,
        );
    }
    // Capture SELECT on initial pointer press, before egui's drag threshold moves
    // the pointer to an adjacent tile. No second Move tool or click-to-relocate mode.
    if state.playtest.is_none()
        && state.dragging.is_none()
        && state.tool == CanvasTool::Select
        && !state.collision_world_pixel_edit
        && response.hovered()
        && ui.input(|input| input.pointer.primary_pressed())
    {
        if let Some(pos) = ui
            .ctx()
            .pointer_hover_pos()
            .and_then(|pointer| world_at(state, pointer))
        {
            state.selected_world = pos;
            state.selection_active = true;
            if select_object_at(state, pos) {
                if let Some(object) = selected_object(state) {
                    let id = object.id.clone();
                    let grab_offset = [
                        pos[0] as i32 - object.anchor[0],
                        pos[1] as i32 - object.anchor[1],
                    ];
                    state.drag_preview_origin = Some(object.anchor);
                    state.dragging_object = Some((id, grab_offset));
                }
            } else {
                state.dragging_object = None;
                state.drag_preview_origin = None;
                state.message = format!("Selected {:?} cell {},{}. Select an object on the canvas, or use the source browser to place one.", state.active_layer, pos[0], pos[1]);
            }
        }
    }
    if state.playtest.is_none() && response.clicked_by(egui::PointerButton::Primary) {
        if let Some(pointer) = response.interact_pointer_pos() {
            if let Some(pos) = world_at(state, pointer) {
                state.selected_world = pos;
                state.selection_active = true;
                if state.dragging.is_none() {
                    if state.collision_world_pixel_edit && state.editor_layout.structural_open {
                        if let Some((index, pixel)) = collision_pixel_under_pointer(state, pointer)
                        {
                            let blocked = state.collision_brush_blocked;
                            edit_collision_pixel(state, index, pixel, blocked);
                        }
                    } else {
                        match state.tool {
                            CanvasTool::Select => {} // handled at pointer press for reliable dragging
                            CanvasTool::Paint => {
                                if state.brush_mode == BrushMode::Tile {
                                    let sheet = state.active_sheet;
                                    let source = selected_source_rect(state);
                                    apply_source(state, pos, sheet, source);
                                } else {
                                    state.message = "Scene painting requires Tile Brush. Terrain Brush edits world semantics; Pixel Brush requires a derived image document.".into();
                                }
                            }
                            CanvasTool::Erase => {
                                if !hide_selected_visual_sample(state, pos) {
                                    erase_source(state, pos);
                                }
                            }
                            CanvasTool::Sample => {
                                select_object_at(state, pos);
                                sample_canvas_source(state, pos);
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some((id, grab_offset)) = state.dragging_object.clone() {
        if ui.input(|input| input.pointer.primary_down()) {
            if let Some(pos) = ui
                .ctx()
                .pointer_hover_pos()
                .and_then(|pointer| world_at(state, pointer))
            {
                let proposed = [
                    pos[0] as i32 - grab_offset[0],
                    pos[1] as i32 - grab_offset[1],
                ];
                // Preview only valid destinations; no transaction is written until release.
                if state.layered_scene.can_move_object_to(&id, proposed) {
                    state.drag_preview_origin = Some(proposed);
                }
            }
            ui.ctx().request_repaint();
        }
        if ui.input(|input| input.pointer.primary_released()) {
            if let Some(pos) = ui
                .ctx()
                .pointer_hover_pos()
                .and_then(|pointer| world_at(state, pointer))
            {
                let destination = [
                    pos[0] as i32 - grab_offset[0],
                    pos[1] as i32 - grab_offset[1],
                ];
                if state.layered_scene.can_move_object_to(&id, destination) {
                    if selected_object(state).is_some_and(|object| object.anchor != destination) {
                        move_selected_object_to_origin(state, id, destination);
                    }
                } else {
                    state.message = "Object drag refused: destination would leave the scene. Original position retained.".into();
                }
            } else {
                state.message =
                    "Object drag canceled outside the editable canvas; scene unchanged.".into();
            }
            state.dragging_object = None;
            state.drag_preview_origin = None;
        }
    }
    if state.playtest.is_none()
        && state.selection_active
        && response.hovered()
        && !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|input| input.key_pressed(egui::Key::Delete))
    {
        let pos = state.selected_world;
        if !remove_selected_object(state) && !hide_selected_visual_sample(state, pos) {
            erase_source(state, pos);
        }
    }
    if let Some(play) = state.playtest.as_ref() {
        egui::Area::new(egui::Id::new("havenwild.pie.canvas.status"))
            .fixed_pos(canvas.min + egui::vec2(14.0, 12.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(20, 29, 27, 220))
                    .show(ui, |ui| {
                        ui.strong("PLAY IN EDITOR | source-addressed scene snapshot");
                        ui.small(format!("Position {:.1},{:.1}  •  blocked steps {}  •  W/A/S/D / arrows  •  Esc = Stop",
                            play.position[0], play.position[1], play.blocked_steps));
                        ui.small("Explicit authored traversal cells override provisional Grass/MudBank/water hints. Objects, cliffs and elevation travel NOT certified.");
                    });
            });
    }
}

fn draw_mask_preview(ui: &mut egui::Ui, mask: u8) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(112.0, 112.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 24, 28));
    let center = rect.center();
    let gap = 2.0;
    let cells = [
        (
            0x1,
            egui::Rect::from_min_max(
                rect.min + egui::vec2(gap, gap),
                center - egui::vec2(gap, gap),
            ),
        ),
        (
            0x2,
            egui::Rect::from_min_max(
                egui::pos2(center.x + gap, rect.top() + gap),
                egui::pos2(rect.right() - gap, center.y - gap),
            ),
        ),
        (
            0x4,
            egui::Rect::from_min_max(
                egui::pos2(rect.left() + gap, center.y + gap),
                egui::pos2(center.x - gap, rect.bottom() - gap),
            ),
        ),
        (
            0x8,
            egui::Rect::from_min_max(
                center + egui::vec2(gap, gap),
                rect.max - egui::vec2(gap, gap),
            ),
        ),
    ];
    for (bit, cell) in cells {
        let fill = if mask & bit != 0 {
            egui::Color32::from_rgb(86, 156, 76)
        } else {
            egui::Color32::from_rgb(44, 49, 56)
        };
        painter.rect_filled(cell, 2.0, fill);
    }
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(105)),
        egui::StrokeKind::Inside,
    );
}

fn recipe_status_glyph(resolution: &str, confidence: Option<&str>) -> &'static str {
    match (resolution, confidence) {
        ("sprite", Some("certified")) => "S+",
        ("sprite", Some("reviewed")) => "S~",
        ("sprite", _) => "S?",
        ("composite", Some("certified")) => "C+",
        ("composite", Some("reviewed")) => "C~",
        ("composite", _) => "C?",
        ("unsupported", Some("certified")) => "X+",
        ("unsupported", Some("reviewed")) => "X~",
        ("unsupported", _) => "X?",
        _ => "·",
    }
}

fn locate_recipe_source(state: &mut Studio, source: &TerrainRecipeSource, label: &str) {
    let Some(sheet_index) = state
        .sheets
        .iter()
        .position(|sheet| sheet.path == source.path)
    else {
        state.terrain_mapper.message = format!(
            "{label} source sheet is not loaded in the Terrain catalog: {}",
            source.path
        );
        return;
    };
    state.active_sheet = sheet_index;
    state.selected_cell = [source.rect[0] / 32, source.rect[1] / 32];
    state.terrain_mapper.message = format!(
        "Located {label} at {},{} on {}",
        source.rect[0], source.rect[1], source.path
    );
}

fn locate_selected_recipe_source(state: &mut Studio) {
    let selected = state.terrain_mapper.selected_state().cloned();
    let Some(selected) = selected else {
        state.terrain_mapper.message = "Selected topology state is unavailable.".into();
        return;
    };
    if let Some(source) = selected.source {
        let label = format!("mask {}", state.terrain_mapper.selected_key());
        locate_recipe_source(state, &source, &label);
        return;
    }
    if let Some(part) = selected.parts.first() {
        let label = format!(
            "mask {} composite part 1",
            state.terrain_mapper.selected_key()
        );
        locate_recipe_source(state, &part.source, &label);
        return;
    }
    state.terrain_mapper.message =
        "Selected topology has no source sprite or composite part to locate.".into();
}

fn locate_selected_canvas_source(state: &mut Studio) {
    let Some(_index) = state
        .scene
        .index(state.selected_world[0], state.selected_world[1])
    else {
        return;
    };
    let Some((path, rect)) = selected_scene_source(state, state.selected_world) else {
        state.message = "Selected layer/canvas cell has no source binding to locate.".into();
        return;
    };
    let source = TerrainRecipeSource { path, rect };
    locate_recipe_source(state, &source, "selected canvas tile");
}

fn save_all_authored_state(state: &mut Studio) {
    let mut saved = Vec::new();
    let mut errors = Vec::new();
    if state.layered_history.is_dirty() || state.scene_needs_initial_save {
        match state
            .layered_scene
            .save_editable_draft(&state.legacy_base_path, &state.layered_path)
        {
            Ok(()) => {
                state.layered_history.mark_saved();
                state.scene_needs_initial_save = false;
                saved.push("Summer authoring scene");
            }
            Err(error) => errors.push(format!("layered canvas: {error}")),
        }
    }
    if state.world_dirty {
        match state.world_doc.save(&state.world_path) {
            Ok(()) => {
                state.world_dirty = false;
                saved.push("world composition");
            }
            Err(error) => errors.push(format!("world composition: {error}")),
        }
    }
    if state.terrain_mapper.dirty {
        match state.terrain_mapper.save() {
            Ok(()) => saved.push("terrain recipe"),
            Err(error) => errors.push(format!("recipe: {error}")),
        }
    }
    if state.semantic_lab_dirty {
        match state.semantic_lab.save(&state.semantic_lab_path) {
            Ok(()) => {
                state.semantic_lab_dirty = false;
                saved.push("semantic resolver lab");
            }
            Err(error) => errors.push(format!("semantic lab: {error}")),
        }
    }
    if let Err(error) = state.editor_layout.save(&state.layout_path) {
        errors.push(format!("layout: {error}"));
    }
    if errors.is_empty() {
        state.message = if saved.is_empty() {
            "Save all: authored documents already clean.".into()
        } else {
            format!("Save all: {}.", saved.join(", "))
        };
    } else {
        state.message = format!("Save all completed with errors: {}", errors.join(" | "));
    }
}

fn draw_recipe_output_preview(
    ui: &mut egui::Ui,
    state: &Studio,
    recipe_state: Option<&TerrainRecipeState>,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(170.0, 170.0), egui::Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(18, 23, 25));
    painter.rect_stroke(
        rect,
        5.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
        egui::StrokeKind::Inside,
    );

    let Some(recipe_state) = recipe_state else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "missing state",
            egui::FontId::proportional(12.0),
            egui::Color32::LIGHT_RED,
        );
        return;
    };

    let draw_source = |source: &TerrainRecipeSource, offset: [i32; 2], bounds: [f32; 4]| {
        let Some(sheet) = state.sheets.iter().find(|sheet| sheet.path == source.path) else {
            return false;
        };
        let Some(texture) = sheet.texture else {
            return false;
        };
        let available = rect.shrink(10.0);
        let bounds_w = (bounds[2] - bounds[0]).max(1.0);
        let bounds_h = (bounds[3] - bounds[1]).max(1.0);
        let scale = (available.width() / bounds_w)
            .min(available.height() / bounds_h)
            .min(4.0);
        let fitted = egui::vec2(bounds_w * scale, bounds_h * scale);
        let origin = available.center() - fitted * 0.5;
        let min = origin
            + egui::vec2(
                (offset[0] as f32 - bounds[0]) * scale,
                (offset[1] as f32 - bounds[1]) * scale,
            );
        let target = egui::Rect::from_min_size(
            min,
            egui::vec2(source.rect[2] as f32 * scale, source.rect[3] as f32 * scale),
        );
        painter.image(
            texture,
            target,
            source_uv(source.rect, sheet.size),
            egui::Color32::WHITE,
        );
        true
    };

    match recipe_state.resolution.as_str() {
        "sprite" => {
            if let Some(source) = &recipe_state.source {
                let bounds = [0.0, 0.0, source.rect[2] as f32, source.rect[3] as f32];
                if !draw_source(source, [0, 0], bounds) {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "source not loaded",
                        egui::FontId::proportional(12.0),
                        egui::Color32::YELLOW,
                    );
                }
            }
        }
        "composite" if !recipe_state.parts.is_empty() => {
            let mut min_x = i32::MAX;
            let mut min_y = i32::MAX;
            let mut max_x = i32::MIN;
            let mut max_y = i32::MIN;
            for part in &recipe_state.parts {
                min_x = min_x.min(part.offset[0]);
                min_y = min_y.min(part.offset[1]);
                max_x = max_x.max(part.offset[0] + part.source.rect[2] as i32);
                max_y = max_y.max(part.offset[1] + part.source.rect[3] as i32);
            }
            let bounds = [min_x as f32, min_y as f32, max_x as f32, max_y as f32];
            let mut all_loaded = true;
            for part in &recipe_state.parts {
                all_loaded &= draw_source(&part.source, part.offset, bounds);
            }
            if !all_loaded {
                painter.text(
                    rect.left_bottom() + egui::vec2(5.0, -5.0),
                    egui::Align2::LEFT_BOTTOM,
                    "some sources not loaded",
                    egui::FontId::proportional(10.0),
                    egui::Color32::YELLOW,
                );
            }
        }
        "unsupported" => {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "unsupported",
                egui::FontId::proportional(13.0),
                egui::Color32::LIGHT_RED,
            );
        }
        _ => {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "unmapped",
                egui::FontId::proportional(13.0),
                egui::Color32::GRAY,
            );
        }
    }
}

fn save_editor_layout(state: &mut Studio) {
    // Shift-isolate is view-only and session-local. Persist the user's actual
    // pre-isolation visibility even when a theme, dock or tool setting is saved
    // while isolation is active. An eye-button edit cancels isolation first.
    let mut disk_layout = state.editor_layout.clone();
    disk_layout.world_layer_visibility = state
        .layer_isolate
        .persisted_visibility(state.editor_layout.world_layer_visibility);
    if let Err(error) = disk_layout.save(&state.layout_path) {
        state.message = format!("Editor layout save failed: {error}");
    }
}

fn set_mapper_dock(state: &mut Studio, dock: SurfaceDock) {
    if state.editor_layout.terrain_mapper.move_to(dock) {
        state.editor_layout.terrain_mapper.visible = true;
        save_editor_layout(state);
    }
}

fn draw_mapper_header(ui: &mut egui::Ui, state: &mut Studio) {
    let current = state.editor_layout.terrain_mapper.dock;
    let locked = state.editor_layout.terrain_mapper.locked;
    let mut requested_dock = None;
    let mut requested_lock = locked;
    let mut requested_compact = state.editor_layout.terrain_mapper_compact;
    let mut hide = false;

    ui.horizontal_wrapped(|ui| {
        ui.strong("Terrain Studio");
        ui.separator();
        for (dock, label) in [
            (SurfaceDock::Left, "Dock L"),
            (SurfaceDock::Right, "Dock R"),
            (SurfaceDock::Bottom, "Dock B"),
            (SurfaceDock::Floating, "Float"),
        ] {
            if ui.selectable_label(current == dock, label).clicked() {
                requested_dock = Some(dock);
            }
        }
        ui.separator();
        ui.checkbox(&mut requested_lock, "Lock");
        if ui
            .selectable_label(requested_compact, if requested_compact { "Compact" } else { "Full" })
            .on_hover_text("Switch this tool panel between compact and feature-rich presentation. The world canvas never resizes.")
            .clicked()
        {
            requested_compact = !requested_compact;
        }
        if ui.button("Hide").clicked() {
            hide = true;
        }
    });

    if requested_lock != locked {
        state.editor_layout.terrain_mapper.locked = requested_lock;
        save_editor_layout(state);
    }
    if requested_compact != state.editor_layout.terrain_mapper_compact {
        state.editor_layout.terrain_mapper_compact = requested_compact;
        save_editor_layout(state);
    }
    if let Some(dock) = requested_dock {
        set_mapper_dock(state, dock);
    }
    if hide {
        state.editor_layout.terrain_mapper.visible = false;
        save_editor_layout(state);
    }
}

fn draw_recovery_summary(ui: &mut egui::Ui, state: &Studio) {
    let (replayed, historical, transparent) = state.terrain_mapper.source_recovery_counts();
    let topology_families = state
        .terrain_mapper
        .historical_count("pass106_seasonal_topology", "topologyFamilies")
        .unwrap_or(0);
    let crosswalk = state
        .terrain_mapper
        .historical_count("b48r15_summer_mapper", "summerCrosswalkEntries")
        .unwrap_or(0);
    let seasonal_addresses = state
        .terrain_mapper
        .historical_count("b48r15_summer_mapper", "seasonalSourceAddresses")
        .unwrap_or(0);

    ui.group(|ui| {
        ui.strong("Recovered Summer authority");
        ui.label(format!(
            "Source cells: {replayed}/{historical} non-transparent recovered · {transparent} transparent"
        ));
        ui.label(format!(
            "Historical evidence: {topology_families} topology families · {crosswalk} Summer crosswalk hints · {seasonal_addresses} seasonal addresses"
        ));
        ui.label(format!(
            "In source-authored Summer base: {} distinct Summer source cells used / {} recovered globally; {} approved object compositions; DG masks certified {}/16",
            state.mapping_lab.summer_source_cells_actually_used,
            state.mapping_lab.summer_recovered_source_cells,
            state.mapping_lab.visual_samples.len(),
            state.mapping_lab.certified_recipe_masks,
        ));
        ui.small(
            "Source-map evidence is preserved separately from visual scene coverage and 16-mask recipe certification; missing legacy semantic records are never guessed.",
        );
    });
}

fn selected_source_rect(state: &Studio) -> [u32; 4] {
    let sheet = &state.sheets[state.active_sheet];
    let x = state.selected_cell[0] * 32;
    let y = state.selected_cell[1] * 32;
    let width = state.selected_extent[0].max(1) * 32;
    let height = state.selected_extent[1].max(1) * 32;
    [
        x,
        y,
        width.min(sheet.size[0].saturating_sub(x)),
        height.min(sheet.size[1].saturating_sub(y)),
    ]
}

fn draw_source_picker(ui: &mut egui::Ui, state: &mut Studio, max_height: f32) -> String {
    let filter = state.source_filter.trim().to_ascii_lowercase();
    let family = state.source_family_filter.as_str();
    let matching = state
        .asset_authority
        .runtime_source_images
        .iter()
        .filter(|entry| {
            family == "All" || entry.family == family || entry.semantic_category == family
        })
        .filter(|entry| {
            filter.is_empty() || entry.source_path.to_ascii_lowercase().contains(&filter)
        })
        .map(|entry| entry.source_path.clone())
        .collect::<Vec<_>>();
    let mut requested_path: Option<String> = None;
    ui.horizontal_wrapped(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.source_filter)
                .hint_text("Find any original ElizaWy PNG…")
                .desired_width(215.0),
        );
        egui::ComboBox::from_id_salt("source.family")
            .selected_text(&state.source_family_filter)
            .show_ui(ui, |ui| {
                for name in ["All", "Terrain", "Vegetation", "Structure", "Objects", "FX"] {
                    ui.selectable_value(&mut state.source_family_filter, name.into(), name);
                }
            });
        egui::ComboBox::from_id_salt("source.sheet")
            .selected_text(&state.sheets[state.active_sheet].path)
            .show_ui(ui, |ui| {
                for path in &matching {
                    if ui
                        .selectable_label(
                            state.sheets[state.active_sheet].path.as_str() == path.as_str(),
                            path,
                        )
                        .clicked()
                    {
                        requested_path = Some(path.clone());
                    }
                }
            });
        ui.small(format!(
            "{} matching · {} original PNGs cataloged · {} loaded on demand",
            matching.len(),
            state.asset_authority.counts.runtime_source_images,
            state.sheets.len()
        ));
        ui.separator();
        for (zoom, label) in [(0.5, "50%"), (0.75, "75%"), (1.0, "100%"), (1.5, "150%")] {
            if ui
                .selectable_label((state.atlas_zoom - zoom).abs() < 0.01, label)
                .clicked()
            {
                state.atlas_zoom = zoom;
            }
        }
    });
    if let Some(path) = requested_path {
        if let Some(index) = state.sheets.iter().position(|sheet| sheet.path == path) {
            state.active_sheet = index;
            state.selected_cell = [0, 0];
            state.selected_extent = [1, 1];
        } else {
            state.pending_source_sheet = Some(path);
            state.message =
                "Loading the selected original image; source files remain read-only.".into();
        }
    }

    let sheet_index = state.active_sheet;
    let sheet_path = state.sheets[sheet_index].path.clone();
    let sheet_size = state.sheets[sheet_index].size;
    let sheet_texture = state.sheets[sheet_index].texture;
    let zoom = state.atlas_zoom.clamp(0.25, 4.0);

    ui.label(format!(
        "Original image unchanged · manually select exact source region · target: {:?}",
        state.active_layer
    ));
    let source_size = state.sheets[state.active_sheet].size;
    ui.horizontal(|ui| {
        ui.label("Region width (cells)");
        ui.add(egui::Slider::new(
            &mut state.selected_extent[0],
            1..=source_size[0].div_ceil(32),
        ));
        ui.label("Height");
        ui.add(egui::Slider::new(
            &mut state.selected_extent[1],
            1..=source_size[1].div_ceil(32),
        ));
        ui.small("No automatic object boundary inference");
    });
    let explicit_rect = selected_source_rect(state);
    ui.small(format!(
        "Explicit source rectangle: {},{} {}×{} original pixels.",
        explicit_rect[0], explicit_rect[1], explicit_rect[2], explicit_rect[3]
    ));
    ui.small("Dark-looking pixels may be source-opaque or transparent canvas. Use tools/inspect_elizawy_source_alpha.py to verify this exact PNG region before changing any art.");
    if state.active_layer == LayerId::Ground && !sheet_path.starts_with("Terrain/") {
        ui.colored_label(egui::Color32::YELLOW,
            "Non-terrain artwork cannot replace Ground. Select Objects, Structures or Foreground on the permanent right layer rail.");
    } else {
        ui.small("Manual visual placement, source-exact and unapproved. It does not certify a complete multi-cell asset or gameplay rules.");
    }
    egui::ScrollArea::both()
        .id_salt("terrain.mapper.source.scroll")
        .max_height(max_height)
        .show(ui, |ui| {
            let size = egui::vec2(sheet_size[0] as f32 * zoom, sheet_size[1] as f32 * zoom);
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
            // Checker is outside original artwork: transparent PNG pixels visibly
            // differ from opaque source-black while inspecting exact regions.
            let checker_size = (32.0 * zoom).max(4.0);
            let checker = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
            let cols = (size.x / checker_size).ceil() as usize;
            let rows = (size.y / checker_size).ceil() as usize;
            for cy in 0..rows {
                for cx in 0..cols {
                    let shade = if (cx + cy) % 2 == 0 { 69 } else { 112 };
                    checker.rect_filled(
                        egui::Rect::from_min_size(
                            rect.min
                                + egui::vec2(cx as f32 * checker_size, cy as f32 * checker_size),
                            egui::Vec2::splat(checker_size),
                        ),
                        0.0,
                        egui::Color32::from_gray(shade),
                    );
                }
            }
            if let Some(texture) = sheet_texture {
                ui.painter().image(
                    texture,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }

            let cell_px = 32.0 * zoom;
            let grid_stroke = egui::Stroke::new(0.5, egui::Color32::from_white_alpha(45));
            for gx in 0..=sheet_size[0] / 32 {
                let x = rect.left() + gx as f32 * cell_px;
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    grid_stroke,
                );
            }
            for gy in 0..=sheet_size[1] / 32 {
                let y = rect.top() + gy as f32 * cell_px;
                ui.painter().line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    grid_stroke,
                );
            }

            if response.clicked() || response.drag_started() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    let source = (pointer - rect.min) / zoom;
                    // Only complete 32x32 contract cells are selectable. Some catalog
                    // sheets have a narrow remainder strip that is not a valid recipe cell.
                    let max_gx = sheet_size[0].saturating_sub(32) / 32;
                    let max_gy = sheet_size[1].saturating_sub(32) / 32;
                    let gx = (source.x.max(0.0) as u32 / 32).min(max_gx);
                    let gy = (source.y.max(0.0) as u32 / 32).min(max_gy);
                    state.selected_cell = [gx, gy];
                    if response.drag_started()
                        && (state.active_layer != LayerId::Ground
                            || sheet_path.starts_with("Terrain/"))
                    {
                        state.dragging = Some((sheet_index, selected_source_rect(state)));
                    }
                }
            }

            let selected = egui::Rect::from_min_size(
                rect.min
                    + egui::vec2(
                        state.selected_cell[0] as f32 * cell_px,
                        state.selected_cell[1] as f32 * cell_px,
                    ),
                egui::vec2(
                    explicit_rect[2] as f32 * zoom,
                    explicit_rect[3] as f32 * zoom,
                ),
            );
            ui.painter().rect_stroke(
                selected,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::YELLOW),
                egui::StrokeKind::Inside,
            );
        });

    ui.label(format!(
        "Selected source cell ({}, {}) · {}",
        state.selected_cell[0], state.selected_cell[1], sheet_path
    ));
    if sheet_size[0] % 32 != 0 || sheet_size[1] % 32 != 0 {
        ui.small(format!(
            "Sheet edge remainder: {}×{} px; only complete 32×32 contract cells are selectable.",
            sheet_size[0] % 32,
            sheet_size[1] % 32
        ));
    }
    if let Some(evidence) = state
        .terrain_mapper
        .source_cell_evidence(&sheet_path, state.selected_cell)
        .cloned()
    {
        let fingerprint = evidence.rgba_sha256.chars().take(12).collect::<String>();
        ui.small(format!(
            "Recovered source evidence: {} · alpha pixels {} · {}…",
            evidence.recovery_state, evidence.alpha_pixels, fingerprint
        ));
    } else if sheet_path == state.terrain_mapper.summer_source.source.path {
        ui.small("No recovered evidence record for this Summer source cell.");
    }

    ui.group(|ui| {
        ui.strong(format!(
            "Canvas selection ({}, {})",
            state.selected_world[0], state.selected_world[1]
        ));
        if let Some(index) = state
            .scene
            .index(state.selected_world[0], state.selected_world[1])
        {
            let tile = state.layered_scene.resolved_legacy_tile(index);
            if let Some((path, rect)) = selected_scene_source(state, state.selected_world) {
                ui.label(format!(
                    "Selected {:?} source: {} @ {},{} {}×{}",
                    state.active_layer, path, rect[0], rect[1], rect[2], rect[3]
                ));
            } else {
                ui.small("No original-source image at this selected layer/cell.");
            }
            if let Some(tile) = tile {
                ui.small(format!(
                    "Inherited v1 hint (not collision authority): {}",
                    tile.semantic_role_hint
                ));
            }
            let canvas_pos = state.selected_world;
            let current_source = selected_source_rect(state);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Locate canvas source").clicked() {
                    locate_selected_canvas_source(state);
                }
                if ui
                    .add_enabled(
                        state.active_layer != LayerId::Ground || sheet_path.starts_with("Terrain/"),
                        egui::Button::new("Place current source on selected visual layer"),
                    )
                    .clicked()
                {
                    apply_source(state, canvas_pos, sheet_index, current_source);
                }
                if ui.button("Erase canvas cell").clicked() {
                    erase_source(state, canvas_pos);
                }
            });
        }
    });
    sheet_path
}

fn record_semantic_edit(
    state: &mut Studio,
    before_cells: Vec<bool>,
    dirty_vertices: Vec<terrain::TerrainVertexCoord>,
) {
    state.semantic_undo.push(SemanticHistoryEntry {
        cells: before_cells,
        dirty_vertices: dirty_vertices.clone(),
    });
    if state.semantic_undo.len() > 64 {
        state.semantic_undo.remove(0);
    }
    state.semantic_redo.clear();
    state.last_dirty_vertices = dirty_vertices;
    state.semantic_lab_dirty = true;
}

fn undo_semantic_edit(state: &mut Studio) {
    let Some(previous) = state.semantic_undo.pop() else {
        state.terrain_mapper.message = "Semantic lab undo history is empty.".into();
        return;
    };
    let current = SemanticHistoryEntry {
        cells: state.semantic_lab.cells.clone(),
        dirty_vertices: previous.dirty_vertices.clone(),
    };
    state.semantic_redo.push(current);
    if state.semantic_redo.len() > 64 {
        state.semantic_redo.remove(0);
    }
    state.semantic_lab.cells = previous.cells;
    state.last_dirty_vertices = previous.dirty_vertices;
    state.semantic_lab_dirty = true;
    state.terrain_mapper.message = "Undid semantic resolver-lab edit.".into();
}

fn redo_semantic_edit(state: &mut Studio) {
    let Some(next) = state.semantic_redo.pop() else {
        state.terrain_mapper.message = "Semantic lab redo history is empty.".into();
        return;
    };
    let current = SemanticHistoryEntry {
        cells: state.semantic_lab.cells.clone(),
        dirty_vertices: next.dirty_vertices.clone(),
    };
    state.semantic_undo.push(current);
    if state.semantic_undo.len() > 64 {
        state.semantic_undo.remove(0);
    }
    state.semantic_lab.cells = next.cells;
    state.last_dirty_vertices = next.dirty_vertices;
    state.semantic_lab_dirty = true;
    state.terrain_mapper.message = "Redid semantic resolver-lab edit.".into();
}

fn draw_semantic_resolver_lab(ui: &mut egui::Ui, state: &mut Studio) {
    ui.separator();
    ui.collapsing("Live semantic resolver lab", |ui| {
        let coverage = state.semantic_lab.topology_coverage();
        let covered = coverage.into_iter().filter(|covered| *covered).count();
        ui.label(format!(
            "Semantic-only {}×{} working lab · {covered}/16 topology masks present · no atlas coordinates in the lab document",
            state.semantic_lab.size[0], state.semantic_lab.size[1]
        ));
        ui.small("Click cells to toggle Grass/Void. A semantic edit invalidates exactly the four surrounding dual-grid vertices. The resolved-vertex grid below is produced through the shared runtime-facing resolver contract, not separate UI-only topology math.");
        ui.small(format!(
            "Working copy: {} · checked-in 16-mask seed remains untouched",
            state.semantic_lab_path.display()
        ));

        egui::Grid::new("dg.semantic.lab.cells")
            .spacing(egui::vec2(3.0, 3.0))
            .show(ui, |ui| {
                for y in 0..state.semantic_lab.size[1] {
                    for x in 0..state.semantic_lab.size[0] {
                        let is_grass = state.semantic_lab.get(x as i32, y as i32);
                        let label = if is_grass { "G" } else { "·" };
                        let response = ui.add_sized(
                            egui::vec2(28.0, 24.0),
                            egui::Button::new(label).selected(is_grass),
                        );
                        if response.clicked() {
                            let before = state.semantic_lab.cells.clone();
                            if state.semantic_lab.toggle(x, y) {
                                let vertices = dirty_vertices_for_cell(x, y).to_vec();
                                record_semantic_edit(state, before, vertices.clone());
                                state.terrain_mapper.message = format!(
                                    "Semantic lab cell ({x},{y}) toggled; exact dirty vertices ({},{}), ({},{}), ({},{}), ({},{}).",
                                    vertices[0].x,
                                    vertices[0].y,
                                    vertices[1].x,
                                    vertices[1].y,
                                    vertices[2].x,
                                    vertices[2].y,
                                    vertices[3].x,
                                    vertices[3].y,
                                );
                            }
                        }
                    }
                    ui.end_row();
                }
            });

        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(!state.semantic_undo.is_empty(), egui::Button::new("Undo semantic"))
                .clicked()
            {
                undo_semantic_edit(state);
            }
            if ui
                .add_enabled(!state.semantic_redo.is_empty(), egui::Button::new("Redo semantic"))
                .clicked()
            {
                redo_semantic_edit(state);
            }
            if state.last_dirty_vertices.is_empty() {
                ui.small("Dirty-vertex tracker: no semantic edit in this session.");
            } else {
                let dirty = state
                    .last_dirty_vertices
                    .iter()
                    .map(|vertex| format!("({}, {})", vertex.x, vertex.y))
                    .collect::<Vec<_>>()
                    .join(" ");
                ui.small(format!("Last semantic invalidation: {dirty}"));
            }
        });

        let resolved_grid = resolve_semantic_lab(&state.semantic_lab, &state.terrain_mapper);
        let resolved_coverage = resolved_grid
            .mask_coverage()
            .into_iter()
            .filter(|covered| *covered)
            .count();
        ui.separator();
        ui.strong("Resolved dual-grid output");
        ui.label(format!(
            "{} vertices · {resolved_coverage}/16 masks represented · {} classified outputs · {} certified outputs",
            resolved_grid.vertices.len(),
            resolved_grid.classified_count(),
            resolved_grid.certified_count()
        ));
        ui.small("Each button is one terrain vertex. `*` marks vertices invalidated by the last semantic edit. Clicking a vertex selects both the resolver output and its recipe mask.");

        egui::Grid::new("dg.semantic.resolved.vertices")
            .spacing(egui::vec2(3.0, 3.0))
            .show(ui, |ui| {
                for y in 0..=state.semantic_lab.size[1] {
                    for x in 0..=state.semantic_lab.size[0] {
                        let Some(vertex) = resolved_grid.vertex(x, y) else {
                            continue;
                        };
                        let dirty = state.last_dirty_vertices.contains(&vertex.coord);
                        let selected = state.selected_lab_vertex == [x, y];
                        let glyph = recipe_status_glyph(
                            &vertex.resolution,
                            vertex.confidence.as_deref(),
                        );
                        let label = format!(
                            "{}{} {}",
                            if dirty { "*" } else { "" },
                            glyph,
                            vertex.recipe_key()
                        );
                        let response = ui.add_sized(
                            egui::vec2(62.0, 25.0),
                            egui::Button::new(label).selected(selected),
                        );
                        if response.clicked() {
                            state.selected_lab_vertex = [x, y];
                            state.terrain_mapper.selected_mask = vertex.mask.0;
                        }
                        response.on_hover_text(format!(
                            "vertex ({x},{y}) · {} · {} · {}",
                            mask_meaning(vertex.mask.0),
                            vertex.resolution,
                            vertex.confidence.as_deref().unwrap_or("unreviewed")
                        ));
                    }
                    ui.end_row();
                }
            });

        state.selected_lab_vertex[0] = state.selected_lab_vertex[0].min(state.semantic_lab.size[0]);
        state.selected_lab_vertex[1] = state.selected_lab_vertex[1].min(state.semantic_lab.size[1]);
        ui.horizontal_wrapped(|ui| {
            ui.label("Inspect vertex");
            ui.add(
                egui::DragValue::new(&mut state.selected_lab_vertex[0])
                    .range(0..=state.semantic_lab.size[0])
                    .prefix("X "),
            );
            ui.add(
                egui::DragValue::new(&mut state.selected_lab_vertex[1])
                    .range(0..=state.semantic_lab.size[1])
                    .prefix("Y "),
            );
        });
        let mask = state.semantic_lab.mask_at_vertex(
            state.selected_lab_vertex[0],
            state.selected_lab_vertex[1],
        );
        let key = format!("{mask:04b}");
        let resolved = state.terrain_mapper.family.states.get(&key).cloned();
        ui.horizontal(|ui| {
            draw_mask_preview(ui, mask);
            ui.vertical(|ui| {
                ui.strong(format!("Resolver output {key}"));
                ui.label(mask_meaning(mask));
                ui.label(format!(
                    "Recipe: {} · {}",
                    resolved
                        .as_ref()
                        .map(|entry| entry.resolution.as_str())
                        .unwrap_or("missing"),
                    resolved
                        .as_ref()
                        .and_then(|entry| entry.confidence.as_deref())
                        .unwrap_or("unreviewed")
                ));
                if ui.button("Edit this mask in recipe").clicked() {
                    state.terrain_mapper.selected_mask = mask;
                }
                let save_label = if state.semantic_lab_dirty {
                    "Save local semantic lab *"
                } else {
                    "Save local semantic lab"
                };
                if ui.button(save_label).clicked() {
                    match state.semantic_lab.save(&state.semantic_lab_path) {
                        Ok(()) => {
                            state.semantic_lab_dirty = false;
                            state.terrain_mapper.message = format!(
                                "Saved semantic-only resolver lab to {}",
                                state.semantic_lab_path.display()
                            );
                        }
                        Err(error) => {
                            state.terrain_mapper.message =
                                format!("Semantic lab save failed: {error}");
                        }
                    }
                }
                if ui.button("Reset from 16-mask seed").clicked() {
                    match SemanticTerrainLab::load(&state.semantic_lab_seed_path) {
                        Ok(seed) => {
                            let before = state.semantic_lab.cells.clone();
                            let dirty_vertices = all_vertex_coords(state.semantic_lab.size);
                            state.semantic_lab = seed;
                            record_semantic_edit(state, before, dirty_vertices);
                            state.selected_lab_vertex = [0, 0];
                            state.terrain_mapper.message =
                                "Reset working semantic lab from the checked-in 16-mask seed; all resolver vertices invalidated; Save all to persist the local copy.".into();
                        }
                        Err(error) => {
                            state.terrain_mapper.message =
                                format!("Semantic lab seed reload failed: {error}");
                        }
                    }
                }
            });
            ui.vertical(|ui| {
                ui.small("Resolved source preview");
                draw_recipe_output_preview(ui, state, resolved.as_ref());
            });
        });
    });
}

fn draw_recipe_mapper(ui: &mut egui::Ui, state: &mut Studio, source_path: &str) {
    ui.separator();
    ui.heading("Summer Terrain Authority");
    ui.small("The normal World brush is source-driven. Core terrain topology now resolves through Jaidyn/Tiled Wang metadata and maps back to exact canonical ElizaWy source pixels; unsupported legacy roles fail closed to the prior source-backed resolver. Source PNGs remain read-only.");

    let summary = state.asset_authority.summer_source_catalog_summary();
    let counts = state.asset_authority.summer_flatworld_pair_counts();
    let total = state.asset_authority.summer_flatworld_corner_count();
    let object_bounds = state
        .asset_authority
        .locally_certified_object_template_count();

    egui::CollapsingHeader::new("Summer source authority")
        .id_salt("havenwild.terrain.rules.summer.source.authority")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!(
                    "{}/{} non-transparent source cells mapped",
                    summary.mapped_non_transparent_cells, summary.non_transparent_cells
                ));
                ui.separator();
                ui.label(format!(
                    "{}/{} runtime cells",
                    state.asset_authority.summer_source_runtime_cell_count(),
                    summary.non_transparent_cells
                ));
                ui.separator();
                ui.label(format!("{} source groups", state.asset_authority.summer_source_group_count()));
                ui.separator();
                ui.label(format!("{} authored topology families", summary.topology_families));
            });
            ui.small(format!(
                "Full sheet: {} cells · {} structural transparent · {} unused transparent. Cliffs/elevation and special overlays remain cataloged but are not silently injected into the flat terrain brush.",
                summary.total_cells,
                summary.structural_transparent_cells,
                summary.unused_transparent_cells
            ));
            if let Some(tiled) = state.tiled_terrain.as_ref() {
                let (mapped, authored) = tiled.mapped_fraction();
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(
                        egui::Color32::LIGHT_GREEN,
                        format!("Tiled live: {mapped}/{authored} Terrain Wang assignments have exact canonical pixel bindings"),
                    );
                    ui.separator();
                    ui.label(format!(
                        "animation frames indexed {}/{} canonical · clips {} complete / {} incomplete",
                        tiled.canonical_animation_frame_count,
                        tiled.animation_frame_count,
                        tiled.canonical_animation_clip_count,
                        tiled.incomplete_animation_clip_count
                    ));
                });
                ui.small("Live core mapping: Grass → Grass · Dirt/Bank → Dirt · Sand → Sand · Shallow Water → Shallow Water · Deep Water → Deep Water. Wet/tidal sand and pebble/path remain evidence-gated rather than guessed. Complete canonical mixed water-boundary TSX clips play on one synchronized environmental clock; homogeneous water fills stay static, coast/river/pond cadence is deliberately slowed, and incomplete clips fail closed to their canonical still frame.");
                if !state.world_visual_cache.tiled_vertex_tiles.is_empty() {
                    ui.colored_label(
                        egui::Color32::LIGHT_GREEN,
                        format!(
                            "Shared native-Wang grid active: {} materialized render vertices",
                            state.world_visual_cache.tiled_vertex_tiles.len()
                        ),
                    );
                } else if let Some(error) = state.world_visual_cache.tiled_solve_error.as_deref() {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!(
                            "Shared native-Wang grid fell back locally: {error}"
                        ),
                    );
                }
            } else {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    "Tiled live terrain bridge unavailable; Studio is using the legacy source-backed resolver. Run Full Gate to hydrate/verify the pinned configured TSX lane.",
                );
            }
        });

    egui::CollapsingHeader::new("World-paint mappings")
        .id_salt("havenwild.terrain.rules.summer.paint.mappings")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("Flat Summer states: {total}/81"));
                ui.separator();
                ui.label(format!("Grass↔Dirt {}/14", counts[0]));
                ui.separator();
                ui.label(format!("Grass↔Water {}/14", counts[1]));
                ui.separator();
                ui.label(format!("Dirt↔Water {}/14", counts[2]));
            });
            if total == 81 && counts == [14, 14, 14] {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "Complete: all 81 Grass/Dirt/Water four-corner states resolve from exact Summer source pixels, including 36 three-material junctions. Diagonal checkerboards keep isolated land/detail spots disconnected.");
            } else {
                ui.colored_label(egui::Color32::LIGHT_RED, "Incomplete runtime authority. Run PCC → Terrain → ONE-PASS Native → Bevy Summer authority convergence before evaluating autotiling.");
            }
            ui.horizontal_wrapped(|ui| {
                ui.label(format!(
                    "Interior variants: Grass {} · Dirt {} · Water {} conservative base",
                    state.asset_authority.summer_flatworld_fill_variant_count("Grass"),
                    state.asset_authority.summer_flatworld_fill_variant_count("MudBank"),
                    state.asset_authority.summer_flatworld_fill_variant_count("RiverWater")
                ));
                ui.separator();
                ui.colored_label(egui::Color32::LIGHT_GREEN, "Water animation: mixed water boundaries only · synchronized clock · coast 4x / river 5x / pond 7x cadence");
            });
            ui.small("Legacy RepeatableFill/detail water cells remain static. Configured TSX clips play only when every authored frame maps back to canonical ElizaWy pixels; partial/unmapped clips remain static. Cliffs remain a separate structural authority; waterfalls follow the cliff pass.");
            egui::Grid::new("summer.flatworld.safe.fills")
                .num_columns(3)
                .spacing(egui::vec2(14.0, 4.0))
                .show(ui, |ui| {
                    for (role, label) in [("Grass", "Grass"), ("MudBank", "Dirt / bank"), ("RiverWater", "Shallow water")] {
                        ui.strong(label);
                        if let Some((path, rect)) = state
                            .asset_authority
                            .summer_flatworld_fill(role)
                            .map(|entry| (entry.source_path.clone(), entry.source_rect_px))
                        {
                            ui.monospace(format!("{} @ {},{}", path, rect[0], rect[1]));
                            if ui.button("Locate").clicked() {
                                state.pending_source_sheet = Some(path);
                                state.selected_cell = [rect[0] / 32, rect[1] / 32];
                                state.selected_extent = [1, 1];
                            }
                        } else {
                            ui.colored_label(egui::Color32::LIGHT_RED, "missing");
                            ui.label("");
                        }
                        ui.end_row();
                    }
                });
        });

    egui::CollapsingHeader::new("Objects / crop safety")
        .id_salt("havenwild.terrain.rules.object.crop.safety")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(format!("Source-reviewed large object crops enabled: {object_bounds}"));
            ui.small("Full Gate certifies v3 edge-guide-safe render bounds from immutable source alpha for all seven promoted trees and both grassland rock/stone templates. Atlas slots remain provenance/placement authority; only the exact visible render crop is sampled. Source PNGs are never edited.");
        });

    egui::CollapsingHeader::new("Advanced: manual DG / evidence tools")
        .id_salt("havenwild.terrain.rules.advanced.dg")
        .default_open(false)
        .show(ui, |ui| {
            ui.small("Normal world painting should not require manual recipe certification. Use this only to inspect evidence or author a genuinely new terrain family.");
            draw_advanced_recipe_mapper(ui, state, source_path);
        });
}

fn draw_advanced_recipe_mapper(ui: &mut egui::Ui, state: &mut Studio, source_path: &str) {
    ui.separator();
    ui.heading("DG recipe certification · Grass / Void");
    let (classified, sprites, composites, unsupported, unmapped) = state.terrain_mapper.counts();
    let certified = state.terrain_mapper.certified_count();
    ui.label(format!(
        "Legacy Grass/Void lab: {classified}/16 classified · {certified}/16 certified · sprites {sprites} · composites {composites} · unsupported {unsupported} · unmapped {unmapped}"
    ));
    ui.small("Legacy diagnostic only. Production terrain authority is the Pattern Matrix above: live Summer corner recipes, owner-side transition families, shoreline visual bridges, and semantic fills.");
    egui::CollapsingHeader::new("How Terrain Rules works")
        .id_salt("havenwild.terrain.rules.help")
        .show(ui, |ui| {
            ui.label("The semantic brush sets Grass or Void per cell. Four adjacent cells form one dual-grid vertex, producing one of 16 four-corner masks.");
            ui.monospace("NW=0001  NE=0010  SW=0100  SE=1000");
            ui.label("For each mask, select an exact original Terrain sprite or an explicitly assembled source-pixel composite. Review and certify that recipe separately.");
            ui.label("The source crosswalk only identifies corresponding artwork; it does not choose this recipe, certify joins, or define collision and elevation.");
            ui.small("Recipe changes affect only the Grass/Void test family in the current lab. Use COL for independent scene collision/height overrides. This is not yet the full world terrain-family brush.");
        });

    egui::Grid::new("dg01.mask.grid")
        .num_columns(4)
        .spacing(egui::vec2(5.0, 5.0))
        .show(ui, |ui| {
            for mask in 0u8..16 {
                let key = format!("{mask:04b}");
                let entry = state.terrain_mapper.family.states.get(&key);
                let resolution = entry
                    .map(|entry| entry.resolution.as_str())
                    .unwrap_or("unmapped");
                let confidence = entry.and_then(|entry| entry.confidence.as_deref());
                let label = format!("{} {}", recipe_status_glyph(resolution, confidence), key);
                let response =
                    ui.selectable_label(state.terrain_mapper.selected_mask == mask, label);
                if response.clicked() {
                    state.terrain_mapper.selected_mask = mask;
                }
                response.on_hover_text(format!(
                    "{} · {} · {} / {}",
                    key,
                    mask_meaning(mask),
                    resolution,
                    confidence.unwrap_or("unreviewed")
                ));
                if mask % 4 == 3 {
                    ui.end_row();
                }
            }
        });

    let selected_mask = state.terrain_mapper.selected_mask;
    let selected_key = state.terrain_mapper.selected_key();
    let selected_state = state.terrain_mapper.selected_state().cloned();
    ui.horizontal(|ui| {
        draw_mask_preview(ui, selected_mask);
        ui.vertical(|ui| {
            ui.strong(format!("Mask {selected_key}"));
            ui.label(mask_meaning(selected_mask));
            if let Some(entry) = &selected_state {
                ui.label(format!("Resolution: {}", entry.resolution));
                ui.label(format!(
                    "Confidence: {}",
                    entry.confidence.as_deref().unwrap_or("unreviewed")
                ));
                if let Some(source) = &entry.source {
                    ui.label(format!(
                        "Source: {} @ {},{} {}×{}",
                        source.path, source.rect[0], source.rect[1], source.rect[2], source.rect[3]
                    ));
                }
                if entry.resolution == "composite" {
                    ui.label(format!("Composite parts: {}", entry.parts.len()));
                }
                if let Some(note) = &entry.note {
                    ui.small(note);
                }
            }
        });
        ui.vertical(|ui| {
            ui.small("Resolved source preview");
            draw_recipe_output_preview(ui, state, selected_state.as_ref());
        });
    });

    let source_rect = [
        state.selected_cell[0] * 32,
        state.selected_cell[1] * 32,
        32,
        32,
    ];
    ui.label(format!(
        "Current atlas selection: {} @ {},{} 32×32",
        source_path, source_rect[0], source_rect[1]
    ));
    // The browser covers all original PNGs; this *terrain-only* certification
    // panel may never accidentally promote Structures, Objects or FX art.
    let terrain_selection = source_path.starts_with("Terrain/")
        && state.source_catalog.contains_rect(source_path, source_rect);
    if !terrain_selection {
        ui.small("Terrain recipe actions require an in-bounds original Terrain sheet selection. Other image families remain available for layered visual authoring only.");
    }

    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                terrain_selection,
                egui::Button::new("Assign selected as sprite"),
            )
            .clicked()
        {
            state
                .terrain_mapper
                .assign_sprite(source_path.to_owned(), source_rect);
        }
        if ui.button("Mark unsupported").clicked() {
            state.terrain_mapper.mark_unsupported();
        }
        if ui.button("Clear mapping").clicked() {
            state.terrain_mapper.clear_selected();
        }
        if ui.button("Next unresolved").clicked()
            && state.terrain_mapper.next_unresolved().is_none()
        {
            state.terrain_mapper.message = "All 16 topology states are classified.".into();
        }
        if ui.button("Locate mapped sprite").clicked() {
            locate_selected_recipe_source(state);
        }
    });

    ui.horizontal_wrapped(|ui| {
        ui.label("Review state:");
        for (confidence, label) in [
            ("candidate", "Candidate"),
            ("reviewed", "Reviewed"),
            ("certified", "Certified"),
        ] {
            let selected = selected_state
                .as_ref()
                .and_then(|entry| entry.confidence.as_deref())
                == Some(confidence);
            if ui.selectable_label(selected, label).clicked() {
                state.terrain_mapper.set_selected_confidence(confidence);
            }
        }
        ui.small("Any source/composite edit demotes the mapping back to candidate.");
    });

    ui.collapsing("Composite recipe parts", |ui| {
        ui.label("Use composites when one source sprite cannot represent a topology, especially diagonal states.");
        ui.horizontal(|ui| {
            ui.label("New-part offset px");
            ui.add(egui::DragValue::new(&mut state.terrain_mapper.composite_offset[0]).prefix("X "));
            ui.add(egui::DragValue::new(&mut state.terrain_mapper.composite_offset[1]).prefix("Y "));
            if ui.add_enabled(terrain_selection, egui::Button::new("Add selected tile as part")).clicked() {
                state
                    .terrain_mapper
                    .add_composite_part(source_path.to_owned(), source_rect);
            }
        });

        let parts = state
            .terrain_mapper
            .selected_state()
            .filter(|entry| entry.resolution == "composite")
            .map(|entry| entry.parts.clone())
            .unwrap_or_default();
        if parts.is_empty() {
            ui.small("No composite parts on this mask.");
        }
        for (index, part) in parts.iter().enumerate() {
            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong(format!("Part {}", index + 1));
                    ui.label(format!(
                        "{} @ {},{} {}×{}",
                        part.source.path,
                        part.source.rect[0],
                        part.source.rect[1],
                        part.source.rect[2],
                        part.source.rect[3]
                    ));
                });
                let mut offset = part.offset;
                ui.horizontal_wrapped(|ui| {
                    ui.label("Offset");
                    let changed_x = ui.add(egui::DragValue::new(&mut offset[0]).prefix("X ")).changed();
                    let changed_y = ui.add(egui::DragValue::new(&mut offset[1]).prefix("Y ")).changed();
                    if changed_x || changed_y {
                        state.terrain_mapper.set_composite_part_offset(index, offset);
                    }
                    if ui.button("Locate").clicked() {
                        locate_recipe_source(
                            state,
                            &part.source,
                            &format!("mask {selected_key} composite part {}", index + 1),
                        );
                    }
                    if ui.add_enabled(terrain_selection, egui::Button::new("Replace from selection")).clicked() {
                        state.terrain_mapper.replace_composite_part(
                            index,
                            source_path.to_owned(),
                            source_rect,
                        );
                    }
                    if ui.button("Remove").clicked() {
                        state.terrain_mapper.remove_composite_part(index);
                    }
                });
            });
        }
    });

    draw_semantic_resolver_lab(ui, state);

    ui.horizontal_wrapped(|ui| {
        let can_undo = state.terrain_mapper.can_undo();
        let can_redo = state.terrain_mapper.can_redo();
        if ui
            .add_enabled(can_undo, egui::Button::new("Undo recipe"))
            .clicked()
        {
            state.terrain_mapper.undo_edit();
        }
        if ui
            .add_enabled(can_redo, egui::Button::new("Redo recipe"))
            .clicked()
        {
            state.terrain_mapper.redo_edit();
        }
        let save_text = if state.terrain_mapper.dirty {
            "Save recipe metadata *"
        } else {
            "Save recipe metadata"
        };
        if ui.button(save_text).clicked() {
            if let Err(error) = state.terrain_mapper.save() {
                state.terrain_mapper.message = format!("Recipe save failed: {error}");
            }
        }
        ui.label(&state.terrain_mapper.message);
    });
}

// Evidence browser is deliberately an adjacent mode of the existing dockable mapper.
// It does not modify source art, source-map recovery, Tiled data, or recipe certification.
fn draw_elizawy_candidate(ui: &mut egui::Ui, state: &Studio, candidate: &ReviewCandidate) -> bool {
    let source = TerrainRecipeSource {
        path: candidate.source_sheet.clone(),
        rect: [
            candidate.source_cell[0] * 32,
            candidate.source_cell[1] * 32,
            32,
            32,
        ],
    };
    let mut locate = false;
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(88.0, 88.0), egui::Sense::hover());
            ui.painter()
                .rect_filled(rect, 3.0, egui::Color32::from_gray(24));
            if let Some(sheet) = state.sheets.iter().find(|s| s.path == source.path) {
                if let Some(texture) = sheet.texture {
                    ui.painter().image(
                        texture,
                        rect.shrink(4.0),
                        source_uv(source.rect, sheet.size),
                        egui::Color32::WHITE,
                    );
                }
            }
            ui.vertical(|ui| {
                ui.strong(format!(
                    "{} · {},{}",
                    candidate.source_sheet, candidate.source_cell[0], candidate.source_cell[1]
                ));
                ui.small(format!(
                    "Historical Summer group: {}",
                    candidate
                        .historical_summer_group_id
                        .as_deref()
                        .unwrap_or("unknown")
                ));
                ui.small(format!(
                    "Historical Summer role: {}",
                    candidate
                        .historical_summer_role
                        .as_deref()
                        .unwrap_or("unknown")
                ));
                if ui.button("Locate in source / DG editor").clicked() {
                    locate = true;
                }
            });
        });
    });
    locate
}

fn draw_review_rows(ui: &mut egui::Ui, state: &mut Studio, rows: &[(u32, String)]) {
    ui.small(format!(
        "{} matching occupied derivative cells (virtualized list)",
        rows.len()
    ));
    egui::ScrollArea::vertical()
        .id_salt("elizawy.review.rows")
        .max_height(340.0)
        .show_rows(ui, 23.0, rows.len(), |ui, range| {
            for index in range {
                let (tile_id, status) = &rows[index];
                if ui
                    .selectable_label(
                        state.review_browser.selected_tile_id == Some(*tile_id),
                        format!("#{tile_id:04}  {status}"),
                    )
                    .clicked()
                {
                    state.review_browser.selected_tile_id = Some(*tile_id);
                }
            }
        });
}

fn draw_review_selection(ui: &mut egui::Ui, state: &mut Studio) {
    let selected = state
        .review_browser
        .registry
        .as_ref()
        .and_then(|r| r.seasons.get(&state.review_browser.season))
        .and_then(|profile| {
            profile
                .cells
                .iter()
                .find(|cell| Some(cell.derivative_tile_id) == state.review_browser.selected_tile_id)
        })
        .cloned();
    let Some(cell) = selected else {
        ui.small(
            "Select an occupied derivative atlas cell to inspect every original-source candidate.",
        );
        return;
    };
    ui.strong(format!(
        "Derivative cell #{} · ({}, {})",
        cell.derivative_tile_id, cell.derivative_cell[0], cell.derivative_cell[1]
    ));
    ui.small(format!(
        "Evidence: {} · review: {} · certified: NO",
        cell.match_status, cell.review_state
    ));
    if cell.tiled_has_collision_definition {
        ui.small("External Tiled collision object(s) are defined — reference only; not Havenwild collision authority.");
    }
    if !cell.tiled_animation_frames.is_empty() {
        ui.small(format!(
            "External Tiled animation: {} frame(s) — reference only.",
            cell.tiled_animation_frames.len()
        ));
    }
    if let Some(tile_type) = &cell.tiled_tile_type {
        ui.small(format!("External Tiled tile type: {tile_type}"));
    }
    ui.collapsing(
        format!(
            "Derivative Wang metadata ({} set(s))",
            cell.tiled_wang_evidence.len()
        ),
        |ui| {
            for (name, value) in &cell.tiled_wang_evidence {
                ui.small(format!(
                    "{name}: {}",
                    value
                        .get("signature")
                        .map(|v| v.to_string())
                        .unwrap_or_default()
                ));
            }
        },
    );
    if cell.original_candidates.is_empty() {
        ui.label("No pixel-identical cell on this season's ORIGINAL terrain sheet. This is not proof the derivative art is unusable.");
    }
    if cell.original_candidates.len() > 1 {
        ui.colored_label(egui::Color32::YELLOW, "AMBIGUOUS: same RGBA appears at multiple original coordinates. Contextual review is required.");
    }
    let mut locate: Option<TerrainRecipeSource> = None;
    for candidate in &cell.original_candidates {
        if draw_elizawy_candidate(ui, state, candidate) {
            locate = Some(TerrainRecipeSource {
                path: candidate.source_sheet.clone(),
                rect: [
                    candidate.source_cell[0] * 32,
                    candidate.source_cell[1] * 32,
                    32,
                    32,
                ],
            });
        }
    }
    if let Some(source) = locate {
        locate_recipe_source(state, &source, "reviewed original pixel candidate");
        state.editor_layout.terrain_mapper_tab = "recipe".into();
        save_editor_layout(state);
    }
    ui.small("Locate changes ONLY the selected original atlas cell. To create a draft recipe, use the separate Source / DG Recipe tab and its explicit controls. A pixel match never certifies topology or collision.");
}

// Historical source-address recovery is shipped with the Bevy project. The more
// detailed derivative M2C4 registry is optional local evidence, not a prerequisite.
fn draw_historical_source_review(ui: &mut egui::Ui, state: &mut Studio) {
    let Ok(ref mut history) = state.historical_evidence else {
        if let Err(error) = &state.historical_evidence {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Bundled historical mapping evidence could not be validated: {error}"),
            );
        }
        return;
    };
    ui.separator();
    ui.strong(format!(
        "Recovered original source evidence · {} regions · {} source sheets",
        history.audit.source_regions, history.audit.source_sheets
    ));
    ui.small(format!("B48R9 source-address history · {} grouped review cases covering {} season-target comparisons · 0 auto-promotions", history.triage.cluster_count, history.triage.row_count));
    ui.horizontal_wrapped(|ui| {
        ui.label("Season:");
        for item in ["all", "summer", "spring", "autumn", "winter", "winter_ice"] {
            ui.selectable_value(&mut history.season, item.into(), item);
        }
    });
    ui.add(
        egui::TextEdit::singleline(&mut history.filter)
            .hint_text("Find family, sheet or atlas...")
            .desired_width(280.0),
    );
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut history.view,
            "regions".into(),
            "All 1,781 source regions",
        );
        ui.selectable_value(
            &mut history.view,
            "review".into(),
            "284 grouped review cases",
        );
    });
    if history.view == "regions" {
        let matches = history.filtered_regions();
        ui.small(format!("{} matching original source addresses. Exact pixel hashes and target atlas candidates are retained; no semantic promotions.", matches.len()));
        egui::ScrollArea::vertical().id_salt("havenwild.historical.region.list")
            .max_height(180.0).show(ui, |ui| {
                for (index, label) in matches.iter().take(125) {
                    if ui.selectable_label(history.selected_region == Some(*index), label).clicked() {
                        history.selected_region = Some(*index);
                    }
                }
                if matches.len() > 125 { ui.small("Showing first 125 matching regions. Search by source sheet/family for more."); }
            });
        if let Some(region) = history
            .selected_region
            .and_then(|i| history.regions.entries.get(i))
        {
            ui.separator();
            ui.strong(format!(
                "{} @ {:?}",
                region.source_path, region.source_rect_px
            ));
            ui.small(format!(
                "{} · {} · pixel SHA-256: {}",
                region.region_id,
                if region.transparent {
                    "historically transparent"
                } else {
                    "has original pixels"
                },
                region.source_pixel_sha256
            ));
            ui.small("Target atlas identity candidates (not gameplay selection):");
            for target in &region.seasonal_candidates {
                ui.monospace(format!(
                    "{} · {} · {} · {:?}",
                    target.season,
                    historical_evidence::canonical_atlas_label(target.canonical_atlas.as_deref()),
                    target.canonical_match_status,
                    target.exact_canonical_cells
                ));
            }
            ui.small("Historical source address verified; local original-art bytes, semantic purpose, collision and runtime binding require separate certification.");
        }
    } else {
        let matches = history.filtered_rows();
        ui.small(format!(
            "{} grouped review cases · source pixels and approvals are unchanged",
            matches.len()
        ));
        egui::ScrollArea::vertical()
            .id_salt("havenwild.historical.source.queue")
            .max_height(170.0)
            .show(ui, |ui| {
                for (index, label) in matches.iter().take(150) {
                    if ui
                        .selectable_label(history.selected_cluster == Some(*index), label)
                        .clicked()
                    {
                        history.selected_cluster = Some(*index);
                    }
                }
                if matches.len() > 150 {
                    ui.small("Showing first 150 matching cases; narrow the search for the rest.");
                }
            });
        if let Some(cluster) = history
            .selected_cluster
            .and_then(|i| history.triage.clusters.get(i))
        {
            ui.separator();
            ui.strong(format!(
                "{} / {}",
                cluster.season,
                cluster.triage_category.replace('_', " ")
            ));
            ui.small(format!(
                "Canonical comparison atlas: {} · match {}",
                historical_evidence::canonical_atlas_label(cluster.canonical_atlas.as_deref()),
                cluster.canonical_match_status
            ));
            ui.small(format!(
                "{} source regions; no visual or runtime certification",
                cluster.member_count
            ));
            for member in cluster.membership.iter().take(8) {
                ui.monospace(format!(
                    "{} @ {:?} · {}",
                    member.source_path, member.source_rect_px, member.family_hint
                ));
            }
            if cluster.membership.len() > 8 {
                ui.small(
                    "More members preserved in content/mapping/recovered/review_triage.v3.json.",
                );
            }
            ui.small("Original split source sheets and consolidated atlas regions are distinct. Historical pixel identity does not certify terrain joins, collision or recipe selection.");
        }
    }
}

fn draw_elizawy_review_browser(ui: &mut egui::Ui, state: &mut Studio) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("ElizaWy / LPC Revised — evidence review");
        if ui
            .add_enabled(
                !state.review_browser.loading,
                egui::Button::new("Reload local registry"),
            )
            .clicked()
        {
            state.review_browser.reload();
        }
    });
    ui.small("Original ElizaWy atlas remains authoritative. External JaidynReiman four-season Tiled art is read-only comparison evidence; no automatic source or recipe promotions.");
    if state.review_browser.loading {
        ui.spinner();
        ui.label("Loading and validating complete review registry off the editor/UI thread...");
        return;
    }
    if let Some(error) = &state.review_browser.error {
        ui.colored_label(
            egui::Color32::YELLOW,
            "Optional M2C4 derivative registry is not installed in this checkout.",
        );
        ui.small(error);
        ui.small("Recovered B48R9 source addresses are available below. No source or recipe promotion is implied.");
    }
    draw_historical_source_review(ui, state);
    let Some(registry) = &state.review_browser.registry else {
        return;
    };
    let mut season = state.review_browser.season.clone();
    let previous_season = season.clone();
    ui.horizontal_wrapped(|ui| {
        for item in ["summer", "spring", "autumn", "winter"] {
            if ui
                .selectable_label(season == item, item.to_uppercase())
                .clicked()
            {
                season = item.into();
            }
        }
    });
    state.review_browser.season = season;
    if previous_season != state.review_browser.season {
        state.review_browser.selected_tile_id = None;
    }
    let summary = &registry.seasons[&state.review_browser.season].summary;
    ui.label(format!(
        "Occupied {} · unique {} · ambiguous {} · unmatched {} · certified 0",
        summary.occupied_derivative_cells,
        summary.unique_candidate_cells,
        summary.ambiguous_candidate_cells,
        summary.unmatched_derivative_cells
    ));
    if state.review_browser.season == "autumn" {
        ui.colored_label(egui::Color32::YELLOW, "Autumn uses external -old.tsx: 58 missing / 8 extra / 10 changed terrain Wang IDs and no fence set. Do not fill gaps by guessing.");
    }
    if state.review_browser.season == "winter" {
        ui.small("External Winter reuses numeric Wang IDs with different material names. Original winter_ice is a separate source variant, not present in this four-season derivative.");
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("Show:");
        for filter in ["All", "Unique", "Ambiguous", "Unmatched"] {
            ui.selectable_value(
                &mut state.review_browser.match_filter,
                filter.into(),
                filter,
            );
        }
        ui.add(
            egui::TextEdit::singleline(&mut state.review_browser.search)
                .hint_text("Tile ID, group, role, Tiled set/type...")
                .desired_width(205.0),
        );
    });
    let rows = registry.seasons[&state.review_browser.season]
        .cells
        .iter()
        .filter(|cell| state.review_browser.matches_filter(cell))
        .map(|cell| {
            let label = match cell.match_status.as_str() {
                "pixel_unique_candidate" => "unique",
                "pixel_ambiguous_candidates" => "ambiguous",
                _ => "unmatched",
            };
            (cell.derivative_tile_id, label.to_owned())
        })
        .collect::<Vec<_>>();
    ui.separator();
    if ui.available_width() >= 680.0 {
        ui.columns(2, |columns| {
            let (left, right) = columns.split_at_mut(1);
            draw_review_rows(&mut left[0], state, &rows);
            egui::ScrollArea::vertical()
                .id_salt("elizawy.review.details.wide")
                .show(&mut right[0], |ui| draw_review_selection(ui, state));
        });
    } else {
        draw_review_rows(ui, state, &rows);
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("elizawy.review.details.narrow")
            .max_height(320.0)
            .show(ui, |ui| draw_review_selection(ui, state));
    }
}

fn draw_production_pattern_matrix(ui: &mut egui::Ui, state: &mut Studio) {
    ui.heading("Live Terrain Rules");
    ui.small("Select a transition family to locate its source tiles. These are verified lookup rules, not editable classifications. Use the World canvas for painting.");
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(
            egui::Color32::LIGHT_GREEN,
            "Runtime authority: exact ElizaWy source regions",
        );
        ui.separator();
        ui.label(format!(
            "{} source groups · {} mapped Summer cells",
            state.asset_authority.summer_source_group_count(),
            state.asset_authority.summer_source_runtime_cell_count()
        ));
        ui.separator();
        ui.label(format!(
            "Water animation: {} certified frames",
            state.asset_authority.summer_water_animation_frame_count()
        ));
    });
    ui.separator();

    let families = [
        ("summer_grass_over_dirt", "Grass ↔ Dirt"),
        ("summer_grass_over_sand", "Grass ↔ Sand"),
        ("summer_pebble_path_over_dirt", "Pebble Path ↔ Dirt"),
        ("summer_grass_bank_pond_a", "Grass ↔ Water"),
        ("summer_dirt_bank_pond_a", "Dirt ↔ Water"),
        ("summer_sand_bank_pond", "Sand ↔ Water"),
        ("summer_sand_over_wet_sand", "Wet Sand ↔ Dry Sand"),
        ("summer_deep_water_basin", "Deep ↔ Shallow Water"),
    ];
    for (family_id, title) in families {
        let family = state
            .asset_authority
            .summer_owner_side_family(family_id)
            .cloned();
        let Some(family) = family else {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Missing runtime family: {title}"),
            );
            continue;
        };
        egui::CollapsingHeader::new(format!("{title} · {}/12 roles", family.roles.len()))
            .id_salt(format!("terrain.studio.pattern.{family_id}"))
            .default_open(false)
            .show(ui, |ui| {
                ui.small(format!("Owner {} · Neighbor {} · Native rule {}", family.owner, family.neighbor, family.topology_rule));
                let role_order = [
                    "outer_north_west", "outer_north", "outer_north_east",
                    "outer_west", "outer_east",
                    "outer_south_west", "outer_south", "outer_south_east",
                    "inner_north_west", "inner_north_east", "inner_south_west", "inner_south_east",
                ];
                egui::Grid::new(format!("terrain.studio.grid.{family_id}"))
                    .num_columns(3)
                    .spacing(egui::vec2(8.0, 5.0))
                    .show(ui, |ui| {
                        for (index, role) in role_order.into_iter().enumerate() {
                            if let Some(entry) = family.roles.get(role) {
                                let short = role
                                    .replace("outer_", "O ")
                                    .replace("inner_", "I ")
                                    .replace('_', " ");
                                if ui.button(short).on_hover_text(format!("{} @ {},{} · exact 32×32 source", entry.source_path, entry.source_rect_px[0], entry.source_rect_px[1])).clicked() {
                                    state.pending_source_sheet = Some(entry.source_path.clone());
                                    state.selected_cell = [entry.source_rect_px[0] / 32, entry.source_rect_px[1] / 32];
                                    state.selected_extent = [1, 1];
                                    state.message = format!("Terrain Studio located {title} / {role} in immutable ElizaWy source.");
                                }
                            } else {
                                ui.add_enabled(false, egui::Button::new(format!("Missing {role}")));
                            }
                            if index % 3 == 2 {
                                ui.end_row();
                            }
                        }
                    });
            });
    }

    ui.separator();
    let pale = state
        .asset_authority
        .summer_shoreline_overlay_family("summer_pale_shore_ring_overlay")
        .cloned();
    egui::CollapsingHeader::new("Legacy Water ↔ Wet Sand pale-shore reference")
        .id_salt("terrain.studio.pale.shore")
        .default_open(false)
        .show(ui, |ui| {
            if let Some(family) = pale {
                ui.colored_label(egui::Color32::LIGHT_GREEN, format!("Certified {} outer + {} inner source roles", family.outer_role_count, family.inner_role_count));
                ui.small("Reference-only in the live renderer. M2D097A retired this second shoreline repaint so configured Tiled/Wang topology is not redrawn afterward. The exact source family remains browsable for comparison and future Deep Sand/shoreline evidence work.");
                ui.horizontal_wrapped(|ui| {
                    for role in ["outer_north", "outer_east", "outer_south", "outer_west", "inner_north_east", "inner_north_west", "inner_south_east", "inner_south_west"] {
                        if let Some(entry) = family.roles.get(role) {
                            if ui.small_button(role.replace('_', " ")).clicked() {
                                state.pending_source_sheet = Some(entry.source_path.clone());
                                state.selected_cell = [entry.source_rect_px[0] / 32, entry.source_rect_px[1] / 32];
                                state.selected_extent = [1, 1];
                            }
                        }
                    }
                });
            } else {
                ui.colored_label(egui::Color32::LIGHT_RED, "Pale-shore source family missing from runtime authority.");
            }
        });

    ui.separator();
    ui.small("Editing a mapping rule is not yet implemented here. Verified source coordinates remain read-only; the World Editor can already place individual source tiles.");
}

fn draw_mapper_contents(ui: &mut egui::Ui, state: &mut Studio, horizontal: bool) {
    draw_mapper_header(ui, state);
    // A guided companion to the World Editor, not a second competing editing UI.
    // Painting and exact atlas stamping already live in the permanent left frames.
    ui.small("Paint materials and place source tiles in the World Editor. Use this panel only to inspect terrain rules and advanced source evidence.");
    ui.horizontal_wrapped(|ui| {
        if ui.button("Terrain Brush on canvas").clicked() {
            choose_brush(state, BrushMode::Terrain);
            state.editor_layout.authoring_library_tab = "Terrain".into();
            state.editor_layout.terrain_mapper.visible = false;
            save_editor_layout(state);
        }
        if ui.button("Open Tile Library").clicked() {
            choose_brush(state, BrushMode::Tile);
            state.editor_layout.authoring_library_tab = "Tiles".into();
            state.editor_layout.terrain_mapper.visible = false;
            save_editor_layout(state);
        }
    });
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        for (tab, label) in [("patterns", "Live terrain rules"), ("source", "Original atlas")] {
            if ui.selectable_label(state.editor_layout.terrain_mapper_tab == tab, label).clicked() {
                state.editor_layout.terrain_mapper_tab = tab.into();
                save_editor_layout(state);
            }
        }
        ui.menu_button("Advanced ▾", |ui| {
            ui.small("Historical evidence and retired recipe experiments. Not required for normal painting.");
            for (tab, label) in [("review", "Source provenance"), ("recipe", "Legacy recipe diagnostics")] {
                if ui.selectable_label(state.editor_layout.terrain_mapper_tab == tab, label).clicked() {
                    state.editor_layout.terrain_mapper_tab = tab.into();
                    save_editor_layout(state);
                    ui.close();
                }
            }
        });
    });
    ui.separator();
    let active_tab = state.editor_layout.terrain_mapper_tab.clone();
    if state.editor_layout.terrain_mapper_compact {
        match active_tab.as_str() {
            "patterns" => {
                ui.small("Compact production-pattern authority view.");
                draw_production_pattern_matrix(ui, state);
            }
            "source" => {
                ui.small("Compact source picker · exact original pixels remain read-only.");
                draw_source_picker(ui, state, 190.0);
            }
            "review" => {
                ui.small("Compact recovered-evidence view · historical addresses only; no automatic promotion.");
                egui::ScrollArea::vertical()
                    .id_salt("elizawy.review.surface.compact")
                    .max_height(230.0)
                    .show(ui, |ui| draw_historical_source_review(ui, state));
            }
            _ => {
                draw_recovery_summary(ui, state);
                ui.separator();
                ui.small("Compact Terrain Rules shows authority status only. Switch this panel to Full for recipe editing and source-region composition.");
            }
        }
        return;
    }
    match active_tab.as_str() {
        "patterns" => {
            egui::ScrollArea::vertical()
                .id_salt("terrain.studio.patterns.scroll")
                .show(ui, |ui| draw_production_pattern_matrix(ui, state));
            return;
        }
        "source" => {
            ui.small("Original source PNGs · select a complete region, then drag to the canvas. No art is modified.");
            let height = (ui.available_height() - 36.0).clamp(220.0, 360.0);
            draw_source_picker(ui, state, height);
            return;
        }
        "review" => {
            egui::ScrollArea::vertical()
                .id_salt("elizawy.review.surface.scroll")
                .show(ui, |ui| draw_elizawy_review_browser(ui, state));
            return;
        }
        _ => {}
    }
    draw_recovery_summary(ui, state);
    ui.separator();

    if horizontal || ui.available_width() >= 760.0 {
        ui.columns(2, |columns| {
            let (left, right) = columns.split_at_mut(1);
            let source_path = draw_source_picker(&mut left[0], state, 300.0);
            draw_recipe_mapper(&mut right[0], state, &source_path);
        });
    } else {
        egui::ScrollArea::vertical()
            .id_salt("terrain.mapper.narrow.scroll")
            .show(ui, |ui| {
                let source_path = draw_source_picker(ui, state, 360.0);
                draw_recipe_mapper(ui, state, &source_path);
            });
    }
}

fn draw_mapper_surface(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    let ctx = viewport_ui.ctx().clone();
    if !state.editor_layout.terrain_mapper.visible {
        state.atlas_rect = egui::Rect::NOTHING;
        return;
    }

    if state.editor_layout.terrain_mapper.dock == SurfaceDock::Center {
        state.editor_layout.terrain_mapper.dock = SurfaceDock::Floating;
    }
    let dock = state.editor_layout.terrain_mapper.dock;
    let preferred = state.editor_layout.terrain_mapper.preferred_size;
    let compact = state.editor_layout.terrain_mapper_compact;
    let mut open = true;
    let mut window = egui::Window::new("Terrain Studio · ElizaWy Authority")
        .id(egui::Id::new("havenwild.terrain.mapper.overlay"))
        .open(&mut open)
        .resizable(true)
        .min_width(if compact { 300.0 } else { 360.0 })
        .min_height(if compact { 180.0 } else { 260.0 });

    window = match dock {
        SurfaceDock::Left => window
            .anchor(egui::Align2::LEFT_TOP, [60.0, 62.0])
            .default_size(egui::vec2(
                preferred[0].clamp(320.0, 620.0),
                preferred[1].clamp(300.0, 720.0),
            )),
        SurfaceDock::Right => window
            .anchor(egui::Align2::RIGHT_TOP, [-58.0, 62.0])
            .default_size(egui::vec2(
                preferred[0].clamp(320.0, 620.0),
                preferred[1].clamp(300.0, 720.0),
            )),
        SurfaceDock::Bottom => window
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -30.0])
            .default_size(egui::vec2(
                preferred[0].max(760.0),
                preferred[1].clamp(230.0, 520.0),
            )),
        SurfaceDock::Floating => window
            .default_pos(egui::pos2(84.0, 88.0))
            .default_size(egui::vec2(preferred[0].max(410.0), preferred[1].max(470.0))),
        SurfaceDock::Center => unreachable!(),
    };

    let response = window.show(&ctx, |ui| {
        draw_mapper_contents(ui, state, dock == SurfaceDock::Bottom)
    });
    state.atlas_rect = response
        .as_ref()
        .map(|response| response.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if let Some(response) = response {
        state.editor_layout.terrain_mapper.preferred_size = [
            response.response.rect.width(),
            response.response.rect.height(),
        ];
    }
    if !open {
        state.editor_layout.terrain_mapper.visible = false;
        save_editor_layout(state);
    }
}

fn automatic_cell_blocked(state: &Studio, index: usize) -> bool {
    let x = index % state.scene.size[0];
    let y = index / state.scene.size[0];
    !playtest::provisional_walkable(
        &state.scene,
        &state.layered_scene.structural_cells,
        x as i32,
        y as i32,
    )
}

fn collision_mask_seed(state: &Studio, index: usize) -> CollisionMask32 {
    if automatic_cell_blocked(state, index) {
        CollisionMask32::full()
    } else {
        CollisionMask32::empty()
    }
}

fn set_collision_mask(state: &mut Studio, index: usize, mask: Option<CollisionMask32>) {
    match state.layered_history.execute(
        &mut state.layered_scene,
        SceneCommand::SetCollisionMask { index, after: mask },
    ) {
        Ok(()) => {
            state.message = format!(
                "Updated pixel collision mask for cell {},{}; Save Scene persists it.",
                index % state.scene.size[0],
                index / state.scene.size[0]
            );
        }
        Err(error) if error == "No scene changes" => {}
        Err(error) => state.message = format!("Collision edit not applied: {error}"),
    }
}

fn edit_collision_pixel(state: &mut Studio, index: usize, pixel: [usize; 2], blocked: bool) {
    let mut mask = state
        .layered_scene
        .collision_mask(index)
        .cloned()
        .unwrap_or_else(|| collision_mask_seed(state, index));
    mask.set_blocked(pixel[0], pixel[1], blocked);
    set_collision_mask(state, index, Some(mask));
}

fn collision_pixel_under_pointer(
    state: &Studio,
    pointer: egui::Pos2,
) -> Option<(usize, [usize; 2])> {
    if !state.scene_rect.contains(pointer) || state.zoom <= 0.0 {
        return None;
    }
    let px = state.scene.tile_size as f32 * state.zoom;
    if !px.is_finite() || px <= 0.0 {
        return None;
    }
    let fx = (pointer.x - state.scene_rect.left()) / px;
    let fy = (pointer.y - state.scene_rect.top()) / px;
    if fx < 0.0 || fy < 0.0 {
        return None;
    }
    let cell_x = fx.floor() as usize;
    let cell_y = fy.floor() as usize;
    let index = state.scene.index(cell_x, cell_y)?;
    let local_x = ((fx - cell_x as f32) * COLLISION_MASK_SIDE as f32)
        .floor()
        .clamp(0.0, (COLLISION_MASK_SIDE - 1) as f32) as usize;
    let local_y = ((fy - cell_y as f32) * COLLISION_MASK_SIDE as f32)
        .floor()
        .clamp(0.0, (COLLISION_MASK_SIDE - 1) as f32) as usize;
    Some((index, [local_x, local_y]))
}

fn draw_collision_mask_editor(ui: &mut egui::Ui, state: &mut Studio, index: usize) {
    let explicit = state.layered_scene.collision_mask(index).cloned();
    let mut preview = explicit
        .clone()
        .unwrap_or_else(|| collision_mask_seed(state, index));
    let automatic = automatic_cell_blocked(state, index);
    let blocked = preview.blocked_count();
    ui.small(format!(
        "Pixel mask: {} · {blocked}/1024 blocked pixels · automatic cell = {}",
        if explicit.is_some() {
            "AUTHORED"
        } else {
            "preview from automatic collision"
        },
        if automatic { "blocked" } else { "walkable" }
    ));
    ui.horizontal_wrapped(|ui| {
        if ui
            .selectable_label(state.collision_brush_blocked, "Block brush")
            .on_hover_text("Paint impassable collision pixels red.")
            .clicked()
        {
            state.collision_brush_blocked = true;
        }
        if ui
            .selectable_label(!state.collision_brush_blocked, "Clear brush")
            .on_hover_text("Clear collision pixels so the player can pass through that exact area.")
            .clicked()
        {
            state.collision_brush_blocked = false;
        }
    });
    ui.horizontal_wrapped(|ui| {
        if ui.button("Fill blocked").on_hover_text("Create an explicit fully blocked 32x32 collision mask.").clicked() {
            set_collision_mask(state, index, Some(CollisionMask32::full()));
            preview = CollisionMask32::full();
        }
        if ui.button("Clear all").on_hover_text("Create an explicit fully walkable 32x32 collision mask.").clicked() {
            set_collision_mask(state, index, Some(CollisionMask32::empty()));
            preview = CollisionMask32::empty();
        }
        if ui
            .add_enabled(explicit.is_some(), egui::Button::new("Revert automatic"))
            .on_hover_text("Remove the authored pixel mask and return this cell to coarse/automatic collision rules.")
            .clicked()
        {
            set_collision_mask(state, index, None);
            preview = collision_mask_seed(state, index);
        }
    });

    let side = ui.available_width().min(360.0).clamp(224.0, 360.0);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(26, 31, 34));
    let unit = side / COLLISION_MASK_SIDE as f32;
    for y in 0..COLLISION_MASK_SIDE {
        let mut run_start = None;
        for x in 0..=COLLISION_MASK_SIDE {
            let is_blocked = x < COLLISION_MASK_SIDE && preview.blocked(x, y);
            match (run_start, is_blocked) {
                (None, true) => run_start = Some(x),
                (Some(start), false) => {
                    let r = egui::Rect::from_min_max(
                        rect.min + egui::vec2(start as f32 * unit, y as f32 * unit),
                        rect.min + egui::vec2(x as f32 * unit, (y + 1) as f32 * unit),
                    );
                    painter.rect_filled(
                        r,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(235, 67, 67, 185),
                    );
                    run_start = None;
                }
                _ => {}
            }
        }
    }
    for line in [0usize, 8, 16, 24, 32] {
        let d = line as f32 * unit;
        painter.line_segment(
            [
                rect.min + egui::vec2(d, 0.0),
                rect.min + egui::vec2(d, side),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(44)),
        );
        painter.line_segment(
            [
                rect.min + egui::vec2(0.0, d),
                rect.min + egui::vec2(side, d),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_white_alpha(44)),
        );
    }
    painter.rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(105)),
        egui::StrokeKind::Inside,
    );

    if response.hovered() && ui.input(|input| input.pointer.primary_down()) {
        if let Some(pointer) = ui.ctx().pointer_hover_pos() {
            let local = pointer - rect.min;
            let x = (local.x / unit).floor().clamp(0.0, 31.0) as usize;
            let y = (local.y / unit).floor().clamp(0.0, 31.0) as usize;
            let blocked = state.collision_brush_blocked;
            edit_collision_pixel(state, index, [x, y], blocked);
            ui.ctx().request_repaint();
        }
    }
    ui.small("Red = impassable. Each logical world cell has a 32×32 authored collision mask. Artwork pixels remain unchanged.");
}

fn draw_collision_overlay(
    painter: &egui::Painter,
    state: &Studio,
    scene: egui::Rect,
    px: f32,
    canvas: egui::Rect,
) {
    if !state.editor_layout.collision_overlay || px < 2.0 {
        return;
    }
    let visible = canvas.intersect(scene);
    if visible.width() <= 0.0 || visible.height() <= 0.0 {
        return;
    }
    let p = painter.with_clip_rect(visible);
    let x0 = (((visible.left() - scene.left()) / px).floor() as isize).max(0) as usize;
    let y0 = (((visible.top() - scene.top()) / px).floor() as isize).max(0) as usize;
    let x1 = (((visible.right() - scene.left()) / px).ceil() as usize) + 1;
    let y1 = (((visible.bottom() - scene.top()) / px).ceil() as usize) + 1;
    let x1 = x1.min(state.scene.size[0]);
    let y1 = y1.min(state.scene.size[1]);
    let red = egui::Color32::from_rgba_unmultiplied(240, 54, 54, 82);
    let explicit_outline = egui::Color32::from_rgba_unmultiplied(255, 125, 125, 170);
    for y in y0..y1 {
        for x in x0..x1 {
            let index = y * state.scene.size[0] + x;
            let cell_rect = egui::Rect::from_min_size(
                scene.min + egui::vec2(x as f32 * px, y as f32 * px),
                egui::Vec2::splat(px),
            );
            if let Some(mask) = state.layered_scene.collision_mask(index) {
                let count = mask.blocked_count();
                if count == 1024 {
                    p.rect_filled(cell_rect, 0.0, red);
                } else if count > 0 {
                    let unit = px / COLLISION_MASK_SIDE as f32;
                    for row in 0..COLLISION_MASK_SIDE {
                        let mut start = None;
                        for col in 0..=COLLISION_MASK_SIDE {
                            let hit = col < COLLISION_MASK_SIDE && mask.blocked(col, row);
                            match (start, hit) {
                                (None, true) => start = Some(col),
                                (Some(run), false) => {
                                    let r = egui::Rect::from_min_max(
                                        cell_rect.min
                                            + egui::vec2(run as f32 * unit, row as f32 * unit),
                                        cell_rect.min
                                            + egui::vec2(
                                                col as f32 * unit,
                                                (row + 1) as f32 * unit,
                                            ),
                                    );
                                    p.rect_filled(r, 0.0, red);
                                    start = None;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                p.rect_stroke(
                    cell_rect,
                    0.0,
                    egui::Stroke::new(1.0, explicit_outline),
                    egui::StrokeKind::Inside,
                );
            } else if automatic_cell_blocked(state, index) {
                p.rect_filled(cell_rect, 0.0, red);
            }
        }
    }
}

fn world_collision_seed(state: &Studio, world: [i64; 2]) -> CollisionMask32 {
    state
        .world_doc
        .effective_collision_mask(world, &state.asset_authority)
        .unwrap_or_else(|| {
            if world_automatic_blocked(state, world) {
                CollisionMask32::full()
            } else {
                CollisionMask32::empty()
            }
        })
}

fn paint_world_collision_brush(state: &mut Studio, world: [i64; 2], pixel: [usize; 2]) {
    let radius = state.collision_brush_radius.max(1) as i64;
    let center_x = world[0] * COLLISION_MASK_SIDE as i64 + pixel[0] as i64;
    let center_y = world[1] * COLLISION_MASK_SIDE as i64 + pixel[1] as i64;
    let extent = radius - 1;
    let mut touched: BTreeSet<[i64; 2]> = BTreeSet::new();
    let mut edits: Vec<([i64; 2], usize, usize)> = Vec::new();
    for dy in -extent..=extent {
        for dx in -extent..=extent {
            if dx * dx + dy * dy > extent * extent && radius > 1 {
                continue;
            }
            let gx = center_x + dx;
            let gy = center_y + dy;
            let target_world = [
                gx.div_euclid(COLLISION_MASK_SIDE as i64),
                gy.div_euclid(COLLISION_MASK_SIDE as i64),
            ];
            let target_pixel = [
                gx.rem_euclid(COLLISION_MASK_SIDE as i64) as usize,
                gy.rem_euclid(COLLISION_MASK_SIDE as i64) as usize,
            ];
            touched.insert(target_world);
            edits.push((target_world, target_pixel[0], target_pixel[1]));
        }
    }
    for target_world in touched {
        let mut mask = world_collision_seed(state, target_world);
        for (edit_world, x, y) in edits
            .iter()
            .copied()
            .filter(|(edit_world, _, _)| *edit_world == target_world)
        {
            let _ = edit_world;
            mask.set_blocked(x, y, state.collision_brush_blocked);
        }
        match state.world_behavior_scope {
            WorldBehaviorScope::LocalArea => {
                state
                    .world_doc
                    .set_collision_mask_untracked(target_world, Some(mask));
            }
            WorldBehaviorScope::TerrainTopology => {
                if let Some((role, neighbor_mask)) =
                    state.world_doc.resolved_terrain_behavior_key(target_world)
                {
                    state.world_doc.set_terrain_collision_profile_untracked(
                        role,
                        neighbor_mask,
                        Some(mask),
                    );
                }
            }
        }
    }
    state.world_dirty = true;
}

fn selected_world_source(state: &Studio, world: [i64; 2]) -> Option<SourceBinding> {
    state.world_doc.resolved_source_with_authorities(
        world,
        &state.asset_authority,
        state.tiled_terrain.as_ref(),
    )
}

fn set_world_collision_for_scope(
    state: &mut Studio,
    world: [i64; 2],
    mask: Option<CollisionMask32>,
) {
    checkpoint_world(state);
    match state.world_behavior_scope {
        WorldBehaviorScope::LocalArea => state.world_doc.set_collision_mask(world, mask),
        WorldBehaviorScope::TerrainTopology => {
            if let Some((role, neighbor_mask)) =
                state.world_doc.resolved_terrain_behavior_key(world)
            {
                state
                    .world_doc
                    .set_terrain_collision_profile(role, neighbor_mask, mask);
            } else {
                state.message =
                    "No semantic terrain topology is resolved for this world cell.".into();
                let _ = state.world_undo.pop();
                return;
            }
        }
    }
    state.world_dirty = true;
}

fn set_world_traversal_for_scope(state: &mut Studio, world: [i64; 2], mode: TraversalMode) {
    checkpoint_world(state);
    let value = (mode != TraversalMode::Auto).then_some(mode);
    match state.world_behavior_scope {
        WorldBehaviorScope::LocalArea => state.world_doc.set_traversal_override(world, value),
        WorldBehaviorScope::TerrainTopology => {
            if let Some((role, neighbor_mask)) =
                state.world_doc.resolved_terrain_behavior_key(world)
            {
                state
                    .world_doc
                    .set_terrain_traversal_profile(role, neighbor_mask, value);
            } else {
                state.message =
                    "No semantic terrain topology is resolved for this world cell.".into();
                let _ = state.world_undo.pop();
                return;
            }
        }
    }
    state.world_dirty = true;
}

fn world_collision_scope_mask(state: &Studio, world: [i64; 2]) -> Option<CollisionMask32> {
    match state.world_behavior_scope {
        WorldBehaviorScope::LocalArea => state.world_doc.collision_mask(world).cloned(),
        WorldBehaviorScope::TerrainTopology => state
            .world_doc
            .resolved_terrain_behavior_key(world)
            .and_then(|(role, neighbor_mask)| {
                state
                    .world_doc
                    .terrain_behavior_profile(&role, neighbor_mask)
                    .and_then(|profile| profile.collision.clone())
            }),
    }
}

fn world_traversal_scope_mode(state: &Studio, world: [i64; 2]) -> TraversalMode {
    match state.world_behavior_scope {
        WorldBehaviorScope::LocalArea => state
            .world_doc
            .cell_override(world)
            .and_then(|item| item.traversal)
            .unwrap_or(TraversalMode::Auto),
        WorldBehaviorScope::TerrainTopology => state
            .world_doc
            .resolved_terrain_behavior_key(world)
            .and_then(|(role, neighbor_mask)| {
                state
                    .world_doc
                    .terrain_behavior_profile(&role, neighbor_mask)
                    .and_then(|profile| profile.traversal)
            })
            .unwrap_or(TraversalMode::Auto),
    }
}

fn draw_world_structural_contents(ui: &mut egui::Ui, state: &mut Studio) {
    if !state.selection_active {
        ui.label("Select a world cell with Select (Q) first.");
        return;
    }
    let world = state.world_selected;
    ui.strong(format!("World cell {},{}", world[0], world[1]));
    if let Some(role) = state.world_doc.resolved_role(world) {
        ui.small(format!("Terrain: {role}"));
    }

    ui.horizontal_wrapped(|ui| {
        ui.label("Edit scope");
        for scope in [WorldBehaviorScope::LocalArea, WorldBehaviorScope::TerrainTopology] {
            if ui
                .selectable_label(state.world_behavior_scope == scope, scope.label())
                .on_hover_text(match scope {
                    WorldBehaviorScope::LocalArea => "Override only these world coordinates. Regeneration preserves the exception.",
                    WorldBehaviorScope::TerrainTopology => "Reusable terrain behavior keyed to semantic material + cardinal topology. This avoids leaking shoreline collision through a reused atlas rectangle; ElizaWy PNGs remain immutable.",
                })
                .clicked()
            {
                state.world_behavior_scope = scope;
            }
        }
    });

    egui::CollapsingHeader::new("Traversal")
        .id_salt("havenwild.world.properties.traversal")
        .default_open(true)
        .show(ui, |ui| {
            let current = world_traversal_scope_mode(state, world);
            ui.horizontal_wrapped(|ui| {
                for mode in [
                    TraversalMode::Auto,
                    TraversalMode::Walkable,
                    TraversalMode::Blocked,
                    TraversalMode::Wadeable,
                    TraversalMode::Swimmable,
                ] {
                    if ui.selectable_label(current == mode, mode.label()).clicked() {
                        set_world_traversal_for_scope(state, world, mode);
                    }
                }
            });
            ui.small(format!(
                "Effective: {} · artwork is unchanged",
                state
                    .world_doc
                    .effective_traversal(world, &state.asset_authority)
                    .label()
            ));
        });

    egui::CollapsingHeader::new("Collision shape")
        .id_salt("havenwild.world.properties.collision")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.selectable_label(state.collision_brush_blocked, "Block").clicked() {
                    state.collision_brush_blocked = true;
                }
                if ui.selectable_label(!state.collision_brush_blocked, "Clear").clicked() {
                    state.collision_brush_blocked = false;
                }
                if ui.small_button("-").on_hover_text("Smaller collision brush").clicked() {
                    state.collision_brush_radius = state.collision_brush_radius.saturating_sub(1).max(1);
                }
                ui.label(format!("{} px", state.collision_brush_radius));
                if ui.small_button("+").on_hover_text("Larger collision brush").clicked() {
                    state.collision_brush_radius = (state.collision_brush_radius + 1).min(8);
                }
                ui.checkbox(&mut state.collision_world_pixel_edit, "Paint on canvas")
                    .on_hover_text("Click-drag directly over visible pixels. One drag is one undo transaction.");
            });
            if state.collision_world_pixel_edit && !state.editor_layout.collision_overlay {
                state.editor_layout.collision_overlay = true;
                save_editor_layout(state);
            }

            let explicit = world_collision_scope_mask(state, world);
            let preview = explicit.clone().unwrap_or_else(|| world_collision_seed(state, world));
            ui.small(format!(
                "{} · {}/1024 blocked · overlay is metadata only",
                if explicit.is_some() { "Explicit" } else { "Inherited preview" },
                preview.blocked_count()
            ));
            ui.horizontal_wrapped(|ui| {
                if ui.button("Fill blocked").clicked() {
                    set_world_collision_for_scope(state, world, Some(CollisionMask32::full()));
                }
                if ui.button("Clear all").clicked() {
                    set_world_collision_for_scope(state, world, Some(CollisionMask32::empty()));
                }
                if ui.add_enabled(explicit.is_some(), egui::Button::new("Inherit")).clicked() {
                    set_world_collision_for_scope(state, world, None);
                }
            });

            if !state.editor_layout.structural_compact {
                let side = ui.available_width().min(300.0).clamp(192.0, 300.0);
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::click_and_drag());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(26, 31, 34));
                let unit = side / COLLISION_MASK_SIDE as f32;
                for y in 0..COLLISION_MASK_SIDE {
                    for x in 0..COLLISION_MASK_SIDE {
                        if preview.blocked(x, y) {
                            painter.rect_filled(
                                egui::Rect::from_min_size(
                                    rect.min + egui::vec2(x as f32 * unit, y as f32 * unit),
                                    egui::Vec2::splat(unit),
                                ),
                                0.0,
                                egui::Color32::from_rgba_unmultiplied(108, 18, 24, 118),
                            );
                        }
                    }
                }
                if response.dragged_by(egui::PointerButton::Primary)
                    || response.clicked_by(egui::PointerButton::Primary)
                {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let local = pointer - rect.min;
                        let x = (local.x / unit).floor().clamp(0.0, 31.0) as usize;
                        let y = (local.y / unit).floor().clamp(0.0, 31.0) as usize;
                        let mut mask = preview.clone();
                        let extent = state.collision_brush_radius.saturating_sub(1) as isize;
                        for dy in -extent..=extent {
                            for dx in -extent..=extent {
                                let px = x as isize + dx;
                                let py = y as isize + dy;
                                if (0..32).contains(&px) && (0..32).contains(&py) {
                                    mask.set_blocked(px as usize, py as usize, state.collision_brush_blocked);
                                }
                            }
                        }
                        set_world_collision_for_scope(state, world, Some(mask));
                    }
                }
            }
        });

    egui::CollapsingHeader::new("Elevation")
        .id_salt("havenwild.world.properties.elevation")
        .default_open(false)
        .show(ui, |ui| {
            let existing = state
                .world_doc
                .cell_override(world)
                .and_then(|item| item.elevation);
            let mut height = existing.unwrap_or(0);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::DragValue::new(&mut height).range(0..=30).prefix("H "))
                    .changed()
                {
                    checkpoint_world(state);
                    if state
                        .world_doc
                        .set_elevation_override(world, Some(height))
                        .is_ok()
                    {
                        state.world_dirty = true;
                    }
                }
                if ui
                    .add_enabled(existing.is_some(), egui::Button::new("Automatic"))
                    .clicked()
                {
                    checkpoint_world(state);
                    if state.world_doc.set_elevation_override(world, None).is_ok() {
                        state.world_dirty = true;
                    }
                }
            });
            ui.small(
                "Elevation is world geography. It is not promoted to every use of a source tile.",
            );
        });

    egui::CollapsingHeader::new("Advanced")
        .id_salt("havenwild.world.properties.advanced")
        .default_open(false)
        .show(ui, |ui| {
            if let Some(source) = selected_world_source(state, world) {
                ui.small(format!("Source: {} @ {:?}", source.source_asset, source.source_rect));
            }
            ui.small(format!(
                "Terrain topology profiles: {} · legacy source profiles retained/inactive for generated terrain: {} · local world overrides: {}",
                state.world_doc.terrain_behavior_profiles.len(),
                state.world_doc.source_behavior_profiles.len(),
                state.world_doc.cell_overrides.len()
            ));
            ui.small("Generated-terrain collision/traversal now uses semantic topology profiles or local overrides. Legacy source-rectangle profiles remain serialized for compatibility but are not applied to generated terrain. Original ElizaWy source bytes remain immutable.");
        });
}

// An authored structural cell is separate from the immutable original artwork.
// Edit commands use the same scene-v2 undo/redo/save path as visual changes.
/// The single Inspector World tab owns collision, traversal and elevation.
/// This is the original authoring body formerly rendered in a floating window.
/// All edits retain their existing undo/history authority.
fn draw_world_properties_contents(ui: &mut egui::Ui, state: &mut Studio) {
    let mut compact = state.editor_layout.structural_compact;
    if ui
        .selectable_label(compact, if compact { "Compact" } else { "Full" })
        .on_hover_text("Show quick collision/elevation edits or the full semantic metadata editor")
        .clicked()
    {
        compact = !compact;
        state.editor_layout.structural_compact = compact;
        save_editor_layout(state);
    }
    ui.small("Explicit scene metadata, never inferred from sprite alpha. Ctrl+Z / Ctrl+Y and Save Scene use the same transaction history.");
    if ui
            .checkbox(&mut state.editor_layout.collision_overlay, "Collision overlay · red = impassable")
            .on_hover_text("Show the effective collision layer over the world. Automatic/coarse blocked cells are translucent red; authored 32x32 masks show their exact blocked pixels.")
            .changed()
        {
            save_editor_layout(state);
        }
    if !state.editor_layout.structural_compact
            && ui
                .checkbox(&mut state.editor_layout.structural_overlay, "Structural metadata outline")
                .on_hover_text("Show elevation/traversal metadata outlines separately from the red collision overlay.")
                .changed()
        {
            save_editor_layout(state);
        }
    ui.separator();
    if state.world_mode {
        draw_world_structural_contents(ui, state);
        return;
    }
    if !state.selection_active {
        ui.label("Select a scene cell with Select (Q) first.");
        return;
    }
    let pos = state.selected_world;
    let Some(index) = state.scene.index(pos[0], pos[1]) else {
        return;
    };
    ui.strong(format!("Cell {},{}", pos[0], pos[1]));
    ui.separator();
    ui.strong("Collision layer");
    ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(state.collision_brush_blocked, "Block")
                .on_hover_text("Collision brush writes impassable red pixels.")
                .clicked()
            {
                state.collision_brush_blocked = true;
            }
            if ui
                .selectable_label(!state.collision_brush_blocked, "Clear")
                .on_hover_text("Collision brush clears exact pixels from the impassable mask.")
                .clicked()
            {
                state.collision_brush_blocked = false;
            }
            ui.checkbox(&mut state.collision_world_pixel_edit, "Edit on world canvas")
                .on_hover_text("When enabled, a primary click on the world edits the exact 1/32-cell collision pixel under the cursor instead of painting artwork. Zoom in for precise correction.");
        });
    if state.collision_world_pixel_edit && !state.editor_layout.collision_overlay {
        state.editor_layout.collision_overlay = true;
        save_editor_layout(state);
    }
    if state.editor_layout.structural_compact {
        let explicit = state.layered_scene.collision_mask(index).cloned();
        ui.small(format!(
            "{} · {} blocked pixels",
            if explicit.is_some() {
                "Authored 32×32 mask"
            } else {
                "Automatic/coarse collision"
            },
            explicit
                .as_ref()
                .map(|mask| mask.blocked_count())
                .unwrap_or(if automatic_cell_blocked(state, index) {
                    1024
                } else {
                    0
                })
        ));
        ui.horizontal_wrapped(|ui| {
            if ui.button("Full blocked").clicked() {
                set_collision_mask(state, index, Some(CollisionMask32::full()));
            }
            if ui.button("Full clear").clicked() {
                set_collision_mask(state, index, Some(CollisionMask32::empty()));
            }
            if ui
                .add_enabled(explicit.is_some(), egui::Button::new("Automatic"))
                .clicked()
            {
                set_collision_mask(state, index, None);
            }
        });
    } else {
        draw_collision_mask_editor(ui, state, index);
    }
    ui.separator();
    ui.strong("Cell / elevation metadata");
    let previous = state
        .layered_scene
        .structural_cells
        .iter()
        .find(|v| v.index == index)
        .cloned();
    let mut cell = previous.clone().unwrap_or(StructuralCell {
        index,
        elevation: None,
        terrain_kind: None,
        water_flow: None,
        blocks_traversal: None,
        connector: None,
    });
    let mut changed = false;
    ui.label("Traversal override:");
    ui.horizontal_wrapped(|ui| {
        for (label, value) in [
            ("Unknown", None),
            ("Walkable", Some(false)),
            ("Blocked", Some(true)),
        ] {
            if ui
                .selectable_label(cell.blocks_traversal == value, label)
                .clicked()
            {
                cell.blocks_traversal = value;
                changed = true;
            }
        }
    });
    ui.label("Elevation: sea = 0, land = 1..30");
    if let Some(mut height) = cell.elevation {
        ui.horizontal(|ui| {
            if ui
                .add(egui::DragValue::new(&mut height).range(0..=30).prefix("H "))
                .changed()
            {
                cell.elevation = Some(height);
                changed = true;
            }
            if ui.small_button("Clear").clicked() {
                cell.elevation = None;
                changed = true;
            }
        });
    } else if ui.button("Set explicit H0").clicked() {
        cell.elevation = Some(0);
        changed = true;
    }

    if !state.editor_layout.structural_compact {
        ui.separator();
        ui.label("Semantic terrain hint:");
        let mut terrain = cell.terrain_kind.clone().unwrap_or_default();
        egui::ComboBox::from_id_salt("havenwild.structural.terrain.kind")
            .selected_text(if terrain.is_empty() {
                "Unspecified".to_owned()
            } else {
                terrain.clone()
            })
            .show_ui(ui, |ui| {
                for label in [
                    "", "grass", "dirt", "sand", "water", "cliff", "road", "rock",
                ] {
                    ui.selectable_value(
                        &mut terrain,
                        label.to_string(),
                        if label.is_empty() {
                            "Unspecified"
                        } else {
                            label
                        },
                    );
                }
            });
        if cell.terrain_kind.as_deref().unwrap_or("") != terrain {
            cell.terrain_kind = if terrain.is_empty() {
                None
            } else {
                Some(terrain)
            };
            changed = true;
        }
        ui.label("Authored connector (preview metadata):");
        let mut connector = cell.connector.clone().unwrap_or_default();
        egui::ComboBox::from_id_salt("havenwild.structural.connector")
            .selected_text(if connector.is_empty() {
                "None".to_owned()
            } else {
                connector.clone()
            })
            .show_ui(ui, |ui| {
                for label in ["", "ramp", "stairs", "ladder", "cliff_vines"] {
                    ui.selectable_value(
                        &mut connector,
                        label.to_string(),
                        if label.is_empty() { "None" } else { label },
                    );
                }
            });
        if cell.connector.as_deref().unwrap_or("") != connector {
            cell.connector = if connector.is_empty() {
                None
            } else {
                Some(connector)
            };
            changed = true;
        }
        ui.separator();
        ui.small("PIE honors explicitly authored traversal overrides over provisional River/Grass hints. Cliff connector travel, footprints and hydrology remain separate certification work.");
    }

    if ui
        .button("Clear this cell's structural overrides")
        .clicked()
    {
        cell.elevation = None;
        cell.terrain_kind = None;
        cell.water_flow = None;
        cell.blocks_traversal = None;
        cell.connector = None;
        changed = true;
    }
    if changed {
        let empty = cell.elevation.is_none()
            && cell.terrain_kind.is_none()
            && cell.water_flow.is_none()
            && cell.blocks_traversal.is_none()
            && cell.connector.is_none();
        let update = SceneCommand::SetStructuralCell {
            index,
            after: if empty { None } else { Some(cell) },
        };
        match state
            .layered_history
            .execute(&mut state.layered_scene, update)
        {
            Ok(()) => {
                state.message = format!(
                    "Updated explicit collision/elevation for cell {},{}; save scene to persist.",
                    pos[0], pos[1]
                )
            }
            Err(error) => state.message = format!("Structural edit not applied: {error}"),
        }
    }
}

fn draw_structural_overlay(
    painter: &egui::Painter,
    state: &Studio,
    scene: egui::Rect,
    px: f32,
    canvas: egui::Rect,
) {
    if !state.editor_layout.structural_overlay || px < 7.0 {
        return;
    }
    let p = painter.with_clip_rect(canvas.intersect(scene));
    for cell in &state.layered_scene.structural_cells {
        let x = cell.index % state.scene.size[0];
        let y = cell.index / state.scene.size[0];
        let rect = egui::Rect::from_min_size(
            scene.min + egui::vec2(x as f32 * px, y as f32 * px),
            egui::Vec2::splat(px),
        );
        if !rect.intersects(canvas) {
            continue;
        }
        let ink = match cell.blocks_traversal {
            Some(true) => egui::Color32::from_rgb(246, 112, 110),
            Some(false) => egui::Color32::from_rgb(98, 217, 172),
            None => egui::Color32::from_rgb(230, 192, 101),
        };
        p.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.5, ink),
            egui::StrokeKind::Inside,
        );
        if px >= 22.0 {
            if let Some(height) = cell.elevation {
                p.text(
                    rect.left_top() + egui::vec2(3.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    format!("H{height}"),
                    egui::FontId::monospace(10.0),
                    ink,
                );
            }
        }
    }
}

fn open_mapper_tab(state: &mut Studio, tab: &str) {
    state.editor_layout.terrain_mapper_tab = tab.into();
    state.editor_layout.terrain_mapper.visible = true;
    save_editor_layout(state);
}

fn active_tool_label(state: &Studio) -> &'static str {
    if state.world_bridge_tool {
        "Bridge Span"
    } else if state.tool == CanvasTool::Paint {
        state.brush_mode.label()
    } else {
        state.tool.label()
    }
}

fn choose_brush(state: &mut Studio, mode: BrushMode) {
    state.tool = CanvasTool::Paint;
    state.brush_mode = mode;
    state.editor_layout.brush_mode = match mode {
        BrushMode::Pixel => "pixel",
        BrushMode::Tile => "tile",
        BrushMode::Terrain => "terrain",
    }
    .into();
    state.world_bridge_tool = false;
    state.world_bridge_drag = None;
    state.editor_layout.tool_context_open = true;
    if mode == BrushMode::Terrain && state.world_mode {
        state.active_layer = world_terrain_role_layer(state.world_terrain_brush.role());
    }
    save_editor_layout(state);
    state.message = match mode {
        BrushMode::Terrain => "Terrain Brush: semantic material stroke with Wang/dual-grid resolution.".into(),
        BrushMode::Tile => "Tile Brush: exact 32×32 source-cell placement; source image never modified.".into(),
        BrushMode::Pixel => "Pixel Brush: project-owned derived image target required; no world pixels will be changed.".into(),
    };
}

/// One set of authoring tools inside the single 15% left dock. No extra
/// fixed-width rail competes with the three contextual tool sections.
fn draw_tool_rail(ui: &mut egui::Ui, state: &mut Studio) {
    ui.horizontal_wrapped(|ui| {
        ui.add_space(7.0);
        ui.small("EDIT");
        if rail_icon_button(
            ui,
            RailIcon::Select,
            state.tool == CanvasTool::Select && !state.world_bridge_tool,
            &command_tooltip(StudioCommand::Select),
            [43.0, 33.0],
        )
        .clicked()
        {
            state.tool = CanvasTool::Select;
            state.world_bridge_tool = false;
            state.world_bridge_drag = None;
            // Keep the unified authoring workspace available; Select has
            // its own context controls and Library, not a blank slab.
        }
        for (mode, icon) in [
            (BrushMode::Pixel, RailIcon::Pixel),
            (BrushMode::Tile, RailIcon::Tile),
            (BrushMode::Terrain, RailIcon::Terrain),
        ] {
            let hover = format!(
                "{} · {}",
                mode.label(),
                match mode {
                    BrushMode::Pixel => "Derived artwork (editing backend pending)",
                    BrushMode::Tile => "Place an exact source tile",
                    BrushMode::Terrain => "Paint semantic terrain",
                }
            );
            if rail_icon_button(
                ui,
                icon,
                state.tool == CanvasTool::Paint && state.brush_mode == mode,
                &hover,
                [43.0, 33.0],
            )
            .clicked()
            {
                choose_brush(state, mode);
            }
        }
        ui.add_space(4.0);
        ui.separator();
        for (tool, icon) in [
            (CanvasTool::Erase, RailIcon::Erase),
            (CanvasTool::Sample, RailIcon::Sample),
        ] {
            if rail_icon_button(
                ui,
                icon,
                state.tool == tool,
                &command_tooltip(tool_command(tool)),
                [43.0, 33.0],
            )
            .clicked()
            {
                state.tool = tool;
                state.world_bridge_tool = false;
                state.editor_layout.tool_context_open = true;
                save_editor_layout(state);
            }
        }
        if rail_icon_button(
            ui,
            RailIcon::Bridge,
            state.world_bridge_tool,
            "Bridge Span: drag between two land banks; source-verified horizontal orientation only",
            [43.0, 33.0],
        )
        .clicked()
            && state.world_mode
        {
            state.world_bridge_tool = !state.world_bridge_tool;
            state.tool = CanvasTool::Select;
            state.active_layer = LayerId::Structures;
            state.editor_layout.tool_context_open = state.world_bridge_tool;
            save_editor_layout(state);
        }
        ui.add_space(4.0);
        ui.separator();
        if rail_icon_button(
            ui,
            RailIcon::Panel,
            state.editor_layout.tool_context_open,
            "Collapse/expand integrated Tooling and Tool Context",
            [43.0, 33.0],
        )
        .clicked()
        {
            state.editor_layout.tool_context_open = !state.editor_layout.tool_context_open;
            save_editor_layout(state);
        }
        if rail_icon_button(
            ui,
            RailIcon::Library,
            state.editor_layout.library_open,
            "Show/hide the shared Asset Browser",
            [43.0, 33.0],
        )
        .clicked()
        {
            state.editor_layout.library_open = !state.editor_layout.library_open;
            if state.editor_layout.library_open && state.editor_layout.library_in_left_workspace {
                state.editor_layout.tool_context_open = true;
            }
            save_editor_layout(state);
        }
        // M2D131: Apps is only in Tools / Ctrl+Space; never duplicate its launcher on the rail.
    });
}

/// Persistent, non-destructive atlas selection. Every complete 32x32 source cell
/// can be selected here and placed by the existing exact-source Tile Brush.
/// This is NOT an automatic Wang classification or a new renderer.
fn request_occupied_source_cells(state: &mut Studio, relative: &str) {
    if !state.occupancy_requests.insert(relative.to_owned()) {
        return;
    }
    let root = project_root::resolve().join("assets/elizawy");
    let sheet = relative.to_owned();
    let destination = Arc::clone(&state.occupied_source_cells);
    std::thread::spawn(move || {
        let result = (|| -> Result<Vec<[u32; 2]>, String> {
            let image = image::open(root.join(&sheet))
                .map_err(|e| e.to_string())?
                .to_rgba8();
            let (width, height) = image.dimensions();
            let mut occupied = Vec::new();
            for y in 0..height / 32 {
                for x in 0..width / 32 {
                    // A cell belongs to the compact library iff original alpha contains artwork.
                    let nonempty = (0..32).any(|dy| {
                        (0..32).any(|dx| image.get_pixel(x * 32 + dx, y * 32 + dy)[3] != 0)
                    });
                    if nonempty {
                        occupied.push([x, y]);
                    }
                }
            }
            Ok(occupied)
        })();
        if let Ok(mut cache) = destination.lock() {
            cache.insert(sheet, result);
        }
    });
}

/// Three views over the same immutable original PNG and identical source addresses.
/// Compact Grid/List filter fully transparent 32px cells; Atlas preserves all cells.
fn draw_authoring_atlas(ui: &mut egui::Ui, state: &mut Studio) {
    ui.horizontal(|ui| {
        ui.label("Search");
        ui.add(egui::TextEdit::singleline(&mut state.source_filter).desired_width(200.0));
        egui::ComboBox::from_id_salt("havenwild.library.category")
            .selected_text(&state.editor_layout.library_category)
            .show_ui(ui, |ui| {
                for category in ["All", "Terrain", "Vegetation", "Objects", "Structure", "FX"] {
                    if ui
                        .selectable_value(
                            &mut state.editor_layout.library_category,
                            category.into(),
                            category,
                        )
                        .changed()
                    {
                        save_editor_layout(state);
                    }
                }
            });
    });
    let query = state.source_filter.trim().to_ascii_lowercase();
    let category = state.editor_layout.library_category.as_str();
    let matches: Vec<String> = state
        .asset_authority
        .runtime_source_images
        .iter()
        .filter(|entry| {
            (category == "All" || entry.semantic_category == category)
                && (query.is_empty() || entry.source_path.to_ascii_lowercase().contains(&query))
        })
        .map(|entry| entry.source_path.clone())
        .collect();
    let current = state
        .sheets
        .get(state.active_sheet)
        .map(|x| x.path.as_str())
        .unwrap_or("No sheet");
    let mut selected_path = None;
    egui::ComboBox::from_id_salt("havenwild.unified.library.source")
        .selected_text(current)
        .width(ui.available_width().max(110.0))
        .show_ui(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    for path in &matches {
                        if ui
                            .selectable_label(current == path.as_str(), path)
                            .clicked()
                        {
                            selected_path = Some(path.clone());
                        }
                    }
                });
        });
    if let Some(path) = selected_path {
        if let Some(index) = state.sheets.iter().position(|x| x.path == path) {
            state.active_sheet = index;
            state.selected_cell = [0, 0];
            state.selected_extent = [1, 1];
        } else {
            state.pending_source_sheet = Some(path);
            state.message = "Loading original source sheet into the shared Library".into();
        }
    }
    let Some(sheet) = state.sheets.get(state.active_sheet) else {
        return;
    };
    let width = sheet.size[0] / 32;
    let height = sheet.size[1] / 32;
    let texture = sheet.texture;
    let dim = sheet.size;
    let path = sheet.path.clone();
    if width == 0 || height == 0 {
        return;
    }
    let registered_regions = state
        .asset_authority
        .canonical_runtime_regions
        .iter()
        .filter(|region| region.source_path == path)
        .count();
    let reviewed_sprites: Vec<(String, [u32; 4])> = state
        .asset_authority
        .object_templates
        .iter()
        .filter(|template| template.parts.len() == 1 && template.parts[0].source_path == path)
        .map(|template| (template.label.clone(), template.parts[0].source_rect_px))
        .collect();
    ui.small(format!(
        "Source authority: {} exact registered region(s) · {} reviewed one-part sprite(s)",
        registered_regions,
        reviewed_sprites.len()
    ));
    if registered_regions == 0 {
        ui.colored_label(egui::Color32::YELLOW,
            "Catalog-only sheet: original art can be inspected, but Generated World placement needs a reviewed source region. No classification is inferred.");
    }
    // Whole-template placement is the only non-terrain asset promotion in this pass.
    // The candidates below remain inspection-only and never create world instances.
    if state.world_mode {
        let reviewed_templates = state
            .asset_authority
            .object_templates
            .iter()
            .filter(|t| t.parts.iter().any(|p| p.source_path == path))
            .filter(|t| t.layer == "objects" || t.layer == "structures")
            .filter(|t| !t.worldgen_tags.iter().any(|tag| tag == "bridge"))
            .map(|t| (t.template_id.clone(), t.label.clone(), t.footprint_cells))
            .collect::<Vec<_>>();
        if !reviewed_templates.is_empty() {
            let mut place_choice = None;
            egui::CollapsingHeader::new("Place reviewed complete objects")
                .id_salt("havenwild.library.approved.template.place")
                .default_open(true)
                .show(ui, |ui| {
                    for (id, label, footprint) in &reviewed_templates {
                        if ui.button(format!("Place {label} · {}×{} tiles", footprint[0], footprint[1]))
                            .on_hover_text("Use exactly the reviewed full sprite/template. One click in World places it as a visual object; no guessed collision or semantic role.")
                            .clicked() { place_choice = Some(id.clone()); }
                    }
                });
            if let Some(id) = place_choice {
                state.world_template_placement = Some(id);
                state.tool = CanvasTool::Select;
                state.message = "Reviewed object selected. Click a materialized world location to place it, or Esc to cancel; nothing is placed until you click.".into();
            }
        }
        if state.world_template_placement.is_some()
            && ui.small_button("Cancel object placement").clicked()
        {
            state.world_template_placement = None;
        }
    }
    if let Some(alpha) = state.alpha_candidates.sheet(&path) {
        ui.small(format!("Original-alpha review: {} isolated region candidate(s) · {} crowded group(s) needing manual split · {} occupied 32px cells",
            alpha.isolated_groups.len(), alpha.crowded_groups, alpha.occupied_cells));
        if !alpha.isolated_groups.is_empty() {
            let mut pick = None;
            egui::CollapsingHeader::new("Unreviewed alpha regions (inspect only)")
                .id_salt("havenwild.library.alpha.evidence.regions")
                .show(ui, |ui| {
                    for (i, group) in alpha.isolated_groups.iter().enumerate() {
                        let [x,y,w,h] = group.rect_px;
                        if ui.small_button(format!("Candidate {} · {},{} · {}×{}px", i+1,x,y,w,h))
                            .on_hover_text("Alpha-separated grid evidence only; not a complete sprite approval, terrain classification or placeable object.")
                            .clicked() { pick = Some(group.rect_px); }
                    }
                });
            if let Some([x, y, w, h]) = pick {
                state.selected_cell = [x / 32, y / 32];
                state.selected_extent = [w / 32, h / 32];
                state.message = "Unreviewed original-alpha bounds selected for inspection. No object or source-region authority was created.".into();
            }
        }
    }
    if !reviewed_sprites.is_empty() {
        let mut reviewed_choice = None;
        egui::CollapsingHeader::new("Reviewed complete source sprites")
            .id_salt("havenwild.asset.source.reviewed.templates")
            .show(ui, |ui| {
                for (label, rect) in &reviewed_sprites {
                    let aligned = rect.iter().all(|value| *value % 32 == 0);
                    if ui.add_enabled(aligned, egui::Button::new(
                        format!("{} · {},{} · {}×{}px", label, rect[0], rect[1], rect[2], rect[3])))
                        .on_hover_text("Select the entire source sprite, never just its top 32px cell; no collision or gameplay role is inferred.")
                        .clicked()
                    {
                        reviewed_choice = Some(*rect);
                    }
                }
            });
        if let Some(rect) = reviewed_choice {
            state.selected_cell = [rect[0] / 32, rect[1] / 32];
            state.selected_extent = [rect[2] / 32, rect[3] / 32];
            choose_brush(state, BrushMode::Tile);
            state.message = if state.world_mode && !path.starts_with("Terrain/") {
                format!("Reviewed full sprite located: {} @ {:?}. World complete-object placement is not yet enabled for this non-Terrain source; no world edit was made.", path, rect)
            } else {
                format!("Reviewed full sprite selected: {} @ {:?}. Source art unchanged; select a valid destination layer.", path, rect)
            };
        }
    }
    let raw_atlas = state.editor_layout.library_view == "Atlas"
        || state.editor_layout.authoring_library_tab == "Source";
    let cells = if raw_atlas {
        (0..height)
            .flat_map(|y| (0..width).map(move |x| [x, y]))
            .collect::<Vec<_>>()
    } else {
        request_occupied_source_cells(state, &path);
        let cached = state
            .occupied_source_cells
            .lock()
            .ok()
            .and_then(|lock| lock.get(&path).cloned());
        match cached {
            Some(Ok(occupied)) => occupied,
            Some(Err(message)) => {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("Source alpha unavailable: {message}"),
                );
                return;
            }
            None => {
                ui.small("Indexing non-transparent source cells in background...");
                return;
            }
        }
    };
    ui.small(format!(
        "{} visible / {} source positions · 32×32 · original art read-only",
        cells.len(),
        width as usize * height as usize
    ));
    let list_mode = state.editor_layout.library_view == "List" && !raw_atlas;
    let cell_size = if list_mode { 42.0 } else { 45.0 };
    // Atlas keeps the *physical* number of source columns. Compact Grid may reflow.
    let columns = if raw_atlas {
        width as usize
    } else if list_mode {
        1
    } else {
        ((ui.available_width().max(cell_size) / cell_size).floor() as usize).max(1)
    };
    let rows = cells.len().div_ceil(columns);
    let mut chosen = None;
    let available_height = ui.available_height().max(72.0);
    egui::ScrollArea::both()
        .id_salt("havenwild.unified.library.atlas")
        .max_height(available_height)
        .show_rows(ui, cell_size, rows, |ui, range| {
            ui.spacing_mut().item_spacing.y = 1.0;
            if raw_atlas {
                ui.set_min_width(width as f32 * cell_size);
            }
            for row in range {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    for col in 0..columns {
                        let index = row * columns + col;
                        let Some(&[cx, cy]) = cells.get(index) else {
                            break;
                        };
                        let tile_width = if list_mode {
                            ui.available_width().max(100.0)
                        } else {
                            44.0
                        };
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(tile_width, 41.0),
                            egui::Sense::click(),
                        );
                        if let Some(texture) = texture {
                            let uv = egui::Rect::from_min_max(
                                egui::pos2(
                                    cx as f32 * 32.0 / dim[0] as f32,
                                    cy as f32 * 32.0 / dim[1] as f32,
                                ),
                                egui::pos2(
                                    (cx + 1) as f32 * 32.0 / dim[0] as f32,
                                    (cy + 1) as f32 * 32.0 / dim[1] as f32,
                                ),
                            );
                            let thumb = egui::Rect::from_min_size(
                                rect.min + egui::vec2(4.0, 3.0),
                                egui::vec2(35.0, 35.0),
                            );
                            ui.painter().image(texture, thumb, uv, egui::Color32::WHITE);
                        }
                        if list_mode {
                            ui.painter().text(
                                rect.min + egui::vec2(50.0, 13.0),
                                egui::Align2::LEFT_TOP,
                                format!("Tile {cx},{cy}  ·  source {},{}", cx * 32, cy * 32),
                                egui::FontId::proportional(12.0),
                                egui::Color32::LIGHT_GRAY,
                            );
                        }
                        if state.selected_cell == [cx, cy] {
                            ui.painter().rect_stroke(
                                if list_mode {
                                    egui::Rect::from_min_size(rect.min, egui::vec2(42.0, 41.0))
                                } else {
                                    rect
                                },
                                1.0,
                                egui::Stroke::new(2.0, egui::Color32::YELLOW),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if response.clicked() {
                            chosen = Some([cx, cy]);
                        }
                        response.on_hover_text(format!(
                            "Cell {cx},{cy} · source {},{} · 32×32",
                            cx * 32,
                            cy * 32
                        ));
                    }
                });
            }
        });
    if let Some(cell) = chosen {
        state.selected_cell = cell;
        state.selected_extent = [1, 1];
        choose_brush(state, BrushMode::Tile);
        state.message = format!("Exact tile {},{} selected. Click World canvas to stamp without changing semantic terrain.", cell[0], cell[1]);
    }
}

/// M2D126: Recovered M2D112 authoring workspace. The active tool, family, and
/// brush properties share one column next to the vector icon rail. Do not
/// substitute a text-only help panel for this real workflow.
/// Context details only; tool selection is authoritative in the Mode & Tools
/// icon strip directly above. No second Select/Paint/Erase/Bridge command set.
fn draw_authoring_tooling(ui: &mut egui::Ui, state: &mut Studio) {
    ui.horizontal(|ui| {
        ui.strong("ACTIVE TOOL");
        if ui
            .button("Browse Objects")
            .on_hover_text("Find reviewed objects in the shared right Asset Browser")
            .clicked()
        {
            state.world_bridge_tool = false;
            state.tool = CanvasTool::Select;
            state.editor_layout.library_open = true;
            state.editor_layout.library_floating = false;
            state.editor_layout.library_in_inspector = true;
            state.editor_layout.library_in_left_workspace = false;
            state.editor_layout.authoring_library_tab = "Assets".into();
            save_editor_layout(state);
        }
    });
    ui.separator();
    ui.label(format!("Active: {}", active_tool_label(state)));
    if state.tool == CanvasTool::Paint && state.brush_mode == BrushMode::Pixel {
        ui.small("Color wheel and palette are in Pixel Studio below. Derived-image painting remains disabled until a project-owned pixel layer exists.");
    } else if state.tool == CanvasTool::Paint && state.brush_mode == BrushMode::Terrain {
        ui.small(format!(
            "Material: {} · choose in Asset Browser",
            state.world_terrain_brush.label()
        ));
        if ui.small_button("Open terrain palette").clicked() {
            state.editor_layout.authoring_library_tab = "Terrain".into();
            state.editor_layout.library_floating = false;
            state.editor_layout.library_open = true;
            save_editor_layout(state);
        }
    } else if state.tool == CanvasTool::Paint && state.brush_mode == BrushMode::Tile {
        let rect = selected_source_rect(state);
        ui.small(format!("Source crop: {} × {}px", rect[2], rect[3]));
        if ui.small_button("Open source Library").clicked() {
            state.editor_layout.authoring_library_tab = "Tiles".into();
            state.editor_layout.library_open = true;
            state.editor_layout.library_floating = false;
            save_editor_layout(state);
        }
    } else if state.world_template_placement.is_some() {
        ui.small("Click the world to place the reviewed complete object. Esc cancels.");
    }
}

fn draw_authoring_brush_settings(ui: &mut egui::Ui, state: &mut Studio) {
    // Only controls backed by the active tool are displayed here.
    if state.world_bridge_tool {
        ui.small("Drag across a valid horizontal water span. One undoable bridge.");
    } else if state.tool == CanvasTool::Select {
        ui.small("Click to select; drag objects to move. Q selects. Ctrl+Z undoes.");
    } else if state.tool == CanvasTool::Paint {
        match state.brush_mode {
            BrushMode::Pixel => {
                ui.small("Pixel brush size and source color are controlled by Pixel Studio below.");
                ui.small("Image-pixel strokes require a derived-image document before enabling world painting.");
            }
            BrushMode::Tile => {
                ui.label("Exact source stamp");
                if state.world_mode
                    && ui
                        .add(
                            egui::Slider::new(
                                &mut state.editor_layout.tile_brush_radius_tiles,
                                1..=4,
                            )
                            .text("Radius (tiles)"),
                        )
                        .changed()
                {
                    save_editor_layout(state);
                }
                let rect = selected_source_rect(state);
                ui.small(format!(
                    "Crop: {},{} · {}×{} px",
                    rect[0], rect[1], rect[2], rect[3]
                ));
                ui.small("Local visual override only; semantics and original PNG unchanged.");
            }
            BrushMode::Terrain => {
                if ui
                    .add(
                        egui::Slider::new(
                            &mut state.editor_layout.terrain_brush_radius_tiles,
                            1..=4,
                        )
                        .text("Radius (tiles)"),
                    )
                    .changed()
                {
                    save_editor_layout(state);
                }
                ui.small("Paint material; shared Wang resolver updates corners and neighbors.");
                egui::CollapsingHeader::new("Evidence-gated manual materials").show(ui, |ui| {
                    for brush in [WorldTerrainBrush::WetSand, WorldTerrainBrush::PebblePath] {
                        if ui
                            .selectable_label(state.world_terrain_brush == brush, brush.label())
                            .clicked()
                        {
                            state.world_terrain_brush = brush;
                            state.editor_layout.terrain_brush_role = brush.role().into();
                            if state.world_mode {
                                state.active_layer = world_terrain_role_layer(brush.role());
                            }
                            save_editor_layout(state);
                        }
                    }
                });
            }
        }
    } else if state.tool == CanvasTool::Erase {
        ui.small("Erase authored overrides only, never original source pixels.");
    } else {
        ui.small("Sample existing source binding or semantic material.");
    }
    ui.separator();
    ui.small("Q Select · B Brush · E Erase · P Sample · Ctrl+1/2/3 brush families");
}

fn draw_authoring_library(ui: &mut egui::Ui, state: &mut Studio) {
    ui.horizontal_wrapped(|ui| {
        for tab in ["Terrain", "Tiles", "Assets", "Source", "Rooms"] {
            if ui
                .selectable_label(state.editor_layout.authoring_library_tab == tab, tab)
                .clicked()
            {
                state.editor_layout.authoring_library_tab = tab.into();
                if tab == "Source" {
                    state.editor_layout.library_view = "Atlas".into();
                }
                save_editor_layout(state);
            }
        }
        ui.separator();
        for view in ["Grid", "List", "Atlas"] {
            if ui
                .selectable_label(state.editor_layout.library_view == view, view)
                .clicked()
            {
                state.editor_layout.library_view = view.into();
                save_editor_layout(state);
            }
        }
        ui.separator();
        let is_left = state.editor_layout.library_in_left_workspace
            && !state.editor_layout.library_floating
            && !state.editor_layout.library_in_inspector;
        let is_bottom = !state.editor_layout.library_in_left_workspace
            && !state.editor_layout.library_floating
            && !state.editor_layout.library_in_inspector;
        if !is_left && ui.small_button("Dock left").clicked() {
            state.editor_layout.library_in_left_workspace = true;
            state.editor_layout.library_in_inspector = false;
            state.editor_layout.library_floating = false;
            state.editor_layout.tool_context_open = true;
            save_editor_layout(state);
        }
        if !is_bottom && ui.small_button("Dock bottom").clicked() {
            state.editor_layout.library_in_left_workspace = false;
            state.editor_layout.library_in_inspector = false;
            state.editor_layout.library_floating = false;
            save_editor_layout(state);
        }
        if !state.editor_layout.library_in_inspector && ui.small_button("Dock Inspector").clicked()
        {
            state.editor_layout.library_in_inspector = true;
            state.editor_layout.library_in_left_workspace = false;
            state.editor_layout.library_floating = false;
            state.editor_layout.library_open = true;
            save_editor_layout(state);
        }
        if !state.editor_layout.library_floating && ui.small_button("Undock").clicked() {
            state.editor_layout.library_floating = true;
            save_editor_layout(state);
        }
        if ui.small_button("Collapse").clicked() {
            state.editor_layout.library_open = false;
            save_editor_layout(state);
        }
    });
    ui.separator();
    if state.editor_layout.authoring_library_tab == "Rooms" {
        draw_room_editor_controls(ui, state);
    } else if state.editor_layout.authoring_library_tab == "Terrain" {
        ui.small("Autotiling terrain · select a material, then paint on the world. Edges follow available source-backed rules.");
        ui.horizontal_wrapped(|ui| {
            for brush in [
                WorldTerrainBrush::Grass,
                WorldTerrainBrush::DirtBank,
                WorldTerrainBrush::Sand,
                WorldTerrainBrush::Water,
                WorldTerrainBrush::DeepWater,
            ] {
                if ui
                    .selectable_label(
                        state.world_terrain_brush == brush
                            && state.brush_mode == BrushMode::Terrain,
                        brush.label(),
                    )
                    .clicked()
                {
                    choose_brush(state, BrushMode::Terrain);
                    state.world_terrain_brush = brush;
                    state.editor_layout.terrain_brush_role = brush.role().into();
                    if state.world_mode {
                        state.active_layer = world_terrain_role_layer(brush.role());
                    }
                    save_editor_layout(state);
                }
            }
        });
        if ui.small_button("Inspect terrain rules").clicked() {
            open_mapper_tab(state, "patterns");
        }
        ui.small("Cliff and waterfall authoring requires certified source recipes; terrain paint does not fake elevation.");
    } else {
        draw_authoring_atlas(ui, state);
    }
}

/// A single project Library tray. Its top grip is always available when
/// docked, even if browsing is collapsed. Dragging UP increases the height.
/// Panel::bottom's implicit resize affordance was effectively invisible, so
/// resize is explicit and persisted when released rather than each frame.
fn draw_library_dock(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if state.editor_layout.library_floating
        || state.editor_layout.library_in_inspector
        || (state.editor_layout.library_in_left_workspace && state.editor_layout.tool_context_open)
    {
        return;
    }
    let available = viewport_ui.available_height();
    let max_height = (available * 0.77).clamp(160.0, 700.0);
    let preferred = if state.editor_layout.library_open {
        state.editor_layout.library_height.clamp(140.0, max_height)
    } else {
        34.0
    };
    egui::Panel::bottom("havenwild.library.bottom")
        .resizable(false)
        .exact_size(preferred)
        .frame(egui::Frame::NONE.fill(canvas_chrome::panel_fill(&state.editor_layout.editor_theme)))
        .show(viewport_ui, |ui| {
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 200.0).max(65.0);
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(width, 13.0), egui::Sense::click_and_drag());
                let grip_color = if response.hovered() || response.dragged() {
                    egui::Color32::from_rgb(110, 191, 190)
                } else {
                    egui::Color32::from_gray(104)
                };
                for dy in [-3.0_f32, 0.0, 3.0] {
                    ui.painter().hline(
                        rect.center().x - 30.0..=rect.center().x + 30.0,
                        rect.center().y + dy,
                        egui::Stroke::new(1.1, grip_color),
                    );
                }
                let response = response
                    .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                    .on_hover_text("Drag UP to expand the Library; drag DOWN to shrink");
                if response.dragged() {
                    let next = (state.editor_layout.library_height - response.drag_delta().y)
                        .clamp(140.0, max_height);
                    state.editor_layout.library_height = next;
                    state.editor_layout.library_open = true;
                    ui.ctx().request_repaint();
                }
                if response.drag_stopped() {
                    save_editor_layout(state);
                }
                if ui
                    .small_button(if state.editor_layout.library_open {
                        "▾ Library"
                    } else {
                        "▴ Library"
                    })
                    .on_hover_text("Expand or collapse the Asset Library")
                    .clicked()
                {
                    state.editor_layout.library_open = !state.editor_layout.library_open;
                    save_editor_layout(state);
                }
                if ui.small_button("Dock left").clicked() {
                    state.editor_layout.library_in_left_workspace = true;
                    state.editor_layout.library_in_inspector = false;
                    state.editor_layout.tool_context_open = true;
                    state.editor_layout.library_open = true;
                    save_editor_layout(state);
                }
                if ui.small_button("Dock Inspector").clicked() {
                    state.editor_layout.library_in_inspector = true;
                    state.editor_layout.library_in_left_workspace = false;
                    state.editor_layout.library_floating = false;
                    state.editor_layout.library_open = true;
                    save_editor_layout(state);
                }
                if ui.small_button("Undock").clicked() {
                    state.editor_layout.library_floating = true;
                    state.editor_layout.library_open = true;
                    save_editor_layout(state);
                }
            });
            if state.editor_layout.library_open {
                draw_authoring_library(ui, state);
            }
        });
}

fn draw_floating_library(ctx: &egui::Context, state: &mut Studio) {
    if !state.editor_layout.library_open || !state.editor_layout.library_floating {
        return;
    }
    let mut open = true;
    // egui's native window edge hit zone was unreliable with the scroll-heavy
    // atlas. Use one visible, input-owned lower-right drag handle instead.
    // The size persists across restarts and cannot be lost behind the canvas.
    let size = state.editor_layout.library_floating_size;
    egui::Window::new("Havenwild Asset Library")
        .id(egui::Id::new("havenwild.library.floating"))
        .open(&mut open)
        .resizable(false)
        .fixed_size(egui::vec2(size[0], size[1]))
        .show(ctx, |ui| {
            let inner_height = (ui.available_height() - 26.0).max(120.0);
            ui.allocate_ui(egui::vec2(ui.available_width(), inner_height), |ui| {
                draw_authoring_library(ui, state);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(30.0, 20.0), egui::Sense::drag());
                let color = if response.hovered() || response.dragged() {
                    egui::Color32::from_rgb(110, 191, 190)
                } else {
                    egui::Color32::from_gray(146)
                };
                for step in 0..3 {
                    let d = step as f32 * 5.0;
                    ui.painter().line_segment(
                        [
                            egui::pos2(rect.right() - 4.0 - d, rect.bottom() - 3.0),
                            egui::pos2(rect.right() - 4.0, rect.bottom() - 3.0 - d),
                        ],
                        egui::Stroke::new(1.5, color),
                    );
                }
                let response = response
                    .on_hover_cursor(egui::CursorIcon::ResizeNwSe)
                    .on_hover_text(
                        "Drag the corner to resize the floating Library (width and height)",
                    );
                if response.dragged() {
                    let delta = response.drag_delta();
                    state.editor_layout.library_floating_size[0] =
                        (state.editor_layout.library_floating_size[0] + delta.x)
                            .clamp(420.0, 1600.0);
                    state.editor_layout.library_floating_size[1] =
                        (state.editor_layout.library_floating_size[1] + delta.y)
                            .clamp(260.0, 1080.0);
                    ui.ctx().request_repaint();
                }
                if response.drag_stopped() {
                    save_editor_layout(state);
                }
                ui.small(format!(
                    "Floating Library · {:.0} × {:.0} · drag corner to resize",
                    size[0], size[1]
                ));
            });
        });
    if !open {
        state.editor_layout.library_open = false;
        save_editor_layout(state);
    }
}

/// Divider is a real drag affordance, not a decoration. Split positions are
/// persisted with the local editor layout and clamped to preserve a usable Library.
fn authoring_splitter(ui: &mut egui::Ui, position: &mut f32, minimum: f32, maximum: f32) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 9.0), egui::Sense::drag());
    let color = if response.hovered() || response.dragged() {
        egui::Color32::LIGHT_BLUE
    } else {
        egui::Color32::from_gray(70)
    };
    ui.painter().line_segment(
        [
            egui::pos2(rect.left() + 6.0, rect.center().y),
            egui::pos2(rect.right() - 6.0, rect.center().y),
        ],
        egui::Stroke::new(2.0, color),
    );
    if response.dragged() {
        *position = (*position + response.drag_delta().y).clamp(minimum, maximum.max(minimum));
        return true;
    }
    false
}

/// The M2D112 tooling + context panes are one coherent authoring workspace.
/// Their dividing grip is draggable, sizes persist, and the workspace can be
/// collapsed to the 62px tool rail without hiding the canvas or the Library.
fn pixel_hsv_to_rgb(h: f32, saturation: f32, value: f32) -> [u8; 3] {
    let h = h.rem_euclid(1.0) * 6.0;
    let c = value * saturation;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let (r, g, b) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = value - c;
    [r, g, b].map(|channel| ((channel + m).clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn pixel_rgb_to_hsv(rgb: [u8; 3]) -> (f32, f32, f32) {
    let [r, g, b] = rgb.map(|channel| f32::from(channel) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    if delta <= 0.0001 {
        return (0.0, 0.0, max);
    }
    let hue = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    (hue / 6.0, delta / max, max)
}

/// Canvas-owned color wheel with an HSV saturation/value square; this changes
/// only the editor's selected pixel color, never the original source PNG.
fn draw_pixel_color_wheel(ui: &mut egui::Ui, state: &mut Studio) {
    use std::f32::consts::TAU;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(126.0, 126.0), egui::Sense::click_and_drag());
    let center = rect.center();
    let rgba = state.editor_layout.pixel_palette_color;
    let (mut hue, mut saturation, mut value) = pixel_rgb_to_hsv([rgba[0], rgba[1], rgba[2]]);
    for i in 0..72 {
        let start = i as f32 / 72.0;
        let end = (i + 1) as f32 / 72.0;
        let angles = [start * TAU, end * TAU];
        let points = [
            egui::pos2(
                center.x + 57.0 * angles[0].cos(),
                center.y + 57.0 * angles[0].sin(),
            ),
            egui::pos2(
                center.x + 57.0 * angles[1].cos(),
                center.y + 57.0 * angles[1].sin(),
            ),
            egui::pos2(
                center.x + 43.0 * angles[1].cos(),
                center.y + 43.0 * angles[1].sin(),
            ),
            egui::pos2(
                center.x + 43.0 * angles[0].cos(),
                center.y + 43.0 * angles[0].sin(),
            ),
        ];
        let rgb = pixel_hsv_to_rgb(start, 1.0, 1.0);
        ui.painter().add(egui::Shape::convex_polygon(
            points.to_vec(),
            egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
            egui::Stroke::NONE,
        ));
    }
    // The inner square supplies saturation (X) and brightness (Y).
    let square = egui::Rect::from_center_size(center, egui::vec2(57.0, 57.0));
    for y in 0..12 {
        for x in 0..12 {
            let sw = square.width() / 12.0;
            let sh = square.height() / 12.0;
            let rgb = pixel_hsv_to_rgb(hue, x as f32 / 11.0, 1.0 - y as f32 / 11.0);
            let cell = egui::Rect::from_min_size(
                square.min + egui::vec2(x as f32 * sw, y as f32 * sh),
                egui::vec2(sw + 0.5, sh + 0.5),
            );
            ui.painter()
                .rect_filled(cell, 0.0, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        }
    }
    let marker = egui::pos2(
        square.left() + saturation * square.width(),
        square.bottom() - value * square.height(),
    );
    ui.painter()
        .circle_stroke(marker, 3.0, egui::Stroke::new(1.3, egui::Color32::BLACK));
    let angle = hue * TAU;
    ui.painter().circle_stroke(
        egui::pos2(center.x + angle.cos() * 50.0, center.y + angle.sin() * 50.0),
        4.0,
        egui::Stroke::new(1.5, egui::Color32::WHITE),
    );
    // Rust 2021 does not permit `&& let` chains. Keep the same drag/click
    // behavior and preserve original source-pixel immutability.
    if response.dragged() || response.clicked() {
        if let Some(pointer) = response.interact_pointer_pos() {
            let offset = pointer - center;
            let distance = offset.length();
            if (41.0..=62.0).contains(&distance) {
                hue = offset.y.atan2(offset.x).rem_euclid(TAU) / TAU;
            } else if square.contains(pointer) {
                saturation = ((pointer.x - square.left()) / square.width()).clamp(0.0, 1.0);
                value = ((square.bottom() - pointer.y) / square.height()).clamp(0.0, 1.0);
            }
            let rgb = pixel_hsv_to_rgb(hue, saturation, value);
            state.editor_layout.pixel_palette_color = [rgb[0], rgb[1], rgb[2], rgba[3]];
            if response.clicked() {
                save_editor_layout(state);
            }
        }
    }
    if response.drag_stopped() {
        save_editor_layout(state);
    }
}

/// A real editor color picker and project-local palette state. Editing source pixels
/// requires the future derived-image backend; these controls never mutate ElizaWy PNGs.
fn draw_pixel_studio(ui: &mut egui::Ui, state: &mut Studio) {
    ui.horizontal(|ui| {
        ui.strong("PIXEL STUDIO");
        if ui.small_button("Pixel mode").clicked() {
            choose_brush(state, BrushMode::Pixel);
        }
    });
    let rgba = state.editor_layout.pixel_palette_color;
    let mut color = egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]);
    ui.horizontal(|ui| {
        draw_pixel_color_wheel(ui, state);
        ui.vertical(|ui| {
            ui.label("Color");
            if ui.color_edit_button_srgba(&mut color).changed() {
                state.editor_layout.pixel_palette_color = color.to_array();
                save_editor_layout(state);
            }
            ui.colored_label(color, "■■");
            ui.small(format!("R {} G {} B {}", color.r(), color.g(), color.b()));
        });
    });
    ui.horizontal_wrapped(|ui| {
        for (name, rgb) in [
            ("Ink", [32, 48, 62]),
            ("Ocean", [27, 105, 143]),
            ("Water", [65, 172, 190]),
            ("Grass", [91, 145, 87]),
            ("Leaf", [151, 187, 109]),
            ("Sand", [225, 196, 135]),
            ("Earth", [146, 99, 72]),
            ("Stone", [121, 119, 135]),
            ("Snow", [218, 239, 240]),
            ("White", [255, 255, 255]),
        ] {
            let chip = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
            if ui
                .add(egui::Button::new("   ").fill(chip))
                .on_hover_text(name)
                .clicked()
            {
                state.editor_layout.pixel_palette_color = [rgb[0], rgb[1], rgb[2], 255];
                save_editor_layout(state);
            }
        }
    });
    if ui
        .add(
            egui::Slider::new(&mut state.editor_layout.pixel_brush_size_px, 1..=32)
                .text("Brush px"),
        )
        .changed()
    {
        save_editor_layout(state);
    }
    ui.small("Source pixels stay immutable. Direct pixel strokes require a derived-asset layer (not yet connected).");
}

/// Scene Navigator is a real selection view over the SAME WorldDocument and
/// SceneV2 as the canvas and inspector; it does not create shadow scene state.
fn draw_scene_navigator(ui: &mut egui::Ui, state: &mut Studio) {
    ui.strong("SCENE NAVIGATOR");
    if let Some(edit) = state.room_authoring.as_ref() {
        ui.label(format!(
            "Room · {}×{}",
            edit.scene.dimensions()[0],
            edit.scene.dimensions()[1]
        ));
        ui.small("Floor and wall topology · portals");
        return;
    }
    if state.world_mode {
        ui.small(format!(
            "World · {} · {} chunks",
            state.world_doc.season,
            state.world_doc.chunks.len()
        ));
        let chunk_rows: Vec<_> = state
            .world_doc
            .chunks
            .iter()
            .map(|chunk| (chunk.coord, chunk.generated_objects.len()))
            .collect();
        let mut selected_chunk = None;
        for (coord, count) in chunk_rows {
            let label = format!("Chunk {},{} · {count} objects", coord[0], coord[1]);
            if ui.selectable_label(false, label).clicked() {
                selected_chunk = Some(coord);
            }
        }
        if let Some(coord) = selected_chunk {
            state.world_selected = [
                coord[0] * state.world_doc.chunk_side as i64,
                coord[1] * state.world_doc.chunk_side as i64,
            ];
            state.world_selected_object = None;
            state.message = format!("Navigator selected chunk {},{}", coord[0], coord[1]);
        }
        ui.separator();
        let objects: Vec<_> = state
            .world_doc
            .generated_objects()
            .map(|object| {
                (
                    object.id.clone(),
                    object.label.clone(),
                    state.world_doc.object_anchor(object),
                )
            })
            .collect();
        egui::CollapsingHeader::new(format!("Structures / objects ({})", objects.len()))
            .id_salt("havenwild.navigator.world.objects")
            .show(ui, |ui| {
                for (id, label, at) in objects.iter().take(200) {
                    if ui
                        .selectable_label(
                            state.world_selected_object.as_ref() == Some(id),
                            format!("{label} · {},{}", at[0], at[1]),
                        )
                        .clicked()
                    {
                        state.world_selected_object = Some(id.clone());
                        state.world_selected = *at;
                        state.active_layer = LayerId::Objects;
                    }
                }
                if objects.len() > 200 {
                    ui.small("Showing first 200; select others on the canvas.");
                }
            });
        ui.separator();
        let links: Vec<_> = state
            .world_doc
            .structure_doors
            .values()
            .map(|link| (link.object_id.clone(), link.destination_scene.clone()))
            .collect();
        egui::CollapsingHeader::new(format!("Doorway scene links ({})", links.len()))
            .id_salt("havenwild.navigator.portal.links")
            .show(ui, |ui| {
                for (object, destination) in links {
                    if ui
                        .selectable_label(false, format!("{object} → {destination}"))
                        .clicked()
                    {
                        state.world_selected_object = Some(object);
                    }
                }
            });
    } else {
        ui.small("Authored scene · live visual layers");
        for id in LayerId::ORDERED {
            if ui
                .selectable_label(state.active_layer == id, format!("{:?}", id))
                .clicked()
            {
                state.active_layer = id;
            }
        }
        ui.small(format!(
            "Selected: {}",
            state.selected_object_id.as_deref().unwrap_or("None")
        ));
    }
}

/// M2D131: bottom Console owns lightweight editor/runtime diagnostics, not authored files.
/// PIE keeps this bottom dock accessible after all authoring docks collapse.
fn draw_console_dock(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if state.message != state.console_last_message {
        state.console_last_message = state.message.clone();
        if !state.message.is_empty() {
            state.console_history.push_back(state.message.clone());
            while state.console_history.len() > 256 {
                state.console_history.pop_front();
            }
        }
    }
    let expanded = state.editor_layout.workspace.bottom.open;
    let height = if expanded {
        state
            .editor_layout
            .workspace
            .bottom
            .extent
            .clamp(90.0, 450.0)
    } else {
        27.0
    };
    egui::Panel::bottom("havenwild.console.bottom")
        .resizable(false)
        .exact_size(height)
        .frame(egui::Frame::NONE.fill(canvas_chrome::panel_fill(&state.editor_layout.editor_theme)))
        .show(viewport_ui, |ui| {
            ui.horizontal(|ui| {
                if ui.small_button(if expanded { "▾ Console" } else { "▴ Console" }).clicked() {
                    state.editor_layout.workspace.bottom.open = !expanded;
                    save_editor_layout(state);
                }
                for tab in ["Console", "Problems", "Jobs", "Debug"] {
                    if ui.selectable_label(state.console_tab == tab, tab).clicked() {
                        state.console_tab = tab.to_owned();
                        state.editor_layout.workspace.bottom.open = true;
                        save_editor_layout(state);
                    }
                }
                if ui.small_button("Clear").on_hover_text("Clear this session's Console history only").clicked() {
                    state.console_history.clear();
                    state.console_last_message.clear();
                }
            });
            if !expanded { return; }
            let available_height = ui.available_height();
            egui::ScrollArea::vertical()
                .id_salt("havenwild.console.scroll")
                .max_height(available_height)
                .stick_to_bottom(true)
                .show(ui, |ui| match state.console_tab.as_str() {
                    "Problems" => {
                        if state.runtime_asset_sync_failures.is_empty() { ui.small("No runtime asset-sync errors reported this session."); }
                        for (path, reason) in &state.runtime_asset_sync_failures {
                            ui.colored_label(egui::Color32::YELLOW, format!("{path}: {reason}"));
                        }
                    }
                    "Jobs" => {
                        ui.small(format!("Asset load queue: {} pending", state.runtime_asset_sync_queue.len()));
                        for job in &state.runtime_asset_sync_queue { ui.small(job); }
                        ui.small("Other PCC/build jobs appear in the external PCC logs until job streaming is connected.");
                    }
                    "Debug" => {
                        ui.small(format!("FPS: {:.0} · zoom: {:.2}x · backend: see PCC launch logs", state.display_fps, state.zoom));
                        ui.small(format!("World: {} · render sheets: {} · awaiting textures: {}", state.world_mode,
                            state.sheets.len(), state.sheets.iter().filter(|sheet| sheet.texture.is_none()).count()));
                    }
                    _ => {
                        for line in &state.console_history { ui.label(line); }
                    }
                });
            let grip = ui.allocate_response(egui::vec2(ui.available_width(), 9.0), egui::Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                .on_hover_text("Drag to resize the Console");
            if grip.dragged() {
                state.editor_layout.workspace.bottom.extent =
                    (state.editor_layout.workspace.bottom.extent - grip.drag_delta().y).clamp(90.0, 450.0);
                ui.ctx().request_repaint();
            }
            if grip.drag_stopped() { save_editor_layout(state); }
        });
}

/// One ownership path for top-center Grid/Chunks and top-right Fit/Zoom.
/// Drawing occurs inside the central canvas Ui so independent dialogs retain
/// their z-order. Geometry was reserved before world input processing.
fn draw_canvas_view_controls(ui: &mut egui::Ui, state: &mut Studio) {
    let canvas = state.canvas_rect;
    if canvas.width() < 330.0 || canvas.height() < 150.0 {
        return;
    }
    let ruler = if canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode) {
        canvas_rulers::TOP
    } else {
        0.0
    };
    let zones = canvas_chrome::overlay_geometry(canvas, ruler);
    let theme = state.editor_layout.editor_theme.clone();
    if zones.grid != egui::Rect::NOTHING {
        ui.scope_builder(egui::UiBuilder::new().max_rect(zones.grid), |ui| {
            canvas_chrome::tool_group(ui, &theme, |ui| {
                ui.horizontal(|ui| {
                    if canvas_chrome::toggle(
                        ui,
                        "Grid",
                        state.editor_layout.tile_grid_visible,
                        "Show or hide the authored 32px terrain-cell grid (view only)",
                        &theme,
                    ) {
                        state.editor_layout.tile_grid_visible =
                            !state.editor_layout.tile_grid_visible;
                        save_editor_layout(state);
                    }
                    if canvas_chrome::toggle(
                        ui,
                        "Chunks",
                        state.editor_layout.chunk_grid_visible,
                        "Show or hide chunk outlines and coordinates (view only)",
                        &theme,
                    ) {
                        state.editor_layout.chunk_grid_visible =
                            !state.editor_layout.chunk_grid_visible;
                        save_editor_layout(state);
                    }
                    if canvas_chrome::toggle(
                        ui,
                        "Rulers",
                        state.editor_layout.rulers_visible,
                        "Show/hide opaque pan/zoom-aware X/Y rulers in the canvas (view only)",
                        &theme,
                    ) {
                        state.editor_layout.rulers_visible = !state.editor_layout.rulers_visible;
                        save_editor_layout(state);
                    }
                });
            });
        });
    }
    if zones.zoom != egui::Rect::NOTHING {
        ui.scope_builder(egui::UiBuilder::new().max_rect(zones.zoom), |ui| {
            canvas_chrome::tool_group(ui, &theme, |ui| {
                ui.horizontal(|ui| {
                    if canvas_chrome::action(
                        ui,
                        "Fit",
                        "Fit the current world/scene to the canvas",
                        &theme,
                    ) {
                        if state.world_mode {
                            if let Some((_, size)) = state.world_doc.materialized_bounds() {
                                let fx = canvas.width() / size[0] as f32 / 32.0;
                                let fy = canvas.height() / size[1] as f32 / 32.0;
                                state.zoom = fx.min(fy).clamp(0.125, 16.0) * 0.92;
                                state.pan = egui::Vec2::ZERO;
                            }
                        } else {
                            let size = state.scene.size;
                            focus_mapping_zone(state, [0, 0], size);
                        }
                    }
                    if canvas_chrome::action(ui, "−", "Zoom out · Ctrl+-", &theme) {
                        zoom_canvas_by(state, canvas.center(), canvas, 0.8);
                    }
                    ui.small(format!("{:.0}%", state.zoom * 100.0));
                    if canvas_chrome::action(ui, "+", "Zoom in · Ctrl+=", &theme) {
                        zoom_canvas_by(state, canvas.center(), canvas, 1.25);
                    }
                });
            });
        });
    }
}

fn canvas_animation_control_rect(canvas: egui::Rect, rulers: bool) -> egui::Rect {
    let x = canvas.left() + 8.0 + if rulers { canvas_rulers::LEFT } else { 0.0 };
    egui::Rect::from_min_size(
        egui::pos2(x, canvas.bottom() - 33.0),
        egui::vec2(108.0, 27.0),
    )
}

// One compact canvas-owned Animation control, bottom-left and never over gameplay.
fn draw_canvas_animation_toggle(ui: &mut egui::Ui, state: &mut Studio) {
    let rect = state.canvas_rect;
    if rect.width() < 200.0 || rect.height() < 100.0 {
        return;
    }
    let anchor = canvas_animation_control_rect(
        rect,
        canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
    );
    // Keep theme styling independent from the mutable layout state used by the toggle.
    let theme = state.editor_layout.editor_theme.clone();
    ui.scope_builder(egui::UiBuilder::new().max_rect(anchor), |ui| {
        canvas_chrome::tool_group(ui, &theme, |ui| {
            if canvas_chrome::toggle(
                ui,
                if state.editor_layout.timeline_open {
                    "▾ Animation"
                } else {
                    "▴ Animation"
                },
                state.editor_layout.timeline_open,
                "Expand or collapse the source-backed animation timeline below the canvas",
                &theme,
            ) {
                state.editor_layout.timeline_open = !state.editor_layout.timeline_open;
                save_editor_layout(state);
            }
        });
    });
}

/// Shared bottom timeline owns preview time, not the asset document or game clock.
/// Selecting a source-backed animated terrain vertex exposes actual frame art
/// and durations from the same cache used by the world renderer.
fn draw_animation_timeline(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if !state.editor_layout.timeline_open {
        return;
    }
    let height = state.editor_layout.timeline_height;
    egui::Panel::bottom("havenwild.animation.timeline")
        .resizable(false)
        .exact_size(height)
        .frame(egui::Frame::NONE.fill(canvas_chrome::panel_fill(&state.editor_layout.editor_theme)))
        .show(viewport_ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Animation Timeline");
                if canvas_chrome::action(
                    ui,
                    if state.editor_layout.timeline_open { "▾" } else { "▴" },
                    "Collapse/expand the canvas-owned animation timeline",
                    &state.editor_layout.editor_theme,
                ) {
                    state.editor_layout.timeline_open = !state.editor_layout.timeline_open;
                    save_editor_layout(state);
                }
                if canvas_chrome::action(ui, "|◀", "Rewind animation preview", &state.editor_layout.editor_theme) {
                    state.animation_preview_ms = 0.0;
                }
                if canvas_chrome::toggle(ui, if state.editor_layout.timeline_playing { "Pause" } else { "Play" }, state.editor_layout.timeline_playing, "Pause/play the editor world-animation preview", &state.editor_layout.editor_theme) {
                    state.editor_layout.timeline_playing = !state.editor_layout.timeline_playing;
                    save_editor_layout(state);
                }
                if canvas_chrome::action(ui, "▶|", "Advance preview 200 ms", &state.editor_layout.editor_theme) {
                    state.editor_layout.timeline_playing = false;
                    state.animation_preview_ms = (state.animation_preview_ms + 200.0).rem_euclid(60_000.0);
                    save_editor_layout(state);
                }
                ui.small(format!("{:.0} ms", state.animation_preview_ms));
                if !state.editor_layout.timeline_open { ui.small("· source-backed preview"); }
            });
            if !state.editor_layout.timeline_open { return; }
            let grip = ui.allocate_response(egui::vec2(ui.available_width(), 9.0), egui::Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                .on_hover_text("Drag the timeline upper edge to resize");
            ui.painter().hline(grip.rect.left()..=grip.rect.right(), grip.rect.center().y,
                egui::Stroke::new(2.0, egui::Color32::from_gray(100)));
            if grip.dragged() {
                state.editor_layout.timeline_height = (state.editor_layout.timeline_height
                    - grip.drag_delta().y).clamp(100.0, 400.0);
                ui.ctx().request_repaint();
            }
            if grip.drag_stopped() { save_editor_layout(state); }
            let mut preview = state.animation_preview_ms;
            if ui.add(egui::Slider::new(&mut preview, 0.0..=10_000.0)
                .text("Scrub ms")).changed() {
                state.animation_preview_ms = preview;
                state.editor_layout.timeline_playing = false;
                save_editor_layout(state);
            }
            let selected_vertex = state.world_selected;
            let preview_parts = if state.world_mode {
                cached_world_vertex_visual(state, selected_vertex)
                    .into_iter().flat_map(|visual| visual.parts)
                    .filter_map(|part| part.animation).collect::<Vec<_>>()
            } else { Vec::new() };
            if let Some(animation) = preview_parts.first() {
                ui.horizontal(|ui| {
                    ui.small(format!("World cell {},{} · {} source frames · {} ms loop",
                        state.world_selected[0], state.world_selected[1],
                        animation.frames.len(), animation.total_duration_ms));
                });
                egui::ScrollArea::horizontal().id_salt("havenwild.timeline.frames").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut frame_start = 0u64;
                        for (index, frame) in animation.frames.iter().enumerate() {
                            let (rect, response) = ui.allocate_exact_size(egui::vec2(48.0, 42.0), egui::Sense::click());
                            if let Some(texture) = state.sheets[frame.sheet_index].texture {
                                ui.painter().image(texture, rect.shrink(3.0),
                                    source_uv(frame.source_rect, state.sheets[frame.sheet_index].size),
                                    egui::Color32::WHITE);
                            }
                            ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0,
                                egui::Color32::from_gray(100)), egui::StrokeKind::Inside);
                            if response.clicked() {
                                state.animation_preview_ms = (frame_start + animation.total_duration_ms
                                    - animation.phase_ms % animation.total_duration_ms) as f64
                                    % animation.total_duration_ms as f64;
                                state.editor_layout.timeline_playing = false;
                            }
                            response.on_hover_text(format!("Frame {} · {} ms · click to preview", index + 1, frame.duration_ms));
                            frame_start += u64::from(frame.duration_ms);
                        }
                    });
                });
            } else {
                ui.small("Select an animated water/shore vertex to inspect its original frames. Playback uses source-authored timings.");
            }
        });
}

fn draw_tool_context(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if !state.editor_layout.tool_context_open {
        return;
    }
    // One resizable dock contains exactly three sections: mode-specific tools,
    // Pixel Studio, and Scene Navigator. No parallel 62px tool rail exists.
    let preferred_width = state.editor_layout.authoring_context_width;
    egui::Panel::left("havenwild.tool.context")
        .resizable(true)
        .default_size(preferred_width)
        .min_size(210.0)
        .max_size(490.0)
        .frame(egui::Frame::NONE.fill(canvas_chrome::panel_fill(&state.editor_layout.editor_theme)))
        .show(viewport_ui, |ui| {
            let height = ui.available_height().max(330.0);
            let top_height = state
                .editor_layout
                .authoring_top_height
                .clamp(125.0, (height - 260.0).max(125.0));
            ui.allocate_ui(egui::vec2(ui.available_width(), top_height), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("havenwild.studio.mode_tools")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.strong("MODE & TOOLS");
                        draw_tool_rail(ui, state);
                        ui.separator();
                        draw_authoring_tooling(ui, state);
                        draw_authoring_brush_settings(ui, state);
                    });
            });
            if authoring_splitter(
                ui,
                &mut state.editor_layout.authoring_top_height,
                125.0,
                (height - 260.0).max(125.0),
            ) {
                save_editor_layout(state);
            }
            let middle_height = state
                .editor_layout
                .authoring_middle_height
                .clamp(120.0, (height - top_height - 120.0).max(120.0));
            ui.allocate_ui(egui::vec2(ui.available_width(), middle_height), |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("havenwild.studio.pixel")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        draw_pixel_studio(ui, state);
                    });
            });
            if authoring_splitter(
                ui,
                &mut state.editor_layout.authoring_middle_height,
                120.0,
                (height - top_height - 120.0).max(120.0),
            ) {
                save_editor_layout(state);
            }
            egui::ScrollArea::vertical()
                .id_salt("havenwild.studio.navigator")
                .show(ui, |ui| {
                    draw_scene_navigator(ui, state);
                    // Legacy optional Library destination remains available for
                    // layouts intentionally configured this way. One browser only.
                    if state.editor_layout.library_in_left_workspace
                        && state.editor_layout.library_open
                        && !state.editor_layout.library_floating
                        && !state.editor_layout.library_in_inspector
                    {
                        ui.separator();
                        egui::CollapsingHeader::new("LIBRARY / PALETTE")
                            .show(ui, |ui| draw_authoring_library(ui, state));
                    }
                });
            let width = ui.max_rect().width();
            if (width - state.editor_layout.authoring_context_width).abs() > 2.0 && width >= 210.0 {
                state.editor_layout.authoring_context_width = width;
                save_editor_layout(state);
            }
        });
}

fn draw_inspector_rail(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if !state.editor_layout.workspace.right.open {
        return;
    }
    let width = state
        .editor_layout
        .workspace
        .right
        .extent
        .clamp(180.0, 700.0);
    let mut measured_width = width;
    egui::Panel::right("havenwild.inspector.rail")
        .resizable(true)
        .default_size(width)
        .size_range(180.0..=700.0)
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(27, 34, 39)))
        .show(viewport_ui, |ui| {
            measured_width = ui.max_rect().width();
            let drawer_height = if state.editor_layout.metrics_drawer_open { 228.0 } else { 0.0 };
            let library_here = state.editor_layout.library_in_inspector && state.editor_layout.library_open
                && !state.editor_layout.library_floating;
            let split_available = (ui.available_height() - drawer_height - 32.0).max(240.0);
            let library_height = if library_here {
                split_available * (1.0 - state.editor_layout.inspector_split_ratio)
            } else { 0.0 };
            let content_height = if library_here {
                split_available - library_height
            } else {
                (ui.available_height() - drawer_height - 28.0).max(80.0)
            };
            // Reserve the Inspector's full half even when its content is short.
            // Otherwise the browser shifts up instead of forming a 50/50 dock.
            egui::ScrollArea::vertical().max_height(content_height)
                .auto_shrink([false, false]).show(ui, |ui| {
                ui.add_space(6.0);
                ui.heading("Inspector");
                ui.horizontal_wrapped(|ui| {
                    for tab in ["Inspect", "World", "Terrain"] {
                        if ui.selectable_label(state.editor_layout.inspector_tab == tab, tab).clicked() {
                            state.editor_layout.inspector_tab = tab.to_owned();
                            // Pixel collision strokes require the explicit World
                            // authoring tab; the adjacent HUD eye is display-only.
                            state.editor_layout.structural_open = tab == "World";
                            if tab != "World" {
                                state.collision_world_pixel_edit = false;
                            }
                            save_editor_layout(state);
                        }
                    }
                });
                ui.separator();
                if state.editor_layout.inspector_tab == "World" {
                    draw_world_properties_contents(ui, state);
                } else if state.editor_layout.inspector_tab == "Terrain" {
                    ui.small("Semantic terrain and source-backed Wang rule diagnostics.");
                    if ui.button("Open Terrain Rules").clicked() {
                        open_mapper_tab(state, "patterns");
                    }
                    ui.small(format!("Selected cell: {},{}", state.world_selected[0], state.world_selected[1]));
                    ui.small(&state.shoreline_audit_summary);
                } else {
                ui.small(format!("Edit: {}", active_tool_label(state)));
                ui.small(format!("Layer: {:?}", state.active_layer));
                ui.separator();
                if state.world_mode {
                    ui.small(format!(
                        "Render sheets: {} registered · {} awaiting image texture · {} queued · {} failed",
                        state.sheets.len(),
                        state.sheets.iter().filter(|sheet| sheet.texture.is_none()).count(),
                        state.runtime_asset_sync_queue.len(),
                        state.runtime_asset_sync_failures.len(),
                    ));
                    if ui.small_button("Rescan world asset bindings")
                        .on_hover_text("Request any source sheets used by the current world but not loaded; no pixels or metadata modified.")
                        .clicked()
                    {
                        state.runtime_asset_sync_revision = None;
                    }
                    if !state.runtime_asset_sync_failures.is_empty() {
                        egui::CollapsingHeader::new("Missing world render assets")
                            .id_salt("havenwild.world.missing.render.asset.paths")
                            .show(ui, |ui| {
                                for (path, error) in &state.runtime_asset_sync_failures {
                                    ui.colored_label(egui::Color32::YELLOW, format!("{path}: {error}"));
                                }
                            });
                    }
                    if ui.small_button("Diagnose visible stones").on_hover_text("Report generated stone instances and whether Objects and minor details are visible; does not invent source art.").clicked() {
                        let stones = state.world_doc.generated_objects().filter(|obj| {
                            obj.label.to_ascii_lowercase().contains("stone")
                                || obj.template_id.to_ascii_lowercase().contains("rock")
                        }).count();
                        let objects_visible = world_layer_visible(state, LayerId::Objects);
                        state.message = format!("Generated stones: {stones}; Objects visible: {objects_visible}; minor LOD visible: {}. If count = 0, regenerate a world to apply M2D117 placement changes.", state.world_detail_lod_visible);
                    }
                    if let Some(id) = state.world_selected_object.clone() {
                        // Complete the immutable object lookup before mutating the world or undo stack.
                        // Holding the generated_objects() iterator inside if-let keeps the
                        // immutable WorldDocument borrow alive until the branch ends (E0502).
                        let selected_object = state.world_doc.generated_objects()
                            .find(|object| object.id == id)
                            .cloned();
                        if let Some(object) = selected_object {
                            ui.strong(&object.label);
                            ui.small(format!("Template: {}", object.template_id));
                            ui.small(format!("Source layer: {}", object.layer));
                            ui.small(format!("Visual cells: {} × {}", object.footprint_cells[0], object.footprint_cells[1]));
                            ui.separator();
                            ui.label("Object position (cells)");
                            let mut position = state.world_doc.object_anchor(&object);
                            ui.horizontal(|ui| {
                                ui.label("X"); ui.add(egui::DragValue::new(&mut position[0]).speed(1.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Y"); ui.add(egui::DragValue::new(&mut position[1]).speed(1.0));
                            });
                            if position != state.world_doc.object_anchor(&object) {
                                if state.world_doc.can_move_generated_object_to(&id, position) {
                                    checkpoint_world(state);
                                    if state.world_doc.move_generated_object(&id, position).is_ok() {
                                        state.world_dirty = true;
                                        state.message = "Moved generated object from Inspector".into();
                                    } else { let _ = state.world_undo.pop(); }
                                } else {
                                    ui.small("Destination must remain in generated world.");
                                }
                            }
                            ui.separator();
                            ui.small("Source-backed. Position edits are stored separately and survive regeneration; this is not collision certification.");
                            if ui.button("Delete generated instance").clicked() {
                                checkpoint_world(state);
                                if state.world_doc.suppress_generated_object(&id) {
                                    state.world_selected_object = None;
                                    state.world_dirty = true;
                                } else { let _ = state.world_undo.pop(); }
                            }
                        } else {
                            ui.small("The selected object is no longer present.");
                        }
                    } else {
                        ui.small("Select a tree, building, or other object on the canvas to inspect and move it.");
                    }
                    ui.separator();
                    ui.strong("Terrain at selection");
                    ui.small(format!("World cell: {}, {}", state.world_selected[0], state.world_selected[1]));
                    let cell = state.world_selected;
                    let roles = state.world_doc.resolved_dual_grid_roles(cell);
                    let four = [roles[0].as_str(), roles[1].as_str(), roles[2].as_str(), roles[3].as_str()];
                    ui.small(format!("NW {} · NE {}", four[0], four[1]));
                    ui.small(format!("SW {} · SE {}", four[2], four[3]));
                    // Exact source lookup is read-only; one-off Tile Brush overrides
                    // must never silently replace the terrain rule everywhere.
                    let located = state.tiled_terrain.as_ref().and_then(|tiled| {
                        if let Some(id) = tiled.resolve_havenwild_corner_tile_id(four, state.world_doc.seed, cell) {
                            return tiled.canonical_visual_binding(id).cloned().map(|source| ("Exact Tiled rule", id, source));
                        }
                        let projected = WorldDocument::land_connectivity_projection(four);
                        tiled.resolve_havenwild_corner_tile_id(projected, state.world_doc.seed, cell)
                            .and_then(|id| tiled.canonical_visual_binding(id).cloned().map(|source| ("Projected fallback", id, source)))
                    });
                    if let Some((status, tile_id, binding)) = located {
                        ui.small(format!("{status} · tile #{tile_id}"));
                        ui.small(format!("{} @ {},{}", binding.source_asset, binding.source_rect[0], binding.source_rect[1]));
                        if ui.button("Locate source in Library").clicked() {
                            state.pending_source_sheet = Some(binding.source_asset.clone());
                            state.pending_source_cell = Some([binding.source_rect[0] / 32, binding.source_rect[1] / 32]);
                            state.selected_cell = [binding.source_rect[0] / 32, binding.source_rect[1] / 32];
                            state.selected_extent = [binding.source_rect[2].div_ceil(32).max(1), binding.source_rect[3].div_ceil(32).max(1)];
                            state.editor_layout.authoring_library_tab = "Tiles".into();
                            state.editor_layout.library_open = true;
                            save_editor_layout(state);
                        }
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "No exact or projected Tiled rule; source-backed fallback may appear.");
                    }
                    if ui.small_button("Inspect transition rules").clicked() {
                        open_mapper_tab(state, "patterns");
                    }
                    ui.separator();
                    ui.small(format!("{} materialized + authored instances", state.world_doc.generated_objects().count()));
                } else if let Some(object) = selected_object(state).cloned() {
                    ui.strong(&object.label);
                    ui.small(format!("Instance: {}", object.id));
                    ui.small(format!("Bounds: {} × {} cells", object.footprint[0], object.footprint[1]));
                    ui.separator();
                    let mut pos = object.anchor;
                    ui.horizontal(|ui| { ui.label("X"); ui.add(egui::DragValue::new(&mut pos[0]).speed(1.0)); });
                    ui.horizontal(|ui| { ui.label("Y"); ui.add(egui::DragValue::new(&mut pos[1]).speed(1.0)); });
                    if pos != object.anchor { move_selected_object_to_origin(state, object.id.clone(), pos); }
                    if ui.button("Remove instance").clicked() { remove_selected_object(state); }
                } else {
                    ui.small("Select a scene object to inspect its instance and position.");
                }
                } // Inspector Inspect tab
            });
            if state.editor_layout.library_in_inspector && !state.editor_layout.library_floating {
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.small_button(if state.editor_layout.library_open { "▾ Assets" } else { "▴ Assets" }).clicked() {
                        state.editor_layout.library_open = !state.editor_layout.library_open;
                        save_editor_layout(state);
                    }
                    ui.small("Contextual Asset Browser");
                });
                if state.editor_layout.library_open {
                    let (grip_rect, grip) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 9.0), egui::Sense::drag());
                    ui.painter().hline(grip_rect.left()..=grip_rect.right(), grip_rect.center().y,
                        egui::Stroke::new(1.5, egui::Color32::from_gray(110)));
                    let grip = grip.on_hover_cursor(egui::CursorIcon::ResizeVertical)
                        .on_hover_text("Drag up/down to resize Inspector asset browser");
                    if grip.dragged() {
                        state.editor_layout.inspector_split_ratio =
                            (state.editor_layout.inspector_split_ratio
                                + grip.drag_delta().y / split_available).clamp(0.20, 0.80);
                        ui.ctx().request_repaint();
                    }
                    if grip.drag_stopped() { save_editor_layout(state); }
                    ui.allocate_ui(egui::vec2(ui.available_width(), library_height.max(110.0)), |ui| {
                        egui::ScrollArea::vertical().id_salt("havenwild.inspector.asset_library")
                            .max_height(library_height.max(110.0))
                            .auto_shrink([false, false])
                            .show(ui, |ui| draw_authoring_library(ui, state));
                    });
                }
            }
            // Reserving inspector-only height keeps the diagnostics below its content,
            // rather than floating over the world canvas or stealing a third dock.
            if state.editor_layout.metrics_drawer_open {
                let remaining = ui.available_height();
                if remaining > drawer_height { ui.add_space(remaining - drawer_height); }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.strong("Metrics");
                    if ui.small_button("▾").on_hover_text("Collapse to status bar").clicked() {
                        state.editor_layout.metrics_drawer_open = false;
                        save_editor_layout(state);
                    }
                });
                ui.small(format!("FPS: {:.1}  |  Zoom: {:.0}%", state.display_fps, state.zoom * 100.0));
                ui.small(format!("Cache: {} vertices", state.world_visual_cache.vertices.len()));
                if state.world_mode {
                    ui.small(format!("Chunks: {}  |  World rev: {}", state.world_doc.chunks.len(), state.world_doc.revision));
                    ui.small(format!("Overrides: {} cells / {} object positions / {} authored objects",
                        state.world_doc.cell_overrides.len(), state.world_doc.moved_generated_objects.len(), state.world_doc.authored_objects.len()));
                    if ui.small_button("Audit shoreline topology")
                        .on_hover_text("Count exact source-backed Wang corners, including land/dirt/sand and coast, versus projected and missing recipes. Does not alter terrain.")
                        .clicked()
                    {
                        let mut exact = 0usize;
                        let mut projected = 0usize;
                        let mut missing = 0usize;
                        let mut first_missing = None;
                        if let Some(tiled) = state.tiled_terrain.as_ref() {
                            for chunk in &state.world_doc.chunks {
                                let side = state.world_doc.chunk_side as i64;
                                let base = [chunk.coord[0] * side, chunk.coord[1] * side];
                                for y in 0..side {
                                    for x in 0..side {
                                        let v = [base[0] + x, base[1] + y];
                                        let roles = state.world_doc.resolved_dual_grid_roles(v);
                                        let ref_roles = [roles[0].as_str(), roles[1].as_str(), roles[2].as_str(), roles[3].as_str()];
                                        // Count every unsupported corner, including sand/dirt/grass
                                        // junctions. Previously this diagnostic only saw coastline.
                                        if tiled.resolve_havenwild_corner_tile_id(ref_roles, state.world_doc.seed, v).is_some() {
                                            exact += 1;
                                        } else {
                                            let pair = world_doc::WorldDocument::land_connectivity_projection(ref_roles);
                                            if tiled.resolve_havenwild_corner_tile_id(pair, state.world_doc.seed, v).is_some() {
                                                projected += 1;
                                            } else {
                                                missing += 1;
                                                if first_missing.is_none() { first_missing = Some((v, roles)); }
                                            }
                                        }
                                    }
                                }
                            }
                            state.shoreline_audit_summary = format!("Terrain Wang exact: {exact} · projected: {projected} · missing: {missing}{}",
                                first_missing.map(|(pos, roles)| format!(" · first {:?}: {:?}", pos, roles)).unwrap_or_default());
                        } else { state.shoreline_audit_summary = "Configured Tiled terrain unavailable; source-authority preflight required".into(); }
                    }
                    if ui.checkbox(&mut state.editor_layout.shoreline_audit_overlay, "Outline missing coast recipes and land junctions")
                        .on_hover_text("Yellow: projected recipe; red: no exact or projected Wang recipe. Overlay is editor-only.")
                        .changed()
                    { save_editor_layout(state); }
                    ui.small(&state.shoreline_audit_summary);
                }
            }
        });
    // Persist only after resizing completes; world/asset state is untouched.
    if viewport_ui
        .ctx()
        .input(|input| input.pointer.any_released())
        && (measured_width - state.editor_layout.workspace.right.extent).abs() > 1.0
    {
        state.editor_layout.workspace.right.extent = measured_width.clamp(180.0, 700.0);
        save_editor_layout(state);
    }
}

/// Pixel-space exclusion zone for the canvas-integrated Layers controls.
/// World/object hit tests use the same rectangle before accepting a stroke.
fn canvas_layer_rect(canvas: egui::Rect, expanded: bool, rulers: bool) -> egui::Rect {
    // Two independent buttons share the same top row, and only this small
    // opaque, click-owned cluster excludes strokes from the world canvas.
    let width = 84.0;
    let height = if expanded { 282.0 } else { 40.0 };
    let inset = if rulers {
        egui::vec2(canvas_rulers::LEFT, canvas_rulers::TOP)
    } else {
        egui::Vec2::ZERO
    };
    egui::Rect::from_min_size(
        canvas.min + inset + egui::vec2(10.0, 10.0),
        egui::vec2(width, height),
    )
}

/// The layer control belongs to the World/Scene canvas like rulers and view
/// controls; it is not a left Panel or a global foreground floating window.
/// Individual icon chips sit directly on the canvas with no enclosing panel.
fn draw_layer_rail(canvas_ui: &mut egui::Ui, state: &mut Studio) {
    let canvas = state.canvas_rect;
    if canvas.width() < 180.0 || canvas.height() < 180.0 {
        return;
    }
    let expanded = state.editor_layout.layers_dock_open;
    let rect = canvas_layer_rect(
        canvas,
        expanded,
        canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
    );
    state.canvas_layers_rect = rect;
    canvas_ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.set_clip_rect(canvas);
            // Canvas overlay: Layers expands the strip; Collision is view-only.
            // A solid backing makes these tools readable over shoreline art.
            // Both stay visible even when the layers themselves are collapsed.
            let header = ui.cursor();
            ui.painter().rect_filled(
                egui::Rect::from_min_size(header.min, egui::vec2(84.0, 33.0)),
                5.0,
                canvas_chrome::panel_fill(&state.editor_layout.editor_theme),
            );
            ui.horizontal(|ui| {
                if rail_icon_button(
                    ui,
                    RailIcon::Layers,
                    expanded,
                    if expanded {
                        "Collapse canvas Layers"
                    } else {
                        "Expand canvas Layers"
                    },
                    [36.0, 31.0],
                )
                .clicked()
                {
                    state.editor_layout.layers_dock_open = !expanded;
                    save_editor_layout(state);
                }
                if rail_icon_button(
                    ui,
                    RailIcon::Collision,
                    state.editor_layout.collision_overlay,
                    "Show/hide collision visualization only — no edit mode or dialog",
                    [36.0, 31.0],
                )
                .clicked()
                {
                    state.editor_layout.collision_overlay = !state.editor_layout.collision_overlay;
                    save_editor_layout(state);
                }
            });
            if !expanded {
                return;
            }
            ui.add_space(3.0);
            for (id, icon, label) in [
                (
                    LayerId::Foreground,
                    RailIcon::Foreground,
                    "Foreground / canopy / overlays",
                ),
                (
                    LayerId::Objects,
                    RailIcon::Objects,
                    "Objects / vegetation / forageables",
                ),
                (
                    LayerId::Structures,
                    RailIcon::Structures,
                    "Structures / buildings / bridge pieces",
                ),
                (
                    LayerId::Elevation,
                    RailIcon::Elevation,
                    "Elevation / cliffs",
                ),
                (LayerId::Water, RailIcon::Water, "Water / water FX"),
                (
                    LayerId::TerrainDetails,
                    RailIcon::Details,
                    "Terrain details / banks / shoreline",
                ),
                (LayerId::Ground, RailIcon::Ground, "Ground terrain"),
            ] {
                let slot = world_layer_slot(id);
                let visible = state.editor_layout.world_layer_visibility[slot];
                let row_rect = ui.cursor();
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(row_rect.min, egui::vec2(62.0, 27.0)),
                    4.0,
                    canvas_chrome::panel_fill(&state.editor_layout.editor_theme),
                );
                ui.horizontal(|ui| {
                    if rail_icon_button(
                        ui,
                        if visible {
                            RailIcon::Eye
                        } else {
                            RailIcon::EyeOff
                        },
                        false,
                        if visible {
                            "Hide this layer"
                        } else {
                            "Show this layer"
                        },
                        [24.0, 26.0],
                    )
                    .clicked()
                    {
                        state.editor_layout.world_layer_visibility[slot] = !visible;
                        state.layer_isolate.cancel();
                        save_editor_layout(state);
                        state.message = format!(
                            "{} layer {}.",
                            label,
                            if visible { "hidden" } else { "visible" }
                        );
                    }
                    if rail_icon_button(
                        ui,
                        icon,
                        state.active_layer == id,
                        &format!("{label} · click to select · Shift-click to isolate"),
                        [28.0, 26.0],
                    )
                    .clicked()
                    {
                        let isolate = ui.input(|input| input.modifiers.shift);
                        state.active_layer = id;
                        let now_isolated = if isolate {
                            state
                                .layer_isolate
                                .toggle(&mut state.editor_layout.world_layer_visibility, slot)
                        } else {
                            false
                        };
                        state.selected_object_id = None;
                        state.dragging_object = None;
                        state.drag_preview_origin = None;
                        state.world_selected_object = None;
                        state.world_dragging_object = None;
                        state.world_drag_preview_anchor = None;
                        state.message = if now_isolated {
                            format!("Isolated {label} layer (Shift-click again to restore).")
                        } else if isolate {
                            format!("Restored previous layer visibility; active layer: {label}.")
                        } else {
                            format!("Active visual layer: {label}.")
                        };
                        save_editor_layout(state);
                    }
                });
            }
        },
    );
}

fn docked_overlay_window<'a>(
    title: &'a str,
    id: &'a str,
    dock: SurfaceDock,
    compact: bool,
    open: &'a mut bool,
) -> egui::Window<'a> {
    let window = egui::Window::new(title)
        .id(egui::Id::new(id))
        .open(open)
        .resizable(true)
        .min_width(if compact { 250.0 } else { 330.0 });
    match dock {
        SurfaceDock::Left => window.anchor(egui::Align2::LEFT_TOP, [60.0, 62.0]),
        SurfaceDock::Right => window.anchor(egui::Align2::RIGHT_TOP, [-58.0, 62.0]),
        SurfaceDock::Bottom => window.anchor(egui::Align2::CENTER_BOTTOM, [0.0, -30.0]),
        SurfaceDock::Floating | SurfaceDock::Center => window.default_pos(egui::pos2(850.0, 110.0)),
    }
}
fn panel_header(
    ui: &mut egui::Ui,
    dock: SurfaceDock,
    compact: bool,
) -> (Option<SurfaceDock>, bool) {
    let mut d = None;
    let mut c = compact;
    ui.horizontal_wrapped(|ui| {
        for (candidate, label) in [
            (SurfaceDock::Left, "Dock L"),
            (SurfaceDock::Right, "Dock R"),
            (SurfaceDock::Bottom, "Dock B"),
            (SurfaceDock::Floating, "Float"),
        ] {
            if ui.selectable_label(dock == candidate, label).clicked() {
                d = Some(candidate);
            }
        }
        ui.separator();
        if ui
            .selectable_label(c, if c { "Compact" } else { "Full" })
            .clicked()
        {
            c = !c;
        }
    });
    (d, c)
}

fn draw_world_generator_panel(ctx: &egui::Context, state: &mut Studio) {
    if !state.editor_layout.world_generator_open {
        state.world_generator_rect = egui::Rect::NOTHING;
        return;
    }
    let dock = state.editor_layout.world_generator_dock;
    let compact = state.editor_layout.world_generator_compact;
    let mut open = true;
    let response = docked_overlay_window(
        "Generated World",
        "havenwild.world.generator.overlay",
        dock,
        compact,
        &mut open,
    )
    .show(ctx, |ui| {
        let (requested_dock, requested_compact) = panel_header(ui, dock, compact);
        if let Some(value) = requested_dock {
            state.editor_layout.world_generator_dock = value;
            save_editor_layout(state);
        }
        if requested_compact != compact {
            state.editor_layout.world_generator_compact = requested_compact;
            save_editor_layout(state);
        }

        egui::CollapsingHeader::new("Generate")
            .id_salt("havenwild.world.generator.generate")
            .default_open(true)
            .show(ui, |ui| {
                let mut seed = state.world_doc.seed;
                ui.horizontal(|ui| {
                    ui.label("Seed");
                    if ui.add(egui::DragValue::new(&mut seed)).changed() {
                        state.world_doc.seed = seed;
                        state.world_dirty = true;
                    }
                });
                let (mut cx, mut cy) = (
                    state.world_doc.center_chunk[0],
                    state.world_doc.center_chunk[1],
                );
                ui.horizontal(|ui| {
                    ui.label("Center CH");
                    ui.add(egui::DragValue::new(&mut cx));
                    ui.add(egui::DragValue::new(&mut cy));
                });
                state.world_doc.center_chunk = [cx, cy];
                let mut radius = state.world_doc.materialized_radius.clamp(1, 4);
                ui.horizontal(|ui| {
                    ui.label("World radius");
                    ui.add(egui::DragValue::new(&mut radius).range(1..=4));
                    ui.small(format!("{}×{} chunks", radius * 2 + 1, radius * 2 + 1));
                });
                state.world_doc.materialized_radius = radius;
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Preview").clicked() {
                        match state
                            .world_doc
                            .preview_regenerate_with_tiled(
                                [cx, cy],
                                radius,
                                &state.asset_authority,
                                state.tiled_terrain.as_ref(),
                            )
                        {
                            Ok(preview) => {
                                state.message = format!(
                                    "Preview: {} chunks · {} generated cells change · {} object sets change · {} local overrides preserved",
                                    preview.chunk_count,
                                    preview.changed_cells,
                                    preview.changed_object_sets,
                                    preview.preserved_cell_overrides
                                );
                            }
                            Err(error) => state.message = format!("Preview refused: {error}"),
                        }
                    }
                    let label = if state.world_doc.chunks.is_empty() {
                        format!("Generate {}×{}", radius * 2 + 1, radius * 2 + 1)
                    } else {
                        format!("Regenerate {}×{}", radius * 2 + 1, radius * 2 + 1)
                    };
                    if ui.button(&label).clicked() {
                        checkpoint_world(state);
                        match materialize_world_with_current_authorities(state, [cx, cy], radius) {
                            Ok(()) => {
                                state.world_mode = true;
                                state.world_dirty = true;
                                state.message = format!(
                                    "Regenerated Generated World {}×{} around CH {cx},{cy}; authored terrain/behavior overrides preserved.",
                                    radius * 2 + 1,
                                    radius * 2 + 1
                                );
                            }
                            Err(error) => {
                                let _ = state.world_undo.pop();
                                state.message = format!("Worldgen refused: {error}");
                            }
                        }
                    }
                    if ui
                        .add_enabled(!state.world_doc.chunks.is_empty(), egui::Button::new("Open canvas"))
                        .clicked()
                    {
                        state.world_mode = true;
                        state.selection_active = false;
                    }
                    if ui
                        .add_enabled(state.world_dirty, egui::Button::new("Save"))
                        .clicked()
                    {
                        save_world_document(state);
                    }
                });
            });

        egui::CollapsingHeader::new("World plan")
            .id_salt("havenwild.generated_world.plan")
            .default_open(true)
            .show(ui, |ui| {
                ui.small("One authoritative Generated World. The old River scene remains only as an internal regression fixture.");
                for island in worldgen_plan::ISLANDS {
                    ui.label(format!("{} · {} · center {},{}", island.label, island.season, island.center[0], island.center[1]));
                }
                if let (Some(season), Some(biome), Some(water_context), Some(region)) = (
                    state.world_doc.generated_season(state.world_selected),
                    state.world_doc.generated_biome(state.world_selected),
                    state.world_doc.generated_water_context(state.world_selected),
                    state.world_doc.generated_region_id(state.world_selected),
                ) {
                    ui.separator();
                    let elevation = state.world_doc.generated_elevation(state.world_selected).unwrap_or(0);
                    let cliff_mask = state.world_doc.generated_cliff_mask(state.world_selected).unwrap_or(0);
                    ui.small(format!(
                        "Selected: {region} · {season} · {biome} · water {water_context} · elevation +{elevation} · cliff mask {cliff_mask:04b}"
                    ));
                }
                ui.small("M2D090 uses exact coordinate-equivalent seasonal source art. Shared cliff topology now comes from cardinal elevation-drop masks; source-art cliff rendering, collision and waterfall/cave attachment will consume that same mask with no parallel editor-only cliff model.");
            });

        ui.separator();
        ui.small("Terrain painting is controlled by the permanent Paint tool. Selecting Paint opens the material palette beside the left rail; world generation stays focused on seed/materialization settings.");

        if !compact {
            egui::CollapsingHeader::new("Authority / diagnostics")
                .id_salt("havenwild.world.generator.authority")
                .default_open(false)
                .show(ui, |ui| {
                    let pair_counts = state.asset_authority.summer_flatworld_pair_counts();
                    ui.label(format!(
                        "Summer source grammar: {}/81 · G↔D {} · G↔W {} · D↔W {} · {} source cells / {} groups",
                        state.asset_authority.summer_flatworld_corner_count(),
                        pair_counts[0],
                        pair_counts[1],
                        pair_counts[2],
                        state.asset_authority.summer_source_runtime_cell_count(),
                        state.asset_authority.summer_source_group_count()
                    ));
                    ui.label(format!(
                        "{} chunks · {} local overrides · {} reusable behavior profiles · {} object tombstones",
                        state.world_doc.chunks.len(),
                        state.world_doc.cell_overrides.len(),
                        state.world_doc.source_behavior_profiles.len(),
                        state.world_doc.suppressed_generated_objects.len()
                    ));
                    let object_count: usize = state.world_doc.chunks.iter().map(|chunk| chunk.generated_objects.len()).sum();
                    ui.label(format!("{} generated source-exact objects · elevations 0..2 active in the current archipelago prototype", object_count));
                    ui.small("Cliff/cave and waterfall showcase features are active. Continuous contour-cliff tiling is still screenshot-driven/evidence-gated so Havenwild does not guess source-atlas cliff roles.");
                });

            egui::CollapsingHeader::new("Materialized chunks · debug")
                .id_salt("havenwild.world.generator.chunks")
                .default_open(false)
                .show(ui, |ui| {
                    ui.small("Chunks are internal 32×32 streaming/render windows in one continuous world; they are not separate outdoor scenes.");
                    if state.world_doc.chunks.is_empty() {
                        ui.label("No chunks materialized.");
                    } else {
                        let rows = state
                            .world_doc
                            .chunks
                            .iter()
                            .map(|chunk| (chunk.coord, chunk.generated_objects.len()))
                            .collect::<Vec<_>>();
                        egui::ScrollArea::vertical()
                            .id_salt("havenwild.world.generator.chunk.debug.rows")
                            .max_height(220.0)
                            .show(ui, |ui| {
                                for (coord, count) in rows {
                                    let active = coord == state.world_doc.center_chunk;
                                    if ui
                                        .selectable_label(
                                            active,
                                            format!("CH {},{} · {} placements", coord[0], coord[1], count),
                                        )
                                        .clicked()
                                    {
                                        state.world_doc.center_chunk = coord;
                                        state.world_mode = true;
                                    }
                                }
                            });
                    }
                });
        }
    });
    state.world_generator_rect = response
        .as_ref()
        .map(|response| response.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if !open {
        state.editor_layout.world_generator_open = false;
        save_editor_layout(state);
    }
}

fn draw_app_launcher(ctx: &egui::Context, state: &mut Studio) {
    if !state.launcher_open {
        state.launcher_rect = egui::Rect::NOTHING;
        return;
    }
    let mut open = true;
    let response = egui::Window::new("Havenwild Tools")
        .id(egui::Id::new("havenwild.apps.launcher.window"))
        .title_bar(false)
        .open(&mut open)
        .anchor(egui::Align2::LEFT_BOTTOM, [58.0, -31.0])
        .default_size(egui::vec2(330.0, 430.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Havenwild Studio");
            ui.small("Tool applications open over the infinite canvas. Docking pins a panel to an edge; it never resizes the world canvas.");
            ui.separator();

            ui.strong("World authoring");
            if ui.button("Collision / Elevation").clicked() {
                state.editor_layout.structural_open = true;
                state.launcher_open = false;
                save_editor_layout(state);
            }
            if ui.button("Generated World").clicked() { state.editor_layout.world_generator_open = true; state.launcher_open = false; save_editor_layout(state); }

            ui.separator();
            ui.strong("Assets / terrain");
            if ui.button("Terrain rules / diagnostics").on_hover_text("Review live mappings; advanced history remains available without duplicating painting tools.").clicked() {
                open_mapper_tab(state, "patterns");
                state.launcher_open = false;
            }
            if ui.small_button("Open source browser in Library").clicked() {
                state.editor_layout.tool_context_open = true;
                state.editor_layout.authoring_library_tab = "Source".into();
                state.launcher_open = false;
                save_editor_layout(state);
            }
            egui::CollapsingHeader::new("Advanced evidence").default_open(false).show(ui, |ui| {
                if ui.small_button("Open evidence review").clicked() {
                    open_mapper_tab(state, "review");
                    state.launcher_open = false;
                }
            });
            ui.small("Atlas placement belongs to the left Library; selections and properties use the right Inspector.");

            ui.separator();
            ui.strong("Studios");
            ui.add_enabled(false, egui::Button::new("Pixel Studio · planned"))
                .on_hover_text("Planned as a full dockable/floating application panel with compact and full presentation; not exposed until real PNG editing is wired.");
            ui.add_enabled(false, egui::Button::new("Animation Studio · planned"))
                .on_hover_text("Planned as the same panel family as Pixel Studio with real timeline/events; not exposed as a fake editor.");

            ui.separator();
            ui.small("Ctrl+Space toggles this launcher.");
        });
    state.launcher_rect = response
        .as_ref()
        .map(|response| response.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if !open {
        state.launcher_open = false;
    }
}

fn hydrate_pending_source(
    mut state: ResMut<Studio>,
    assets: Res<AssetServer>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    if let Some(requested) = state.pending_source_sheet.take() {
        let requested_cell = state.pending_source_cell.take();
        match load_source_sheet_on_demand(&mut state, &assets, &mut user_textures, &requested) {
            Ok(index) => {
                state.active_sheet = index;
                state.selected_cell = requested_cell.unwrap_or([0, 0]);
                if requested_cell.is_none() {
                    state.selected_extent = [1, 1];
                }
                state.message = format!("Original ElizaWy source loaded: {requested}. No asset composition has been inferred.");
            }
            Err(error) => state.message = format!("Source selection refused: {error}"),
        }
    }
    // Regeneration can introduce references to previously unseen canonical
    // Structure/Objects/FX sheets. The startup-only hydration list misses them.
    // Scan once per world revision and hand Bevy at most four new PNG handles
    // per UI frame. This never promotes semantic roles or changes source bytes.
    if state.runtime_asset_sync_revision != Some(state.world_doc.revision) {
        state.runtime_asset_sync_queue =
            render_asset_sync::missing_world_source_paths(&state.world_doc, &state.sheet_lookup)
                .into();
        state.runtime_asset_sync_revision = Some(state.world_doc.revision);
        state.runtime_asset_sync_failures.clear();
    }
    for _ in 0..4 {
        let Some(path) = state.runtime_asset_sync_queue.pop_front() else {
            break;
        };
        if state.sheet_lookup.contains_key(&path) {
            continue;
        }
        if let Err(error) =
            load_source_sheet_on_demand(&mut state, &assets, &mut user_textures, &path)
        {
            bevy::log::warn!("World render dependency unavailable: {path}: {error}");
            state.runtime_asset_sync_failures.insert(path, error);
        }
    }
}

// ElizaWy PNG bytes are immutable and generally encode straight/unmultiplied RGBA.
// egui's Bevy user texture pass composites with premultiplied-alpha blending.
// Transparent RGB (for example the source tree sheet's [0,48,35,0] padding and
// [255,255,255,0] slot guide) otherwise contributes color despite zero alpha,
// yielding green rectangular silhouettes and bright vertical guide strokes.
// This conversion changes only Bevy's decoded *in-memory UI texture*; it never
// writes to the original, pinned source PNG or changes atlas crop coordinates.
fn premultiply_elizawy_sheet_color(color: Color) -> Color {
    let mut linear = color.to_linear();
    linear.red *= linear.alpha;
    linear.green *= linear.alpha;
    linear.blue *= linear.alpha;
    Color::LinearRgba(linear)
}

fn premultiply_elizawy_sheet_texture(image: &mut Image) -> Result<(), String> {
    for y in 0..image.height() {
        for x in 0..image.width() {
            let source = image.get_color_at(x, y).map_err(|error| {
                format!("ElizaWy decoded source cannot be inspected at {x},{y}: {error}")
            })?;
            // Opaque original artwork must stay byte-equivalent on the GPU too.
            if source.to_linear().alpha >= 1.0 {
                continue;
            }
            image
                .set_color_at(x, y, premultiply_elizawy_sheet_color(source))
                .map_err(|error| {
                    format!("Cannot premultiply ElizaWy UI texel at {x},{y}: {error}")
                })?;
        }
    }
    Ok(())
}

// Keep this before draw_ui, including for lazily-loaded sheets. Rehydrate after
// AssetServer hot reload, which restores the original straight-alpha pixels.
fn prepare_elizawy_egui_textures(
    mut state: ResMut<Studio>,
    mut images: ResMut<Assets<Image>>,
    mut events: MessageReader<AssetEvent<Image>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            for sheet in &mut state.sheets {
                if sheet.handle.id() == *id {
                    sheet.premultiplied_for_egui = false;
                }
            }
        }
    }
    // Bound first-load work. Initial/Source Browser sheets become visible as soon as
    // each has its correct premultiplied texture instead of blocking one long frame.
    let mut prepared_this_frame = 0;
    for sheet in &mut state.sheets {
        if sheet.premultiplied_for_egui {
            continue;
        }
        if prepared_this_frame >= 2 {
            break;
        }
        // Assets<Image>::get_mut returns AssetMut<Image>, not &mut Image.
        // The mutable binding is required to dereference the smart asset guard
        // into a true mutable image reference without bypassing change tracking.
        let Some(mut image) = images.get_mut(&sheet.handle) else {
            continue;
        };
        premultiply_elizawy_sheet_texture(&mut image).unwrap_or_else(|error| {
            panic!(
                "Cannot prepare source-exact egui texture {}: {error}",
                sheet.path
            )
        });
        sheet.premultiplied_for_egui = true;
        prepared_this_frame += 1;
    }
}

#[cfg(test)]
mod egui_source_alpha_tests {
    use super::*;

    #[test]
    fn transparent_green_and_white_guides_cannot_add_color_to_egui() {
        for color in [
            Color::srgba_u8(0, 48, 35, 0),
            Color::srgba_u8(255, 255, 255, 0),
        ] {
            let fixed = premultiply_elizawy_sheet_color(color).to_linear();
            assert_eq!([fixed.red, fixed.green, fixed.blue, fixed.alpha], [0.0; 4]);
        }
    }

    #[test]
    fn translucent_color_is_premultiplied_and_opaque_color_is_unchanged() {
        let source = Color::srgba_u8(120, 180, 70, 128);
        let straight = source.to_linear();
        let fixed = premultiply_elizawy_sheet_color(source).to_linear();
        assert!((fixed.alpha - straight.alpha).abs() < 0.0001);
        assert!((fixed.red - straight.red * straight.alpha).abs() < 0.0001);
        let opaque = Color::srgba_u8(70, 130, 50, 255).to_linear();
        let fixed_opaque =
            premultiply_elizawy_sheet_color(Color::srgba_u8(70, 130, 50, 255)).to_linear();
        assert!((opaque.green - fixed_opaque.green).abs() < 0.00001);
    }
}

fn draw_ui(mut contexts: EguiContexts, mut state: ResMut<Studio>, time: Res<Time>) -> Result {
    let dt = time.delta_secs();
    if state.editor_layout.timeline_playing && dt.is_finite() {
        state.animation_preview_ms =
            (state.animation_preview_ms + f64::from(dt.max(0.0)) * 1000.0).rem_euclid(60_000.0);
    }
    if dt > 0.001 {
        let fps = 1.0 / dt;
        state.display_fps = if state.display_fps <= 0.0 {
            fps
        } else {
            state.display_fps * 0.88 + fps * 0.12
        };
    }
    // One egui context belongs to the only Bevy primary camera. No offscreen pass.
    state.review_browser.poll();
    for sheet in &mut state.sheets {
        sheet.texture = if sheet.premultiplied_for_egui {
            contexts.image_id(&sheet.handle)
        } else {
            None
        };
    }
    let ctx = contexts.ctx_mut()?;
    if state.review_browser.loading {
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
    if !state.creator_visuals_applied {
        apply_creator_visuals(ctx, &state.theme);
        canvas_chrome::apply_theme(ctx, &state.editor_layout.editor_theme);
        state.creator_visuals_applied = true;
    }
    if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        if state.playtest.is_some() || state.world_playtest.is_some() {
            stop_playtest(&mut state);
        } else if state.world_template_placement.take().is_some() {
            state.message = "Reviewed object placement cancelled; no world changes made".into();
        } else if state.world_door_mapping.take().is_some() {
            state.message = "Doorway mapping cancelled; no world changes made".into();
        }
    }
    if let Some(playtest) = state.playtest.as_mut() {
        let direction = ctx.input(|input| {
            [
                (input.key_down(egui::Key::D) || input.key_down(egui::Key::ArrowRight)) as i32
                    as f32
                    - (input.key_down(egui::Key::A) || input.key_down(egui::Key::ArrowLeft)) as i32
                        as f32,
                (input.key_down(egui::Key::S) || input.key_down(egui::Key::ArrowDown)) as i32
                    as f32
                    - (input.key_down(egui::Key::W) || input.key_down(egui::Key::ArrowUp)) as i32
                        as f32,
            ]
        });
        playtest.step(direction, time.delta_secs());
        let position = playtest.position;
        let size = playtest.snapshot.size;
        state.pan = egui::vec2(
            (size[0] as f32 * 0.5 - position[0]) * 32.0 * state.zoom,
            (size[1] as f32 * 0.5 - position[1]) * 32.0 * state.zoom,
        );
        ctx.request_repaint();
    }
    if let Some(mut world_play) = state.world_playtest.take() {
        let direction = ctx.input(|input| {
            [
                (input.key_down(egui::Key::D) || input.key_down(egui::Key::ArrowRight)) as i32
                    as f32
                    - (input.key_down(egui::Key::A) || input.key_down(egui::Key::ArrowLeft)) as i32
                        as f32,
                (input.key_down(egui::Key::S) || input.key_down(egui::Key::ArrowDown)) as i32
                    as f32
                    - (input.key_down(egui::Key::W) || input.key_down(egui::Key::ArrowUp)) as i32
                        as f32,
            ]
        });
        if !ctx.egui_wants_keyboard_input() && ctx.input(|input| input.key_pressed(egui::Key::F)) {
            match world_play.interact(&state.asset_authority) {
                Ok(message) => state.message = message,
                Err(reason) => state.message = reason,
            }
        }
        if !ctx.egui_wants_keyboard_input() && ctx.input(|input| input.key_pressed(egui::Key::F8)) {
            match world_play.enter_test_cave() {
                Ok(message) => state.message = message,
                Err(reason) => state.message = reason,
            }
        }
        world_play.step(&state.asset_authority, direction, dt);
        if world_play.interior.is_none() {
            if let Some((origin, size)) = world_play.snapshot.materialized_bounds() {
                state.pan = egui::vec2(
                    (origin[0] as f32 + size[0] as f32 * 0.5 - world_play.position[0])
                        * 32.0
                        * state.zoom,
                    (origin[1] as f32 + size[1] as f32 * 0.5 - world_play.position[1])
                        * 32.0
                        * state.zoom,
                );
            }
        }
        state.world_playtest = Some(world_play);
        ctx.request_repaint();
    }
    if state.playtest.is_none()
        && state.world_playtest.is_none()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S))
    {
        save_scene(&mut state);
    }
    if state.playtest.is_none()
        && state.world_playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Z))
    {
        if let Some(edit) = state.room_authoring.as_mut() {
            edit.undo();
        } else {
            undo_scene_edit(&mut state);
        }
    }
    if state.playtest.is_none()
        && state.world_playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Y))
    {
        if let Some(edit) = state.room_authoring.as_mut() {
            edit.redo();
        } else {
            redo_scene_edit(&mut state);
        }
    }
    if state.playtest.is_none()
        && state.world_playtest.is_none()
        && ctx.input(|input| input.key_pressed(egui::Key::F11))
    {
        state.focus_mode = !state.focus_mode;
        if state.focus_mode {
            state.tool = CanvasTool::Select;
            state.launcher_open = false;
        }
    }
    if !ctx.egui_wants_keyboard_input() && ctx.input(|input| input.key_pressed(egui::Key::F6)) {
        if state.playtest.is_some() || state.world_playtest.is_some() {
            stop_playtest(&mut state);
        } else {
            start_playtest(&mut state);
        }
    }
    if state.playtest.is_none()
        && state.world_playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Space))
    {
        state.launcher_open = !state.launcher_open;
    }
    if state.playtest.is_none()
        && state.world_playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
    {
        let brush_shortcut = ctx.input(|input| {
            if input.modifiers.ctrl && !input.modifiers.shift && !input.modifiers.alt {
                if input.key_pressed(egui::Key::Num1) {
                    Some(BrushMode::Pixel)
                } else if input.key_pressed(egui::Key::Num2) {
                    Some(BrushMode::Tile)
                } else if input.key_pressed(egui::Key::Num3) {
                    Some(BrushMode::Terrain)
                } else {
                    None
                }
            } else {
                None
            }
        });
        if let Some(mode) = brush_shortcut {
            choose_brush(&mut state, mode);
        }
        if ctx.input(|input| {
            input.modifiers.ctrl
                && !input.modifiers.alt
                && (input.key_pressed(egui::Key::Equals) || input.key_pressed(egui::Key::Plus))
        }) {
            let canvas = state.canvas_rect;
            if canvas.width() > 0.0 {
                zoom_canvas_by(&mut state, canvas.center(), canvas, 1.25);
            }
        } else if ctx.input(|input| {
            input.modifiers.ctrl && !input.modifiers.alt && input.key_pressed(egui::Key::Minus)
        }) {
            let canvas = state.canvas_rect;
            if canvas.width() > 0.0 {
                zoom_canvas_by(&mut state, canvas.center(), canvas, 1.0 / 1.25);
            }
        }
        let requested_tool = ctx.input(|input| {
            if input.modifiers.ctrl || input.modifiers.alt {
                None
            } else if input.key_pressed(egui::Key::Q) {
                Some(CanvasTool::Select)
            } else if input.key_pressed(egui::Key::B) {
                Some(CanvasTool::Paint)
            } else if input.key_pressed(egui::Key::E) {
                Some(CanvasTool::Erase)
            } else if input.key_pressed(egui::Key::P) {
                Some(CanvasTool::Sample)
            } else {
                None
            }
        });
        if let Some(tool) = requested_tool {
            state.tool = tool;
            if state.world_mode && tool == CanvasTool::Paint {
                state.active_layer = world_terrain_role_layer(state.world_terrain_brush.role());
            }
            if tool == CanvasTool::Paint {
                state.editor_layout.tool_context_open = true;
            }
            state.message = format!(
                "{} selected from keyboard.",
                if tool == CanvasTool::Paint {
                    state.brush_mode.label()
                } else {
                    tool.label()
                }
            );
        }
    }

    // egui 0.36 panels are children of a root Ui rather than Context-bound
    // containers. Keep one viewport Ui for all top-level docked surfaces so the
    // product/action/status bars, mapper docks, and center canvas reserve space
    // from the same authoritative rectangle.
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("havenwild.viewport.root"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    // Native Windows owns the title bar and its system controls.
    // Do not render a second non-functional ForgeGUI title bar inside the client area.

    egui::Panel::top("havenwild.action.chrome").show(&mut viewport_ui, |ui| {
        ui.horizontal(|ui| {
            if state.playtest.is_some() || state.world_playtest.is_some() {
                ui.small("PLAYING | source-backed snapshot | W/A/S/D | Esc to stop");
            } else {
            ui.menu_button("File", |ui| {
                if ui.button(if state.world_mode { "Save World     Ctrl+S" } else { "Save Scene     Ctrl+S" }).clicked() { save_scene(&mut state); }
                if ui.button("Save All (including advanced drafts)").clicked() { save_all_authored_state(&mut state); }
                ui.label("Original artwork remains read-only.");
            });
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(if state.world_mode { !state.world_undo.is_empty() } else { state.layered_history.can_undo() }, egui::Button::new("Undo Edit    Ctrl+Z")).clicked() { undo_scene_edit(&mut state); }
                if ui.add_enabled(if state.world_mode { !state.world_redo.is_empty() } else { state.layered_history.can_redo() }, egui::Button::new("Redo Edit    Ctrl+Y")).clicked() { redo_scene_edit(&mut state); }
                if ui.add_enabled(state.selection_active && !state.world_mode, egui::Button::new("Remove selected object / erase tile     Delete")).clicked() {
                    let pos = state.selected_world;
                    if !hide_selected_visual_sample(&mut state, pos) { erase_source(&mut state, pos); }
                }
                if state.world_mode { ui.small("World Delete suppresses the selected worldgen placement directly on the canvas."); }
            });
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut state.editor_layout.rulers_visible, "Show canvas rulers (tile coordinates)").changed() { save_editor_layout(&mut state); }
                ui.menu_button("Canvas grids", |ui| {
                    if ui.checkbox(&mut state.editor_layout.chunk_grid_visible, "World chunk outlines / coordinates").changed() { save_editor_layout(&mut state); }
                    if ui.checkbox(&mut state.editor_layout.tile_grid_visible, "32px tile grid over authored scene").changed() { save_editor_layout(&mut state); }
                    ui.separator();
                    ui.small(if state.world_mode { "Grid follows the materialized world region." } else { "Summer scene grid overlay; use World Generator for materialized chunks." });
                    for side in world_chunks::ALLOWED_CHUNK_TILES {
                        if ui.selectable_value(&mut state.editor_layout.chunk_size_tiles, side, format!("{side} x {side} logical tiles")).changed() {
                            save_editor_layout(&mut state);
                        }
                    }
                });
                if ui.checkbox(&mut state.editor_layout.mapping_status_visible, "Show reviewed-source markers (when present)").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.editor_layout.inspect_alpha, "Inspect selected tile transparency").changed() { save_editor_layout(&mut state); }
                if ui.button("World Properties in Inspector").clicked() {
                    state.editor_layout.workspace.right.open = true;
                    state.editor_layout.inspector_tab = "World".into();
                    state.editor_layout.structural_open = true;
                    save_editor_layout(&mut state);
                }
                if ui.checkbox(&mut state.editor_layout.workspace.right.open, "Right Inspector / Asset Browser dock").changed() {
                    save_editor_layout(&mut state);
                }
                if ui.checkbox(&mut state.editor_layout.tool_context_open, "Left three-section tooling dock").changed() {
                    save_editor_layout(&mut state);
                }
                ui.small("Panels can be collapsed independently; drag the Inspector edge to resize it.");
                if ui.checkbox(&mut state.focus_mode, "Focus canvas     F11").changed() && state.focus_mode {
                    state.tool = CanvasTool::Select;
                }
            });
            if !state.world_mode {
                ui.menu_button("Scene", |ui| {
                    if ui.button("Save scene corrections").clicked() { save_scene(&mut state); }
                    if ui.button("Fit Summer canvas").clicked() {
                        let size = state.scene.size;
                        focus_mapping_zone(&mut state, [0, 0], size);
                    }
                    ui.small("Legacy Summer certification scene. Generated World is the primary authoring surface.");
                    ui.small(format!(
                        "Summer source coverage: {} original cells used here / {} recovered globally · {} source-bound visual samples.",
                        state.mapping_lab.summer_source_cells_actually_used,
                        state.mapping_lab.summer_recovered_source_cells,
                        state.mapping_lab.visual_samples.len(),
                    ));
                    ui.small("The 16-mask Grass/Void recipe is a legacy diagnostic lab, not production terrain authority.");
                    ui.small(format!("{} manually placed source-bound instances in this regression scene.", state.layered_scene.objects.len()));
                });
            }
            ui.menu_button("Tools", |ui| {
                if ui
                    .button("Havenwild Tool Launcher    Ctrl+Space")
                    .on_hover_text("Open the single application launcher. Terrain Studio, Generated World and World Properties are not duplicated in this menu.")
                    .clicked()
                {
                    state.launcher_open = !state.launcher_open;
                }
                ui.small("Editor applications are launched from one authoritative surface.");
            });
            ui.menu_button("Theme", |ui| {
                for preset in canvas_chrome::THEME_NAMES {
                    if ui.selectable_label(state.editor_layout.editor_theme == preset, preset).clicked() {
                        state.editor_layout.editor_theme = preset.to_owned();
                        state.creator_visuals_applied = false;
                        save_editor_layout(&mut state);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("Active Summer visual study uses original PNGs unchanged over a pinned River-ground fixture; never replace source pixels.");
                ui.label("Select drags complete objects; save edits in the protected Summer draft. Earlier River and mapping drafts remain untouched.");
                ui.label("Mapped source cells are evidence, NOT automatically certified DG recipes.");
                ui.label("Select is the safe default: clicking does not paint.");
                ui.label("Paint opens the semantic material chooser at left; source-region placement in the legacy scene follows the active canvas layer.");
                ui.label("Select and drag objects directly in scene or generated world. Layers live over the canvas left edge; inspector is docked right. Delete suppresses a generated object without erasing ground.");
                ui.label("Eyedropper samples terrain/source meaning. Middle/right drag pans.");
                ui.label("Worldgen composes only canonical ElizaWy source regions. Regeneration preserves authored world corrections and tombstones; no image asset is generated.");
            });
            }
            ui.separator();
            if state.world_mode {
                ui.small(format!("Studio {} · World {}{} · {} chunks · {}", env!("CARGO_PKG_VERSION"), state.world_doc.world_id, if state.world_dirty { " *" } else { "" }, state.world_doc.chunks.len(), state.tool.label()));
            } else {
                ui.small(format!("{}{} · {}", state.layered_scene.name, if state.layered_history.is_dirty() || state.scene_needs_initial_save { " *" } else { "" }, if state.playtest.is_some() { "PIE / source-snapshot" } else { state.tool.label() }));
            }
            if state.focus_mode && ui.small_button("Exit Focus").clicked() { state.focus_mode = false; }
        });
    });

    if !state.focus_mode {
        egui::Panel::top("havenwild.command.toolbar").show(&mut viewport_ui, |ui| {
            if state.room_authoring.is_some() {
                ui.horizontal_wrapped(|ui| {
                    ui.strong("House Room");
                    if ui.button("Save Room · Ctrl+S").clicked() { save_house_room_editor(&mut state); }
                    let dirty = state.room_authoring.as_ref().is_some_and(|edit| edit.dirty);
                    if ui.button(if dirty { "Save & Return to World" } else { "Return to World" }).clicked() {
                        if !dirty || save_house_room_editor(&mut state) {
                            state.room_authoring = None;
                            state.room_authoring_object = None;
                        }
                    }
                    if ui.button("Undo").clicked() {
                        if let Some(edit) = state.room_authoring.as_mut() { edit.undo(); }
                    }
                    if ui.button("Redo").clicked() {
                        if let Some(edit) = state.room_authoring.as_mut() { edit.redo(); }
                    }
                    ui.label("2×2 chunks · authored floors · automatic wall topology");
                });
                return;
            }
            ui.horizontal_wrapped(|ui| {
                // Select/Paint/Erase/Pick live only on the permanent left tool rail.
                // Keep the top command bar for document/history/view actions so the
                // same tools are not presented twice.
                if ui
                    .add_enabled(
                        if state.world_mode { !state.world_undo.is_empty() } else { state.layered_history.can_undo() },
                        egui::Button::new(command_spec(StudioCommand::Undo).label),
                    )
                    .on_hover_text(command_tooltip(StudioCommand::Undo))
                    .clicked()
                {
                    undo_scene_edit(&mut state);
                }
                if ui
                    .add_enabled(
                        if state.world_mode { !state.world_redo.is_empty() } else { state.layered_history.can_redo() },
                        egui::Button::new(command_spec(StudioCommand::Redo).label),
                    )
                    .on_hover_text(command_tooltip(StudioCommand::Redo))
                    .clicked()
                {
                    redo_scene_edit(&mut state);
                }
                if ui
                    .button("Save")
                    .on_hover_text(command_tooltip(StudioCommand::SaveScene))
                    .clicked()
                {
                    save_scene(&mut state);
                }
                ui.separator();
                if ui
                    .button(if state.playtest.is_some() || state.world_playtest.is_some() { "Stop PIE" } else { "Play in Editor" })
                    .on_hover_text(if state.world_mode { "Start/stop World PIE with LPC character, reach an authored house door using F, and walk into/out of the saved room. Cave test uses F8.".to_owned() } else { command_tooltip(StudioCommand::TogglePlay) })
                    .clicked()
                {
                    if state.playtest.is_some() || state.world_playtest.is_some() { stop_playtest(&mut state); } else { start_playtest(&mut state); }
                }
                ui.separator();
            });
        });
    }

    if !state.focus_mode {
        egui::Panel::bottom("havenwild.status.chrome").show(&mut viewport_ui, |ui| {
            ui.horizontal(|ui| {
                if state.room_authoring.is_some() {
                    let edit = state.room_authoring.as_ref().expect("guarded room edit");
                    ui.label(if edit.dirty {
                        "House Room *"
                    } else {
                        "House Room saved"
                    });
                    ui.separator();
                    ui.small("Floor/wall topology · project-owned document · source art read-only");
                } else if state.world_mode {
                    ui.label(if state.world_dirty {
                        "World *"
                    } else {
                        "World saved"
                    });
                    ui.separator();
                    let (chunk, local) = world_chunks::chunk_and_local(
                        state.world_selected,
                        state.world_doc.chunk_side,
                    );
                    ui.small(format!(
                        "World cell {},{} · CH {},{} / local {},{} · {}",
                        state.world_selected[0],
                        state.world_selected[1],
                        chunk[0],
                        chunk[1],
                        local[0],
                        local[1],
                        active_tool_label(&state)
                    ));
                } else {
                    ui.label(
                        if state.layered_history.is_dirty() || state.scene_needs_initial_save {
                            "Summer scene *"
                        } else {
                            "Scene saved"
                        },
                    );
                    ui.separator();
                    let (chunk, local) = world_chunks::chunk_and_local(
                        [
                            state.selected_world[0] as i64,
                            state.selected_world[1] as i64,
                        ],
                        state.editor_layout.chunk_size_tiles,
                    );
                    ui.small(format!(
                        "Cell {},{} · CH {},{} / local {},{} · {}",
                        state.selected_world[0],
                        state.selected_world[1],
                        chunk[0],
                        chunk[1],
                        local[0],
                        local[1],
                        state.tool.label()
                    ));
                }
                ui.separator();
                ui.small(format!("{:.0} FPS", state.display_fps));
                ui.separator();
                ui.small(&state.message);
                ui.separator();
                if ui
                    .selectable_label(
                        state.editor_layout.metrics_drawer_open,
                        if state.editor_layout.metrics_drawer_open {
                            "Metrics ▾"
                        } else {
                            "Metrics ▴"
                        },
                    )
                    .clicked()
                {
                    state.editor_layout.metrics_drawer_open =
                        !state.editor_layout.metrics_drawer_open;
                    save_editor_layout(&mut state);
                }
            });
        });
    }

    // Canvas-first: 62px tool rail, optional active-tool settings, one inspector.
    // Layers are rendered as icon-only controls on the canvas itself.
    if !state.focus_mode
        && state.playtest.is_none()
        && state.world_playtest.is_none()
        && state.room_authoring.is_some()
    {
        draw_library_dock(&mut viewport_ui, &mut state);
        draw_floating_library(ctx, &mut state);
    } else if !state.focus_mode && state.playtest.is_none() && state.world_playtest.is_none() {
        draw_inspector_rail(&mut viewport_ui, &mut state);
        draw_library_dock(&mut viewport_ui, &mut state);
        draw_tool_context(&mut viewport_ui, &mut state);
        draw_floating_library(ctx, &mut state);
        // Terrain materials now live in the left tool context, not a floating duplicate.
        state.structural_rect = egui::Rect::NOTHING;
        draw_mapper_surface(&mut viewport_ui, &mut state);
        draw_world_generator_panel(ctx, &mut state);
        draw_app_launcher(ctx, &mut state);
    } else {
        state.atlas_rect = egui::Rect::NOTHING;
        state.structural_rect = egui::Rect::NOTHING;
        state.world_generator_rect = egui::Rect::NOTHING;
        state.launcher_rect = egui::Rect::NOTHING;
    }

    // Both bottom surfaces are reserved only from the canvas region between the
    // left and right docks, never across the entire application viewport.
    if !state.focus_mode {
        draw_console_dock(&mut viewport_ui, &mut state);
        if state.playtest.is_none() && state.world_playtest.is_none() {
            draw_animation_timeline(&mut viewport_ui, &mut state);
        }
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(&mut viewport_ui, |ui| {
            draw_world(ui, &mut state);
            // Native canvas HUD: no independent dock, no foreground Area that
            // can paint over Studio dialogs and no extra canvas-width allocation.
            if !state.focus_mode
                && state.playtest.is_none()
                && state.world_playtest.is_none()
                && state.room_authoring.is_none()
            {
                draw_layer_rail(ui, &mut state);
                draw_canvas_view_controls(ui, &mut state);
                draw_canvas_animation_toggle(ui, &mut state);
            }
        });
    // Properties belong to the permanent right Inspector dock. No duplicate popup.

    // No egui window-drag/resize overlay: native Windows owns these interactions.

    if state.playtest.is_none() && state.world_playtest.is_none() {
        if let Some((sheet, source)) = state.dragging {
            if let Some(pointer) = ctx.pointer_hover_pos() {
                let preview_min = if state.world_mode {
                    state
                        .world_doc
                        .materialized_bounds()
                        .and_then(|(origin, _)| {
                            world_pointer_from_screen(&state, pointer).map(|world| {
                                let px = 32.0 * state.zoom;
                                state.scene_rect.min
                                    + egui::vec2(
                                        (world[0] - origin[0]) as f32 * px,
                                        (world[1] - origin[1]) as f32 * px,
                                    )
                            })
                        })
                } else {
                    world_at(&state, pointer).map(|pos| {
                        let px = 32.0 * state.zoom;
                        state.scene_rect.min + egui::vec2(pos[0] as f32 * px, pos[1] as f32 * px)
                    })
                };
                if let Some(min) = preview_min {
                    let target = egui::Rect::from_min_size(
                        min,
                        egui::vec2(source[2] as f32 * state.zoom, source[3] as f32 * state.zoom),
                    );
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Tooltip,
                        egui::Id::new("atlas_drag_preview"),
                    ));
                    if let Some(texture) = state.sheets[sheet].texture {
                        painter.image(
                            texture,
                            target,
                            source_uv(source, state.sheets[sheet].size),
                            egui::Color32::from_white_alpha(175),
                        );
                    }
                    painter.rect_stroke(
                        target,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN),
                        egui::StrokeKind::Inside,
                    );
                }
            }
            if ctx.input(|input| input.pointer.primary_released()) {
                let pointer = ctx.pointer_hover_pos();
                if state.world_mode {
                    if let Some(world) =
                        pointer.and_then(|pointer| world_pointer_from_screen(&state, pointer))
                    {
                        let binding = SourceBinding {
                            source_asset: state.sheets[sheet].path.clone(),
                            source_rect: source,
                        };
                        state.tool = CanvasTool::Paint;
                        state.brush_mode = BrushMode::Tile;
                        state.editor_layout.brush_mode = "tile".into();
                        let local_only = ctx.input(|input| input.modifiers.shift);
                        if !binding.source_asset.starts_with("Terrain/") {
                            state.message =
                                "World drag-drop accepts Terrain source regions only.".into();
                        } else if !state.asset_authority.contains_binding(&binding) {
                            state.message = "World drag-drop refused: source region is not canonical ElizaWy authority.".into();
                        } else {
                            world_stroke_begin(&mut state, world, Some(binding), false, local_only);
                            world_stroke_finish(&mut state);
                        }
                    } else {
                        state.message =
                            "Canceled source drag outside the materialized World canvas".into();
                    }
                } else if let Some(pos) = pointer.and_then(|pointer| world_at(&state, pointer)) {
                    state.tool = CanvasTool::Paint;
                    state.brush_mode = BrushMode::Tile;
                    state.editor_layout.brush_mode = "tile".into();
                    apply_source(&mut state, pos, sheet, source);
                } else {
                    state.message = "Canceled drag outside Game Canvas".into();
                }
                state.dragging = None;
            }
        }
    }
    Ok(())
}

fn runtime_window_resizable(backend: &str, allow_dx12_resize: bool) -> bool {
    !backend.eq_ignore_ascii_case("dx12") || allow_dx12_resize
}

fn main() -> bevy::app::AppExit {
    // Keep native Windows title-bar/system controls authoritative; custom in-client chrome stays retired.
    let native_frame = true;
    let backend = std::env::var("WGPU_BACKEND").unwrap_or_else(|_| "auto".into());
    let allow_dx12_resize = std::env::var("HAVENWILD_DX12_ALLOW_RESIZE")
        .map(|value| value == "1")
        .unwrap_or(false);
    let resizable = runtime_window_resizable(&backend, allow_dx12_resize);
    let dx12_safe = backend.eq_ignore_ascii_case("dx12") && !allow_dx12_resize;
    let title_suffix = if dx12_safe { " [DX12 safe window]" } else { "" };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    file_path: project_root::resolve()
                        .join("assets/elizawy")
                        .to_string_lossy()
                        .into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!(
                            "Havenwild — Bevy Studio v{}{}",
                            env!("CARGO_PKG_VERSION"),
                            title_suffix
                        ),
                        decorations: native_frame,
                        resolution: (1440, 900).into(),
                        resizable,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.07, 0.10, 0.09)))
        .add_systems(Startup, startup)
        .add_systems(
            EguiPrimaryContextPass,
            (
                hydrate_pending_source,
                prepare_elizawy_egui_textures,
                draw_ui,
            )
                .chain(),
        )
        .run()
}

#[cfg(test)]
mod runtime_window_policy_tests {
    use super::runtime_window_resizable;

    #[test]
    fn vulkan_remains_resizable() {
        assert!(runtime_window_resizable("vulkan", false));
    }

    #[test]
    fn dx12_is_fixed_size_unless_explicitly_opted_into_resize_diagnostics() {
        assert!(!runtime_window_resizable("dx12", false));
        assert!(runtime_window_resizable("dx12", true));
    }
}

#[cfg(test)]
mod source_exact_render_tests {
    use super::snapped_scene_edge;
    #[test]
    fn multi_cell_source_region_right_edge_matches_following_tile_left_edge() {
        for ppp in [1.0, 1.25, 1.5, 2.0] {
            for zoom in [0.375, 0.7, 1.0, 1.25, 2.0] {
                let px = 32.0 * zoom;
                let region_right = snapped_scene_edge(43.375, 7.0 + 3.0, px, ppp);
                let next_cell_left = snapped_scene_edge(43.375, 10.0, px, ppp);
                assert_eq!(region_right, next_cell_left);
            }
        }
    }
    #[test]
    fn adjacent_cells_share_the_exact_same_physical_pixel_edge() {
        for ppp in [1.0, 1.25, 1.5, 2.0] {
            for zoom in [0.375, 0.7, 1.0, 1.25, 2.0] {
                let px = 32.0 * zoom;
                for x in 0..40 {
                    let right = snapped_scene_edge(43.375, x as f32 + 1.0, px, ppp);
                    let next_left = snapped_scene_edge(43.375, (x + 1) as f32, px, ppp);
                    assert_eq!(right, next_left);
                    assert!(((right * ppp) - (right * ppp).round()).abs() < 0.0001);
                }
            }
        }
    }
}
