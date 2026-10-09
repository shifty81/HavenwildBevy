# M2D126 — recovered integrated authoring UI + visible resizable Library

## Source-of-truth comparison

The user supplied the original CURRENT → IMPROVED concept, and M2D112 source showed draggable splitters plus three authoring areas. Later updates retained the source-backed tools but separated them into a thin rail, frequently collapsed context, and bottom Library whose resize affordance was difficult to find. This change reconnects existing controls instead of inventing a second editor and explicitly restores the higher-density usable tool layout.

## Native Studio changes

- The left icon rail remains 62 px. Beside it the restored Tooling surface exposes Select, Pixel, Tile, Terrain, Objects, Erase, Sample, Bridge and active tool information, with a Tool Context section below. The vertical divider is draggable and persisted. There is exactly one tool state (same `choose_brush`, scene commands, shortcuts and real backend), not a second brush engine.
- Legacy local editor layouts undergo **one** migration, restoring the integrated authoring workspace and single Library. Future user collapse and resizing persist, including across Studio restarts.
- The bottom Asset Library has a visually discernible top grip that you can drag **up to expand / down to shrink**. The header remains on screen when collapsed; the tray remains undockable. The Library has the same Terrain / Tiles / Assets / Source / Rooms and Grid / List / Atlas views over the same selection authority. Height saved on release avoids disk writes every mouse frame.
- Layers stays as an unboxed compact canvas top-left icon strip with eye visibility, active layer, isolate and collision overlay. Does not create a new Layers panel, change hit testing or move asset mappings.

## Boundaries

This is GUI recomposition, not completion of the pixel-painting backend, visual color wheel, brush-hardness algorithm, missing source-region classifications, interior tile art or waterfall authoring. No ElizaWy source artwork, world saves, portals, object records, runtime renderer or PIE movement algorithms changed. Native Rust Full Quality Gate and DPI/drag visual acceptance on Windows required before certification.

## Test matrix

1. Fresh extracted v0.10.44 opens Tooling and Tool Context; collapse rail and reopen; Settings remain bound to the selected real tool.
2. Upgrade from v0.10.43 with locally persisted collapsed/hidden layout: first start restores the recovered workspace. Collapse again, save, restart: user choice is retained.
3. Drag Library top grip up/down; save and restart. Collapse and expand from its header; undock and redock. Ensure atlas exact cells and review-only alpha regions retain their selection.
4. Toggle and Shift-isolate Layers without obstructing canvas mouse input; layer icons do not cover floating studios.
5. Pick reviewed whole sprite from Library and place it; undo; regenerate world; original pixels and renderer must remain unchanged.
6. Run native `cargo fmt --check`, `cargo check`, `cargo test --all-targets` and PCC Full Gate; inspect Windows screenshots at 100%, 125% and 150% DPI. The source-only checks do not certify these.
