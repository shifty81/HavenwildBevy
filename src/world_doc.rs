//! Editable world composition. Generation chooses/places canonical ElizaWy records only;
//! it never creates image assets or mutates source pixels.
use crate::{
    asset_authority::{AssetAuthority, ObjectTemplate, PaletteEntry, SummerVisualPart},
    atomic_file::write_atomic,
    scene_v2::{CollisionMask32, SourceBinding},
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
pub const WORLD_GENERATOR_ID: &str = "havenwild.source_backed.generated_world.v2";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedCell {
    pub role: String,
    #[serde(default = "default_generated_season")]
    pub season: String,
    #[serde(default = "default_generated_biome")]
    pub biome: String,
    #[serde(default = "default_generated_region")]
    pub region_id: String,
    pub source: SourceBinding,
}

fn default_generated_season() -> String {
    "summer".into()
}
fn default_generated_biome() -> String {
    "legacy".into()
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
    pub generation_corrections: Vec<GenerationCorrection>,
    #[serde(default)]
    pub generation_correction_cycle_initialized: bool,
    pub suppressed_generated_objects: BTreeSet<String>,
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
            generation_corrections: Vec::new(),
            generation_correction_cycle_initialized: true,
            suppressed_generated_objects: BTreeSet::new(),
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
                    if !authority.contains_binding(&part.source) {
                        return Err(format!(
                            "World placement source outside unified ElizaWy lane: {} {:?}",
                            part.source.source_asset, part.source.source_rect
                        ));
                    }
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
        if !(0..=4).contains(&radius) {
            return Err("World preview radius must be 0..4".into());
        }
        let mut changed_cells = 0;
        let mut changed_object_sets = 0;
        let mut chunk_count = 0;
        for cy in center[1] - i64::from(radius)..=center[1] + i64::from(radius) {
            for cx in center[0] - i64::from(radius)..=center[0] + i64::from(radius) {
                chunk_count += 1;
                let candidate =
                    generate_chunk(self.seed, [cx, cy], authority, &self.generation_corrections)?;
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
        if !(0..=4).contains(&radius) {
            return Err("World materialization radius must be 0..4".into());
        }
        self.repair_invalid_generation_corrections(authority)?;
        if !self.generation_correction_cycle_initialized {
            self.promote_legacy_visual_overrides_to_generation_corrections(authority)?;
        }
        for role in ["Grass", "RiverWater", "MudBank"] {
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
    pub fn generated_region_id(&self, world: [i64; 2]) -> Option<&str> {
        self.generated_cell(world)
            .map(|cell| cell.region_id.as_str())
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
        self.resolved_visual_parts_with_authority_at(world, authority, None)
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
                            }]
                        })
                    }
                    None => {}
                }
            }
        }

        // One semantic dual-grid authority drives generated terrain and authored
        // terrain alike. Every rendered output tile reads the four surrounding
        // logical terrain cells; source artwork remains immutable.
        let corner_cells = [
            [world[0] - 1, world[1] - 1], // NW
            [world[0], world[1] - 1],     // NE
            [world[0] - 1, world[1]],     // SW
            world,                        // SE
        ];
        let roles = [
            self.resolved_role(corner_cells[0]),
            self.resolved_role(corner_cells[1]),
            self.resolved_role(corner_cells[2]),
            self.resolved_role(corner_cells[3]),
        ];
        let [Some(nw), Some(ne), Some(sw), Some(se)] = roles else {
            return self.generated_cell(world).map(|cell| {
                vec![SummerVisualPart {
                    source: cell.source.clone(),
                    destination_rect_px: [0, 0, 32, 32],
                }]
            });
        };
        let role_refs = [nw.as_str(), ne.as_str(), sw.as_str(), se.as_str()];
        if role_refs.iter().all(|role| *role == role_refs[0]) {
            // M2D081C: homogeneous terrain is spatial, not temporal. RiverWater
            // remains on the conservative static source fill until explicit
            // ordered animation evidence exists.
            let _ = elapsed_ms;
            let entry = authority.summer_flatworld_fill_for_world(role_refs[0], self.seed, world);
            return entry.map(|entry| {
                vec![SummerVisualPart {
                    source: entry.binding(),
                    destination_rect_px: [0, 0, 32, 32],
                }]
            });
        }
        if let Some(parts) = authority.summer_flatworld_visual_parts(role_refs) {
            return Some(parts);
        }

        // Three-material junctions and intentionally unsupported material pairs
        // fail closed to the directly authored SE material. They never borrow
        // cliff/fixture art or a color-inferred region.
        authority.summer_flatworld_fill(role_refs[3]).map(|entry| {
            vec![SummerVisualPart {
                source: entry.binding(),
                destination_rect_px: [0, 0, 32, 32],
            }]
        })
    }

    pub fn resolved_source_with_authority(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
    ) -> Option<SourceBinding> {
        let parts = self.resolved_visual_parts_with_authority(world, authority)?;
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
        authority: &AssetAuthority,
    ) -> Option<CollisionMask32> {
        if let Some(mask) = self.collision_mask(world) {
            return Some(mask.clone());
        }
        let source = self.resolved_source_with_authority(world, authority)?;
        self.source_behavior_profile(&source)
            .and_then(|profile| profile.collision.clone())
    }
    pub fn effective_traversal(
        &self,
        world: [i64; 2],
        authority: &AssetAuthority,
    ) -> TraversalMode {
        if let Some(mode) = self.cell_override(world).and_then(|item| item.traversal) {
            return mode;
        }
        if let Some(source) = self.resolved_source_with_authority(world, authority) {
            if let Some(mode) = self
                .source_behavior_profile(&source)
                .and_then(|profile| profile.traversal)
            {
                return mode;
            }
        }
        match self.resolved_role(world).as_deref() {
            Some("RiverWater") => TraversalMode::Swimmable,
            Some("Grass" | "MudBank") => TraversalMode::Walkable,
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
            baseline_source_for(self.seed, world, authority)?;
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
        let coords: Vec<_> = self.chunks.iter().map(|chunk| chunk.coord).collect();
        let mut replacements = Vec::with_capacity(coords.len());
        let mut changed_cells = 0usize;
        for coord in coords {
            let candidate =
                generate_chunk(self.seed, coord, authority, &self.generation_corrections)?;
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
            baseline_source_for(self.seed, world, authority)?;
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
                        baseline_source_for(self.seed, world, authority)?;
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

    pub fn generated_objects(&self) -> impl Iterator<Item = &WorldObject> {
        self.chunks
            .iter()
            .flat_map(|c| c.generated_objects.iter())
            .filter(|o| !self.suppressed_generated_objects.contains(&o.id))
    }
    pub fn suppress_generated_object(&mut self, id: &str) -> bool {
        if self
            .chunks
            .iter()
            .any(|c| c.generated_objects.iter().any(|o| o.id == id))
        {
            self.suppressed_generated_objects.insert(id.to_owned());
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
                let exact_detail = object.footprint_cells == [1, 1]
                    && object.parts.iter().all(|part| {
                        part.source.source_rect[2] == 32 && part.source.source_rect[3] == 32
                    });
                exact_detail || authority.locally_certified_object_template(&object.template_id)
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
) -> Result<(String, u8, SourceBinding), String> {
    let role = role_for(seed, world);
    let mask = role_neighbor_mask(seed, world, role);
    let roles = [
        role_for(seed, [world[0] - 1, world[1] - 1]),
        role_for(seed, [world[0], world[1] - 1]),
        role_for(seed, [world[0] - 1, world[1]]),
        role_for(seed, world),
    ];
    let selected = if roles.iter().all(|candidate| *candidate == roles[0]) {
        authority.summer_flatworld_fill_for_world(roles[0], seed, world)
    } else {
        authority
            .summer_flatworld_corner(roles)
            .or_else(|| authority.summer_flatworld_fill_for_world(role, seed, world))
    }
    .ok_or_else(|| format!("Missing Summer flatworld source for {role}"))?;
    Ok((role.to_owned(), mask, selected.binding()))
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

fn footprint_is_grass(seed: u64, anchor: [i64; 2], footprint: [u32; 2]) -> bool {
    (0..footprint[1]).all(|y| {
        (0..footprint[0]).all(|x| {
            role_for(seed, [anchor[0] + i64::from(x), anchor[1] + i64::from(y)]) == "Grass"
        })
    })
}

fn generate_chunk(
    seed: u64,
    coord: [i64; 2],
    authority: &AssetAuthority,
    corrections: &[GenerationCorrection],
) -> Result<WorldChunk, String> {
    let mut cells = Vec::with_capacity(WORLD_CHUNK_SIDE * WORLD_CHUNK_SIDE);
    for ly in 0..WORLD_CHUNK_SIDE {
        for lx in 0..WORLD_CHUNK_SIDE {
            let world = [
                coord[0] * WORLD_CHUNK_SIDE as i64 + lx as i64,
                coord[1] * WORLD_CHUNK_SIDE as i64 + ly as i64,
            ];
            let (role, mask, generated_source) = baseline_source_for(seed, world, authority)?;
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
                region_id: worldgen_plan::region_id(seed, world).into(),
                source,
            });
        }
    }
    // One-cell details are always safe. Larger trees/rocks re-enter worldgen only
    // after the local source-bound mapper has isolated their alpha-connected sprite
    // component and AssetAuthority has loaded that certified crop metadata.
    let details: Vec<_> = authority
        .terrain_object_templates()
        .filter(|template| {
            template.footprint_cells == [1, 1]
                && template.parts.len() == 1
                && template.parts[0].source_rect_px[2] == 32
                && template.parts[0].source_rect_px[3] == 32
        })
        .collect();
    let large_objects: Vec<_> = authority
        .terrain_object_templates()
        .filter(|template| {
            template.footprint_cells != [1, 1]
                && authority.locally_certified_object_template(&template.template_id)
                && (template.label.contains("tree")
                    || template.label.contains("rock")
                    || template.label.contains("stone"))
        })
        .collect();
    let mut objects = Vec::new();
    for ly in 0..WORLD_CHUNK_SIDE {
        for lx in 0..WORLD_CHUNK_SIDE {
            let world = [
                coord[0] * WORLD_CHUNK_SIDE as i64 + lx as i64,
                coord[1] * WORLD_CHUNK_SIDE as i64 + ly as i64,
            ];
            if role_for(seed, world) != "Grass" {
                continue;
            }
            let hash = coord_hash(seed, world, 0x6f626a65637473);
            let tree_density = worldgen_plan::tree_density_percent(seed, world);
            let (kind, template) = if tree_density > 0
                && scatter_slot(seed, world, 6, 0x6d616a6f725f6f62, tree_density)
            {
                (
                    "certified biome object",
                    choose_template(&large_objects, hash >> 8),
                )
            } else if scatter_slot(seed, world, 9, 0x64657461696c5f6f, 28) {
                ("detail", choose_template(&details, hash))
            } else {
                continue;
            };
            let Some(template) = template else {
                continue;
            };
            let anchor = if template.footprint_cells == [1, 1] {
                world
            } else {
                [
                    world[0] - i64::from((template.footprint_cells[0] - 1) / 2),
                    world[1] - i64::from(template.footprint_cells[1] - 1),
                ]
            };
            if !footprint_is_grass(seed, anchor, template.footprint_cells) {
                continue;
            }
            let id = format!(
                "{}:{}:{}:{}:{}:{}",
                WORLD_GENERATOR_ID, coord[0], coord[1], lx, ly, template.template_id
            );
            let parts = template
                .parts
                .iter()
                .map(|part| WorldObjectPart {
                    offset_px: part.offset_px,
                    source: SourceBinding {
                        source_asset: part.source_path.clone(),
                        source_rect: part.source_rect_px,
                    },
                })
                .collect();
            objects.push(WorldObject {
                id,
                template_id: template.template_id.clone(),
                label: format!("Worldgen {kind}: {}", template.label),
                layer: template.layer.clone(),
                anchor_world: anchor,
                footprint_cells: template.footprint_cells,
                parts,
                generator_id: WORLD_GENERATOR_ID.into(),
            });
        }
    }
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
    fn summer_flatworld_cells_never_resolve_from_cliff_sheets() {
        let a = authority();
        let mut w = WorldDocument::new(73);
        w.materialize_3x3([0, 0], &a).unwrap();
        assert!(w
            .chunks
            .iter()
            .flat_map(|chunk| &chunk.generated_cells)
            .all(|cell| cell.source.source_asset == "Terrain/terrain_summer.png"));

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
    fn quarantining_large_generated_objects_removes_uncertified_composites() {
        let mut w = WorldDocument::new(103);
        w.chunks.push(WorldChunk {
            coord: [0, 0],
            generator_id: WORLD_GENERATOR_ID.into(),
            generated_cells: Vec::new(),
            generated_objects: vec![WorldObject {
                id: "large".into(),
                template_id: "large".into(),
                label: "large".into(),
                layer: "Objects".into(),
                anchor_world: [0, 0],
                footprint_cells: [3, 4],
                parts: vec![WorldObjectPart {
                    offset_px: [0, 0],
                    source: SourceBinding {
                        source_asset: "Terrain/trees_summer.png".into(),
                        source_rect: [0, 0, 96, 128],
                    },
                }],
                generator_id: WORLD_GENERATOR_ID.into(),
            }],
        });
        assert_eq!(w.quarantine_uncertified_large_objects(&authority()), 1);
        assert!(w.chunks[0].generated_objects.is_empty());
    }
}
