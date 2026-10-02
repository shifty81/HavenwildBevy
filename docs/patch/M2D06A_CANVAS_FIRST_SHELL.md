# M2D06-A — Canvas-first shell / overlay tool applications

## Intent

Normalize the Havenwild Bevy editor shell without changing the authored Summer world, source-art authority, terrain evidence, collision data, or runtime generation claims.

## Implemented

- Permanent left direct-edit tool rail: Select, Paint, Erase, Pick.
- Permanent right active-layer rail for the seven existing SceneV2 visual layers.
- Bottom-left `HW` launcher, toggled by Ctrl+Space, expanding over the canvas.
- Real wired launcher entries for Scene/Layers, Collision/Elevation, Source Browser, Terrain Rules, Source Evidence and selected-object properties.
- Truthful disabled entries for World Generator, Chunk Manager, Pixel Studio and Animation Studio; no fake functionality.
- Source/Terrain/Evidence, Scene/Layers and Collision/Elevation are `egui::Window` overlay applications. Dock L/R/B pins them over the corresponding edge and does not reserve canvas layout space.
- Compact/Full presentation state for the three implemented application-panel families.
- Top command toolbar using central `src/editor_commands.rs` metadata.
- Direct canvas shortcuts: Q Select, B Paint, E Erase, P Pick. Existing Ctrl+S, Ctrl+Z, Ctrl+Y and F11 remain; F6 toggles PIE; Ctrl+Space toggles the launcher.
- Shortcut text is generated from the same command metadata used by the rails/toolbar.
- Static shell selftest and Rust source audit updated so old canvas-reserving feature panels cannot silently return.

## Preserved

- Existing Summer scene/draft paths and exact ElizaWy source bindings.
- M2D05 recovered historical evidence and zero-promotion policy.
- Current chunk coordinate/grid visualization.
- SceneV2 undo/redo/save, generated-object tombstones, collision/elevation structural cells and PIE snapshot behavior.
- No source PNG bytes are included or modified.

## Explicitly not implemented

M2D06-A does not implement multi-chunk materialization, procedural world generation, semantic world brushes, Pixel Studio PNG editing, or Animation Studio timeline authoring. Their launcher entries are disabled until functional passes land.

## Required certification

Run PCC Full Quality Gate on Windows, then launch DX12 Studio and verify: rails remain narrow; launcher opens bottom-left and expands over canvas; feature panels float/dock without changing canvas dimensions; Compact/Full works; Q/B/E/P/F6/Ctrl+Space shortcuts match tooltips; existing Summer editing and Source Evidence still work.
