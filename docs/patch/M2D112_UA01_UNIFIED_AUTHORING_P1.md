# Havenwild M2D112 / UA-01 P1 — Unified authoring shell and in-canvas source access

**Source:** 0.10.30 -> 0.10.31. **Patch, not source rollup.**

## What is implemented

- Preserve existing 62px icon rail, authoritative right inspector, canvas layer rail and world document.
- Replace the single left tool-options scroll with **three independent vertical frames**, each independently scrollable: [1] Tooling, [2] Tool Options, [3] Library / Palette.
- Drag both horizontal dividers to adjust frame height. Persist the values and context width to the current workstation-local EditorLayout JSON with safe defaults for older layout files.
- Library Terrain tab chooses real semantic Terrain Brush materials. Library Tiles and Source tabs offer an exact-source PNG picker and virtualized 32x32 cell thumbnails, without opening Terrain Studio. Clicking a thumbnail selects the source cell and switches to **Tile Brush**.
- Source sheets come from the existing canonical asset authority, load on demand, retain original PNG pixels, and are applied by the existing exact-source Tile Brush path (not the semantic brush path). The advanced Source Browser is retained.
- Pixel Brush has an interactive color picker/swatch and existing size control; **no claim of functioning source pixel painting**. This color is not used to modify source PNGs; derived image document work is P6.
- World and Scene mouse wheel zoom remain anchored under the cursor. On-canvas +/- controls and Ctrl+= / Ctrl+- zoom around **visible canvas center**, scaling pan consistently.
- Ctrl+1 / Ctrl+2 / Ctrl+3 switch Pixel/Tile/Terrain Brush families; Q/B/E/P preserved. No keyboard input while text field focused, or while PIE runs.
- Adds numerical anchoring Rust unit regression and PCC source-contract audit.

## Explicitly not implemented

- No changes to Tiled Wang topology, shoreline/inner-coast mappings, WetSand or PebblePath certification. These remain incomplete.
- No derived sprite-pixel painting, tile stamp shapes/fills/scatter, advanced brush presets, customizable keybinding UI, or end-to-end visual rule repair. These have dedicated P2–P6 follow-ups.
- No source image mutation, asset reclassification, semantic world data migration, or duplicate terrain resolver.

## Windows validation

1. Confirm baseline M2D110 has passed Full Gate (if not, retain debug evidence before installing M2D112).
2. Drop patch ZIP in project root and apply with PCC. Source must show 0.10.31.
3. Run Full Gate with Rust check/test/build and correct any native compiler failures; do not publish a failing gate.
4. Launch Vulkan Studio: Select and brushes retain a usable lower Library; drag two dividers and restart to confirm saved heights. Collapse context with X or rail toggle.
5. Library -> Source/Tiles: search `terrain_summer`, pick cells and stamp in World; inspect ground semantic values remain unchanged. Switch to Terrain and paint Grass/Sand material normally.
6. With world off-center, press + and - on canvas: the **world coordinate at viewport center** should stay fixed. Wheel over a specific shoreline tile: the **tile below pointer** should remain fixed. Verify the same in Scene. Test Ctrl+1/2/3 and Ctrl+=/Ctrl+-.
7. Compare source PNG/TSX hashes before and after; should remain identical. Do not count this as coastline certification.

## Next milestone

UA-01 P2/P3 will expose effective rule and source Wang tuple inline in Inspector and Library, create candidate source tile previews and a derived rule mapping override, persist and undo it. This vertical slice will allow a misrendered grass/sand corner to be fixed globally without editing the original atlas. Certification must include screenshot-neighborhood fixtures, source hash parity and real native world/PIE rendering.
