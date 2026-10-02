# M2D07 A–O — Unified ElizaWy Asset Authority + Source-Backed World Composition

Source target: **0.7.0**  
Patch base: **0.6.0 (M2D06-B)**

## Non-negotiable art rule

Havenwild Bevy does **not** generate image assets. The only art used by this pass comes from the existing hydrated ElizaWy/LPC Revised repository. World generation means generating **composition, semantic layout, and placement records** that reference canonical ElizaWy source regions. Original PNG bytes remain immutable.

## Fifteen cumulative passes

1. **M2D07-A — Canonical source lane**: normalize all 320 runtime PNG records into one ElizaWy authority surface.
2. **M2D07-B — Historical evidence collapse**: carry forward 1,781 historical source-region records without auto-promoting semantic meaning.
3. **M2D07-C — Exact runtime-region index**: normalize 2,183 exact source regions that can be traced to canonical source images.
4. **M2D07-D — Fixture role evidence**: preserve observed Grass, MudBank and RiverWater usage from the existing source-authored Summer fixture.
5. **M2D07-E — Observed topology evidence**: derive cardinal-neighbor role patterns from the existing 40×28 fixture so composition prefers source regions observed in a matching local context; this is evidence, not universal terrain certification.
6. **M2D07-F — Source-exact object templates**: normalize 20 existing Summer object templates (trees, shrubs, flowers, rocks, etc.) without creating new artwork.
7. **M2D07-G — Runtime authority guard**: Rust-side validation refuses art, regions, templates, or world overrides outside the canonical lane.
8. **M2D07-H — Signed chunk document**: add persistent 32×32 signed-coordinate world chunks under a workstation-local world document.
9. **M2D07-I — Deterministic 3×3 materialization**: materialize nine chunks around a chosen center from a seed using only canonical source regions/templates.
10. **M2D07-J — Regeneration preview**: calculate generator-owned cell/object differences before applying regeneration.
11. **M2D07-K — Authored override preservation**: visual, semantic, elevation, collision, and worldgen-placement tombstone corrections survive regeneration.
12. **M2D07-L — Infinite-canvas world mode**: render the materialized 3×3 directly on the existing canvas with signed CH coordinates, pan/zoom, selection, source pick, exact source override and revert-to-generator editing.
13. **M2D07-M — Collision/elevation world editing**: reuse the 32×32 collision masks and red overlay on materialized world cells; authored collision and height corrections persist separately from PNGs.
14. **M2D07-N — Real overlay applications**: enable World Generator, Chunk Manager, and ElizaWy Asset Authority as floating/dock-over-canvas panels launched from HW; docking never resizes the canvas.
15. **M2D07-O — PCC/validation hardening**: add deterministic lane checks, world-contract tests, source-version/asset-lane dashboard reporting, and keep World PIE disabled until chunk runtime/traversal is genuinely certified.

## Current world-composition profile

`havenwild.source_backed.riverlands.v1` is deliberately a **composition proof**, not the final Havenwild terrain compiler. It creates a deterministic Summer river/grass/mud layout, prefers source regions observed under matching fixture-neighbor patterns, and places only existing source-exact Terrain templates.

The following remain explicitly unclaimed:

- universal/complete terrain transition correctness;
- the still-unfinished 16-state Summer Grass/Void recipe (`0/16 certified` remains honest);
- final cliff/waterfall/shore resolution;
- chunk runtime streaming or world PIE;
- procedural structures/settlements;
- automatic canonical collision for every ElizaWy object;
- final seasonal world generation.

## Editor workflow after Windows certification

1. Open **HW → World Generator**.
2. Pick a seed and center chunk; use **Preview** if desired.
3. **Generate 3×3** and open the world canvas.
4. Use the permanent direct-tool rail to select/paint/erase/pick.
5. Use **Collision / Elevation** for red 32×32 pixel masks and height overrides.
6. Delete a selected worldgen placement to create an authored tombstone.
7. Save World (`Ctrl+S`).
8. Regenerate 3×3. Authored corrections remain.
9. Return to **Summer scene** for the existing certified PIE path. World PIE remains disabled in 0.7.0.

## Build/certification status

The source-only structural/Python checks can be run without Cargo. A Windows `PCC.cmd full` is still mandatory before marking 0.7.0 GREEN because this handoff environment does not contain the Rust toolchain or GPU runtime.
