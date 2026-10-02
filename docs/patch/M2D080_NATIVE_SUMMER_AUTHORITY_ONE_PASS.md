# M2D080 — Native Summer Authority One-Pass Convergence

## Scope

This pass replaces incremental/heuristic Summer terrain reconstruction with one reviewed source-driven authority derived from the mature Havenwild Native mapping while preserving the current Bevy/egui Studio shell.

## Source/art policy

- `Terrain/terrain_summer.png` remains immutable and pinned to SHA-256 `1251a6ea556330190ccb1e3af166eb69fbec4b7a3f7d51c1f54728abd75bd752`.
- The recovered Native complete map catalogs all 416 32x32 cells: 305/305 non-transparent cells mapped, 14 structural transparent cells, 97 unused transparent cells.
- The recovered Native topology retains 20 source-authored Summer families.
- No generated atlas is visual runtime authority. Pixel-color classification is diagnostic only.
- Old implementation rules are explicitly reviewed in `native_summer_authority_review.v1.json`; forced mud banks and broad visual fallbacks are removed.

## Flat Summer world paint

The normal GRS/DIR/WTR brush uses one shared four-corner semantic resolver:

- Grass ↔ Dirt: 14/14 mixed states
- Grass ↔ Water: 14/14 mixed states
- Dirt ↔ Water: 14/14 mixed states
- 3 solid fills
- 45/45 total pairwise flat states

36 mixed states are direct exact 32x32 source regions. Six checkerboard states are exact-source 16x16 quadrant compositions taken from canonical parent regions; no artwork is synthesized or altered.

Generated rivers no longer force Dirt/MudBank. Dirt appears only when the semantic world actually contains Dirt/MudBank.

## Native rules audit

KEEP / normalize:
- complete Summer source catalog;
- explicit outer/inner topology geometry;
- source-backed Grass/Dirt, Grass/Water and Dirt/Water families;
- valid sand/path/depth/overlay families as cataloged future materials.

REMOVE / demote:
- generated `lpc_mapped_terrain_v7_32` atlas as visual authority;
- runtime pixel-color guessing;
- forced MudBank around all water;
- cliff/elevation sprites in flat terrain brushes;
- broad fixture fallback into unrelated art.

## User-facing workflow

Normal world authoring is canvas-first. Terrain Rules now summarizes source authority and paint coverage; manual DG/evidence tooling is collapsed under Advanced.

World Properties uses collapsible sections for Traversal, Collision Shape, Elevation and Advanced metadata. Collision/traversal can be authored at either:

- `Reusable profile`: Havenwild metadata keyed to an exact source binding; or
- `This area only`: one-off world-coordinate override.

Neither scope edits source PNGs.

Traversal modes: Auto, Walkable, Blocked, Wadeable, Swimmable.

Collision authoring supports direct canvas painting, darker translucent overlay, 1/2/4/8px brush radius, one transaction per stroke, and cursor-anchored zoom to 16x.

## Object safety

Earlier experimental connected-alpha tree/rock crop metadata is quarantined because it could include guide pixels or clip trunks. Until exact object bounds are explicitly reviewed, broken large procedural crops are omitted rather than rendered incorrectly. Original object PNGs remain unchanged.

## Verification

The pass adds `tools/m2d080_native_summer_authority_selftest.py` and wires it into PCC audit/full-gate lanes. Local Rust compilation remains the final certification on the user's Windows environment.
