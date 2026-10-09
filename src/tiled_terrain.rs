//! JaidynReiman / Tiled terrain authority bridge.
//!
//! This module consumes TSX through `rs-tiled` instead of reconstructing atlas
//! semantics from guessed source coordinates. Havenwild remains the world-state
//! authority; Tiled owns authored tile topology (Wang sets), tile metadata and
//! animation/collision evidence for the hydrated configured LPC derivative.
//!
//! M2D095B keeps exact authored Wang signatures as the primary authority and
//! adds a Havenwild-native sparse Corner solver for the two-material diagonal
//! cases that Jaidyn intentionally leaves unassigned. The solver derives visual
//! topology only; Havenwild semantic/world state remains unchanged.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

use crate::source_catalog::SourceCatalog;
use image::RgbaImage;

use tiled::{Loader, WangSetType};

mod native_wang;
pub use native_wang::NativeCornerSolve;

pub const TILE_SIZE_PX: u32 = 32;
pub const JAIDYN_RUNTIME_DIR: &str = "_jaidyn_tiled";
pub const SUMMER_TSX: &str = "lpc-tileset-terrain-summer.tsx";
pub const SPRING_TSX: &str = "lpc-tileset-terrain-spring.tsx";
pub const AUTUMN_TSX: &str = "lpc-tileset-terrain-autumn-old.tsx";
pub const WINTER_TSX: &str = "lpc-tileset-terrain-winter.tsx";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WangKind {
    Corner,
    Edge,
    Mixed,
}

impl WangKind {
    fn from_tiled(value: &WangSetType) -> Self {
        match value {
            WangSetType::Corner => Self::Corner,
            WangSetType::Edge => Self::Edge,
            WangSetType::Mixed => Self::Mixed,
        }
    }
}

fn canonical_wang_set_name(name: &str) -> String {
    let trimmed = name.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "summer terrain" | "spring terrain" | "autumn terrain" | "winter terrain" => {
            "Terrain".to_owned()
        }
        _ => trimmed.to_owned(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TiledAnimationFrame {
    pub tile_id: u32,
    pub duration_ms: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TiledTileMetadata {
    pub tile_id: u32,
    pub user_type: String,
    pub probability: f32,
    pub animation_frame_count: usize,
    pub animation_frames: Vec<TiledAnimationFrame>,
    pub has_collision: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WangAssignment {
    pub tile_id: u32,
    /// Tiled order: top, top-right, right, bottom-right, bottom,
    /// bottom-left, left, top-left. 0 means unset.
    pub wang_id: [u8; 8],
}

#[derive(Clone, Debug)]
pub struct WangAuthoritySet {
    /// Canonical Havenwild-facing Wang set name. Seasonal source labels such as
    /// `Winter Terrain` normalize to `Terrain`.
    pub name: String,
    /// Exact authored Tiled label retained for diagnostics/provenance.
    pub source_name: String,
    pub kind: WangKind,
    /// Tiled Wang IDs are one-based; index 0 here corresponds to Wang ID 1.
    pub materials: Vec<String>,
    pub assignments: Vec<WangAssignment>,
    exact: HashMap<[u8; 8], Vec<u32>>,
}

impl WangAuthoritySet {
    pub fn material_id(&self, material: &str) -> Option<u8> {
        self.materials
            .iter()
            .position(|name| name == material)
            .and_then(|index| u8::try_from(index + 1).ok())
    }

    pub fn exact_candidates(&self, wang_id: [u8; 8]) -> &[u32] {
        self.exact
            .get(&wang_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Deterministic *uniform* exact-candidate choice.
    ///
    /// This intentionally does not claim Tiled Terrain Brush parity when two
    /// exact candidates have different `probability` values. The M2D094B
    /// acceptance verifier detects that case and blocks runtime promotion until
    /// weighted selection is implemented.
    pub fn choose_exact_uniform(
        &self,
        wang_id: [u8; 8],
        seed: u64,
        world: [i32; 2],
    ) -> Option<u32> {
        let candidates = self.exact_candidates(wang_id);
        if candidates.is_empty() {
            return None;
        }
        let hash = stable_choice_hash(seed, world, wang_id);
        Some(candidates[(hash as usize) % candidates.len()])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TiledResolvedTile {
    /// Bevy AssetServer path relative to Havenwild's configured `assets/elizawy`
    /// asset root.
    pub tileset_asset_path: String,
    pub tile_id: u32,
    pub source_rect: AtlasRect,
}

#[derive(Clone, Debug)]
pub struct TiledTerrainAuthority {
    pub tsx_path: PathBuf,
    /// AssetServer path relative to Havenwild's `assets/elizawy` root.
    pub image_asset_path: String,
    pub image_size: [u32; 2],
    pub tile_size: [u32; 2],
    pub margin: u32,
    pub spacing: u32,
    pub columns: u32,
    pub tile_count: u32,
    pub sets: BTreeMap<String, WangAuthoritySet>,
    pub tiles: BTreeMap<u32, TiledTileMetadata>,
}

impl TiledTerrainAuthority {
    pub fn load_tsx(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let mut loader = Loader::new();
        let tileset = loader
            .load_tsx_tileset(path)
            .map_err(|error| format!("Cannot parse Tiled TSX {}: {error}", path.display()))?;

        if tileset.tile_width != TILE_SIZE_PX || tileset.tile_height != TILE_SIZE_PX {
            return Err(format!(
                "Unsupported configured LPC tile size {}x{} in {}; expected {}x{}",
                tileset.tile_width,
                tileset.tile_height,
                path.display(),
                TILE_SIZE_PX,
                TILE_SIZE_PX
            ));
        }
        if tileset.columns == 0 {
            return Err(format!(
                "Tiled TSX {} has zero atlas columns",
                path.display()
            ));
        }

        let image = tileset.image.as_ref().ok_or_else(|| {
            format!(
                "Tiled TSX {} is not a regular atlas tileset; one shared image is required",
                path.display()
            )
        })?;
        let image_name = image
            .source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Tiled TSX {} has a non-UTF8 image path", path.display()))?;
        let image_width = u32::try_from(image.width)
            .map_err(|_| format!("Negative image width in {}", path.display()))?;
        let image_height = u32::try_from(image.height)
            .map_err(|_| format!("Negative image height in {}", path.display()))?;
        if image_width == 0 || image_height == 0 {
            return Err(format!(
                "Tiled TSX {} has a zero-size atlas image",
                path.display()
            ));
        }

        let mut sets = BTreeMap::new();
        for set in &tileset.wang_sets {
            let mut assignments = set
                .wang_tiles
                .iter()
                .map(|(tile_id, wang)| WangAssignment {
                    tile_id: *tile_id,
                    wang_id: wang.wang_id.0,
                })
                .collect::<Vec<_>>();
            assignments.sort_by_key(|entry| entry.tile_id);

            let mut exact: HashMap<[u8; 8], Vec<u32>> = HashMap::new();
            for entry in &assignments {
                exact.entry(entry.wang_id).or_default().push(entry.tile_id);
            }
            for candidates in exact.values_mut() {
                candidates.sort_unstable();
            }

            let source_name = set.name.trim().to_owned();
            let authority = WangAuthoritySet {
                name: canonical_wang_set_name(&source_name),
                source_name,
                kind: WangKind::from_tiled(&set.wang_set_type),
                materials: set
                    .wang_colors
                    .iter()
                    .map(|color| color.name.clone())
                    .collect(),
                assignments,
                exact,
            };
            if sets.insert(authority.name.clone(), authority).is_some() {
                return Err(format!(
                    "Duplicate canonical Wang set name {:?} in configured TSX {}",
                    set.name,
                    path.display()
                ));
            }
        }

        let tiles = tileset
            .tiles()
            .map(|(tile_id, tile)| {
                (
                    tile_id,
                    TiledTileMetadata {
                        tile_id,
                        user_type: tile.user_type.clone(),
                        probability: tile.probability,
                        animation_frame_count: tile.animation.as_ref().map_or(0, Vec::len),
                        animation_frames: tile
                            .animation
                            .as_ref()
                            .map(|frames| {
                                frames
                                    .iter()
                                    .map(|frame| TiledAnimationFrame {
                                        tile_id: frame.tile_id,
                                        duration_ms: frame.duration,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        has_collision: tile.collision.is_some(),
                    },
                )
            })
            .collect();

        let authority = Self {
            tsx_path: path.to_path_buf(),
            image_asset_path: format!("{JAIDYN_RUNTIME_DIR}/{image_name}"),
            image_size: [image_width, image_height],
            tile_size: [tileset.tile_width, tileset.tile_height],
            margin: tileset.margin,
            spacing: tileset.spacing,
            columns: tileset.columns,
            tile_count: tileset.tilecount,
            sets,
            tiles,
        };
        authority.validate_assignment_bounds()?;
        Ok(authority)
    }

    pub fn set(&self, name: &str) -> Option<&WangAuthoritySet> {
        self.sets.get(name).or_else(|| {
            self.sets
                .values()
                .find(|set| set.source_name.eq_ignore_ascii_case(name))
        })
    }

    /// Return the source rectangle only when the tile is actually inside the
    /// configured atlas image. Tiled permits local IDs that are not trivially
    /// bounded by `tilecount`, so image bounds are the final authority here.
    pub fn source_rect(&self, tile_id: u32) -> Option<AtlasRect> {
        let stride_x = self.tile_size[0].checked_add(self.spacing)?;
        let stride_y = self.tile_size[1].checked_add(self.spacing)?;
        let column = tile_id % self.columns;
        let row = tile_id / self.columns;
        let x = self.margin.checked_add(column.checked_mul(stride_x)?)?;
        let y = self.margin.checked_add(row.checked_mul(stride_y)?)?;
        let right = x.checked_add(self.tile_size[0])?;
        let bottom = y.checked_add(self.tile_size[1])?;
        if right > self.image_size[0] || bottom > self.image_size[1] {
            return None;
        }
        Some(AtlasRect {
            x,
            y,
            width: self.tile_size[0],
            height: self.tile_size[1],
        })
    }

    fn validate_assignment_bounds(&self) -> Result<(), String> {
        for set in self.sets.values() {
            for assignment in &set.assignments {
                if self.source_rect(assignment.tile_id).is_none() {
                    return Err(format!(
                        "Wang set {:?} in {} references tile {} outside atlas {:?}",
                        set.name,
                        self.tsx_path.display(),
                        assignment.tile_id,
                        self.image_size
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn resolve_exact(
        &self,
        set_name: &str,
        wang_id: [u8; 8],
        seed: u64,
        world: [i32; 2],
    ) -> Option<TiledResolvedTile> {
        let tile_id = self
            .set(set_name)?
            .choose_exact_uniform(wang_id, seed, world)?;
        let source_rect = self.source_rect(tile_id)?;
        Some(TiledResolvedTile {
            tileset_asset_path: self.image_asset_path.clone(),
            tile_id,
            source_rect,
        })
    }

    /// Resolve a Tiled Corner Wang set from Havenwild corner semantics.
    ///
    /// Input order is `[north_west, north_east, south_west, south_east]`.
    /// No fuzzy matching or role guessing occurs: every semantic name must exist
    /// exactly in the selected Tiled Wang set, and the resulting exact Wang ID
    /// must have at least one authored candidate.
    pub fn resolve_corner_materials(
        &self,
        set_name: &str,
        corners: [&str; 4],
        seed: u64,
        world: [i32; 2],
    ) -> Option<TiledResolvedTile> {
        let set = self.set(set_name)?;
        if set.kind != WangKind::Corner {
            return None;
        }
        let nw = set.material_id(corners[0])?;
        let ne = set.material_id(corners[1])?;
        let sw = set.material_id(corners[2])?;
        let se = set.material_id(corners[3])?;
        // Tiled: top, top-right, right, bottom-right, bottom,
        // bottom-left, left, top-left.
        self.resolve_exact(set_name, [0, ne, 0, se, 0, sw, 0, nw], seed, world)
    }

    /// Resolve a semantic Corner-material vertex grid through the project-native
    /// sparse Wang solver. The input is row-major and uses authored Tiled material
    /// names. World/gameplay semantics are not modified; only a derived visual
    /// vertex grid may be repaired when Jaidyn has no exact checkerboard tile.
    pub fn solve_corner_material_grid(
        &self,
        set_name: &str,
        vertex_size: [u32; 2],
        semantic_materials: &[String],
        seed: u64,
    ) -> Result<NativeCornerSolve, String> {
        self.solve_corner_material_grid_with_origin(
            set_name,
            vertex_size,
            semantic_materials,
            seed,
            [0, 0],
        )
    }

    pub fn solve_corner_material_grid_with_origin(
        &self,
        set_name: &str,
        vertex_size: [u32; 2],
        semantic_materials: &[String],
        seed: u64,
        output_origin: [i32; 2],
    ) -> Result<NativeCornerSolve, String> {
        let set = self
            .set(set_name)
            .ok_or_else(|| format!("Unknown Tiled Wang set {set_name:?}"))?;
        if set.kind != WangKind::Corner {
            return Err(format!("Tiled Wang set {set_name:?} is not a Corner set"));
        }
        let mut ids = Vec::with_capacity(semantic_materials.len());
        for material in semantic_materials {
            let id = set.material_id(material).ok_or_else(|| {
                format!(
                    "Material {material:?} is not authored in Wang set {:?}",
                    set.source_name
                )
            })?;
            ids.push(id);
        }
        native_wang::solve_corner_grid_with_origin(set, vertex_size, &ids, seed, output_origin)
    }

    pub fn animated_tile_count(&self) -> usize {
        self.tiles
            .values()
            .filter(|tile| tile.animation_frame_count > 0)
            .count()
    }

    pub fn collision_tile_count(&self) -> usize {
        self.tiles
            .values()
            .filter(|tile| tile.has_collision)
            .count()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalSourceTile {
    pub source_asset: String,
    pub source_rect: [u32; 4],
}

/// One canonical original-art frame from a configured Tiled animation.
///
/// Jaidyn/Tiled owns timing/topology metadata; the rendered pixels remain an
/// exact RGBA match from the canonical ElizaWy source catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalAnimationFrame {
    pub source: CanonicalSourceTile,
    pub duration_ms: u32,
}

/// Renderer-ready animation clip whose every frame has been proven to map back
/// to canonical ElizaWy pixels. Partial clips are never promoted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalAnimationClip {
    pub source_tile_id: u32,
    pub frames: Vec<CanonicalAnimationFrame>,
    pub total_duration_ms: u64,
}

impl CanonicalAnimationClip {
    pub fn frame_index_at(&self, elapsed_ms: u64, phase_ms: u64) -> Option<usize> {
        if self.frames.is_empty() || self.total_duration_ms == 0 {
            return None;
        }
        let mut cursor = elapsed_ms
            .wrapping_add(phase_ms)
            .rem_euclid(self.total_duration_ms);
        for (index, frame) in self.frames.iter().enumerate() {
            let duration = u64::from(frame.duration_ms);
            if cursor < duration {
                return Some(index);
            }
            cursor = cursor.saturating_sub(duration);
        }
        Some(self.frames.len() - 1)
    }

    pub fn binding_at(&self, elapsed_ms: u64, phase_ms: u64) -> Option<&CanonicalSourceTile> {
        self.frame_index_at(elapsed_ms, phase_ms)
            .and_then(|index| self.frames.get(index))
            .map(|frame| &frame.source)
    }
}

#[derive(Clone, Debug)]
pub struct CanonicalTiledTerrainAuthority {
    pub tiled: TiledTerrainAuthority,
    canonical_by_tile: BTreeMap<u32, CanonicalSourceTile>,
    canonical_animation_by_tile: BTreeMap<u32, CanonicalAnimationClip>,
    pub terrain_assignment_count: usize,
    pub canonical_terrain_assignment_count: usize,
    pub animation_frame_count: usize,
    pub canonical_animation_frame_count: usize,
    pub canonical_animation_clip_count: usize,
    pub incomplete_animation_clip_count: usize,
}

fn canonical_season_rank(path: &str) -> u8 {
    let lower = path.to_ascii_lowercase();
    if lower == "terrain/terrain_summer.png" {
        0
    } else if lower.contains("_summer") {
        1
    } else if lower.contains("_spring") || lower.contains("_autumn") || lower.contains("_winter") {
        3
    } else {
        2
    }
}

fn rgba_tile_bytes(image: &RgbaImage, x: u32, y: u32) -> Option<Vec<u8>> {
    if x.checked_add(TILE_SIZE_PX)? > image.width() || y.checked_add(TILE_SIZE_PX)? > image.height()
    {
        return None;
    }
    let stride = usize::try_from(image.width()).ok()?.checked_mul(4)?;
    let start_x = usize::try_from(x).ok()?.checked_mul(4)?;
    let mut out = Vec::with_capacity((TILE_SIZE_PX * TILE_SIZE_PX * 4) as usize);
    let raw = image.as_raw();
    for row in y..y + TILE_SIZE_PX {
        let start = usize::try_from(row)
            .ok()?
            .checked_mul(stride)?
            .checked_add(start_x)?;
        let end = start.checked_add((TILE_SIZE_PX * 4) as usize)?;
        out.extend_from_slice(raw.get(start..end)?);
    }
    Some(out)
}

fn rgba_tile_visible(bytes: &[u8]) -> bool {
    bytes.chunks_exact(4).any(|rgba| rgba[3] != 0)
}

fn load_rgba(path: &Path) -> Result<RgbaImage, String> {
    image::ImageReader::open(path)
        .map_err(|error| format!("Cannot open image {}: {error}", path.display()))?
        .decode()
        .map_err(|error| format!("Cannot decode PNG {}: {error}", path.display()))
        .map(|image| image.to_rgba8())
}

impl CanonicalTiledTerrainAuthority {
    /// Build the live Summer terrain bridge without promoting the configured
    /// derivative atlas to art authority. Jaidyn owns Wang topology/animation
    /// metadata; rendered pixels come only from exact RGBA matches against the
    /// canonical ElizaWy Terrain catalog.
    pub fn load_summer(root: &Path, catalog: &SourceCatalog) -> Result<Self, String> {
        let asset_root = root.join("assets/elizawy");
        let tsx_path = asset_root.join(JAIDYN_RUNTIME_DIR).join(SUMMER_TSX);
        let tiled = TiledTerrainAuthority::load_tsx(&tsx_path)?;
        let derivative_path = asset_root.join(&tiled.image_asset_path);
        let derivative = load_rgba(&derivative_path)?;
        if derivative.dimensions() != (tiled.image_size[0], tiled.image_size[1]) {
            return Err(format!(
                "Configured Summer atlas dimensions differ from TSX: {:?} vs {:?}",
                derivative.dimensions(),
                tiled.image_size
            ));
        }

        let mut exact_pixels: HashMap<Vec<u8>, Vec<CanonicalSourceTile>> = HashMap::new();
        for entry in catalog.terrain_entries() {
            if canonical_season_rank(&entry.path) >= 3 {
                continue;
            }
            let source_path = asset_root.join(&entry.path);
            if !source_path.is_file() {
                continue;
            }
            let image = load_rgba(&source_path)?;
            if image.dimensions() != (entry.size[0], entry.size[1]) {
                return Err(format!(
                    "Canonical Terrain image dimensions differ from manifest for {}",
                    entry.path
                ));
            }
            let cells_x = entry.size[0] / TILE_SIZE_PX;
            let cells_y = entry.size[1] / TILE_SIZE_PX;
            for cy in 0..cells_y {
                for cx in 0..cells_x {
                    let x = cx * TILE_SIZE_PX;
                    let y = cy * TILE_SIZE_PX;
                    let Some(bytes) = rgba_tile_bytes(&image, x, y) else {
                        continue;
                    };
                    if !rgba_tile_visible(&bytes) {
                        continue;
                    }
                    exact_pixels
                        .entry(bytes)
                        .or_default()
                        .push(CanonicalSourceTile {
                            source_asset: entry.path.clone(),
                            source_rect: [x, y, TILE_SIZE_PX, TILE_SIZE_PX],
                        });
                }
            }
        }
        for candidates in exact_pixels.values_mut() {
            candidates.sort_by(|left, right| {
                (
                    canonical_season_rank(&left.source_asset),
                    left.source_asset.as_str(),
                    left.source_rect[1],
                    left.source_rect[0],
                )
                    .cmp(&(
                        canonical_season_rank(&right.source_asset),
                        right.source_asset.as_str(),
                        right.source_rect[1],
                        right.source_rect[0],
                    ))
            });
        }

        let terrain = tiled
            .set("Terrain")
            .ok_or_else(|| "Configured Summer TSX has no Terrain Wang set".to_owned())?;
        let mut needed = std::collections::BTreeSet::new();
        for assignment in &terrain.assignments {
            needed.insert(assignment.tile_id);
            if let Some(tile) = tiled.tiles.get(&assignment.tile_id) {
                for frame in &tile.animation_frames {
                    needed.insert(frame.tile_id);
                }
            }
        }

        let mut canonical_by_tile = BTreeMap::new();
        for tile_id in needed {
            let Some(rect) = tiled.source_rect(tile_id) else {
                continue;
            };
            let Some(bytes) = rgba_tile_bytes(&derivative, rect.x, rect.y) else {
                continue;
            };
            if let Some(binding) = exact_pixels.get(&bytes).and_then(|items| items.first()) {
                canonical_by_tile.insert(tile_id, binding.clone());
            }
        }

        let terrain_assignment_count = terrain.assignments.len();
        let canonical_terrain_assignment_count = terrain
            .assignments
            .iter()
            .filter(|assignment| canonical_by_tile.contains_key(&assignment.tile_id))
            .count();
        let animation_frames = terrain
            .assignments
            .iter()
            .filter_map(|assignment| tiled.tiles.get(&assignment.tile_id))
            .flat_map(|tile| tile.animation_frames.iter())
            .collect::<Vec<_>>();
        let animation_frame_count = animation_frames.len();
        let canonical_animation_frame_count = animation_frames
            .iter()
            .filter(|frame| canonical_by_tile.contains_key(&frame.tile_id))
            .count();

        let mut canonical_animation_by_tile = BTreeMap::new();
        let mut incomplete_animation_clip_count = 0usize;
        for assignment in &terrain.assignments {
            let Some(tile) = tiled.tiles.get(&assignment.tile_id) else {
                continue;
            };
            if tile.animation_frames.is_empty() {
                continue;
            }
            let mut frames = Vec::with_capacity(tile.animation_frames.len());
            let mut complete = true;
            let mut total_duration_ms = 0u64;
            for frame in &tile.animation_frames {
                if frame.duration_ms == 0 {
                    complete = false;
                    break;
                }
                let Some(source) = canonical_by_tile.get(&frame.tile_id).cloned() else {
                    complete = false;
                    break;
                };
                total_duration_ms = total_duration_ms.saturating_add(u64::from(frame.duration_ms));
                frames.push(CanonicalAnimationFrame {
                    source,
                    duration_ms: frame.duration_ms,
                });
            }
            if complete && total_duration_ms > 0 && frames.len() == tile.animation_frames.len() {
                canonical_animation_by_tile.insert(
                    assignment.tile_id,
                    CanonicalAnimationClip {
                        source_tile_id: assignment.tile_id,
                        frames,
                        total_duration_ms,
                    },
                );
            } else {
                incomplete_animation_clip_count += 1;
            }
        }
        let canonical_animation_clip_count = canonical_animation_by_tile.len();

        Ok(Self {
            tiled,
            canonical_by_tile,
            canonical_animation_by_tile,
            terrain_assignment_count,
            canonical_terrain_assignment_count,
            animation_frame_count,
            canonical_animation_frame_count,
            canonical_animation_clip_count,
            incomplete_animation_clip_count,
        })
    }

    pub fn canonical_binding(&self, tile_id: u32) -> Option<&CanonicalSourceTile> {
        self.canonical_by_tile.get(&tile_id)
    }

    pub fn canonical_animation_clip(&self, tile_id: u32) -> Option<&CanonicalAnimationClip> {
        self.canonical_animation_by_tile.get(&tile_id)
    }

    /// Return the canonical still frame that can safely stand in for this
    /// configured tile. Animated tiles whose configured base tile does not have
    /// an exact original-pixel match may still render when the complete authored
    /// animation clip maps back to canonical ElizaWy frames.
    pub fn canonical_visual_binding(&self, tile_id: u32) -> Option<&CanonicalSourceTile> {
        self.canonical_binding(tile_id).or_else(|| {
            self.canonical_animation_clip(tile_id)
                .and_then(|clip| clip.frames.first())
                .map(|frame| &frame.source)
        })
    }

    pub fn tile_has_canonical_animation(&self, tile_id: u32) -> bool {
        self.canonical_animation_clip(tile_id).is_some()
    }

    pub fn terrain_tile_is_water_boundary(&self, tile_id: u32) -> bool {
        let Some(set) = self.tiled.set("Terrain") else {
            return false;
        };
        let Some(assignment) = set
            .assignments
            .iter()
            .find(|assignment| assignment.tile_id == tile_id)
        else {
            return false;
        };
        let mut material_ids = BTreeSet::new();
        let mut has_water = false;
        for material_id in assignment.wang_id.iter().copied().filter(|id| *id != 0) {
            material_ids.insert(material_id);
            let index = usize::from(material_id - 1);
            if matches!(
                set.materials.get(index).map(String::as_str),
                Some("Shallow Water" | "Deep Water")
            ) {
                has_water = true;
            }
        }
        // Keep homogeneous water fills static. Live terrain animation is reserved
        // for authored boundaries where motion communicates surf/depth change.
        has_water && material_ids.len() > 1
    }

    /// Stable synchronized phase for environmental terrain animation. Tiled
    /// starts matching tile animations on one shared clock; a linear per-cell
    /// offset made the coast read like chase lights. Keep every tile in the same
    /// authored cycle and use only a seed-stable global offset. The world argument
    /// remains in the contract so a future evidence-backed water-body phase can be
    /// added without changing callers.
    pub fn world_animation_phase_ms(seed: u64, _world: [i64; 2], period_ms: u64) -> u64 {
        if period_ms == 0 {
            return 0;
        }
        (seed ^ (seed >> 32)).wrapping_mul(17) % period_ms
    }

    pub fn canonical_animation_binding_at(
        &self,
        tile_id: u32,
        seed: u64,
        world: [i64; 2],
        elapsed_ms: u64,
    ) -> Option<&CanonicalSourceTile> {
        let clip = self.canonical_animation_clip(tile_id)?;
        let phase = Self::world_animation_phase_ms(seed, world, clip.total_duration_ms);
        clip.binding_at(elapsed_ms, phase)
    }

    pub fn havenwild_material(role: &str) -> Option<&'static str> {
        match role {
            "Grass" => Some("Grass"),
            "MudBank" => Some("Dirt"),
            "Sand" => Some("Sand"),
            "RiverWater" => Some("Shallow Water"),
            "DeepWater" => Some("Deep Water"),
            // These legacy Havenwild roles remain on the previous source-backed
            // resolver until exact source/demo evidence proves their Tiled identity.
            "WetSand" | "PebblePath" => None,
            _ => None,
        }
    }

    pub fn resolve_havenwild_corner_tile_id(
        &self,
        corners: [&str; 4],
        seed: u64,
        world: [i64; 2],
    ) -> Option<u32> {
        let set = self.tiled.set("Terrain")?;
        if set.kind != WangKind::Corner {
            return None;
        }
        let nw = set.material_id(Self::havenwild_material(corners[0])?)?;
        let ne = set.material_id(Self::havenwild_material(corners[1])?)?;
        let sw = set.material_id(Self::havenwild_material(corners[2])?)?;
        let se = set.material_id(Self::havenwild_material(corners[3])?)?;
        let wang_id = [0, ne, 0, se, 0, sw, 0, nw];
        let candidates = set
            .exact_candidates(wang_id)
            .iter()
            .copied()
            .filter(|tile_id| self.canonical_visual_binding(*tile_id).is_some())
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return None;
        }
        let world32 = [i32::try_from(world[0]).ok()?, i32::try_from(world[1]).ok()?];
        let hash = stable_choice_hash(seed, world32, wang_id);
        Some(candidates[(hash as usize) % candidates.len()])
    }

    pub fn resolve_havenwild_corner(
        &self,
        corners: [&str; 4],
        seed: u64,
        world: [i64; 2],
    ) -> Option<CanonicalSourceTile> {
        let tile_id = self.resolve_havenwild_corner_tile_id(corners, seed, world)?;
        self.canonical_visual_binding(tile_id).cloned()
    }

    /// Resolve an authored core-terrain tile and evaluate its canonical animation
    /// frame. The cached world renderer uses the full clip directly; this helper
    /// remains useful for deterministic sampling/tests.
    pub fn resolve_havenwild_corner_animated(
        &self,
        corners: [&str; 4],
        seed: u64,
        world: [i64; 2],
        elapsed_ms: u64,
    ) -> Option<CanonicalSourceTile> {
        let tile_id = self.resolve_havenwild_corner_tile_id(corners, seed, world)?;
        self.canonical_animation_binding_at(tile_id, seed, world, elapsed_ms)
            .cloned()
            .or_else(|| self.canonical_visual_binding(tile_id).cloned())
    }

    /// Solve one shared Havenwild semantic-material window through the native
    /// sparse Wang solver, then reselect every authored topology from candidates
    /// that have a canonical original-pixel binding. This is the live world
    /// topology path: repaired visual vertices are shared by neighboring tiles,
    /// while gameplay/world semantics remain untouched.
    pub fn solve_havenwild_corner_grid(
        &self,
        vertex_size: [u32; 2],
        havenwild_roles: &[String],
        seed: u64,
        output_origin: [i64; 2],
    ) -> Result<NativeCornerSolve, String> {
        let materials = havenwild_roles
            .iter()
            .map(|role| {
                Self::havenwild_material(role)
                    .map(str::to_owned)
                    .ok_or_else(|| {
                        format!("Havenwild role {role:?} is not live-certified for Tiled terrain")
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let origin = [
            i32::try_from(output_origin[0])
                .map_err(|_| "Tiled solve X origin is outside i32 range".to_owned())?,
            i32::try_from(output_origin[1])
                .map_err(|_| "Tiled solve Y origin is outside i32 range".to_owned())?,
        ];
        let mut solved = self.tiled.solve_corner_material_grid_with_origin(
            "Terrain",
            vertex_size,
            &materials,
            seed,
            origin,
        )?;
        let set = self
            .tiled
            .set("Terrain")
            .ok_or_else(|| "Configured Summer TSX has no Terrain Wang set".to_owned())?;
        let width = solved.width;
        let height = solved.height;
        let vertex_width = vertex_size[0];
        let vertex_index =
            |x: u32, y: u32| -> usize { (y as usize) * (vertex_width as usize) + x as usize };
        for y in 0..height {
            for x in 0..width {
                let corners = [
                    solved.visual_vertex_ids[vertex_index(x, y)],
                    solved.visual_vertex_ids[vertex_index(x + 1, y)],
                    solved.visual_vertex_ids[vertex_index(x, y + 1)],
                    solved.visual_vertex_ids[vertex_index(x + 1, y + 1)],
                ];
                let wang_id = [0, corners[1], 0, corners[3], 0, corners[2], 0, corners[0]];
                let candidates = set
                    .exact_candidates(wang_id)
                    .iter()
                    .copied()
                    .filter(|tile_id| self.canonical_visual_binding(*tile_id).is_some())
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    return Err(format!(
                        "Authored Tiled topology at world ({},{}) has no canonical ElizaWy pixel binding",
                        output_origin[0] + i64::from(x),
                        output_origin[1] + i64::from(y)
                    ));
                }
                let world = [
                    origin[0].saturating_add(x as i32),
                    origin[1].saturating_add(y as i32),
                ];
                let hash = stable_choice_hash(seed, world, wang_id);
                let index = (y as usize) * (width as usize) + x as usize;
                solved.tile_ids[index] = candidates[(hash as usize) % candidates.len()];
            }
        }
        Ok(solved)
    }

    pub fn mapped_fraction(&self) -> (usize, usize) {
        (
            self.canonical_terrain_assignment_count,
            self.terrain_assignment_count,
        )
    }
}

fn stable_choice_hash(seed: u64, world: [i32; 2], wang_id: [u8; 8]) -> u64 {
    // FNV-1a, intentionally local and deterministic across Rust versions.
    let mut hash = 0xcbf29ce484222325u64 ^ seed;
    for byte in world[0]
        .to_le_bytes()
        .into_iter()
        .chain(world[1].to_le_bytes())
        .chain(wang_id)
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn corner_signature_uses_tiled_clockwise_order() {
        let set = WangAuthoritySet {
            name: "Terrain".into(),
            source_name: "Terrain".into(),
            kind: WangKind::Corner,
            materials: vec!["Grass".into(), "Sand".into(), "Deep Water".into()],
            assignments: vec![WangAssignment {
                tile_id: 17,
                wang_id: [0, 1, 0, 3, 0, 2, 0, 1],
            }],
            exact: HashMap::from([([0, 1, 0, 3, 0, 2, 0, 1], vec![17])]),
        };
        assert_eq!(set.material_id("Grass"), Some(1));
        assert_eq!(set.material_id("Deep Water"), Some(3));
        assert_eq!(
            set.choose_exact_uniform([0, 1, 0, 3, 0, 2, 0, 1], 4, [2, 7]),
            Some(17)
        );
        assert_eq!(set.material_id("PebblePath"), None);
    }

    #[test]
    fn deterministic_variant_choice_does_not_depend_on_hashmap_order() {
        let mut exact = HashMap::new();
        exact.insert([0, 1, 0, 1, 0, 1, 0, 1], vec![5, 9, 12]);
        let set = WangAuthoritySet {
            name: "Terrain".into(),
            source_name: "Terrain".into(),
            kind: WangKind::Corner,
            materials: vec!["Grass".into()],
            assignments: Vec::new(),
            exact,
        };
        let first = set.choose_exact_uniform([0, 1, 0, 1, 0, 1, 0, 1], 99, [10, -4]);
        for _ in 0..10 {
            assert_eq!(
                first,
                set.choose_exact_uniform([0, 1, 0, 1, 0, 1, 0, 1], 99, [10, -4])
            );
        }
    }

    #[test]
    fn real_rs_tiled_parser_path_is_exercised() {
        let path =
            std::env::temp_dir().join(format!("havenwild_m2d094b_{}_good.tsx", std::process::id()));
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="fixture" tilewidth="32" tileheight="32" tilecount="4" columns="2">
 <image source="fixture.png" width="64" height="64"/>
 <wangsets>
  <wangset name="Winter Terrain" type="corner" tile="0">
   <wangcolor name="Grass" color="#00ff00" tile="0" probability="1"/>
   <wangcolor name="Deep Water" color="#0000ff" tile="1" probability="1"/>
   <wangtile tileid="1" wangid="0,1,0,2,0,1,0,1"/>
  </wangset>
 </wangsets>
 <tile id="1" type="water">
  <objectgroup><object id="1" x="0" y="0" width="32" height="32"/></objectgroup>
  <animation><frame tileid="1" duration="120"/></animation>
 </tile>
</tileset>"##;
        fs::write(&path, xml).expect("write fixture TSX");
        let authority = TiledTerrainAuthority::load_tsx(&path).expect("parse fixture TSX");
        let _ = fs::remove_file(&path);

        assert_eq!(authority.image_asset_path, "_jaidyn_tiled/fixture.png");
        assert_eq!(authority.image_size, [64, 64]);
        assert_eq!(authority.animated_tile_count(), 1);
        assert_eq!(authority.collision_tile_count(), 1);
        assert_eq!(
            authority.set("Terrain").unwrap().materials,
            ["Grass", "Deep Water"]
        );
        assert_eq!(
            authority.set("Terrain").unwrap().source_name,
            "Winter Terrain"
        );
        assert!(authority.set("Winter Terrain").is_some());
        let resolved = authority
            .resolve_corner_materials(
                "Terrain",
                ["Grass", "Grass", "Grass", "Deep Water"],
                7,
                [3, 4],
            )
            .expect("authored Wang signature should resolve");
        assert_eq!(resolved.tile_id, 1);
        assert_eq!(
            resolved.source_rect,
            AtlasRect {
                x: 32,
                y: 0,
                width: 32,
                height: 32
            }
        );
        assert!(authority
            .resolve_corner_materials(
                "Terrain",
                ["PebblePath", "Grass", "Grass", "Grass"],
                7,
                [3, 4],
            )
            .is_none());
    }

    #[test]
    fn out_of_atlas_wang_assignment_is_rejected() {
        let path =
            std::env::temp_dir().join(format!("havenwild_m2d094b_{}_bad.tsx", std::process::id()));
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="fixture" tilewidth="32" tileheight="32" tilecount="4" columns="2">
 <image source="fixture.png" width="64" height="64"/>
 <wangsets><wangset name="Terrain" type="corner" tile="0">
  <wangcolor name="Grass" color="#00ff00" tile="0" probability="1"/>
  <wangtile tileid="99" wangid="0,1,0,1,0,1,0,1"/>
 </wangset></wangsets>
</tileset>"##;
        fs::write(&path, xml).expect("write fixture TSX");
        let result = TiledTerrainAuthority::load_tsx(&path);
        let _ = fs::remove_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn canonical_bridge_prefers_primary_summer_terrain_sheet() {
        assert_eq!(canonical_season_rank("Terrain/terrain_summer.png"), 0);
        assert_eq!(canonical_season_rank("Terrain/cliff_summer.png"), 1);
        assert_eq!(canonical_season_rank("Terrain/Waterfall.png"), 2);
        assert_eq!(canonical_season_rank("Terrain/terrain_spring.png"), 3);
    }

    #[test]
    fn havenwild_core_roles_translate_without_guessing_legacy_shore_roles() {
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("Grass"),
            Some("Grass")
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("MudBank"),
            Some("Dirt")
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("Sand"),
            Some("Sand")
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("RiverWater"),
            Some("Shallow Water")
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("DeepWater"),
            Some("Deep Water")
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("WetSand"),
            None
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::havenwild_material("PebblePath"),
            None
        );
    }

    #[test]
    fn real_hydrated_summer_native_solver_covers_core_sparse_pairs_when_present() {
        let path = Path::new("assets")
            .join("elizawy")
            .join(JAIDYN_RUNTIME_DIR)
            .join(SUMMER_TSX);
        if !path.is_file() {
            // Source-only packages deliberately exclude hydrated third-party art.
            return;
        }
        let authority = TiledTerrainAuthority::load_tsx(&path).expect("load hydrated Summer TSX");
        let core_pairs = [
            ("Grass", "Sand"),
            ("Grass", "Dirt"),
            ("Grass", "Shallow Water"),
            ("Sand", "Shallow Water"),
            ("Dirt", "Shallow Water"),
            ("Shallow Water", "Deep Water"),
        ];

        for (index, (a, b)) in core_pairs.into_iter().enumerate() {
            let semantic = vec![
                a.to_owned(),
                b.to_owned(),
                a.to_owned(),
                b.to_owned(),
                b.to_owned(),
                a.to_owned(),
                b.to_owned(),
                a.to_owned(),
                a.to_owned(),
                b.to_owned(),
                a.to_owned(),
                b.to_owned(),
                b.to_owned(),
                a.to_owned(),
                b.to_owned(),
                a.to_owned(),
            ];
            let seed = 0x4857_4e47_u64 + index as u64;
            let first = authority
                .solve_corner_material_grid("Terrain", [4, 4], &semantic, seed)
                .unwrap_or_else(|error| panic!("{a}<->{b} native solve failed: {error}"));
            let second = authority
                .solve_corner_material_grid("Terrain", [4, 4], &semantic, seed)
                .expect("repeat native solve");
            assert_eq!(first, second, "{a}<->{b} must be deterministic");
            assert!(
                first.initial_unresolved_cells > 0,
                "fixture should exercise sparse diagonal topology"
            );
            assert_eq!(first.tile_ids.len(), 9);
            for tile_id in &first.tile_ids {
                assert!(
                    authority.source_rect(*tile_id).is_some(),
                    "{a}<->{b} resolved out-of-atlas tile {tile_id}"
                );
            }
        }
    }

    #[test]
    fn canonical_animation_clip_respects_authored_frame_durations() {
        let clip = CanonicalAnimationClip {
            source_tile_id: 9,
            frames: vec![
                CanonicalAnimationFrame {
                    source: CanonicalSourceTile {
                        source_asset: "Terrain/terrain_summer.png".into(),
                        source_rect: [0, 0, 32, 32],
                    },
                    duration_ms: 100,
                },
                CanonicalAnimationFrame {
                    source: CanonicalSourceTile {
                        source_asset: "Terrain/terrain_summer.png".into(),
                        source_rect: [32, 0, 32, 32],
                    },
                    duration_ms: 200,
                },
            ],
            total_duration_ms: 300,
        };
        assert_eq!(clip.frame_index_at(0, 0), Some(0));
        assert_eq!(clip.frame_index_at(99, 0), Some(0));
        assert_eq!(clip.frame_index_at(100, 0), Some(1));
        assert_eq!(clip.frame_index_at(299, 0), Some(1));
        assert_eq!(clip.frame_index_at(300, 0), Some(0));
        assert_eq!(clip.frame_index_at(0, 100), Some(1));
    }

    #[test]
    fn world_animation_phase_is_seed_stable_and_synchronized_across_space() {
        let period = 480;
        let origin = CanonicalTiledTerrainAuthority::world_animation_phase_ms(7, [0, 0], period);
        let east = CanonicalTiledTerrainAuthority::world_animation_phase_ms(7, [1, 0], period);
        let west = CanonicalTiledTerrainAuthority::world_animation_phase_ms(7, [-1, 0], period);
        let far = CanonicalTiledTerrainAuthority::world_animation_phase_ms(7, [91, -57], period);
        assert_eq!(origin, east);
        assert_eq!(origin, west);
        assert_eq!(origin, far);
        assert_eq!(
            origin,
            CanonicalTiledTerrainAuthority::world_animation_phase_ms(7, [0, 0], period)
        );
        assert_eq!(
            CanonicalTiledTerrainAuthority::world_animation_phase_ms(7, [3, -5], 0),
            0
        );
    }
}
