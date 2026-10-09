# M2D114 — Unified tool rail and simplified Terrain Studio

**Source 0.10.32 → 0.10.33.** Standalone Havenwild Bevy, incremental PCC patch. Original art, terrain rules, and world saves remain untouched.

## Problems corrected

- The left icon rail, upper Tooling frame, and floating Terrain Studio exposed duplicate tool selection.
- Pixel Brush reused the Decorations icon; Tile Brush reused Ground; bridge editing reused the Structures house; panel visibility reused Foreground. The icon system was therefore ambiguous.
- Every unselected rail control had an outlined box, making the 7-layer visibility strip especially noisy.
- Terrain Studio mixed production patterns, source catalog, expired Grass/Void recipe experiments, and historical evidence at the same top-level.

## Changes

- Centralized semantic 24-unit vector glyphs in `src/editor_icons.rs`. Shared rendering for tool and layer rails, no new dependencies or assets. Independent icons for Pixel, Tile, Terrain, Bridge and Panel.
- Quiet inactive controls, subtle hover treatment, selective teal accent, consistent line weight/geometry. Source and artwork pixels untouched.
- **Only the rail chooses tools.** The upper-left frame displays active tool/color, the middle-left frame owns settings, the lower-left frame owns tile/material selection. No duplicate brush/material selectors in the upper frame.
- Terrain Studio becomes a guided *advanced reference* companion: `Live terrain rules`, `Original atlas`, and hidden `Advanced` submenu for source evidence and the legacy experiment. Quick buttons return directly to Terrain Brush / Tile Library on the canvas. No duplicated everyday painting application.
- Apps launcher opens Source Browser directly in the canonical Library; advanced evidence remains discoverable.
- The right Inspector now shows the selected world's NW/NE/SW/SE material tuple, exact/projected/missing Tiled status, original canonical tile identity, and a working `Locate source in Library` action. Source location survives on-demand atlas loading instead of resetting to (0,0).
- Existing renderer, Pattern Matrix source families, legacy metadata, layers, object editing, source PNGs, and PCC patch/governance retained.

## Explicitly not delivered

- The Terrain rule editor is still *inspection*, not editable Wang/tuple reassignment. The existing sea/coast rendering gaps are outstanding. No fake success claim is made.
- The old floating Terrain Studio backend remains for advanced diagnostics, not primary authoring; full migration of rule repair to Inspector requires the next governed pass.
- Native Windows Rust compilation and live screenshot acceptance are required.

## Acceptance

1. One icon per primary action, no Select/Pixel/Tile/Terrain duplicates in top frame; context panel has its own icon; bridge no longer looks like Structures.
2. Tooltips correctly identify each action; changing tools retains existing undo/terrain semantics.
3. Visibility eye toggles, shift-isolate and layer active state remain independent of brush selection.
4. Source atlas picker and semantic Terrain palette work from lower-left Library with floating Terrain Studio closed.
5. Opening Terrain Studio displays Live Rules / Original Atlas; advanced data only appears via deliberate menu.
6. Source-contract audit and native Full Quality Gate green before certification.
