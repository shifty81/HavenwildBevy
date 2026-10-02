# M2D06-B — Pixel Collision Authoring

## Goal

Keep collision as a lightweight, editable world-data layer while allowing corrections down to the exact 32x32 source-pixel grid of each logical Havenwild terrain cell.

## UX

- The permanent right layer rail gains **COL**. It toggles the collision overlay and opens the existing Collision / Elevation application panel.
- Collision overlay is translucent red. Red means impassable.
- Automatic/coarse collision remains visible when no explicit pixel mask exists.
- A cell may own one explicit 32x32 collision mask. Each set bit blocks the corresponding source-pixel position.
- Compact panel mode exposes Block/Clear, full-blocked, full-clear, automatic/revert, and the world-canvas edit toggle.
- Full panel mode adds an interactive 32x32 mask canvas.
- `Edit on world canvas` maps a click directly to the corresponding collision pixel under the world artwork. High zoom is recommended for exact corrections.

## Authority and precedence

1. Explicit 32x32 collision mask, if present.
2. Explicit coarse `blocks_traversal` structural override.
3. Existing provisional source-scene terrain policy.

An all-clear pixel mask is intentionally meaningful: it says the entire cell is explicitly walkable even if coarse/automatic collision would have blocked it. `Revert automatic` removes the mask and restores the lower authority.

## Persistence

Pixel collision is stored in `SceneV2.collision_cells` separately from visual layers and structural cells. Existing v2 drafts load with an empty collision layer through serde defaults. Collision edits are SceneCommands, participate in undo/redo, mark the scene dirty, and persist through the existing Save Scene workflow.

## Source-art safety

Collision is never inferred by sampling artist alpha in this pass. Original PNGs, source rectangles, historical mapping evidence, and visual layers remain untouched.

## PIE

PIE snapshots the authored pixel collision layer. Player body checks use the 32x32 mask before coarse cell traversal policy, allowing a tree trunk, cliff edge, shoreline, doorway, or similar obstacle to occupy only the pixels that need to block movement.

## Follow-up

- group drag-painted pixels into one undo transaction/stroke;
- generate first-pass masks from certified object/terrain collision templates rather than alpha inference;
- multi-cell object collision template authoring;
- collision copy/paste/mirror operations;
- chunk-level collision streaming once M2D06 world chunk materialization lands.
