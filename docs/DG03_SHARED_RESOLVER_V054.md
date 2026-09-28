# DG-03 shared terrain resolver — v0.5.4

DG-03 removes the remaining UI-only split between semantic terrain editing and recipe resolution.

## Runtime-facing contract

`src/terrain_resolver.rs` consumes the semantic-only `SemanticTerrainLab` plus the current `TerrainMapper` recipe family and produces one `ResolvedTerrainVertex` for every dual-grid vertex. Each resolved record carries the vertex coordinate, 4-bit `CornerMask`, recipe resolution kind, and review/certification state.

The 5×5 checked-in semantic seed therefore resolves to a complete 6×6 / 36-vertex output grid. The same module is intended to be reused by the future world renderer rather than duplicating corner-mask logic in the editor.

## Dirty updates

A changed semantic cell invalidates exactly four surrounding terrain vertices. Studio records and highlights those exact vertices after a cell toggle. Resetting the whole semantic lab conservatively invalidates the whole vertex grid. Semantic edits have a separate 64-step undo/redo history.

## Source safety

Recipe source rectangles are checked against `content/catalog/core_source_manifest.json`. Missing source identities, zero/invalid rectangles, and out-of-bounds sprite/composite references are rejected by both the Rust startup validation and the Cargo-free Python terrain validator.

## Commands

```bat
PCC.cmd terrain resolver
PCC.cmd terrain pipeline
PCC.cmd audit
```

`terrain pipeline` verifies the semantic family matches the recipe family, resolves 36/36 vertices, confirms 16/16 mask coverage, verifies every vertex reaches a recipe state, and checks the four-dirty-vertex contract. Recipe classification/certification remains a separate manual visual-review task.
