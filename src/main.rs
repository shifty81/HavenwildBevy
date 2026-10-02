//! Fresh independent ElizaWy-first world canvas / mapping seed.
//! One primary Bevy camera and one original-pixel atlas renderer. No PCC gates,
//! historical Bevy validation probes, alternative world renderers, or game claims.
mod asset_authority;
mod atomic_file;
mod canvas_rulers;
mod document;
mod editor_commands;
mod editor_layout;
mod elizawy_review;
mod historical_evidence;
mod playtest;
mod project_root;
mod scene_v2;
mod semantic_lab;
mod source_catalog;
mod summer_seed;
mod terrain;
mod terrain_mapper;
mod terrain_resolver;
mod world_chunks;
mod world_doc;

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
use editor_layout::EditorLayout;
use elizawy_review::{ReviewBrowser, ReviewCandidate};
use forge_gui_chrome::{apply_creator_visuals, SurfaceDock, WindowSnapState};
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
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use terrain_mapper::{mask_meaning, TerrainMapper, TerrainRecipeSource, TerrainRecipeState};
use terrain_resolver::{all_vertex_coords, dirty_vertices_for_cell, resolve_semantic_lab};
use world_doc::{TraversalMode, WorldDocument};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldTerrainBrush {
    Grass,
    DirtBank,
    Water,
}

impl WorldTerrainBrush {
    fn role(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::DirtBank => "MudBank",
            Self::Water => "RiverWater",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Self::Grass => "GRS",
            Self::DirtBank => "DIR",
            Self::Water => "WTR",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::DirtBank => "Dirt / river bank",
            Self::Water => "River water",
        }
    }

    fn from_role(role: &str) -> Option<Self> {
        match role {
            "Grass" => Some(Self::Grass),
            "MudBank" => Some(Self::DirtBank),
            "RiverWater" => Some(Self::Water),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorldBehaviorScope {
    SourceProfile,
    LocalArea,
}

impl WorldBehaviorScope {
    fn label(self) -> &'static str {
        match self {
            Self::SourceProfile => "Reusable profile",
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
    playtest_layered: Option<SceneV2>,
    playtest: Option<PlaySession>,
    saved_editor_view: Option<(f32, egui::Vec2)>,
    mapping_lab: MappingLabIndex,
    sheets: Vec<Sheet>,
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
    source_catalog: SourceCatalog,
    asset_authority: AssetAuthority,
    world_doc: WorldDocument,
    world_path: PathBuf,
    world_mode: bool,
    world_dirty: bool,
    world_selected: [i64; 2],
    world_selected_object: Option<String>,
    world_undo: Vec<WorldDocument>,
    world_redo: Vec<WorldDocument>,
    water_animation_ms: u64,
    // A world paint stroke is authored locally while the pointer is down, then
    // topology-safe corrections are promoted to generator knowledge once on release.
    // This keeps Paint responsive and avoids regenerating every materialized chunk
    // for every mouse-move event.
    world_stroke_cells: Vec<[i64; 2]>,
    world_stroke_seen: BTreeSet<[i64; 2]>,
    world_stroke_binding: Option<SourceBinding>,
    world_stroke_role: Option<String>,
    world_terrain_brush: WorldTerrainBrush,
    world_stroke_erase: bool,
    world_stroke_local_only: bool,
    asset_authority_filter: String,
    pending_source_sheet: Option<String>,
    source_catalog_images: usize,
    source_catalog_terrain: usize,
    semantic_lab: SemanticTerrainLab,
    semantic_lab_path: PathBuf,
    semantic_lab_seed_path: PathBuf,
    semantic_lab_dirty: bool,
    semantic_undo: Vec<SemanticHistoryEntry>,
    semantic_redo: Vec<SemanticHistoryEntry>,
    last_dirty_vertices: Vec<terrain::TerrainVertexCoord>,
    selected_lab_vertex: [usize; 2],
    atlas_open: bool,
    atlas_zoom: f32,
    evidence_open: bool,
    launcher_open: bool,
    collision_world_pixel_edit: bool,
    collision_brush_blocked: bool,
    collision_brush_radius: usize,
    collision_stroke_active: bool,
    world_behavior_scope: WorldBehaviorScope,
    tool: CanvasTool,
    inspect_alpha: bool,
    focus_mode: bool,
    properties_open: bool,
    zoom: f32,
    pan: egui::Vec2,
    canvas_rect: egui::Rect,
    scene_rect: egui::Rect,
    canvas_controls_rect: egui::Rect,
    atlas_rect: egui::Rect,
    evidence_rect: egui::Rect,
    layers_rect: egui::Rect,
    structural_rect: egui::Rect,
    world_generator_rect: egui::Rect,
    chunk_manager_rect: egui::Rect,
    asset_authority_rect: egui::Rect,
    launcher_rect: egui::Rect,
    dragging: Option<(usize, [u32; 4])>,
    undo: Vec<Scene>,
    redo: Vec<Scene>,
    dirty: bool,
    message: String,
    theme: ForgeTheme,
    creator_visuals_applied: bool,
    native_frame: bool,
    snap: WindowSnapState,
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
    });
    Ok(state.sheets.len() - 1)
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

fn startup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut textures: ResMut<EguiUserTextures>,
    mut settings: ResMut<EguiGlobalSettings>,
) {
    settings.auto_create_primary_context = false;
    commands.spawn((Camera2d, PrimaryEguiContext));
    let root = project_root::resolve();
    // Native Windows decorations are mandatory until custom chrome has OS-level tests.
    let native_frame = true;
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
    bevy::log::info!(
        "Terrain authoring startup: {} / {} Terrain manifest sheets hydrated; {} total image entries; semantic resolver lab {}/16 masks",
        sheets.len(),
        source_catalog_terrain,
        source_catalog_images,
        resolver_coverage
    );
    // The pinned source-authored River supplies immutable ground UNDER the opening Summer
    // object composition. Guessed M2D02-D visual samples stay retired.
    let initial_zoom = 0.70;
    let initial_pan = egui::Vec2::ZERO;
    let summer_object_count = layered_scene.objects.len();
    commands.insert_resource(Studio {
        scene,
        layered_scene,
        layered_history: SceneHistory::default(),
        layered_path,
        scene_needs_initial_save,
        legacy_base_path,
        active_layer: LayerId::Objects,
        playtest_layered: None,
        playtest: None,
        saved_editor_view: None,
        mapping_lab,
        sheets,
        active_sheet: 0,
        selected_cell: [0, 0],
        selected_extent: [1, 1],
        selected_world: [0, 0],
        selection_active: false,
        source_filter: String::new(),
        source_family_filter: "All".into(),
        source_catalog,
        asset_authority,
        world_doc,
        world_path,
        world_mode: false,
        world_dirty: repaired_world_rules > 0 || quarantined_world_objects > 0,
        world_selected: [0, 0],
        world_selected_object: None,
        world_undo: Vec::new(),
        world_redo: Vec::new(),
        water_animation_ms: 0,
        world_stroke_cells: Vec::new(),
        world_stroke_seen: BTreeSet::new(),
        world_stroke_binding: None,
        world_stroke_role: None,
        world_terrain_brush: WorldTerrainBrush::Grass,
        world_stroke_erase: false,
        world_stroke_local_only: false,
        asset_authority_filter: String::new(),
        pending_source_sheet: None,
        source_catalog_images,
        source_catalog_terrain,
        semantic_lab,
        semantic_lab_path,
        semantic_lab_seed_path,
        semantic_lab_dirty: false,
        semantic_undo: Vec::new(),
        semantic_redo: Vec::new(),
        last_dirty_vertices: Vec::new(),
        selected_lab_vertex: [0, 0],
        atlas_open: editor_layout.terrain_mapper.visible,
        atlas_zoom: 0.75,
        evidence_open: false,
        launcher_open: false,
        collision_world_pixel_edit: false,
        collision_brush_blocked: true,
        collision_brush_radius: 1,
        collision_stroke_active: false,
        world_behavior_scope: WorldBehaviorScope::LocalArea,
        tool: CanvasTool::Select,
        inspect_alpha: editor_layout.inspect_alpha,
        focus_mode: false,
        properties_open: false,
        zoom: initial_zoom,
        pan: initial_pan,
        canvas_rect: egui::Rect::NOTHING,
        scene_rect: egui::Rect::NOTHING,
        canvas_controls_rect: egui::Rect::NOTHING,
        atlas_rect: egui::Rect::NOTHING,
        evidence_rect: egui::Rect::NOTHING,
        layers_rect: egui::Rect::NOTHING,
        structural_rect: egui::Rect::NOTHING,
        world_generator_rect: egui::Rect::NOTHING,
        chunk_manager_rect: egui::Rect::NOTHING,
        asset_authority_rect: egui::Rect::NOTHING,
        launcher_rect: egui::Rect::NOTHING,
        dragging: None,
        selected_object_id: None,
        dragging_object: None,
        drag_preview_origin: None,
        undo: Vec::new(),
        redo: Vec::new(),
        dirty: false,
        message: if repaired_world_rules > 0 || quarantined_world_objects > 0 {
            format!(
                "World safety repair: {repaired_world_rules} unsafe learned rule(s) repaired; {quarantined_world_objects} uncertified large generated tree/rock object(s) quarantined. Build the Summer autotile map in PCC, regenerate, then Save World."
            )
        } else {
            format!("Summer composition ready: {} individually placed original-source instances. GRS/DIR/WTR use four-corner source-only autotiling; build/update its local map from PCC Terrain when coverage changes.", summer_object_count)
        },
        theme: ForgeTheme::from_preset(ForgeThemePreset::MidnightMint),
        creator_visuals_applied: false,
        native_frame,
        snap: WindowSnapState::default(),
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

fn world_at(state: &Studio, pointer: egui::Pos2) -> Option<[usize; 2]> {
    if !state.canvas_rect.contains(pointer)
        || !state.scene_rect.contains(pointer)
        || canvas_rulers::blocks_scene_edit(
            state.canvas_rect,
            pointer,
            canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode),
        )
        || state.canvas_controls_rect.contains(pointer)
        || (state.atlas_open && state.atlas_rect.contains(pointer))
        || (state.evidence_open && state.evidence_rect.contains(pointer))
        || (state.editor_layout.layers_open && state.layers_rect.contains(pointer))
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
    let Some(object) = state.layered_scene.object_at(pos) else {
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
    state.message = format!("Selected {label} on {layer:?}; drag with SEL or double-click for optional properties. Bounds are visual, not collision.");
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
            state.properties_open = false;
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
                state.message = format!("Placed one source-exact complete object on {:?} at foot ({}, {}). Use SEL to drag, Scene panel coordinates to relocate, or Delete to remove. No asset boundary or collision certified.", state.active_layer, pos[0], pos[1]);
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

fn start_playtest(state: &mut Studio) {
    if state.playtest.is_some() {
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
    if !state.inspect_alpha || state.playtest.is_some() {
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
        state.atlas_open = true;
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
        for edit in &layer.edits {
            let Some(source) = &edit.source else {
                continue;
            };
            let Some(sheet) = state
                .sheets
                .iter()
                .find(|sheet| sheet.path == source.source_asset)
            else {
                continue;
            };
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
                let Some(sheet) = state
                    .sheets
                    .iter()
                    .find(|sheet| sheet.path == source.source_asset)
                else {
                    continue;
                };
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
                        source_uv(source.source_rect, sheet.size),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
    }
}

fn world_object_at(state: &Studio, world: [i64; 2]) -> Option<String> {
    state
        .world_doc
        .generated_objects()
        .filter(|object| {
            (0..2).all(|axis| {
                world[axis] >= object.anchor_world[axis]
                    && world[axis] - object.anchor_world[axis]
                        < i64::from(object.footprint_cells[axis])
            })
        })
        .last()
        .map(|object| object.id.clone())
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
        || state.canvas_controls_rect.contains(pointer)
        || (state.atlas_open && state.atlas_rect.contains(pointer))
        || (state.evidence_open && state.evidence_rect.contains(pointer))
        || (state.editor_layout.layers_open && state.layers_rect.contains(pointer))
        || (state.editor_layout.structural_open && state.structural_rect.contains(pointer))
        || (state.editor_layout.world_generator_open
            && state.world_generator_rect.contains(pointer))
        || (state.editor_layout.chunk_manager_open && state.chunk_manager_rect.contains(pointer))
        || (state.editor_layout.asset_authority_open
            && state.asset_authority_rect.contains(pointer))
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
    let semantic = state
        .asset_authority
        .unique_fixture_role(binding)
        .map(str::to_owned);
    state.world_doc.set_visual_override(world, binding.clone());
    state.world_doc.set_semantic_override(world, semantic);
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
    state.world_stroke_binding = None;
    state.world_stroke_role = (!erase).then(|| state.world_terrain_brush.role().to_owned());
    state.world_stroke_erase = erase;
    state.world_stroke_local_only = true;
    world_stroke_apply_cell(state, world);
}

fn world_stroke_apply_cell(state: &mut Studio, world: [i64; 2]) {
    if !state.world_stroke_seen.insert(world) {
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
        // Normal PNT is semantic terrain paint and does not require Source Browser.
        paint_world_local_preview(state, world, &binding);
    }
    state.world_selected = world;
    state.selection_active = true;
    state.world_stroke_cells.push(world);
}

fn world_stroke_apply_segment(state: &mut Studio, to: [i64; 2]) {
    let Some(from) = state.world_stroke_cells.last().copied() else {
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
        state.world_stroke_seen.clear();
        state.world_stroke_binding = None;
        state.world_stroke_role = None;
        return;
    }
    let cells = std::mem::take(&mut state.world_stroke_cells);
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
    if !state.editor_layout.collision_overlay {
        return;
    }
    let visible = canvas.intersect(region);
    if visible.width() <= 0.0 {
        return;
    }
    let x0 = (((visible.left() - region.left()) / px).floor() as isize).max(0) as usize;
    let y0 = (((visible.top() - region.top()) / px).floor() as isize).max(0) as usize;
    let x1 = ((((visible.right() - region.left()) / px).ceil() as usize) + 1).min(size[0]);
    let y1 = ((((visible.bottom() - region.top()) / px).ceil() as usize) + 1).min(size[1]);
    let p = painter.with_clip_rect(canvas);
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
                if px < 16.0 {
                    if mask.blocked_count() > 0 {
                        p.rect_filled(
                            cell,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(112, 18, 24, 76),
                        );
                    }
                } else {
                    let unit = px / COLLISION_MASK_SIDE as f32;
                    for py in 0..COLLISION_MASK_SIDE {
                        for px_i in 0..COLLISION_MASK_SIDE {
                            if mask.blocked(px_i, py) {
                                p.rect_filled(
                                    egui::Rect::from_min_size(
                                        cell.min + egui::vec2(px_i as f32 * unit, py as f32 * unit),
                                        egui::Vec2::splat(unit),
                                    ),
                                    0.0,
                                    egui::Color32::from_rgba_unmultiplied(104, 14, 22, 112),
                                );
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

fn draw_world_workspace(ui: &mut egui::Ui, state: &mut Studio) {
    let available = ui.available_size().max(egui::vec2(200.0, 160.0));
    let (canvas, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
    state.canvas_rect = canvas;
    state.canvas_controls_rect = egui::Rect::from_min_size(
        egui::pos2(canvas.right() - 340.0, canvas.top() + 8.0),
        egui::vec2(340.0, 40.0),
    );
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(20, 29, 27));
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
            let old_zoom = state.zoom;
            let new_zoom = (old_zoom * (wheel / 240.0).exp()).clamp(0.125, 16.0);
            if (new_zoom - old_zoom).abs() > f32::EPSILON {
                if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                    let old_center = canvas.center() + state.pan;
                    let scale = new_zoom / old_zoom;
                    let new_center = pointer - (pointer - old_center) * scale;
                    state.pan = new_center - canvas.center();
                }
                state.zoom = new_zoom;
            }
        }
    }
    if response.dragged_by(egui::PointerButton::Middle)
        || response.dragged_by(egui::PointerButton::Secondary)
    {
        state.pan += response.drag_delta();
    }
    let px = 32.0 * state.zoom;
    let region = egui::Rect::from_center_size(
        canvas.center() + state.pan,
        egui::vec2(size[0] as f32 * px, size[1] as f32 * px),
    );
    state.scene_rect = region;
    let visible = canvas.intersect(region);
    if visible.width() > 0.0 {
        let x0 = (((visible.left() - region.left()) / px).floor() as isize).max(0) as usize;
        let y0 = (((visible.top() - region.top()) / px).floor() as isize).max(0) as usize;
        let x1 = ((((visible.right() - region.left()) / px).ceil() as usize) + 1).min(size[0]);
        let y1 = ((((visible.bottom() - region.top()) / px).ceil() as usize) + 1).min(size[1]);

        let sheet_lookup: std::collections::HashMap<&str, usize> = state
            .sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| (sheet.path.as_str(), index))
            .collect();

        let mut terrain_mesh: Option<(egui::TextureId, egui::Mesh)> = None;
        for ly in y0..y1 {
            for lx in x0..x1 {
                let world = [origin[0] + lx as i64, origin[1] + ly as i64];
                let Some(parts) = state.world_doc.resolved_visual_parts_with_authority_at(
                    world,
                    &state.asset_authority,
                    Some(state.water_animation_ms),
                ) else {
                    continue;
                };
                let cell_min = region.min + egui::vec2(lx as f32 * px, ly as f32 * px);
                for part in parts {
                    let Some(sheet_index) =
                        sheet_lookup.get(part.source.source_asset.as_str()).copied()
                    else {
                        continue;
                    };
                    let sheet = &state.sheets[sheet_index];
                    let Some(texture) = sheet.texture else {
                        continue;
                    };
                    let [dx, dy, dw, dh] = part.destination_rect_px;
                    let target = egui::Rect::from_min_size(
                        cell_min + egui::vec2(dx as f32 * px / 32.0, dy as f32 * px / 32.0),
                        egui::vec2(dw as f32 * px / 32.0, dh as f32 * px / 32.0),
                    );
                    let needs_new_mesh = terrain_mesh.as_ref().is_none_or(|(id, _)| *id != texture);
                    if needs_new_mesh {
                        if let Some((_, mesh)) = terrain_mesh.take() {
                            painter.add(egui::Shape::mesh(mesh));
                        }
                        terrain_mesh = Some((texture, egui::Mesh::with_texture(texture)));
                    }
                    if let Some((_, mesh)) = terrain_mesh.as_mut() {
                        mesh.add_rect_with_uv(
                            target,
                            source_uv(part.source.source_rect, sheet.size),
                            egui::Color32::WHITE,
                        );
                    }
                }
            }
        }
        if let Some((_, mesh)) = terrain_mesh.take() {
            painter.add(egui::Shape::mesh(mesh));
        }

        let visible_world_min = [origin[0] + x0 as i64, origin[1] + y0 as i64];
        let visible_world_max = [origin[0] + x1 as i64, origin[1] + y1 as i64];
        let draw_minor_details = px >= 5.5;
        let min_chunk = [
            visible_world_min[0].div_euclid(state.world_doc.chunk_side as i64),
            visible_world_min[1].div_euclid(state.world_doc.chunk_side as i64),
        ];
        let max_chunk = [
            visible_world_max[0].div_euclid(state.world_doc.chunk_side as i64),
            visible_world_max[1].div_euclid(state.world_doc.chunk_side as i64),
        ];
        let pixels_per_point = ui.ctx().pixels_per_point().max(1.0);
        let snap_point = |value: f32| (value * pixels_per_point).round() / pixels_per_point;
        for cy in min_chunk[1]..=max_chunk[1] {
            for cx in min_chunk[0]..=max_chunk[0] {
                let Some(chunk) = state.world_doc.chunk([cx, cy]) else {
                    continue;
                };
                for object in &chunk.generated_objects {
                    if state
                        .world_doc
                        .suppressed_generated_objects
                        .contains(&object.id)
                    {
                        continue;
                    }
                    let margin = 4_i64;
                    let footprint = [
                        i64::from(object.footprint_cells[0]),
                        i64::from(object.footprint_cells[1]),
                    ];
                    if object.anchor_world[0] + footprint[0] + margin < visible_world_min[0]
                        || object.anchor_world[1] + footprint[1] + margin < visible_world_min[1]
                        || object.anchor_world[0] - margin > visible_world_max[0]
                        || object.anchor_world[1] - margin > visible_world_max[1]
                    {
                        continue;
                    }
                    if !draw_minor_details && object.label.contains("detail") {
                        continue;
                    }
                    for part in &object.parts {
                        let Some(sheet_index) =
                            sheet_lookup.get(part.source.source_asset.as_str()).copied()
                        else {
                            continue;
                        };
                        let sheet = &state.sheets[sheet_index];
                        let Some(texture) = sheet.texture else {
                            continue;
                        };
                        let x = (object.anchor_world[0] - origin[0]) as f32
                            + part.offset_px[0] as f32 / 32.0;
                        let y = (object.anchor_world[1] - origin[1]) as f32
                            + part.offset_px[1] as f32 / 32.0;
                        let min = region.min + egui::vec2(x * px, y * px);
                        let size_px = egui::vec2(
                            part.source.source_rect[2] as f32 * px / 32.0,
                            part.source.source_rect[3] as f32 * px / 32.0,
                        );
                        let target = egui::Rect::from_min_max(
                            egui::pos2(snap_point(min.x), snap_point(min.y)),
                            egui::pos2(
                                snap_point(min.x + size_px.x),
                                snap_point(min.y + size_px.y),
                            ),
                        );
                        if target.intersects(canvas) {
                            // Objects remain direct source-region draws. Do not batch
                            // them into shared geometry: exact alpha edges and source
                            // boundaries are more important than saving a few object
                            // shapes, while terrain remains batched separately.
                            painter.image(
                                texture,
                                target,
                                source_uv(part.source.source_rect, sheet.size),
                                egui::Color32::WHITE,
                            );
                        }
                    }
                }
            }
        }
    }
    draw_world_collision_overlay(&painter, state, region, canvas, origin, size, px);
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

    if primary_pressed {
        if let Some(pointer) = pointer {
            if let Some((world, pixel)) =
                world_pointer_cell(state, pointer, region, origin, size, px)
            {
                state.world_selected = world;
                state.selection_active = true;
                if state.collision_world_pixel_edit && state.editor_layout.structural_open {
                    checkpoint_world(state);
                    state.collision_stroke_active = true;
                    paint_world_collision_brush(state, world, pixel);
                } else {
                    match state.tool {
                        CanvasTool::Select => {
                            state.world_selected_object = world_object_at(state, world);
                        }
                        CanvasTool::Paint => {
                            world_semantic_stroke_begin(state, world, false);
                        }
                        CanvasTool::Erase => {
                            world_semantic_stroke_begin(state, world, true);
                        }
                        CanvasTool::Sample => {
                            if let Some(role) = state.world_doc.resolved_role(world) {
                                if let Some(brush) = WorldTerrainBrush::from_role(&role) {
                                    state.world_terrain_brush = brush;
                                }
                            }
                            if let Some(source) = state
                                .world_doc
                                .resolved_source_with_authority(world, &state.asset_authority)
                            {
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
    egui::Area::new(egui::Id::new("havenwild.world.view.controls"))
        .fixed_pos(egui::pos2(canvas.right() - 330.0, canvas.top() + 8.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(25, 33, 37, 232))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button("Fit 3×3").clicked() {
                            let fx = canvas.width() / size[0] as f32 / 32.0;
                            let fy = canvas.height() / size[1] as f32 / 32.0;
                            state.zoom = fx.min(fy).clamp(0.125, 16.0) * 0.92;
                            state.pan = egui::Vec2::ZERO;
                        }
                        if state.collision_world_pixel_edit {
                            ui.separator();
                            ui.label("COL");
                            if ui
                                .small_button("-")
                                .on_hover_text("Smaller collision brush")
                                .clicked()
                            {
                                state.collision_brush_radius =
                                    state.collision_brush_radius.saturating_sub(1).max(1);
                            }
                            ui.label(format!("{}px", state.collision_brush_radius));
                            if ui
                                .small_button("+")
                                .on_hover_text("Larger collision brush")
                                .clicked()
                            {
                                state.collision_brush_radius =
                                    (state.collision_brush_radius + 1).min(8);
                            }
                        }
                        if ui.small_button("Summer scene").clicked() {
                            state.world_mode = false;
                            state.selection_active = false;
                        }
                    });
                });
        });
}

fn draw_world(ui: &mut egui::Ui, state: &mut Studio) {
    if state.world_mode {
        draw_world_workspace(ui, state);
        return;
    }
    let available = ui.available_size().max(egui::vec2(200.0, 160.0));
    let (canvas, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
    state.canvas_rect = canvas;
    // Rulers are painted inside the allocated canvas; no dock, layout shrink or extra scene layer.
    let ruler_inset =
        if canvas_rulers::visible(state.editor_layout.rulers_visible, state.focus_mode) {
            egui::vec2(canvas_rulers::LEFT, canvas_rulers::TOP)
        } else {
            egui::Vec2::ZERO
        };
    state.canvas_controls_rect = egui::Rect::from_min_size(
        egui::pos2(canvas.right() - 285.0, canvas.top() + ruler_inset.y + 6.0),
        egui::vec2(285.0, 40.0),
    );
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(20, 29, 27));
    if state.playtest.is_none() && response.hovered() {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if wheel.abs() > 0.01 {
            let previous = state.zoom;
            let next = (previous * (wheel / 240.0).exp()).clamp(0.125, 16.0);
            if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                // Zoom around pointer, not scene center. Keeps hovered world coordinate
                // fixed and prevents a jump when stepping through zoom levels.
                let ratio = next / previous;
                state.pan = state.pan * ratio + (pointer - canvas.center()) * (1.0 - ratio);
            }
            state.zoom = next;
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
                let Some(sheet) = state.sheets.iter().find(|s| s.path == tile.source_asset) else {
                    continue;
                };
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
    // The marker is an editor playtest gizmo, NOT generated player art or part of scene data.
    if let Some(play) = &state.playtest {
        let center = scene.min + egui::vec2(play.position[0] * px, play.position[1] * px);
        if canvas.contains(center) {
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
            state.selection_active.then_some(state.selected_world),
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
                state.message = format!("Selected {:?} cell {},{}. Select a placed instance in Layers, or open SRC and place one on Objects.", state.active_layer, pos[0], pos[1]);
            }
        }
    }
    if state.playtest.is_none()
        && response.double_clicked_by(egui::PointerButton::Primary)
        && state.selected_object_id.is_some()
    {
        state.properties_open = true;
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
                                let sheet = state.active_sheet;
                                let source = selected_source_rect(state);
                                apply_source(state, pos, sheet, source);
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
    // These view controls are canvas chrome, not a separate dock or inspector.
    let control_pos = egui::pos2(canvas.right() - 278.0, canvas.top() + ruler_inset.y + 9.0);
    if state.playtest.is_none() {
        egui::Area::new(egui::Id::new("havenwild.canvas.view.controls"))
        .fixed_pos(control_pos)
        .show(ui.ctx(), |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(25, 33, 37, 232))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.small_button("Fit").on_hover_text("Fit current Summer scene to canvas").clicked() {
                            let size = state.scene.size;
                            focus_mapping_zone(state, [0,0], size);
                        }
                        if ui.small_button("+").clicked() { state.zoom = (state.zoom * 1.25).min(16.0); }
                        if ui.small_button("-").clicked() { state.zoom = (state.zoom / 1.25).max(0.125); }
                        if state.selected_object_id.is_some() && ui.small_button("Properties").on_hover_text("Show optional object properties; no permanent Inspector column").clicked() { state.properties_open = true; }
                    });
                });
        });
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
    state.editor_layout.terrain_mapper.visible = state.atlas_open;
    state.editor_layout.inspect_alpha = state.inspect_alpha;
    if let Err(error) = state.editor_layout.save(&state.layout_path) {
        state.message = format!("Editor layout save failed: {error}");
    }
}

fn set_mapper_dock(state: &mut Studio, dock: SurfaceDock) {
    if state.editor_layout.terrain_mapper.move_to(dock) {
        state.atlas_open = true;
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
        ui.strong("ElizaWy tools");
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
        state.atlas_open = false;
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
        .filter(|entry| family == "All" || entry.family == family)
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
                for name in ["All", "Terrain", "Structure", "Objects", "FX"] {
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
            "Non-terrain artwork cannot replace Ground. Select Objects, Structures or Foreground under LYR.");
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
    ui.small("The normal World brush is source-driven: semantic terrain resolves through the recovered Native/LPC topology into exact Terrain/terrain_summer.png pixels. All 305 Summer source cells are runtime-selectable; the source PNG remains read-only and the legacy color classifier remains diagnostic only.");

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
                    "Interior variants: Grass {} · Dirt {} · Water {}",
                    state.asset_authority.summer_flatworld_fill_variant_count("Grass"),
                    state.asset_authority.summer_flatworld_fill_variant_count("MudBank"),
                    state.asset_authority.summer_flatworld_fill_variant_count("RiverWater")
                ));
                ui.separator();
                ui.label(format!(
                    "Water cycle: {} source-backed phases @ {} ms",
                    state.asset_authority.summer_water_animation_frame_count(),
                    state.asset_authority.summer_water_animation_frame_duration_ms()
                ));
            });
            ui.small("Water animation is currently a source-backed Summer runtime candidate for visual review; shoreline topology stays static and source-exact. Cliffs and waterfall animation remain separate structural passes.");
            egui::Grid::new("summer.flatworld.safe.fills")
                .num_columns(3)
                .spacing(egui::vec2(14.0, 4.0))
                .show(ui, |ui| {
                    for (role, label) in [("Grass", "GRS"), ("MudBank", "DIR"), ("RiverWater", "WTR")] {
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
            ui.small("Experimental v1 connected-alpha tree/rock crops are quarantined because they could include guide bars, opaque neighbor pixels, or clipped trunks. Large procedural objects stay disabled until their exact source bounds are reviewed; source PNGs are never edited.");
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
        "Current recipe: {classified}/16 classified · {certified}/16 certified · sprites {sprites} · composites {composites} · unsupported {unsupported} · unmapped {unmapped}"
    ));
    ui.small("This counter is intentionally narrower than the recovered Summer source map. It measures only the new dual-grid Grass/Void recipe family.");
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

fn draw_mapper_contents(ui: &mut egui::Ui, state: &mut Studio, horizontal: bool) {
    draw_mapper_header(ui, state);
    ui.horizontal_wrapped(|ui| {
        for (tab, label) in [
            ("source", "Source Browser"),
            ("recipe", "Terrain Rules"),
            ("review", "Source Evidence"),
        ] {
            if ui
                .selectable_label(state.editor_layout.terrain_mapper_tab == tab, label)
                .clicked()
            {
                state.editor_layout.terrain_mapper_tab = tab.into();
                save_editor_layout(state);
            }
        }
    });
    ui.separator();
    let active_tab = state.editor_layout.terrain_mapper_tab.clone();
    if state.editor_layout.terrain_mapper_compact {
        match active_tab.as_str() {
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
    if !state.atlas_open {
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
    let mut window = egui::Window::new("ElizaWy · Source / Terrain / Evidence")
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
        state.atlas_open = false;
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
            WorldBehaviorScope::SourceProfile => {
                if let Some(source) = state
                    .world_doc
                    .resolved_source_with_authority(target_world, &state.asset_authority)
                {
                    state
                        .world_doc
                        .set_source_collision_profile_untracked(source, Some(mask));
                }
            }
        }
    }
    state.world_dirty = true;
}

fn selected_world_source(state: &Studio, world: [i64; 2]) -> Option<SourceBinding> {
    state
        .world_doc
        .resolved_source_with_authority(world, &state.asset_authority)
}

fn set_world_collision_for_scope(
    state: &mut Studio,
    world: [i64; 2],
    mask: Option<CollisionMask32>,
) {
    checkpoint_world(state);
    match state.world_behavior_scope {
        WorldBehaviorScope::LocalArea => state.world_doc.set_collision_mask(world, mask),
        WorldBehaviorScope::SourceProfile => {
            if let Some(source) = selected_world_source(state, world) {
                state.world_doc.set_source_collision_profile(source, mask);
            } else {
                state.message = "No exact source binding is resolved for this world cell.".into();
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
        WorldBehaviorScope::SourceProfile => {
            if let Some(source) = selected_world_source(state, world) {
                state.world_doc.set_source_traversal_profile(source, value);
            } else {
                state.message = "No exact source binding is resolved for this world cell.".into();
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
        WorldBehaviorScope::SourceProfile => {
            selected_world_source(state, world).and_then(|source| {
                state
                    .world_doc
                    .source_behavior_profile(&source)
                    .and_then(|profile| profile.collision.clone())
            })
        }
    }
}

fn world_traversal_scope_mode(state: &Studio, world: [i64; 2]) -> TraversalMode {
    match state.world_behavior_scope {
        WorldBehaviorScope::LocalArea => state
            .world_doc
            .cell_override(world)
            .and_then(|item| item.traversal)
            .unwrap_or(TraversalMode::Auto),
        WorldBehaviorScope::SourceProfile => selected_world_source(state, world)
            .and_then(|source| {
                state
                    .world_doc
                    .source_behavior_profile(&source)
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
        for scope in [WorldBehaviorScope::LocalArea, WorldBehaviorScope::SourceProfile] {
            if ui
                .selectable_label(state.world_behavior_scope == scope, scope.label())
                .on_hover_text(match scope {
                    WorldBehaviorScope::LocalArea => "Override only these world coordinates. Regeneration preserves the exception.",
                    WorldBehaviorScope::SourceProfile => "Reusable Havenwild metadata keyed to this exact source region. The ElizaWy PNG is never edited.",
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
                "Reusable behavior profiles: {} · local world overrides: {}",
                state.world_doc.source_behavior_profiles.len(),
                state.world_doc.cell_overrides.len()
            ));
            ui.small("Reusable profiles and local overrides are Havenwild metadata. Original ElizaWy source bytes remain immutable.");
        });
}

// An authored structural cell is separate from the immutable original artwork.
// Edit commands use the same scene-v2 undo/redo/save path as visual changes.
fn draw_structural_panel(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if !state.editor_layout.structural_open {
        state.structural_rect = egui::Rect::NOTHING;
        return;
    }
    let ctx = viewport_ui.ctx().clone();
    let dock = state.editor_layout.structural_dock;
    let compact = state.editor_layout.structural_compact;
    let mut open = true;
    let mut window = egui::Window::new("World Properties")
        .id(egui::Id::new("havenwild.structural.editor.overlay"))
        .open(&mut open)
        .resizable(true)
        .min_width(if compact { 240.0 } else { 285.0 });
    window = match dock {
        SurfaceDock::Left => window.anchor(egui::Align2::LEFT_TOP, [60.0, 62.0]),
        SurfaceDock::Right => window.anchor(egui::Align2::RIGHT_TOP, [-58.0, 62.0]),
        SurfaceDock::Bottom => window.anchor(egui::Align2::CENTER_BOTTOM, [0.0, -30.0]),
        SurfaceDock::Floating | SurfaceDock::Center => window.default_pos(egui::pos2(1020.0, 96.0)),
    };
    let response = window.show(&ctx, |ui| {
        let mut requested_dock = None;
        let mut requested_compact = compact;
        ui.horizontal_wrapped(|ui| {
            for (candidate, label) in [
                (SurfaceDock::Left, "Dock L"),
                (SurfaceDock::Right, "Dock R"),
                (SurfaceDock::Bottom, "Dock B"),
                (SurfaceDock::Floating, "Float"),
            ] {
                if ui.selectable_label(dock == candidate, label).clicked() {
                    requested_dock = Some(candidate);
                }
            }
            ui.separator();
            if ui
                .selectable_label(requested_compact, if requested_compact { "Compact" } else { "Full" })
                .on_hover_text("Compact keeps only immediate collision/elevation controls. Full exposes semantic terrain and connector metadata.")
                .clicked()
            {
                requested_compact = !requested_compact;
            }
        });
        if let Some(candidate) = requested_dock {
            state.editor_layout.structural_dock = candidate;
            save_editor_layout(state);
        }
        if requested_compact != state.editor_layout.structural_compact {
            state.editor_layout.structural_compact = requested_compact;
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
        if state.world_mode { draw_world_structural_contents(ui, state); return; }
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
                if explicit.is_some() { "Authored 32×32 mask" } else { "Automatic/coarse collision" },
                explicit.as_ref().map(|mask| mask.blocked_count()).unwrap_or(if automatic_cell_blocked(state, index) { 1024 } else { 0 })
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
                if ui.selectable_label(cell.blocks_traversal == value, label).clicked() {
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
                    for label in ["", "grass", "dirt", "sand", "water", "cliff", "road", "rock"] {
                        ui.selectable_value(
                            &mut terrain,
                            label.to_string(),
                            if label.is_empty() { "Unspecified" } else { label },
                        );
                    }
                });
            if cell.terrain_kind.as_deref().unwrap_or("") != terrain {
                cell.terrain_kind = if terrain.is_empty() { None } else { Some(terrain) };
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
                cell.connector = if connector.is_empty() { None } else { Some(connector) };
                changed = true;
            }
            ui.separator();
            ui.small("PIE honors explicitly authored traversal overrides over provisional River/Grass hints. Cliff connector travel, footprints and hydrology remain separate certification work.");
        }

        if ui.button("Clear this cell's structural overrides").clicked() {
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
            match state.layered_history.execute(&mut state.layered_scene, update) {
                Ok(()) => {
                    state.message = format!(
                        "Updated explicit collision/elevation for cell {},{}; save scene to persist.",
                        pos[0], pos[1]
                    )
                }
                Err(error) => state.message = format!("Structural edit not applied: {error}"),
            }
        }
    });
    state.structural_rect = response
        .as_ref()
        .map(|response| response.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if !open {
        state.editor_layout.structural_open = false;
        save_editor_layout(state);
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
    state.atlas_open = true;
    state.editor_layout.terrain_mapper.visible = true;
    save_editor_layout(state);
}

fn draw_tool_rail(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    egui::Panel::left("havenwild.tool.rail")
        .resizable(false)
        .default_size(52.0)
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(27, 34, 39, 232)))
        .show(viewport_ui, |ui| {
            ui.set_min_width(52.0);
            ui.set_max_width(52.0);
            ui.add_space(7.0);
            ui.small("EDIT")
                .on_hover_text("Permanent direct-canvas tool rail. Advanced tool applications open from the Havenwild launcher at the bottom.");
            for (tool, glyph) in [
                (CanvasTool::Select, "SEL"),
                (CanvasTool::Paint, "PNT"),
                (CanvasTool::Erase, "ERS"),
                (CanvasTool::Sample, "PIP"),
            ] {
                let command = tool_command(tool);
                if ui
                    .add_sized([43.0, 29.0], egui::Button::selectable(state.tool == tool, glyph))
                    .on_hover_text(command_tooltip(command))
                    .clicked()
                {
                    state.tool = tool;
                    state.message = format!(
                        "{} tool. Advanced applications are available from the Havenwild launcher; original source bytes remain untouched.",
                        tool.label()
                    );
                }
                ui.add_space(2.0);
            }

            if state.world_mode {
                ui.separator();
                ui.small("TERR")
                    .on_hover_text("Semantic terrain brushes. Paint meaning; Havenwild resolves exact ElizaWy source tiles automatically from local topology.");
                for brush in [
                    WorldTerrainBrush::Grass,
                    WorldTerrainBrush::DirtBank,
                    WorldTerrainBrush::Water,
                ] {
                    if ui
                        .add_sized(
                            [43.0, 27.0],
                            egui::Button::selectable(state.world_terrain_brush == brush, brush.glyph()),
                        )
                        .on_hover_text(format!(
                            "{} terrain brush · click/drag with PNT · exact-source autotile",
                            brush.label()
                        ))
                        .clicked()
                    {
                        state.world_terrain_brush = brush;
                        state.tool = CanvasTool::Paint;
                        state.message = format!(
                            "{} terrain brush selected. Hold left mouse and drag on the World canvas; exact-source autotiling updates around the stroke.",
                            brush.label()
                        );
                    }
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let launcher = command_spec(StudioCommand::ToggleLauncher);
                if ui
                    .add_sized([43.0, 35.0], egui::Button::selectable(state.launcher_open, "HW"))
                    .on_hover_text(command_tooltip(StudioCommand::ToggleLauncher))
                    .clicked()
                {
                    state.launcher_open = !state.launcher_open;
                }
                ui.small("APPS").on_hover_text(format!("{} · {}", launcher.label, launcher.shortcut));
            });
        });
}

fn draw_layer_rail(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    egui::Panel::right("havenwild.layer.rail")
        .resizable(false)
        .default_size(52.0)
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(27, 34, 39, 232)))
        .show(viewport_ui, |ui| {
            ui.set_min_width(52.0);
            ui.set_max_width(52.0);
            ui.add_space(7.0);
            ui.small("LYR").on_hover_text("Permanent active-layer rail. Click a layer to make it the direct-edit destination.");
            for (id, glyph, label) in [
                (LayerId::Foreground, "FG", "Foreground"),
                (LayerId::Objects, "OBJ", "Objects / vegetation / reeds"),
                (LayerId::Structures, "STR", "Structures"),
                (LayerId::Elevation, "ELV", "Elevation / cliff faces"),
                (LayerId::Water, "WTR", "Water / FX"),
                (LayerId::TerrainDetails, "DET", "Terrain details / banks"),
                (LayerId::Ground, "GRD", "Ground terrain"),
            ] {
                if ui
                    .add_sized([43.0, 27.0], egui::Button::selectable(state.active_layer == id, glyph))
                    .on_hover_text(format!("Active layer: {label}"))
                    .clicked()
                {
                    state.active_layer = id;
                    state.selected_object_id = None;
                    state.dragging_object = None;
                    state.drag_preview_origin = None;
                    state.message = format!("Active visual layer: {label}.");
                }
                ui.add_space(1.0);
            }
            ui.separator();
            if ui
                .add_sized(
                    [43.0, 27.0],
                    egui::Button::selectable(state.editor_layout.collision_overlay, "COL"),
                )
                .on_hover_text("Collision layer overlay. Red = impassable. Click to toggle; the full Collision / Elevation editor opens so exact 32×32 masks can be corrected.")
                .clicked()
            {
                state.editor_layout.collision_overlay = !state.editor_layout.collision_overlay;
                if state.editor_layout.collision_overlay {
                    state.editor_layout.structural_open = true;
                }
                save_editor_layout(state);
            }
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui
                    .add_sized([43.0, 29.0], egui::Button::new("•••"))
                    .on_hover_text("Open the feature-rich Scene / Layers panel. It floats or docks over the canvas without resizing it.")
                    .clicked()
                {
                    state.editor_layout.layers_open = true;
                    save_editor_layout(state);
                }
            });
        });
}

fn draw_layer_panel(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if !state.editor_layout.layers_open {
        state.layers_rect = egui::Rect::NOTHING;
        return;
    }
    let ctx = viewport_ui.ctx().clone();
    let dock = state.editor_layout.scene_layers_dock;
    let compact = state.editor_layout.scene_layers_compact;
    let mut open = true;
    let mut window = egui::Window::new("Scene / Layers")
        .id(egui::Id::new("havenwild.layers.overlay"))
        .open(&mut open)
        .resizable(true)
        .min_width(if compact { 230.0 } else { 300.0 });
    window = match dock {
        SurfaceDock::Left => window.anchor(egui::Align2::LEFT_TOP, [60.0, 62.0]),
        SurfaceDock::Right => window.anchor(egui::Align2::RIGHT_TOP, [-58.0, 62.0]),
        SurfaceDock::Bottom => window.anchor(egui::Align2::CENTER_BOTTOM, [0.0, -30.0]),
        SurfaceDock::Floating | SurfaceDock::Center => window.default_pos(egui::pos2(92.0, 130.0)),
    };
    let response = window.show(&ctx, |ui| {
        let mut requested_dock = None;
        let mut requested_compact = compact;
        ui.horizontal_wrapped(|ui| {
            for (candidate, label) in [
                (SurfaceDock::Left, "Dock L"),
                (SurfaceDock::Right, "Dock R"),
                (SurfaceDock::Bottom, "Dock B"),
                (SurfaceDock::Floating, "Float"),
            ] {
                if ui.selectable_label(dock == candidate, label).clicked() {
                    requested_dock = Some(candidate);
                }
            }
            ui.separator();
            if ui
                .selectable_label(requested_compact, if requested_compact { "Compact" } else { "Full" })
                .on_hover_text("Compact keeps the active-layer and save controls. Full adds object lists and source-placement helpers.")
                .clicked()
            {
                requested_compact = !requested_compact;
            }
        });
        if let Some(candidate) = requested_dock {
            state.editor_layout.scene_layers_dock = candidate;
            save_editor_layout(state);
        }
        if requested_compact != state.editor_layout.scene_layers_compact {
            state.editor_layout.scene_layers_compact = requested_compact;
            save_editor_layout(state);
        }

        if state.world_mode {
            ui.strong("World composition layers");
            ui.small(format!("{} chunks · {} authored overrides · {} worldgen tombstones", state.world_doc.chunks.len(), state.world_doc.cell_overrides.len(), state.world_doc.suppressed_generated_objects.len()));
            ui.separator();
            ui.label("Base semantic composition · worldgen-owned");
            ui.label("ElizaWy visual resolution · exact canonical source regions");
            ui.label("Authored visual overrides · preserved on regenerate");
            ui.label("Elevation overrides · logical 0..30");
            ui.label("Collision masks · 32×32 pixels per cell");
            ui.label("Worldgen placements · source-exact templates + tombstones");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.add_enabled(!state.world_undo.is_empty(), egui::Button::new("Undo")).clicked() { undo_scene_edit(state); }
                if ui.add_enabled(!state.world_redo.is_empty(), egui::Button::new("Redo")).clicked() { redo_scene_edit(state); }
                if ui.add_enabled(state.world_dirty, egui::Button::new("Save World")).clicked() { save_world_document(state); }
            });
            return;
        }

        ui.small(format!(
            "{} · {} placed objects · {}",
            state.layered_scene.name,
            state.layered_scene.objects.len(),
            if state.layered_history.is_dirty() { "unsaved changes" } else { "saved" }
        ));
        ui.separator();
        ui.small("Active placement layer");
        let previous_layer = state.active_layer;
        for (id, label) in [
            (LayerId::Foreground, "Foreground"),
            (LayerId::Objects, "Objects / vegetation / reeds"),
            (LayerId::Structures, "Structures"),
            (LayerId::Elevation, "Elevation / cliff faces"),
            (LayerId::Water, "Water / FX"),
            (LayerId::TerrainDetails, "Terrain details / banks"),
            (LayerId::Ground, "Ground — terrain only"),
        ] {
            ui.selectable_value(&mut state.active_layer, id, label);
        }
        if state.active_layer != previous_layer {
            state.selected_object_id = None;
            state.dragging_object = None;
            state.drag_preview_origin = None;
        }

        if !state.editor_layout.scene_layers_compact {
            ui.separator();
            ui.strong(format!("Placed objects ({})", state.layered_scene.objects.len()));
            if state.layered_scene.objects.is_empty() {
                ui.small("No placed objects in this scene. Open Source Browser from the Havenwild launcher, select an original source region, then use Paint (B).");
                if ui.button("Open Source Browser").clicked() {
                    open_mapper_tab(state, "source");
                }
            } else {
                let mut rows = state
                    .layered_scene
                    .objects
                    .iter()
                    .map(|object| {
                        (
                            object.id.clone(),
                            object.label.clone(),
                            object.layer,
                            object.anchor,
                            object.footprint,
                        )
                    })
                    .collect::<Vec<_>>();
                rows.reverse();
                egui::ScrollArea::vertical()
                    .id_salt("havenwild.scene.objects.list")
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for (id, label, layer, anchor, footprint) in rows {
                            let is_selected = state.selected_object_id.as_deref() == Some(id.as_str());
                            if ui
                                .selectable_label(
                                    is_selected,
                                    format!("{} · {:?} ({},{})", id, layer, anchor[0], anchor[1]),
                                )
                                .on_hover_text(label)
                                .clicked()
                            {
                                state.selected_object_id = Some(id);
                                state.active_layer = layer;
                                state.selected_world = [
                                    (anchor[0] + (footprint[0] as i32 - 1) / 2) as usize,
                                    (anchor[1] + footprint[1] as i32 - 1) as usize,
                                ];
                                state.selection_active = true;
                                state.tool = CanvasTool::Select;
                            }
                        }
                    });
            }
            if let Some(object) = selected_object(state).cloned() {
                ui.separator();
                ui.strong(format!("Selected: {}", object.id));
                if let Some(part) = object.parts.first() {
                    ui.small(&part.source.source_asset);
                    ui.small(format!(
                        "Original pixels {:?}; {} × {} cell footprint",
                        part.source.source_rect, object.footprint[0], object.footprint[1]
                    ));
                }
                let mut position = object.anchor;
                ui.horizontal(|ui| {
                    ui.label("X");
                    ui.add(egui::DragValue::new(&mut position[0]).speed(1.0));
                    ui.label("Y");
                    ui.add(egui::DragValue::new(&mut position[1]).speed(1.0));
                });
                if position != object.anchor {
                    move_selected_object_to_origin(state, object.id.clone(), position);
                }
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("Find on canvas").clicked() {
                        let center = [object.anchor[0] as usize, object.anchor[1] as usize];
                        focus_mapping_zone(
                            state,
                            center,
                            [object.footprint[0] as usize, object.footprint[1] as usize],
                        );
                        state.selection_active = true;
                    }
                    if ui.small_button("Sample original").clicked() {
                        let pos = state.selected_world;
                        sample_canvas_source(state, pos);
                    }
                    if ui.small_button("Remove").clicked() {
                        remove_selected_object(state);
                    }
                });
            }
        }

        ui.separator();
        let can_place = state.selection_active
            && state.active_sheet < state.sheets.len()
            && (state.active_layer != LayerId::Ground
                || state.sheets[state.active_sheet].path.starts_with("Terrain/"));
        if ui
            .add_enabled(can_place, egui::Button::new("Place selected source at cell"))
            .clicked()
        {
            let pos = state.selected_world;
            let sheet = state.active_sheet;
            let rect = selected_source_rect(state);
            apply_source(state, pos, sheet, rect);
            state.tool = CanvasTool::Select;
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(state.layered_history.can_undo(), egui::Button::new("Undo"))
                .clicked()
            {
                undo_scene_edit(state);
            }
            if ui
                .add_enabled(state.layered_history.can_redo(), egui::Button::new("Redo"))
                .clicked()
            {
                redo_scene_edit(state);
            }
            if ui.button("Save Scene").clicked() {
                save_scene(state);
            }
        });
    });
    state.layers_rect = response
        .as_ref()
        .map(|response| response.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if !open {
        state.editor_layout.layers_open = false;
        save_editor_layout(state);
    }
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
        "World Generator",
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
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Preview").clicked() {
                        match state
                            .world_doc
                            .preview_regenerate_3x3([cx, cy], &state.asset_authority)
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
                        "Generate 3×3"
                    } else {
                        "Regenerate 3×3"
                    };
                    if ui.button(label).clicked() {
                        checkpoint_world(state);
                        match state
                            .world_doc
                            .materialize_3x3([cx, cy], &state.asset_authority)
                        {
                            Ok(()) => {
                                state.world_mode = true;
                                state.world_dirty = true;
                                state.message = format!(
                                    "Regenerated source-backed 3×3 around CH {cx},{cy}; authored terrain/behavior overrides preserved."
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

        egui::CollapsingHeader::new("Terrain paint")
            .id_salt("havenwild.world.generator.terrain")
            .default_open(true)
            .show(ui, |ui| {
                ui.small("Paint semantic terrain; the shared resolver chooses exact ElizaWy source regions. Source PNGs are immutable.");
                ui.horizontal_wrapped(|ui| {
                    for brush in [
                        WorldTerrainBrush::Grass,
                        WorldTerrainBrush::DirtBank,
                        WorldTerrainBrush::Water,
                    ] {
                        if ui
                            .selectable_label(state.world_terrain_brush == brush, brush.label())
                            .clicked()
                        {
                            state.world_terrain_brush = brush;
                            state.tool = CanvasTool::Paint;
                        }
                    }
                });
                ui.small("Water may meet Grass directly. Dirt/Mud bank appears only where DIR semantics are actually authored or generated by an explicit profile.");
            });

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
                    ui.small("Cliff/elevation art is catalogued separately and is not eligible for the flat Summer brush until the cliff pass is enabled.");
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

fn draw_chunk_manager_panel(ctx: &egui::Context, state: &mut Studio) {
    if !state.editor_layout.chunk_manager_open {
        state.chunk_manager_rect = egui::Rect::NOTHING;
        return;
    }
    let dock = state.editor_layout.chunk_manager_dock;
    let compact = state.editor_layout.chunk_manager_compact;
    let mut open = true;
    let response=docked_overlay_window("Chunk Manager","havenwild.world.chunks.overlay",dock,compact,&mut open).show(ctx,|ui|{let(d,c)=panel_header(ui,dock,compact);if let Some(v)=d{state.editor_layout.chunk_manager_dock=v;save_editor_layout(state);}if c!=compact{state.editor_layout.chunk_manager_compact=c;save_editor_layout(state);}ui.small("Chunks are 32×32 working windows in one signed-coordinate world; they do not own separate art.");if state.world_doc.chunks.is_empty(){ui.label("No chunks materialized.");return;}if ui.button("Open world canvas").clicked(){state.world_mode=true;}let rows:Vec<_>=state.world_doc.chunks.iter().map(|ch|(ch.coord,ch.generated_objects.len())).collect();egui::ScrollArea::vertical().max_height(if compact{210.0}else{360.0}).show(ui,|ui|{for(coord,count)in rows{let active=coord==state.world_doc.center_chunk;if ui.selectable_label(active,format!("CH {},{} · {} source-backed placements",coord[0],coord[1],count)).clicked(){state.world_doc.center_chunk=coord;state.world_mode=true;}}});if !compact{ui.label(format!("Revision {} · seed {}",state.world_doc.revision,state.world_doc.seed));ui.label(format!("Overrides {} · tombstones {}",state.world_doc.cell_overrides.len(),state.world_doc.suppressed_generated_objects.len()));}});
    state.chunk_manager_rect = response
        .as_ref()
        .map(|r| r.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if !open {
        state.editor_layout.chunk_manager_open = false;
        save_editor_layout(state);
    }
}

fn draw_asset_authority_panel(ctx: &egui::Context, state: &mut Studio) {
    if !state.editor_layout.asset_authority_open {
        state.asset_authority_rect = egui::Rect::NOTHING;
        return;
    }
    let dock = state.editor_layout.asset_authority_dock;
    let compact = state.editor_layout.asset_authority_compact;
    let mut open = true;
    let response=docked_overlay_window("ElizaWy Asset Authority","havenwild.assets.authority.overlay",dock,compact,&mut open).show(ctx,|ui|{let(d,c)=panel_header(ui,dock,compact);if let Some(v)=d{state.editor_layout.asset_authority_dock=v;save_editor_layout(state);}if c!=compact{state.editor_layout.asset_authority_compact=c;save_editor_layout(state);}ui.strong("One defined ElizaWy asset lane");ui.small("Original source images remain immutable. Historical crosswalks, runtime regions, known roles and source-exact templates are normalized here. Generated artwork is forbidden.");ui.label(format!("{} images · {} runtime regions · {} historical regions",state.asset_authority.counts.runtime_source_images,state.asset_authority.counts.canonical_runtime_regions,state.asset_authority.counts.historical_source_regions));ui.add(egui::TextEdit::singleline(&mut state.asset_authority_filter).hint_text("Find source image..."));let q=state.asset_authority_filter.trim().to_lowercase();let rows:Vec<_>=state.asset_authority.runtime_source_images.iter().filter(|i|q.is_empty()||i.source_path.to_lowercase().contains(&q)).take(if compact{30}else{90}).map(|i|(i.source_path.clone(),i.image_size_px)).collect();egui::ScrollArea::vertical().max_height(if compact{180.0}else{300.0}).show(ui,|ui|{for(path,size)in rows{if ui.button(format!("{path} · {}×{}",size[0],size[1])).clicked(){state.pending_source_sheet=Some(path);open_mapper_tab(state,"source");}}});if !compact{for role in["Grass","MudBank","RiverWater"]{ui.label(format!("{role}: {} source-backed choices",state.asset_authority.role_palette(role).map(|x|x.len()).unwrap_or(0)));}ui.label(format!("Object templates: {}",state.asset_authority.object_templates.len()));}});
    state.asset_authority_rect = response
        .as_ref()
        .map(|r| r.response.rect)
        .unwrap_or(egui::Rect::NOTHING);
    if !open {
        state.editor_layout.asset_authority_open = false;
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
            if ui.button("Scene / Layers").clicked() {
                state.editor_layout.layers_open = true;
                state.launcher_open = false;
                save_editor_layout(state);
            }
            if ui.button("Collision / Elevation").clicked() {
                state.editor_layout.structural_open = true;
                state.launcher_open = false;
                save_editor_layout(state);
            }
            if ui.button("World Generator").clicked() { state.editor_layout.world_generator_open = true; state.launcher_open = false; save_editor_layout(state); }
            if ui.button("Chunk Manager").clicked() { state.editor_layout.chunk_manager_open = true; state.launcher_open = false; save_editor_layout(state); }

            ui.separator();
            ui.strong("Assets / terrain");
            if ui.button("Source Browser").clicked() {
                open_mapper_tab(state, "source");
                state.launcher_open = false;
            }
            if ui.button("Terrain Rules").clicked() {
                open_mapper_tab(state, "recipe");
                state.launcher_open = false;
            }
            if ui.button("Source Evidence").clicked() {
                open_mapper_tab(state, "review");
                state.launcher_open = false;
            }
            if ui.button("ElizaWy Asset Authority").clicked() { state.editor_layout.asset_authority_open = true; state.launcher_open = false; save_editor_layout(state); }
            if ui
                .add_enabled(state.selected_object_id.is_some(), egui::Button::new("Object Properties"))
                .clicked()
            {
                state.properties_open = true;
                state.launcher_open = false;
            }

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
        match load_source_sheet_on_demand(&mut state, &assets, &mut user_textures, &requested) {
            Ok(index) => {
                state.active_sheet = index;
                state.selected_cell = [0, 0];
                state.selected_extent = [1, 1];
                state.message = format!("Original ElizaWy source loaded: {requested}. No asset composition has been inferred.");
            }
            Err(error) => state.message = format!("Source selection refused: {error}"),
        }
    }
}

fn draw_ui(mut contexts: EguiContexts, mut state: ResMut<Studio>, time: Res<Time>) -> Result {
    // One egui context belongs to the only Bevy primary camera. No offscreen pass.
    state.review_browser.poll();
    for sheet in &mut state.sheets {
        sheet.texture = contexts.image_id(&sheet.handle);
    }
    let ctx = contexts.ctx_mut()?;
    state.water_animation_ms = time.elapsed().as_millis() as u64;
    if state.world_mode && state.asset_authority.summer_water_animation_frame_count() > 1 {
        ctx.request_repaint_after(std::time::Duration::from_millis(110));
    }
    if state.review_browser.loading {
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
    }
    if !state.creator_visuals_applied {
        apply_creator_visuals(ctx, &state.theme);
        state.creator_visuals_applied = true;
    }
    if state.playtest.is_some() && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        stop_playtest(&mut state);
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
    if state.playtest.is_none()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::S))
    {
        save_scene(&mut state);
    }
    if state.playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Z))
    {
        undo_scene_edit(&mut state);
    }
    if state.playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Y))
    {
        redo_scene_edit(&mut state);
    }
    if state.playtest.is_none()
        && state.selected_object_id.is_some()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.key_pressed(egui::Key::I))
    {
        state.properties_open = !state.properties_open;
    }
    if state.playtest.is_none() && ctx.input(|input| input.key_pressed(egui::Key::F11)) {
        state.focus_mode = !state.focus_mode;
        if state.focus_mode {
            state.tool = CanvasTool::Select;
            state.launcher_open = false;
        }
    }
    if !ctx.egui_wants_keyboard_input() && ctx.input(|input| input.key_pressed(egui::Key::F6)) {
        if state.playtest.is_some() {
            stop_playtest(&mut state);
        } else if state.world_mode {
            state.message = "World composition PIE is not certified yet; return to Summer Scene for current PIE.".into();
        } else {
            start_playtest(&mut state);
        }
    }
    if state.playtest.is_none()
        && !ctx.egui_wants_keyboard_input()
        && ctx.input(|input| input.modifiers.ctrl && input.key_pressed(egui::Key::Space))
    {
        state.launcher_open = !state.launcher_open;
    }
    if state.playtest.is_none() && !ctx.egui_wants_keyboard_input() {
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
            state.message = format!("{} tool selected from keyboard.", tool.label());
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
            if ui.add_enabled(state.playtest.is_some() || !state.world_mode, egui::Button::new(if state.playtest.is_some() { "■ Stop PIE" } else if state.world_mode { "▶ World PIE · pending" } else { "▶ Play Scene" }))
                .on_hover_text(if state.world_mode && state.playtest.is_none() { "World composition PIE is disabled until chunk traversal/runtime streaming is certified." } else { "Play the same authoring scene in a sandbox; source art is never changed by PIE." }).clicked() {
                if state.playtest.is_some() { stop_playtest(&mut state); } else { start_playtest(&mut state); }
            }
            if state.playtest.is_some() {
                ui.small("PLAYING | original-source art | W/A/S/D | Esc to stop");
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
                if ui.checkbox(&mut state.inspect_alpha, "Inspect selected tile transparency").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.editor_layout.layers_open, "Scene / Layers panel (overlay)").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.editor_layout.structural_open, "Collision / Elevation panel (overlay)").changed() { save_editor_layout(&mut state); }
                if ui.add_enabled(state.selected_object_id.is_some(), egui::Button::new("Selected object properties (on demand)")).clicked() { state.properties_open = true; }
                if ui.checkbox(&mut state.focus_mode, "Focus canvas     F11").changed() && state.focus_mode {
                    state.tool = CanvasTool::Select;
                }
            });
            ui.menu_button("Scene", |ui| {
                if ui.button("Save mapping-lab corrections").clicked() { save_scene(&mut state); }
                if ui.button("Fit Summer canvas").clicked() {
                    let size = state.scene.size;
                    focus_mapping_zone(&mut state, [0, 0], size);
                }
                ui.small("The active Summer World uses original River ground plus editable, individually placed original-source scenery. The source PNGs and prior local drafts remain immutable.");
                ui.small(format!(
                    "Summer source coverage: {} original cells used here / {} recovered globally · {} approved object compositions · DG {}/16 certified.",
                    state.mapping_lab.summer_source_cells_actually_used,
                    state.mapping_lab.summer_recovered_source_cells,
                    state.mapping_lab.visual_samples.len(),
                    state.mapping_lab.certified_recipe_masks,
                ));
                ui.small("0/16 DG recipe masks certified; object footprints/terrain rules still require review.");
                ui.small("Atlas sheets stay in Source Browser; original collection demo scenes are reference only.");
                ui.separator();
                ui.small("One active layered canvas. Older drafts and original River source remain protected.");
                ui.small(format!("{} manually placed source-bound instances in the current scene.", state.layered_scene.objects.len()));
            });
            ui.menu_button("Tools", |ui| {
                if ui
                    .button("Havenwild Tool Launcher    Ctrl+Space")
                    .on_hover_text(command_tooltip(StudioCommand::ToggleLauncher))
                    .clicked()
                {
                    state.launcher_open = !state.launcher_open;
                }
                ui.separator();
                if ui.button("Scene / Layers panel").clicked() {
                    state.editor_layout.layers_open = true;
                    save_editor_layout(&mut state);
                }
                if ui.button("Collision / Elevation panel").clicked() {
                    state.editor_layout.structural_open = true;
                    save_editor_layout(&mut state);
                }
                if ui.button("Source Browser").clicked() {
                    open_mapper_tab(&mut state, "source");
                }
                if ui.button("Terrain Rules").clicked() {
                    open_mapper_tab(&mut state, "recipe");
                }
                if ui.button("Source Evidence").clicked() {
                    open_mapper_tab(&mut state, "review");
                }
                if ui.button("Source / mapping information").clicked() {
                    state.evidence_open = !state.evidence_open;
                }
                ui.separator();
                if ui.button("World Generator").clicked() { state.editor_layout.world_generator_open = true; save_editor_layout(&mut state); }
                if ui.button("Chunk Manager").clicked() { state.editor_layout.chunk_manager_open = true; save_editor_layout(&mut state); }
                if ui.button("ElizaWy Asset Authority").clicked() { state.editor_layout.asset_authority_open = true; save_editor_layout(&mut state); }
                ui.add_enabled(false, egui::Button::new("Pixel Studio · planned"));
                ui.add_enabled(false, egui::Button::new("Animation Studio · planned"));
            });
            ui.menu_button("Help", |ui| {
                ui.label("Active Summer visual study uses original PNGs unchanged over a pinned River-ground fixture; never replace source pixels.");
                ui.label("SEL selects and drags complete objects; save edits in the protected Summer draft. Earlier River and mapping drafts remain untouched.");
                ui.label("Mapped source cells are evidence, NOT automatically certified DG recipes.");
                ui.label("Select is the safe default: clicking does not paint.");
                ui.label("PNT places complete source regions as instances on Objects/Structures/Elevation/Foreground; Ground/Water/detail edits remain per-cell.");
                ui.label("SEL selects and drags placed objects; use Scene / Layers for the instance list and position controls. ERS/Delete removes an object without erasing the ground underneath.");
                ui.label("PIP samples a source cell. Middle/right drag pans.");
                ui.label("Worldgen composes only canonical ElizaWy source regions. Regeneration preserves authored world corrections and tombstones; no image asset is generated.");
            });
            }
            ui.separator();
            if state.world_mode {
                ui.small(format!("Studio 0.8.1 · World {}{} · {} chunks · {}", state.world_doc.world_id, if state.world_dirty { " *" } else { "" }, state.world_doc.chunks.len(), state.tool.label()));
            } else {
                ui.small(format!("{}{} · {}", state.layered_scene.name, if state.layered_history.is_dirty() || state.scene_needs_initial_save { " *" } else { "" }, if state.playtest.is_some() { "PIE / source-snapshot" } else { state.tool.label() }));
            }
            if state.focus_mode && ui.small_button("Exit Focus").clicked() { state.focus_mode = false; }
        });
    });

    if !state.focus_mode {
        egui::Panel::top("havenwild.command.toolbar").show(&mut viewport_ui, |ui| {
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
                    .add_enabled(state.playtest.is_some() || !state.world_mode, egui::Button::new(if state.playtest.is_some() { "Stop PIE" } else if state.world_mode { "World PIE · pending" } else { "Play" }))
                    .on_hover_text(if state.world_mode && state.playtest.is_none() { "World composition PIE is intentionally disabled until chunk traversal/runtime streaming is certified.".to_owned() } else { command_tooltip(StudioCommand::TogglePlay) })
                    .clicked()
                {
                    if state.playtest.is_some() { stop_playtest(&mut state); } else { start_playtest(&mut state); }
                }
                ui.separator();
                if ui
                    .selectable_label(state.editor_layout.chunk_grid_visible, "Chunks")
                    .on_hover_text("Toggle world chunk outlines and coordinates. World mode shows real materialized 32×32 chunk boundaries; Summer scene mode shows authoring coordinates.")
                    .clicked()
                {
                    state.editor_layout.chunk_grid_visible = !state.editor_layout.chunk_grid_visible;
                    save_editor_layout(&mut state);
                }
                if ui
                    .selectable_label(state.editor_layout.tile_grid_visible, "Tiles")
                    .on_hover_text("Toggle the logical terrain-cell grid over the authored scene.")
                    .clicked()
                {
                    state.editor_layout.tile_grid_visible = !state.editor_layout.tile_grid_visible;
                    save_editor_layout(&mut state);
                }
                ui.separator();
                ui.small(format!("Layer: {:?}", state.active_layer));
            });
        });
    }

    if !state.focus_mode {
        egui::Panel::bottom("havenwild.status.chrome").show(&mut viewport_ui, |ui| {
            ui.horizontal(|ui| {
                if state.world_mode {
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
                        state.tool.label()
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
                ui.small(&state.message);
            });
        });
    }

    // Only the compact direct-tool rail and compact layer rail are permanent.
    // Feature-rich applications are overlay windows; docking never resizes the canvas.
    if !state.focus_mode && state.playtest.is_none() {
        draw_tool_rail(&mut viewport_ui, &mut state);
        draw_layer_rail(&mut viewport_ui, &mut state);
        draw_layer_panel(&mut viewport_ui, &mut state);
        draw_structural_panel(&mut viewport_ui, &mut state);
        draw_mapper_surface(&mut viewport_ui, &mut state);
        draw_world_generator_panel(ctx, &mut state);
        draw_chunk_manager_panel(ctx, &mut state);
        draw_asset_authority_panel(ctx, &mut state);
        draw_app_launcher(ctx, &mut state);
    } else {
        state.atlas_rect = egui::Rect::NOTHING;
        state.layers_rect = egui::Rect::NOTHING;
        state.structural_rect = egui::Rect::NOTHING;
        state.world_generator_rect = egui::Rect::NOTHING;
        state.chunk_manager_rect = egui::Rect::NOTHING;
        state.asset_authority_rect = egui::Rect::NOTHING;
        state.launcher_rect = egui::Rect::NOTHING;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(&mut viewport_ui, |ui| {
            draw_world(ui, &mut state);
        });

    // Selection popup is optional and floats above the canvas. It never reserves a
    // permanent inspector/sidebar and never changes the central layout rectangle.
    if state.properties_open && state.playtest.is_none() && !state.focus_mode {
        let mut open = true;
        egui::Window::new("Object properties · on demand")
            .id(egui::Id::new("havenwild.object.properties.floating"))
            .open(&mut open)
            .default_pos(egui::pos2(355.0, 100.0))
            .default_size(egui::vec2(305.0, 235.0))
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(object) = selected_object(&state).cloned() {
                    ui.strong(&object.label);
                    ui.small(format!("Instance: {} · {:?}", object.id, object.layer));
                    if let Some(part) = object.parts.first() {
                        ui.small(format!("Original: {}", part.source.source_asset));
                        ui.small(format!("Source rect: {:?} · visual footprint: {:?}",
                            part.source.source_rect, object.footprint));
                    }
                    ui.separator();
                    let mut position = object.anchor;
                    ui.horizontal(|ui| {
                        ui.label("X"); ui.add(egui::DragValue::new(&mut position[0]).speed(1.0));
                        ui.label("Y"); ui.add(egui::DragValue::new(&mut position[1]).speed(1.0));
                    });
                    if position != object.anchor {
                        move_selected_object_to_origin(&mut state, object.id.clone(), position);
                    }
                    ui.small("Visual bounds only. No unsupported collision or tree-trunk geometry is inferred.");
                    ui.horizontal(|ui| {
                        if ui.button("Find").clicked() {
                            focus_mapping_zone(&mut state,
                                [object.anchor[0] as usize, object.anchor[1] as usize],
                                [object.footprint[0] as usize, object.footprint[1] as usize]);
                        }
                        if ui.button("Remove instance").clicked() { remove_selected_object(&mut state); }
                    });
                } else { ui.label("Select a placed object on the canvas first."); }
            });
        if !open || state.selected_object_id.is_none() {
            state.properties_open = false;
        }
    }

    // No egui window-drag/resize overlay: native Windows owns these interactions.

    if state.evidence_open && !state.focus_mode && state.playtest.is_none() {
        let mut open = true;
        let evidence = egui::Window::new("Source / mapping information")
            .open(&mut open)
            .default_size(egui::vec2(520.0, 340.0))
            .show(ctx, |ui| {
                ui.label("ElizaWy/LPC Revised source art stays byte-identical. Mapping metadata and editor layout are separate from hydrated art.");
                draw_recovery_summary(ui, &state);
                ui.separator();
                let (classified, sprites, composites, unsupported, unmapped) =
                    state.terrain_mapper.counts();
                ui.label(format!(
                    "Current DG Grass/Void recipe: {classified}/16 · S={sprites} C={composites} X={unsupported} unmapped={unmapped}"
                ));
                ui.label(format!("Selected world: {:?}", state.selected_world));
                ui.small("The old Summer source mapping is no longer represented as 'lost' simply because the new 16-mask recipe is not yet certified.");
            });
        state.evidence_rect = evidence
            .map(|response| response.response.rect)
            .unwrap_or(egui::Rect::NOTHING);
        if !open {
            state.evidence_open = false;
        }
    } else {
        state.evidence_rect = egui::Rect::NOTHING;
    }

    if state.playtest.is_none() {
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

fn main() -> bevy::app::AppExit {
    // Native Windows decorations are mandatory until custom chrome has OS-level tests.
    let native_frame = true;
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
                        title: "Havenwild — Bevy Studio v0.8.1".into(),
                        decorations: native_frame,
                        resolution: (1440, 900).into(),
                        resizable: true,
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
            (hydrate_pending_source, draw_ui).chain(),
        )
        .run()
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
