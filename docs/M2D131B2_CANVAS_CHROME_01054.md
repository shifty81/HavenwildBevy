# M2D131B2 — Canvas Chrome and Theme Foundation (v0.10.54)

## Scope

- One new Rust module `src/canvas_chrome.rs` owns opaque panel/toggle/action styling and responsive canvas zones for Grid/Chunks and Zoom.
- Top-center Grid and Chunks toggles reuse the existing persisted `EditorLayout` visibility flags; main command toolbar no longer duplicates them.
- Top-right Fit, Zoom out, live zoom percentage, Zoom in is shared by World and Summer Scene. Existing camera behaviors and bounds remain authoritative.
- Live FPS moves to the existing bottom status bar; zoom no longer mixes performance display with navigation.
- Animation toggle and playback controls, the layer strip background, and the Timeline surface reuse shared theme-owned opaque colors.
- Three local editor themes (North Star, Slate, High Contrast) are selectable from the top `Theme` menu and persisted under the workstation-local editor layout. ForgeGUI still owns the base shell visuals; the new adapter changes egui semantic surface/widget colors only.
- Input exclusion: Grid/Chunks, Zoom, Layers and Animation overlay rects are registered before world/scene hit-testing. PIE and Focus omit editing-only canvas view chrome.

## Deliberate exclusions

- No ElizaWy art, atlas recipes, terrain/world semantics, collision editing, worldgen, PIE or asset authority changes.
- No full dock graph renderer or mode registry in this phase.
- No invented GPU renderer string in status; backend remains inspectable through launch logs/diagnostics until native runtime reporting is wired.
- No full Pixel Studio image authoring in this pass.

## Native gates required on Windows

1. `cargo fmt -- --check` / `cargo fmt` if needed. The package was authored without local Rust/rustfmt; Windows PCC may normalize formatting before the build.
2. `cargo check` and `cargo test --all-targets`.
3. Verify World and Summer Scene Grid/Chunks independently, that the overlays do not paint beneath buttons, Fit/Zoom behave at varied DPI and small windows, FPS updates in the bottom status, Animation/Layer chrome is solid, and theme switch persists after relaunch.
4. Test Play-in-Editor suppression of editing overlays and restoration of the user layout.

Structural Python checks and disposable PCC install tests do NOT claim native Rust/GPU success.
