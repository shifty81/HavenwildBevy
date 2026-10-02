# M2D07R — Worldgen Learning + World Canvas Performance

Source 0.7.2 → 0.7.3.

## Changes

- World Paint teaches deterministic generation mappings by default.
- Shift+Paint remains an explicit coordinate-local override.
- Existing 0.7.2 visual corrections are conservatively promoted into learned rules on first regeneration when their role is compatible.
- Corrections are keyed by terrain role, N/E/S/W topology mask, and originally generated source region.
- Regeneration applies learned rules instead of merely replaying painted coordinates.
- Grass interior variation is reduced to remove salt-and-pepper source selection.
- Trees, rocks, and details use deterministic spaced scatter slots, river clearance, and footprint checks.
- World-canvas wheel zoom is cursor anchored.
- Source UVs use a half-texel inset to reduce atlas-edge bleed on exact source regions.
- Visible terrain and object draws are coalesced into texture-run meshes.
- Object traversal is restricted to visible materialized chunks.
- Minor detail objects and tile-grid work are suppressed at distant zoom levels.
- Sorted chunk and local-override lookups use binary search.
- Regeneration preserves the current camera pan/zoom.

## Authority

Original ElizaWy source artwork remains immutable. This patch changes composition,
mapping authority, and editor rendering only; it does not synthesize replacement art.
