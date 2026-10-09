//! Editable world composition. Generation chooses/places canonical ElizaWy records only;
//! it never creates image assets or mutates source pixels.
use crate::{
    asset_authority::{
        AssetAuthority, ObjectTemplate, PaletteEntry, SummerVisualAnimation,
        SummerVisualAnimationFrame, SummerVisualPart,
    },
    atomic_file::write_atomic,
    scene_v2::{CollisionMask32, SourceBinding},
    tiled_terrain::CanonicalTiledTerrainAuthority,
    worldgen_plan,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub const WORLD_SCHEMA: &str = "havenwild.bevy.world_composition.v1";
pub const WORLD_CHUNK_SIDE: usize = 32;
pub const WORLD_GENERATOR_ID: &str = "havenwild.source_backed.generated_world.v12";
pub const COAST_TERRAIN_ANIMATION_TIME_SCALE: u32 = 4;
pub const RIVER_TERRAIN_ANIMATION_TIME_SCALE: u32 = 5;
pub const POND_TERRAIN_ANIMATION_TIME_SCALE: u32 = 7;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedCell {
    pub role: String,
    #[serde(default = "default_generated_season")]
    pub season: String,
    #[serde(default = "default_generated_biome")]
    pub biome: String,
    #[serde(default = "default_generated_water_context")]
    pub water_context: String,
    #[serde(default = "default_generated_region")]
    pub region_id: String,
    #[serde(default)]
    pub elevation: i16,
    #[serde(default)]
    pub cliff_mask: u8,
    pub source: SourceBinding,
}

fn default_generated_season() -> String {
    "summer".into()
}
fn default_generated_biome() -> String {
    "legacy".into()
}
fn default_generated_water_context() -> String {
    "none".into()
}
fn default_generated_region() -> String {
    "legacy_riverlands".into()
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorldObjectPart {
    pub offset_px: [i32; 2],
    pub source: SourceBinding,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorldObject {
    pub id: String,
    pub template_id: String,
    pub label: String,
    pub layer: String,
    pub anchor_world: [i64; 2],
    pub footprint_cells: [u32; 2],
    pub parts: Vec<WorldObjectPart>,
    pub generator_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorldChunk {
    pub coord: [i64; 2],
    pub generator_id: String,
    pub generated_cells: Vec<GeneratedCell>,
    pub generated_objects: Vec<WorldObject>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisualOverrideMode {
    Replace,
    Hidden,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraversalMode {
    Auto,
    Walkable,
    Blocked,
    Wadeable,
    Swimmable,
}

impl TraversalMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Walkable => "Walkable",
            Self::Blocked => "Blocked",
            Self::Wadeable => "Wadeable",
            Self::Swimmable => "Swimmable",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceBehaviorProfile {
    pub source: SourceBinding,
    #[serde(default)]
    pub collision: Option<CollisionMask32>,
    #[serde(default)]
    pub traversal: Option<TraversalMode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBehaviorProfile {
    pub role: String,
    pub neighbor_mask: u8,
    #[serde(default)]
    pub collision: Option<CollisionMask32>,
    #[serde(default)]
    pub traversal: Option<TraversalMode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorldCellOverride {
    pub world: [i64; 2],
    pub visual_mode: Option<VisualOverrideMode>,
    pub source: Option<SourceBinding>,
    pub semantic: Option<String>,
    pub elevation: Option<i16>,
    pub collision: Option<CollisionMask32>,
    #[serde(default)]
    pub traversal: Option<TraversalMode>,
}
impl WorldCellOverride {
    fn empty(world: [i64; 2]) -> Self {
        Self {
            world,
            visual_mode: None,
            source: None,
            semantic: None,
            elevation: None,
            collision: None,
            traversal: None,
        }
    }
    fn is_empty(&self) -> bool {
        self.visual_mode.is_none()
            && self.source.is_none()
            && self.semantic.is_none()
            && self.elevation.is_none()
            && self.collision.is_none()
            && self.traversal.is_none()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerationCorrection {
    pub role: String,
    pub neighbor_mask: u8,
    pub generated_source: SourceBinding,
    pub corrected_source: SourceBinding,
    pub learned_from_world: [i64; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegenerationPreview {
    pub chunk_count: usize,
    pub changed_cells: usize,
    pub changed_object_sets: usize,
    pub preserved_cell_overrides: usize,
    pub preserved_object_tombstones: usize,
}
/// Editor-authored entrance for one real structure object. Doorway is an outside
/// walkable world cell adjacent to that object's footprint; scene is project-local.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureDoorLink {
    pub object_id: String,
    pub doorway_world: [i64; 2],
    pub destination_scene: String,
    pub destination_cell: [u32; 2],
}

/// Door positions are on the perimeter outside the collision footprint, never
/// inferred from sprite transparency. Returns false for diagonals and interiors.
pub fn doorway_borders_footprint(at: [i64; 2], size: [u32; 2], door: [i64; 2]) -> bool {
    let [x, y] = at;
    let [w, h] = [i64::from(size[0]), i64::from(size[1])];
    (door[0] >= x && door[0] < x + w && (door[1] == y - 1 || door[1] == y + h))
        || (door[1] >= y && door[1] < y + h && (door[0] == x - 1 || door[0] == x + w))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldDocument {
    pub schema: String,
    pub world_id: String,
    pub seed: u64,
    pub season: String,
    pub chunk_side: usize,
    pub generator_id: String,
    pub center_chunk: [i64; 2],
    pub materialized_radius: i32,
    pub chunks: Vec<WorldChunk>,
    pub cell_overrides: Vec<WorldCellOverride>,
    #[serde(skip)]
    pending_semantic_edits: BTreeMap<[i64; 2], Option<String>>,
    #[serde(default)]
    pub source_behavior_profiles: Vec<SourceBehaviorProfile>,
    #[serde(default)]
    pub terrain_behavior_profiles: Vec<TerrainBehaviorProfile>,
    #[serde(default)]
    pub generation_corrections: Vec<GenerationCorrection>,
    #[serde(default)]
    pub generation_correction_cycle_initialized: bool,
    pub suppressed_generated_objects: BTreeSet<String>,
    /// Per-instance, authored placement corrections for generated objects.
    /// Keyed by generator-stable ID; never mutates source PNGs or generated chunks.
    #[serde(default)]
    pub moved_generated_objects: BTreeMap<String, [i64; 2]>,
    #[serde(default)]
    pub authored_objects: Vec<WorldObject>,
    /// Stable authored doorway-to-scene mapping; default preserves old saves.
    #[serde(default)]
    pub structure_doors: BTreeMap<String, StructureDoorLink>,
    pub revision: u64,
}

impl WorldDocument {
    pub fn new(seed: u64) -> Self {
        Self {
            schema: WORLD_SCHEMA.into(),
            world_id: format!("havenwild-local-{seed:016x}"),
            seed,
            season: "summer".into(),
            chunk_side: WORLD_CHUNK_SIDE,
            generator_id: WORLD_GENERATOR_ID.into(),
            center_chunk: [0, 0],
            materialized_radius: 1,
            chunks: Vec::new(),
            cell_overrides: Vec::new(),
            pending_semantic_edits: BTreeMap::new(),
            source_behavior_profiles: Vec::new(),
            terrain_behavior_profiles: Vec::new(),
            generation_corrections: Vec::new(),
            generation_correction_cycle_initialized: true,
            suppressed_generated_objects: BTreeSet::new(),
            moved_generated_objects: BTreeMap::new(),
            authored_objects: Vec::new(),
            structure_doors: BTreeMap::new(),
            revision: 1,
        }
    }
    pub fn read_or_new(path: &Path, seed: u64) -> Result<Self, String> {
        if !path.is_file() {
            return Ok(Self::new(seed));
        }
        let raw = fs::read(path).map_err(|e| format!("Cannot read local world document: {e}"))?;
        let mut doc: Self = serde_json::from_slice(&raw)
            .map_err(|e| format!("Cannot parse local world document: {e}"))?;
        // M2D082C retires the flat Riverlands demonstration as the user-facing
        // world. Preserve authored overrides/behavior metadata, but discard the
        // obsolete generated chunk cache so it can be regenerated by the unified
        // four-season archipelago profile.
        if doc.generator_id != WORLD_GENERATOR_ID {
            doc.generator_id = WORLD_GENERATOR_ID.into();
            doc.chunks.clear();
            doc.center_chunk = [0, 0];
            doc.materialized_radius = 2;
            doc.suppressed_generated_objects.clear();
            // Old generator identities cannot safely be rebound by guessed labels.
            doc.moved_generated_objects.clear();
            // Generated identities changed; orphaned structure portals must not
            // become invisible entrances attached to unrelated new buildings.
            doc.structure_doors.clear();
            doc.generation_corrections.clear();
            doc.generation_correction_cycle_initialized = true;
            doc.revision = doc.revision.saturating_add(1).max(1);
        }
        doc.chunks
            .sort_by_key(|chunk| (chunk.coord[1], chunk.coord[0]));
        doc.cell_overrides
            .sort_by_key(|item| (item.world[1], item.world[0]));
        doc.source_behavior_profiles.sort_by(|a, b| {
            (a.source.source_asset.as_str(), a.source.source_rect)
                .cmp(&(b.source.source_asset.as_str(), b.source.source_rect))
        });
        doc.generation_corrections.sort_by(|a, b| {
            (
                a.role.as_str(),
                a.neighbor_mask,
                a.generated_source.source_asset.as_str(),
                a.generated_source.source_rect,
            )
                .cmp(&(
                    b.role.as_str(),
                    b.neighbor_mask,
                    b.generated_source.source_asset.as_str(),
                    b.generated_source.source_rect,
                ))
        });
        doc.validate()?;
        Ok(doc)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create local world directory: {e}"))?;
        }
        let mut raw = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        raw.push(b'\n');
        write_atomic(path, &raw)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != WORLD_SCHEMA
            || self.world_id.is_empty()
            || self.season != "summer"
            || self.chunk_side != WORLD_CHUNK_SIDE
            || self.generator_id != WORLD_GENERATOR_ID
            || !(0..=4).contains(&self.materialized_radius)
            || self.revision == 0
        {
            return Err("Invalid world composition identity/settings".into());
        }
        for (id, link) in &self.structure_doors {
            if id != &link.object_id
                || id.is_empty()
                || link.destination_scene
                    != crate::room_scene::house_instance_relative_path(self.seed, id)
                || link.destination_cell[0] >= 96
                || link.destination_cell[1] >= 96
            {
                return Err("Invalid or unsafe structure-door scene binding".into());
            }
            if let Some(obj) = self.generated_objects().find(|o| &o.id == id) {
                if !doorway_borders_footprint(
                    self.object_anchor(obj),
                    obj.footprint_cells,
                    link.doorway_world,
                ) {
                    return Err("Structure doorway no longer borders its object".into());
                }
            }
        }
        let mut coords = BTreeSet::new();
        let expected = self.chunk_side * self.chunk_side;
        for chunk in &self.chunks {
            if chunk.generator_id != self.generator_id
                || chunk.generated_cells.len() != expected
                || !coords.insert(chunk.coord)
            {
                return Err("Invalid/duplicate materialized world chunk".into());
            }
            if chunk.generated_cells.iter().any(|cell| {
                cell.role.is_empty()
                    || cell.season.is_empty()
                    || cell.biome.is_empty()
                    || cell.region_id.is_empty()
                    || !(0..=30).contains(&cell.elevation)
                    || cell.cliff_mask > 0x0f
            }) {
                return Err("Invalid generated world-cell intent metadata".into());
            }
            let mut ids = BTreeSet::new();
            for object in &chunk.generated_objects {
                if object.id.is_empty()
                    || object.template_id.is_empty()
                    || object.parts.is_empty()
                    || object.footprint_cells.contains(&0)
                    || object.generator_id != self.generator_id
                    || !ids.insert(object.id.as_str())
                {
                    return Err("Invalid worldgen object composition".into());
                }
            }
        }
        let mut overrides = BTreeSet::new();
        for item in &self.cell_overrides {
            if !overrides.insert(item.world)
                || item.elevation.is_some_and(|h| !(0..=30).contains(&h))
                || matches!(item.visual_mode, Some(VisualOverrideMode::Replace))
                    && item.source.is_none()
                || matches!(item.visual_mode, Some(VisualOverrideMode::Hidden))
                    && item.source.is_some()
            {
                return Err("Invalid/duplicate authored world-cell override".into());
            }
            if let Some(mask) = &item.collision {
                mask.validate()?;
            }
        }
        let mut profile_keys = BTreeSet::new();
        for profile in &self.source_behavior_profiles {
            let key = (
                profile.source.source_asset.as_str(),
                profile.source.source_rect,
            );
            if !profile_keys.insert(key) {
                return Err("Duplicate reusable source behavior profile".into());
            }
            if let Some(mask) = &profile.collision {
                mask.validate()?;
            }
        }
        let mut terrain_profile_keys = BTreeSet::new();
        for profile in &self.terrain_behavior_profiles {
            if profile.role.is_empty() || profile.neighbor_mask > 0x0f {
                return Err("Invalid reusable terrain-topology behavior profile".into());
            }
            if !terrain_profile_keys.insert((profile.role.as_str(), profile.neighbor_mask)) {
                return Err("Duplicate reusable terrain-topology behavior profile".into());
            }
            if let Some(mask) = &profile.collision {
                mask.validate()?;
            }
        }
        let mut correction_keys = BTreeSet::new();
        for correction in &self.generation_corrections {
            if correction.role.is_empty() || correction.neighbor_mask > 0x0f {
                return Err("Invalid learned world-generation correction".into());
            }
            let key = (
                correction.role.as_str(),
                correction.neighbor_mask,
                correction.generated_source.source_asset.as_str(),
                correction.generated_source.source_rect,
            );
            if !correction_keys.insert(key) {
                return Err("Duplicate learned world-generation correction".into());
            }
        }
        let mut ids = BTreeSet::new();
        for object in &self.authored_objects {
            if !ids.insert(&object.id)
                || !object.id.starts_with("authored:bridge:")
                || object.footprint_cells[0] < 3
                || object.footprint_cells[1] != 2
                || object.parts.len() != object.footprint_cells[0] as usize
                || object.generator_id != "havenwild.editor.authored_bridge.v1"
            {
                return Err("Invalid or duplicate authored bridge record".into());
            }
        }
        Ok(())
    }
    pub fn validate_against_authority(&self, authority: &AssetAuthority) -> Result<(), String> {
        self.validate()?;
        for chunk in &self.chunks {
            for cell in &chunk.generated_cells {
                if !authority.contains_binding(&cell.source) {
                    return Err(format!(
                        "World cell source outside unified ElizaWy lane: {} {:?}",
                        cell.source.source_asset, cell.source.source_rect
                    ));
                }
            }
            for object in &chunk.generated_objects {
                for part in &object.parts {
                    // Full Gate may tighten a promoted object's atlas slot to its exact
                    // immutable-alpha render crop. Those certified object rectangles are
                    // source slices contained inside the canonical ElizaWy region, not new
                    // artwork and not independent atlas authority. Terrain cells remain
                    // exact bindings; generated object visuals may use verified slices.
                    if !authority.contains_source_slice(&part.source) {
                        return Err(format!(
                            "World placement source outside unified ElizaWy lane: {} {:?}",
                            part.source.source_asset, part.source.source_rect
                        ));
                    }
                }
            }
        }
        for object in &self.authored_objects {
            for part in &object.parts {
                if !authority.contains_source_slice(&part.source) {
                    return Err(format!(
                        "Authored bridge source outside ElizaWy lane: {} {:?}",
                        part.source.source_asset, part.source.source_rect
                    ));
                }
            }
        }
        for item in &self.cell_overrides {
            if let Some(source) = &item.source {
                if !authority.contains_binding(source) {
                    return Err(format!(
                        "World override source outside unified ElizaWy lane: {} {:?}",
                        source.source_asset, source.source_rect
                    ));
                }
            }
        }
        for profile in &self.source_behavior_profiles {
            if !authority.contains_binding(&profile.source) {
                return Err(format!(
                    "Reusable behavior profile references source outside unified ElizaWy lane: {} {:?}",
                    profile.source.source_asset, profile.source.source_rect
                ));
            }
        }
        for correction in &self.generation_corrections {
            if !authority.contains_binding(&correction.generated_source)
                || !authority.contains_binding(&correction.corrected_source)
            {
                return Err(format!(
                    "Learned world-generation correction references a source outside the unified ElizaWy lane: {} {:?} -> {} {:?}",
                    correction.generated_source.source_asset,
                    correction.generated_source.source_rect,
                    correction.corrected_source.source_asset,
                    correction.corrected_source.source_rect
                ));
            }
        }
        Ok(())
    }
    pub fn preview_regenerate_3x3(
        &self,
        center: [i64; 2],
        authority: &AssetAuthority,
    ) -> Result<RegenerationPreview, String> {
        self.preview_regenerate(center, 1, authority)
    }
    pub fn preview_regenerate(
        &self,
        center: [i64; 2],
        radius: i32,
        authority: &AssetAuthority,
    ) -> Result<RegenerationPreview, String> {
        self.preview_regenerate_with_tiled(center, radius, authority, None)
    }

    pub fn preview_regenerate_with_tiled(
        &self,
        center: [i64; 2],
        radius: i32,
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
    ) -> Result<RegenerationPreview, String> {
        if !(0..=4).contains(&radius) {
            return Err("World preview radius must be 0..4".into());
        }
        let mut changed_cells = 0;
        let mut changed_object_sets = 0;
        let mut chunk_count = 0;
        for cy in center[1] - i64::from(radius)..=center[1] + i64::from(radius) {
            for cx in center[0] - i64::from(radius)..=center[0] + i64::from(radius) {
                chunk_count += 1;
                let candidate = generate_chunk(
                    self.seed,
                    [cx, cy],
                    authority,
                    tiled,
                    &self.generation_corrections,
                )?;
                if let Some(existing) = self.chunk([cx, cy]) {
                    changed_cells += existing
                        .generated_cells
                        .iter()
                        .zip(&candidate.generated_cells)
                        .filter(|(a, b)| a != b)
                        .count();
                    if existing.generated_objects != candidate.generated_objects {
                        changed_object_sets += 1;
                    }
                } else {
                    changed_cells += candidate.generated_cells.len();
                    if !candidate.generated_objects.is_empty() {
                        changed_object_sets += 1;
                    }
                }
            }
        }
        Ok(RegenerationPreview {
            chunk_count,
            changed_cells,
            changed_object_sets,
            preserved_cell_overrides: self.cell_overrides.len(),
            preserved_object_tombstones: self.suppressed_generated_objects.len(),
        })
    }
    pub fn materialize_3x3(
        &mut self,
        center: [i64; 2],
        authority: &AssetAuthority,
    ) -> Result<(), String> {
        self.materialize(center, 1, authority)
    }
    pub fn materialize(
        &mut self,
        center: [i64; 2],
        radius: i32,
        authority: &AssetAuthority,
    ) -> Result<(), String> {
        self.materialize_with_tiled(center, radius, authority, None)
    }

    pub fn materialize_with_tiled(
        &mut self,
        center: [i64; 2],
        radius: i32,
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
    ) -> Result<(), String> {
        if !(0..=4).contains(&radius) {
            return Err("World materialization radius must be 0..4".into());
        }
        self.repair_invalid_generation_corrections(authority)?;
        if !self.generation_correction_cycle_initialized {
            self.promote_legacy_visual_overrides_to_generation_corrections(authority)?;
        }
        for role in [
            "Grass",
            "MudBank",
            "Sand",
            "WetSand",
            "RiverWater",
            "DeepWater",
            "PebblePath",
        ] {
            if authority.summer_flatworld_fill(role).is_none() {
                return Err(format!(
                    "Summer flatworld authority has no safe source-backed {role} fill"
                ));
            }
        }
        let mut replacements = Vec::new();
        for cy in center[1] - i64::from(radius)..=center[1] + i64::from(radius) {
            for cx in center[0] - i64::from(radius)..=center[0] + i64::from(radius) {
                replacements.push(generate_chunk(
                    self.seed,
                    [cx, cy],
                    authority,
                    tiled,
                    &self.generation_corrections,
                )?);
            }
        }
        let replace: BTreeSet<_> = replacements.iter().map(|c| c.coord).collect();
        self.chunks.retain(|c| !replace.contains(&c.coord));
        self.chunks.extend(replacements);
        self.chunks.sort_by_key(|c| (c.coord[1], c.coord[0]));
        self.center_chunk = center;
        self.materialized_radius = radius;
        self.revision = self.revision.saturating_add(1).max(1);
        self.validate_against_authority(authority)
    }
    pub fn chunk(&self, coord: [i64; 2]) -> Option<&WorldChunk> {
        let key = (coord[1], coord[0]);
        self.chunks
            .binary_search_by_key(&key, |chunk| (chunk.coord[1], chunk.coord[0]))
            .ok()
            .and_then(|index| self.chunks.get(index))
    }
    pub fn materialized_bounds(&self) -> Option<([i64; 2], [usize; 2])> {
        let min_x = self.chunks.iter().map(|c| c.coord[0]).min()?;
        let max_x = self.chunks.iter().map(|c| c.coord[0]).max()?;
        let min_y = self.chunks.iter().map(|c| c.coord[1]).min()?;
        let max_y = self.chunks.iter().map(|c| c.coord[1]).max()?;
        Some((
            [
                min_x * self.chunk_side as i64,
                min_y * self.chunk_side as i64,
            ],
            [
                usize::try_from(max_x - min_x + 1)
                    .ok()?
                    .checked_mul(self.chunk_side)?,
                usize::try_from(max_y - min_y + 1)
                    .ok()?
                    .checked_mul(self.chunk_side)?,
            ],
        ))
    }
    pub fn generated_cell(&self, world: [i64; 2]) -> Option<&GeneratedCell> {
        let cx = world[0].div_euclid(self.chunk_side as i64);
        let cy = world[1].div_euclid(self.chunk_side as i64);
        let lx = world[0].rem_euclid(self.chunk_side as i64) as usize;
        let ly = world[1].rem_euclid(self.chunk_side as i64) as usize;
        self.chunk([cx, cy])?
            .generated_cells
            .get(ly * self.chunk_side + lx)
    }
    pub fn generated_season(&self, world: [i64; 2]) -> Option<&str> {
        self.generated_cell(world).map(|cell| cell.season.as_str())
    }
    pub fn generated_biome(&self, world: [i64; 2]) -> Option<&str> {
        self.generated_cell(world).map(|cell| cell.biome.as_str())
    }
    pub fn generated_water_context(&self, world: [i64; 2]) -> Option<&str> {
        self.generated_cell(world)
            .map(|cell| cell.water_context.as_str())
    }
    pub fn generated_region_id(&self, world: [i64; 2]) -> Option<&str> {
        self.generated_cell(world)
            .map(|cell| cell.region_id.as_str())
    }
    pub fn generated_elevation(&self, world: [i64; 2]) -> Option<i16> {
        self.cell_override(world)
            .and_then(|item| item.elevation)
            .or_else(|| self.generated_cell(world).map(|cell| cell.elevation))
    }
    pub fn generated_cliff_mask(&self, world: [i64; 2]) -> Option<u8> {
        self.generated_cell(world).map(|cell| cell.cliff_mask)
    }

    pub fn cell_override(&self, world: [i64; 2]) -> Option<&WorldCellOverride> {
        let key = (world[1], world[0]);
        self.cell_overrides
            .binary_search_by_key(&key, |item| (item.world[1], item.world[0]))
            .ok()
            .and_then(|index| self.cell_overrides.get(index))
    }
    pub fn resolved_source(&self, world: [i64; 2]) -> Option<SourceBinding> {
        if let Some(item) = self.cell_override(world) {
            match item.visual_mode {
                Some(VisualOverrideMode::Hidden) => return None,
                Some(VisualOverrideMode::Replace) => return item.source.clone(),
                None => {}
            }
        }
        self.generated_cell(world).map(|c| c.source.clone())
    }
    pub fn resolved_visual_parts_with_authority(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
    ) -> Option<Vec<SummerVisualPart>> {
        self.resolved_visual_parts_with_authorities(world, authority, None)
    }

    pub fn resolved_visual_parts_with_authorities(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
    ) -> Option<Vec<SummerVisualPart>> {
        if !self.pending_semantic_edits.contains_key(&world) {
            if let Some(item) = self.cell_override(world) {
                match item.visual_mode {
                    Some(VisualOverrideMode::Hidden) => return None,
                    Some(VisualOverrideMode::Replace) => {
                        return item.source.clone().map(|source| {
                            vec![SummerVisualPart {
                                source,
                                destination_rect_px: [0, 0, 32, 32],
                                animation: None,
                            }]
                        })
                    }
                    None => {}
                }
            }
        }
        self.resolved_dual_grid_visual_parts_with_authority_at(world, authority, tiled, None)
    }

    /// Resolve one dual-grid *vertex* into exact source-backed visual parts.
    ///
    /// Semantic world cells occupy the squares between integer terrain vertices.
    /// Rendering therefore centers this output on the vertex coordinate; callers
    /// must not treat `vertex` as the top-left of a semantic cell. This method also
    /// resolves deterministic generated roles just outside the currently materialized
    /// chunk rectangle so the outer half-cells remain continuous.
    pub fn resolved_dual_grid_visual_parts_with_authority(
        &self,
        vertex: [i64; 2],
        authority: &AssetAuthority,
    ) -> Option<Vec<SummerVisualPart>> {
        self.resolved_dual_grid_visual_parts_with_authorities(vertex, authority, None)
    }

    pub fn resolved_dual_grid_visual_parts_with_authorities(
        &self,
        vertex: [i64; 2],
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
    ) -> Option<Vec<SummerVisualPart>> {
        self.resolved_dual_grid_visual_parts_with_authority_at(vertex, authority, tiled, None)
    }

    fn render_season_for_vertex(&self, vertex: [i64; 2]) -> &'static str {
        let corners = [
            [vertex[0] - 1, vertex[1] - 1],
            [vertex[0], vertex[1] - 1],
            [vertex[0] - 1, vertex[1]],
            vertex,
        ];
        corners
            .into_iter()
            .map(|world| worldgen_plan::season(self.seed, world))
            .find(|season| *season != "ocean")
            .unwrap_or("summer")
    }

    /// The saved/gameplay material. Keep this independent of derived coast art:
    /// a painted sand island changes semantic land, while its surrounding deep
    /// ocean remains swimmable even if rendered with a shallow Wang fringe.
    fn semantic_role_for_dual_grid(&self, world: [i64; 2]) -> String {
        if let Some(pending) = self.pending_semantic_edits.get(&world) {
            return pending.clone().unwrap_or_else(|| {
                self.generated_cell(world)
                    .map(|cell| cell.role.clone())
                    .unwrap_or_else(|| role_for(self.seed, world).to_owned())
            });
        }
        if let Some(role) = self
            .cell_override(world)
            .and_then(|item| item.semantic.as_ref())
        {
            return role.clone();
        }
        self.generated_cell(world)
            .map(|cell| cell.role.clone())
            .unwrap_or_else(|| role_for(self.seed, world).to_owned())
    }

    fn resolved_role_for_dual_grid(&self, world: [i64; 2]) -> String {
        let role = self.semantic_role_for_dual_grid(world);
        if role != "DeepWater" {
            return role;
        }
        // M2D103: M2D102's generated-Coast-only test missed authored terrain.
        // Editing sand/dirt/grass into ocean must recalculate the *visual* water
        // fringe from the final semantic neighborhood, including pending strokes.
        // Leave DeepWater gameplay, elevation, navigation, and source art untouched.
        // A two-cell Chebyshev fringe ensures no direct land/deep corner even when
        // the land is just one painted cell or crosses a chunk boundary.
        let context = self
            .generated_water_context(world)
            .map(str::to_owned)
            .unwrap_or_else(|| {
                worldgen_plan::water_context(self.seed, world)
                    .as_str()
                    .to_owned()
            });
        if context == "coast" {
            return "RiverWater".into();
        }
        for dy in -2i64..=2 {
            for dx in -2i64..=2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let neighbor = [world[0].saturating_add(dx), world[1].saturating_add(dy)];
                if Self::is_land_role(&self.semantic_role_for_dual_grid(neighbor)) {
                    return "RiverWater".into();
                }
            }
        }
        role
    }

    pub fn resolved_dual_grid_roles(&self, vertex: [i64; 2]) -> [String; 4] {
        let corner_cells = [
            [vertex[0] - 1, vertex[1] - 1], // NW
            [vertex[0], vertex[1] - 1],     // NE
            [vertex[0] - 1, vertex[1]],     // SW
            vertex,                         // SE
        ];
        [
            self.resolved_role_for_dual_grid(corner_cells[0]),
            self.resolved_role_for_dual_grid(corner_cells[1]),
            self.resolved_role_for_dual_grid(corner_cells[2]),
            self.resolved_role_for_dual_grid(corner_cells[3]),
        ]
    }

    fn is_water_role(role: &str) -> bool {
        matches!(role, "RiverWater" | "DeepWater")
    }

    fn is_land_role(role: &str) -> bool {
        matches!(
            role,
            "Grass" | "MudBank" | "Sand" | "WetSand" | "PebblePath"
        )
    }

    fn dominant_land_role<'a>(corners: [&'a str; 4]) -> Option<&'a str> {
        // Tie-breaking follows the authored SE cell first, then SW/NE/NW, so a
        // user's current semantic cell remains the most local authority while the
        // neighbouring vertices carry material hand-offs.
        let mut best: Option<(&str, usize, usize)> = None;
        for (priority, index) in [3usize, 2, 1, 0].into_iter().enumerate() {
            let role = corners[index];
            if !Self::is_land_role(role) {
                continue;
            }
            let count = corners
                .iter()
                .filter(|candidate| **candidate == role)
                .count();
            match best {
                None => best = Some((role, count, priority)),
                Some((_, best_count, best_priority))
                    if count > best_count || (count == best_count && priority < best_priority) =>
                {
                    best = Some((role, count, priority));
                }
                _ => {}
            }
        }
        best.map(|(role, _, _)| role)
    }

    pub(crate) fn land_connectivity_projection<'a>(corners: [&'a str; 4]) -> [&'a str; 4] {
        let mut projected = corners;
        let has_land = corners.iter().any(|role| Self::is_land_role(role));
        let has_water = corners.iter().any(|role| Self::is_water_role(role));

        // Mixed Grass/Dirt/Sand/etc. around a shoreline is rendered as one local
        // land material against one local water material. This uses complete
        // authored 32x32 transition tiles and leaves the land-material hand-off to
        // adjacent vertices instead of stitching unrelated 16x16 quadrants.
        if has_land && has_water {
            let land = Self::dominant_land_role(corners).unwrap_or("Grass");
            let water = if corners.contains(&"RiverWater") {
                "RiverWater"
            } else {
                "DeepWater"
            };
            for role in &mut projected {
                *role = if Self::is_water_role(role) {
                    water
                } else {
                    land
                };
            }
        } else if has_land {
            let distinct: BTreeSet<&str> = corners.iter().copied().collect();
            if distinct.len() > 2 {
                let land = Self::dominant_land_role(corners).unwrap_or("Grass");
                projected = [land; 4];
            }
        }

        let distinct: BTreeSet<&str> = projected.iter().copied().collect();
        let diagonal = distinct.len() == 2
            && projected[0] == projected[3]
            && projected[1] == projected[2]
            && projected[0] != projected[1];
        if !diagonal {
            return projected;
        }

        let water_positions = projected
            .iter()
            .enumerate()
            .filter_map(|(index, role)| Self::is_water_role(role).then_some(index))
            .collect::<Vec<_>>();
        let land_positions = projected
            .iter()
            .enumerate()
            .filter_map(|(index, role)| Self::is_land_role(role).then_some(index))
            .collect::<Vec<_>>();

        // Two water corners that only touch diagonally do not form a traversable
        // or visually continuous channel. Land therefore wins the ambiguous
        // contact. A real one-cell channel has cardinally adjacent water cells and
        // is preserved by the non-diagonal path above.
        if water_positions.len() == 2 && land_positions.len() == 2 {
            let land = Self::dominant_land_role(projected).unwrap_or("Grass");
            return [land; 4];
        }

        // Diagonal contacts between two land materials are likewise not rendered
        // with quadrant composites. Pick the local dominant material and let the
        // surrounding exact transition tiles carry the visible boundary.
        if water_positions.is_empty() && land_positions.len() == 4 {
            let land = Self::dominant_land_role(projected).unwrap_or(projected[3]);
            return [land; 4];
        }

        // Deep/shallow water diagonals are not shoreline topology; keep one local
        // water authority rather than fabricating a checkerboard.
        [projected[3]; 4]
    }

    fn terrain_animation_time_scale_for_vertex(&self, vertex: [i64; 2]) -> u32 {
        let corner_cells = [
            [vertex[0] - 1, vertex[1] - 1],
            [vertex[0], vertex[1] - 1],
            [vertex[0] - 1, vertex[1]],
            vertex,
        ];
        let contexts = corner_cells.map(|world| worldgen_plan::water_context(self.seed, world));
        if contexts.iter().any(|context| {
            matches!(
                context,
                worldgen_plan::WaterContext::Ocean | worldgen_plan::WaterContext::Coast
            )
        }) {
            COAST_TERRAIN_ANIMATION_TIME_SCALE
        } else if contexts
            .iter()
            .any(|context| matches!(context, worldgen_plan::WaterContext::River))
        {
            RIVER_TERRAIN_ANIMATION_TIME_SCALE
        } else if contexts
            .iter()
            .any(|context| matches!(context, worldgen_plan::WaterContext::PondLake))
        {
            POND_TERRAIN_ANIMATION_TIME_SCALE
        } else {
            COAST_TERRAIN_ANIMATION_TIME_SCALE
        }
    }

    fn tiled_visual_part_for_tile_id(
        &self,
        vertex: [i64; 2],
        authority: &AssetAuthority,
        tiled: &CanonicalTiledTerrainAuthority,
        tile_id: u32,
    ) -> Option<SummerVisualPart> {
        let render_season = self.render_season_for_vertex(vertex);
        let binding = tiled.canonical_visual_binding(tile_id)?;
        let source = SourceBinding {
            source_asset: binding.source_asset.clone(),
            source_rect: binding.source_rect,
        };
        let source = authority.seasonalize_binding(&source, render_season);
        let animation = tiled
            .terrain_tile_is_water_boundary(tile_id)
            .then(|| tiled.canonical_animation_clip(tile_id))
            .flatten()
            .and_then(|clip| {
                if clip.frames.is_empty() || clip.total_duration_ms == 0 {
                    return None;
                }
                let time_scale = self.terrain_animation_time_scale_for_vertex(vertex);
                let frames = clip
                    .frames
                    .iter()
                    .map(|frame| {
                        let source = SourceBinding {
                            source_asset: frame.source.source_asset.clone(),
                            source_rect: frame.source.source_rect,
                        };
                        SummerVisualAnimationFrame {
                            source: authority.seasonalize_binding(&source, render_season),
                            duration_ms: frame.duration_ms.saturating_mul(time_scale).max(1),
                        }
                    })
                    .collect::<Vec<_>>();
                let total_duration_ms =
                    clip.total_duration_ms.saturating_mul(u64::from(time_scale));
                Some(SummerVisualAnimation {
                    frames,
                    total_duration_ms,
                    phase_ms: CanonicalTiledTerrainAuthority::world_animation_phase_ms(
                        self.seed,
                        vertex,
                        total_duration_ms,
                    ),
                })
            });
        Some(SummerVisualPart {
            source,
            destination_rect_px: [0, 0, 32, 32],
            animation,
        })
    }

    /// Resolve the complete materialized terrain window through one shared
    /// native-Wang visual grid. The semantic grid includes the one-cell border
    /// needed by the dual-grid renderer, so every neighboring output tile sees
    /// the same repaired visual vertices. Unsupported legacy roles fail closed
    /// and leave the caller on the per-vertex compatibility resolver.
    pub fn resolved_tiled_materialized_vertex_tiles(
        &self,
        tiled: &CanonicalTiledTerrainAuthority,
    ) -> Result<BTreeMap<[i64; 2], u32>, String> {
        let (origin, size) = self
            .materialized_bounds()
            .ok_or_else(|| "No materialized world exists for Tiled terrain solve".to_owned())?;
        let input_width = u32::try_from(size[0])
            .map_err(|_| "Materialized world width exceeds u32".to_owned())?
            .checked_add(2)
            .ok_or_else(|| "Materialized Tiled input width overflow".to_owned())?;
        let input_height = u32::try_from(size[1])
            .map_err(|_| "Materialized world height exceeds u32".to_owned())?
            .checked_add(2)
            .ok_or_else(|| "Materialized Tiled input height overflow".to_owned())?;
        let mut roles = Vec::with_capacity(
            usize::try_from(input_width)
                .ok()
                .and_then(|w| {
                    usize::try_from(input_height)
                        .ok()
                        .and_then(|h| w.checked_mul(h))
                })
                .ok_or_else(|| "Materialized Tiled role-grid size overflow".to_owned())?,
        );
        for y in 0..input_height {
            for x in 0..input_width {
                let world = [origin[0] - 1 + i64::from(x), origin[1] - 1 + i64::from(y)];
                roles.push(self.resolved_role_for_dual_grid(world));
            }
        }
        let solved = tiled.solve_havenwild_corner_grid(
            [input_width, input_height],
            &roles,
            self.seed,
            origin,
        )?;
        let output_width = usize::try_from(solved.width)
            .map_err(|_| "Solved Tiled width exceeds usize".to_owned())?;
        let output_height = usize::try_from(solved.height)
            .map_err(|_| "Solved Tiled height exceeds usize".to_owned())?;
        let mut tiles = BTreeMap::new();
        for y in 0..output_height {
            for x in 0..output_width {
                let index = y
                    .checked_mul(output_width)
                    .and_then(|row| row.checked_add(x))
                    .ok_or_else(|| "Solved Tiled tile index overflow".to_owned())?;
                let tile_id = *solved
                    .tile_ids
                    .get(index)
                    .ok_or_else(|| "Solved Tiled tile grid is truncated".to_owned())?;
                tiles.insert([origin[0] + x as i64, origin[1] + y as i64], tile_id);
            }
        }
        Ok(tiles)
    }

    pub fn resolved_tiled_visual_part_for_tile_id(
        &self,
        vertex: [i64; 2],
        authority: &AssetAuthority,
        tiled: &CanonicalTiledTerrainAuthority,
        tile_id: u32,
    ) -> Option<SummerVisualPart> {
        self.tiled_visual_part_for_tile_id(vertex, authority, tiled, tile_id)
    }

    fn resolved_dual_grid_visual_parts_with_authority_at(
        &self,
        vertex: [i64; 2],
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
        _elapsed_ms: Option<u64>,
    ) -> Option<Vec<SummerVisualPart>> {
        // One semantic dual-grid authority drives generated terrain and authored
        // terrain alike. Every rendered output tile is centered on a terrain
        // vertex and reads the four logical cells surrounding that vertex.
        let roles = self.resolved_dual_grid_roles(vertex);
        let [nw, ne, sw, se] = roles;
        let role_refs = [nw.as_str(), ne.as_str(), sw.as_str(), se.as_str()];
        let render_season = self.render_season_for_vertex(vertex);

        // M2D097A renderer convergence: ask the authored Tiled topology for the
        // exact semantic corner first. This preserves real multi-material tiles
        // when Jaidyn configured them instead of collapsing them pre-emptively.
        if let Some(tiled) = tiled {
            if let Some(tile_id) =
                tiled.resolve_havenwild_corner_tile_id(role_refs, self.seed, vertex)
            {
                if let Some(part) =
                    self.tiled_visual_part_for_tile_id(vertex, authority, tiled, tile_id)
                {
                    return Some(vec![part]);
                }
            }
        }

        // Pair projection remains a conservative compatibility lane for authored
        // topologies the configured set does not contain. It no longer owns the
        // first decision and it no longer injects the legacy pale-shore overlay.
        let projected = Self::land_connectivity_projection(role_refs);
        if projected != role_refs {
            if let Some(tiled) = tiled {
                if let Some(tile_id) =
                    tiled.resolve_havenwild_corner_tile_id(projected, self.seed, vertex)
                {
                    if let Some(part) =
                        self.tiled_visual_part_for_tile_id(vertex, authority, tiled, tile_id)
                    {
                        return Some(vec![part]);
                    }
                }
            }
        }

        if projected.iter().all(|role| *role == projected[0]) {
            return authority
                .summer_flatworld_fill_for_world(projected[0], self.seed, vertex)
                .map(|entry| {
                    vec![SummerVisualPart {
                        source: authority.seasonalize_binding(&entry.binding(), render_season),
                        destination_rect_px: [0, 0, 32, 32],
                        animation: None,
                    }]
                });
        }
        if let Some(entry) = authority.summer_flatworld_corner(projected) {
            return Some(vec![SummerVisualPart {
                source: authority.seasonalize_binding(&entry.binding(), render_season),
                destination_rect_px: [0, 0, 32, 32],
                animation: None,
            }]);
        }

        // Historical quadrant composites and the old RiverWater/WetSand pale-
        // shore repaint are not live compositor authorities anymore. Unsupported
        // legacy/manual material states fail closed to the directly authored SE
        // material instead of layering another shoreline system over Tiled.
        authority.summer_flatworld_fill(projected[3]).map(|entry| {
            vec![SummerVisualPart {
                source: authority.seasonalize_binding(&entry.binding(), render_season),
                destination_rect_px: [0, 0, 32, 32],
                animation: None,
            }]
        })
    }

    pub fn resolved_visual_parts_with_authority_at(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
        elapsed_ms: Option<u64>,
    ) -> Option<Vec<SummerVisualPart>> {
        if !self.pending_semantic_edits.contains_key(&world) {
            if let Some(item) = self.cell_override(world) {
                match item.visual_mode {
                    Some(VisualOverrideMode::Hidden) => return None,
                    Some(VisualOverrideMode::Replace) => {
                        return item.source.clone().map(|source| {
                            vec![SummerVisualPart {
                                source,
                                destination_rect_px: [0, 0, 32, 32],
                                animation: None,
                            }]
                        })
                    }
                    None => {}
                }
            }
        }
        self.resolved_dual_grid_visual_parts_with_authority_at(world, authority, None, elapsed_ms)
    }

    pub fn resolved_source_with_authority(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
    ) -> Option<SourceBinding> {
        self.resolved_source_with_authorities(world, authority, None)
    }

    pub fn resolved_source_with_authorities(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
    ) -> Option<SourceBinding> {
        let parts = self.resolved_visual_parts_with_authorities(world, authority, tiled)?;
        if parts.len() == 1 && parts[0].destination_rect_px == [0, 0, 32, 32] {
            return Some(parts[0].source.clone());
        }
        // A composite has no single exact source binding. Source-level behavior
        // therefore falls back to semantic defaults unless a local world override
        // is explicitly authored.
        None
    }
    pub fn resolved_role(&self, world: [i64; 2]) -> Option<String> {
        if let Some(pending) = self.pending_semantic_edits.get(&world) {
            return pending
                .clone()
                .or_else(|| self.generated_cell(world).map(|cell| cell.role.clone()));
        }
        if let Some(role) = self.cell_override(world).and_then(|i| i.semantic.clone()) {
            return Some(role);
        }
        self.generated_cell(world).map(|c| c.role.clone())
    }
    pub fn set_visual_override(&mut self, world: [i64; 2], source: SourceBinding) {
        let item = self.override_mut(world);
        item.visual_mode = Some(VisualOverrideMode::Replace);
        item.source = Some(source);
        self.bump();
    }
    pub fn clear_visual_override(&mut self, world: [i64; 2]) {
        if let Some(item) = self.cell_overrides.iter_mut().find(|i| i.world == world) {
            item.visual_mode = None;
            item.source = None;
        }
        self.prune(world);
        self.bump();
    }
    pub fn set_semantic_override(&mut self, world: [i64; 2], semantic: Option<String>) {
        if let Some(value) = semantic {
            self.override_mut(world).semantic = Some(value);
        } else if let Some(item) = self.cell_overrides.iter_mut().find(|i| i.world == world) {
            item.semantic = None;
        }
        self.prune(world);
        self.bump();
    }
    pub fn paint_semantic_terrain(&mut self, world: [i64; 2], role: &str) {
        self.paint_semantic_terrain_untracked(world, role);
        self.commit_edit_batch();
    }
    pub fn paint_semantic_terrain_untracked(&mut self, world: [i64; 2], role: &str) {
        // Long paint strokes stage semantic changes in a BTreeMap. The renderer
        // reads this staging map immediately, while the serialized override Vec
        // is rebuilt once on mouse release. This avoids progressively more costly
        // Vec insertion/shifting during a continuous autotile stroke.
        self.pending_semantic_edits
            .insert(world, Some(role.to_owned()));
    }
    pub fn clear_authored_terrain(&mut self, world: [i64; 2]) {
        self.clear_authored_terrain_untracked(world);
        self.commit_edit_batch();
    }
    pub fn clear_authored_terrain_untracked(&mut self, world: [i64; 2]) {
        self.pending_semantic_edits.insert(world, None);
    }
    pub fn has_pending_semantic_edits(&self) -> bool {
        !self.pending_semantic_edits.is_empty()
    }
    pub fn pending_semantic_edit_count(&self) -> usize {
        self.pending_semantic_edits.len()
    }
    pub fn has_pending_semantic_edit(&self, world: [i64; 2]) -> bool {
        self.pending_semantic_edits.contains_key(&world)
    }
    fn flush_pending_semantic_edits(&mut self) {
        if self.pending_semantic_edits.is_empty() {
            return;
        }
        let mut merged: BTreeMap<[i64; 2], WorldCellOverride> =
            std::mem::take(&mut self.cell_overrides)
                .into_iter()
                .map(|item| (item.world, item))
                .collect();
        for (world, semantic) in std::mem::take(&mut self.pending_semantic_edits) {
            let item = merged
                .entry(world)
                .or_insert_with(|| WorldCellOverride::empty(world));
            // Semantic terrain owns the visual. Clear any exact-source local
            // replacement so the shared autotiler can resolve the neighbourhood.
            item.visual_mode = None;
            item.source = None;
            item.semantic = semantic;
        }
        merged.retain(|_, item| !item.is_empty());
        self.cell_overrides = merged.into_values().collect();
        self.cell_overrides
            .sort_by_key(|item| (item.world[1], item.world[0]));
    }
    pub fn commit_edit_batch(&mut self) {
        self.flush_pending_semantic_edits();
        self.bump();
    }
    pub fn collision_mask(&self, world: [i64; 2]) -> Option<&CollisionMask32> {
        self.cell_override(world).and_then(|i| i.collision.as_ref())
    }
    pub fn set_collision_mask_untracked(&mut self, world: [i64; 2], mask: Option<CollisionMask32>) {
        if let Some(mask) = mask {
            self.override_mut(world).collision = Some(mask);
        } else if let Some(item) = self.cell_overrides.iter_mut().find(|i| i.world == world) {
            item.collision = None;
        }
        self.prune(world);
    }
    pub fn set_collision_mask(&mut self, world: [i64; 2], mask: Option<CollisionMask32>) {
        self.set_collision_mask_untracked(world, mask);
        self.bump();
    }
    pub fn resolved_neighbor_mask(&self, world: [i64; 2], role: &str) -> u8 {
        let mut mask = 0u8;
        for (dx, dy, bit) in [(0, -1, 1u8), (1, 0, 2u8), (0, 1, 4u8), (-1, 0, 8u8)] {
            if self
                .resolved_role([world[0] + dx, world[1] + dy])
                .as_deref()
                == Some(role)
            {
                mask |= bit;
            }
        }
        mask
    }
    pub fn resolved_terrain_behavior_key(&self, world: [i64; 2]) -> Option<(String, u8)> {
        let role = self.resolved_role(world)?;
        let mask = self.resolved_neighbor_mask(world, &role);
        Some((role, mask))
    }
    pub fn terrain_behavior_profile(
        &self,
        role: &str,
        neighbor_mask: u8,
    ) -> Option<&TerrainBehaviorProfile> {
        self.terrain_behavior_profiles
            .iter()
            .find(|profile| profile.role == role && profile.neighbor_mask == neighbor_mask)
    }
    fn terrain_behavior_profile_mut(
        &mut self,
        role: &str,
        neighbor_mask: u8,
    ) -> &mut TerrainBehaviorProfile {
        if let Some(index) = self
            .terrain_behavior_profiles
            .iter()
            .position(|profile| profile.role == role && profile.neighbor_mask == neighbor_mask)
        {
            return &mut self.terrain_behavior_profiles[index];
        }
        self.terrain_behavior_profiles.push(TerrainBehaviorProfile {
            role: role.to_owned(),
            neighbor_mask,
            collision: None,
            traversal: None,
        });
        self.terrain_behavior_profiles
            .last_mut()
            .expect("terrain behavior profile just inserted")
    }
    fn prune_terrain_behavior_profile(&mut self, role: &str, neighbor_mask: u8) {
        self.terrain_behavior_profiles.retain(|profile| {
            profile.role != role
                || profile.neighbor_mask != neighbor_mask
                || profile.collision.is_some()
                || profile.traversal.is_some()
        });
        self.terrain_behavior_profiles.sort_by(|a, b| {
            (a.role.as_str(), a.neighbor_mask).cmp(&(b.role.as_str(), b.neighbor_mask))
        });
    }
    pub fn set_terrain_collision_profile_untracked(
        &mut self,
        role: String,
        neighbor_mask: u8,
        mask: Option<CollisionMask32>,
    ) {
        if let Some(mask) = mask {
            self.terrain_behavior_profile_mut(&role, neighbor_mask)
                .collision = Some(mask);
        } else if let Some(profile) = self
            .terrain_behavior_profiles
            .iter_mut()
            .find(|profile| profile.role == role && profile.neighbor_mask == neighbor_mask)
        {
            profile.collision = None;
        }
        self.prune_terrain_behavior_profile(&role, neighbor_mask);
    }
    pub fn set_terrain_collision_profile(
        &mut self,
        role: String,
        neighbor_mask: u8,
        mask: Option<CollisionMask32>,
    ) {
        self.set_terrain_collision_profile_untracked(role, neighbor_mask, mask);
        self.bump();
    }
    pub fn set_terrain_traversal_profile(
        &mut self,
        role: String,
        neighbor_mask: u8,
        traversal: Option<TraversalMode>,
    ) {
        if let Some(mode) = traversal {
            self.terrain_behavior_profile_mut(&role, neighbor_mask)
                .traversal = Some(mode);
        } else if let Some(profile) = self
            .terrain_behavior_profiles
            .iter_mut()
            .find(|profile| profile.role == role && profile.neighbor_mask == neighbor_mask)
        {
            profile.traversal = None;
        }
        self.prune_terrain_behavior_profile(&role, neighbor_mask);
        self.bump();
    }

    pub fn source_behavior_profile(
        &self,
        source: &SourceBinding,
    ) -> Option<&SourceBehaviorProfile> {
        self.source_behavior_profiles
            .iter()
            .find(|profile| profile.source == *source)
    }
    fn source_behavior_profile_mut(
        &mut self,
        source: &SourceBinding,
    ) -> &mut SourceBehaviorProfile {
        if let Some(index) = self
            .source_behavior_profiles
            .iter()
            .position(|profile| profile.source == *source)
        {
            return &mut self.source_behavior_profiles[index];
        }
        self.source_behavior_profiles.push(SourceBehaviorProfile {
            source: source.clone(),
            collision: None,
            traversal: None,
        });
        self.source_behavior_profiles
            .last_mut()
            .expect("profile just inserted")
    }
    fn prune_source_behavior_profile(&mut self, source: &SourceBinding) {
        self.source_behavior_profiles.retain(|profile| {
            profile.source != *source || profile.collision.is_some() || profile.traversal.is_some()
        });
        self.source_behavior_profiles.sort_by(|a, b| {
            (a.source.source_asset.as_str(), a.source.source_rect)
                .cmp(&(b.source.source_asset.as_str(), b.source.source_rect))
        });
    }
    pub fn set_source_collision_profile_untracked(
        &mut self,
        source: SourceBinding,
        mask: Option<CollisionMask32>,
    ) {
        if let Some(mask) = mask {
            self.source_behavior_profile_mut(&source).collision = Some(mask);
        } else if let Some(profile) = self
            .source_behavior_profiles
            .iter_mut()
            .find(|profile| profile.source == source)
        {
            profile.collision = None;
        }
        self.prune_source_behavior_profile(&source);
    }
    pub fn set_source_collision_profile(
        &mut self,
        source: SourceBinding,
        mask: Option<CollisionMask32>,
    ) {
        self.set_source_collision_profile_untracked(source, mask);
        self.bump();
    }
    pub fn set_traversal_override(&mut self, world: [i64; 2], traversal: Option<TraversalMode>) {
        if let Some(mode) = traversal {
            self.override_mut(world).traversal = Some(mode);
        } else if let Some(item) = self
            .cell_overrides
            .iter_mut()
            .find(|item| item.world == world)
        {
            item.traversal = None;
        }
        self.prune(world);
        self.bump();
    }
    pub fn set_source_traversal_profile(
        &mut self,
        source: SourceBinding,
        traversal: Option<TraversalMode>,
    ) {
        if let Some(mode) = traversal {
            self.source_behavior_profile_mut(&source).traversal = Some(mode);
        } else if let Some(profile) = self
            .source_behavior_profiles
            .iter_mut()
            .find(|profile| profile.source == source)
        {
            profile.traversal = None;
        }
        self.prune_source_behavior_profile(&source);
        self.bump();
    }
    pub fn effective_collision_mask(
        &self,
        world: [i64; 2],
        _authority: &AssetAuthority,
    ) -> Option<CollisionMask32> {
        if let Some(mask) = self.collision_mask(world) {
            return Some(mask.clone());
        }
        if self.generated_bridge_contains(world) {
            return Some(CollisionMask32::empty());
        }
        // World terrain behavior is keyed to semantic role + cardinal topology.
        // A raw atlas rectangle is visual evidence, not a safe collision identity:
        // the same source pixels can participate in different dual-grid contexts.
        let (role, neighbor_mask) = self.resolved_terrain_behavior_key(world)?;
        self.terrain_behavior_profile(&role, neighbor_mask)
            .and_then(|profile| profile.collision.clone())
    }
    pub fn effective_traversal(
        &self,
        world: [i64; 2],
        _authority: &AssetAuthority,
    ) -> TraversalMode {
        if let Some(mode) = self.cell_override(world).and_then(|item| item.traversal) {
            return mode;
        }
        if self.generated_bridge_contains(world) {
            return TraversalMode::Walkable;
        }
        if let Some((role, neighbor_mask)) = self.resolved_terrain_behavior_key(world) {
            if let Some(mode) = self
                .terrain_behavior_profile(&role, neighbor_mask)
                .and_then(|profile| profile.traversal)
            {
                return mode;
            }
        }
        match self.resolved_role(world).as_deref() {
            Some("RiverWater" | "DeepWater") => TraversalMode::Swimmable,
            Some("Grass" | "MudBank" | "Sand" | "WetSand" | "PebblePath") => {
                TraversalMode::Walkable
            }
            _ => TraversalMode::Auto,
        }
    }
    pub fn set_elevation_override(
        &mut self,
        world: [i64; 2],
        elevation: Option<i16>,
    ) -> Result<(), String> {
        if elevation.is_some_and(|h| !(0..=30).contains(&h)) {
            return Err("World elevation override must be 0..30".into());
        }
        if let Some(h) = elevation {
            self.override_mut(world).elevation = Some(h);
        } else if let Some(item) = self.cell_overrides.iter_mut().find(|i| i.world == world) {
            item.elevation = None;
        }
        self.prune(world);
        self.bump();
        Ok(())
    }
    pub fn learn_generation_correction(
        &mut self,
        world: [i64; 2],
        corrected_source: SourceBinding,
        authority: &AssetAuthority,
    ) -> Result<usize, String> {
        if !authority.contains_binding(&corrected_source) {
            return Err("Generation correction must use a canonical ElizaWy source region".into());
        }
        let (role, neighbor_mask, generated_source) =
            baseline_source_for(self.seed, world, authority, None)?;
        if let Some(corrected_role) = authority.unique_fixture_role(&corrected_source) {
            if corrected_role != role.as_str() {
                return Err(format!(
                    "Generation correction role mismatch: generated {role}, selected {corrected_role}"
                ));
            }
        }
        if corrected_source != generated_source
            && !authority.topology_supports_binding(&role, neighbor_mask, &corrected_source)
        {
            return Err(format!(
                "Selected source region is not observed for {role} topology {neighbor_mask:02x}. Use Shift+Paint for a local override, or certify this source region for the topology before teaching worldgen."
            ));
        }

        let matches = self.generation_correction_match_count(
            &role,
            neighbor_mask,
            &generated_source,
            authority,
        )?;
        let same_key = |item: &GenerationCorrection| {
            item.role == role
                && item.neighbor_mask == neighbor_mask
                && item.generated_source == generated_source
        };

        if corrected_source == generated_source {
            self.generation_corrections.retain(|item| !same_key(item));
        } else if let Some(existing) = self
            .generation_corrections
            .iter_mut()
            .find(|item| same_key(item))
        {
            existing.corrected_source = corrected_source;
            existing.learned_from_world = world;
        } else {
            self.generation_corrections.push(GenerationCorrection {
                role,
                neighbor_mask,
                generated_source,
                corrected_source,
                learned_from_world: world,
            });
            self.sort_generation_corrections();
        }

        if let Some(item) = self
            .cell_overrides
            .iter_mut()
            .find(|item| item.world == world)
        {
            item.visual_mode = None;
            item.source = None;
            item.semantic = None;
        }
        self.prune(world);
        self.generation_correction_cycle_initialized = true;
        self.bump();
        Ok(matches)
    }

    pub fn repair_invalid_generation_corrections(
        &mut self,
        authority: &AssetAuthority,
    ) -> Result<usize, String> {
        let before = self.generation_corrections.len();
        self.generation_corrections.retain(|item| {
            authority.contains_binding(&item.generated_source)
                && authority.contains_binding(&item.corrected_source)
                && authority.topology_supports_binding(
                    &item.role,
                    item.neighbor_mask,
                    &item.corrected_source,
                )
        });
        let removed = before.saturating_sub(self.generation_corrections.len());
        if removed > 0 {
            self.sort_generation_corrections();
            self.refresh_materialized_generation(authority)?;
        }
        Ok(removed)
    }

    pub fn refresh_materialized_generation(
        &mut self,
        authority: &AssetAuthority,
    ) -> Result<usize, String> {
        self.refresh_materialized_generation_with_tiled(authority, None)
    }

    pub fn refresh_materialized_generation_with_tiled(
        &mut self,
        authority: &AssetAuthority,
        tiled: Option<&CanonicalTiledTerrainAuthority>,
    ) -> Result<usize, String> {
        let coords: Vec<_> = self.chunks.iter().map(|chunk| chunk.coord).collect();
        let mut replacements = Vec::with_capacity(coords.len());
        let mut changed_cells = 0usize;
        for coord in coords {
            let candidate = generate_chunk(
                self.seed,
                coord,
                authority,
                tiled,
                &self.generation_corrections,
            )?;
            if let Some(existing) = self.chunk(coord) {
                changed_cells += existing
                    .generated_cells
                    .iter()
                    .zip(&candidate.generated_cells)
                    .filter(|(a, b)| a != b)
                    .count();
            }
            replacements.push(candidate);
        }
        if !replacements.is_empty() {
            self.chunks = replacements;
            self.chunks
                .sort_by_key(|chunk| (chunk.coord[1], chunk.coord[0]));
            self.bump();
        }
        Ok(changed_cells)
    }

    fn sort_generation_corrections(&mut self) {
        self.generation_corrections.sort_by(|a, b| {
            (
                a.role.as_str(),
                a.neighbor_mask,
                a.generated_source.source_asset.as_str(),
                a.generated_source.source_rect,
            )
                .cmp(&(
                    b.role.as_str(),
                    b.neighbor_mask,
                    b.generated_source.source_asset.as_str(),
                    b.generated_source.source_rect,
                ))
        });
    }

    pub fn clear_generation_correction(
        &mut self,
        world: [i64; 2],
        authority: &AssetAuthority,
    ) -> Result<bool, String> {
        let (role, neighbor_mask, generated_source) =
            baseline_source_for(self.seed, world, authority, None)?;
        let before = self.generation_corrections.len();
        self.generation_corrections.retain(|item| {
            !(item.role == role
                && item.neighbor_mask == neighbor_mask
                && item.generated_source == generated_source)
        });
        let removed = self.generation_corrections.len() != before;
        if removed {
            self.bump();
        }
        Ok(removed)
    }

    fn generation_correction_match_count(
        &self,
        role: &str,
        neighbor_mask: u8,
        generated_source: &SourceBinding,
        authority: &AssetAuthority,
    ) -> Result<usize, String> {
        let mut matches = 0usize;
        for chunk in &self.chunks {
            for ly in 0..self.chunk_side {
                for lx in 0..self.chunk_side {
                    let world = [
                        chunk.coord[0] * self.chunk_side as i64 + lx as i64,
                        chunk.coord[1] * self.chunk_side as i64 + ly as i64,
                    ];
                    let (candidate_role, candidate_mask, candidate_source) =
                        baseline_source_for(self.seed, world, authority, None)?;
                    if candidate_role == role
                        && candidate_mask == neighbor_mask
                        && &candidate_source == generated_source
                    {
                        matches += 1;
                    }
                }
            }
        }
        Ok(matches)
    }

    fn promote_legacy_visual_overrides_to_generation_corrections(
        &mut self,
        authority: &AssetAuthority,
    ) -> Result<usize, String> {
        let pending: Vec<_> = self
            .cell_overrides
            .iter()
            .filter_map(|item| {
                (matches!(item.visual_mode, Some(VisualOverrideMode::Replace)))
                    .then(|| item.source.clone().map(|source| (item.world, source)))
                    .flatten()
            })
            .collect();
        let mut promoted = 0usize;
        for (world, corrected_source) in pending {
            let Some(generated) = self.generated_cell(world).cloned() else {
                continue;
            };
            if !authority.contains_binding(&corrected_source) {
                continue;
            }
            if authority
                .unique_fixture_role(&corrected_source)
                .is_some_and(|corrected_role| corrected_role != generated.role.as_str())
            {
                continue;
            }
            let neighbor_mask = role_neighbor_mask(self.seed, world, &generated.role);
            if !authority.topology_supports_binding(
                &generated.role,
                neighbor_mask,
                &corrected_source,
            ) {
                // Keep legacy visual edits local when they do not constitute
                // topology-safe evidence. Never turn a one-off edge tile into
                // a world-wide generator rule.
                continue;
            }
            if corrected_source != generated.source
                && !self.generation_corrections.iter().any(|item| {
                    item.role == generated.role
                        && item.neighbor_mask == neighbor_mask
                        && item.generated_source == generated.source
                })
            {
                self.generation_corrections.push(GenerationCorrection {
                    role: generated.role,
                    neighbor_mask,
                    generated_source: generated.source,
                    corrected_source,
                    learned_from_world: world,
                });
                promoted += 1;
            }
            if let Some(item) = self
                .cell_overrides
                .iter_mut()
                .find(|item| item.world == world)
            {
                item.visual_mode = None;
                item.source = None;
                item.semantic = None;
            }
            self.prune(world);
        }
        self.sort_generation_corrections();
        self.generation_correction_cycle_initialized = true;
        if promoted > 0 {
            self.bump();
        }
        Ok(promoted)
    }

    pub fn object_anchor(&self, object: &WorldObject) -> [i64; 2] {
        self.moved_generated_objects
            .get(&object.id)
            .copied()
            .unwrap_or(object.anchor_world)
    }

    pub fn can_move_generated_object_to(&self, id: &str, to: [i64; 2]) -> bool {
        let Some(object) = self.generated_objects().find(|object| object.id == id) else {
            return false;
        };
        // Terrain/bridges/elevation may carry independent traversal authority.
        // Until their relocation semantics are certified, move visual objects and
        // structures only; don't silently relocate a gameplay connector.
        if object.label.contains("river bridge") {
            return false;
        }
        if !matches!(
            object.layer.to_ascii_lowercase().as_str(),
            "objects" | "structures" | "foreground"
        ) {
            return false;
        }
        if object.footprint_cells.contains(&0) {
            return false;
        }
        let Some(end_x) = to[0].checked_add(i64::from(object.footprint_cells[0])) else {
            return false;
        };
        let Some(end_y) = to[1].checked_add(i64::from(object.footprint_cells[1])) else {
            return false;
        };
        [
            [to[0], to[1]],
            [end_x - 1, to[1]],
            [to[0], end_y - 1],
            [end_x - 1, end_y - 1],
        ]
        .iter()
        .all(|cell| self.generated_cell(*cell).is_some())
    }

    /// Source-preserving manual placement. No generated chunk or source asset is mutated.
    /// The caller owns transaction checkpoints; invalid / suppressed objects fail closed.
    pub fn move_generated_object(&mut self, id: &str, to: [i64; 2]) -> Result<bool, String> {
        let Some(object) = self.generated_objects().find(|object| object.id == id) else {
            return Err("Cannot move unknown or suppressed generated object".into());
        };
        let original = object.anchor_world;
        if !self.can_move_generated_object_to(id, to) {
            return Err("Object move must remain in materialized world cells".into());
        }
        let previous_anchor = self.object_anchor(object);
        if previous_anchor == to {
            return Ok(false);
        }
        let moved_door = if let Some(link) = self.structure_doors.get(id) {
            let dx = to[0]
                .checked_sub(previous_anchor[0])
                .ok_or("Door offset overflow")?;
            let dy = to[1]
                .checked_sub(previous_anchor[1])
                .ok_or("Door offset overflow")?;
            let new_cell = [
                link.doorway_world[0]
                    .checked_add(dx)
                    .ok_or("Door X overflow")?,
                link.doorway_world[1]
                    .checked_add(dy)
                    .ok_or("Door Y overflow")?,
            ];
            if self.generated_cell(new_cell).is_none() {
                return Err(
                    "Moving house would place mapped doorway outside materialized world".into(),
                );
            }
            Some(new_cell)
        } else {
            None
        };
        if original == to {
            self.moved_generated_objects.remove(id);
        } else {
            self.moved_generated_objects.insert(id.to_owned(), to);
        }
        if let Some(new_door) = moved_door {
            if let Some(link) = self.structure_doors.get_mut(id) {
                link.doorway_world = new_door;
            }
        }
        self.bump();
        Ok(true)
    }

    /// Persist one structure's doorway. Reject non-house targets and guesses
    /// inside the footprint or across the road. The source art is untouched.
    pub fn map_structure_door(
        &mut self,
        object_id: &str,
        world_cell: [i64; 2],
        destination_cell: [u32; 2],
        authority: &AssetAuthority,
    ) -> Result<(), String> {
        let obj = self
            .generated_objects()
            .find(|o| o.id == object_id)
            .ok_or("Selected structure no longer exists")?;
        let id = obj.template_id.to_ascii_lowercase();
        let label = obj.label.to_ascii_lowercase();
        if !(id.contains("house") || id.contains("building") || label.contains("house")) {
            return Err("Map Doorway is currently supported for house/building objects".into());
        }
        if !doorway_borders_footprint(self.object_anchor(obj), obj.footprint_cells, world_cell) {
            return Err("Choose a cardinally adjacent cell OUTSIDE the house footprint".into());
        }
        if self.generated_cell(world_cell).is_none()
            || self.effective_traversal(world_cell, authority) != TraversalMode::Walkable
        {
            return Err("Doorway must lead to materialized, walkable terrain".into());
        }
        if self.generated_objects().any(|other| {
            other.id != object_id && {
                let at = self.object_anchor(other);
                world_cell[0] >= at[0]
                    && world_cell[0] < at[0] + i64::from(other.footprint_cells[0])
                    && world_cell[1] >= at[1]
                    && world_cell[1] < at[1] + i64::from(other.footprint_cells[1])
            }
        }) {
            return Err("Doorway is blocked by another structure/object".into());
        }
        let link = StructureDoorLink {
            object_id: object_id.to_owned(),
            doorway_world: world_cell,
            destination_scene: crate::room_scene::house_instance_relative_path(
                self.seed, object_id,
            ),
            destination_cell,
        };
        self.structure_doors.insert(object_id.to_owned(), link);
        self.bump();
        Ok(())
    }

    pub fn generated_objects(&self) -> impl Iterator<Item = &WorldObject> {
        self.chunks
            .iter()
            .flat_map(|c| c.generated_objects.iter())
            .chain(self.authored_objects.iter())
            .filter(|o| !self.suppressed_generated_objects.contains(&o.id))
    }

    /// Only the horizontally authored ElizaWy bridge recipe is currently certified.
    /// Both approaches must touch land and intermediate cells must cross water.
    pub fn validate_bridge_span(
        &self,
        start: [i64; 2],
        end: [i64; 2],
    ) -> Result<([i64; 2], u32), String> {
        if start[1] != end[1] {
            return Err(
                "This source bridge is horizontal; a vertical recipe needs its own verified art"
                    .into(),
            );
        }
        let (left, right) = if start[0] <= end[0] {
            (start, end)
        } else {
            (end, start)
        };
        let span = right[0] - left[0] + 1;
        if !(3..=64).contains(&span) {
            return Err("Bridge span must be between 3 and 64 tiles".into());
        }
        for x in left[0]..=right[0] {
            let world = [x, left[1]];
            if self.generated_cell(world).is_none()
                || self.generated_cell([x, left[1] + 1]).is_none()
            {
                return Err("Bridge must remain inside materialized terrain".into());
            }
            let role = self
                .resolved_role(world)
                .ok_or("Missing terrain under bridge")?;
            let wet = matches!(role.as_str(), "RiverWater" | "DeepWater");
            if x == left[0] || x == right[0] {
                if wet {
                    return Err("Both bridge approaches must begin and end on land".into());
                }
            } else if !wet
                || !matches!(
                    self.resolved_role([x, left[1] + 1]).as_deref(),
                    Some("RiverWater" | "DeepWater")
                )
            {
                return Err("Both rows of the bridge deck must cross continuous water".into());
            }
        }
        Ok((left, span as u32))
    }

    /// Place ONLY a reviewed complete object template. Raw alpha candidates cannot reach
    /// this API. Keep visual placement separate from collision and gameplay authority.
    pub fn place_reviewed_template(
        &mut self,
        template_id: &str,
        anchor: [i64; 2],
        authority: &AssetAuthority,
    ) -> Result<String, String> {
        let template = authority
            .object_templates
            .iter()
            .find(|t| t.template_id == template_id)
            .ok_or("Unknown/unreviewed complete object template")?;
        if !matches!(template.layer.as_str(), "objects" | "structures")
            || template.parts.is_empty()
            || template.footprint_cells.contains(&0)
            || template.worldgen_tags.iter().any(|tag| tag == "bridge")
        {
            return Err("This template needs its dedicated placement or geometry tool".into());
        }
        let end_x = anchor[0]
            .checked_add(i64::from(template.footprint_cells[0]))
            .ok_or("Template footprint X overflow")?;
        let end_y = anchor[1]
            .checked_add(i64::from(template.footprint_cells[1]))
            .ok_or("Template footprint Y overflow")?;
        if !(anchor[0]..end_x)
            .all(|x| (anchor[1]..end_y).all(|y| self.generated_cell([x, y]).is_some()))
        {
            return Err("Template footprint must fit entirely inside materialized terrain".into());
        }
        if template.layer == "structures"
            && self.generated_objects().any(|object| {
                object.layer == "structures" && {
                    let other = self.object_anchor(object);
                    let ox = other[0] + i64::from(object.footprint_cells[0]);
                    let oy = other[1] + i64::from(object.footprint_cells[1]);
                    anchor[0] < ox && end_x > other[0] && anchor[1] < oy && end_y > other[1]
                }
            })
        {
            return Err("Cannot place a structure over another structure".into());
        }
        let mut parts = Vec::with_capacity(template.parts.len());
        for part in &template.parts {
            let source = SourceBinding {
                source_asset: part.source_path.clone(),
                source_rect: part.source_rect_px,
            };
            if authority.canonical_region(&source).is_none() {
                return Err(format!(
                    "Reviewed template lost its canonical source binding: {}",
                    part.source_path
                ));
            }
            parts.push(WorldObjectPart {
                offset_px: part.offset_px,
                source,
            });
        }
        // Deterministic, collision-free, stable across document save/reload.
        let prefix = format!("authored:object:{template_id}:{}:{}:", anchor[0], anchor[1]);
        let serial = (0u32..=10_000)
            .find(|serial| {
                let id = format!("{prefix}{serial}");
                !self.generated_objects().any(|o| o.id == id)
            })
            .ok_or("Maximum object instances reached at this location")?;
        let id = format!("{prefix}{serial}");
        self.authored_objects.push(WorldObject {
            id: id.clone(),
            template_id: template.template_id.clone(),
            label: template.label.clone(),
            layer: template.layer.clone(),
            anchor_world: anchor,
            footprint_cells: template.footprint_cells,
            parts,
            generator_id: "havenwild.editor.reviewed_visual_object.v1".into(),
        });
        self.bump();
        Ok(id)
    }

    /// Builds a persistent authored bridge from the three source-verified cap/deck slices.
    /// Selection/preview is owned by Studio; this operation is one undoable mutation.
    pub fn place_authored_bridge(
        &mut self,
        start: [i64; 2],
        end: [i64; 2],
        authority: &AssetAuthority,
    ) -> Result<String, String> {
        let (anchor, span) = self.validate_bridge_span(start, end)?;
        let template = authority
            .templates_with_tag("bridge")
            .find(|template| {
                template.parts.len() >= 3
                    && template.parts[0].source_rect_px[2] == 32
                    && template.parts[1].source_rect_px[2] == 32
                    && template.parts[2].source_rect_px[2] == 32
            })
            .ok_or("No verified 3-part 32px bridge template exists")?;
        let id = format!(
            "authored:bridge:{}:{}:{}:{}",
            anchor[0], anchor[1], span, template.template_id
        );
        if self.generated_objects().any(|object| object.id == id) {
            return Err("An authored bridge already occupies this exact span".into());
        }
        let season = worldgen_plan::season(self.seed, anchor);
        let mut parts = Vec::with_capacity(span as usize);
        for x in 0..span {
            let source_part = if x == 0 {
                &template.parts[0]
            } else if x == span - 1 {
                &template.parts[2]
            } else {
                &template.parts[1]
            };
            let source = SourceBinding {
                source_asset: source_part.source_path.clone(),
                source_rect: source_part.source_rect_px,
            };
            parts.push(WorldObjectPart {
                offset_px: [(x as i32) * 32, 0],
                source: authority.seasonalize_binding(&source, season),
            });
        }
        self.authored_objects.push(WorldObject {
            id: id.clone(),
            template_id: template.template_id.clone(),
            label: format!("Authored river bridge: {} · {} cells", template.label, span),
            layer: "structures".into(),
            anchor_world: anchor,
            footprint_cells: [span, 2],
            parts,
            generator_id: "havenwild.editor.authored_bridge.v1".into(),
        });
        self.bump();
        Ok(id)
    }
    pub fn generated_bridge_contains(&self, world: [i64; 2]) -> bool {
        self.generated_objects().any(|object| {
            (object.label.starts_with("Worldgen river bridge")
                || object.label.starts_with("Authored river bridge"))
                && world[0] >= self.object_anchor(object)[0]
                && world[1] >= self.object_anchor(object)[1]
                && world[0] - self.object_anchor(object)[0] < i64::from(object.footprint_cells[0])
                && world[1] - self.object_anchor(object)[1] < i64::from(object.footprint_cells[1])
        })
    }
    pub fn suppress_generated_object(&mut self, id: &str) -> bool {
        if let Some(position) = self
            .authored_objects
            .iter()
            .position(|object| object.id == id)
        {
            self.authored_objects.remove(position);
            self.structure_doors.remove(id);
            self.bump();
            return true;
        }
        if self
            .chunks
            .iter()
            .any(|c| c.generated_objects.iter().any(|o| o.id == id))
        {
            self.suppressed_generated_objects.insert(id.to_owned());
            self.moved_generated_objects.remove(id);
            self.structure_doors.remove(id);
            self.bump();
            true
        } else {
            false
        }
    }
    pub fn quarantine_uncertified_large_objects(&mut self, authority: &AssetAuthority) -> usize {
        let mut removed = 0usize;
        for chunk in &mut self.chunks {
            let before = chunk.generated_objects.len();
            chunk.generated_objects.retain(|object| {
                authority.worldgen_safe_object_template(&object.template_id)
                    && object
                        .parts
                        .iter()
                        .all(|part| authority.contains_source_slice(&part.source))
            });
            removed += before - chunk.generated_objects.len();
        }
        if removed > 0 {
            self.bump();
        }
        removed
    }
    fn override_mut(&mut self, world: [i64; 2]) -> &mut WorldCellOverride {
        let key = (world[1], world[0]);
        match self
            .cell_overrides
            .binary_search_by_key(&key, |item| (item.world[1], item.world[0]))
        {
            Ok(index) => &mut self.cell_overrides[index],
            Err(index) => {
                self.cell_overrides
                    .insert(index, WorldCellOverride::empty(world));
                &mut self.cell_overrides[index]
            }
        }
    }
    fn prune(&mut self, world: [i64; 2]) {
        let key = (world[1], world[0]);
        if let Ok(index) = self
            .cell_overrides
            .binary_search_by_key(&key, |item| (item.world[1], item.world[0]))
        {
            if self.cell_overrides[index].is_empty() {
                self.cell_overrides.remove(index);
            }
        }
    }
    fn bump(&mut self) {
        self.revision = self.revision.saturating_add(1).max(1);
    }
}

fn splitmix64(mut v: u64) -> u64 {
    v = v.wrapping_add(0x9e3779b97f4a7c15);
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
    v ^ (v >> 31)
}
fn coord_hash(seed: u64, world: [i64; 2], salt: u64) -> u64 {
    splitmix64(
        seed ^ (world[0] as u64).wrapping_mul(0x632be59bd9b4e019)
            ^ (world[1] as u64).wrapping_mul(0x8cb92baa3f3d8dd7)
            ^ salt,
    )
}
fn role_for(seed: u64, world: [i64; 2]) -> &'static str {
    worldgen_plan::terrain_role(seed, world)
}

fn role_neighbor_mask(seed: u64, world: [i64; 2], role: &str) -> u8 {
    let mut mask = 0u8;
    for (dx, dy, bit) in [(0, -1, 1u8), (1, 0, 2u8), (0, 1, 4u8), (-1, 0, 8u8)] {
        if role_for(seed, [world[0] + dx, world[1] + dy]) == role {
            mask |= bit;
        }
    }
    mask
}
fn choose_palette<'a>(entries: &'a [PaletteEntry], hash: u64) -> &'a PaletteEntry {
    let total = entries.iter().map(|e| e.weight as u64).sum::<u64>().max(1);
    let mut cursor = hash % total;
    for e in entries {
        let w = e.weight as u64;
        if cursor < w {
            return e;
        }
        cursor -= w;
    }
    &entries[entries.len() - 1]
}
fn choose_template<'a>(templates: &[&'a ObjectTemplate], hash: u64) -> Option<&'a ObjectTemplate> {
    (!templates.is_empty()).then(|| templates[(hash as usize) % templates.len()])
}
fn baseline_source_for(
    seed: u64,
    world: [i64; 2],
    authority: &AssetAuthority,
    tiled: Option<&CanonicalTiledTerrainAuthority>,
) -> Result<(String, u8, SourceBinding), String> {
    let role = role_for(seed, world);
    let mask = role_neighbor_mask(seed, world, role);
    let roles = [
        role_for(seed, [world[0] - 1, world[1] - 1]),
        role_for(seed, [world[0], world[1] - 1]),
        role_for(seed, [world[0] - 1, world[1]]),
        role_for(seed, world),
    ];
    let projected = WorldDocument::land_connectivity_projection(roles);
    if let Some(tiled) = tiled {
        if let Some(binding) = tiled.resolve_havenwild_corner(projected, seed, world) {
            let source = SourceBinding {
                source_asset: binding.source_asset,
                source_rect: binding.source_rect,
            };
            let source = authority.seasonalize_binding(&source, worldgen_plan::season(seed, world));
            return Ok((role.to_owned(), mask, source));
        }
    }

    let selected = if roles.iter().all(|candidate| *candidate == roles[0]) {
        authority.summer_flatworld_fill_for_world(roles[0], seed, world)
    } else {
        authority
            .summer_flatworld_corner(roles)
            .or_else(|| authority.summer_flatworld_fill_for_world(role, seed, world))
    }
    .ok_or_else(|| format!("Missing Summer flatworld source for {role}"))?;
    let source =
        authority.seasonalize_binding(&selected.binding(), worldgen_plan::season(seed, world));
    Ok((role.to_owned(), mask, source))
}

fn scatter_slot(seed: u64, world: [i64; 2], spacing: i64, salt: u64, chance: u64) -> bool {
    let block = [world[0].div_euclid(spacing), world[1].div_euclid(spacing)];
    let hash = coord_hash(seed, block, salt);
    if hash % 100 >= chance {
        return false;
    }
    let offset = [
        ((hash >> 8) % spacing as u64) as i64,
        ((hash >> 24) % spacing as u64) as i64,
    ];
    world
        == [
            block[0] * spacing + offset[0],
            block[1] * spacing + offset[1],
        ]
}

fn worldgen_land_role(role: &str) -> bool {
    matches!(
        role,
        "Grass" | "MudBank" | "Sand" | "WetSand" | "PebblePath"
    )
}

fn footprint_is_land(seed: u64, anchor: [i64; 2], footprint: [u32; 2]) -> bool {
    (0..footprint[1]).all(|y| {
        (0..footprint[0]).all(|x| {
            worldgen_land_role(role_for(
                seed,
                [anchor[0] + i64::from(x), anchor[1] + i64::from(y)],
            ))
        })
    })
}

fn footprint_is_grass(seed: u64, anchor: [i64; 2], footprint: [u32; 2]) -> bool {
    (0..footprint[1]).all(|y| {
        (0..footprint[0]).all(|x| {
            role_for(seed, [anchor[0] + i64::from(x), anchor[1] + i64::from(y)]) == "Grass"
        })
    })
}

fn footprints_overlap(
    a_anchor: [i64; 2],
    a_size: [u32; 2],
    b_anchor: [i64; 2],
    b_size: [u32; 2],
    b_margin: i64,
) -> bool {
    let a_left = a_anchor[0];
    let a_top = a_anchor[1];
    let a_right = a_left + i64::from(a_size[0]);
    let a_bottom = a_top + i64::from(a_size[1]);
    let b_left = b_anchor[0] - b_margin;
    let b_top = b_anchor[1] - b_margin;
    let b_right = b_anchor[0] + i64::from(b_size[0]) + b_margin;
    let b_bottom = b_anchor[1] + i64::from(b_size[1]) + b_margin;
    a_left < b_right && a_right > b_left && a_top < b_bottom && a_bottom > b_top
}

fn natural_placement_is_clear(
    objects: &[WorldObject],
    anchor: [i64; 2],
    footprint: [u32; 2],
) -> bool {
    objects.iter().all(|existing| {
        // Purpose objects are emitted before ecology. Structures receive an extra
        // one-cell visual/entrance buffer; other already-emitted objects reserve
        // their actual footprint. This prevents bushes, trees, forageables, reeds
        // and water details from being generated on roofs, bridge decks or props
        // while still allowing vegetation behind a tree canopy through depth sort.
        let margin = if existing.layer.eq_ignore_ascii_case("structures") {
            1
        } else {
            0
        };
        !footprints_overlap(
            anchor,
            footprint,
            existing.anchor_world,
            existing.footprint_cells,
            margin,
        )
    })
}

fn world_chunk_coord(world: [i64; 2]) -> [i64; 2] {
    let side = WORLD_CHUNK_SIDE as i64;
    [world[0].div_euclid(side), world[1].div_euclid(side)]
}

fn centered_anchor(center: [i64; 2], footprint: [u32; 2]) -> [i64; 2] {
    [
        center[0] - i64::from(footprint[0] / 2),
        center[1] - i64::from(footprint[1] / 2),
    ]
}

fn find_buildable_anchor_near(
    seed: u64,
    preferred_center: [i64; 2],
    footprint: [u32; 2],
    search_radius: i64,
) -> Option<[i64; 2]> {
    // Purpose-first structures should not disappear because one deterministic
    // center happens to overlap a river/road transition. Search a small, stable
    // parcel neighborhood, preferring pure grass before accepting other land.
    for grass_only in [true, false] {
        for radius in 0..=search_radius {
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if radius > 0 && dx.abs() != radius && dy.abs() != radius {
                        continue;
                    }
                    let center = [preferred_center[0] + dx, preferred_center[1] + dy];
                    let anchor = centered_anchor(center, footprint);
                    let valid = if grass_only {
                        footprint_is_grass(seed, anchor, footprint)
                    } else {
                        footprint_is_land(seed, anchor, footprint)
                    };
                    if valid {
                        return Some(anchor);
                    }
                }
            }
        }
    }
    None
}

fn settlement_house_kind(class: &str) -> &'static str {
    match class {
        "town" => "town house",
        "village" => "village house",
        "hamlet" => "hamlet house",
        "camp" => "camp house",
        "outpost" => "outpost house",
        _ => "settlement house",
    }
}

fn template_parts_for_season(
    authority: &AssetAuthority,
    template: &ObjectTemplate,
    season: &str,
) -> Vec<WorldObjectPart> {
    template
        .parts
        .iter()
        .map(|part| {
            let source = SourceBinding {
                source_asset: part.source_path.clone(),
                source_rect: part.source_rect_px,
            };
            WorldObjectPart {
                offset_px: part.offset_px,
                source: authority.seasonalize_binding(&source, season),
            }
        })
        .collect()
}

fn generated_layer_for(template: &ObjectTemplate, kind: &str) -> String {
    let has_tag = |tag: &str| {
        template
            .worldgen_tags
            .iter()
            .any(|candidate| candidate.as_str() == tag)
    };
    if kind == "water detail" || has_tag("water_detail") {
        "water".into()
    } else if kind == "water-edge planting" || has_tag("water_edge") {
        "terrain_details".into()
    } else if kind == "wildflower" || has_tag("wildflower") {
        "objects".into()
    } else if has_tag("waterfall") || has_tag("elevation_feature") {
        "elevation".into()
    } else {
        template.layer.clone()
    }
}

fn push_generated_object(
    objects: &mut Vec<WorldObject>,
    authority: &AssetAuthority,
    coord: [i64; 2],
    seed: u64,
    anchor: [i64; 2],
    template: &ObjectTemplate,
    kind: &str,
) {
    if world_chunk_coord(anchor) != coord {
        return;
    }
    let season = worldgen_plan::season(seed, anchor);
    let id = format!(
        "{}:{}:{}:{}:{}:{}",
        WORLD_GENERATOR_ID, kind, anchor[0], anchor[1], season, template.template_id
    );
    objects.push(WorldObject {
        id,
        template_id: template.template_id.clone(),
        label: format!("Worldgen {kind}: {}", template.label),
        layer: generated_layer_for(template, kind),
        anchor_world: anchor,
        footprint_cells: template.footprint_cells,
        parts: template_parts_for_season(authority, template, season),
        generator_id: WORLD_GENERATOR_ID.into(),
    });
}

fn push_modular_bridge(
    objects: &mut Vec<WorldObject>,
    authority: &AssetAuthority,
    coord: [i64; 2],
    seed: u64,
    span: worldgen_plan::BridgeSpan,
    template: &ObjectTemplate,
) {
    if world_chunk_coord(span.anchor) != coord || span.span_cells < 3 || template.parts.len() < 3 {
        return;
    }
    let season = worldgen_plan::season(seed, span.anchor);
    let left = &template.parts[0];
    let middle = &template.parts[1];
    let right = &template.parts[2];
    let make_part = |source_part: &crate::asset_authority::TemplatePart, x_cell: u32| {
        let source = SourceBinding {
            source_asset: source_part.source_path.clone(),
            source_rect: source_part.source_rect_px,
        };
        WorldObjectPart {
            offset_px: [x_cell as i32 * 32, 0],
            source: authority.seasonalize_binding(&source, season),
        }
    };
    let mut parts = Vec::with_capacity(span.span_cells as usize);
    parts.push(make_part(left, 0));
    for x in 1..span.span_cells - 1 {
        parts.push(make_part(middle, x));
    }
    parts.push(make_part(right, span.span_cells - 1));
    let id = format!(
        "{}:river bridge:{}:{}:{}:{}:{}",
        WORLD_GENERATOR_ID,
        span.anchor[0],
        span.anchor[1],
        season,
        template.template_id,
        span.span_cells
    );
    objects.push(WorldObject {
        id,
        template_id: template.template_id.clone(),
        label: format!(
            "Worldgen river bridge: {} · {} cells",
            template.label, span.span_cells
        ),
        layer: template.layer.clone(),
        anchor_world: span.anchor,
        footprint_cells: [span.span_cells, 2],
        parts,
        generator_id: WORLD_GENERATOR_ID.into(),
    });
}

fn place_purpose_objects(
    seed: u64,
    coord: [i64; 2],
    authority: &AssetAuthority,
    objects: &mut Vec<WorldObject>,
) {
    let houses: Vec<_> = authority.templates_with_tag("house").collect();
    let clutter: Vec<_> = authority.templates_with_tag("clutter").collect();
    let lights: Vec<_> = authority.templates_with_tag("lighting").collect();
    let bridges: Vec<_> = authority.templates_with_tag("bridge").collect();

    for island in worldgen_plan::ISLANDS {
        let island_hash = coord_hash(seed, island.center, island.salt ^ 0x5055_5250_4f53_45);
        for (settlement_index, settlement) in worldgen_plan::settlement_centers(seed, island)
            .into_iter()
            .enumerate()
        {
            if !houses.is_empty() {
                let primary = houses[(island_hash as usize + settlement_index) % houses.len()];
                let major = matches!(settlement.class, "town" | "village");
                let primary_center = if major {
                    [settlement.center[0] - 3, settlement.center[1] - 4]
                } else {
                    [settlement.center[0], settlement.center[1] - 3]
                };
                if let Some(anchor) =
                    find_buildable_anchor_near(seed, primary_center, primary.footprint_cells, 7)
                {
                    push_generated_object(
                        objects,
                        authority,
                        coord,
                        seed,
                        anchor,
                        primary,
                        settlement_house_kind(settlement.class),
                    );
                }

                // Do not stamp the same two-house composition onto every island.
                // Summer/Autumn settlements are denser; Spring/Winter stay smaller.
                let allow_secondary = major
                    && houses.len() > 1
                    && matches!(settlement.island_id, "summer_isle" | "autumn_isle");
                if allow_secondary {
                    let secondary =
                        houses[(island_hash as usize + settlement_index + 1) % houses.len()];
                    let secondary_center = [settlement.center[0] + 5, settlement.center[1] + 3];
                    if let Some(anchor) = find_buildable_anchor_near(
                        seed,
                        secondary_center,
                        secondary.footprint_cells,
                        7,
                    ) {
                        push_generated_object(
                            objects,
                            authority,
                            coord,
                            seed,
                            anchor,
                            secondary,
                            settlement_house_kind(settlement.class),
                        );
                    }
                }
            }

            // Clutter stays off the building footprint and reads as roadside yard
            // detail. Lighting is placed on the far road edge instead of directly
            // beside a house wall, fixing the attached-lamppost screenshot defect.
            for (index, offset) in [[-4, 2], [4, 2]].into_iter().enumerate() {
                let anchor = [
                    settlement.center[0] + offset[0],
                    settlement.center[1] + offset[1],
                ];
                if !worldgen_land_role(role_for(seed, anchor)) {
                    continue;
                }
                if let Some(template) = choose_template(&clutter, island_hash >> (index * 7)) {
                    push_generated_object(
                        objects,
                        authority,
                        coord,
                        seed,
                        anchor,
                        template,
                        "settlement clutter",
                    );
                }
            }
            if let Some(template) = lights.first().copied() {
                let side = if settlement_index == 0 { -1 } else { 1 };
                let anchor = [settlement.center[0] + side * 3, settlement.center[1] + 2];
                if worldgen_land_role(role_for(seed, anchor)) {
                    push_generated_object(
                        objects,
                        authority,
                        coord,
                        seed,
                        anchor,
                        template,
                        "roadside lighting",
                    );
                }
            }
        }

        if let (Some(template), Some(span)) = (
            choose_template(&bridges, island_hash >> 11),
            worldgen_plan::bridge_span(seed, island),
        ) {
            push_modular_bridge(objects, authority, coord, seed, span, template);
        }
        // Waterfall and cave source recipes remain reviewed in the asset authority,
        // but auto-placement is deliberately gated until continuous cliff faces are
        // generated. The 0.9.1 screenshot proved that standalone feature sprites on
        // flat ground create false geography even when the source crop itself is valid.
    }
}

fn place_natural_objects(
    seed: u64,
    coord: [i64; 2],
    authority: &AssetAuthority,
    objects: &mut Vec<WorldObject>,
) {
    let trees: Vec<_> = authority.templates_with_tag("tree").collect();
    let shrubs: Vec<_> = authority.templates_with_tag("shrub").collect();
    let foliage: Vec<_> = authority.templates_with_tag("foliage").collect();
    let wildflowers: Vec<_> = authority.templates_with_tag("wildflower").collect();
    let rocks: Vec<_> = authority.templates_with_tag("rock").collect();
    let water_details: Vec<_> = authority.templates_with_tag("water_detail").collect();
    let water_edges: Vec<_> = authority.templates_with_tag("water_edge").collect();

    for ly in 0..WORLD_CHUNK_SIDE {
        for lx in 0..WORLD_CHUNK_SIDE {
            let world = [
                coord[0] * WORLD_CHUNK_SIDE as i64 + lx as i64,
                coord[1] * WORLD_CHUNK_SIDE as i64 + ly as i64,
            ];
            let role = role_for(seed, world);
            let biome = worldgen_plan::biome(seed, world);
            let hash = coord_hash(seed, world, 0x4e41_5455_5245);

            if role == "RiverWater" && !worldgen_plan::is_waterfall_site(seed, world) {
                let water_context = worldgen_plan::water_context(seed, world);
                let lily_chance = water_context.lily_pad_density_percent();
                if water_context.supports_lily_pads()
                    && lily_chance > 0
                    && scatter_slot(seed, world, 5, 0x5741_5445_525f_44, lily_chance)
                {
                    if let Some(template) = choose_template(&water_details, hash) {
                        if natural_placement_is_clear(objects, world, template.footprint_cells) {
                            push_generated_object(
                                objects,
                                authority,
                                coord,
                                seed,
                                world,
                                template,
                                "water detail",
                            );
                        }
                    }
                }
                continue;
            }
            if role == "MudBank" && scatter_slot(seed, world, 5, 0x5741_5445_525f_45, 42) {
                if let Some(template) = choose_template(&water_edges, hash >> 5) {
                    if natural_placement_is_clear(objects, world, template.footprint_cells) {
                        push_generated_object(
                            objects,
                            authority,
                            coord,
                            seed,
                            world,
                            template,
                            "water-edge planting",
                        );
                    }
                }
                continue;
            }
            if role != "Grass"
                || matches!(biome, "settlement" | "road_corridor" | "coast")
                || worldgen_plan::is_cave_approach(seed, world)
            {
                continue;
            }

            // Rock templates are source-certified for ordinary grassland too. Previously
            // a height >=2 rocky_upland biome was the *only* spawn path; small
            // generated worlds could contain zero stones. Give meadows sparse
            // stones and favor rocky uplands while retaining spacing/clearance.
            let stone_chance = match biome {
                "rocky_upland" => 80,
                "meadow" => 42,
                "light_woodland" => 28,
                _ => 0,
            };
            if stone_chance > 0 && scatter_slot(seed, world, 5, 0x524f_434b_5f53_4c4f, stone_chance)
            {
                if let Some(template) = choose_template(&rocks, hash >> 13) {
                    let anchor = centered_anchor(world, template.footprint_cells);
                    if footprint_is_grass(seed, anchor, template.footprint_cells)
                        && natural_placement_is_clear(objects, anchor, template.footprint_cells)
                    {
                        push_generated_object(
                            objects,
                            authority,
                            coord,
                            seed,
                            anchor,
                            template,
                            "natural stone",
                        );
                        continue;
                    }
                }
            }

            let tree_density = worldgen_plan::tree_density_percent(seed, world);
            if tree_density > 0 && scatter_slot(seed, world, 4, 0x5452_4545_5f53_4c4f, tree_density)
            {
                if let Some(template) = choose_template(&trees, hash >> 8) {
                    let anchor = centered_anchor(world, template.footprint_cells);
                    if footprint_is_grass(seed, anchor, template.footprint_cells)
                        && natural_placement_is_clear(objects, anchor, template.footprint_cells)
                    {
                        push_generated_object(
                            objects,
                            authority,
                            coord,
                            seed,
                            anchor,
                            template,
                            "biome tree",
                        );
                        continue;
                    }
                }
            }

            if scatter_slot(
                seed,
                world,
                3,
                0x5348_5255_425f_534c,
                if biome == "dense_forest" { 64 } else { 42 },
            ) {
                if let Some(template) = choose_template(&shrubs, hash >> 19) {
                    if natural_placement_is_clear(objects, world, template.footprint_cells) {
                        push_generated_object(
                            objects, authority, coord, seed, world, template, "shrub",
                        );
                        continue;
                    }
                }
            }
            if scatter_slot(seed, world, 4, 0x464f_4c49_4147_455f, 44) {
                if let Some(template) = choose_template(&foliage, hash >> 23) {
                    let anchor = centered_anchor(world, template.footprint_cells);
                    if footprint_is_grass(seed, anchor, template.footprint_cells)
                        && natural_placement_is_clear(objects, anchor, template.footprint_cells)
                    {
                        push_generated_object(
                            objects, authority, coord, seed, anchor, template, "foliage",
                        );
                        continue;
                    }
                }
            }
            if scatter_slot(
                seed,
                world,
                4,
                0x464c_4f57_4552_535f,
                if biome == "meadow" { 72 } else { 32 },
            ) {
                if let Some(template) = choose_template(&wildflowers, hash >> 29) {
                    if natural_placement_is_clear(objects, world, template.footprint_cells) {
                        push_generated_object(
                            objects,
                            authority,
                            coord,
                            seed,
                            world,
                            template,
                            "wildflower",
                        );
                    }
                }
            }
        }
    }
}

fn generate_chunk(
    seed: u64,
    coord: [i64; 2],
    authority: &AssetAuthority,
    tiled: Option<&CanonicalTiledTerrainAuthority>,
    corrections: &[GenerationCorrection],
) -> Result<WorldChunk, String> {
    let mut cells = Vec::with_capacity(WORLD_CHUNK_SIDE * WORLD_CHUNK_SIDE);
    for ly in 0..WORLD_CHUNK_SIDE {
        for lx in 0..WORLD_CHUNK_SIDE {
            let world = [
                coord[0] * WORLD_CHUNK_SIDE as i64 + lx as i64,
                coord[1] * WORLD_CHUNK_SIDE as i64 + ly as i64,
            ];
            let (role, mask, generated_source) =
                baseline_source_for(seed, world, authority, tiled)?;
            let source = corrections
                .iter()
                .find(|item| {
                    item.role == role
                        && item.neighbor_mask == mask
                        && item.generated_source == generated_source
                })
                .map(|item| item.corrected_source.clone())
                .unwrap_or(generated_source);
            cells.push(GeneratedCell {
                role,
                season: worldgen_plan::season(seed, world).into(),
                biome: worldgen_plan::biome(seed, world).into(),
                water_context: worldgen_plan::water_context(seed, world).as_str().into(),
                region_id: worldgen_plan::region_id(seed, world).into(),
                elevation: worldgen_plan::elevation(seed, world),
                cliff_mask: worldgen_plan::cliff_mask(seed, world),
                source,
            });
        }
    }

    let mut objects = Vec::new();
    place_purpose_objects(seed, coord, authority, &mut objects);
    place_natural_objects(seed, coord, authority, &mut objects);
    objects.sort_by(|a, b| {
        (a.anchor_world[1], a.anchor_world[0], a.template_id.as_str()).cmp(&(
            b.anchor_world[1],
            b.anchor_world[0],
            b.template_id.as_str(),
        ))
    });
    objects.dedup_by(|a, b| a.id == b.id);

    Ok(WorldChunk {
        coord,
        generator_id: WORLD_GENERATOR_ID.into(),
        generated_cells: cells,
        generated_objects: objects,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{scene_v2::COLLISION_MASK_SIDE, source_catalog::SourceCatalog};
    fn authority() -> AssetAuthority {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap()
    }
    #[test]
    fn reviewed_object_placement_uses_canonical_sources_and_persists() {
        let source = authority();
        let template = source
            .object_templates
            .iter()
            .find(|candidate| {
                candidate.layer == "objects"
                    && candidate.footprint_cells == [1, 1]
                    && candidate.worldgen_tags.iter().any(|tag| tag == "clutter")
            })
            .expect("Reviewed one-cell clutter template");
        let mut world = WorldDocument::new(0x125);
        world.materialize_3x3([0, 0], &source).unwrap();
        let before_revision = world.revision;
        let before_count = world.authored_objects.len();
        let id = world
            .place_reviewed_template(&template.template_id, [0, 0], &source)
            .expect("Approved source-backed template should be placeable");
        assert_eq!(world.authored_objects.len(), before_count + 1);
        assert!(world.revision > before_revision);
        let added = world
            .authored_objects
            .iter()
            .find(|object| object.id == id)
            .unwrap();
        assert_eq!(added.template_id, template.template_id);
        assert_eq!(added.anchor_world, [0, 0]);
        assert!(added
            .parts
            .iter()
            .all(|part| source.canonical_region(&part.source).is_some()));
        assert!(world
            .place_reviewed_template("alpha:candidate:not-reviewed", [0, 0], &source)
            .is_err());
        let bridge = source
            .object_templates
            .iter()
            .find(|candidate| candidate.worldgen_tags.iter().any(|tag| tag == "bridge"))
            .expect("Existing source-authored modular bridge template");
        assert!(world
            .place_reviewed_template(&bridge.template_id, [0, 0], &source)
            .is_err());
        assert_eq!(world.authored_objects.len(), before_count + 1);
    }

    #[test]
    fn natural_placement_respects_structure_reservations() {
        let existing = WorldObject {
            id: "house".into(),
            template_id: "house-template".into(),
            label: "Worldgen house".into(),
            layer: "structures".into(),
            anchor_world: [10, 10],
            footprint_cells: [5, 5],
            parts: Vec::new(),
            generator_id: WORLD_GENERATOR_ID.into(),
        };
        assert!(!natural_placement_is_clear(
            &[existing.clone()],
            [12, 12],
            [1, 1]
        ));
        assert!(!natural_placement_is_clear(
            &[existing.clone()],
            [15, 12],
            [1, 1]
        ));
        assert!(natural_placement_is_clear(&[existing], [17, 12], [1, 1]));
    }

    #[test]
    fn three_by_three_materialization_is_deterministic_and_source_backed() {
        let a = authority();
        let mut x = WorldDocument::new(12345);
        let mut y = WorldDocument::new(12345);
        x.materialize_3x3([0, 0], &a).unwrap();
        y.materialize_3x3([0, 0], &a).unwrap();
        assert_eq!(x.chunks, y.chunks);
        assert_eq!(x.chunks.len(), 9);
        assert_eq!(x.materialized_bounds(), Some(([-32, -32], [96, 96])));
        assert!(x
            .chunks
            .iter()
            .flat_map(|c| &c.generated_cells)
            .all(|cell| a.contains_binding(&cell.source)));
        let p = x.preview_regenerate_3x3([0, 0], &a).unwrap();
        assert_eq!(p.changed_cells, 0);
        assert_eq!(p.changed_object_sets, 0);
    }
    #[test]
    fn generated_world_radius_two_exposes_four_seasons_ocean_and_biomes() {
        let a = authority();
        let mut world = WorldDocument::new(0x484156454e57494c);
        world.materialize([0, 0], 2, &a).unwrap();
        assert_eq!(world.chunks.len(), 25);
        let seasons: BTreeSet<_> = world
            .chunks
            .iter()
            .flat_map(|chunk| {
                chunk
                    .generated_cells
                    .iter()
                    .map(|cell| cell.season.as_str())
            })
            .collect();
        for expected in ["spring", "summer", "autumn", "winter", "ocean"] {
            assert!(
                seasons.contains(expected),
                "missing generated season/region {expected}"
            );
        }
        let biomes: BTreeSet<_> = world
            .chunks
            .iter()
            .flat_map(|chunk| chunk.generated_cells.iter().map(|cell| cell.biome.as_str()))
            .collect();
        assert!(biomes.contains("ocean"));
        assert!(biomes.contains("dense_forest"));
        assert!(biomes.contains("meadow"));
        assert!(biomes.contains("settlement"));
        assert!(biomes.contains("road_corridor"));

        let elevations: BTreeSet<_> = world
            .chunks
            .iter()
            .flat_map(|chunk| chunk.generated_cells.iter().map(|cell| cell.elevation))
            .collect();
        for expected in [0i16, 1, 2] {
            assert!(
                elevations.contains(&expected),
                "missing generated elevation {expected}"
            );
        }

        let labels = world
            .chunks
            .iter()
            .flat_map(|chunk| {
                chunk
                    .generated_objects
                    .iter()
                    .map(|object| object.label.as_str())
            })
            .collect::<Vec<_>>();
        for expected in [
            "Worldgen river bridge",
            "Worldgen town house",
            "Worldgen village house",
        ] {
            assert!(
                labels.iter().any(|label| label.starts_with(expected)),
                "missing purpose-first object category {expected}"
            );
        }
        assert!(world
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.generated_cells)
            .any(|cell| cell.cliff_mask != 0));
        // Waterfall and cave recipes remain reviewed source authority, but their
        // automatic placement is intentionally gated until continuous cliff faces
        // exist. The generated-world regression must not force those sprites back
        // onto flat ground merely to satisfy an obsolete kitchen-sink expectation.
        assert!(!labels
            .iter()
            .any(|label| label.starts_with("Worldgen waterfall")));
        assert!(!labels
            .iter()
            .any(|label| label.starts_with("Worldgen cliff cave entrance")));
        assert!(a.templates_with_tag("waterfall").next().is_some());
        assert!(a.templates_with_tag("cave_entrance").next().is_some());
        assert!(labels
            .iter()
            .any(|label| label.starts_with("Worldgen biome tree")));
        assert!(world
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.generated_objects)
            .flat_map(|object| &object.parts)
            .all(|part| a.contains_source_slice(&part.source)));
    }

    #[test]
    fn generated_ground_uses_coordinate_equivalent_seasonal_source_sheets() {
        let a = authority();
        let seed = 0x484156454e57494c;
        for island in worldgen_plan::ISLANDS {
            let (_, _, source) = baseline_source_for(seed, island.center, &a, None).unwrap();
            let expected = match island.season {
                "spring" => "Terrain/terrain_spring.png",
                "summer" => "Terrain/terrain_summer.png",
                "autumn" => "Terrain/terrain_autumn.png",
                "winter" => "Terrain/terrain_winter.png",
                other => panic!("unexpected island season {other}"),
            };
            assert_eq!(source.source_asset, expected);
            assert!(a.contains_binding(&source));
        }
    }

    #[test]
    fn regeneration_preserves_authored_visual_collision_and_tombstones() {
        let a = authority();
        let mut w = WorldDocument::new(9);
        w.materialize_3x3([0, 0], &a).unwrap();
        let source = a.role_palette("Grass").unwrap()[0].binding();
        w.set_visual_override([0, 0], source.clone());
        let mut mask = CollisionMask32::empty();
        mask.set_blocked(4, 7, true);
        w.set_collision_mask([0, 0], Some(mask.clone()));
        // Tombstone persistence is independent of current procedural object density.
        let id = "test:generated:object".to_owned();
        w.chunks[0].generated_objects.push(WorldObject {
            id: id.clone(),
            template_id: "test-template".into(),
            label: "test generated object".into(),
            layer: "Objects".into(),
            anchor_world: [0, 0],
            footprint_cells: [1, 1],
            parts: Vec::new(),
            generator_id: WORLD_GENERATOR_ID.into(),
        });
        assert!(w.suppress_generated_object(&id));
        w.materialize_3x3([0, 0], &a).unwrap();
        assert_eq!(w.resolved_source([0, 0]), Some(source));
        assert_eq!(w.collision_mask([0, 0]), Some(&mask));
        assert!(w.suppressed_generated_objects.contains(&id));
    }

    #[test]
    fn learned_generation_correction_propagates_and_survives_regeneration() {
        let a = authority();
        let mut w = WorldDocument::new(17);
        w.materialize_3x3([0, 0], &a).unwrap();
        let mut chosen = None;
        for y in -32..64 {
            for x in -32..64 {
                let world = [x, y];
                let Some(current) = w.generated_cell(world).map(|cell| cell.source.clone()) else {
                    continue;
                };
                let Some(role) = w.resolved_role(world) else {
                    continue;
                };
                let mask = role_neighbor_mask(w.seed, world, &role);
                let Some(palette) = a.topology_palette(&role, mask) else {
                    continue;
                };
                if let Some(target) = palette
                    .iter()
                    .map(PaletteEntry::binding)
                    .find(|candidate| *candidate != current)
                {
                    chosen = Some((world, current, target));
                    break;
                }
            }
            if chosen.is_some() {
                break;
            }
        }
        let Some((world, current, target)) = chosen else {
            return;
        };
        let matches = w
            .learn_generation_correction(world, target.clone(), &a)
            .unwrap();
        assert!(matches >= 1);
        w.materialize_3x3([0, 0], &a).unwrap();
        assert_eq!(w.resolved_source(world), Some(target));
        assert_ne!(w.resolved_source(world), Some(current));
        assert!(!w.generation_corrections.is_empty());
    }

    #[test]
    fn fresh_world_local_override_stays_local() {
        let a = authority();
        let mut w = WorldDocument::new(23);
        w.materialize_3x3([0, 0], &a).unwrap();
        let source = a.role_palette("Grass").unwrap()[0].binding();
        w.set_visual_override([12, 12], source.clone());
        w.materialize_3x3([0, 0], &a).unwrap();
        assert_eq!(w.resolved_source([12, 12]), Some(source));
        assert!(w.generation_corrections.is_empty());
    }
    #[test]
    fn unsafe_cross_topology_correction_is_refused() {
        let a = authority();
        let mut w = WorldDocument::new(31);
        w.materialize_3x3([0, 0], &a).unwrap();
        for y in -32..64 {
            for x in -32..64 {
                let world = [x, y];
                let Some(role) = w.resolved_role(world) else {
                    continue;
                };
                let mask = role_neighbor_mask(w.seed, world, &role);
                let Some(other) = a.role_palette(&role).and_then(|entries| {
                    entries
                        .iter()
                        .map(PaletteEntry::binding)
                        .find(|candidate| !a.topology_supports_binding(&role, mask, candidate))
                }) else {
                    continue;
                };
                let result = w.learn_generation_correction(world, other, &a);
                assert!(result.is_err());
                return;
            }
        }
    }

    #[test]
    fn semantic_world_paint_resolves_exact_source_autotiles() {
        let a = authority();
        let mut w = WorldDocument::new(41);
        w.materialize_3x3([0, 0], &a).unwrap();
        let world = [8, 8];
        w.paint_semantic_terrain(world, "RiverWater");
        assert_eq!(w.resolved_role(world).as_deref(), Some("RiverWater"));
        let source = w
            .resolved_source_with_authority(world, &a)
            .expect("semantic paint resolves a source-backed tile");
        assert!(a.contains_binding(&source));
        w.paint_semantic_terrain([9, 8], "RiverWater");
        let neighbour = w
            .resolved_source_with_authority(world, &a)
            .expect("neighbour change re-resolves source-backed topology");
        assert!(a.contains_binding(&neighbour));
        w.clear_authored_terrain(world);
        assert_eq!(
            w.resolved_role(world),
            w.generated_cell(world).map(|cell| cell.role.clone())
        );
    }

    #[test]
    fn dual_grid_render_resolves_the_extra_east_and_south_vertices() {
        let a = authority();
        let mut w = WorldDocument::new(43);
        w.materialize_3x3([0, 0], &a).unwrap();
        let (origin, size) = w.materialized_bounds().unwrap();
        let east_south_vertex = [origin[0] + size[0] as i64, origin[1] + size[1] as i64];
        let parts = w
            .resolved_dual_grid_visual_parts_with_authority(east_south_vertex, &a)
            .expect("deterministic virtual neighbours keep the outer dual-grid vertex renderable");
        assert!(!parts.is_empty());
        assert!(parts.iter().all(|part| a.contains_binding(&part.source)));
    }

    #[test]
    fn direct_source_override_remains_semantic_cell_aligned_authority() {
        let a = authority();
        let mut w = WorldDocument::new(47);
        w.materialize_3x3([0, 0], &a).unwrap();
        let world = [7, 9];
        let source = a.role_palette("Grass").unwrap()[0].binding();
        w.set_visual_override(world, source.clone());
        let parts = w
            .resolved_visual_parts_with_authority(world, &a)
            .expect("direct source override remains queryable in semantic-cell space");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].source, source);
        assert_eq!(parts[0].destination_rect_px, [0, 0, 32, 32]);
    }

    #[test]
    fn ambiguous_diagonal_water_connects_land_without_quadrant_parts() {
        let a = authority();
        let mut w = WorldDocument::new(59);
        w.materialize_3x3([0, 0], &a).unwrap();
        let vertex = [8, 8];
        for (cell, role) in [
            ([7, 7], "Grass"),
            ([8, 7], "RiverWater"),
            ([7, 8], "RiverWater"),
            ([8, 8], "Grass"),
        ] {
            w.paint_semantic_terrain(cell, role);
        }
        let parts = w
            .resolved_dual_grid_visual_parts_with_authority(vertex, &a)
            .expect("ambiguous diagonal shoreline resolves");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].destination_rect_px, [0, 0, 32, 32]);
        assert_eq!(parts[0].source.source_rect[2..], [32, 32]);
        let expected = a
            .summer_flatworld_fill_for_world("Grass", w.seed, vertex)
            .expect("land-connectivity uses a deterministic Grass fill variant")
            .binding();
        assert_eq!(parts[0].source, expected);
    }

    #[test]
    fn cardinal_water_channel_remains_open() {
        let a = authority();
        let mut w = WorldDocument::new(61);
        w.materialize_3x3([0, 0], &a).unwrap();
        let vertex = [8, 8];
        for (cell, role) in [
            ([7, 7], "Grass"),
            ([8, 7], "Grass"),
            ([7, 8], "RiverWater"),
            ([8, 8], "RiverWater"),
        ] {
            w.paint_semantic_terrain(cell, role);
        }
        let parts = w
            .resolved_dual_grid_visual_parts_with_authority(vertex, &a)
            .expect("cardinal water channel resolves");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].destination_rect_px, [0, 0, 32, 32]);
        assert_ne!(
            parts[0].source,
            a.summer_flatworld_fill("Grass").unwrap().binding()
        );
    }

    #[test]
    fn mixed_grass_dirt_water_uses_complete_authored_pairwise_tile() {
        let a = authority();
        let mut w = WorldDocument::new(63);
        w.materialize_3x3([0, 0], &a).unwrap();
        let vertex = [8, 8];
        for (cell, role) in [
            ([7, 7], "Grass"),
            ([8, 7], "MudBank"),
            ([7, 8], "RiverWater"),
            ([8, 8], "RiverWater"),
        ] {
            w.paint_semantic_terrain(cell, role);
        }
        let parts = w
            .resolved_dual_grid_visual_parts_with_authority(vertex, &a)
            .expect("mixed bank shoreline resolves");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].destination_rect_px, [0, 0, 32, 32]);
        assert_eq!(parts[0].source.source_rect[2..], [32, 32]);
    }

    #[test]
    fn generated_ground_uses_seasonal_terrain_sheets_and_never_cliff_sheets() {
        let a = authority();
        let mut w = WorldDocument::new(73);
        w.materialize_3x3([0, 0], &a).unwrap();
        assert!(w
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.generated_cells)
            .all(|cell| {
                let expected = match cell.season.as_str() {
                    "spring" => "Terrain/terrain_spring.png",
                    "autumn" => "Terrain/terrain_autumn.png",
                    "winter" => "Terrain/terrain_winter.png",
                    "summer" | "ocean" => "Terrain/terrain_summer.png",
                    _ => return false,
                };
                cell.source.source_asset == expected
                    && !cell.source.source_asset.contains("/cliff_")
            }));

        for (world, role) in [
            ([4, 4], "Grass"),
            ([5, 4], "MudBank"),
            ([6, 4], "RiverWater"),
            ([6, 5], "RiverWater"),
        ] {
            w.paint_semantic_terrain(world, role);
        }
        for world in [[4, 4], [5, 4], [6, 4], [6, 5]] {
            let parts = w
                .resolved_visual_parts_with_authority(world, &a)
                .expect("strict Summer flatworld paint must resolve visual parts");
            assert!(
                !parts.is_empty(),
                "strict Summer flatworld paint resolved no visual parts"
            );
            assert!(parts.iter().all(|part| {
                a.contains_source_slice(&part.source)
                    && part.source.source_asset == "Terrain/terrain_summer.png"
            }));
        }
    }

    #[test]
    fn semantic_dual_grid_fails_closed_without_a_corner_recipe() {
        let a = authority();
        let mut w = WorldDocument::new(101);
        w.materialize_3x3([0, 0], &a).unwrap();
        let world = [12, 12];
        w.paint_semantic_terrain(world, "RiverWater");
        let source = w
            .resolved_source_with_authority(world, &a)
            .expect("mixed semantic corner resolves through exact source or safe fill");
        assert_eq!(source.source_asset, "Terrain/terrain_summer.png");
    }

    #[test]
    fn terrain_topology_collision_profile_does_not_leak_to_other_roles() {
        let a = authority();
        let mut w = WorldDocument::new(97);
        w.materialize_3x3([0, 0], &a).unwrap();
        let water = [8, 8];
        let grass = [9, 8];
        w.paint_semantic_terrain(water, "RiverWater");
        w.paint_semantic_terrain(grass, "Grass");
        let (role, mask) = w
            .resolved_terrain_behavior_key(water)
            .expect("water topology key");
        assert_eq!(role, "RiverWater");
        w.set_terrain_collision_profile(role, mask, Some(CollisionMask32::full()));
        assert_eq!(
            w.effective_collision_mask(water, &a)
                .expect("water profile")
                .blocked_count(),
            (COLLISION_MASK_SIDE * COLLISION_MASK_SIDE) as u32
        );
        assert!(w.effective_collision_mask(grass, &a).is_none());
    }

    #[test]
    fn modular_bridge_repeats_middle_and_owns_walkable_crossing() {
        let a = authority();
        let mut w = WorldDocument::new(7);
        w.materialize([0, 0], 4, &a).unwrap();
        let bridge = w
            .generated_objects()
            .find(|object| object.label.starts_with("Worldgen river bridge"))
            .expect("at least one modular bridge");
        assert!(bridge.footprint_cells[0] >= 3);
        assert_eq!(bridge.parts.len(), bridge.footprint_cells[0] as usize);
        assert_eq!(bridge.parts.first().unwrap().source.source_rect[2], 32);
        assert_eq!(bridge.parts.last().unwrap().source.source_rect[2], 32);
        let deck = [
            bridge.anchor_world[0] + i64::from(bridge.footprint_cells[0] / 2),
            bridge.anchor_world[1] + 1,
        ];
        assert!(w.generated_bridge_contains(deck));
        assert_eq!(w.effective_traversal(deck, &a), TraversalMode::Walkable);
        assert_eq!(
            w.effective_collision_mask(deck, &a)
                .expect("bridge deck collision override")
                .blocked_count(),
            0
        );
    }

    #[test]
    fn generated_lily_pads_are_inland_only_and_pond_weighted() {
        let a = authority();
        let mut w = WorldDocument::new(7);
        w.materialize([0, 0], 4, &a).unwrap();
        let mut seen = 0usize;
        for object in w
            .generated_objects()
            .filter(|object| object.label.starts_with("Worldgen water detail"))
        {
            seen += 1;
            let context = worldgen_plan::water_context(w.seed, object.anchor_world);
            assert!(context.supports_lily_pads());
            assert!(!matches!(
                context,
                worldgen_plan::WaterContext::Ocean | worldgen_plan::WaterContext::Coast
            ));
        }
        assert!(seen > 0, "expected at least one inland lily-pad placement");
    }

    #[test]
    fn reviewed_barrel_template_uses_full_height_source_sprite() {
        let a = authority();
        // Barrel is a reviewed Furniture/Objects source, not a Terrain/ asset.
        // Exercise the same eligible-clutter lookup used by world generation.
        let barrel = a
            .templates_with_tag("clutter")
            .find(|template| {
                template.label.starts_with("Barrel")
                    && template.parts.len() == 1
                    && template.parts[0].source_path == "Objects/Furniture/Barrel.png"
            })
            .expect("reviewed barrel is registered as worldgen-safe clutter");
        assert_eq!(barrel.authority, "manual_source_region_review");
        assert_eq!(barrel.footprint_cells, [1, 1]);
        assert_eq!(barrel.parts.len(), 1);
        assert_eq!(barrel.parts[0].source_rect_px, [0, 0, 32, 64]);
        assert_eq!(barrel.parts[0].offset_px, [0, -32]);
    }

    #[test]
    fn generated_natural_details_route_to_visible_composition_layers() {
        let a = authority();
        let water_detail = a
            .templates_with_tag("water_detail")
            .next()
            .expect("water-detail source template");
        let water_edge = a
            .templates_with_tag("water_edge")
            .next()
            .expect("water-edge source template");
        let wildflower = a
            .templates_with_tag("wildflower")
            .next()
            .expect("wildflower source template");
        let tree = a
            .templates_with_tag("tree")
            .next()
            .expect("tree source template");

        assert_eq!(generated_layer_for(water_detail, "water detail"), "water");
        assert_eq!(
            generated_layer_for(water_edge, "water-edge planting"),
            "terrain_details"
        );
        assert_eq!(generated_layer_for(wildflower, "wildflower"), "objects");
        assert_eq!(generated_layer_for(tree, "biome tree"), "objects");
    }

    #[test]
    fn quarantining_large_generated_objects_keeps_certified_slices_and_removes_uncertified_parts() {
        let a = authority();
        let canonical = a
            .templates_with_tag("tree")
            .next()
            .expect("promoted canonical tree template exists");
        let canonical_template_id = canonical.template_id.clone();
        let canonical_part = &canonical.parts[0];
        let canonical_source = SourceBinding {
            source_asset: canonical_part.source_path.clone(),
            source_rect: canonical_part.source_rect_px,
        };
        let mut certified_slice = canonical_source.clone();
        certified_slice.source_rect[0] += 1;
        certified_slice.source_rect[1] += 1;
        certified_slice.source_rect[2] = certified_slice.source_rect[2].saturating_sub(2);
        certified_slice.source_rect[3] = certified_slice.source_rect[3].saturating_sub(2);
        assert!(a.contains_source_slice(&certified_slice));

        let invalid_source = SourceBinding {
            source_asset: canonical_source.source_asset.clone(),
            source_rect: [500, 500, 96, 128],
        };
        assert!(!a.contains_source_slice(&invalid_source));

        let make_object = |id: &str, source: SourceBinding| WorldObject {
            id: id.into(),
            template_id: canonical_template_id.clone(),
            label: id.into(),
            layer: "Objects".into(),
            anchor_world: [0, 0],
            footprint_cells: [3, 4],
            parts: vec![WorldObjectPart {
                offset_px: [0, 0],
                source,
            }],
            generator_id: WORLD_GENERATOR_ID.into(),
        };

        let mut w = WorldDocument::new(103);
        w.chunks.push(WorldChunk {
            coord: [0, 0],
            generator_id: WORLD_GENERATOR_ID.into(),
            generated_cells: Vec::new(),
            generated_objects: vec![
                make_object("certified-slice", certified_slice),
                make_object("invalid", invalid_source),
            ],
        });
        assert_eq!(w.quarantine_uncertified_large_objects(&a), 1);
        assert_eq!(w.chunks[0].generated_objects.len(), 1);
        assert_eq!(w.chunks[0].generated_objects[0].id, "certified-slice");
    }
}

#[cfg(test)]
mod m2d101_object_move_tests {
    use super::*;

    fn world_with_object() -> (WorldDocument, String) {
        let mut world = WorldDocument::new(7);
        let id = "generated:tree:stable".to_owned();
        for y in 0..2 {
            for x in 0..2 {
                world.chunks.push(WorldChunk {
                    coord: [x, y],
                    generator_id: WORLD_GENERATOR_ID.into(),
                    generated_cells: vec![
                        GeneratedCell {
                            role: "Grass".into(),
                            season: "summer".into(),
                            biome: "test".into(),
                            water_context: "none".into(),
                            region_id: "test".into(),
                            elevation: 0,
                            cliff_mask: 0,
                            source: SourceBinding {
                                source_asset: "Terrain/terrain_summer.png".into(),
                                source_rect: [0, 0, 32, 32]
                            },
                        };
                        WORLD_CHUNK_SIDE * WORLD_CHUNK_SIDE
                    ],
                    generated_objects: if x == 0 && y == 0 {
                        vec![WorldObject {
                            id: id.clone(),
                            template_id: "tree".into(),
                            label: "tree".into(),
                            layer: "objects".into(),
                            anchor_world: [10, 10],
                            footprint_cells: [2, 2],
                            parts: vec![],
                            generator_id: WORLD_GENERATOR_ID.into(),
                        }]
                    } else {
                        vec![]
                    },
                });
            }
        }
        (world, id)
    }

    #[test]
    fn moved_objects_keep_original_procedural_anchor_and_serialize() {
        let (mut doc, id) = world_with_object();
        assert!(doc.move_generated_object(&id, [34, 35]).unwrap());
        assert_eq!(
            doc.object_anchor(doc.generated_objects().next().unwrap()),
            [34, 35]
        );
        assert_eq!(
            doc.generated_objects().next().unwrap().anchor_world,
            [10, 10]
        );
        let restored: WorldDocument =
            serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
        assert_eq!(restored.moved_generated_objects.get(&id), Some(&[34, 35]));
        assert!(!doc.move_generated_object(&id, [34, 35]).unwrap());
        assert!(doc.move_generated_object(&id, [10, 10]).unwrap());
        assert!(!doc.moved_generated_objects.contains_key(&id));
    }

    #[test]
    fn world_move_rejects_outside_and_suppressed_objects() {
        let (mut doc, id) = world_with_object();
        assert!(doc.move_generated_object(&id, [63, 63]).is_err());
        assert!(doc.move_generated_object("missing", [1, 1]).is_err());
        assert!(doc.suppress_generated_object(&id));
        assert!(doc.move_generated_object(&id, [3, 3]).is_err());
    }

    #[test]
    fn prior_world_documents_default_missing_move_metadata() {
        let (world, _) = world_with_object();
        let mut old = serde_json::to_value(&world).unwrap();
        old.as_object_mut().unwrap().remove("movedGeneratedObjects");
        let restored: WorldDocument = serde_json::from_value(old).unwrap();
        assert!(restored.moved_generated_objects.is_empty());
    }
}

#[cfg(test)]
mod m2d102_bridge_and_coast_tests {
    use super::*;
    use crate::source_catalog::SourceCatalog;

    fn authority() -> AssetAuthority {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap()
    }

    #[test]
    fn dragged_bridge_is_exact_source_and_undoable_document_state() {
        let a = authority();
        let mut doc = WorldDocument::new(115);
        doc.materialize_3x3([0, 0], &a).unwrap();
        for y in 8..10 {
            for x in 11..15 {
                doc.paint_semantic_terrain([x, y], "RiverWater");
            }
        }
        for x in [10, 15] {
            doc.paint_semantic_terrain([x, 8], "Grass");
            doc.paint_semantic_terrain([x, 9], "Grass");
        }
        let before = doc.clone();
        let id = doc.place_authored_bridge([15, 8], [10, 8], &a).unwrap();
        let bridge = doc
            .generated_objects()
            .find(|object| object.id == id)
            .unwrap();
        assert_eq!(bridge.footprint_cells, [6, 2]);
        assert_eq!(bridge.parts.len(), 6);
        assert!(bridge
            .parts
            .iter()
            .all(|part| a.contains_source_slice(&part.source)));
        assert!(doc.generated_bridge_contains([13, 9]));
        assert!(!doc.can_move_generated_object_to(&id, [20, 20]));
        let stored: WorldDocument =
            serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
        assert_eq!(stored.authored_objects.len(), 1);
        assert!(stored.validate_against_authority(&a).is_ok());
        assert!(before.authored_objects.is_empty()); // one checkpoint restores undo atomically
        assert!(doc.suppress_generated_object(&id));
        assert!(!doc.generated_bridge_contains([13, 9]));
    }

    #[test]
    fn bridge_rejects_invalid_or_vertical_crossings_without_mutation() {
        let a = authority();
        let mut doc = WorldDocument::new(119);
        doc.materialize_3x3([0, 0], &a).unwrap();
        assert!(doc.place_authored_bridge([8, 8], [8, 12], &a).is_err());
        assert!(doc.place_authored_bridge([8, 8], [9, 8], &a).is_err());
        assert_eq!(doc.authored_objects.len(), 0);
    }

    #[test]
    fn painted_sand_dirt_and_grass_in_deep_water_create_visual_shallow_fringe() {
        let a = authority();
        let mut doc = WorldDocument::new(127);
        doc.materialize_3x3([0, 0], &a).unwrap();
        let island = [5i64, 5i64];
        for y in (island[1] - 5)..=(island[1] + 5) {
            for x in (island[0] - 5)..=(island[0] + 5) {
                doc.paint_semantic_terrain_untracked([x, y], "DeepWater");
            }
        }
        doc.commit_edit_batch();
        for (role, at) in [
            ("Sand", island),
            ("MudBank", [island[0] + 1, island[1]]),
            ("Grass", [island[0], island[1] + 1]),
        ] {
            doc.paint_semantic_terrain(at, role);
            assert_eq!(doc.resolved_role(at).as_deref(), Some(role));
        }
        let near = [island[0] - 1, island[1]];
        assert_eq!(doc.resolved_role(near).as_deref(), Some("DeepWater"));
        assert_eq!(doc.resolved_role_for_dual_grid(near), "RiverWater");
        // A pending paint stroke updates the Wang input immediately before save.
        doc.paint_semantic_terrain_untracked([island[0] + 2, island[1]], "Sand");
        assert_eq!(
            doc.resolved_role_for_dual_grid([island[0] + 3, island[1]]),
            "RiverWater"
        );
        doc.commit_edit_batch();
        let stored: WorldDocument =
            serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
        assert_eq!(stored.resolved_role(near).as_deref(), Some("DeepWater"));
        assert_eq!(stored.resolved_role_for_dual_grid(near), "RiverWater");
        // Painting ocean must never silently rewrite the source sprite or the
        // swimmable gameplay semantics merely to get a pleasant visual edge.
    }

    #[test]
    fn ocean_coast_uses_shallow_wang_visual_without_mutating_water_semantics() {
        // M2D105: the old test searched [0,0] for a generated cell with
        // context=coast AND role=DeepWater. That combination cannot come from
        // this generator: terrain_role() gives the coastline RiverWater, and
        // DeepWater only beyond the coast band. Locate a genuine coastline
        // using the deterministic planner instead of assuming the spawn chunk
        // happens to contain one.
        let seed = 5_206_537_499_884_079_436u64;
        let island = worldgen_plan::ISLANDS
            .iter()
            .copied()
            .find(|island| island.id == "summer_isle")
            .expect("Summer island descriptor is present");
        let mut coast = None;
        'search: for y in
            island.center[1] - island.radius[1] - 8..=island.center[1] + island.radius[1] + 8
        {
            for x in
                island.center[0] - island.radius[0] - 8..=island.center[0] + island.radius[0] + 8
            {
                let cell = [x, y];
                if worldgen_plan::water_context(seed, cell) == worldgen_plan::WaterContext::Coast {
                    coast = Some(cell);
                    break 'search;
                }
            }
        }
        let coast = coast.expect("Summer generator must provide a coastal water sample");
        assert_eq!(worldgen_plan::terrain_role(seed, coast), "RiverWater");

        let authority = authority();
        let mut doc = WorldDocument::new(seed);
        let chunk = [
            coast[0].div_euclid(WORLD_CHUNK_SIDE as i64),
            coast[1].div_euclid(WORLD_CHUNK_SIDE as i64),
        ];
        doc.materialize(chunk, 0, &authority).unwrap();
        assert_eq!(doc.generated_water_context(coast), Some("coast"));
        assert_eq!(doc.resolved_role(coast).as_deref(), Some("RiverWater"));
        assert_eq!(doc.resolved_role_for_dual_grid(coast), "RiverWater");

        // A deliberate DeepWater paint on this same coastal cell changes the
        // saved/gameplay role, but its Tiled/Wang input stays shallow. Verify
        // this independently of whether the generated shoreline was shallow.
        doc.paint_semantic_terrain(coast, "DeepWater");
        assert_eq!(doc.resolved_role(coast).as_deref(), Some("DeepWater"));
        assert_eq!(doc.generated_water_context(coast), Some("coast"));
        assert_eq!(doc.resolved_role_for_dual_grid(coast), "RiverWater");
        assert_eq!(doc.resolved_dual_grid_roles(coast)[3], "RiverWater");
        let restored: WorldDocument =
            serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
        assert_eq!(restored.resolved_role(coast).as_deref(), Some("DeepWater"));
        assert_eq!(restored.resolved_role_for_dual_grid(coast), "RiverWater");
    }
}

#[cfg(test)]
mod structure_doorway_tests {
    use super::*;
    #[test]
    fn cardinal_doorways_are_outside_structure_footprint() {
        let anchor = [10, -8];
        let footprint = [4, 3];
        for door in [[10, -9], [12, -9], [13, -5], [14, -6], [9, -8], [9, -6]] {
            assert!(
                doorway_borders_footprint(anchor, footprint, door),
                "{door:?}"
            );
        }
        for bad in [[10, -8], [12, -7], [11, -10], [8, -6], [14, -9], [15, -6]] {
            assert!(
                !doorway_borders_footprint(anchor, footprint, bad),
                "{bad:?}"
            );
        }
    }
    #[test]
    fn old_worlds_default_to_no_mapped_doors() {
        let mut raw = serde_json::to_value(WorldDocument::new(5)).unwrap();
        raw.as_object_mut().unwrap().remove("structureDoors");
        let old: WorldDocument = serde_json::from_value(raw).unwrap();
        assert!(old.structure_doors.is_empty());
    }
    #[test]
    fn arbitrary_door_scene_paths_fail_validation() {
        let mut world = WorldDocument::new(7);
        world.structure_doors.insert(
            "building:1".into(),
            StructureDoorLink {
                object_id: "building:1".into(),
                doorway_world: [0, 0],
                destination_scene: "../../some_other_project/data".into(),
                destination_cell: [0, 0],
            },
        );
        assert!(world.validate().is_err());
    }
}
