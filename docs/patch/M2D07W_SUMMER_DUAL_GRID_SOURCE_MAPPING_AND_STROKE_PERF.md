# M2D07W — Summer dual-grid source mapping and stroke performance

## Goal
Make normal Havenwild World painting reliable for the flat Summer MVP using only exact original ElizaWy source regions from `Terrain/terrain_summer.png`.

## Runtime correction
- GRS / DIR / WTR are semantic brushes.
- A rendered terrain tile is resolved from four semantic corners in NW/NE/SW/SE order.
- The semantic cell at the rendered tile coordinate is the SE corner; this removes the previous one-cell/cardinal ambiguity.
- Mixed terrain uses `cornerRecipes`; missing or ambiguous recipes fail closed to a safe same-family fill.
- Cliff sheets and broad fixture-role fallbacks are excluded.

## Automatic source mapping
`tools/build_summer_flatworld_mapping.py` scans the locally hydrated, SHA-256 verified original `Terrain/terrain_summer.png`.
It uses only canonical 32×32 regions already present in Havenwild's runtime authority and family hints for Grass/Dirt/Water transitions. It classifies the four corners of each eligible exact source cell and emits only confident two-material recipes.

The generated mapping lives at `.forgepy/terrain/summer_flatworld_runtime.local.json`; no source PNG is modified or copied into generated runtime art.

PCC Terrain menu:
- `9` builds/rebuilds the Summer autotile map.
- `10` reports mapping status.

Studio must be restarted after rebuilding the local map so the Asset Authority reloads it.

## Object correctness quarantine
Large generated tree/rock composites are excluded unless a local source-bound certification exists. The same Summer mapping build scans the hash-verified original tree/rock sheets, finds the largest alpha-connected sprite component inside/around each existing template region, excludes disconnected guide/separator fragments, and writes `.forgepy/terrain/worldgen_object_bounds.local.json`.

On the next Studio launch those certified source crops are loaded into Asset Authority and the affected tree/rock templates may re-enter world generation. Existing materialized large objects without certification are quarantined. One-cell exact 32×32 detail templates remain eligible throughout.

## Paint performance
A drag stroke now:
- uses a `BTreeSet` to avoid quadratic duplicate-cell checks;
- interpolates skipped mouse cells so fast drags remain continuous;
- changes semantic overrides without incrementing document revision per cell;
- commits one document revision when the stroke ends;
- uses binary-search insertion/removal for the sorted authored override vector.

This removes the progressive slowdown caused by repeated linear scans, sorting, and revision churn while holding the mouse button over a large area.

## Certification
Cargo-free checks cover source structure, world/asset/collision/PIE contracts, PCC patch validation, and the M2D07W contract. The Windows-local Full Quality Gate remains authoritative for Rust compilation and runtime certification.
