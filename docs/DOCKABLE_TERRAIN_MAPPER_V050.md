# Dockable Terrain Mapper v0.5

The Terrain Mapper is now a persistent editor surface rather than a fixed floating utility.

Supported placements:

- Left dock;
- Right dock (default, 440 px preferred width);
- Bottom dock (horizontal two-column presentation);
- Floating.

The panel uses a responsive presentation:

- side docks stack Source → Recovery Evidence → DG Recipe vertically;
- bottom/wide layouts split source atlas and recipe controls into two columns;
- source atlas has internal scrolling, 32 px grid overlay, and 50/75/100/150% zoom controls;
- panel width/height is resizable with minimum readable dimensions;
- panel docking/visibility/lock state is stored locally in `.forgepy/editor_layout.local.json`.

The permanent Game Canvas remains the center authority. Docked mapper panels are registered before the central panel so they reserve workspace rather than covering the canvas.

## Summer evidence in the mapper

The mapper header reports two different progress measures:

1. recovered Summer source-map coverage (305/305 non-transparent cells);
2. current DG Grass/Void recipe certification (0–16 states).

Selecting a Summer atlas cell shows its recovered source evidence, alpha-pixel count, and a short per-cell RGBA fingerprint.

## Current canvas correction controls

The acceptance canvas now supports:

- select/stamp as before;
- `Delete` to erase the selected draft tile;
- `Erase selected` toolbar action;
- undo/redo for erase and placement;
- saving empty/void draft cells.

This is still a source-address acceptance canvas. Semantic Paint/Erase + Save & Resolve belongs to the later live dual-grid canvas pass and must not be falsely claimed here.
