# M2D092C — Unified Generated World compositor, terrain behavior and modular bridge

Source checkpoint: **0.9.7**  
Prerequisite: **0.9.6 / M2D092B**

## Why this pass exists

Windows screenshots exposed several symptoms with one shared architectural cause: Generated World was not fully consuming the existing Scene-v2 layer model. Generated objects were effectively rendered in one broad pass, reusable terrain collision was keyed to raw source rectangles, exact collision overlay drawing was expensive, minor detail had a single hard zoom cutoff, and Wood Bridge A was treated as one fixed 96x64 stamp.

## Composition authority

Generated World now uses the canonical layer order for discrete placements:

1. Ground
2. Terrain Details
3. Water
4. Elevation
5. Structures
6. Objects
7. Foreground

Within a layer, generated objects sort by their ground/depth foot (`anchor Y + footprint height`) with stable X/ID ties. Selection follows the same visual ordering and uses complete source-part visual bounds rather than only logical footprints. This supports correct top-down overlap: a forageable can exist north/behind a tree and be occluded by the tree without making the whole canopy a collision or placement exclusion rectangle.

The Generated World Layers panel now persists real visibility controls for all seven lanes. Ground currently owns the resolved terrain base; discrete generated placements use their declared layer. Further source-certified sub-layering (for example bridge foreground rails) can build on this authority without a second compositor.

## Terrain collision/traversal authority

Generated terrain no longer treats `source image + source rectangle` as a universal behavior identity. Exact source rectangles are visual evidence and may participate in multiple autotile contexts.

Reusable generated-terrain behavior is now keyed by:

- semantic role; and
- cardinal same-role topology mask.

Precedence is:

1. local authored cell override;
2. generated bridge traversal footprint;
3. reusable semantic/topology profile;
4. semantic default.

Legacy source-rectangle profiles remain serialized for compatibility but are inactive for generated-terrain collision/traversal. Source-profile APIs remain available for future discrete source-bound object workflows.

## Collision overlay / overview performance

The collision overlay no longer attempts full 32x32 pixel-mask visualization at all zoom levels:

- below 6 px/cell: collision raster overlay is suppressed;
- 6–16 px/cell: blocked cells render as coarse overlays;
- 16+ px/cell: exact 32x32 masks render with contiguous horizontal blocked pixels merged into runs.

Minor generated detail uses hysteresis: hide below 4.5 px/cell and restore above 6.5 px/cell. Below 6 px/cell the terrain renderer uses already-materialized exact source bindings instead of re-running full dual-grid resolution for every visible vertex each frame. This is a source-backed overview LOD, not a replacement-art path.

This pass intentionally does **not** claim a finished chunk-mesh cache; Windows profiling after this LOD reduction will decide whether a later persistent chunk render cache is still necessary.

## Modular Wood Bridge A

The previously reviewed `[128,0,96,64]` source area from `Structure/Bridges/Wood Bridge A - Rails.png` is represented as three immutable exact slices:

- left: `[128,0,32,64]`
- repeatable middle: `[160,0,32,64]`
- right: `[192,0,32,64]`

Worldgen measures a contiguous road/interior-water crossing, requires valid land on both sides, includes one land approach cell on each side and repeats the middle slice to fit the span. Generated bridge footprint is walkable over the underlying water unless a local authored override takes precedence.

The full upstream rails/no-rails sheets remain available for source review. Exact deck-vs-foreground-rail role classification is **not** guessed in this pass; the modular bridge remains on Structures until that source-role split is certified.

## Remaining visual acceptance blockers

This pass does not declare the persistent large-tree green rectangle closed. The original ElizaWy source remains immutable. The new layer/depth/visual-bounds foundation is in place, but the currently promoted tree render sample must still be checked on Windows against the hydrated original and tightened per object if the artifact persists. Shore/coast screenshot acceptance also remains required.

Continuous cliff source-role projection, waterfall placement and cave placement remain gated behind the established M2D092A topology/evidence rules.
