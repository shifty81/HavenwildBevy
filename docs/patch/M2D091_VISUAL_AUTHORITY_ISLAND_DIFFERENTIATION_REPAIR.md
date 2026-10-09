# M2D091 — Visual authority / seasonal-island differentiation repair

## Why this pass exists

The first M2D090 kitchen-sink screenshots proved that content coverage alone was not enough. The four seasonal islands shared essentially the same layout, beaches and roads were visually oversized, interior water was ringed by too much dirt, object anchors made lamp posts read as attached to buildings, bridge/waterfall/cave source usage was not sufficiently contextual, and large seasonal tree crops still exposed the source-sheet guide rectangle.

## Changes

- Generated World generator authority advances to `havenwild.source_backed.generated_world.v4`; prior generated chunks are rebuilt while authored overrides/tombstones remain local authority.
- World profile advances to `havenwild.four_season_archipelago.v3`. Each seasonal island now has its own macro silhouette, water layout, settlement identities, road drift, elevation shape and vegetation density.
- Coast thresholds are compressed to reduce the oversized Sand/WetSand rings. Interior banks default to Grass with deterministic seasonal erosion clusters rather than a continuous MudBank ring.
- Generated roads are one semantic cell wide and use the warm dirt/MudBank family. PebblePath remains available for manual/POI authoring instead of forming giant plazas.
- Bridge placement searches for a real main-road/interior-water crossing before placing the reviewed complete 96x64 arched bridge. Islands without a valid crossing do not receive a fabricated bridge.
- The reviewed waterfall source region changes from `[0,0,96,160]` to `[0,32,96,128]`, intentionally excluding the north/top cap. Automatic waterfall placement is gated until continuous cliff faces can provide a real drop.
- The cave showcase region changes from the mixed partial `[192,288,96,96]` crop to the complete wide `[224,288,160,96]` entrance section. Automatic cave placement is gated until continuous cliff faces can own the entrance.
- Settlement clutter and lamps move away from house footprints.
- Large `Terrain/trees_*` object sampling now insets two full source pixels on all sides while preserving the world-space footprint. This specifically targets the screenshot-confirmed atlas-guide rectangle; visual Windows confirmation is still required before calling it fixed.

## Source authority

No ElizaWy source PNG bytes are edited or generated. Runtime continues to select exact source-backed regions from the pinned asset lane. The crop changes above are reviewed source-region selections, not generated art.

## Certification

`tools/m2d091_visual_authority_repair_selftest.py` locks the v3/v4 world contracts, corrected reviewed feature crops, compact terrain policy, contextual bridge/waterfall placement and two-pixel tree sampling. Windows PCC Full Gate remains the authoritative Rust compile/runtime certification.
