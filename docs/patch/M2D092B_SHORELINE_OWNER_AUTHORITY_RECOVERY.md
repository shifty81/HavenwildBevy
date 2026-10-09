# M2D092B — Shoreline owner-authority recovery

Source: **0.9.6**  
Baseline: **0.9.5 / M2D092A**  
Generated World cache authority: **v6**  
Summer runtime terrain profile: **v8**

## Why this recovery exists

The October 6 visual build still showed incorrect sandy outer-island areas, square/notched shoreline corners and other apparently random coast defects. Several project chats from that work session subsequently became unavailable, so this pass reconstructs the durable state from the user's local 0.9.5 working folder plus the checked-in Native ElizaWy topology evidence rather than relying on chat history.

No 0.9.5 cliff or settlement work is discarded. M2D092B is cumulative over M2D092A.

## Root cause 1 — three extended terrain families were projected backwards

The Native seasonal topology catalog already defines every relevant ElizaWy family using `shared_8_neighbor_owner_side`, including its actual owner and neighbour material. The live profile builder still used an older generic foreground/background ordering for several extended materials. Three families were therefore reversed:

- `summer_sand_over_wet_sand`: source owner is **WetSand**, neighbour **Sand**.
- `summer_sand_bank_pond`: source owner is **Shallow/RiverWater**, neighbour **Sand**.
- `summer_deep_water_basin`: source owner is **DeepWater**, neighbour **Shallow/RiverWater**.

Reversing owner and neighbour also reverses which source tiles mean outer versus inner corners. That directly produces hooks, notches and square-looking breaks around beaches and water bands even though every source rectangle is technically valid artwork.

Runtime profile v8 now consumes the Native owner/neighbour records directly for all eight active pairwise families. Family names are not used to infer visual ownership.

## Exact regression anchors

The new gate pins representative source rectangles from the immutable Summer atlas:

- WetSand/Sand `outer_north_west`: `[288,160,32,32]`
- WetSand/Sand `inner_south_east`: `[288,256,32,32]`
- RiverWater/Sand `outer_north_west`: `[0,640,32,32]`
- RiverWater/Sand `inner_south_east`: `[96,640,32,32]`
- DeepWater/RiverWater `outer_north_west`: `[0,736,32,32]`
- DeepWater/RiverWater `inner_south_east`: `[96,736,32,32]`

The Full Gate now verifies all available outer/inner roles for all eight owner-side transition families, not only these anchors.

## Pale-shore source family

`summer_pale_shore_ring_overlay` is now explicitly certified as the exact source-authored **12-role** family:

- 8 outer roles
- 4 inner roles
- owner: Shallow/RiverWater
- visual neighbour: pale shore

It remains **addressable but non-automatic**. There is no homogeneous pale-shore fill tile proven by this family, so the runtime does not pretend pale shore is another ordinary terrain material or wrap every island in it. Automatic compositing/placement remains screenshot-gated.

## Root cause 2 — blocky island-edge noise

`normalized_island_score` previously used a piecewise `coarse_hash(..., scale=5)` value directly. Every 5x5 world-cell block therefore shared the same edge perturbation. Thresholding that value into shallow water, wet sand and sand could expose visible square blocks along an otherwise curved island.

The edge perturbation now uses deterministic smooth interpolation between the same coarse lattice samples. Generation remains stable for a given seed, but adjacent world cells no longer jump at every 5-cell boundary.

## Cache and lineage

Generated World advances from generator v5 to **v6**. This is intentional: a workstation that already materialized 0.9.5 chunks must not keep displaying the old bad shoreline cache after the source repair is installed.

The following remain intact from 0.9.5:

- class-correct settlement house semantics and buildable-parcel fallback
- persisted 4-bit N/E/S/W elevation-drop cliff mask
- shared intended cliff consumers (Studio/runtime/collision/navigation/waterfalls/caves)
- exact-source/generated-art-off policy
- waterfall and cave automatic placement gates pending continuous cliff face classification

## Acceptance

Source-side acceptance requires:

1. `tools/build_summer_complete_runtime.py --check`
2. M2D080 / M2D081 / M2D082C / M2D082D / M2D082F
3. M2D090 / M2D091
4. `tools/m2d092b_shoreline_owner_authority_selftest.py`
5. cliff source-authority test
6. Rust source audit
7. source-package policy test

The Windows machine remains authoritative for Cargo compilation and rendered visual acceptance. After patch application, run Full Quality Gate and regenerate the same Generated World seed. Inspect outer beaches, wet-sand/sand corners, shallow/deep water bands, narrow channels and island diagonals before promoting automatic pale shore or continuing source-art cliff projection.
