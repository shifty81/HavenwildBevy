# ElizaWy / LPC Revised asset authority — clean reset

## Canonical active art

The original user-supplied packs `Terrain.zip`, `Structure.zip`, `Objects.zip`, `FX.zip`, `Characters.zip` plus `Credits.zip` are the active source family. ForgePY hydrates validated local working copies into `assets/elizawy/{Terrain,Structure,Objects,FX}` with their exact original relative paths and credits. The collection-wide `assets/elizawy/Credits.txt` is copied from the authoritative source when available. `assets/optional/Characters.zip` is a local optional cache when sourced, and can be hydrated on demand into `assets/elizawy/Characters/`; it is nevertheless part of the same canonical collection. This is multi-artist LPC Revised work: individual tiles credit Eliza Wyatt and other contributors. Credits.zip describes OGA-BY 3.0 and local credits break down per component. Preserve those, check attribution on releases, and do not claim Eliza Wyatt personally drew every pixel.

`reference/` may be hydrated locally for original test scene and palette references; it is NOT production scene data. If locally retained, `assets/optional/FourSeasonAlternative.zip` has 1,959 overlapping paths with different bytes, including a DIFFERENT `Terrain/Waterfall.png` and Terrain/Credits.txt; do NOT overlay its files into the canonical tree. Explicitly choose and document any later alternative adoption. It may have supplemental assets and different cell layouts; do not infer coordinate equivalence.

## Source index is not a validator

`content/catalog/core_source_manifest.json` is the tracked canonical non-character hydration contract. ForgePY generates `content/catalog/source_index.local.json` for the 349-file core collection and `content/catalog/characters_index.local.json` from character archive metadata for local navigation/provenance. Normal PCC workflows do not individually hash all hydrated character files. Historical collision counts remain documented; a detailed alternate collision index should be regenerated only when the alternate archive is intentionally inspected. Neither constitutes visual approval, collision/navigation proof, licensing adjudication or an automatic publication gate. There are no carried-over Havenwild source-lock validators, hard-coded ancestral Git requirements or PCC status gates.

## Mapping records and layering

Maintain a canonical source descriptor independent of gameplay meaning: original asset path, byte identity, pixel rect and optional ordered animation frames, assembly pieces, anchor/crop, intended layer. Separate working material/terrain recipe IDs, tile adjacency constraints and placements. A mapping/assembly may span multiple 32px tiles; don't stretch water/cliffs/caves to fake modular pieces. Layer roles: Ground; Transitions; Cliff/Portals; Water/Contacts; Waterfall/FX; Objects; Foreground. Higher structural elevation and lower pond recesses have real height/collision; decorative imagery does not fabricate a path.

Start with `Terrain/terrain_summer.png` 512×832 at 32px tiles. The starting river's Grass, RiverWater and MudBank roles reference the HISTORICAL first-cells from a previous project solely as editable draft placeholders, *not approved source-to-role mapping*. Every other sheet is `source_only` until reviewed. Do not silently copy summer coordinates into spring/autumn/winter without exact structural correspondence checks.

## Correct interaction

Atlas and world share one typed source pointer. Select exact source image and pixel rect -> drag actual source pixels -> snapped ghost -> replace one instance OR explicitly review/update a recipe -> undo -> save mapping document -> reopen -> identical result. World changes must not mutate original PNG. Unsupported source/collision/animation combinations are visible unresolved work items, never inferred as approved. Publish only after manual source-exact visual, placement/collision/navigation and client/editor parity review.
