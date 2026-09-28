//! Fresh independent ElizaWy-first world canvas / mapping seed.
//! One primary Bevy camera and one original-pixel atlas renderer. No PCC gates,
//! historical Bevy validation probes, alternative world renderers, or game claims.
mod atomic_file;
mod canvas_rulers;
mod document;
mod editor_layout;
mod elizawy_review;
mod playtest;
mod scene_v2;
mod semantic_lab;
mod source_catalog;
mod terrain;
mod terrain_mapper;
mod terrain_resolver;

use bevy::{image::ImagePlugin, prelude::*};
use bevy_egui::{
    egui, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, EguiTextureHandle,
    EguiUserTextures, PrimaryEguiContext,
};
use document::Scene;
use editor_layout::EditorLayout;
use elizawy_review::{ReviewBrowser, ReviewCandidate};
use forge_gui_chrome::{apply_creator_visuals, SurfaceDock, WindowSnapState};
use forge_gui_theme::{ForgeTheme, ForgeThemePreset};
use playtest::PlaySession;
use scene_v2::{LayerId, PaintCell, SceneCommand, SceneHistory, SceneV2, SourceBinding};
use semantic_lab::SemanticTerrainLab;
use source_catalog::SourceCatalog;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use terrain_mapper::{mask_meaning, TerrainMapper, TerrainRecipeSource, TerrainRecipeState};
use terrain_resolver::{all_vertex_coords, dirty_vertices_for_cell, resolve_semantic_lab};

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
            Self::Paint => "Paint source tile",
            Self::Erase => "Erase source tile",
            Self::Sample => "Sample source tile",
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
    legacy_base_path: PathBuf,
    active_layer: LayerId,
    playtest_layered: Option<SceneV2>,
    playtest: Option<PlaySession>,
    saved_editor_view: Option<(f32, egui::Vec2)>,
    document_path: PathBuf,
    source_scene_path: PathBuf,
    mapping_lab: MappingLabIndex,
    migration_preview: Option<SceneV2>,
    sheets: Vec<Sheet>,
    active_sheet: usize,
    selected_cell: [u32; 2],
    selected_extent: [u32; 2], // explicit user-chosen source-region size in 32px cells
    selected_world: [usize; 2],
    selection_active: bool,
    source_filter: String,
    source_family_filter: String,
    source_catalog: SourceCatalog,
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
    tool: CanvasTool,
    inspect_alpha: bool,
    focus_mode: bool,
    zoom: f32,
    pan: egui::Vec2,
    canvas_rect: egui::Rect,
    scene_rect: egui::Rect,
    canvas_controls_rect: egui::Rect,
    canvas_selection_rect: egui::Rect,
    atlas_rect: egui::Rect,
    evidence_rect: egui::Rect,
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Native Windows decorations are mandatory until custom chrome has OS-level tests.
    let native_frame = true;
    // One user-visible certification workspace. The generated scene is a checked-in
    // reference; all manual edits save separately and survive scene regeneration.
    let source_scene_path = root.join("content/scenes/elizawy_mapping_certification.scene.json");
    let document_path =
        root.join("content/scenes/derived/elizawy_mapping_certification.source_exact.draft.json");
    let load_path = if document_path.is_file() {
        &document_path
    } else {
        &source_scene_path
    };
    let scene = Scene::load(load_path)
        .unwrap_or_else(|error| panic!("Cannot open mapping certification scene: {error}"));
    let legacy_base_path = load_path.to_path_buf();
    let layered_path =
        root.join("content/scenes/derived/elizawy_mapping_certification.layered.draft.json");
    let layered_scene = if layered_path.is_file() {
        let doc = SceneV2::read_draft(&layered_path).unwrap_or_else(|error| {
            panic!("Cannot read existing layered edits without replacing them: {error}")
        });
        doc.verify_legacy_source(&legacy_base_path)
            .unwrap_or_else(|error| panic!("Existing layered edits refer to different v1 source bytes: {error}. Preserve the draft and reconcile explicitly."));
        doc
    } else {
        SceneV2::preview_import(&legacy_base_path)
            .unwrap_or_else(|error| panic!("Cannot start source-exact layered scene: {error}"))
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
    // The pinned source-authored River fixture is the opening scene; guessed M2D02-D
    // visual samples are retired, never painted as approved objects.
    let initial_zoom = 0.70;
    let initial_pan = egui::Vec2::ZERO;
    commands.insert_resource(Studio {
        scene,
        layered_scene,
        layered_history: SceneHistory::default(),
        layered_path,
        legacy_base_path,
        active_layer: LayerId::Objects,
        playtest_layered: None,
        playtest: None,
        saved_editor_view: None,
        document_path,
        source_scene_path,
        mapping_lab,
    migration_preview: None,
        sheets,
        active_sheet: 0,
        selected_cell: [0, 0],
        selected_extent: [1, 1],
        selected_world: [0, 0],
        selection_active: false,
        source_filter: String::new(),
        source_family_filter: "All".into(),
        source_catalog,
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
        tool: CanvasTool::Select,
        inspect_alpha: editor_layout.inspect_alpha,
        focus_mode: false,
        zoom: initial_zoom,
        pan: initial_pan,
        canvas_rect: egui::Rect::NOTHING,
        scene_rect: egui::Rect::NOTHING,
        canvas_controls_rect: egui::Rect::NOTHING,
        canvas_selection_rect: egui::Rect::NOTHING,
        atlas_rect: egui::Rect::NOTHING,
        evidence_rect: egui::Rect::NOTHING,
        dragging: None,
        undo: Vec::new(),
        redo: Vec::new(),
        dirty: false,
        message: "Original River mapping fixture: inspect or replace source tiles. Guessed trees/rocks retired; your older assembled draft remains preserved. No recipes certified.".into(),
        theme: ForgeTheme::from_preset(ForgeThemePreset::MidnightMint),
        creator_visuals_applied: false,
        native_frame,
        snap: WindowSnapState::default(),
        terrain_mapper,
        review_browser: ReviewBrowser::new(root.join(".forgepy/recovered_mapping/elizawy_tiled_review_registry.v1.json")),
        editor_layout,
        layout_path,
    });
}

fn source_uv(rect: [u32; 4], sheet: [u32; 2]) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(
            rect[0] as f32 / sheet[0] as f32,
            rect[1] as f32 / sheet[1] as f32,
        ),
        egui::pos2(
            (rect[0] + rect[2]) as f32 / sheet[0] as f32,
            (rect[1] + rect[3]) as f32 / sheet[1] as f32,
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
        || (state.selection_active && state.canvas_selection_rect.contains(pointer))
        || (state.atlas_open && state.atlas_rect.contains(pointer))
        || (state.evidence_open && state.evidence_rect.contains(pointer))
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
            state.selected_world = pos;
            state.selection_active = true;
            state.message = format!("Cleared {:?} visual layer only at ({}, {}). Original source and other layers preserved.", state.active_layer, pos[0], pos[1]);
        }
        Err(error) => state.message = format!("Layer erase refused: {error}"),
    }
}

fn selected_scene_source(state: &Studio, pos: [usize; 2]) -> Option<(String, [u32; 4])> {
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
    if rect[2] != 32 || rect[3] != 32 || rect[0] % 32 != 0 || rect[1] % 32 != 0 {
        state.message = "A multi-cell composition must be inspected with its exact source extent; no crop guessed.".into();
        return;
    }
    if let Some(sheet) = state.sheets.iter().position(|item| item.path == path) {
        state.active_sheet = sheet;
        state.selected_cell = [rect[0] / 32, rect[1] / 32];
        state.message = format!(
            "Selected original source {path} at ({}, {}).",
            pos[0], pos[1]
        );
    } else {
        state.pending_source_sheet = Some(path);
        state.message = "Loading selected original image on demand. Re-select its source cell to inspect the exact address.".into();
    }
}

fn save_scene(state: &mut Studio) {
    if !state.layered_history.is_dirty() {
        state.message = "Layered scene already saved; original ElizaWy assets unchanged.".into();
        return;
    }
    state.message = match state
        .layered_scene
        .save_editable_draft(&state.legacy_base_path, &state.layered_path)
    {
        Ok(()) => {
            state.layered_history.mark_saved();
            "Saved source-exact layered scene corrections. Original v1 fixture, source art and recipe authority remain unchanged.".into()
        }
        Err(error) => format!("Save layered draft failed; previous file preserved: {error}"),
    };
}

fn undo_scene_edit(state: &mut Studio) {
    if state.layered_history.can_undo() {
        state.layered_history.undo(&mut state.layered_scene);
        state.message = "Undid layered scene transaction".into();
    }
}

fn redo_scene_edit(state: &mut Studio) {
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
    match PlaySession::start(&source_exact_snapshot, suggested) {
        Ok(play) => {
            state.playtest_layered = Some(state.layered_scene.clone());
            state.saved_editor_view = Some((state.zoom, state.pan));
            state.zoom = state.zoom.max(0.8).min(1.25);
            state.dragging = None;
            state.playtest = Some(play);
            state.message = "PIE: walking original source-addressed Summer River snapshot. W/A/S/D or arrows, Esc/Stop exits. Provisional v1 terrain-only role collision; not production navigation certification.".into();
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
        // Existing complete v2 object parts remain explicit source addresses. There are
        // no automatically invented rects, outlines, shadows, or object compositions.
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
                let x = object.anchor[0] as f32 + part.offset[0] as f32 / 32.0;
                let y = object.anchor[1] as f32 + part.offset[1] as f32 / 32.0;
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

fn draw_world(ui: &mut egui::Ui, state: &mut Studio) {
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
        egui::pos2(canvas.right() - 202.0, canvas.top() + ruler_inset.y + 6.0),
        egui::vec2(202.0, 40.0),
    );
    state.canvas_selection_rect = egui::Rect::from_min_size(
        canvas.min + ruler_inset + egui::vec2(7.0, 7.0),
        egui::vec2(310.0, 58.0),
    );
    let painter = ui.painter().with_clip_rect(canvas);
    painter.rect_filled(canvas, 0.0, egui::Color32::from_rgb(20, 29, 27));
    if state.playtest.is_none() && response.hovered() {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        if wheel.abs() > 0.01 {
            state.zoom = (state.zoom * (wheel / 240.0).exp()).clamp(0.125, 6.0);
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
    if state.playtest.is_none() && state.selection_active {
        if let Some(selected) = state
            .scene
            .index(state.selected_world[0], state.selected_world[1])
        {
            let x = selected % state.scene.size[0];
            let y = selected / state.scene.size[0];
            let rect = if let Some(object) = state
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
    if state.playtest.is_none() && response.clicked_by(egui::PointerButton::Primary) {
        if let Some(pointer) = response.interact_pointer_pos() {
            if let Some(pos) = world_at(state, pointer) {
                state.selected_world = pos;
                state.selection_active = true;
                if state.dragging.is_none() {
                    match state.tool {
                        CanvasTool::Select => {}
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
                            if !locate_selected_visual_sample(state, pos) {
                                sample_canvas_source(state, pos);
                            }
                        }
                    }
                }
            }
        }
    }
    if state.playtest.is_none()
        && state.selection_active
        && response.hovered()
        && !ui.ctx().egui_wants_keyboard_input()
        && ui.input(|input| input.key_pressed(egui::Key::Delete))
    {
        let pos = state.selected_world;
        if !hide_selected_visual_sample(state, pos) {
            erase_source(state, pos);
        }
    }
    // These view controls are canvas chrome, not a separate dock or inspector.
    let control_pos = egui::pos2(canvas.right() - 196.0, canvas.top() + ruler_inset.y + 9.0);
    if state.playtest.is_none() {
        egui::Area::new(egui::Id::new("havenwild.canvas.view.controls"))
            .fixed_pos(control_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgba_unmultiplied(25, 33, 37, 232))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .small_button("Fit")
                                .on_hover_text("Fit authored River fixture to canvas")
                                .clicked()
                            {
                                let size = state.scene.size;
                                focus_mapping_zone(state, [0, 0], size);
                            }
                            if ui.small_button("+").clicked() {
                                state.zoom = (state.zoom * 1.25).min(6.0);
                            }
                            if ui.small_button("-").clicked() {
                                state.zoom = (state.zoom / 1.25).max(0.125);
                            }
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
                        ui.small("Debug player marker; v1 terrain-sheet Grass/MudBank passable; water, cliffs and unknown blocked. NOT certified collision.");
                    });
            });
    }
    if state.playtest.is_none() && state.selection_active {
        let selected = state.selected_world;
        let has_tile = selected_scene_source(state, selected).is_some();
        egui::Area::new(egui::Id::new("havenwild.canvas.selection.actions"))
            .fixed_pos(canvas.min + ruler_inset + egui::vec2(9.0, 9.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgba_unmultiplied(25, 33, 37, 235))
                    .show(ui, |ui| {
                        let visual = state
                            .mapping_lab
                            .visual_at(selected, &state.scene)
                            .map(|v| (v.label.clone(), v.purpose.clone()));
                        if let Some((label, purpose)) = &visual {
                            ui.small(format!(
                                "{label} · {purpose} · source-backed draft (not certified)"
                            ));
                        } else {
                            ui.small(format!(
                                "{:?} visual layer ({}, {}) · select to inspect source",
                                state.active_layer, selected[0], selected[1]
                            ));
                        }
                        ui.horizontal(|ui| {
                            if visual.is_some() {
                                if ui.small_button("Inspect original").clicked() {
                                    locate_selected_visual_sample(state, selected);
                                }
                                if ui.small_button("Remove object").clicked() {
                                    hide_selected_visual_sample(state, selected);
                                }
                            } else {
                                if ui
                                    .add_enabled(has_tile, egui::Button::new("Sample tile"))
                                    .clicked()
                                {
                                    sample_canvas_source(state, selected);
                                }
                                if ui
                                    .add_enabled(has_tile, egui::Button::new("Source info"))
                                    .clicked()
                                {
                                    locate_selected_canvas_source(state);
                                }
                                if ui
                                    .add_enabled(has_tile, egui::Button::new("Erase tile"))
                                    .clicked()
                                {
                                    erase_source(state, selected);
                                }
                            }
                        });
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
    let Some(index) = state
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
    if state.layered_history.is_dirty() {
        match state
            .layered_scene
            .save_editable_draft(&state.legacy_base_path, &state.layered_path)
        {
            Ok(()) => {
                state.layered_history.mark_saved();
                saved.push("source-exact layered scene draft");
            }
            Err(error) => errors.push(format!("layered canvas: {error}")),
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
    let mut hide = false;

    ui.horizontal_wrapped(|ui| {
        ui.strong("Advanced terrain mapping");
        ui.separator();
        for (dock, label) in [
            (SurfaceDock::Left, "Left"),
            (SurfaceDock::Right, "Right"),
            (SurfaceDock::Bottom, "Bottom"),
            (SurfaceDock::Floating, "Float"),
        ] {
            if ui.selectable_label(current == dock, label).clicked() {
                requested_dock = Some(dock);
            }
        }
        ui.separator();
        ui.checkbox(&mut requested_lock, "Lock");
        if ui.button("Hide").clicked() {
            hide = true;
        }
    });

    if requested_lock != locked {
        state.editor_layout.terrain_mapper.locked = requested_lock;
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
            "In pinned source-authored River: {} distinct Summer source cells used / {} recovered globally; {} approved object compositions; DG masks certified {}/16",
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
        .source_catalog
        .entries
        .iter()
        .filter(|entry| family == "All" || entry.path.split('/').next() == Some(family))
        .filter(|entry| filter.is_empty() || entry.path.to_ascii_lowercase().contains(&filter))
        .map(|entry| entry.path.clone())
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
            state.source_catalog_images,
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
    ui.horizontal(|ui| {
        ui.label("Region width (cells)");
        ui.add(egui::Slider::new(&mut state.selected_extent[0], 1..=12));
        ui.label("Height");
        ui.add(egui::Slider::new(&mut state.selected_extent[1], 1..=12));
        ui.small("No automatic object boundary inference");
    });
    let explicit_rect = selected_source_rect(state);
    ui.small(format!(
        "Explicit source rectangle: {},{} {}×{} original pixels.",
        explicit_rect[0], explicit_rect[1], explicit_rect[2], explicit_rect[3]
    ));
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
    ui.heading("DG recipe certification · Grass / Void");
    let (classified, sprites, composites, unsupported, unmapped) = state.terrain_mapper.counts();
    let certified = state.terrain_mapper.certified_count();
    ui.label(format!(
        "Current recipe: {classified}/16 classified · {certified}/16 certified · sprites {sprites} · composites {composites} · unsupported {unsupported} · unmapped {unmapped}"
    ));
    ui.small("This counter is intentionally narrower than the recovered Summer source map. It measures only the new dual-grid Grass/Void recipe family.");

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
        ui.colored_label(egui::Color32::LIGHT_RED, error);
        ui.small("Generate with: python tools\\export_elizawy_review_registry.py <archive.zip> --original-root .\\assets\\elizawy --strict-archive-sha");
        return;
    }
    let Some(registry) = &state.review_browser.registry else {
        ui.label("Registry not loaded; select Reload.");
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
    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                state.editor_layout.terrain_mapper_tab == "recipe",
                "Terrain rules (advanced)",
            )
            .clicked()
        {
            state.editor_layout.terrain_mapper_tab = "recipe".into();
            save_editor_layout(state);
        }
        if ui
            .selectable_label(
                state.editor_layout.terrain_mapper_tab == "review",
                "ElizaWy source matches",
            )
            .clicked()
        {
            state.editor_layout.terrain_mapper_tab = "review".into();
            save_editor_layout(state);
        }
    });
    ui.separator();
    if state.editor_layout.terrain_mapper_tab == "review" {
        egui::ScrollArea::vertical()
            .id_salt("elizawy.review.surface.scroll")
            .show(ui, |ui| draw_elizawy_review_browser(ui, state));
        return;
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
        state.editor_layout.terrain_mapper.dock = SurfaceDock::Right;
    }
    let dock = state.editor_layout.terrain_mapper.dock;
    let preferred = state.editor_layout.terrain_mapper.preferred_size;

    match dock {
        SurfaceDock::Left => {
            let response = egui::Panel::left("havenwild.terrain.mapper.left")
                .resizable(true)
                .min_size(300.0)
                .default_size(preferred[0].clamp(300.0, 640.0))
                .show(viewport_ui, |ui| draw_mapper_contents(ui, state, false));
            state.atlas_rect = response.response.rect;
            state.editor_layout.terrain_mapper.preferred_size[0] = response.response.rect.width();
        }
        SurfaceDock::Right => {
            let response = egui::Panel::right("havenwild.terrain.mapper.right")
                .resizable(true)
                .min_size(300.0)
                .default_size(preferred[0].clamp(300.0, 640.0))
                .show(viewport_ui, |ui| draw_mapper_contents(ui, state, false));
            state.atlas_rect = response.response.rect;
            state.editor_layout.terrain_mapper.preferred_size[0] = response.response.rect.width();
        }
        SurfaceDock::Bottom => {
            let response = egui::Panel::bottom("havenwild.terrain.mapper.bottom")
                .resizable(true)
                .min_size(230.0)
                .default_size(preferred[1].clamp(230.0, 620.0))
                .show(viewport_ui, |ui| draw_mapper_contents(ui, state, true));
            state.atlas_rect = response.response.rect;
            state.editor_layout.terrain_mapper.preferred_size[1] = response.response.rect.height();
        }
        SurfaceDock::Floating => {
            let mut open = true;
            let response = egui::Window::new("Terrain Mapper — original ElizaWy source")
                .id(egui::Id::new("havenwild.terrain.mapper.floating"))
                .open(&mut open)
                .default_pos(egui::pos2(42.0, 82.0))
                .default_size(egui::vec2(preferred[0].max(560.0), preferred[1].max(640.0)))
                .min_width(320.0)
                .min_height(260.0)
                .resizable(true)
                .show(&ctx, |ui| draw_mapper_contents(ui, state, false));
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
        SurfaceDock::Center => unreachable!(),
    }
}

fn draw_tool_rail(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    egui::Panel::left("havenwild.tool.rail")
        .resizable(false)
        .default_size(52.0)
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(27, 34, 39, 226)))
        .show(viewport_ui, |ui| {
            ui.set_min_width(52.0);
            ui.set_max_width(52.0);
            ui.add_space(7.0);
            for (tool, glyph, description) in [
                (CanvasTool::Select, "SEL", "Select source cell without changing it"),
                (CanvasTool::Paint, "PNT", "Place the selected 32px source tile"),
                (CanvasTool::Erase, "ERS", "Erase a source tile, retaining its semantic hint"),
                (CanvasTool::Sample, "PIP", "Pick a source tile from this scene"),
            ] {
                if ui.add_sized([43.0, 29.0], egui::Button::selectable(state.tool == tool, glyph))
                    .on_hover_text(description).clicked() {
                    state.tool = tool;
                    state.message = format!("{} tool. Original source image on selected v2 visual layer; source bytes remain unchanged.", tool.label());
                }
                ui.add_space(2.0);
            }
            ui.separator();
            ui.add_enabled(false, egui::Button::new("MOV")).on_hover_text("Complete object move is pending live object-instance selection and source-verified compositions");
            ui.add_enabled(false, egui::Button::new("REG")).on_hover_text("Multi-cell scene brush still pending; manual source-region selection is available in SRC");
            ui.separator();
            if ui.add_sized([43.0, 29.0], egui::Button::new("LYR"))
                .on_hover_text("Show or hide compact scene layer guide").clicked() {
                state.editor_layout.layers_open = !state.editor_layout.layers_open;
                save_editor_layout(state);
            }
            if ui.add_sized([43.0, 29.0], egui::Button::new("SRC"))
                .on_hover_text("Original ElizaWy source browser / advanced mapping").clicked() {
                state.atlas_open = !state.atlas_open;
                state.editor_layout.terrain_mapper.visible = state.atlas_open;
                save_editor_layout(state);
            }
        });
}

fn draw_layer_guide(viewport_ui: &mut egui::Ui, state: &mut Studio) {
    if !state.editor_layout.layers_open {
        return;
    }
    egui::Panel::left("havenwild.layers.compact")
        .resizable(false)
        .default_size(178.0)
        .frame(egui::Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(29, 37, 42, 239)))
        .show(viewport_ui, |ui| {
            ui.set_min_width(178.0);
            ui.set_max_width(178.0);
            ui.horizontal(|ui| {
                ui.strong("Layers");
                if ui.small_button("x").clicked() {
                    state.editor_layout.layers_open = false;
                    save_editor_layout(state);
                }
            });
            ui.separator();
            ui.small("Choose exactly where the ORIGINAL pixels are placed. Ground replacement is separate from transparent overlays.");
            for (id, label) in [
                (LayerId::Foreground, "Foreground"),
                (LayerId::Objects, "Objects / forage / reeds"),
                (LayerId::Structures, "Structures"),
                (LayerId::Elevation, "Elevation / cliff faces"),
                (LayerId::Water, "Water / FX"),
                (LayerId::TerrainDetails, "Terrain details / banks"),
                (LayerId::Ground, "Ground — terrain only"),
            ] {
                ui.selectable_value(&mut state.active_layer, id, label);
            }
            ui.small("All visual layers render in editor and PIE. No inferred collisions or DG certification.");
        });
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
    if state.playtest.is_none() && ctx.input(|input| input.key_pressed(egui::Key::F11)) {
        state.focus_mode = !state.focus_mode;
        if state.focus_mode {
            state.tool = CanvasTool::Select;
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
            if ui.button(if state.playtest.is_some() { "■ Stop PIE" } else { "▶ Play Scene" })
                .on_hover_text("Play the same authoring scene in a sandbox; scene files and mappings are never changed by PIE.").clicked() {
                if state.playtest.is_some() { stop_playtest(&mut state); } else { start_playtest(&mut state); }
            }
            if state.playtest.is_some() {
                ui.small("PLAYING | original-source art | W/A/S/D | Esc to stop");
            } else {
            ui.menu_button("File", |ui| {
                if ui.button("Save Scene     Ctrl+S").clicked() { save_scene(&mut state); }
                if ui.button("Save All (including advanced drafts)").clicked() { save_all_authored_state(&mut state); }
                ui.label("Original artwork remains read-only.");
            });
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(state.layered_history.can_undo(), egui::Button::new("Undo Scene Edit    Ctrl+Z")).clicked() { undo_scene_edit(&mut state); }
                if ui.add_enabled(state.layered_history.can_redo(), egui::Button::new("Redo Scene Edit    Ctrl+Y")).clicked() { redo_scene_edit(&mut state); }
                if ui.add_enabled(state.selection_active, egui::Button::new("Remove selected object / erase tile     Delete")).clicked() {
                    let pos = state.selected_world;
                    if !hide_selected_visual_sample(&mut state, pos) { erase_source(&mut state, pos); }
                }
            });
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut state.editor_layout.rulers_visible, "Show canvas rulers (tile coordinates)").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.editor_layout.mapping_status_visible, "Show reviewed-source markers (when present)").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.inspect_alpha, "Inspect selected tile transparency").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.editor_layout.layers_open, "Compact Layers guide").changed() { save_editor_layout(&mut state); }
                if ui.checkbox(&mut state.focus_mode, "Focus canvas     F11").changed() && state.focus_mode {
                    state.tool = CanvasTool::Select;
                }
            });
            ui.menu_button("Scene", |ui| {
                if ui.button("Save mapping-lab corrections").clicked() { save_scene(&mut state); }
                if ui.button("Fit original River fixture").clicked() {
                    let size = state.scene.size;
                    focus_mapping_zone(&mut state, [0, 0], size);
                }
                ui.small("Original v1 River is read-only; active corrections are sparse Scene V2 layer edits. No guessed object crops.");
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
                if ui.button("Preview separate v2 scene draft").clicked() {
                    if !state.scene.hidden_visual_samples.is_empty() {
                        state.message = "V2 import is not activated for removed assembled objects yet. Your v1 derived draft retains them; no data is discarded.".into();
                    } else if state.dirty {
                        state.message = "Save current v1 scene edits before taking a v2 migration preview.".into();
                    } else {
                        state.migration_preview = None;
                        let source = if state.document_path.is_file() { &state.document_path } else { &state.source_scene_path };
                        match SceneV2::preview_import(source) {
                            Ok(preview) => {
                                state.message = format!("V2 migration preview: {} cells, seven logical layers; original v1 remains untouched.", preview.legacy_base.len());
                                state.migration_preview = Some(preview);
                            }
                            Err(error) => state.message = format!("V2 migration preview refused: {error}"),
                        }
                    }
                }
                let mut create_draft = false;
                if let Some(preview) = state.migration_preview.as_ref() {
                    ui.small(format!("Ready: {} source cells; {} objects; source SHA-256 {}...",
                        preview.legacy_base.len(), preview.objects.len(), &preview.legacy_origin.source_sha256[..12]));
                    ui.small("Legacy one-time v2 import preview only; live layered edits use their own distinct derived draft.");
                    create_draft = ui.button("Create separate v2 draft").clicked();
                }
                if create_draft {
                    let source = if state.document_path.is_file() { state.document_path.clone() } else { state.source_scene_path.clone() };
                    let destination = SceneV2::draft_path(&state.source_scene_path);
                    if state.dirty {
                        state.message = "Unsaved v1 changes: save and regenerate the migration preview first.".into();
                    } else {
                        let result = state.migration_preview.as_ref().expect("preview visible")
                            .save_new_draft(&source, &destination);
                        match result {
                            Ok(()) => {
                                state.message = format!("Created {}. Original v1 unchanged; this preview is not yet the live renderer.", destination.display());
                                state.migration_preview = None;
                            }
                            Err(error) => state.message = format!("V2 draft not created: {error}"),
                        }
                    }
                }
                ui.separator();
                ui.add_enabled(false, egui::Button::new("Preview regeneration"));
                ui.add_enabled(false, egui::Button::new("Regenerate selection / scene"));
                ui.small("Unavailable: no authoritative scene generator/override adapter yet. No fake refresh.");
            });
            ui.menu_button("Tools", |ui| {
                if ui.button("ElizaWy source / advanced terrain mapping").clicked() {
                    state.atlas_open = !state.atlas_open;
                    state.editor_layout.terrain_mapper.visible = state.atlas_open;
                    save_editor_layout(&mut state);
                }
                if ui.button("Source / mapping information").clicked() { state.evidence_open = !state.evidence_open; }
                ui.add_enabled(false, egui::Button::new("Whole-scene Pixel mode (M2D03)"));
            });
            ui.menu_button("Help", |ui| {
                ui.label("Pinned original Summer River fixture; source PNGs are never reconstructed or modified.");
                ui.label("Click to select. Paint/erase source-addressed tiles explicitly; Save writes a new derived draft. Older assembled drafts remain untouched.");
                ui.label("Mapped source cells are evidence, NOT automatically certified DG recipes.");
                ui.label("Select is the safe default: clicking does not paint.");
                ui.label("PNT replaces one terrain cell; ERS/Delete removes a selected sample or clears a terrain cell.");
                ui.label("PIP samples a source cell. Middle/right drag pans.");
                ui.label("Layered objects and regeneration require scene v2.");
            });
            }
            ui.separator();
            ui.small(format!("{}{} · {}", state.scene.name, if state.layered_history.is_dirty() { " *" } else { "" },
                if state.playtest.is_some() { "PIE / source-snapshot" } else { state.tool.label() }));
            if state.focus_mode && ui.small_button("Exit Focus").clicked() { state.focus_mode = false; }
        });
    });

    if !state.focus_mode {
        egui::Panel::bottom("havenwild.status.chrome").show(&mut viewport_ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(if state.layered_history.is_dirty() {
                    "Layered draft *"
                } else {
                    "Scene saved"
                });
                ui.separator();
                ui.small(format!(
                    "Cell {},{}  ·  {}",
                    state.selected_world[0],
                    state.selected_world[1],
                    state.tool.label()
                ));
                ui.separator();
                ui.small(&state.message);
            });
        });
    }

    // Keep the tool rail and its optional 3-group layer guide outside canvas space.
    if !state.focus_mode && state.playtest.is_none() {
        draw_tool_rail(&mut viewport_ui, &mut state);
        draw_layer_guide(&mut viewport_ui, &mut state);
    }

    // Docked mapper panels are registered before the central canvas so they reserve
    // usable workspace instead of covering it. Floating keeps the same live state.
    if !state.focus_mode && state.playtest.is_none() {
        draw_mapper_surface(&mut viewport_ui, &mut state);
    } else {
        state.atlas_rect = egui::Rect::NOTHING;
    }

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(&mut viewport_ui, |ui| {
            draw_world(ui, &mut state);
        });

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
                if let Some(pos) = world_at(&state, pointer) {
                    let px = 32.0 * state.zoom;
                    let target = egui::Rect::from_min_size(
                        state.scene_rect.min + egui::vec2(pos[0] as f32 * px, pos[1] as f32 * px),
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
                if let Some(pos) = ctx
                    .pointer_hover_pos()
                    .and_then(|pointer| world_at(&state, pointer))
                {
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
                    file_path: format!("{}/assets/elizawy", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Havenwild — Bevy Studio".into(),
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
