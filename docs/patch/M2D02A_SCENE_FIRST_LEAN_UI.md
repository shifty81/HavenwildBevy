# M2D02-A — Scene-First Lean UI (native test candidate)

**Base**: source v0.5.6, ForgePY 0.4.6, PCC 1.2.6, M2C4 and M2D01 installed. Focused root-drop patch. No assets, recipes, active worldgen or scene fixture rewritten.

## What changes

- Preserve native Windows title, narrow separate global File/Edit/View/Scene/Tools/Help row, full central scene canvas. Keep a fixed ~52px dark semi-opaque rail *outside* the canvas; optional 178px adjacent layer guide, also outside the canvas.
- Select is now safe startup default, instead of stamping a tile on any scene click. Four currently working rail actions: Select, Paint 32px source tile, Erase 32px source tile (preserves legacy semantic hint), and Sample 32px source tile. Whole-object Move and Region remain disabled until v2 object/transaction support. Source drag/drop expresses explicit paint intent.
- Three compact layer groups show the proposed seven logical render categories. **The existing v1 scene is one combined source-tile plane; the category guide does not claim visibility, painting or storage of seven real layers.** It starts collapsed. Workspace-local layout JSON defaults are backwards compatible.
- View options include transparent-pixel checkerboard *under the selected source tile only* (for identifying alpha vs source colors); this is not a claim to repair magenta artwork or atlas bleed. Uses existing nearest-image policy. Canvas upper-right Fit, + and - actions. F11 focus hides rail, status and advanced mapper without changing persistent docking preferences.
- When a cell is selected, a compact canvas overlay allows Sample, Source info or Erase; these operate on one source cell only, not a presumed multi-tile object. Ctrl+S saves the v1 scene using the existing atomic writer; Ctrl+Z/Y and Delete use the existing draft history. **Whole-array undo remains until M2D02-B.** Existing Save All for technical documents is still available under File.
- Native scene generator/override adapter is absent, so Preview regeneration, Regenerate and whole-scene Pixel tools are explicitly disabled with explanation. No simulated success or source promotion. M2C4 ElizaWy review and DG technical recipe remain accessible under Tools/Source, preserving recovered evidence and existing 0/16 certification state.

## Windows test after PCC update

1. Option 11 apply, Option 1 Full Gate, Option 3 DX12 or Option 4 Vulkan. Record any Rust compile error/debug bundle. A Python/static PASS cannot substitute for these.
2. Opening Studio: canvas dominant, mapper hidden on a fresh local layout, Select active; existing saved layout choices may reopen panels. Normal click selects and **must not** modify/save scene. Click the rock and use alpha checker View option. Determine if colors originate in the pixels, transparency or atlas sampling; do not edit original art.
3. Source button opens the existing mapper. Select a source cell, choose Paint, place a tile; choose Sample to pick that tile, Erase to remove it; confirm Delete, Ctrl+Z, Ctrl+Y, Ctrl+S, reopen.
4. LYR opens three compact groups with an explicit v1 combined-plane notice; close it without modifying the authored scene. F11 focus on/off restores prior optional tools; Fit/+/- work without canvas stamping.
5. Scene menu honestly disables Regenerate. Inspect M2C4 review still works. No new recipe certification, original art modifications or scene schema migration should occur.

## Next

M2D02-B: explicit scene-v2 draft/adapter, stable assembled-object IDs, real sparse semantic/visual layers, operation-based undo and ownership/tombstones. Only after B should whole-object selection, scene-level Pixel and generation buttons be enabled.
