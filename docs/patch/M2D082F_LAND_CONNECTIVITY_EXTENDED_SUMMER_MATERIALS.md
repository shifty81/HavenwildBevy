# M2D082F — Land connectivity + extended Summer materials

Source **0.8.9** supersedes the rejected 0.8.8 live shoreline projection while retaining the 0.8.7 semantic-cell / dual-grid coordinate fix.

## Live shoreline rule

- Historical 16×16 quadrant composites remain exact-source evidence only; live Generated World terrain no longer renders them.
- A diagonal pair of water cells does not constitute a continuous channel. When two land cells and two water cells only touch diagonally, land connects.
- Cardinally adjacent water remains open, preserving deliberate one-cell rivers/channels.
- Grass/Dirt/Sand/etc. meeting water are reduced locally to one land material against one water material so the renderer can select one complete authored 32×32 source tile. Neighboring dual-grid vertices carry the material hand-off.

## Promoted source-backed Summer materials

The semantic terrain lane now includes:

- Grass — 6 homogeneous variants
- MudBank / Dirt — 6 variants
- Sand — 3 variants
- WetSand — 3 variants
- RiverWater / shallow water — 1 conservative base
- DeepWater — authored center of the deep-water basin family
- PebblePath — 4 variants

Source-authored transition families are promoted where exact role evidence exists: Grass↔Sand, Sand↔WetSand, Sand↔RiverWater, RiverWater↔DeepWater, and the eight outer PebblePath↔MudBank states. No PNG bytes are generated or modified.

## Generated World material profile

The four-season archipelago currently renders from the certified Summer source authority and now uses:

`DeepWater → RiverWater → WetSand → Sand → inland land`

Interior river/lake borders can produce MudBank, and each major island receives a deterministic meandering PebblePath road spine as an interim connectivity surface before the full settlement/POI road-graph pass.

## UI adjustment

Terrain-material abbreviations are removed from the permanent left tool rail. Material selection lives in World Generator / World Authoring, which now exposes all seven semantic materials. The larger workspace-bar/tabbed Editor Shell v2 refactor remains a subsequent pass.
