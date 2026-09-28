# Start here — Havenwild Bevy standalone

## Goal

Develop the Havenwild Bevy game/editor lane as an independent project with its own source history, build/run workflow, and local asset hydration. Do not make the previous Havenwild checkout a runtime dependency.

## Bootstrap

1. Extract this source-only repository into a new folder.
2. Run `PCC.cmd assets sync` if the local seed is not already hydrated, or use ForgePY directly with `ForgePY.cmd assets sync --source "<authoritative asset location>"` for first-source selection.
3. Run `PCC.cmd audit` for the fast Cargo-free source/asset/terrain integrity pass, including the full semantic -> dual-grid vertex -> recipe pipeline.
4. Run `PCC.cmd full`. PCC remains open on failure, records logs/receipts, and creates a debug bundle automatically.
5. Run `PCC.cmd run dx12` (or `Studio.cmd`).
6. Commit the generated `Cargo.lock` after the first successful dependency resolution/build.

ForgePY accepts an existing project containing `assets/elizawy`, an extracted `Terrain/Structure/Objects/FX` source tree, or source pack ZIP staging. It validates known source identities before copying anything into the local project.

## Current visual acceptance — M2D03-A

Use the active `Havenwild Latest` project and its 349 verified original core assets. Run `python tools/audit_elizawy_asset_consumption.py --check --verify-local` to distinguish a complete hydrated source set from in-scene usage. The single **SRC** browser now lists every one of the 320 original PNGs (Terrain 29, Structure 98, Objects 188, FX 5). Original credits and GIF records remain separate metadata; Characters.zip is optional and unhydrated at last check.

Press **LYR** and select Objects, Structure, Water, Elevation, Foreground, Terrain Details, or Ground. Open SRC and select a source image/category; manually select an exact rectangle (width/height in 32px cells). With Objects selected, place visually verified vegetation above grass or reeds above water; inspect the original alpha and verify underlying terrain remains visible. Non-Terrain files cannot replace Ground. Undo/redo, save to the separate `content/scenes/derived/elizawy_mapping_certification.layered.draft.json`, close/reopen and test PIE visual parity. No original v1 River fixture, PNG or older draft may change. Do not call source-region selection a certified object prefab or DG rule.

The older M2D02-D randomly scattered trees/rocks were retired. The original Summer demo PNG is **reference-only**, not the editable scene or flattened PIE backdrop. Certified cliff topology, collision, waterfall animation and a composed original-demo-style world remain future work.

## Architecture rule

Source assets, authored mapping/scene documents, and gameplay meaning are separate layers. Source PNGs remain immutable. Future terrain recipes, assemblies, animation frames, collision, elevation, navigation, and gameplay metadata must reference source identity rather than rewriting source pixels.

## Handoff rule

Once the local seed has been hydrated, asset payloads stay local. All subsequent project handoffs/packages are source-only; ForgePY/PCC and GitHub manage source while the ignored local asset tree remains in place.

## First terrain-compiler checkpoint

DG-00 is represented by versioned data contracts under `content/terrain/`. `PCC.cmd terrain status` verifies the scaffold and `PCC.cmd terrain pipeline` verifies the shared resolved 6×6 vertex path from the immutable 5×5 semantic seed. `PCC.cmd terrain certify` is intentionally expected to fail until all 16 Summer Grass/Void masks have explicit source-certified sprite/composite/unsupported mappings. Do not replace those unmapped entries with guesses.
