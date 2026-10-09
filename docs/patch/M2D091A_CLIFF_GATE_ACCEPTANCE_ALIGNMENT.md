# M2D091A — Cliff-gate acceptance alignment

Source: **0.9.3**  
Baseline: **0.9.2 / M2D091**

## Why this patch exists

M2D091 deliberately stopped automatic waterfall and cave-entrance placement until continuous cliff faces are implemented. The Python visual-authority gate already certified that policy, but the older `generated_world_radius_two_exposes_four_seasons_ocean_and_biomes` Rust regression still required both generated object labels. Windows Full Gate therefore reached Cargo successfully and passed 81/82 tests, then failed only because the regression contradicted the active cliff gate.

## Repair

- Preserve all 0.9.2 runtime/worldgen behavior.
- Keep bridge, town-house and biome-tree population mandatory.
- Require generated waterfall/cave objects to remain absent while the cliff gate is active.
- Verify the reviewed `waterfall` and `cave_entrance` templates remain present in source authority so the gate does not accidentally delete the recipes.
- Update the M2D091 Python guard so a future stale Rust expectation cannot silently reintroduce flat-ground cliff features.

## Windows acceptance

Run PCC **Full Quality Gate**. `cargo test --all-targets` should pass all 82 tests. The existing PCC rustfmt normalization step may rewrite formatting before tests; that is expected and is followed by an automatic `cargo fmt -- --check` recheck.
