# M2D03-A — Original ElizaWy catalog + live source-exact scene layers

## Installation / identity

Cumulative source checkpoint from the **M2D02-F1 Windows-green local baseline**. Root-drop `.pccpatch.zip` with `PCC` Option 11 -> 2, then Option 1 Full Gate and Option 3 DX12. Source-only complete rollup is a separately packaged checkpoint and must not be fed to the PCC update inbox. Active user working directory: `C:\Users\Shifty\Desktop\Havenwild Latest`; previous `Havenwild Bevy` installation is a retained read-only donor, never an update destination.

## Audit: present is not the same as used

The canonical non-character manifest contains 349 original files: **320 PNGs, 28 TXT credit files, 1 GIF**. The family split is Terrain 29 PNG + 1 TXT; Structure 98 PNG + 14 TXT + 1 GIF; Objects 188 PNG + 12 TXT; FX 5 PNG + 1 TXT. These files are discoverable and their identities are retained in `core_source_manifest.json` and `mapping_inventory.v1.json`, but previously Studio only loaded the 29 Terrain PNGs. The active pinned River v1 fixture references **two source PNG paths** (terrain_summer and cliff_summer). It does *not* demonstrate usage of all available source art.

`Characters.zip` is a separate optional original collection member, not counted in 349, not present in the user's last asset status; never claim it is hydrated or production-ready. `FourSeasonAlternative.zip` has its own different source identity and is never auto-merged into the canonical original pack. The original `Credits.zip`/per-file credit documents must remain intact; this pack is LPC Revised multi-author artwork, not solely ElizaWy's personal artwork. Preserve provenance/license review for any release.

## Source browser and load behavior

All **320** canonical original PNGs now appear in one filtered Source Browser, grouped by Terrain / Structure / Objects / FX. It does not invent a new atlas, flatten art, regenerate anything, or preload all image textures. The existing 29 Terrain PNGs remain requested for old recipe previews; the 291 non-Terrain PNGs are loaded only when selected or referenced by a local live scene draft. The audited manifest constrains paths and image dimensions. Text/credit files and the GIF remain cataloged as non-PNG source records, not misrepresented as editable texture sheets.

Source selection supports an explicit source rectangle with a user-selected cell width/height. This is manual region selection, **not automatic detection/certification of sprite/object boundaries**. Manual stamps are labeled unapproved source-bound visual placements. The adjacent Grass/Void DG recipe controls fail closed for non-Terrain selections and out-of-bounds 32px cells; searching a Structure/Object/FX source cannot accidentally assign it to the terrain recipe. Empty object-layer selections no longer misleadingly report the inherited Ground sprite as an object. Any complete-object composition requires actual visual/provenance review before a reusable prefab/recipe may be published.

## One scene / active v2 visual layers

The source-exact imported **v1 River fixture remains immutable**. `SceneV2` (the existing B1 type/transactions, not another format) is now the active derived overlay for authoring. Sparse visual layer edits use the existing Ground, TerrainDetails, Water, Elevation, Structures, Objects, Foreground channels. The Layers button selects the target explicitly. Ground accepts only Terrain artwork; vegetation, water reeds, building parts and other overlays do not replace the original base. Transparent source pixels reveal the composited lower layer. The renderer preserves exact source rectangles rather than manufacturing a replacement texture. Multi-cell regions are drawn at their actual source-pixel size. Existing user v1 drafts are imported as the baseline if present. The new save path is `content/scenes/derived/elizawy_mapping_certification.layered.draft.json`, separate from every previous draft. Saving revalidates original v1 bytes and uses the existing atomic writer. No silent migration, overwriting source, or legacy derived-draft deletion.

PIE uses a frozen copy of the exact same visual v2 document plus its resolved Ground base. Its provisional v1 traversal does not infer collision, elevation, bridge access or water-anchoring semantics from artwork. The painter is shared; no flattened demo screenshot is used as runtime background. The unverified `M2D02-D` arbitrary rock/tree layout remains retired.

## Honest acceptance and limitations

This corrective pass restores visual authoring access and layering, *not* a completed authored Summer demo. The 305/305 Summer occupied-source address recovery is not rule approval; the Grass/Void dual-grid remains 0/16 certified. Other seasons have no blanket automatic approvals. The two original source PNG paths used by the pinned fixture and 318 unused paths are explicitly reported. No unknown source-object boundaries, placement rules, source-black vs transparent pixels, or alpha issues are silently guessed or corrected. `--verify-local` can check all 349 hydrated hashes on the user's Windows working installation.

Windows acceptance: open SRC and filter through each family; choose Objects layer in LYR and place a *manually inspected* plants_summer vegetation cell above grass; choose reeds from the original image and manually place above river; switch to Ground and confirm non-Terrain art cannot replace ground; undo, save, reopen, PIE and check same visual overlay; original asset hash gate still 349/349. Observe any black rectangle carefully: check source alpha before claiming a renderer fix. Reconstructing the original flattened Summer demonstration as a full editable scene, certified cliff corner/shoreline recipes, real object anchoring, animated FX and character rigging are **still open work**.

## Source-only checks

- `python tools/audit_elizawy_asset_consumption.py --check` — exact canonical/inventory family coverage, browser reachability and incomplete scene coverage.
- `python tools/audit_elizawy_asset_consumption.py --check --verify-local` — additionally hashes all 349 hydrated original files when running in `Havenwild Latest`.
- `python tools/rust_source_audit.py` — fails if catalog/layer/PIE integration disconnects.
- `python tools/pie_source_contract_selftest.py` — static PIE/v2 snapshot contract.
- Native Windows `PCC` full gate and UI test required: this authoring environment does **not** contain Cargo/Rustc or hydrated art, so structural Python success is not a native build/visual pass.
