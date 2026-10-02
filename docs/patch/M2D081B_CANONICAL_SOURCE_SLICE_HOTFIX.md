# M2D081B — Canonical Source Slice Hotfix

This 0.8.1 -> 0.8.1 hotfix corrects the remaining Summer composite regression exposed by the Windows cargo test gate.

The M2D081 three-material/checkerboard resolver composes some 32x32 output cells from four exact 16x16 quadrants cut from canonical 32x32 regions in `Terrain/terrain_summer.png`. `AssetAuthority::contains_binding()` intentionally accepts only exact canonical/full bindings, so it must not be used to validate those quadrant slices.

M2D081B adds `contains_source_slice()` as a distinct authority predicate. A source slice is accepted only when it is an exact binding or is wholly contained inside a canonical runtime region on the same immutable source image. The flatworld no-cliff regression now requires every visual part to be a canonical source slice and to remain on `Terrain/terrain_summer.png`.

No terrain recipes, source pixels, generated assets, animation data, world documents, collision, traversal, or editor behavior are changed.
