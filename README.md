## M2D03-A source-first corrective checkpoint

All 349 canonical non-character files are inventoried; 320 original PNGs are now searchable in Studio's SRC browser across Terrain/Structure/Objects/FX, with non-Terrain sheets loaded on demand rather than permanently preloaded. The existing Scene V2 sparse visual layers are active for explicit manual source placement and are composited above the unchanged inherited River ground; PIE freezes the same layered visual data. This is a **source access and layering checkpoint**, not complete 349-asset gameplay utilization, visual alpha certification, scene composition certification, or a completed Summer reference recreation. Details and Windows tests: `docs/patch/M2D03A_ORIGINAL_SOURCE_CATALOG_LIVE_SCENE_LAYERS.md`.

> **Current visual baseline (M2D02-E):** the synthetic M2D02-D rock/tree scene is retired. Studio opens the pinned source-addressed Summer River fixture only; original assets are in Source Browser. See `docs/patch/M2D02E_SOURCE_EXACT_RECONCILIATION.md`.

# Havenwild — standalone Bevy lane

This repository is the clean, independent Bevy lane for Havenwild. It is intentionally source-only: game/editor source, authored scene data, mapping contracts, asset manifests, and ForgePY are versioned; the large ElizaWy/LPC Revised source collection is hydrated locally by ForgePY from the authoritative asset source you already maintain.

## Current checkpoint

Source checkpoint **v0.5.6**, ForgePY **0.4.6**, PCC **1.2.6**. This pass hardens the pre-build DG-03 lane: PCC now runs a Cargo-free Rust source/module/dependency audit, project metadata pins `bevy_egui` 0.42.0 alongside Bevy 0.19.0 and ForgeGUI, every semantic cell is checked for exact four-vertex invalidation including edges/corners, corner-bit orientation is regression-locked, ForgePY FULL honors every Doctor failure, and a successful Cargo build must also reproduce its generated lockfile with `cargo metadata --locked`.

## First start on Windows

```bat
ForgePY.cmd setup --source "D:\path\to\your\ElizaWy-or-existing-Havenwild-asset-source"
ForgePY.cmd run --backend dx12
```

`Studio.cmd` is the direct DX12 Studio launcher through PCC. Use `PCC.cmd` for the persistent interactive control menu, or `ForgePY.cmd` for the lower-level backend menu.

ForgePY remembers the validated asset source in `.forgepy/local.json`, copies verified working assets into `assets/elizawy/`, builds a local source catalog, fetches Rust dependencies, streams Cargo output into the console, and records logs under `.forgepy/logs/`.

## Project boundary

- This is its own Bevy repository/project, not a branch of the older Havenwild editor/runtime tree.
- The older Havenwild project remains a read-only donor when a proven feature is intentionally ported.
- ForgeGUI_Core stays an external pinned dependency. Cargo/ForgePY obtains it; it is not copied into this repository.
- ElizaWy/LPC Revised art stays immutable at source. ForgePY copies validated working files into this project; the mapper edits scene/mapping documents, never the PNGs.
- Characters are optional/on-demand because the archive expands to 63,991 files; after local hydration they remain ignored and never enter source handoffs.
- The alternative four-season archive is never automatically merged into the canonical family.
- Future handoffs are source-only. Hydrated art, character files, optional archives, and generated asset catalogs remain local and ignored.

## Current implemented Bevy surface

M2D02-D corrects the former M2D02-C atlas-board mistake. Studio now opens a single **assembled 72x28 ElizaWy Summer visual-integration landscape**. The protected original 40x28 Summer River fixture is embedded byte-for-byte, surrounded by authored source-addressed grassland and 18 separate source-linked tree/rock visual samples. Original atlas PNGs are never pasted onto the scene. Select a complete visual sample to inspect or remove it; undo/redo and the separate `content/scenes/derived/elizawy_mapping_certification.assembled.draft.json` retain corrections. The former C board draft is preserved at its different old filename. This is still an integration draft: recovered source identity (305 Summer cells) is **not** recipe certification (0/16 DG). See `docs/patch/M2D02D_ASSEMBLED_MAPPING_ENVIRONMENT.md`.

The present runtime is a deliberately small mapping/editor seed: one Bevy window, one primary Bevy/egui context, one permanent central pixel canvas, the four established sheets plus every hydrated Terrain image from the canonical manifest (29 Terrain images in the current manifest), a dockable Terrain Mapper (left/right/bottom/floating), exact source-rect placement, click/drag replacement, Delete/Erase removal, pan/zoom, undo/redo, and persistent draft-scene save. Mapper layout is workstation-local under `.forgepy/` and never enters source handoffs.

The standalone split also wires ForgeGUI creator visuals/icon fonts plus ForgeGUI borderless resize/snap helpers. Native Windows framing remains available with `--native-frame`.

This is not yet the full Havenwild game client. World generation, shared game/editor rendering contracts, collisions/navigation, characters, gameplay, multiplayer, Pixel Studio, Logic Studio, Sound Studio, and the rest of the planned tool suite still need implementation.

See `docs/STANDALONE_AUDIT.md`, `docs/FORGEPY_WORKFLOW.md`, and `docs/ROADMAP.md`.

## Project Control Center / GitHub refresh

`PCC.cmd` launches the persistent Project Control Center supervisor; ForgePY is its backend. Once this standalone source lives in its own GitHub checkout, PCC build/full jobs use ForgePY to fetch `origin` and fast-forward a **clean** current branch before compiling. Local modified work is never reset, stashed, rebased, or overwritten; refresh is skipped and the local worktree is preserved.

```bat
PCC.cmd source status
PCC.cmd source refresh
PCC.cmd build
PCC.cmd full
```

## Project Control Center v1.2 / ForgePY v0.4

`PCC.cmd` is now the resilient top-level operator. ForgePY remains the backend. PCC keeps a persistent interactive host, streams child output live, records `.pcc/logs` and JSON receipts, auto-builds a source-only debug bundle on failed jobs, provides Git/assets/terrain/update/diagnostic surfaces, and governs transactional source-only `.pccpatch.zip` updates. A child Cargo/ForgePY/Git failure returns to PCC instead of making the window disappear.

DG-01/DG-02/DG-03 is live in the dockable Terrain Mapper: the 16-mask Grass/Void board can classify an exact source cell as a sprite, build and directly edit composite parts with pixel offsets, locate/replace/remove individual parts, preview the actual resolved source pixels, mark a topology unsupported, save metadata, and step through unresolved masks. A separate 5×5 semantic-only lab drives the shared NW/NE/SW/SE resolver, intentionally covers all 16 masks without storing atlas coordinates, and now resolves the complete 6×6 terrain-vertex field with dirty-vertex highlighting and semantic undo/redo. The mapper also carries forward the recovered Summer authority separately: all **305/305 non-transparent cells** from `terrain_summer.png` are source-indexed again, while the older 641-entry crosswalk/topology/tuple work is tracked as historical evidence until its raw metadata is recovered and normalized. `PCC.cmd terrain recovery` validates this boundary; `PCC.cmd terrain recover --source <older Havenwild root>` searches a donor source tree for the old metadata without packaging its assets.

For the corrected v0.5.6 Windows certification/build session, follow `docs/BUILD_DEBUG_SESSION_V056.md`; it distinguishes expected recipe incompleteness from build failures and preserves the evidence needed for the next correction pass.

The v0.5.6 FULL gate verifies the Rust 1.95+ toolchain/rustfmt prerequisites and requires a generated `Cargo.lock` after the successful Cargo build so the first Windows GREEN checkpoint is dependency-reproducible.
