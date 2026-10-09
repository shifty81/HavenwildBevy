# Havenwild M2D117 — UA-03 docks, room/cave model and stone distribution

**Source:** 0.10.34 → 0.10.35. **Engine:** Bevy 0.19.0; original source art immutable. **Status:** source-stage/structural tests only until native Windows Full Gate.

## Working editor changes

- **Layers:** a true `egui::Panel::left` adjacent to the tools, allocated before the canvas, no foreground `egui::Area` and no invisible 69px canvas blocker. Eye toggles, active layer, Shift-isolate and collision remain unchanged. Tool rail includes dedicated Layers and Library toggles.
- **Library:** one shared `draw_authoring_library` rendered either in a resizable bottom panel or an undocked floating window, never as a duplicate browser. State (`open`, `floating`, `height`, `tab`, `category`, `view`) persists in local editor layout. Terrain selection remains semantic, Tile Brush selects exact original 32px source cells.
- **Grid/List/Atlas:** Grid/List use asynchronously decoded, cached PNG alpha occupancy to omit empty 32px positions; Atlas retains the *entire* original source grid including transparent addresses. All selections retain exact original pixel coordinates. Source PNGs remain read-only. Filters use catalog-backed categories Terrain/Vegetation/Objects/Structure/FX, plus free-text search. Decoding runs off the UI thread.
- **Tool context:** one left Tooling and Brush Settings area; source library no longer duplicated in left vertical stack.

## New nonvisual room/cave document foundation

`src/room_scene.rs` is a pure Rust/serde room document. A room is normally 2×2 chunks (64×64 tiles); supported authored extents may be 2×3, 3×2, 3×3; 1×1 and >3 are rejected. Painting floor (`paint_floor` or rectangle) recalculates the four-neighbor wall mask. Users will eventually choose source-backed floor and wall art with a wall tint that doesn't mutate original source pixels. Portal links carry stable IDs, kind, destination, spawn cell and optional world-return anchor; duplicates, missing identities and non-floor entrances are rejected. Seeded caves generate a deterministic connected carved floor and derive their wall topology. Save/load via serde. Native Rust unit tests cover these contracts.

**Not yet connected to Studio UI or runtime:** room floor brush, wall preview, generated cave viewport, door/cave entrance placement, PIE interaction/transition, exit back to world, player character customization/animation composition. Those need a separate end-to-end Windows-tested pass; do not represent this data model as playable.

## Stone defect

Previously the source-authored `rock` templates could *only* spawn in `rocky_upland`, a biome requiring elevation >=2; at lower generated elevations no stones appeared. The same certified rock templates are now also sparse on grassland meadow/light woodland, with highest density at uplands. Collision, source bounds, footprints and overlap checks are preserved. World regeneration is required for re-evaluation; existing saved suppression/move overrides remain authoritative. **Actual visual stone visibility needs runtime screenshot verification.**

## Native verification

1. Apply the governed patch to a project showing source `0.10.34` (or extract the complete rollup into a new folder).
2. Run PCC Full Quality Gate, collect compiler and test outputs. The autoformat phase can normalize any Rust formatting before building.
3. In Vulkan Studio, verify Layers never cover floating panels, collapse/reopen, Shift-isolate, bottom Library resize and undock, Grid compact omissions vs Atlas physical spaces, source stamp coordinates and World PIE.
4. Regenerate a test world near meadow and upland; compare actual stone objects in viewport and Inspector. Do not declare the stone issue resolved from the catalog alone.
5. Next playable milestone: hook `RoomScene` to the Scene canvas floor brush and render source-backed wall recipes, register a house door and exit link, and switch World PIE ↔ room PIE with the same avatar and return anchor. Follow with seeded cave entries and elevation-backed cliff/waterfall tools.
