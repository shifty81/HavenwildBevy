# M2D092C3 — Coast / layer / vegetation runtime recovery

Source checkpoint: **0.10.0**  
Prerequisite: **0.9.9 / M2D092C2**

This pass responds directly to the October 6 Generated World screenshots and runtime observations.

## Coastline

The mandatory generated `RiverWater -> WetSand` edge was unsupported by the certified Summer transition graph. The live resolver therefore failed closed to complete 32x32 fill tiles, producing the square orange shoreline blocks around every island. Automatic coast now uses the exact supported chain `DeepWater -> RiverWater -> Sand -> Grass`. WetSand remains available for manual/detail authoring and later certified overlays.

## Inland banks

Seasonal dirt erosion now uses smoothly interpolated clustered noise and requires neighbouring bank support, preventing isolated one-cell MudBank hooks around ponds/rivers.

## Layers

Generated semantic terrain is partitioned into Ground / Terrain Details / Water at render time. Generated water details route to Water, water-edge plants and wildflowers to Terrain Details, normal vegetation to Objects, and structures remain Structures. The Layers panel counts both semantic terrain and discrete placements.

## Stable vegetation visibility

Object collection queries an eight-cell neighbour margin before exact visual-bounds culling. This prevents sprites that overhang from a neighbouring chunk from disappearing during pan. Minor-detail overview LOD is restricted to the absolute far zoom.

## Cache / provenance

Generated World advances to v8 so stale v7 chunks are rebuilt. No ElizaWy source PNG is modified or generated. Continuous cliffs, waterfall/cave placement, tree render-bound certification and bridge rail/deck splitting remain separately gated.
