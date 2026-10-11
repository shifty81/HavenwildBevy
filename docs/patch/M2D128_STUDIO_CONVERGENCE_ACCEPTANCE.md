# M2D128 — Unified Havenwild Studio | v0.10.46

## Source authority and visual target

The approved Havenwild Studio concept is the acceptance target, with three **independently resizable sections in one left workspace**: (1) Pixel Studio, (2) Game Tools, (3) Scene Navigator. The central world canvas retains unboxed Layers icons. The right Inspector has contextual properties above one shared Asset Browser. The animation timeline owns the bottom edge above the status bar, **always present** even when compact. Neither asset browsing nor app menus may conceal the world or create duplicate state.

## Implemented here

- Left Pixel Studio: actual HSV ring and saturation/brightness square, palette swatches, size control, safe source-art immutability. No fake pixel strokes.
- Middle Game Tools: original Select/Tile/Terrain/Object/Erase/Sample/Bridge controls and real brush settings, unchanged document authority.
- Bottom-left Scene Navigator: uses live world chunk data, runtime source-backed object IDs, per-structure portal links, selected object/world cell and authored scene layers; selection is reflected in the right Inspector.
- Shared Asset Browser: defaults to the lower Inspector, with persistent drag resize, plus left/bottom/floating legacy relocations and ONE authoritative asset/source selection.
- Persistent Animation Timeline: play, pause, rewind, 200ms stepping and scrubbing are connected to the *existing world renderer's source-backed animation resolver*. On a selected animated vertex, the timeline shows original frames and per-frame durations; no new image copies or fake animation assets.
- Layout state: one-time migration for existing M2D127 layouts, then user-chosen sizes, collapsed states, and docks persist. Previous asset and world data stay immutable.

## Explicitly *not* certified/implemented

Direct world pixel paint/undo/derived image save; production sprite animation editing, animation event keyframes, onion skin; complete creative-suite toolset; object/audio animation curves; fully rendered title-concept parity. These need separate engine/asset-document milestones, not UI toggles. Source-backed world animation already plays; this pass adds preview transport control, not authoring/export.

## Native Windows gate and visual acceptance

1. PCC Full Gate must compile Rust tests, renderer and Studio; all source/asset/PIE tests pass.
2. Pixel/Game/Scene sections coexist at once; both left dividers drag independently and survive restart.
3. Color wheel changes selected editor color but NEVER original PNG; Pixel mode must clearly signal no derived image selected.
4. Selecting a world object in Scene Navigator selects the same object in Inspector. Door link entries target existing world objects.
5. Right Inspector Asset Browser can be resized, undocked, docked bottom or docked left; same crop is selected in each place.
6. Timeline pause freezes world water animation, scrub visibly changes frames, play resumes on the same canvas and frame durations follow original Tiled metadata; timeline stays above status bar.
7. Canvas Layers remains an unboxed top-left control and cannot steal unrelated canvas clicks or obscure the floating Library.
8. World PIE transitions, per-house saved rooms, terrain render bindings, stones and existing authored work have no runtime regressions.

**Certification:** Source-contract checks and PCC simulation are preflight, not a substitute for a native Bevy build or visual test.
