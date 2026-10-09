# M2D103 — corrected painted coasts, context actions, and terrain QA

**Cumulative replacement for M2D102, from M2D100 source 0.10.21.** The user has not installed M2D101 or M2D102; do not install either first. This release includes all 22 files from M2D102 plus these targeted changes, version 0.10.24. It does not change the ElizaWy/character source art or the existing 7-layer persistent world schema.

## Root cause and patch boundary

- M2D102 consulted generator `Coast` water context but returned manual semantic overrides before that branch. Painting isolated sand/dirt/grass into deep water bypassed the shallow-water fringe. `semantic_role_for_dual_grid` now reads final semantic overrides/pending edits independently; only the visual water material uses `resolved_role_for_dual_grid` to substitute shallow-water Wang roles within two cells of any authored or generated land. `resolved_role` continues to represent **DeepWater** semantics, swimming/traversal and persistence. No ocean water is automatically converted into gameplay shallow water.
- The editor has a second, broader problem: Summer Terrain TSX has 331 Wang entries, **all covering only one or two material colors per tile**. Arbitrary grass/sand/dirt/water triple corners have no exact source art. The fallback to local projection/flat fill is still visible for such vertices; it is not a certified final fix. The full-screen Wang solve is all-or-nothing when an unsupported multi-material junction appears. Metrics now counts projected/missing recipes across **all** terrain rather than only water-to-land.
- After right click, the Generated World canvas now provides a context menu for selecting an object, sampling the semantic brush, painting one semantic terrain cell, and removing an authored override. Middle button pans; right button is no longer used for panning. A canvas background mouse click is gated by the background response hover so menu clicks should not paint beneath overlays. Undo still uses the existing world snapshot mechanism.
- The M2D102 15/70/15 docks, on-canvas layers and zoom/FPS, inspector metrics drawer, authored horizontal bridge, object dragging and vegetation variation remain included unchanged.

## Seasonal source audit

The supplied original Jaidyn TSX Summer, Spring and Winter files each have 331 matching tile IDs and exact Wang pattern strings. Winter material color **3** is `Snow` rather than `Dirt`, and color **8** is `Ice` rather than `Melted Ice`, so identical IDs do **not** mean identical terrain semantics. The supplied *autumn-old* TSX has only 281 entries and differs in tile pattern / assignment; it is not certified as an identical Wang authority. Runtime currently seasonalizes supported source regions by matching source rectangle coordinates, which is not a universal seasonal content migration contract. **Do not claim full season parity until source-coordinate assets, animated clips and semantic roles are verified per season.**

## Windows acceptance checklist (pending)

1. Confirm 0.10.24 using PCC after applying from exact M2D100 0.10.21; run Full Gate. Use Vulkan first, DX12 second if available.
2. Paint a 1x1 and 3x3 sand island within generated deep ocean, then extend with dirt/grass; test immediate re-resolution and save/reload. Observe nearest water as shallow-water **art**, keeping swim/deep gameplay state. Repeat across a 32-cell chunk border.
3. Paint deep water over an island, then erase authored terrain. Confirm appearance responds to semantic edits and original generation comes back on erase.
4. Open right-click context menu over selected terrain. Verify sample, paint single tile, select object, undo and erase. Confirm right-click no longer pans, middle-drag pans, menu clicks don't paint the world underneath.
5. Open Inspector > Metrics; audit **all** unsupported Wang terrain, then highlight yellow projected or red missing tiles. Record coordinates, the four semantic corner roles and affected source binding. Expect remaining three-material corners; fixing them requires an **authored layering or new original transition-art plan**, not a fabricated tile index.
6. Inspect 15/70/15 docks and on-canvas layers/zoom. Drag a house or tree across chunk boundary and reopen save. Place a horizontal bridge with valid land endpoints. Test undo/redo, collision and traversal.
7. Do not claim seasonal inheritance from summer beyond the verified spring/winter matching TSX pattern/ID sets. Autumn and special winter/snow semantics remain separately gated.

## Pixel authoring and world-scene transition architecture — design work, not implemented in M2D103

A compact **Pixel** tool mode in the existing left dock should change the central canvas to an editable image/layer surface while reusing selections, palette, pixel brush, undo/redo and safe derived-asset outputs. Source ElizaWy art must remain immutable; edits create derived metadata-backed tiles/objects and feed the shared terrain/scene system. Editing a World chunk or a multi-chunk Scene is a project/world choice, not a requirement to open a new runtime renderer. The user should be able to select a world region, enter Pixel mode, publish updated regions to the shared catalog and return without restarting Studio.

A runtime **world scene** should be a lazily loaded rectangular region spanning one or many 32x32 chunks, keyed by persistent world coordinates and saved deltas. Crossing adjacent **traversable** borders continues into neighboring streamed chunks under the same coordinate authority, without forcing a doorway/teleport. Authored interior/cave/transitional scenes use explicit link portals with spawn, facing, collision and return-point validation. Non-traversable coast/cliff/world bounds have no mandatory transition. The world-view/editor should be able to author, save and test a multi-chunk scene in place. Treat streaming, disk budget and seam persistence as requirements for a later native feature gate, **not** as already implemented.

## Limitations

- New M2D103 Rust changes have not been compiled or interactively validated on Windows here.
- Existing triple-material projection, legacy WetSand/PebblePath unsupported materials, vertically oriented bridges, full seasonal coverage, full pixel-authoring mode, automatic scene streaming and portals are **not fixed or finished** by this patch.
