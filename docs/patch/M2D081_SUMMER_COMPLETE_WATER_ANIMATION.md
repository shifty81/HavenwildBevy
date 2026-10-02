# M2D081 — Summer Complete + Source-Backed Water Animation Candidate

## Baseline

- From Havenwild Bevy source `0.8.0` + M2D080A compile repair
- To source `0.8.1`
- ForgePY remains `0.4.6`
- PCC remains `1.2.6`
- Bevy remains `0.19.0`

## Purpose

Finish the flat Summer authority before enabling cliffs. This pass is built from the certified
0.8.0 source-only checkpoint and the checked-in Native/LPC Summer recovery metadata. It does
not modify or generate replacement PNG artwork.

## Runtime authority changes

- Promotes all 305 non-transparent `Terrain/terrain_summer.png` cells from the Native complete
  map into canonical runtime-selectable authority.
- Preserves 41 recovered Summer source groups and 20 mature topology families.
- Runtime exact-region count becomes 2316 (previously 2203).
- Builds a complete Grass / Dirt / Water four-corner grammar:
  - 3 homogeneous states
  - 42 two-material mixed states (14/14 for each material pair)
  - 36 three-material Grass/Dirt/Water junction states
  - 81/81 total states
- Three-material junctions are four exact 16x16 quadrants cut from canonical 32x32 source
  regions. They are compositions of original source pixels, never generated replacement art.

## Visual regressions addressed

### Isolated dirt / island bridging

The six diagonal checkerboard cases no longer retain the old ambiguous fixed composite. The
runtime profile uses exact one-corner source quadrants with a defined background material so
isolated detail/land spots do not visually bridge through each other:

- Grass + Dirt -> Grass background; Dirt corners remain isolated.
- Grass + Water -> Water background; Grass island corners remain isolated.
- Dirt + Water -> Water background; Dirt-bank corners remain isolated.

This is intentionally conservative for the flat Summer lane. If later hydrology needs an
alternate diagonal ownership mode, it should be an explicit semantic topology, not a guessed
visual fallback.

### Grass / Dirt / Water interior variation

Homogeneous terrain no longer has to repeat one fill forever. Exact source-backed variants are
selected deterministically by world seed and cell coordinate:

- Grass: 6 Summer fill variants; the plain fill is weighted more heavily.
- Dirt: 6 Summer fill variants; the plain fill is weighted more heavily.
- Water: 8 Summer fill variants.

The same seed and coordinate always resolve the same non-animated fill variant.

### Water animation candidate

The eight `summer_water_fill` source cells are exposed as a low-frequency homogeneous-water
runtime cycle at 220 ms per phase. This is deliberately marked
`runtime_candidate_requires_user_visual_review` because the recovered donor classifies them as
source variants rather than proving an upstream animation timing contract. Shoreline topology
never cycles: only fully homogeneous RiverWater interiors use the time-aware path.

If the visual review shows that these eight cells are spatial variants rather than animation
phases, the animation metadata can be disabled without changing the 305-cell Summer authority.
The explicit Mountain/Animated-Water and Waterfall animation families remain reserved for the
cliff/waterfall stages.

## Placement / anchoring

Flat terrain continues to render source rectangles into the exact selected 32x32 destination
cell. No object bottom-anchor rule is applied to terrain. If a selected exact source cell has
transparent pixels on its top/left edge, the opaque art may visually occupy only part of the
selection square; M2D081 does not shift or rewrite those source pixels.

## Editor UX

Terrain Rules now reports:

- 305/305 Summer runtime cells
- 41 recovered source groups
- 81/81 Grass/Dirt/Water states
- 14/14 pairwise coverage for all three pairs
- Grass/Dirt/Water interior variant counts
- water source-backed phase count and frame duration

The normal semantic World brush remains Grass / Dirt / Water. The remaining Summer groups are
available as exact source authority/manual source selections but are not silently assigned new
semantic gameplay meaning.

## Deliberately deferred

M2D081 does **not** enable cliffs or waterfall assembly. Next stages are:

1. M2D090 — complete Summer cliffs/elevation as structural boundaries while preserving surface
   terrain.
2. M2D100 — complete waterfalls, including source/outlet water connections, directional/body
   animation groups, top/lip/auxiliary pieces, and verification of the candidate north-facing
   top-atlas piece against the original ElizaWy demonstration.

## Gate requirements

The Windows PCC Full Quality Gate remains the compile/runtime authority. New source checks cover:

- deterministic asset-lane rebuild
- deterministic Summer v4 profile rebuild
- 305 cells / 41 groups
- 2316 runtime regions
- 81/81 G/D/W grammar
- 36 three-material junctions
- deterministic fill variation
- 8 source-backed water phases
- disconnected checkerboard policy
- source mutation OFF / generated replacement art OFF
