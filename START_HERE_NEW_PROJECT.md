# Start here — Havenwild Bevy standalone

> **Current patch intake:** PCC 1.3.5 uses manifest-first root ZIP discovery and also supports manual extract/overwrite. For governed intake, drop the ZIP unextracted in the project root or `updates/inbox`; browser-renamed ZIPs are accepted when they contain `pcc_patch.json`. For manual overwrite, extract the payload over the project and then run Full Quality Gate before publishing.



## Current checkpoint — M2D090A / source 0.9.1

0.9.1 is a certification-only repair: the Generated World behavior remains the M2D090 0.9.0 kitchen-sink implementation, but the stale Rust test now accepts the intended seasonal `terrain_*` ground sheets while still rejecting `cliff_*` sheets as cell-ground authority.

Generated World is the primary user-facing map. Source 0.9.0 regenerates the old sparse cache as the four-season kitchen-sink showcase: exact seasonal terrain/tree/plant/wildflower/cliff source substitution, 0/+1/+2 elevation intent, hamlet/town anchors, roads and cave spurs, source-exact houses/clutter/lighting, bridges, waterfalls, cliff/cave showcase features and dense biome-aware nature population. The full 320-image ElizaWy lane remains discoverable; automatic placement is context/tag gated and generated artwork is forbidden.

Run PCC Full Gate after applying this source. Then launch Studio and inspect Generated World; use screenshots to drive the next cliff/shoreline/population corrections instead of editing the immutable ElizaWy PNGs.

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


## Historical checkpoint — M2D07Q canonical template-region repair / source 0.7.2

After hydrating the existing ElizaWy assets and passing PCC Full Gate, open Studio and use **HW → World Generator** to materialize a deterministic 3×3 working region. The generator composes only exact canonical ElizaWy regions/templates; it never creates artwork. World edits are saved to the workstation-local `.forgepy/world/havenwild_world.local.json`, while original PNGs remain immutable. Use Collision / Elevation for the red 32×32 pixel mask layer. Regeneration preserves authored corrections and placement tombstones. World PIE is intentionally pending; use the Summer Scene for the existing PIE path.

## Current interactive acceptance — M2D04 canvas-first Summer study

In `C:\Users\Shifty\Desktop\Havenwild Latest`, place **only** `Havenwild_Bevy_M2D04_Canvas_First_Summer_Study_CUMULATIVE_FROM_A.pccpatch.zip` unextracted in project root. Apply via PCC 11 → 2, run Option 1 Full Quality Gate, then run DX12 (Option 3) or Vulkan. This is cumulative A→D04 and may be used over installed B or C. The complete source ZIP is a recovery snapshot, NOT a PCC inbox patch.

Launch Studio: the large canvas is clear, LYR/SRC/advanced terrain mapping are CLOSED unless requested. The initial Summer study should show **36 separate original-source objects** (one whole brick house, nine trees, vegetation, flowers, rocks and water-edge studies) over the intact pinned River terrain, not the former bare River. The seed image bytes are not in the package: hydrated `assets/elizawy` must be available. If missing, run `ForgePY.cmd assets sync --source "<authoritative local collection>"`; never substitute generated artwork. Source exact SHA-256: `python tools/audit_elizawy_asset_consumption.py --check --verify-local` validates all locally hydrated originals.

Test by clicking an EXISTING tree in SEL and holding left mouse to drag, releasing to commit. Delete it; Undo restores it. Open LYR if you want the instance list; use SRC on demand to inspect and select original source regions. Ctrl+S (Save Scene) creates/updates ONLY `content/scenes/derived/summer_world.layered.draft.json`; relaunch and confirm the edit persists. The old C-era `content/scenes/derived/elizawy_mapping_certification.layered.draft.json` and v1 drafts are preserved, NOT automatically merged/replaced. The existing **Play Scene** button is the single PIE entry; it toggles to Stop and renders a snapshot of the live same scene including unsaved visuals. Esc exits. Capture screenshots after opening and inside PIE and attach the automatically generated PCC debug bundle on any gate/runtime issue.

This is a **visual composition study**, not certified complete Summer demo: no bridge, waterfall, drainage/cliff topology, structure collision, navigation or dual-grid certification is claimed. The underlying 40×28 River terrain remains the original fixture, with source-bound decorative placements on top. There is no separate V2 view or second game launcher. Do not mark this pass Windows GREEN before its native Full Gate and manual interaction/PIE tests pass.

## Architecture rule

Source assets, authored mapping/scene documents, and gameplay meaning are separate layers. Source PNGs remain immutable. Future terrain recipes, assemblies, animation frames, collision, elevation, navigation, and gameplay metadata must reference source identity rather than rewriting source pixels.

## Handoff rule

Once the local seed has been hydrated, asset payloads stay local. All subsequent project handoffs/packages are source-only; ForgePY/PCC and GitHub manage source while the ignored local asset tree remains in place.

## First terrain-compiler checkpoint

DG-00 is represented by versioned data contracts under `content/terrain/`. `PCC.cmd terrain status` verifies the scaffold and `PCC.cmd terrain pipeline` verifies the shared resolved 6×6 vertex path from the immutable 5×5 semantic seed. `PCC.cmd terrain certify` is intentionally expected to fail until all 16 Summer Grass/Void masks have explicit source-certified sprite/composite/unsupported mappings. Do not replace those unmapped entries with guesses.


### GitHub source authority

Canonical origin: `https://github.com/shifty81/HavenwildBevy.git` (`main`). Run Full Gate first. Primary workflow option 2 can then initialize/adopt Git and publish the exact certified source; existing GitHub history is preserved and adopted with a mixed reset that does not overwrite working files.
