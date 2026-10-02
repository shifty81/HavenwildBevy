# M2D04 — Canvas-first Summer composition study

## What this actually delivers

- One *active* derived Summer scene: an immutable, unchanged source-pinned 40×28 River ground fixture plus 36 individually selectable/movable/erasable source-bound instances in the existing `SceneV2` runtime. **This is a visible authored content change** from C's zero-object default, not another empty scene and not a flattened demonstration screenshot.
- Nine exact original single-tree image regions (`Terrain/trees_summer.png`), one complete original brick house source image (`Structure/Structures/Brick House B.png`), source-backed summer plants/shrubs, flowers, rocks and explicitly provisional river-edge planting studies from `Terrain/plants_summer.png`. Five manifest-listed PNG paths in total; every rectangle in exact source bounds; original 349-file catalog and source bytes unchanged.
- Derived seed is `content/scenes/summer_world.visual_seed.v1.json`, pinned to original `content/scenes/elizawy_mapping_certification.scene.json` SHA256. It is used ONLY when the new working draft does not exist; the program does NOT repopulate/regenerate an existing user edited Summer draft on subsequent launch. The 36 objects are source placements for review, **not** certified prefab, interaction, topological or collision definitions.
- Saves ONLY to `content/scenes/derived/summer_world.layered.draft.json`. The prior C layered draft, legacy v1 local draft, original mapping fixture, source manifest, source PNGs, and retired historical docs are never overwritten by startup or scene saving. The original River remains available as a recovery/regression reference rather than being a competing UI scene selector.
- Canvas-first once-only layout migration: left tool rail remains 52px outside canvas; LYR closes by default, advanced mapper/source picker opens only on demand as an optionally dockable/floating window, no default right-side mapping review, no permanent inspector. The selected-object properties window opens only via selection + Properties, double-click, or `I`. New user layout decisions persist after first normalization.
- Single existing **Play Scene / Stop PIE** control takes the active layered scene snapshot, including unsaved visual placements. SEL uses left-click + hold-and-drag with live preview; Delete, Undo/Redo, and Save Scene use the same transaction/history system.

## Source exactness, not unsupported certification

`Terrain/trees_summer.png`, `Terrain/plants_summer.png`, `Terrain/wildflowers_summer.png`, `Terrain/Rocks, Grasslands.png`, and `Structure/Structures/Brick House B.png` were checked against the original public artist repository's source PNG representations and the canonical project's file dimensions/byte counts. Rectangles were selected from visibly distinct whole sprites or deliberately labeled provisional visual details. The workbench does not generate or modify source pixels. The canonical original SHA-256 values remain in `content/catalog/core_source_manifest.json` for authoritative local verification, not in this package.

The earlier guessed/retired M2D02-D atlas board and derivative Tiled study are NOT reintroduced. Structural height, river drainage, cliff corner/end grammar, waterfall drop, bridge deck, approved object-footprint collision, navigation, full original DemoGame Summer recreation and all 16-mask DG classification/certification remain OPEN. Grass/water placement of tentative water-edge studies does not establish ecological/art approval or movement behavior. `305/305` recovered Summer source cells is distinct from DG `0/16` certified.

## Acceptance sequence on user's Windows workstation

1. Drop cumulative `.pccpatch.zip` unextracted into the root of `Havenwild Latest`; use PCC 11 → 2, then Full Gate; do not feed source-only ZIP to PCC. Existing `assets/elizawy` stays local and ignored.
2. Run `python tools/audit_elizawy_asset_consumption.py --check --verify-local` — actual hydrated file hashes must pass. A failing asset is a hard source requirement; do not substitute fallback sprites.
3. Run Studio via one existing PCC DX12/Vulkan route. Confirm the canvas opens without old C left/right panel sandwich, and **immediately contains the 36 distinct objects**; LYR and source/terrain tools should be hidden initially but open/close on demand.
4. Click the obvious whole house or tree under SEL, hold left button and drag. Verify live move preview, release commit, Delete and Undo/Redo, optional contextual properties only if opened. Save Scene, restart and verify exactly the persisted state; original pinned River hash unchanged. Existing C-era local layered draft remains at its prior path.
5. Use **Play Scene**, confirm the same 36 object visuals plus any unsaved moved object are visible; movement remains provisional terrain-role collision. Stop with existing toggle/Esc and verify editing resumes at prior view and edits persist. Do not interpret this as production gameplay certification.
6. Supply screenshots (Studio and PIE) and auto-collected debug bundle if any startup, rendering, input or build error occurs. An intermittent historical DX12 `ResizeBuffers 0x887A0001` should be reproduced with evidence before speculative renderer changes.

## Verification boundary

Source-only Python validators, source document SHA/bounds/placement checks, previous historical selftests, PCC manifest validator and A/C package replay are available in this environment. **No Cargo/rustc, original hydrated art corpus or Windows GPU here.** Thus neither a successful native build nor pixel-perfect art/interaction acceptance may be claimed from packaging alone. Cumulative patch replays identically from published A and previous installed C; do not push an unverified update to GitHub main automatically.
