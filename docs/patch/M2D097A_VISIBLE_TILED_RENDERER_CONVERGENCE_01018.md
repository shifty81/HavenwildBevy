# M2D097A — Visible Tiled/native-Wang renderer convergence (0.10.18)

## Purpose

M2D097A is the first pass whose acceptance target is a visibly different terrain renderer rather than only a stronger mapping/certification authority. It removes the remaining split between near/far terrain rendering and retires legacy shoreline composition that could repaint an already-resolved Tiled boundary.

## Live renderer changes

- All terrain zoom levels now render the same cached dual-grid vertex geometry.
- The old far/Fit World path that drew `GeneratedCell.source` in semantic-cell top-left space is retired.
- Materialized terrain attempts the exact configured Jaidyn Tiled Corner-Wang topology before Havenwild's legacy land-connectivity projection.
- A single origin-stable native-Wang solve is cached for the materialized world so neighboring tiles share the same repaired visual vertices.
- If a shared solve cannot consume an evidence-gated/manual role, rendering fails closed to the per-vertex compatibility resolver instead of inventing art.
- Configured tiles are rendered only through exact canonical ElizaWy pixel bindings; the derivative Jaidyn atlas is never runtime art authority.

## Shoreline changes

- Generated coast is now `DeepWater -> RiverWater/Shallow Water -> Sand -> Grass`.
- Generated `WetSand` tidal rings are retired until hydrated evidence proves the relationship between Havenwild `WetSand`, configured `Deep Sand`, and authored shore animation.
- Generated `PebblePath` remains retired as a default road/shore material.
- The historical `summer_pale_shore_ring_overlay` remains cataloged and inspectable but is no longer applied as a second live `RiverWater <-> WetSand` compositor pass after Tiled resolution.
- Historical owner-side source families remain preserved as evidence/fallback data; they are not deleted.

## Animation presentation

- Complete configured TSX animation clips play live only when every authored frame maps exactly to canonical original ElizaWy pixels and all durations are valid.
- Partial/unmapped clips fail closed to a canonical still visual.
- Authored frame durations remain timing authority.
- A deterministic signed world-coordinate phase offset prevents long coastlines from flashing in lockstep.
- Animation advances only cached source-frame selection; topology is not recomputed per frame.

## Cache/version contract

- Source version: 0.10.18.
- Generated World cache generation: v11.
- Vulkan remains the recommended normal resizable Studio backend.
- `WetSand` and `PebblePath` remain editable only under the Legacy/evidence-gated Paint section.

## Regression policy

M2D097A adds `tools/m2d097a_visible_renderer_selftest.py` to both PCC static audit and Full Gate. The gate explicitly rejects:

- generated WetSand/PebblePath worldgen returns,
- the old live `summer_shoreline_overlay_for_pair` repaint,
- a far terrain `px < 6` renderer split,
- direct `GeneratedCell.source` terrain drawing in the terrain mesh,
- direct Jaidyn derivative-atlas runtime art,
- animation presentation that bypasses complete canonical frame proof.

Older milestone tests were updated only where they had frozen superseded renderer behavior. Their source/provenance/topology guarantees remain required.

## Source-side certification completed

Without native Windows build access, the following source-side checks were run successfully after convergence:

- M2D07 / M2D080 / M2D081 / M2D082 family,
- M2D090 / M2D091 / M2D092B / M2D092C,
- M2D093A / M2D093B,
- M2D094B,
- M2D095D,
- M2D096A / M2D096C / M2D096D / M2D096E,
- M2D097A visible renderer convergence,
- native-Wang deterministic stress run,
- Rust source structural audit,
- Python source compilation,
- source-package boundary policy.

## Native acceptance still required

When Windows build access returns:

1. Run `PCC.cmd` -> Full Quality Gate.
2. Require Cargo fmt/check/tests/build and lock verification to pass.
3. Run Studio option 1 (Vulkan).
4. Require title `v0.10.18`.
5. Regenerate the same comparison seed/world.
6. Compare coast at near zoom and Fit World: topology must not jump or shift between LODs.
7. Inspect Grass/Sand, Sand/Shallow, Shallow/Deep, river mouths, dark-water fills, and any configured animated shoreline/water tiles.
8. Inspect M2D096B/M2D096C hydrated reports before promoting `WetSand -> Deep Sand` or any additional material alias.
