//! Canonical Havenwild Bevy authority for existing ElizaWy source art.
//! World composition may select/place these records; it never creates image assets.

use crate::{scene_v2::SourceBinding, source_catalog::SourceCatalog};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub const RUNTIME_SCHEMA: &str = "havenwild.bevy.elizawy.runtime_index.v1";

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityCounts {
    pub runtime_source_images: usize,
    pub historical_source_regions: usize,
    pub canonical_runtime_regions: usize,
    pub fixture_role_regions: usize,
    pub object_templates: usize,
    pub historical_review_clusters: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityPolicy {
    pub art_authority: String,
    pub generated_artwork_allowed: bool,
    pub world_generation_meaning: String,
    pub source_pixels_immutable: bool,
    pub runtime_placement_must_reference_canonical_source_region: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeImage {
    pub asset_id: String,
    pub source_path: String,
    pub source_sha256: String,
    pub image_size_px: [u32; 2],
    pub family: String,
    pub runtime_placeable: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleEvidence {
    pub role: String,
    pub count: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRegion {
    pub canonical_region_id: String,
    pub source_asset_id: String,
    pub source_path: String,
    pub source_rect_px: [u32; 4],
    pub family_hints: Vec<String>,
    pub seasons: Vec<String>,
    pub fixture_role_evidence: Vec<RoleEvidence>,
    pub fixture_use_count: usize,
    pub semantic_authority: String,
    pub runtime_art_authority: String,
    pub generated_artwork: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteEntry {
    pub canonical_region_id: String,
    pub source_path: String,
    pub source_rect_px: [u32; 4],
    pub weight: usize,
}
impl PaletteEntry {
    pub fn binding(&self) -> SourceBinding {
        SourceBinding {
            source_asset: self.source_path.clone(),
            source_rect: self.source_rect_px,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CornerCompositePart {
    /// Canonical 32x32 source region that owns this exact source-pixel slice.
    /// The part may reference a smaller rectangle inside that region; no new
    /// artwork is generated or written back to the source atlas.
    pub canonical_region_id: String,
    pub source_path: String,
    pub source_rect_px: [u32; 4],
    pub destination_rect_px: [u32; 4],
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CornerCompositeRecipe {
    pub parts: Vec<CornerCompositePart>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummerVisualPart {
    pub source: SourceBinding,
    /// Destination rectangle inside one logical 32x32 rendered terrain cell.
    pub destination_rect_px: [u32; 4],
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SummerSourceCatalogSummary {
    pub donor_complete_map: String,
    pub donor_topology: String,
    pub total_cells: usize,
    pub mapped_non_transparent_cells: usize,
    pub non_transparent_cells: usize,
    pub structural_transparent_cells: usize,
    pub unused_transparent_cells: usize,
    pub primary_groups: usize,
    pub topology_families: usize,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SummerSourceGroup {
    pub group_label: String,
    pub classification: String,
    pub cell_count: usize,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub runtime_ids: Vec<String>,
    #[serde(default)]
    pub pcg_use: Vec<String>,
    #[serde(default)]
    pub cells: Vec<PaletteEntry>,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SummerAnimation {
    pub frame_duration_ms: u64,
    pub phase_mode: String,
    pub source_authority: String,
    pub visual_certification: String,
    #[serde(default)]
    pub frames: Vec<PaletteEntry>,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SummerFlatworldProfile {
    pub schema: String,
    pub version: u32,
    pub source_atlas: String,
    #[serde(default)]
    pub source_atlas_sha256: String,
    #[serde(default)]
    pub source_catalog: SummerSourceCatalogSummary,
    pub safe_fill: BTreeMap<String, PaletteEntry>,
    #[serde(default)]
    pub transition_families: BTreeMap<String, BTreeMap<String, PaletteEntry>>,
    #[serde(default)]
    pub corner_recipes: BTreeMap<String, PaletteEntry>,
    #[serde(default)]
    pub corner_composites: BTreeMap<String, CornerCompositeRecipe>,
    #[serde(default)]
    pub source_groups: BTreeMap<String, SummerSourceGroup>,
    #[serde(default)]
    pub fill_variants: BTreeMap<String, Vec<PaletteEntry>>,
    #[serde(default)]
    pub animations: BTreeMap<String, SummerAnimation>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePart {
    pub offset_px: [i32; 2],
    pub canonical_region_id: String,
    pub source_path: String,
    pub source_rect_px: [u32; 4],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectTemplate {
    pub template_id: String,
    pub label: String,
    pub layer: String,
    pub footprint_cells: [u32; 2],
    pub parts: Vec<TemplatePart>,
    pub observed_instances: usize,
    pub authority: String,
    pub worldgen_eligible: bool,
    pub default_collision: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalObjectPartOverride {
    source_path: String,
    source_rect_px: [u32; 4],
    offset_px: [i32; 2],
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalObjectTemplateOverride {
    template_id: String,
    parts: Vec<LocalObjectPartOverride>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalObjectOverrides {
    schema: String,
    version: u32,
    templates: Vec<LocalObjectTemplateOverride>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetAuthority {
    pub schema: String,
    pub version: u32,
    pub policy: AuthorityPolicy,
    pub lane: String,
    pub lane_sha256: String,
    pub counts: AuthorityCounts,
    pub runtime_source_images: Vec<RuntimeImage>,
    pub canonical_runtime_regions: Vec<RuntimeRegion>,
    pub fixture_role_palette: BTreeMap<String, Vec<PaletteEntry>>,
    pub fixture_topology_palette: BTreeMap<String, Vec<PaletteEntry>>,
    pub object_templates: Vec<ObjectTemplate>,
    #[serde(skip)]
    pub summer_flatworld: SummerFlatworldProfile,
    #[serde(skip)]
    local_source_regions: BTreeSet<(String, [u32; 4])>,
    #[serde(skip)]
    locally_certified_object_templates: BTreeSet<String>,
}

impl AssetAuthority {
    pub fn load(path: &Path, catalog: &SourceCatalog) -> Result<Self, String> {
        let raw =
            fs::read(path).map_err(|e| format!("Cannot read ElizaWy runtime authority: {e}"))?;
        let mut authority: Self = serde_json::from_slice(&raw)
            .map_err(|e| format!("Cannot parse ElizaWy runtime authority: {e}"))?;
        let content_root = path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| {
                "Cannot resolve content root for Summer flatworld authority".to_owned()
            })?;
        let flatworld_path = content_root
            .join("terrain")
            .join("recipes")
            .join("summer_flatworld_runtime.v1.json");
        let flatworld_raw = fs::read(&flatworld_path)
            .map_err(|e| format!("Cannot read Summer flatworld authority: {e}"))?;
        authority.summer_flatworld = serde_json::from_slice(&flatworld_raw)
            .map_err(|e| format!("Cannot parse Summer flatworld authority: {e}"))?;

        // A hydrated Tiled reference is local mapping metadata only. It never
        // supplies artwork: every entry must still resolve back to an exact
        // canonical ElizaWy source region from Terrain/terrain_summer.png.
        if let Some(project_root) = content_root.parent() {
            let local_flatworld = project_root
                .join(".forgepy")
                .join("terrain")
                .join("summer_flatworld_runtime.local.json");
            if local_flatworld.is_file() {
                let raw = fs::read(&local_flatworld)
                    .map_err(|e| format!("Cannot read hydrated Summer flatworld mapping: {e}"))?;
                let local: SummerFlatworldProfile = serde_json::from_slice(&raw)
                    .map_err(|e| format!("Cannot parse hydrated Summer flatworld mapping: {e}"))?;
                // Never let an older pixel-inferred/local profile replace the
                // newer checked-in source-derived Native authority. Version 3+
                // local profiles may still intentionally refine the same contract.
                if local.version >= authority.summer_flatworld.version {
                    authority.summer_flatworld = local;
                }
            }
        }
        if let Some(project_root) = content_root.parent() {
            let object_bounds = project_root
                .join(".forgepy")
                .join("terrain")
                .join("worldgen_object_bounds.local.json");
            if object_bounds.is_file() {
                let raw = fs::read(&object_bounds)
                    .map_err(|e| format!("Cannot read local object-bound authority: {e}"))?;
                let overrides: LocalObjectOverrides = serde_json::from_slice(&raw)
                    .map_err(|e| format!("Cannot parse local object-bound authority: {e}"))?;
                if overrides.schema != "havenwild.terrain.worldgen_object_bounds.v1" {
                    return Err("Invalid local object-bound authority identity".into());
                }
                // Version 1 was produced by the experimental connected-alpha crop
                // pass and could retain guide bars / neighboring pixels. It is
                // deliberately quarantined. Only a future source-reviewed v2 may
                // re-enable generated large object templates.
                if overrides.version < 2 {
                    authority.validate(catalog)?;
                    return Ok(authority);
                }
                for override_template in overrides.templates {
                    let Some(template) = authority
                        .object_templates
                        .iter_mut()
                        .find(|template| template.template_id == override_template.template_id)
                    else {
                        return Err(format!(
                            "Local object-bound authority references unknown template: {}",
                            override_template.template_id
                        ));
                    };
                    if template.parts.len() != override_template.parts.len() {
                        return Err(format!(
                            "Local object-bound part count differs for {}",
                            template.template_id
                        ));
                    }
                    for (part, correction) in template
                        .parts
                        .iter_mut()
                        .zip(override_template.parts.iter())
                    {
                        if part.source_path != correction.source_path
                            || !catalog
                                .contains_rect(&correction.source_path, correction.source_rect_px)
                        {
                            return Err(format!(
                                "Invalid local source crop for {}",
                                template.template_id
                            ));
                        }
                        part.source_rect_px = correction.source_rect_px;
                        part.offset_px = correction.offset_px;
                        authority
                            .local_source_regions
                            .insert((correction.source_path.clone(), correction.source_rect_px));
                    }
                    authority
                        .locally_certified_object_templates
                        .insert(template.template_id.clone());
                }
            }
        }

        authority.validate(catalog)?;
        Ok(authority)
    }

    pub fn validate(&self, catalog: &SourceCatalog) -> Result<(), String> {
        if self.schema != RUNTIME_SCHEMA
            || self.version != 1
            || self.lane != "content/assets/authority/elizawy_asset_lane.v1.json"
            || self.lane_sha256.len() != 64
            || self.policy.generated_artwork_allowed
            || !self.policy.source_pixels_immutable
            || !self
                .policy
                .runtime_placement_must_reference_canonical_source_region
            || self.counts.runtime_source_images != self.runtime_source_images.len()
            || self.counts.canonical_runtime_regions != self.canonical_runtime_regions.len()
            || self.counts.object_templates != self.object_templates.len()
            || self.counts.runtime_source_images != catalog.image_count()
        {
            return Err("ElizaWy asset authority identity/count/policy mismatch".into());
        }

        let mut image_paths = BTreeSet::new();
        for image in &self.runtime_source_images {
            let Some(catalog_entry) = catalog.entry(&image.source_path) else {
                return Err(format!(
                    "Authority source image absent from canonical manifest: {}",
                    image.source_path
                ));
            };
            if !image.runtime_placeable
                || image.asset_id != format!("elizawy:{}", image.source_path)
                || image.source_sha256.len() != 64
                || image.image_size_px != catalog_entry.size
                || !image_paths.insert(image.source_path.as_str())
            {
                return Err(format!(
                    "Invalid/duplicate runtime image authority: {}",
                    image.source_path
                ));
            }
        }

        let mut region_ids = BTreeSet::new();
        for region in &self.canonical_runtime_regions {
            if region.generated_artwork
                || region.runtime_art_authority != "exact_elizawy_source_region"
                || !region_ids.insert(region.canonical_region_id.as_str())
                || !catalog.contains_rect(&region.source_path, region.source_rect_px)
                || region.source_asset_id != format!("elizawy:{}", region.source_path)
            {
                return Err(format!(
                    "Invalid canonical runtime region: {}",
                    region.canonical_region_id
                ));
            }
        }

        for (role, entries) in &self.fixture_role_palette {
            if role.is_empty() || entries.is_empty() {
                return Err("Empty fixture role palette".into());
            }
            for entry in entries {
                if entry.weight == 0
                    || !region_ids.contains(entry.canonical_region_id.as_str())
                    || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
                {
                    return Err(format!(
                        "Invalid source-backed palette entry for role {role}"
                    ));
                }
            }
        }
        for (key, entries) in &self.fixture_topology_palette {
            if key.is_empty() || entries.is_empty() {
                return Err("Empty fixture topology palette".into());
            }
            for entry in entries {
                if entry.weight == 0
                    || !region_ids.contains(entry.canonical_region_id.as_str())
                    || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
                {
                    return Err(format!("Invalid observed topology palette entry: {key}"));
                }
            }
        }

        if self.summer_flatworld.schema != "havenwild.terrain.summer_flatworld_runtime.v1"
            || !matches!(self.summer_flatworld.version, 1..=6)
            || self.summer_flatworld.source_atlas != "Terrain/terrain_summer.png"
            || self.summer_flatworld.safe_fill.len() < 3
        {
            return Err("Invalid Summer flatworld authority identity".into());
        }
        if !self.summer_flatworld.source_atlas_sha256.is_empty() {
            let Some(image) = self
                .runtime_source_images
                .iter()
                .find(|image| image.source_path == self.summer_flatworld.source_atlas)
            else {
                return Err(
                    "Summer flatworld source atlas is absent from runtime authority".into(),
                );
            };
            if image.source_sha256 != self.summer_flatworld.source_atlas_sha256 {
                return Err(
                    "Summer flatworld source hash does not match canonical ElizaWy authority"
                        .into(),
                );
            }
        }
        for role in ["Grass", "MudBank", "RiverWater"] {
            let Some(entry) = self.summer_flatworld.safe_fill.get(role) else {
                return Err(format!("Summer flatworld missing safe fill for {role}"));
            };
            if entry.source_path != self.summer_flatworld.source_atlas
                || !region_ids.contains(entry.canonical_region_id.as_str())
                || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
            {
                return Err(format!("Invalid Summer flatworld safe fill for {role}"));
            }
        }
        if self.summer_flatworld.version >= 4 {
            let source_cell_total: usize = self
                .summer_flatworld
                .source_groups
                .values()
                .map(|group| group.cell_count)
                .sum();
            if self.summer_flatworld.source_groups.len() != 41 || source_cell_total != 305 {
                return Err(format!(
                    "Summer complete runtime authority expected 41 groups / 305 cells, got {} / {}",
                    self.summer_flatworld.source_groups.len(),
                    source_cell_total
                ));
            }
            for (group_id, group) in &self.summer_flatworld.source_groups {
                if group.cell_count != group.cells.len() || group.cells.is_empty() {
                    return Err(format!("Invalid Summer source group count: {group_id}"));
                }
                for entry in &group.cells {
                    if entry.weight == 0
                        || entry.source_path != self.summer_flatworld.source_atlas
                        || !region_ids.contains(entry.canonical_region_id.as_str())
                        || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
                    {
                        return Err(format!("Invalid Summer source group entry: {group_id}"));
                    }
                }
            }
            for (role, expected) in [("Grass", 6usize), ("MudBank", 6), ("RiverWater", 1)] {
                let Some(entries) = self.summer_flatworld.fill_variants.get(role) else {
                    return Err(format!(
                        "Summer complete authority missing fill variants for {role}"
                    ));
                };
                if entries.len() != expected {
                    return Err(format!(
                        "Summer complete authority expected {expected} fill variants for {role}, got {}",
                        entries.len()
                    ));
                }
                for entry in entries {
                    if entry.weight == 0
                        || entry.source_path != self.summer_flatworld.source_atlas
                        || !region_ids.contains(entry.canonical_region_id.as_str())
                        || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
                    {
                        return Err(format!("Invalid Summer fill variant for {role}"));
                    }
                }
            }
            if self.summer_flatworld.version >= 5 {
                if self.summer_flatworld.animations.contains_key("RiverWater") {
                    return Err("RiverWater animation is not certified: RepeatableFill variants are static source/detail choices, not temporal frames".into());
                }
                if self.summer_flatworld.animations.values().any(|animation| {
                    animation.source_authority == "summer_water_fill_source_variants"
                }) {
                    return Err("RepeatableFill source variants cannot be promoted into animation authority".into());
                }
            } else {
                let Some(water) = self.summer_flatworld.animations.get("RiverWater") else {
                    return Err("Summer complete authority missing RiverWater animation".into());
                };
                if water.frame_duration_ms == 0
                    || water.frames.len() != 8
                    || water.source_authority != "summer_water_fill_source_variants"
                {
                    return Err("Invalid Summer RiverWater animation authority".into());
                }
                for entry in &water.frames {
                    if entry.source_path != self.summer_flatworld.source_atlas
                        || !region_ids.contains(entry.canonical_region_id.as_str())
                        || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
                    {
                        return Err("Invalid Summer RiverWater animation frame".into());
                    }
                }
            }
        }

        for (family, masks) in &self.summer_flatworld.transition_families {
            if !matches!(
                family.as_str(),
                "grass_dirt" | "grass_shallows" | "dirt_shallows"
            ) {
                return Err(format!(
                    "Unknown Summer flatworld transition family: {family}"
                ));
            }
            for (mask, entry) in masks {
                if u8::from_str_radix(mask, 16).is_err()
                    || entry.source_path != self.summer_flatworld.source_atlas
                    || !region_ids.contains(entry.canonical_region_id.as_str())
                    || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
                {
                    return Err(format!(
                        "Invalid Summer flatworld transition {family}:{mask}"
                    ));
                }
            }
        }
        for (corners, entry) in &self.summer_flatworld.corner_recipes {
            let roles = corners.split('|').collect::<Vec<_>>();
            if roles.len() != 4
                || roles
                    .iter()
                    .any(|role| !matches!(*role, "Grass" | "MudBank" | "RiverWater"))
                || entry.source_path != self.summer_flatworld.source_atlas
                || !region_ids.contains(entry.canonical_region_id.as_str())
                || !catalog.contains_rect(&entry.source_path, entry.source_rect_px)
            {
                return Err(format!("Invalid Summer flatworld corner recipe: {corners}"));
            }
        }
        for (corners, recipe) in &self.summer_flatworld.corner_composites {
            let roles = corners.split('|').collect::<Vec<_>>();
            if roles.len() != 4
                || roles
                    .iter()
                    .any(|role| !matches!(*role, "Grass" | "MudBank" | "RiverWater"))
                || recipe.parts.is_empty()
            {
                return Err(format!(
                    "Invalid Summer flatworld composite identity: {corners}"
                ));
            }
            for part in &recipe.parts {
                let Some(parent) = self
                    .canonical_runtime_regions
                    .iter()
                    .find(|region| region.canonical_region_id == part.canonical_region_id)
                else {
                    return Err(format!(
                        "Summer composite {corners} references an unknown canonical region"
                    ));
                };
                let [sx, sy, sw, sh] = part.source_rect_px;
                let [px, py, pw, ph] = parent.source_rect_px;
                let [dx, dy, dw, dh] = part.destination_rect_px;
                let source_inside_parent = part.source_path == self.summer_flatworld.source_atlas
                    && parent.source_path == part.source_path
                    && sx >= px
                    && sy >= py
                    && sx.checked_add(sw).is_some_and(|right| right <= px + pw)
                    && sy.checked_add(sh).is_some_and(|bottom| bottom <= py + ph);
                let destination_inside_cell = dw > 0
                    && dh > 0
                    && dx.checked_add(dw).is_some_and(|right| right <= 32)
                    && dy.checked_add(dh).is_some_and(|bottom| bottom <= 32);
                if sw == 0
                    || sh == 0
                    || !source_inside_parent
                    || !destination_inside_cell
                    || !catalog.contains_rect(&part.source_path, part.source_rect_px)
                {
                    return Err(format!(
                        "Invalid exact-source Summer composite part: {corners}"
                    ));
                }
            }
        }

        if self.summer_flatworld.version >= 4 && self.summer_flatworld_corner_count() != 81 {
            return Err(format!(
                "Summer complete runtime authority expected 81/81 corner states, got {}",
                self.summer_flatworld_corner_count()
            ));
        }

        let mut template_ids = BTreeSet::new();
        for template in &self.object_templates {
            if template.template_id.is_empty()
                || !template_ids.insert(template.template_id.as_str())
                || !template.worldgen_eligible
                || template.parts.is_empty()
                || template.footprint_cells.contains(&0)
            {
                return Err("Invalid/duplicate ElizaWy object template".into());
            }
            for part in &template.parts {
                if !region_ids.contains(part.canonical_region_id.as_str())
                    || !catalog.contains_rect(&part.source_path, part.source_rect_px)
                {
                    return Err(format!(
                        "Object template {} references non-canonical source region",
                        template.template_id
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn canonical_region(&self, source: &SourceBinding) -> Option<&RuntimeRegion> {
        self.canonical_runtime_regions.iter().find(|region| {
            region.source_path == source.source_asset && region.source_rect_px == source.source_rect
        })
    }
    pub fn contains_binding(&self, source: &SourceBinding) -> bool {
        self.canonical_region(source).is_some()
            || self
                .local_source_regions
                .contains(&(source.source_asset.clone(), source.source_rect))
    }
    pub fn contains_source_slice(&self, source: &SourceBinding) -> bool {
        if self.contains_binding(source) {
            return true;
        }
        let [sx, sy, sw, sh] = source.source_rect;
        if sw == 0 || sh == 0 {
            return false;
        }
        self.canonical_runtime_regions.iter().any(|region| {
            if region.source_path != source.source_asset {
                return false;
            }
            let [px, py, pw, ph] = region.source_rect_px;
            sx >= px
                && sy >= py
                && sx
                    .checked_add(sw)
                    .zip(px.checked_add(pw))
                    .is_some_and(|(source_right, parent_right)| source_right <= parent_right)
                && sy
                    .checked_add(sh)
                    .zip(py.checked_add(ph))
                    .is_some_and(|(source_bottom, parent_bottom)| source_bottom <= parent_bottom)
        })
    }
    pub fn locally_certified_object_template(&self, template_id: &str) -> bool {
        self.locally_certified_object_templates
            .contains(template_id)
    }
    pub fn locally_certified_object_template_count(&self) -> usize {
        self.locally_certified_object_templates.len()
    }
    pub fn unique_fixture_role(&self, source: &SourceBinding) -> Option<&str> {
        let region = self.canonical_region(source)?;
        (region.fixture_role_evidence.len() == 1)
            .then(|| region.fixture_role_evidence[0].role.as_str())
    }
    pub fn role_palette(&self, role: &str) -> Option<&[PaletteEntry]> {
        self.fixture_role_palette.get(role).map(Vec::as_slice)
    }
    pub fn topology_palette(&self, role: &str, mask: u8) -> Option<&[PaletteEntry]> {
        self.fixture_topology_palette
            .get(&format!("{role}:{mask:02x}"))
            .map(Vec::as_slice)
    }
    pub fn topology_supports_binding(&self, role: &str, mask: u8, source: &SourceBinding) -> bool {
        self.topology_palette(role, mask).is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry.source_path == source.source_asset
                    && entry.source_rect_px == source.source_rect
            })
        })
    }
    pub fn summer_source_catalog_summary(&self) -> &SummerSourceCatalogSummary {
        &self.summer_flatworld.source_catalog
    }
    pub fn summer_flatworld_fill(&self, role: &str) -> Option<&PaletteEntry> {
        self.summer_flatworld.safe_fill.get(role)
    }
    pub fn summer_source_group_count(&self) -> usize {
        self.summer_flatworld.source_groups.len()
    }
    pub fn summer_source_runtime_cell_count(&self) -> usize {
        self.summer_flatworld
            .source_groups
            .values()
            .map(|group| group.cell_count)
            .sum()
    }
    pub fn summer_flatworld_fill_variant_count(&self, role: &str) -> usize {
        self.summer_flatworld
            .fill_variants
            .get(role)
            .map(Vec::len)
            .unwrap_or(0)
    }
    pub fn summer_water_animation_frame_count(&self) -> usize {
        self.summer_flatworld
            .animations
            .get("RiverWater")
            .map(|animation| animation.frames.len())
            .unwrap_or(0)
    }
    pub fn summer_water_animation_frame_duration_ms(&self) -> u64 {
        self.summer_flatworld
            .animations
            .get("RiverWater")
            .map(|animation| animation.frame_duration_ms)
            .unwrap_or(0)
    }
    pub fn summer_flatworld_fill_for_world(
        &self,
        role: &str,
        seed: u64,
        world: [i64; 2],
    ) -> Option<&PaletteEntry> {
        let entries = self.summer_flatworld.fill_variants.get(role)?;
        if entries.is_empty() {
            return self.summer_flatworld_fill(role);
        }
        let total: usize = entries.iter().map(|entry| entry.weight).sum();
        if total == 0 {
            return self.summer_flatworld_fill(role);
        }
        let mut hash = seed
            ^ (world[0] as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (world[1] as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        hash ^= hash >> 30;
        hash = hash.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        hash ^= hash >> 27;
        hash = hash.wrapping_mul(0x94D0_49BB_1331_11EB);
        hash ^= hash >> 31;
        let mut cursor = (hash as usize) % total;
        for entry in entries {
            if cursor < entry.weight {
                return Some(entry);
            }
            cursor -= entry.weight;
        }
        entries.last()
    }
    pub fn summer_flatworld_animation_frame(
        &self,
        role: &str,
        elapsed_ms: u64,
    ) -> Option<&PaletteEntry> {
        let animation = self.summer_flatworld.animations.get(role)?;
        if animation.frames.is_empty() || animation.frame_duration_ms == 0 {
            return None;
        }
        let index = ((elapsed_ms / animation.frame_duration_ms) as usize) % animation.frames.len();
        animation.frames.get(index)
    }
    pub fn summer_flatworld_transition(&self, family: &str, mask: u8) -> Option<&PaletteEntry> {
        self.summer_flatworld
            .transition_families
            .get(family)
            .and_then(|masks| masks.get(&format!("{mask:02x}")))
    }
    pub fn summer_flatworld_corner(&self, corners: [&str; 4]) -> Option<&PaletteEntry> {
        self.summer_flatworld.corner_recipes.get(&corners.join("|"))
    }
    pub fn summer_flatworld_composite(&self, corners: [&str; 4]) -> Option<&CornerCompositeRecipe> {
        self.summer_flatworld
            .corner_composites
            .get(&corners.join("|"))
    }
    pub fn summer_flatworld_visual_parts(
        &self,
        corners: [&str; 4],
    ) -> Option<Vec<SummerVisualPart>> {
        if let Some(entry) = self.summer_flatworld_corner(corners) {
            return Some(vec![SummerVisualPart {
                source: entry.binding(),
                destination_rect_px: [0, 0, 32, 32],
            }]);
        }
        self.summer_flatworld_composite(corners).map(|recipe| {
            recipe
                .parts
                .iter()
                .map(|part| SummerVisualPart {
                    source: SourceBinding {
                        source_asset: part.source_path.clone(),
                        source_rect: part.source_rect_px,
                    },
                    destination_rect_px: part.destination_rect_px,
                })
                .collect()
        })
    }
    pub fn summer_flatworld_corner_count(&self) -> usize {
        self.summer_flatworld.safe_fill.len()
            + self.summer_flatworld.corner_recipes.len()
            + self.summer_flatworld.corner_composites.len()
    }
    pub fn summer_flatworld_pair_counts(&self) -> [usize; 3] {
        let mut counts = [0usize; 3];
        for key in self
            .summer_flatworld
            .corner_recipes
            .keys()
            .chain(self.summer_flatworld.corner_composites.keys())
        {
            let roles: BTreeSet<_> = key.split('|').collect();
            if roles.len() != 2 {
                continue;
            }
            if roles.contains("Grass") && roles.contains("MudBank") {
                counts[0] += 1;
            } else if roles.contains("Grass") && roles.contains("RiverWater") {
                counts[1] += 1;
            } else if roles.contains("MudBank") && roles.contains("RiverWater") {
                counts[2] += 1;
            }
        }
        counts
    }

    pub fn summer_flatworld_transition_counts(&self) -> [usize; 3] {
        [
            self.summer_flatworld
                .transition_families
                .get("grass_dirt")
                .map(BTreeMap::len)
                .unwrap_or(0),
            self.summer_flatworld
                .transition_families
                .get("grass_shallows")
                .map(BTreeMap::len)
                .unwrap_or(0),
            self.summer_flatworld
                .transition_families
                .get("dirt_shallows")
                .map(BTreeMap::len)
                .unwrap_or(0),
        ]
    }

    pub fn terrain_object_templates(&self) -> impl Iterator<Item = &ObjectTemplate> {
        self.object_templates.iter().filter(|template| {
            template.worldgen_eligible
                && template
                    .parts
                    .iter()
                    .all(|part| part.source_path.starts_with("Terrain/"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_authority_is_source_only_and_has_no_generated_artwork() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        let authority = AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap();
        assert_eq!(authority.counts.runtime_source_images, 320);
        assert_eq!(authority.counts.historical_source_regions, 1781);
        assert_eq!(authority.counts.canonical_runtime_regions, 2316);
        assert_eq!(authority.counts.fixture_role_regions, 64);
        assert_eq!(authority.lane_sha256.len(), 64);
        assert!(!authority.policy.generated_artwork_allowed);
        assert!(authority.role_palette("Grass").is_some());
        assert!(authority.role_palette("RiverWater").is_some());
        assert!(authority.role_palette("MudBank").is_some());
        assert!(!authority.fixture_topology_palette.is_empty());
        assert!(authority.terrain_object_templates().count() > 10);
        assert_eq!(authority.summer_source_group_count(), 41);
        assert_eq!(authority.summer_source_runtime_cell_count(), 305);
        assert_eq!(authority.summer_flatworld_corner_count(), 81);
        assert_eq!(authority.summer_flatworld_pair_counts(), [14, 14, 14]);
        assert_eq!(authority.summer_flatworld_fill_variant_count("Grass"), 6);
        assert_eq!(authority.summer_flatworld_fill_variant_count("MudBank"), 6);
        assert_eq!(
            authority.summer_flatworld_fill_variant_count("RiverWater"),
            1
        );
        assert_eq!(authority.summer_water_animation_frame_count(), 0);
        assert_eq!(authority.summer_water_animation_frame_duration_ms(), 0);
    }

    #[test]
    fn summer_complete_three_material_grammar_is_total_and_source_backed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        let authority = AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap();
        let roles = ["Grass", "MudBank", "RiverWater"];
        for nw in roles {
            for ne in roles {
                for sw in roles {
                    for se in roles {
                        if nw == ne && ne == sw && sw == se {
                            assert!(authority.summer_flatworld_fill(nw).is_some());
                            continue;
                        }
                        let parts = authority
                            .summer_flatworld_visual_parts([nw, ne, sw, se])
                            .unwrap_or_else(|| panic!("missing Summer state: {nw}|{ne}|{sw}|{se}"));
                        assert!(!parts.is_empty());
                        assert!(parts.iter().all(|part| {
                            part.source.source_asset == "Terrain/terrain_summer.png"
                        }));
                    }
                }
            }
        }
    }

    #[test]
    fn summer_fill_variation_is_deterministic_and_uses_multiple_grass_cells() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        let authority = AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap();
        let seed = 0x4841_5645_4e57_494c;
        let mut rects = BTreeSet::new();
        for y in 0..24 {
            for x in 0..24 {
                let a = authority
                    .summer_flatworld_fill_for_world("Grass", seed, [x, y])
                    .unwrap();
                let b = authority
                    .summer_flatworld_fill_for_world("Grass", seed, [x, y])
                    .unwrap();
                assert_eq!(a.source_rect_px, b.source_rect_px);
                rects.insert(a.source_rect_px);
            }
        }
        assert!(
            rects.len() >= 4,
            "grass variation collapsed to too few source cells"
        );
    }

    #[test]
    fn summer_composite_quadrants_are_canonical_source_slices() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        let authority = AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap();
        let parts = authority
            .summer_flatworld_visual_parts(["Grass", "Grass", "MudBank", "RiverWater"])
            .expect("three-material Summer state must resolve");
        assert_eq!(parts.len(), 4);
        assert!(parts.iter().all(|part| {
            part.source.source_asset == "Terrain/terrain_summer.png"
                && part.source.source_rect[2..] == [16, 16]
                && !authority.contains_binding(&part.source)
                && authority.contains_source_slice(&part.source)
        }));
        let escaped = SourceBinding {
            source_asset: "Terrain/terrain_summer.png".into(),
            source_rect: [511, 831, 16, 16],
        };
        assert!(!authority.contains_source_slice(&escaped));
    }

    #[test]
    fn summer_water_variants_are_static_and_not_animation_frames() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog =
            SourceCatalog::load(&root.join("content/catalog/core_source_manifest.json")).unwrap();
        let authority = AssetAuthority::load(
            &root.join("content/assets/authority/elizawy_runtime_index.v1.json"),
            &catalog,
        )
        .unwrap();
        assert_eq!(authority.summer_water_animation_frame_count(), 0);
        assert_eq!(authority.summer_water_animation_frame_duration_ms(), 0);
        assert!(authority
            .summer_flatworld_animation_frame("RiverWater", 10_000)
            .is_none());
        assert_eq!(
            authority.summer_flatworld_fill_variant_count("RiverWater"),
            1
        );
        let base = authority
            .summer_flatworld_fill_for_world("RiverWater", 42, [7, -3])
            .unwrap();
        assert_eq!(base.source_path, "Terrain/terrain_summer.png");
        assert_eq!(base.source_rect_px, [384, 512, 32, 32]);
    }
}
