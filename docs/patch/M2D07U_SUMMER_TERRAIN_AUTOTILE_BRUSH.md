# M2D07U — Summer Terrain Autotile Brush

Source 0.7.5 → 0.7.6.

This pass turns the materialized World canvas into a semantic terrain editor for the first Summer MVP lane.

- PNT paints terrain meaning instead of requiring a manually selected source cell.
- World brushes: Grass, Dirt / river bank (`MudBank`), and River water (`RiverWater`).
- Hold left mouse and drag to paint continuously.
- ERS removes authored terrain semantics and returns cells to generated terrain.
- PIP samples the terrain role under the pointer and also locates its exact source region.
- Edited terrain resolves exact canonical ElizaWy source regions from the existing role/topology authority on demand; source PNG bytes remain immutable.
- Cardinal neighbours are re-resolved automatically, so painted boundaries autotile immediately.
- Regeneration keeps semantic world overrides; direct Source Browser drag/drop remains the advanced exact-source generator-correction lane.
- No cliffs are added in this pass. Cliff certification remains a separate lane.
