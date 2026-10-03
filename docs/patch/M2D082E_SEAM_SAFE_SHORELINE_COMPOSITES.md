# M2D082E — Seam-safe shoreline composite repair

Source 0.8.8 keeps the M2D082D coordinate authority and changes only seam-sensitive Summer terrain resolution.

- Grass/Dirt/Water three-material vertices resolve to one complete authored 32x32 shoreline tile.
- The exact water-vs-land corner mask is preserved; only the local Grass/Dirt distinction is projected to one land material for that vertex.
- Pairwise diagonal checkerboards collapse to a source-authored one-corner tile rather than splicing four 16x16 quadrants.
- Historical quadrant composites remain in the authority as evidence and tooling data, but are not used by the live resolver for these seam-sensitive cases.
- Direct Tile overrides and the M2D082D half-cell vertex alignment are unchanged.

This targets the small square shoreline artifacts visible in tight mixed-bank islands, coves, and pinches.
