# M2D097B — Water scrutiny / ecology / prop-sprite repair (source 0.10.19)

## Purpose

This pass responds directly to the first M2D097A Vulkan visual acceptance screenshots. The visible defects were: shoreline animation reading as fast chase lights, weak distinction between ocean/river/pond behavior, lily pads spawning in ocean shallows, pond-edge dirt patches reading like broken path tiles, and the reviewed barrel rendering only its top half.

## Changes

- Added `WaterContext` (`None`, `Ocean`, `Coast`, `River`, `PondLake`) in `worldgen_plan.rs`.
- Generated cells persist `waterContext`; old saves deserialize with `none`.
- Generated World diagnostics show selected-cell water context.
- Lily-pad/water-detail ecology now consumes WaterContext instead of only `RiverWater`:
  - Ocean/Coast: 0% / prohibited.
  - River: sparse 14% scatter chance.
  - Pond/Lake: 58% scatter chance.
- Pond/lake dirt-bank erosion uses one-third of the normal seasonal bank percentage (minimum 4), while rivers retain the stronger seasonal erosion profile.
- Removed the linear x/y animation phase gradient that caused a chase-light effect around coastlines. All matching terrain animations now share one seed-stable clock.
- Live terrain animation is limited to **mixed water-boundary Wang tiles**. Homogeneous water fills remain static.
- Presentation cadence is intentionally slowed relative to configured Tiled frame durations:
  - coast/ocean boundary: 4x
  - river boundary: 5x
  - pond/lake boundary: 7x
- Original ElizaWy pixels remain renderer art authority; partial/unmapped Tiled clips still fail closed.
- Corrected the reviewed `Objects/Furniture/Barrel.png` template from `[0,0,32,32]` to the full `[0,0,32,64]` first sprite, with visual offset `[0,-32]` so the 1x1 footprint remains foot-anchored.
- Regenerated the ElizaWy asset lane/runtime index after the barrel authority correction.
- Advanced Generated World cache identity from v11 to v12 so stale ecology/object placement is rematerialized.
- Added `tools/m2d097b_water_scrutiny_selftest.py` to both static audit and Full Gate.
- Made older M2D090/M2D091/M2D092B/M2D092C/M2D093A/M2D096A gates accept newer world-generator/cache milestones instead of hard-coding v11/status strings.

## Acceptance boundary

Source-side structural/self-test acceptance can run anywhere. Native Windows Cargo fmt/check/test/build and Vulkan visual acceptance remain required. The next visual inspection should use the same seed and specifically compare: synchronized/slower coast motion, calm pond edges, absence of ocean lily pads, sparse river lily pads, and full-height barrel rendering.
