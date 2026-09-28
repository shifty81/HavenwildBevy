# ElizaWy Summer Terrain Source Notes

Source inspected: local hydrated `Terrain/terrain_summer.png`.

- Exact sheet size: **512×832 px**.
- Havenwild authoring grid: **32×32 px**, yielding a 16×26 cell address space.
- The sheet is **not treated as a textbook 16-mask autotile strip**. It visibly contains larger assembled motifs and multiple terrain/water treatments across the sheet.
- DG-01 therefore maps semantics to exact source rectangles explicitly rather than inferring that neighboring atlas cells correspond to mask order.
- Single 32×32 cells remain the first mapping unit, but the recipe format and editor support **composite** states with multiple source cells and pixel offsets.
- Larger authored motifs can later become higher-level assembly/stamp recipes without changing the semantic world contract.

No source coordinates in this document are promoted as terrain roles. Visual role assignment remains an explicit mapper/review action.
