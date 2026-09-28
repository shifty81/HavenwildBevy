# M2D02-B2 — Embedded Canvas Rulers (incremental, requires B1)

- This is a display-only extension of the **M2D02-B1** baseline, not a new workspace or another dock.
- Default-on, translucent horizontal 24px and vertical 32px ruler bands are drawn by the existing clipped canvas painter *after artwork*; no physical space reserved or scene pixels changed.
- The ruler origin tracks `scene_rect.min`, using loaded scene tile boundaries and the same `tile_size * zoom` used by painting/picking. Adaptive 1/2/5 divisions make labels legible from 0.125x to 6x; negative/off-scene coordinates follow pan continuously.
- Cyan cursor markers and gold selected-cell markers remain in the ruler bands, not over artwork.
- The overlay bands are excluded from paint/select/sample hit-testing. Floating view and selection controls move just below/inside the ruler corner while the overlay is enabled.
- The View menu exposes **Show canvas rulers (tile coordinates)**; the local editor-layout v1 JSON stores the optional preference with a backward-compatible default of true.
- F11 Focus hides the visual overlay temporarily without resetting the saved preference. The no-ruler layout keeps the original control placement.
- Source artwork, v1 scene fixture, independent v2 draft, history, DG recipe, registry, Cargo.lock and other game data are unchanged.
- Native Windows Cargo compile, formatting gate and interaction tests remain necessary; local Python audit is not a substitute.

## Acceptance checks
1. Toggle View → Show canvas rulers; no canvas resize, origin and ticks follow pan/zoom.
2. Try painting with the pointer in ruler bands: no scene edit; immediately inside bands: normal click works.
3. Zoom to minimum/maximum, pan outside 40×28; labels remain readable and align with tile boundaries.
4. Select a cell: yellow ruler indicator follows it; cursor gives cyan indicator. Floating selection and Fit controls do not cover rulers.
5. Toggle F11 Focus, return; the preference stays intact. Close/reopen Studio; preference persists.
6. Continue M2D02-B1 Scene → Preview separate v2 scene draft; no changes in generated v2 output.
