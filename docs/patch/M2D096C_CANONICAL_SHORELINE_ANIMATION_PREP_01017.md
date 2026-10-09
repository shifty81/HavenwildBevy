# M2D096C — canonical shoreline-animation preparation / source 0.10.17

## Scope

This is a pre-build preparation pass over M2D096A/096B. It does not enable live terrain animation and does not promote legacy `WetSand` or `PebblePath` semantics.

## Added

- Canonical animation clips in `src/tiled_terrain.rs`.
- Every promoted frame must be an exact RGBA match to canonical ElizaWy source pixels.
- Zero-duration or partially canonical clips fail closed.
- Deterministic signed world-space animation phase contract for future shoreline/water presentation.
- Pinned ElizaWy `_ Test Scenes` visual-reference manifest under `content/terrain/evidence/`.
- M2D096C source-only animation contract self-test.
- M2D096C shoreline promotion-readiness classifier driven by the hydrated M2D096B report.
- PCC Terrain option 14 and Full Gate classification phase.

## Explicitly not promoted

- `WetSand -> Deep Sand` remains unproven.
- `PebblePath` remains prohibited as an inferred terrain transition.
- Configured Jaidyn pixels never become runtime art authority.
- Live animation presentation remains OFF pending hydrated evidence and Vulkan visual acceptance.

## Native acceptance still required

1. Windows Full Gate.
2. Confirm canonical clip/partial-clip counts in Studio startup log.
3. Review M2D096B and M2D096C reports.
4. Vulkan visual acceptance before enabling live animated shoreline presentation.
