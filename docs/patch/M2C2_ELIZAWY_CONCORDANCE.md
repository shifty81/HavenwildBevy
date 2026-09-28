# Havenwild Bevy — M2C2 ElizaWy Concordance & Tiled Evidence Intake

**2026-09-27 · read-only, additive PCC patch · V7 quarantined**

This patch adds reproducible checks for the recovered, index-backed **original ElizaWy** evidence. It does not add a new terrain renderer, modify recipes, download art, import V7 tuple signatures, certify old mappings, or change GUI input/window behavior.

## Included tools

- `tools/audit_elizawy_concordance.py`: finds the five required ElizaWy evidence files through `.forgepy/recovered_mapping/index.json`, normalizes Windows paths, checks each recovered SHA, verifies Summer's 416 unique positions, 305 occupied/111 transparent cells, 41 groups, 20 topology families and five-season role coordinates, B48R7/B48R9 crosswalk flags and canonical bounds, provenance source lock and mismatch, plus SHA/IHDR dimensions for 13 optional hydrated Terrain source assets listed in `content/catalog/core_source_manifest.json`. Writes `.forgepy/recovered_mapping/elizawy_concordance.v1.json`. `evidenceReady` is distinct from `pixelSourceVerified`. **Missing art does not falsely claim pixel certification.**
- `tools/inspect_elizawy_tiled.py`: inspects user-supplied Tiled derivative ZIP or directory **without extraction**; handles `.tsx` and `.tsj`, plus `.tmx`/`.tmj` map `firstgid` references, Wang/legacy terrain sets, tile-local IDs, image/grid parameters, animations, collision presence and transformations. Rejects unsafe ZIP paths, ZIP overlarge members and XML DTD/entities. Report is evidence-only; no position transfer or automatic matching.
- `tools/elizawy_m2c2_selftest.py`: synthetic no-network inspection and archive-safety tests.

## Once the root-drop patch is applied

From the Havenwild Bevy project root in PowerShell:

```powershell
python tools\elizawy_m2c2_selftest.py
python tools\audit_elizawy_concordance.py
```

The latter uses the current **already recovered** `.forgepy/recovered_mapping/raw` files and does not overwrite them. If necessary, rerun your existing `PCC.cmd terrain recover --source "C:\path\to\historical\Havenwild"` to populate the index/raw copies.

Optional image source proof uses the **existing** governed ForgePY hydration contract:

```powershell
ForgePY.cmd assets sync --source "D:\path\to\authoritative\ElizaWy\LPC"
python tools\audit_elizawy_concordance.py
```

The root passed to `assets sync` should follow ForgePY's supported source-tree layout; consult `assets/README.md`. The report checks files at `assets/elizawy/Terrain/...` by default; alternatively run `python tools\audit_elizawy_concordance.py --assets-root "D:\your\raw\LPC"`. Do not use an unrelated repack as though it were the original version.

For the **separately downloaded** JaidynReiman Tiled archive (not part of this patch):

```powershell
python tools\inspect_elizawy_tiled.py "D:\Downloads\lpc-revised-exterior-tilesets.zip" --output ".forgepy\recovered_mapping\tiled_derivative_inspection.v1.json"
```

Original fixed source revision: `https://github.com/ElizaWy/LPC/tree/f07f7f5892e67c932c68f70bb04472f2c64e46bc`; third-party derivative listing: `https://opengameart.org/content/lpc-revised-fully-configured-4-seasons-tilesets-for-tiled-map-editor`.

## Actual test against recovered uploaded metadata in clean staged source + M2AB

- Concordance: evidence **5/5**, role coordinates **0 mismatches**, Summer **416/416** unique coordinates and **305** occupied cells, **41** primary groups, topology **20 families × 5 seasons**, **1,280** role references; B48R7 **290** entries, B48R9 **1,781** entries from **40** external reference sheets; no canonical target positions outside 16×26.
- Source PNGs available in staged source-only test: **0/13**; therefore **pixelSourceVerified=false**. These files must be hydrated locally to complete the source-art pass.
- Historical topology JSON SHA `2b448c1a...` differs from its provenance record's `f099edfb...`: unresolved review flag, **not silently repaired**.
- Synthetic TSX/TSJ, Wang, animation, collision, unsafe archive/DTD checks: PASS. Actual Tiled derivative ZIP is **not in the supplied files** and therefore its contents have not been certified.
- Windows Bevy build, gameplay/runtime, live Mapping Studio, art seam tests: **not run** in this patch.

## Safety/authority gates before future promotion

1. Hydrate and hash-check the exact pinned original source art, including seasonal terrain/cliffs and waterfall variants, keeping licenses and individual credits.
2. Review source/derived-image pixel hashes, ambiguous matches, alpha/transparent compositing, seasonal variation and animation offsets.
3. Resolve the historical topology provenance hash mismatch without modifying original evidence.
4. Keep `MountainRock` floor presentation separate from structural `Rock_Dark` cliff collision. Preserve true +1 cliffs and levels 0–30; do not restore the old two-level-minimum rule.
5. Promote only manually approved ElizaWy family/role mappings through new derived registry and runtime validation. V7 remains independent donor data.
