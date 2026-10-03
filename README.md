
### M2D082F1 — deterministic fill regression repair

Source 0.8.10 keeps the 0.8.9 runtime/world behavior intact and repairs the diagonal-land Rust regression so it validates the deterministic source-backed Grass fill variant chosen for the current seed/vertex instead of pinning one arbitrary safe-fill atlas coordinate. It also carries the rustfmt normalization already performed by the Windows Full Gate.
# Havenwild Bevy — standalone source

**Current deploy checkpoint:** source **0.8.10**, ForgePY **0.4.10**, PCC **1.3.5**. The PCC primary operator flow is **1 Full Gate → 2 Commit + Push certified source → 3 Patch Scan / Review / Apply**. Fresh/source-only deployments hydrate missing core ElizaWy assets automatically from the exact pinned GitHub revision first; remembered/local mirrors are fallback-only. Source-only packaging now excludes root update ZIPs, legacy `.patch` handoffs, local operational state, hydrated assets, derived workstation drafts, and one-off repair/package scripts so a packaged source checkpoint does not recursively carry deployment debris.


### M2D082F land-connectivity + full Summer material promotion

Source 0.8.9 keeps the corrected 0.8.7 semantic-cell/dual-grid coordinate authority and replaces the rejected 0.8.8 shoreline projection with a simpler visual rule: **ambiguous diagonal water does not cut a square notch between near-touching land**. Live mixed terrain uses complete authored 32x32 source tiles only; historical 16x16 quadrant composites remain evidence/tooling records and are no longer used by the live world resolver. Cardinally continuous water stays open.

The Summer semantic lane now exposes **Grass, Dirt/MudBank, Sand, Wet Sand, Shallow/River Water, Deep Water and Pebble/Stone Path**. Existing source groups provide 3 Sand variants, 3 WetSand variants, 4 PebblePath variants and one conservative DeepWater center, plus source-authored Grass↔Sand, Sand↔WetSand, Sand↔Water, Shallow↔Deep and PebblePath↔Dirt transition roles where those families actually exist. Generated World now uses a visible coast sequence **Deep Ocean → Shallow Water → Wet Sand → Dry Sand → inland land**, dirt along interior river/lake edges, and a deterministic meandering PebblePath road spine per island as the first road placeholder.

Material abbreviations have been removed from the permanent left tool rail; material choice now lives in World Generator / World Authoring. This is an intermediate shell cleanup ahead of the larger workspace-bar/tabbed application refactor.


### M2D081C water authority + PCC manifest-first intake

- `summer_water_fill` remains an eight-cell recovered **RepeatableFill** source group, but it is no longer treated as an ordered temporal animation.
- Normal homogeneous RiverWater now uses one conservative static source fill. The other seven cells remain available as explicit source/detail regions for later deliberate placement.
- RiverWater animation is disabled until source evidence identifies an explicitly ordered frame sequence.
- PCC root-drop discovery is manifest-first: any root ZIP containing a valid root `pcc_patch.json` is discovered even if the browser renamed it. Ordinary source/debug ZIPs remain ignored.
- Manual extract/overwrite remains supported. If the installed target versions and exact declared file hashes already match a governed patch, PCC reconciles/archives that handoff instead of trying to apply it again. Older superseded patch handoffs are archived rather than poisoning Full Gate.
- M2D082 cliff groundwork is now deterministic: `Terrain/cliff_summer.png` exposes **205 canonical 32x32 cliff source cells** and **212 retained historical reference entries** across five evidence families, with zero semantic/runtime promotions until topology is classified.

### M2D082D dual-grid coordinate authority repair

Source 0.8.7 fixes the live Generated World paint/render mismatch: semantic cells remain integer-aligned authoring squares, while dual-grid output tiles render centered on their terrain vertices with the required half-cell offset and N+1 by M+1 vertex coverage. One Grass click now resolves around the selected cell instead of down/right from it. Exact-source Direct Tile overrides remain locked to the selected 32x32 semantic cell. Deterministic virtual neighbours close the outer materialized boundary without inventing artwork.

### M2D082E seam-safe shoreline composite experiment — superseded
Source 0.8.8 attempted full-tile projection for seam-sensitive junctions. Visual review showed that it traded 16x16 seam leaks for larger square water cutouts, so 0.8.9 supersedes that live behavior. The historical patch/evidence remains documented; the runtime now uses the land-connectivity-first rule described above.

### M2D082C1 terrain authority version-skew repair

Source 0.8.6 keeps the M2D082C Generated World behavior intact while aligning all Summer runtime consumers with profile v6. M2D080 compatibility checks, Rust AssetAuthority validation, and Native→Bevy convergence now accept the v6 mixed-bank/static-water authority. Terrain certification no longer hard-codes a specific 0.8.x Studio patch version.

## M2D082C Generated World consolidation

- **Generated World is now the primary editor surface.** A fresh or migrated world auto-materializes a 5×5 chunk working region; the old River/Summer scene remains internal regression evidence instead of the normal authoring destination.
- The macro planner establishes four ocean-separated island intents: Spring, Summer, Autumn and Winter. Each island has deterministic interior river/lake intent plus coast, meadow, light-woodland and dense-forest biome regions. Seasonal **visual** substitution is still evidence-gated; current certified ground rendering remains the Summer terrain authority until the other seasons are mapped.
- Mixed Grass/Dirt/Water junctions are now **water-continuity-first**. All 36 three-material corner states use RiverWater as the exact-source quadrant background so narrow channels, coves and mixed-bank inlets do not fragment into the visible wedge/notch defects reported during visual review.
- Large tree draws use a one-source-pixel UV inset on `Terrain/trees_*` regions. This preserves the world-space footprint while excluding the atlas guide pixels that were appearing as faint rectangular/vertical bars around trees. Source PNG bytes remain untouched.
- `content/worldgen/elizawy_worldgen_asset_catalog.v1.json` exposes **all 320 canonical ElizaWy PNGs** to Generated World discovery: 29 Terrain/Nature, 98 Structure, 188 Object and 5 FX sheets. Automatic placement remains authority-gated; unmapped sheets are not randomly scattered.
- `content/worldgen/havenwild_generated_world.v1.json` locks the purpose-first roadmap for settlements, meandering connectivity roads, biomes and deterministic weekly-reset procedural caves (`hash(world_seed,cave_id,game_week)`).

Large ElizaWy/LPC source assets remain local/hydrated and are intentionally not included in this repository handoff.

---

# M2D07 A–O — unified ElizaWy asset authority + source-backed 3×3 world composition

Source checkpoint **v0.7.2**. M2D07Q repairs the first Windows Cargo-test authority failure exposed after M2D07P: every source-exact Summer object-template region is now registered in the same canonical ElizaWy runtime lane before templates are emitted. The 20 object-template regions are source metadata only; no artwork is generated or altered. This cumulative pass collapses known ElizaWy source identity/evidence into one canonical asset lane and adds deterministic 32×32 chunk composition on the existing infinite canvas. **No image asset is generated**: worldgen may only select/place exact canonical ElizaWy source regions and source-exact templates. Authored visual/semantic/elevation/collision corrections and worldgen-placement tombstones survive 3×3 regeneration. World Generator, Chunk Manager and ElizaWy Asset Authority are real overlay panels launched from HW; they never resize the canvas. World PIE remains intentionally disabled until runtime chunk traversal/streaming is certified. See `docs/patch/M2D07_UNIFIED_ELIZAWY_WORLD_COMPOSITION_15_PASS.md`.

## M2D06-B — Pixel collision authoring / red impassable overlay (source 0.6.0)

Adds a dedicated authored collision layer without changing original PNG pixels. The permanent right layer rail gains a COL toggle; the Collision / Elevation application panel can show effective impassable areas as a translucent red world overlay, switch Block/Clear brushes, create/remove explicit 32x32 collision masks per logical world cell, and edit the selected mask down to individual source-pixel resolution. Full mode includes an interactive 32x32 mask editor. Optional Edit on world canvas converts a click to the exact 1/32-cell collision pixel under the cursor, allowing corrections against the visible world artwork at high zoom. Explicit pixel masks take precedence over coarse traversal overrides and provisional terrain collision in PIE; reverting the mask restores automatic/coarse behavior. All edits use the existing SceneV2 transaction/undo/save path and never infer collision automatically from sprite alpha. See `docs/patch/M2D06B_PIXEL_COLLISION_AUTHORING.md`. Windows Full Gate and interaction verification remain required.

## M2D06-A — Canvas-first shell / overlay tool applications (source 0.5.9)

Locks the Havenwild Studio shell around the workflow approved after M2D05-B: the left direct-edit tool rail and right active-layer rail are the only permanent side rails; feature-rich tools open as floating/dock-over-canvas windows and **never resize or reflow the world canvas**. A bottom-left Havenwild launcher (Ctrl+Space) opens the real Scene/Layers, Collision/Elevation, Source Browser, Terrain Rules, Source Evidence and selected-object panels. World Generator, Pixel Studio and Animation Studio are listed but intentionally disabled until their real functionality is wired. A second top command toolbar exposes real authoring commands, and centralized command metadata drives Q/B/E/P canvas shortcuts plus tooltip shortcut text. Source/terrain/evidence, collision/elevation and Scene/Layers support Compact/Full presentation while preserving one underlying authoring state. See `docs/patch/M2D06A_CANVAS_FIRST_SHELL.md`. Windows Full Gate and visual interaction remain required.

## M2D05-B — Historical Evidence null-atlas repair (source 0.5.8)

Fixes the Windows Full Gate 2026-09-29 after M2D05-A: JSON `canonicalAtlas: null` is meaningful for 21 source-region seasonal comparisons and their 21 review clusters without a canonical target. Both fields deserialize as Option<String>; no source artwork or evidence JSON is modified. Invalid one/multiple matches without a named atlas still fail validation. The Source Evidence browser displays an explicit no-target message and can filter winter_ice. The previous 50 passing Rust tests, M2D05-A chunk overlay, collision editing, Summer scene, and M2C4 optional evidence behavior remain unchanged. See docs/patch/M2D05B_EVIDENCE_NULL_ATLAS_REPAIR.md. Windows Full Gate and Studio launch are still required to certify this follow-up.

## M2D05-A — Recovered evidence / world-authoring foundation (source 0.5.7)

PCC cumulative patch from the exact M2D04 source 0.5.6. Versioned, read-only B48R9 source recovery with all 1,781 regions in the Source Evidence browser; optional M2C4 derivative evidence no longer prevents access; one runtime project root resolver; configurable canvas-only chunk grid; selected-cell structural/collision-height editor with scene-v2 undo/redo/save, explicit PIE traversal overrides and retained Terrain Rules help. This is not implemented chunk streaming, full semantic terrain painting, pixel image editing, automatic approvals or full finished Summer scene. See `docs/patch/M2D05A_CHUNKS_EVIDENCE_STRUCTURAL_AUTHORITY.md` and `docs/architecture/HAVENWILD_WORLD_AUTHORING_TARGET.md`. All existing M2D04 artwork/scene controls and local draft paths remain intact. Native Windows Cargo gate and visual verification are required before marking GREEN.

## M2D04 — Canvas-first Summer composition study (source-only checkpoint)

The **initial Summer working document is now visibly populated** with 36 independently placed source-bound original PNG regions (house, nine complete tree sprites, foliage, shrubs, flowers, rocks and tentative water-edge details). Its 40×28 protected River-ground base is **not** a finished re-creation of the artist's larger original Summer demo; unreviewed cliff/bridge/waterfall topology and production collision have NOT been fabricated or certified. The bundled seed references original locally hydrated art; this ZIP does not include original PNG bytes. New work is saved in `content/scenes/derived/summer_world.layered.draft.json`; the original pinned terrain and ALL earlier derived River drafts are left untouched.

On first M2D04 launch the canvas occupies the main workspace: 52px tool rail only; LYR and SRC are on demand, mapper defaults to a resizable floating window, and selected-object properties open only if explicitly requested. Existing saved two-sidebar C layouts are migrated **once**, then future user layout choices persist. SEL clicks and drags the individual placed objects; Save Scene preserves their changes; the **existing one Play Scene/Stop PIE toggle** snapshots the same live scene. See `docs/patch/M2D04_CANVAS_FIRST_SUMMER_COMPOSITION_STUDY.md` for scope, original-source provenance, recovery and Windows tests. Native Windows build, source PNG hash verification and visual acceptance remain pending. GitHub `main` is the previously published M2D03-A baseline until verified.

# Havenwild — standalone Bevy lane

This repository is the clean, independent Bevy lane for Havenwild. It is intentionally source-only: game/editor source, authored scene data, mapping contracts, asset manifests, and ForgePY are versioned; the large ElizaWy/LPC Revised source collection is hydrated locally by ForgePY from the exact pinned ElizaWy GitHub revision first, with remembered/local mirrors used only as fallback.

## Current checkpoint

Historical DG-03 checkpoint **v0.6.0**, ForgePY **0.4.6**, PCC **1.2.6**. That pass hardened the pre-build DG-03 lane: PCC now runs a Cargo-free Rust source/module/dependency audit, project metadata pins `bevy_egui` 0.42.0 alongside Bevy 0.19.0 and ForgeGUI, every semantic cell is checked for exact four-vertex invalidation including edges/corners, corner-bit orientation is regression-locked, ForgePY FULL honors every Doctor failure, and a successful Cargo build must also reproduce its generated lockfile with `cargo metadata --locked`.

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

Studio opens the 40×28 active Summer composition: immutable source-addressed original River terrain beneath 36 individually editable original-source art instances. Its live document is the source-fingerprinted seven-layer scene model, saved atomically to its own workstation-local Summer draft. All 320 cataloged original PNGs are searchable by family and non-Terrain images load on demand. No source-art pixels are silently generated, modified, or replaced. The former M2D02-D guessed 72×28 tree/rock layout was retired; its historical documentation and locally retained drafts are not the active scene. Source recovery 305/305 occupied Summer addresses is separate from 0/16 current Grass/Void mask certification.

The Scene / Layers panel is available with LYR, CLOSED by default to preserve the canvas, and reports the actual placed-object count (initial Summer seed: 36). Complete objects are source-bound instances, independent of the underlying terrain. Selecting and dragging an object in SEL shows an in-canvas movement preview; the release commits one undoable move. The existing Play Scene button takes a sandbox snapshot of the same authored visuals. Collision and structural elevation are still provisional and not certified. Correct multi-piece cliffs, waterfall and bridge placement, object-geometry certification, characters and the full authored Summer demonstration remain open, rather than being represented as completed.

Native Windows decorations remain authoritative. `PCC.cmd` owns source-only update intake, Full Quality Gate and automatic debug evidence. Large local assets and workstation drafts remain excluded from GitHub; `main` published M2D03-A as the initial baseline and later passes should only be pushed after native quality gates and user verification.


### First deployment bootstrap

A source-only ZIP intentionally contains neither hydrated ElizaWy art nor a `.git` directory. On a fresh extraction, do this once before the normal 1 -> 2 workflow:

1. `PCC.cmd` -> **6 Assets / hydration** -> **2 Hydrate / repair core assets (pinned GitHub first)**. No local asset path is normally required. PCC/ForgePY restores the exact pinned ElizaWy GitHub revision and verifies it against the checked-in manifest; binary assets are byte-exact, while the legacy `Credits.txt` manifest entries tolerate only Git LF/Windows-CRLF newline materialization and still preserve the raw GitHub blob bytes locally. Hydrated art remains ignored and is never packaged. Use the separate local fallback only if upstream hydration is unavailable.
2. `PCC.cmd` -> **8 Source control / GitHub** -> **4 First-time Git bootstrap**. For a new/empty remote PCC creates a local source baseline. If the remote already has the selected branch, PCC fetches that history and uses a mixed reset to attach the extracted folder without replacing its files; the extracted source then appears as ordinary local changes for certification.
3. Run **1 Full Quality Gate**.
4. Run **2 Commit + Push certified source to GitHub**. The canonical origin is `https://github.com/shifty81/HavenwildBevy.git`; if local Git/origin is missing, option 2 can initialize/adopt it without changing the certified source bytes.

Full Gate now requires the complete 349-file core manifest immediately after governed patch intake. A partial tree automatically enters pinned-GitHub repair, then must pass full manifest identity verification before any expensive source/Cargo certification continues. This prevents a 321/349 tree from looking ready just because the four active Summer sheets are present.

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


## Canonical GitHub authority

Havenwild Bevy's source repository is `https://github.com/shifty81/HavenwildBevy.git` on branch `main`. PCC 1.3.2 certifies a publishable-source fingerprint independently of Git history, so a clean extracted deployment can run Full Gate first and let primary option 2 initialize Git or attach/adopt the canonical origin afterward. If the local PCC bootstrap commit and GitHub history are unrelated, PCC uses `git reset --mixed origin/main`: HEAD/index attach to GitHub history while the current working files remain untouched and become normal certified source changes. No automatic merge, rebase, hard reset, stash, or force-push is used.
