# M2D093A — Flat terrain authority convergence / Terrain Studio foundation

Source transition: **0.10.0 -> 0.10.1**. ForgePY remains **0.4.10** and PCC remains **1.3.5**.

## Runtime / worldgen

- Generated World advances to **v9** so cached v8 terrain/object populations rebuild.
- Ocean coast returns to `DeepWater -> RiverWater -> WetSand -> Sand -> Grass` without inventing a direct RiverWater/WetSand atlas family.
- `RiverWater <-> WetSand` uses the exact certified **12-role pale-shore** source family as a visual bridge; `WetSand <-> Sand` uses its separate exact owner-side family.
- Spring, Summer and Autumn primary river corridors now reach the irregular island edge and open into shallow ocean. Coastal river shoulders inherit WetSand/Sand rather than inland grass/mud bank materials.
- Purpose objects reserve their footprints before ecology placement, preventing natural placements from blindly overlapping houses, bridges and other existing generated objects.

## Source-object render bounds

- Full Gate now derives local v2 render bounds from manifest-verified immutable ElizaWy alpha for **7 seasonal tree templates + 2 Grasslands rock/stone templates**.
- Trees use a Spring/Summer/Autumn/Winter union-alpha crop so seasonal substitution keeps one destination anchor.
- Rocks/stones use exact single-source alpha bounds from `Terrain/Rocks, Grasslands.png`.
- Atlas/source slots remain provenance and placement identities. Render crop, logical footprint, depth, collision and interaction remain distinct contracts.
- The old tree-specific two-pixel UV inset workaround is removed. Original PNGs are never edited.

## Terrain Studio / PCC UX

- Terrain Studio is promoted as the primary terrain-authority application surface.
- Production Pattern Matrix exposes the live Summer fills, owner-side transition families and shoreline visual bridge.
- The old 16-mask Grass/Void recipe remains a legacy diagnostic/test lab only.
- The PCC dashboard now reports production terrain authority instead of presenting the legacy `0/16` counter as the primary state.
- Tool rail uses normalized icon drawing rather than three-letter direct tool glyphs.

## Evidence policy

- Water-detail `RepeatableFill` cells remain static/evidence-gated. No unverified water animation is promoted.
- JaidynReiman/Tiled and Havenwild Native mappings remain evidence/reference lanes; original pinned ElizaWy PNGs remain pixel/source authority and Havenwild certified recipes remain runtime authority.

## Acceptance

On Windows, run Full Gate, regenerate the same seed, then verify:

1. Water/WetSand/Sand/Grass coast corners have no square fallback cells.
2. Spring/Summer/Autumn intended rivers open into the ocean and mouth shoulders blend through the beach materials.
3. Natural vegetation/rocks no longer spawn over generated structures/bridges.
4. Large trees and promoted rocks/stones no longer show rectangular/halo atlas-slot artifacts.
5. Terrain Studio opens as the primary terrain app and shows production pattern families; legacy 16-mask diagnostics are clearly secondary.
6. Layer visibility and existing compositor behavior remain intact.
