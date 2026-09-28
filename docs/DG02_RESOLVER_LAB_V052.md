# DG-02 — Semantic Resolver Lab / Terrain Mapper v0.5.2

## Purpose

This pass makes the dual-grid path diagnosable before the runtime world renderer is allowed to claim completion. The authoring lane now keeps three authorities separate:

1. **semantic cells** — the persisted 5×5 `semantic_lab` document contains only Grass/Void booleans;
2. **topology resolution** — `corner_mask_for_vertex` samples NW/NE/SW/SE and produces the canonical 4-bit mask;
3. **source recipe** — `summer_grass_void.v1.json` maps that mask to one sprite, a composite, unsupported, or unmapped.

The source-address acceptance canvas remains a separate draft/provenance surface. Stamping a source tile never changes its legacy semantic role hint.

## Debug-ready mapper upgrades

- Reads the canonical 349-file source manifest and exposes every hydrated `Terrain/` image. The current manifest contains 29 Terrain images and 320 total images.
- Keeps the four established Summer/Waterfall sheets first for continuity.
- Adds path filtering so the larger Terrain list stays usable in narrow docks.
- Shows a real resolved-source preview for sprite and composite recipes.
- Composite parts are individually locatable, replaceable from the current source selection, offset-editable, and removable.
- `Locate mapped sprite` falls back to composite part 1 when the recipe is composite.
- The semantic resolver lab is transactionally saved and uses no atlas paths or rectangles.
- The checked-in 5×5 seed pattern covers all 16 binary topology masks.

## Certification boundary

`rendererPerformsTopologyResolution` remains `false` in the runtime contract. This pass proves the authoring/resolver path and source preview; it does **not** falsely claim that the Havenwild gameplay renderer has switched to semantic terrain yet.

Run:

```bat
PCC.cmd terrain resolver
PCC.cmd terrain fixtures
PCC.cmd terrain recovery
PCC.cmd full
```

`terrain resolver` must report 16/16 mask coverage. Strict recipe certification remains intentionally red until every mask is explicitly source-certified as sprite/composite/unsupported.

## Tonight's visual checks

1. Open Terrain Mapper and verify the sheet selector exposes the wider Terrain catalog.
2. Filter for `summer`, `water`, `cliff`, etc. and verify selection remains readable when docked left/right/bottom.
3. Assign one mask as a sprite and verify the resolved preview displays the exact selected source rectangle.
4. Convert a diagonal mask to a composite, add multiple parts, edit offsets, locate/replace/remove each part, and verify the preview updates immediately.
5. Toggle cells in the semantic resolver lab, inspect vertices, and verify the mask changes deterministically.
6. Save/reopen both recipe metadata and semantic lab and verify persistence.
7. Run FULL; if Cargo/build fails, use the generated debug bundle, which now includes the semantic lab and its Rust source.

## Immutable seed vs local working copy

`content/terrain/previews/dg01_grass_void.semantic_lab.v1.json` is the gate fixture and stays source-controlled. Studio experimentation is loaded from/saved to `.forgepy/semantic_lab.local.json` when present. A corrupt local file falls back to the checked-in seed and remains available for diagnostics instead of preventing Studio startup.
