# Havenwild Bevy v0.5.4 Build / Debug Session

Source v0.5.4 / ForgePY 0.4.5 / PCC 1.2.4 adds the DG-03 shared terrain resolver while preserving the v0.5.3 governed-update and static-audit hardening.

## 1. Apply and preflight

Place the cumulative `.pccpatch.zip` in the project root or `updates/inbox` without extracting it.

```bat
PCC.cmd updates apply
PCC.cmd audit
```

Expected static results include 349/349 core assets, 1120/1120 valid Summer scene placements, 305/305 Summer source recovery, 16/16 fixture masks, 16/16 semantic masks, and a DG resolved pipeline of 36/36 vertices. The recipe may still correctly report 0 classified/certified mappings.

## 2. Full Windows certification

```bat
PCC.cmd full
```

FULL requires Rust 1.95+, Cargo, rustfmt, Cargo check/tests/build, and a generated `Cargo.lock`. Do not interpret an intentionally uncertified terrain recipe as a build failure.

## 3. Studio DG-03 acceptance

```bat
PCC.cmd run dx12
```

Open Terrain Mapper and expand **Live semantic resolver lab**.

- Confirm the 5×5 semantic grid and 6×6 resolved vertex grid are visible.
- Toggle one semantic cell and verify exactly four resolved vertices receive the `*` dirty marker.
- Use **Undo semantic** and **Redo semantic** and confirm the semantic pattern and resolved masks move together.
- Click a resolved vertex and confirm it selects the same mask in the recipe board.
- Confirm the selected vertex still shows the real-source resolved preview surface.
- Exercise recipe sprite/composite assignment, review-state demotion, recipe undo/redo, Save all, relaunch, and persistence.
- Confirm an edge source selection never produces an out-of-bounds 32×32 recipe source.

## 4. Renderer diagnostics

If normal DX12 launch fails:

```bat
PCC.cmd run dx12 --verbose-gpu
PCC.cmd run vulkan
PCC.cmd run vulkan --verbose-gpu
```

Keep the newest `.pcc/debug/Havenwild_Bevy_DebugBundle_*.zip`. v0.5.4 bundles include `src/terrain_resolver.rs` in addition to the prior semantic/recipe/session evidence.
