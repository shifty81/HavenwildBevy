# M2D096B — Shoreline animation evidence audit

Status: evidence instrumentation only. No terrain role, source coordinate, animation, collision, or runtime art is promoted by this pass.

## Why this exists

M2D096A moved the live core terrain renderer onto the configured Jaidyn/Tiled Wang authority while preserving exact original ElizaWy pixels as art authority. `WetSand` and `PebblePath` intentionally remain fail-closed because their old Havenwild names were inferred historically and are not yet proven to equal Jaidyn's configured material names.

The Summer reference screenshot shows a multi-band coast (deep water, depth bands, pale shallow/surf band, darker shoreline sand, light sand, grass). Visual inspection alone cannot prove whether the darker sand is a static `Deep Sand` material, part of a surf animation, or both.

## External evidence checked

The ElizaWy/LPC repository `_ Test Scenes` directory contains:

- `DemoGame - 1 - Spring.png`
- `DemoGame - 2 - Summer.png`
- `DemoGame - 3 - Autumn.png`
- `DemoGame - 4 - Winter.png`
- `DemoGame - Seasons.gif`
- `Test Landscape.png`

These are visual references, not a runnable map/demo project.

ElizaWy's `Terrain/Credits.txt` describes the all-season Terrain set as original grass, dirt, and shallows by Lanea Zimmerman, edited, animated, and recolored by Eliza Wyatt. JaidynReiman's configured Tiled release states that the tileset is fully animated and that all source assets with animations were configured as animations.

Therefore the TSX animation declarations are the strongest machine-readable authority for deciding which resolved shoreline tiles animate.

## Project instrumentation

`tools/audit_jaidyn_shoreline_animation.py` reads the hydrated Summer TSX and reports, without mutation:

- the configured `Terrain` Wang material IDs;
- every Wang assignment involving `Sand`, `Deep Sand`, `Shallow Water`, or `Deep Water`;
- which of those assignment tiles own `<animation>` declarations;
- exact animation frame tile IDs and durations;
- pure-fill vs mixed-material transition ownership;
- partner-material counts for animated transition tiles.

The report is written to:

`artifacts/terrain-certification/M2D096B_JaidynShorelineAnimation.json`

The interpretation section explicitly distinguishes these facts:

1. `Deep Sand` is a separate configured Terrain material when present.
2. A `Deep Sand` Wang assignment may or may not itself be animated.
3. Animated shallow-water/surf assignments may visually advance over static darker sand.
4. None of those facts alone prove Havenwild's legacy `WetSand` semantic equals Jaidyn `Deep Sand`.

## PCC integration

PCC -> Terrain / dual-grid / mapping authority -> `13. Jaidyn shoreline / animation evidence audit`

Full Gate also runs the evidence audit after the pinned Jaidyn package has been verified/hydrated. A malformed or unreadable TSX fails certification; the observed animation classification does not fail or promote runtime behavior.

## Promotion rule for the next runtime pass

Do not promote `WetSand -> Deep Sand` from appearance alone.

Promotion requires a consistent chain of evidence:

1. configured Jaidyn Wang identity;
2. exact original-pixel concordance back to ElizaWy art;
3. ElizaWy seasonal/test-scene composition consistent with the role;
4. animation frames/durations retained exactly when authored;
5. no conflict with existing Havenwild semantic/world-state meaning.

Only then should the renderer replace the legacy shoreline bridge and enable authored shoreline animation.
