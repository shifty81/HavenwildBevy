# M2D090A — Seasonal Ground Gate Repair

Source **0.9.1** is a gate-only correction on top of the 0.9.0 ElizaWy kitchen-sink Generated World.

The 0.9.0 runtime intentionally seasonalizes generated ground by substituting coordinate-equivalent `terrain_spring.png`, `terrain_autumn.png`, and `terrain_winter.png` regions for the Summer authority. The older Rust regression `summer_flatworld_cells_never_resolve_from_cliff_sheets` still asserted that *every* generated cell had to use `Terrain/terrain_summer.png`, so it rejected the intended seasonal ground even though the new seasonal-source and kitchen-sink tests passed.

M2D090A changes no world-generation behavior. The regression now verifies the actual contract:

- Spring generated ground uses `Terrain/terrain_spring.png`.
- Summer and open-ocean generated ground use `Terrain/terrain_summer.png`.
- Autumn generated ground uses `Terrain/terrain_autumn.png`.
- Winter generated ground uses `Terrain/terrain_winter.png`.
- Generated ground must never source from a `Terrain/cliff_*` sheet.
- Manually authored semantic Summer terrain continues to resolve from `Terrain/terrain_summer.png`.

This keeps cliff artwork in the object/feature/elevation presentation lane while allowing the exact seasonal terrain substitutions introduced by M2D090.
