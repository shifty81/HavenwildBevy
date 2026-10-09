# Havenwild M2D107 — Compact rail + distinct brush pass (cumulative from 0.10.26)

**Install contract:** `0.10.26 → 0.10.28`, includes the *unbuilt* M2D106 premultiplied-alpha correction. Do **not** install M2D106 separately when applying this patch.

## Implemented

- An independent 62px full-height icon rail, instead of the persistent empty left 15%-wide dock. The contextual brush/details panel opens when a brush/eraser/sampler is selected, can be collapsed, and can be resized. Select starts with the context collapsed by default. Existing on-canvas layer strip, right dock, status metrics drawer, and upper-right canvas controls remain.
- Separate **Pixel Brush**, **Tile Brush**, and **Terrain Brush** identities, preserving the active brush mode, the Terrain material selection, the tool-context visibility, and each brush's size settings in local editor layout. Tool changing does not overwrite terrain/source selection. B returns to the selected painting mode.
- **Terrain Brush (Generated World)** reuses semantic painting and native Wang/dual-grid resolution. The user-selected brush radius setting (1 = a single cell, up to 7×7 cells for 4) is applied along one interpolated, undoable pointer stroke.
- **Tile Brush (Generated World)** uses the exact selected 32×32 Terrain source slice, stored as location-specific visual overrides only, without rewriting source PNG or changing terrain gameplay semantics. It reuses the shared interpolated/undoable stroke engine and supports a radius setting (1 = single tile; 4 = 7×7 tiles). Source Browser opens from the tool context. Other asset categories / multi-cell sprites are currently blocked in Generated World rather than silently corrupting terrain; Summer Scene retains multi-cell exact visual source placement via its existing scene tool.
- **Pixel Brush** is a distinct selectable mode and has its own remembered size, but **pixel authoring is not yet implemented**. The canvas refuses to paint or erase world/scene terrain while Pixel is selected. A project-owned derived image document, palette/brush engine, asset registry binding, and pixel-level undo are required in a later pass before it can edit art. Original ElizaWy source files stay immutable.
- Retains M2D106's premultiplied egui texture preparation for the green-rectangle/white-guide tree bug (native visual test pending).

## Explicitly not claimed

- Native Bevy build, drag/gesture acceptance, Pixel brush image edits, complete atlas classification, repaired inner/wet/dry coast recipes, guaranteed multi-cell scene streaming.
- Tile Brush does not place arbitrary buildings/trees as world objects yet. Only exact 32px Terrain visuals are safe on the current Generated World tile-override path; complete object authoring requires object template/binding integration.
- The existing source atlas roles and seasonal mapping are still incomplete. This pass does not invent atlas labels or force previously evidence-blocked materials into PCG.

## Windows verification sequence

1. Apply the cumulative `.pccpatch.zip` to the installed **0.10.26** Havenwild root through PCC. Confirm source `0.10.28`.
2. Run **Full Quality Gate**. `cargo fmt` may normalize Rust formatting on the first run; PCC will recheck.
3. Launch Vulkan Studio and verify the large tree transparency against the previous screenshot.
4. Select (Q): only a narrow far-left toolbar should reserve space. Select a tree and move it one cell; Undo. Canvas layers should remain pinned at its left edge, independent of the tools.
5. Select **Terrain Brush**, choose Sand, paint a small strip. Undo should revert the whole stroke; game terrain semantics must change. Try radius 2 and verify the brush area.
6. Select **Tile Brush**, open Source Browser and select a certified exact 32×32 Terrain tile. Stamp it on the Generated World, save/reopen, undo, and verify that underlying terrain role/collision remains the same. A non-Terrain or 64px region must be refused with an actionable error.
7. Select **Pixel Brush**, click a world cell and verify **no world data changes**; the contextual panel should explain the derived-asset backend is pending.
8. Expand/collapse tool context, check left rail remains fixed at 62px; reopen Studio and verify brush mode, context visibility, and size settings persist.
9. Test the existing right inspector/metrics, source browser, bridge span, and Generated World right-click to verify no regression.
10. Provide PCC bundle and screenshots if the Rust build or canvas behavior differs. The known shoreline mapping issues require a separate atlas-to-runtime mapping coverage pass.
