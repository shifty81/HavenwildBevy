//! Scene v2 foundation. Separately imported drafts; no implicit v1 fixture replacement.
//! Visual edits are sparse, semantic hints remain in the untouched legacy snapshot,
//! and source artwork and ElizaWy mapping/certification are never written here.
use crate::atomic_file::{write_atomic, write_atomic_new};
use crate::document::{Scene, Tile};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub const SCHEMA: &str = "havenwild.elizawy.scene.v2.draft";
const LEGACY_SCHEMA: &str = "havenwild.elizawy.scene.v1";

// No dependency/lockfile change. Only used to fingerprint the exact imported v1 bytes.
// FIPS 180-4 SHA-256, with standard empty/abc test vectors below.
fn sha256_hex(input: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let len_bits = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&len_bits.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, chunk) in block.chunks_exact(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes(chunk.try_into().expect("four-byte SHA word"));
        }
        for i in 16..64 {
            let a = w[i - 15];
            let b = w[i - 2];
            let s0 = a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3);
            let s1 = b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (dst, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *dst = dst.wrapping_add(value);
        }
    }
    h.iter()
        .map(|word| format!("{word:08x}"))
        .collect::<Vec<_>>()
        .join("")
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LayerId {
    Ground,
    TerrainDetails,
    Water,
    Elevation,
    Structures,
    Objects,
    Foreground,
}

impl LayerId {
    pub const ORDERED: [Self; 7] = [
        Self::Ground,
        Self::TerrainDetails,
        Self::Water,
        Self::Elevation,
        Self::Structures,
        Self::Objects,
        Self::Foreground,
    ];
    pub fn group(self) -> &'static str {
        match self {
            Self::Ground | Self::TerrainDetails | Self::Water | Self::Elevation => "Landscape",
            Self::Structures | Self::Objects => "World",
            Self::Foreground => "Foreground",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceBinding {
    pub source_asset: String,
    pub source_rect: [u32; 4],
}
impl SourceBinding {
    fn validate(&self) -> Result<(), String> {
        let path = &self.source_asset;
        if path.is_empty()
            || path.contains('\\')
            || path.contains(':')
            || path.starts_with('/')
            || path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || !path.ends_with(".png")
            || self.source_rect[2] == 0
            || self.source_rect[3] == 0
            || self.source_rect[0]
                .checked_add(self.source_rect[2])
                .is_none()
            || self.source_rect[1]
                .checked_add(self.source_rect[3])
                .is_none()
        {
            return Err("Unsafe or incomplete exact source-art binding".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CellOverride {
    pub index: usize,
    pub source: Option<SourceBinding>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualLayer {
    pub id: LayerId,
    pub edits: Vec<CellOverride>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyOrigin {
    pub schema: String,
    pub source_file: String,
    pub source_sha256: String,
    pub source_bytes: usize,
    pub imported_tile_count: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObjectOrigin {
    Manual,
    Generated { generator_id: String },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectPart {
    pub offset: [i32; 2],
    pub source: SourceBinding,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlacedObject {
    pub id: String,
    pub label: String,
    pub layer: LayerId,
    pub anchor: [i32; 2],
    pub footprint: [u32; 2],
    pub parts: Vec<ObjectPart>,
    pub origin: ObjectOrigin,
}
impl PlacedObject {
    /// A manually selected region is one complete source-bound instance. Do not
    /// infer an object's silhouette or remove allegedly empty/black source pixels.
    pub fn manual_region(
        id: String,
        layer: LayerId,
        anchor: [i32; 2],
        source: SourceBinding,
    ) -> Self {
        let width = source.source_rect[2].div_ceil(32);
        let height = source.source_rect[3].div_ceil(32);
        Self {
            id,
            label: format!(
                "Manual region: {} @ {:?}",
                source.source_asset, source.source_rect
            ),
            layer,
            anchor,
            footprint: [width, height],
            parts: vec![ObjectPart {
                offset: [0, 0],
                source,
            }],
            origin: ObjectOrigin::Manual,
        }
    }

    pub fn contains_cell(&self, cell: [usize; 2]) -> bool {
        (0..2).all(|axis| {
            i64::try_from(cell[axis]).ok().is_some_and(|coordinate| {
                let start = i64::from(self.anchor[axis]);
                coordinate >= start && coordinate - start < i64::from(self.footprint[axis])
            })
        })
    }

    /// The clicked cell is the vegetation/structure's foot (bottom center), not
    /// an arbitrary top-left stamp. This does not claim a verified art socket.
    pub fn origin_at_foot(cell: [usize; 2], footprint: [u32; 2]) -> Option<[i32; 2]> {
        if footprint.contains(&0) {
            return None;
        }
        let x = i64::try_from(cell[0]).ok()? - i64::from((footprint[0] - 1) / 2);
        let y = i64::try_from(cell[1]).ok()? - i64::from(footprint[1] - 1);
        Some([i32::try_from(x).ok()?, i32::try_from(y).ok()?])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemovalTombstone {
    pub object_id: String,
    pub generator_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectedRegion {
    pub id: String,
    pub bounds: [usize; 4],
    pub reason: String,
}

pub const COLLISION_MASK_SIDE: usize = 32;

/// Pixel-exact authored collision for one 32x32 world cell. A set bit is impassable.
/// This is gameplay/editor metadata only; source artwork is never modified or sampled
/// automatically into collision authority.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollisionMask32 {
    pub rows: Vec<u32>,
}
impl CollisionMask32 {
    pub fn empty() -> Self {
        Self {
            rows: vec![0; COLLISION_MASK_SIDE],
        }
    }
    pub fn full() -> Self {
        Self {
            rows: vec![u32::MAX; COLLISION_MASK_SIDE],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.rows.len() != COLLISION_MASK_SIDE {
            return Err("Collision mask must contain exactly 32 rows".into());
        }
        Ok(())
    }
    pub fn blocked(&self, x: usize, y: usize) -> bool {
        x < COLLISION_MASK_SIDE && y < COLLISION_MASK_SIDE && (self.rows[y] & (1u32 << x)) != 0
    }
    pub fn set_blocked(&mut self, x: usize, y: usize, blocked: bool) {
        if x >= COLLISION_MASK_SIDE || y >= COLLISION_MASK_SIDE {
            return;
        }
        let bit = 1u32 << x;
        if blocked {
            self.rows[y] |= bit;
        } else {
            self.rows[y] &= !bit;
        }
    }
    pub fn blocked_count(&self) -> u32 {
        self.rows.iter().map(|row| row.count_ones()).sum()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollisionCell {
    pub index: usize,
    pub mask: CollisionMask32,
}

/// Separate structural data channel. Empty on import: a v1 visual role is NOT
/// enough evidence to invent real height, flow, collision or traversal authority.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralCell {
    pub index: usize,
    pub elevation: Option<i16>,
    pub terrain_kind: Option<String>,
    pub water_flow: Option<[i8; 2]>,
    pub blocks_traversal: Option<bool>,
    pub connector: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneV2 {
    pub schema: String,
    pub scene_id: String,
    pub name: String,
    pub size: [usize; 2],
    pub tile_size: u32,
    pub legacy_origin: LegacyOrigin,
    /// Imported v1 data is a source-exact snapshot, not guessed layers or regenerated semantics.
    pub legacy_base: Vec<Tile>,
    #[serde(default)]
    pub structural_cells: Vec<StructuralCell>,
    #[serde(default)]
    pub collision_cells: Vec<CollisionCell>,
    pub visual_layers: Vec<VisualLayer>,
    pub objects: Vec<PlacedObject>,
    pub removed_generated_objects: Vec<RemovalTombstone>,
    pub protected_regions: Vec<ProtectedRegion>,
    pub next_object_serial: u64,
}
impl SceneV2 {
    pub fn preview_import(path: &Path) -> Result<Self, String> {
        let source_bytes = fs::read(path).map_err(|e| format!("Cannot read v1 source: {e}"))?;
        Self::from_v1_bytes(path, &source_bytes)
    }
    pub fn from_v1_bytes(path: &Path, bytes: &[u8]) -> Result<Self, String> {
        let source: Scene =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid v1 JSON: {e}"))?;
        source.validate()?;
        if !source.hidden_visual_samples.is_empty() {
            return Err("V2 import would discard assembled-scene object removals; retain the v1 derived draft until object migration is supported".into());
        }
        let name = path
            .file_name()
            .ok_or("Original scene has no file name")?
            .to_string_lossy()
            .into_owned();
        let hash = sha256_hex(bytes);
        let imported_tile_count = source.tiles.len();
        let doc = Self {
            schema: SCHEMA.into(),
            scene_id: format!("v1-import-{}", &hash[..16]),
            name: source.name,
            size: source.size,
            tile_size: source.tile_size,
            legacy_origin: LegacyOrigin {
                schema: LEGACY_SCHEMA.into(),
                source_file: name,
                source_sha256: hash,
                source_bytes: bytes.len(),
                imported_tile_count,
            },
            legacy_base: source.tiles,
            structural_cells: Vec::new(), // evidence is absent in the v1 source-only fixture
            collision_cells: Vec::new(), // collision authority is authored separately from source art
            visual_layers: LayerId::ORDERED
                .iter()
                .copied()
                .map(|id| VisualLayer { id, edits: vec![] })
                .collect(),
            objects: Vec::new(),
            removed_generated_objects: Vec::new(),
            protected_regions: Vec::new(),
            next_object_serial: 1,
        };
        doc.validate()?;
        Ok(doc)
    }
    pub fn draft_path(source_path: &Path) -> PathBuf {
        source_path.with_extension("v2.draft.json")
    }
    pub fn validate(&self) -> Result<(), String> {
        let count = self.size[0]
            .checked_mul(self.size[1])
            .ok_or("Scene size overflow")?;
        if self.schema != SCHEMA
            || self.size.contains(&0)
            || self.tile_size != 32
            || self.legacy_base.len() != count
            || self.legacy_origin.schema != LEGACY_SCHEMA
            || self.legacy_origin.imported_tile_count != count
            || self.legacy_origin.source_bytes == 0
            || self.legacy_origin.source_sha256.len() != 64
            || !self
                .legacy_origin
                .source_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
            || self.legacy_origin.source_file.is_empty()
            || self.legacy_origin.source_file.contains('/')
            || self.legacy_origin.source_file.contains('\\')
            || self.legacy_origin.source_file.contains(':')
            || self.legacy_origin.source_file == "."
            || self.legacy_origin.source_file == ".."
            || self.scene_id.is_empty()
            || self.next_object_serial == 0
            || self.visual_layers.len() != LayerId::ORDERED.len()
        {
            return Err("Invalid scene v2 schema, origin, size or layer structure".into());
        }
        for (index, layer) in self.visual_layers.iter().enumerate() {
            if layer.id != LayerId::ORDERED[index] {
                return Err("Scene v2 layers are out of canonical order".into());
            }
            let mut seen = BTreeSet::new();
            for edit in &layer.edits {
                if edit.index >= count || !seen.insert(edit.index) {
                    return Err("Out-of-bounds or duplicate cell override".into());
                }
                if let Some(source) = &edit.source {
                    source.validate()?;
                }
            }
        }
        let mut semantic_indices = BTreeSet::new();
        for cell in &self.structural_cells {
            if cell.index >= count
                || !semantic_indices.insert(cell.index)
                || cell.elevation.is_some_and(|v| !(0..=30).contains(&v))
                || cell
                    .water_flow
                    .is_some_and(|[x, y]| !(-1..=1).contains(&x) || !(-1..=1).contains(&y))
            {
                return Err(
                    "Invalid/duplicate structural cell or out-of-contract test elevation/flow"
                        .into(),
                );
            }
        }
        let mut collision_indices = BTreeSet::new();
        for collision in &self.collision_cells {
            if collision.index >= count || !collision_indices.insert(collision.index) {
                return Err("Invalid/duplicate pixel collision cell".into());
            }
            collision.mask.validate()?;
        }
        let mut ids = BTreeSet::new();
        for object in &self.objects {
            self.validate_object(object)?;
            if !ids.insert(object.id.as_str()) {
                return Err("Duplicate scene object ID".into());
            }
        }
        let mut removed = BTreeSet::new();
        for entry in &self.removed_generated_objects {
            if entry.object_id.is_empty()
                || entry.generator_id.is_empty()
                || ids.contains(entry.object_id.as_str())
                || !removed.insert(entry.object_id.as_str())
            {
                return Err("Invalid generated object removal record".into());
            }
        }
        let mut regions = BTreeSet::new();
        for region in &self.protected_regions {
            let [x, y, w, h] = region.bounds;
            if region.id.is_empty()
                || !regions.insert(&region.id)
                || w == 0
                || h == 0
                || x.checked_add(w).is_none_or(|right| right > self.size[0])
                || y.checked_add(h).is_none_or(|bottom| bottom > self.size[1])
            {
                return Err("Invalid protected region".into());
            }
        }
        Ok(())
    }
    /// Return the uppermost complete object instance whose rectangular footprint
    /// contains a scene cell. This is an editor-selection bound, NOT pixel alpha,
    /// collision, or certified asset-geometry inference.
    pub fn object_at(&self, cell: [usize; 2]) -> Option<&PlacedObject> {
        LayerId::ORDERED.iter().rev().find_map(|layer| {
            self.objects
                .iter()
                .rev()
                .find(|object| object.layer == *layer && object.contains_cell(cell))
        })
    }
    pub fn collision_mask(&self, index: usize) -> Option<&CollisionMask32> {
        self.collision_cells
            .iter()
            .find(|cell| cell.index == index)
            .map(|cell| &cell.mask)
    }

    fn replace_collision_mask(&mut self, index: usize, mask: Option<CollisionMask32>) {
        self.collision_cells.retain(|cell| cell.index != index);
        if let Some(mask) = mask {
            self.collision_cells.push(CollisionCell { index, mask });
            self.collision_cells.sort_by_key(|cell| cell.index);
        }
    }
    /// Side-effect-free drag destination probe. The same object bounds check is used
    /// by SceneCommand::MoveObject at commit; no ephemeral invalid scene is written.
    pub fn can_move_object_to(&self, id: &str, anchor: [i32; 2]) -> bool {
        self.objects
            .iter()
            .find(|object| object.id == id)
            .is_some_and(|object| {
                let mut proposed = object.clone();
                proposed.anchor = anchor;
                self.validate_object(&proposed).is_ok()
            })
    }

    fn validate_object(&self, object: &PlacedObject) -> Result<(), String> {
        if object.id.is_empty()
            || object.label.is_empty()
            || object.parts.is_empty()
            || object.footprint.contains(&0)
            || object.anchor.iter().any(|v| *v < 0)
            || (object.anchor[0] as u64 + object.footprint[0] as u64) > self.size[0] as u64
            || (object.anchor[1] as u64 + object.footprint[1] as u64) > self.size[1] as u64
        {
            return Err("Invalid object identity, dimensions, footprint or anchor".into());
        }
        if let ObjectOrigin::Generated { generator_id } = &object.origin {
            if generator_id.is_empty() {
                return Err("Generated object needs generator identity".into());
            }
        }
        for part in &object.parts {
            part.source.validate()?;
        }
        Ok(())
    }
    pub fn verify_legacy_source(&self, path: &Path) -> Result<(), String> {
        if path.file_name().and_then(|v| v.to_str())
            != Some(self.legacy_origin.source_file.as_str())
        {
            return Err("v1 filename changed since preview".into());
        }
        let raw = fs::read(path).map_err(|e| format!("Cannot recheck original v1 scene: {e}"))?;
        if raw.len() != self.legacy_origin.source_bytes
            || sha256_hex(&raw) != self.legacy_origin.source_sha256
        {
            return Err("Original v1 source changed since the preview. Refresh migration preview; no files were modified.".into());
        }
        Ok(())
    }
    /// A v2 draft is a separate file. Existing drafts are never overwritten by migration.
    pub fn save_new_draft(&self, source_path: &Path, draft_path: &Path) -> Result<(), String> {
        self.validate()?;
        self.verify_legacy_source(source_path)?;
        if source_path == draft_path || draft_path.exists() {
            return Err(
                "Destination is the source or a v2 draft already exists; no file replaced".into(),
            );
        }
        if draft_path != Self::draft_path(source_path) {
            return Err("Unexpected v2 draft path".into());
        }
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        write_atomic_new(draft_path, &bytes)?;
        Ok(())
    }
    /// Save only an editable local v2 draft; never rewrite the imported v1 source.
    /// Revalidate the exact original bytes so stale v2 state cannot overwrite another baseline.
    pub fn save_editable_draft(&self, original: &Path, derived: &Path) -> Result<(), String> {
        self.validate()?;
        self.verify_legacy_source(original)?;
        if original == derived
            || derived
                .file_name()
                .and_then(|name| name.to_str())
                .is_none_or(|name| {
                    name != "elizawy_mapping_certification.layered.draft.json"
                        && name != "summer_world.layered.draft.json"
                })
            || derived
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                != Some("derived")
        {
            return Err(
                "Source v1 and governed local layered-draft destination must stay separate".into(),
            );
        }
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        write_atomic(derived, &bytes)
    }

    pub fn read_draft(path: &Path) -> Result<Self, String> {
        let raw = fs::read(path).map_err(|e| e.to_string())?;
        let doc: Self = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        doc.validate()?;
        Ok(doc)
    }
    pub fn stable_cell_id(&self, index: usize) -> Option<String> {
        if index >= self.legacy_base.len() {
            return None;
        }
        Some(format!(
            "cell/{}/{}",
            index % self.size[0],
            index / self.size[0]
        ))
    }
    pub fn next_object_id(&self) -> String {
        format!("object:{}", self.next_object_serial)
    }
    fn layer(&self, layer: LayerId) -> &VisualLayer {
        &self.visual_layers[LayerId::ORDERED
            .iter()
            .position(|id| *id == layer)
            .expect("known layer")]
    }
    fn layer_mut(&mut self, layer: LayerId) -> &mut VisualLayer {
        &mut self.visual_layers[LayerId::ORDERED
            .iter()
            .position(|id| *id == layer)
            .expect("known layer")]
    }
    fn cell_override(&self, layer: LayerId, index: usize) -> Option<CellOverride> {
        self.layer(layer)
            .edits
            .iter()
            .find(|e| e.index == index)
            .cloned()
    }
    fn replace_cell_override(&mut self, layer: LayerId, index: usize, value: Option<CellOverride>) {
        let entries = &mut self.layer_mut(layer).edits;
        entries.retain(|e| e.index != index);
        if let Some(value) = value {
            entries.push(value);
            entries.sort_by_key(|e| e.index);
        }
    }
    /// Base semantics are retained; Ground edits can mask an inherited v1 image without erasing its role.
    pub fn resolved_legacy_tile(&self, index: usize) -> Option<Tile> {
        let mut tile = self.legacy_base.get(index)?.clone();
        if let Some(edit) = self.cell_override(LayerId::Ground, index) {
            match edit.source {
                Some(source) => {
                    tile.source_asset = source.source_asset;
                    tile.source_rect = source.source_rect;
                    tile.status = "derived".into();
                }
                None => tile.clear_source_binding(),
            }
        }
        Some(tile)
    }
}

#[derive(Clone, Debug)]
pub struct PaintCell {
    pub index: usize,
    pub source: Option<SourceBinding>,
}
#[derive(Clone, Debug)]
pub enum SceneCommand {
    PaintCells {
        layer: LayerId,
        cells: Vec<PaintCell>,
    },
    PlaceObject(PlacedObject),
    MoveObject {
        id: String,
        to: [i32; 2],
    },
    RemoveObject {
        id: String,
    },
    /// A manual semantic/collision edit. Original pixels and visual layers remain unchanged.
    SetStructuralCell {
        index: usize,
        after: Option<StructuralCell>,
    },
    /// Pixel-level collision mask for one 32x32 logical cell. None restores automatic/coarse collision.
    SetCollisionMask {
        index: usize,
        after: Option<CollisionMask32>,
    },
}
#[derive(Clone, Debug)]
enum Delta {
    Cell {
        layer: LayerId,
        index: usize,
        before: Option<CellOverride>,
        after: Option<CellOverride>,
    },
    Object {
        id: String,
        before: Option<PlacedObject>,
        after: Option<PlacedObject>,
    },
    Tombstone {
        id: String,
        before: Option<RemovalTombstone>,
        after: Option<RemovalTombstone>,
    },
    NextSerial {
        before: u64,
        after: u64,
    },
    Structural {
        index: usize,
        before: Option<StructuralCell>,
        after: Option<StructuralCell>,
    },
    Collision {
        index: usize,
        before: Option<CollisionMask32>,
        after: Option<CollisionMask32>,
    },
}
impl Delta {
    fn apply(&self, doc: &mut SceneV2, forward: bool) {
        match self {
            Self::Cell {
                layer,
                index,
                before,
                after,
            } => doc.replace_cell_override(
                *layer,
                *index,
                if forward {
                    after.clone()
                } else {
                    before.clone()
                },
            ),
            Self::Object { id, before, after } => {
                doc.objects.retain(|o| &o.id != id);
                let selected = if forward { after } else { before };
                if let Some(obj) = selected {
                    doc.objects.push(obj.clone());
                }
                doc.objects.sort_by(|a, b| a.id.cmp(&b.id));
            }
            Self::Tombstone { id, before, after } => {
                doc.removed_generated_objects.retain(|o| &o.object_id != id);
                let selected = if forward { after } else { before };
                if let Some(obj) = selected {
                    doc.removed_generated_objects.push(obj.clone());
                }
                doc.removed_generated_objects
                    .sort_by(|a, b| a.object_id.cmp(&b.object_id));
            }
            Self::Structural {
                index,
                before,
                after,
            } => {
                doc.structural_cells.retain(|cell| cell.index != *index);
                if let Some(cell) = if forward { after } else { before } {
                    doc.structural_cells.push(cell.clone());
                    doc.structural_cells.sort_by_key(|cell| cell.index);
                }
            }
            Self::Collision {
                index,
                before,
                after,
            } => {
                doc.replace_collision_mask(
                    *index,
                    if forward {
                        after.clone()
                    } else {
                        before.clone()
                    },
                );
            }
            Self::NextSerial { before, after } => {
                doc.next_object_serial = if forward { *after } else { *before }
            }
        }
    }
}
#[derive(Debug)]
pub struct Transaction {
    pub label: String,
    deltas: Vec<Delta>,
    before_revision: u64,
    after_revision: u64,
}
#[derive(Default, Debug)]
pub struct SceneHistory {
    undo: Vec<Transaction>,
    redo: Vec<Transaction>,
    next_revision: u64,
    current_revision: u64,
    saved_revision: u64,
}
impl SceneHistory {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn is_dirty(&self) -> bool {
        self.current_revision != self.saved_revision
    }
    pub fn mark_saved(&mut self) {
        self.saved_revision = self.current_revision;
    }
    pub fn execute(&mut self, doc: &mut SceneV2, command: SceneCommand) -> Result<(), String> {
        doc.validate()?;
        let (label, deltas) = match command {
            SceneCommand::PaintCells { layer, cells } => {
                if cells.is_empty() {
                    return Err("Empty paint transaction".into());
                }
                let mut touched = BTreeSet::new();
                let mut deltas = Vec::new();
                for change in cells {
                    if change.index >= doc.legacy_base.len() || !touched.insert(change.index) {
                        return Err("Invalid or duplicate brush cell; stroke not applied".into());
                    }
                    if let Some(source) = &change.source {
                        source.validate()?;
                    }
                    let before = doc.cell_override(layer, change.index);
                    let after = Some(CellOverride {
                        index: change.index,
                        source: change.source,
                    });
                    if before != after {
                        deltas.push(Delta::Cell {
                            layer,
                            index: change.index,
                            before,
                            after,
                        });
                    }
                }
                ("Paint scene cells".to_string(), deltas)
            }
            SceneCommand::PlaceObject(object) => {
                doc.validate_object(&object)?;
                if doc.objects.iter().any(|o| o.id == object.id)
                    || doc
                        .removed_generated_objects
                        .iter()
                        .any(|o| o.object_id == object.id)
                {
                    return Err(
                        "Object ID already exists or has a generated removal tombstone".into(),
                    );
                }
                let id = object.id.clone();
                let mut deltas = vec![Delta::Object {
                    id: id.clone(),
                    before: None,
                    after: Some(object),
                }];
                if let Some(serial) = id
                    .strip_prefix("object:")
                    .and_then(|s| s.parse::<u64>().ok())
                {
                    let after = doc.next_object_serial.max(serial.saturating_add(1));
                    if after != doc.next_object_serial {
                        deltas.push(Delta::NextSerial {
                            before: doc.next_object_serial,
                            after,
                        });
                    }
                }
                ("Place complete object".to_string(), deltas)
            }
            SceneCommand::MoveObject { id, to } => {
                let before = doc
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .cloned()
                    .ok_or("Object not found")?;
                let mut after = before.clone();
                after.anchor = to;
                doc.validate_object(&after)?;
                (
                    "Move complete object".to_string(),
                    if before == after {
                        Vec::new()
                    } else {
                        vec![Delta::Object {
                            id,
                            before: Some(before),
                            after: Some(after),
                        }]
                    },
                )
            }
            SceneCommand::RemoveObject { id } => {
                let object = doc
                    .objects
                    .iter()
                    .find(|o| o.id == id)
                    .cloned()
                    .ok_or("Object not found")?;
                let mut deltas = vec![Delta::Object {
                    id: id.clone(),
                    before: Some(object.clone()),
                    after: None,
                }];
                if let ObjectOrigin::Generated { generator_id } = &object.origin {
                    deltas.push(Delta::Tombstone {
                        id: id.clone(),
                        before: None,
                        after: Some(RemovalTombstone {
                            object_id: id,
                            generator_id: generator_id.clone(),
                        }),
                    });
                }
                ("Remove complete object".to_string(), deltas)
            }
            SceneCommand::SetStructuralCell { index, after } => {
                if index >= doc.legacy_base.len() {
                    return Err("Structural edit is outside the active scene".into());
                }
                if let Some(ref cell) = after {
                    if cell.index != index
                        || cell.elevation.is_some_and(|v| !(0..=30).contains(&v))
                        || cell
                            .water_flow
                            .is_some_and(|[x, y]| !(-1..=1).contains(&x) || !(-1..=1).contains(&y))
                    {
                        return Err("Invalid explicit structural edit".into());
                    }
                }
                let before = doc
                    .structural_cells
                    .iter()
                    .find(|cell| cell.index == index)
                    .cloned();
                (
                    "Edit structural cell".to_string(),
                    if before == after {
                        Vec::new()
                    } else {
                        vec![Delta::Structural {
                            index,
                            before,
                            after,
                        }]
                    },
                )
            }
            SceneCommand::SetCollisionMask { index, after } => {
                if index >= doc.legacy_base.len() {
                    return Err("Collision edit is outside the active scene".into());
                }
                if let Some(mask) = &after {
                    mask.validate()?;
                }
                let before = doc.collision_mask(index).cloned();
                (
                    "Edit pixel collision mask".to_string(),
                    if before == after {
                        Vec::new()
                    } else {
                        vec![Delta::Collision {
                            index,
                            before,
                            after,
                        }]
                    },
                )
            }
        };
        if deltas.is_empty() {
            return Err("No scene changes".into());
        }
        for delta in &deltas {
            delta.apply(doc, true);
        }
        debug_assert!(
            doc.validate().is_ok(),
            "validated scene command must leave a valid scene"
        );
        self.next_revision = self.next_revision.saturating_add(1);
        let tx = Transaction {
            label,
            deltas,
            before_revision: self.current_revision,
            after_revision: self.next_revision,
        };
        self.current_revision = self.next_revision;
        self.undo.push(tx);
        self.redo.clear();
        if self.undo.len() > 128 {
            self.undo.remove(0);
        }
        Ok(())
    }
    pub fn undo(&mut self, doc: &mut SceneV2) -> Option<String> {
        let tx = self.undo.pop()?;
        for delta in tx.deltas.iter().rev() {
            delta.apply(doc, false);
        }
        self.current_revision = tx.before_revision;
        let label = tx.label.clone();
        self.redo.push(tx);
        Some(label)
    }
    pub fn redo(&mut self, doc: &mut SceneV2) -> Option<String> {
        let tx = self.redo.pop()?;
        for delta in &tx.deltas {
            delta.apply(doc, true);
        }
        self.current_revision = tx.after_revision;
        let label = tx.label.clone();
        self.undo.push(tx);
        Some(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn example() -> SceneV2 {
        let mut scene = Scene {
            schema: LEGACY_SCHEMA.into(),
            name: "Test".into(),
            size: [3, 2],
            tile_size: 32,
            tiles: (0..6)
                .map(|_| Tile {
                    semantic_role_hint: "RiverWater".into(),
                    source_asset: "Terrain/terrain_summer.png".into(),
                    source_rect: [0, 0, 32, 32],
                    status: "draft".into(),
                })
                .collect(),
            hidden_visual_samples: Vec::new(),
        };
        // Distinct semantic hints survive import and source-only erasure unchanged.
        scene.tiles[3].semantic_role_hint = "Grass".into();
        let bytes = serde_json::to_vec(&scene).unwrap();
        SceneV2::from_v1_bytes(Path::new("example.scene.json"), &bytes).unwrap()
    }
    fn binding(x: u32) -> SourceBinding {
        SourceBinding {
            source_asset: "Terrain/terrain_summer.png".into(),
            source_rect: [x, 0, 32, 32],
        }
    }
    #[test]
    fn pixel_collision_mask_is_sparse_transactional_and_source_independent() {
        let mut doc = example();
        let original = doc.legacy_base.clone();
        let mut history = SceneHistory::default();
        let mut mask = CollisionMask32::full();
        mask.set_blocked(10, 11, false);
        history
            .execute(
                &mut doc,
                SceneCommand::SetCollisionMask {
                    index: 1,
                    after: Some(mask.clone()),
                },
            )
            .unwrap();
        assert_eq!(doc.collision_mask(1), Some(&mask));
        assert!(!doc.collision_mask(1).unwrap().blocked(10, 11));
        assert_eq!(doc.legacy_base, original);
        history.undo(&mut doc);
        assert!(doc.collision_mask(1).is_none());
        history.redo(&mut doc);
        assert_eq!(doc.collision_mask(1), Some(&mask));
        history
            .execute(
                &mut doc,
                SceneCommand::SetCollisionMask {
                    index: 1,
                    after: None,
                },
            )
            .unwrap();
        assert!(doc.collision_mask(1).is_none());
    }

    #[test]
    fn structural_collision_is_transactional_separate_from_source_pixels() {
        let mut doc = example();
        let original = doc.legacy_base.clone();
        let mut history = SceneHistory::default();
        let explicit = StructuralCell {
            index: 1,
            elevation: Some(1),
            terrain_kind: Some("grass".into()),
            water_flow: None,
            blocks_traversal: Some(true),
            connector: None,
        };
        history
            .execute(
                &mut doc,
                SceneCommand::SetStructuralCell {
                    index: 1,
                    after: Some(explicit.clone()),
                },
            )
            .unwrap();
        assert_eq!(doc.structural_cells, vec![explicit.clone()]);
        assert_eq!(doc.legacy_base, original);
        history.undo(&mut doc);
        assert!(doc.structural_cells.is_empty());
        history.redo(&mut doc);
        assert_eq!(doc.structural_cells, vec![explicit]);
        assert!(history
            .execute(
                &mut doc,
                SceneCommand::SetStructuralCell {
                    index: 1,
                    after: Some(StructuralCell {
                        index: 1,
                        elevation: Some(31),
                        terrain_kind: None,
                        water_flow: None,
                        blocks_traversal: None,
                        connector: None
                    })
                }
            )
            .is_err());
    }
    #[test]
    fn sha256_vectors_and_source_fingerprint() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let v2 = example();
        assert_eq!(v2.legacy_origin.source_sha256.len(), 64);
    }
    #[test]
    fn canonical_v1_fixture_import_roundtrip_preserves_all_tiles_and_roles() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("content/scenes/summer_river.scene.json");
        let before = fs::read(&path).unwrap();
        let v1: Scene = serde_json::from_slice(&before).unwrap();
        let v2 = SceneV2::from_v1_bytes(&path, &before).unwrap();
        assert_eq!(v2.size, [40, 28]);
        assert_eq!(v2.legacy_base.len(), 1120);
        for (old, new) in v1.tiles.iter().zip(&v2.legacy_base) {
            assert_eq!(old, new);
        }
        assert_eq!(v2.visual_layers.len(), 7);
        assert!(v2.visual_layers.iter().all(|l| l.edits.is_empty()));
        assert!(
            v2.structural_cells.is_empty(),
            "cannot fabricate elevation/collision/flow from visual-only v1"
        );
        assert_eq!(v2.stable_cell_id(41).as_deref(), Some("cell/1/1"));
        let roundtrip: SceneV2 = serde_json::from_slice(&serde_json::to_vec(&v2).unwrap()).unwrap();
        roundtrip.validate().unwrap();
        assert_eq!(roundtrip.legacy_base, v1.tiles);
        assert_eq!(
            fs::read(&path).unwrap(),
            before,
            "migration never changes original v1 bytes"
        );
    }
    #[test]
    fn failed_input_and_protected_destination_never_replace_v1_or_draft() {
        let malformed=b"{\"schema\":\"havenwild.elizawy.scene.v1\",\"name\":\"bad\",\"size\":[2,2],\"tile_size\":32,\"tiles\":[]}";
        assert!(SceneV2::from_v1_bytes(Path::new("bad.scene.json"), malformed).is_err());
        let root = std::env::temp_dir().join(format!("havenwild-v2-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("test.scene.json");
        let legacy = Scene {
            schema: LEGACY_SCHEMA.into(),
            name: "test".into(),
            size: [1, 1],
            tile_size: 32,
            tiles: vec![Tile {
                semantic_role_hint: "Grass".into(),
                source_asset: "Terrain/terrain_summer.png".into(),
                source_rect: [0, 0, 32, 32],
                status: "draft".into(),
            }],
            hidden_visual_samples: Vec::new(),
        };
        let original = serde_json::to_vec_pretty(&legacy).unwrap();
        fs::write(&source, &original).unwrap();
        let doc = SceneV2::preview_import(&source).unwrap();
        let destination = SceneV2::draft_path(&source);
        doc.save_new_draft(&source, &destination).unwrap();
        let draft_bytes = fs::read(&destination).unwrap();
        assert!(doc.save_new_draft(&source, &destination).is_err());
        assert_eq!(fs::read(&destination).unwrap(), draft_bytes);
        assert_eq!(fs::read(&source).unwrap(), original);
        let loaded = SceneV2::read_draft(&destination).unwrap();
        assert_eq!(loaded.legacy_base, legacy.tiles);
        fs::write(&source, b"changed").unwrap();
        assert!(doc.verify_legacy_source(&source).is_err());
        assert_eq!(fs::read(&destination).unwrap(), draft_bytes);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn whole_stroke_is_one_sparse_transaction_and_semantics_survive_undo() {
        let mut doc = example();
        let mut history = SceneHistory::default();
        history
            .execute(
                &mut doc,
                SceneCommand::PaintCells {
                    layer: LayerId::Ground,
                    cells: vec![
                        PaintCell {
                            index: 0,
                            source: Some(binding(32)),
                        },
                        PaintCell {
                            index: 3,
                            source: None,
                        },
                    ],
                },
            )
            .unwrap();
        assert!(history.can_undo());
        assert_eq!(doc.visual_layers[0].edits.len(), 2);
        assert_eq!(
            doc.resolved_legacy_tile(3).unwrap().semantic_role_hint,
            "Grass"
        );
        assert!(doc.resolved_legacy_tile(3).unwrap().is_empty());
        history.mark_saved();
        assert!(!history.is_dirty());
        history.undo(&mut doc);
        assert!(history.is_dirty());
        assert!(doc.visual_layers[0].edits.is_empty());
        history.redo(&mut doc);
        assert!(!history.is_dirty());
        assert_eq!(doc.visual_layers[0].edits.len(), 2);
        assert_eq!(doc.legacy_base[0].source_rect, [0, 0, 32, 32]);
        let baseline = doc.visual_layers[0].edits.clone();
        assert!(history
            .execute(
                &mut doc,
                SceneCommand::PaintCells {
                    layer: LayerId::Ground,
                    cells: vec![
                        PaintCell {
                            index: 1,
                            source: Some(binding(32))
                        },
                        PaintCell {
                            index: 1,
                            source: None
                        }
                    ]
                }
            )
            .is_err());
        assert_eq!(
            doc.visual_layers[0].edits, baseline,
            "failed commands are atomic"
        );
    }
    #[test]
    fn generated_removal_creates_restorable_tombstone_manual_removal_does_not() {
        let mut doc = example();
        let mut history = SceneHistory::default();
        let object = PlacedObject {
            id: doc.next_object_id(),
            label: "Rock".into(),
            layer: LayerId::Objects,
            anchor: [0, 0],
            footprint: [2, 1],
            parts: vec![ObjectPart {
                offset: [0, 0],
                source: binding(0),
            }],
            origin: ObjectOrigin::Generated {
                generator_id: "seed:42".into(),
            },
        };
        history
            .execute(&mut doc, SceneCommand::PlaceObject(object.clone()))
            .unwrap();
        assert_eq!(doc.next_object_serial, 2);
        history
            .execute(
                &mut doc,
                SceneCommand::MoveObject {
                    id: object.id.clone(),
                    to: [1, 0],
                },
            )
            .unwrap();
        assert_eq!(doc.objects[0].anchor, [1, 0]);
        history
            .execute(
                &mut doc,
                SceneCommand::RemoveObject {
                    id: object.id.clone(),
                },
            )
            .unwrap();
        assert!(doc.objects.is_empty());
        assert_eq!(doc.removed_generated_objects.len(), 1);
        history.undo(&mut doc);
        assert_eq!(doc.objects.len(), 1);
        assert!(doc.removed_generated_objects.is_empty());
        history.redo(&mut doc);
        assert_eq!(doc.removed_generated_objects[0].object_id, object.id);
        doc.validate().unwrap();
        let manual = PlacedObject {
            id: doc.next_object_id(),
            label: "Flower".into(),
            layer: LayerId::Objects,
            anchor: [0, 0],
            footprint: [1, 1],
            parts: vec![ObjectPart {
                offset: [0, 0],
                source: binding(0),
            }],
            origin: ObjectOrigin::Manual,
        };
        history
            .execute(&mut doc, SceneCommand::PlaceObject(manual.clone()))
            .unwrap();
        history
            .execute(&mut doc, SceneCommand::RemoveObject { id: manual.id })
            .unwrap();
        assert_eq!(
            doc.removed_generated_objects.len(),
            1,
            "removing a manually placed object does not create a generation tombstone"
        );
        doc.validate().unwrap();
    }
    #[test]
    fn decoration_overlay_retains_underlying_ground_and_original_source_bytes() {
        let mut doc = example();
        let base = doc.legacy_base.clone();
        let source = SourceBinding {
            source_asset: "Terrain/plants_summer.png".into(),
            source_rect: [0, 0, 32, 64],
        };
        let mut history = SceneHistory::default();
        history
            .execute(
                &mut doc,
                SceneCommand::PaintCells {
                    layer: LayerId::Objects,
                    cells: vec![PaintCell {
                        index: 3,
                        source: Some(source.clone()),
                    }],
                },
            )
            .unwrap();
        assert_eq!(
            doc.legacy_base, base,
            "placing vegetation may not replace grass"
        );
        assert_eq!(doc.resolved_legacy_tile(3).unwrap(), base[3]);
        assert_eq!(doc.visual_layers[5].edits[0].source.as_ref(), Some(&source));
        let frozen_pie = doc.clone();
        history.undo(&mut doc);
        assert!(doc.visual_layers[5].edits.is_empty());
        assert_eq!(
            frozen_pie.visual_layers[5].edits[0].source.as_ref(),
            Some(&source),
            "PIE visual snapshot remains independent of later editor undo"
        );
        history.redo(&mut doc);
        doc.validate().unwrap();
    }
    #[test]
    fn local_layered_draft_save_reopens_and_refuses_changed_original() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let root = std::env::temp_dir().join(format!(
            "havenwild-layered-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("original.scene.json");
        let legacy = Scene {
            schema: LEGACY_SCHEMA.into(),
            name: "ground".into(),
            size: [2, 1],
            tile_size: 32,
            tiles: vec![
                Tile {
                    semantic_role_hint: "Grass".into(),
                    source_asset: "Terrain/terrain_summer.png".into(),
                    source_rect: [0, 0, 32, 32],
                    status: "draft".into()
                };
                2
            ],
            hidden_visual_samples: Vec::new(),
        };
        let original = serde_json::to_vec_pretty(&legacy).unwrap();
        fs::write(&source, &original).unwrap();
        let mut doc = SceneV2::preview_import(&source).unwrap();
        let mut history = SceneHistory::default();
        history
            .execute(
                &mut doc,
                SceneCommand::PaintCells {
                    layer: LayerId::Objects,
                    cells: vec![PaintCell {
                        index: 0,
                        source: Some(SourceBinding {
                            source_asset: "Objects/Furniture/test.png".into(),
                            source_rect: [0, 0, 64, 32],
                        }),
                    }],
                },
            )
            .unwrap();
        let draft = root.join("derived/elizawy_mapping_certification.layered.draft.json");
        doc.save_editable_draft(&source, &draft).unwrap();
        let saved = fs::read(&draft).unwrap();
        let restored = SceneV2::read_draft(&draft).unwrap();
        restored.verify_legacy_source(&source).unwrap();
        assert_eq!(
            restored.visual_layers[5].edits[0]
                .source
                .as_ref()
                .unwrap()
                .source_rect,
            [0, 0, 64, 32]
        );
        assert_eq!(fs::read(&source).unwrap(), original);
        fs::write(&source, b"changed").unwrap();
        assert!(restored.save_editable_draft(&source, &draft).is_err());
        assert_eq!(fs::read(&draft).unwrap(), saved);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn full_manual_object_move_remove_preserves_original_terrain_and_source_region() {
        let mut doc = example();
        let base = doc.legacy_base.clone();
        let mut history = SceneHistory::default();
        let source = SourceBinding {
            source_asset: "Objects/manual_reed.png".into(),
            source_rect: [32, 64, 32, 64],
        };
        let foot = PlacedObject::origin_at_foot([1, 1], [1, 2]).unwrap();
        assert_eq!(foot, [1, 0]);
        let object = PlacedObject::manual_region(
            doc.next_object_id(),
            LayerId::Objects,
            foot,
            source.clone(),
        );
        assert_eq!(object.footprint, [1, 2]);
        history
            .execute(&mut doc, SceneCommand::PlaceObject(object.clone()))
            .unwrap();
        assert_eq!(doc.object_at([1, 1]).unwrap().id, object.id);
        assert!(doc.object_at([0, 1]).is_none());
        assert_eq!(doc.legacy_base, base);
        assert_eq!(doc.objects[0].parts[0].source, source);
        history
            .execute(
                &mut doc,
                SceneCommand::MoveObject {
                    id: object.id.clone(),
                    to: [2, 0],
                },
            )
            .unwrap();
        assert!(doc.object_at([1, 1]).is_none());
        assert_eq!(doc.object_at([2, 1]).unwrap().id, object.id);
        history
            .execute(
                &mut doc,
                SceneCommand::RemoveObject {
                    id: object.id.clone(),
                },
            )
            .unwrap();
        assert!(doc.objects.is_empty());
        assert_eq!(doc.legacy_base, base);
        history.undo(&mut doc);
        assert_eq!(doc.objects[0].parts[0].source, source);
        history.undo(&mut doc);
        assert_eq!(doc.objects[0].anchor, foot);
        history.redo(&mut doc);
        assert_eq!(doc.objects[0].anchor, [2, 0]);
    }

    #[test]
    fn drag_preview_and_committed_object_move_use_same_scene_bounds() {
        let mut scene = example();
        let mut history = SceneHistory::default();
        let art = SourceBinding {
            source_asset: "Terrain/plants_summer.png".into(),
            source_rect: [0, 0, 32, 32],
        };
        let object = PlacedObject::manual_region("object:1".into(), LayerId::Objects, [1, 1], art);
        history
            .execute(&mut scene, SceneCommand::PlaceObject(object))
            .unwrap();
        assert!(scene.can_move_object_to("object:1", [2, 1]));
        assert!(!scene.can_move_object_to("object:1", [-1, 1]));
        assert!(!scene.can_move_object_to("object:1", [3, 1]));
        assert!(!scene.can_move_object_to("not-present", [1, 1]));
    }

    #[test]
    fn object_anchor_refuses_out_of_bounds_instead_of_silently_clamping() {
        let mut doc = example();
        let mut history = SceneHistory::default();
        assert_eq!(PlacedObject::origin_at_foot([0, 0], [1, 2]), Some([0, -1]));
        assert!(PlacedObject::origin_at_foot([0, 0], [0, 2]).is_none());
        let original = doc.legacy_base.clone();
        let object = PlacedObject::manual_region(
            doc.next_object_id(),
            LayerId::Objects,
            [0, -1],
            binding(0),
        );
        assert!(history
            .execute(&mut doc, SceneCommand::PlaceObject(object))
            .is_err());
        assert_eq!(doc.legacy_base, original);
        assert!(doc.objects.is_empty());
    }

    #[test]
    fn visual_changes_do_not_mutate_structural_channels_or_invent_certification() {
        let mut doc = example();
        let mut history = SceneHistory::default();
        doc.structural_cells.push(StructuralCell {
            index: 0,
            elevation: Some(1),
            terrain_kind: Some("grass".into()),
            water_flow: None,
            blocks_traversal: Some(false),
            connector: None,
        });
        let original = doc.structural_cells.clone();
        history
            .execute(
                &mut doc,
                SceneCommand::PaintCells {
                    layer: LayerId::Ground,
                    cells: vec![PaintCell {
                        index: 0,
                        source: Some(binding(32)),
                    }],
                },
            )
            .unwrap();
        assert_eq!(doc.structural_cells, original);
        assert_eq!(doc.legacy_base[0].semantic_role_hint, "RiverWater");
        assert!(SourceBinding {
            source_asset: "../unsafe.png".into(),
            source_rect: [0, 0, 32, 32]
        }
        .validate()
        .is_err());
        assert!(SourceBinding {
            source_asset: "Terrain/terrain_summer.png".into(),
            source_rect: [u32::MAX, 0, 32, 32]
        }
        .validate()
        .is_err());
        doc.validate().unwrap();
    }
}
