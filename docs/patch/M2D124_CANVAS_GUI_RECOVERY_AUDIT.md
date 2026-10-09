# Havenwild M2D124 — Canvas-first GUI regression recovery

## Evidence and cause

Compared recovered v0.10.41 source, pre-M2D117 v0.10.34 source, accepted UA-01 design, M2D115 discussion and the 2026-10-09 Studio screenshot.

- M2D117 moved `draw_layer_rail` from canvas to `egui::Panel::left("havenwild.layers.dock")`, adding a 75-pixel permanent column, contrary to user's canvas-edge layer-control direction.
- M2D112/114 left 274-pixel Tooling and Tool Options columns open at startup; later Library moved to bottom, leaving the old left context mostly instructions. Source-contract tests explicitly asserted `tool_context_open: true` and the Layers dock even though those are not desired product acceptance rules.
- The initial professional icon-rail/design concept was not a complete working pixel/animation/editor backend. Do not conflate concept imagery with implemented features.
- Existing functional source-backed workflows were not removed: Select/Tile/Terrain brush command paths, layers' eye toggles and Shift-isolate, tool hotkeys, bottom Library with Grid/List/Atlas, Tiled terrain inspector, reviewed complete-object placement, World PIE, room and doorway authoring, undo.
- The Pixel Brush still has no derived image pixel-edit backend; only color and brush size preferences exist. A polished visual control would be misleading until real pixel editing is supported.

## Source changes in this update

1. Layer controls render through `canvas_ui.scope_builder` in the actual canvas UI. There is **no separate Layers panel**, `Area` on a global foreground order, or extra width reservation. Icons have individual dark contrast chips only, not a full-height dock or panel container.
2. One compact Layers icon expands/collapses the vertical seven-layer control near the top-left canvas corner. Eye visibility, active layer, Shift isolate and collision overlay remain connected to the same canonical state. Pointer hit zones are calculated before painting to prevent accidental edits through the controls.
3. New layouts default to no bulky Tooling/Tool Options context slab. Existing old workstations migrate once; if the user deliberately reopens tool settings, the preference remains saved. All active-tool settings are in a single scrollable section instead of split placeholder frames. The bottom library still owns the asset selection.
4. Older Python source checks now assert canvas-native Layers and non-default tool panel rather than the unwanted opposite. New M2D124 contract checks prevent regression.

## Boundaries and next UI work

- This is a **targeted regression recovery**, not a claim of complete concept parity. Pixel art authoring, rich brush presets, contextual ribbon, icon visual refinement, toolbar tool registry, full dock persistence and task-specific menus remain to be implemented/certified.
- This update does **not** reclassify terrain or mutate ElizaWy assets, worldgen, PIE data or source sprites.
- Source-contract scripts are a preliminary gate only. Native Windows `cargo fmt --check`, `cargo check`, `cargo test --all-targets` and screenshot/video UI certification at multiple DPI/window sizes remain mandatory.
- Verify that Layers controls do not steal world clicks, do not cover modal windows, and do not overlap rulers/view controls when the viewport is narrow. Verify state saving and reopening of Settings, Library, and Layers.
