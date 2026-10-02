# M2D07V — Summer Flatworld Strict Mapping

Source 0.7.6 -> 0.7.7.

## Why

The first semantic GRS / DIR / WTR brush resolved through broad historical fixture palettes. Those palettes intentionally carried evidence from many Terrain sources, including cliff fragments and partial cells, so the normal World brush could paint visually unrelated pieces even though the selected semantic material was correct.

## Changes

- Add `content/terrain/recipes/summer_flatworld_runtime.v1.json` as a small, explicit Summer-only runtime profile.
- Safe exact-source fills are pinned to `Terrain/terrain_summer.png` for Grass, Dirt/Bank and RiverWater.
- Transition candidates are constrained to recovered Summer source-family membership: `grass_dirt`, `grass_shallows`, and `dirt_shallows`.
- Cliff sheets and broad fixture-role fallback are forbidden in the normal flatworld brush path.
- Missing transition masks fail closed to the appropriate safe fill instead of guessing.
- World generation and semantic World painting use the same strict Summer profile.
- Terrain Rules opens with a simplified **Summer Flatworld Mapper** summary. The old Grass/Void DG recipe editor remains available under a collapsed Advanced section rather than dominating the normal workflow.

## Scope

This is the playable Summer-flatworld lane. It does not claim all 15 boundary states are visually certified yet. The current recovered source evidence safely resolves a subset of transition masks; unresolved masks remain visually plain until reviewed instead of emitting unrelated art.

Cliffs remain a separate later pass.
