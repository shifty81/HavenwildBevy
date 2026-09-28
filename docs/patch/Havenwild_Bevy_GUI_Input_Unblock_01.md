# Havenwild Bevy GUI Input Unblock 01

## Scope
Fix the Studio-wide GUI interaction failure only.

## Root cause addressed
`draw_ui` created `havenwild.viewport.resize.handles` as a foreground `egui::Area` whose minimum size was the entire viewport. That transparent foreground surface was layered above the complete Studio and could participate in hit-testing ahead of the product bar, action bar, mapper, windows, and canvas.

## Change
- Remove the viewport-sized foreground resize-host Area.
- Keep custom window snapping active.
- Temporarily delegate resizing to the window/backend until ForgeGUI_Core supplies edge-only resize hit regions.
- Do not alter terrain mapping, scene data, runtime-root handling, asset authority, or project control behavior.

## Test
1. Build normally.
2. Launch Studio.
3. Verify `Save all`, `Source info`, `Terrain Mapper`, `Fit`, `+`, `-`, checkbox, Undo/Redo all respond.
4. Open Terrain Mapper and verify source picker, mask buttons, drag/drop and dock controls respond.
5. Verify canvas primary click, middle/right pan, wheel zoom and Delete work.
6. Verify Source Info window can be interacted with and closed.
