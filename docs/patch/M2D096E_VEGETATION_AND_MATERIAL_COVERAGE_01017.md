# M2D096E — vegetation normalization + complete material accounting / source 0.10.17

This source-only prebuild checkpoint accumulates M2D096C and M2D096D while the Windows build machine is unavailable.

## Terrain authority

- M2D096A live Tiled/canonical-pixel core renderer remains intact.
- M2D096C prepares canonical animation clips only when every frame maps to exact original ElizaWy pixels; presentation remains OFF.
- M2D096D explicitly accounts for all 12 Summer Terrain Wang materials. Unknown material semantics fail closed.
- `WetSand -> Deep Sand` remains blocked pending hydrated animation/composition evidence.
- `PebblePath` remains a manual/POI legacy material and is not promoted into Tiled core terrain.

## Vegetation correction

- Seven Summer wildflower fixture placements moved from `terrain_details` to `objects`.
- Generated wildflowers route to `objects`; water-edge planting remains `terrain_details`.
- Wildflower runtime template now publishes layer `objects` and remains worldgen eligible.
- Runtime source images retain immutable filesystem-family provenance but also expose `semanticCategory`.
- Flower/wildflower source images classify as `Vegetation` for Source Browser filtering.
- Source Browser now includes a `Vegetation` semantic filter.

## Native acceptance pending

Windows Full Gate, Cargo fmt/check/test/build, hydration evidence reports, and Vulkan visual acceptance remain required before this checkpoint is called native GREEN.
