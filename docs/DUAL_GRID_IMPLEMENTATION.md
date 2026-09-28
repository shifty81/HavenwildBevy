# Havenwild Bevy dual-grid implementation baseline

Status: DG-01 mapper scaffold implemented; all 16 semantic topology masks are fixture-covered, while visual ElizaWy mappings remain intentionally uncertified.

## Locked separation

Havenwild saves semantic world data, not atlas coordinates. The topology resolver determines terrain shape; the ElizaWy recipe layer chooses source-exact artwork; the renderer draws already-resolved output. Surface terrain, elevation/cliffs, water/hydrology, traversal, and decoration remain separate concerns.

The initial proof is deliberately binary: Grass versus Void, 16 corner states, using NW=1, NE=2, SW=4, SE=8. Multi-material tuples come later. The two diagonal cases (`0110`, `1001`) may require a sprite, a composite, or an explicit unsupported result; they must not be guessed.

## Coordinate and invalidation contract

A visual terrain vertex samples four semantic world cells. Changing semantic cell `(x,y)` invalidates exactly the four surrounding terrain vertices `(x,y)`, `(x+1,y)`, `(x,y+1)`, `(x+1,y+1)` for the DG-00 surface resolver. Chunk halos are read-only context and may not duplicate cell authority.

## Source authority

ElizaWy PNG bytes remain immutable and hydrated locally. Recipe metadata carries stable IDs and provenance (`source path`, `source rect`, later offsets/layer/collision), while saved worlds contain terrain/elevation/water/traversal semantics only.

## DG-00 / DG-01 files

- `content/terrain/dual_grid_contract.v1.json` — coordinate/mask/invalidation contract.
- `content/terrain/recipes/summer_grass_void.v1.json` — mandatory 16-state certification board; initially unmapped.
- `content/terrain/fixtures/dual_grid_reference.v1.json` — checked-in irregular semantic regression pattern.
- `tools/terrain_validate.py` — PCC status, fixture-coverage and certification validator.
- `src/terrain.rs` — pure semantic cell/coordinate/mask foundation.
- `src/terrain_mapper.rs` — source-backed recipe metadata editor state.
- Bevy Atlas window — 16-mask authoring board with sprite/composite/unsupported classification and save/reload.

## Implementation order

1. DG-00 contract/foundation.
2. DG-01 source-atlas mapper with 32px snapping and semantic assignment.
3. DG-02 Grass/Void 16-state certification against hydrated ElizaWy source.
4. DG-03 semantic painting + exact dirty-vertex rebuild + undo/redo/save/reload.
5. DG-04 deterministic regression suite including irregular reference and chunk seams.
6. DG-05 convert the existing 40x28 river fixture to semantic terrain while retaining A/B comparison during migration.
7. DG-06 Grass/Water shoreline topology.
8. DG-07 elevation/cliff compiler beginning at 0/+1.
9. DG-08 multi-level geography/collision/depth.
10. DG-09 traversal and hydrology/waterfalls.
11. DG-10 deterministic decoration.
12. DG-11 PCG semantic inputs.

The first certification target is not “pretty terrain.” It is the ability to reconstruct arbitrary binary terrain shapes from semantic cells using real, manually certified ElizaWy recipes, with identical save/reload and chunk-boundary output.
