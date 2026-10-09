# M2D095B - Jaidyn Tiled / Native Wang Project Collapse

Source remains **0.10.11**. This is a governed hotfix/convergence pass, not a second project launcher.

## Purpose

M2D095A proved that `bevy_map_autotile 0.6.0` can consume the Jaidyn/Tiled topology, but it also pulled a full Bevy 0.18 dependency family into the Bevy 0.19 Havenwild project. M2D095B removes that dependency and keeps the useful algorithmic idea inside Havenwild.

## Authority after this pass

- Havenwild semantic/world state remains authoritative.
- `tiled 0.16` remains the parser for the hydrated Jaidyn TSX files.
- Jaidyn Wang assignments remain authored visual-topology evidence.
- `src/tiled_terrain/native_wang.rs` is the project-owned sparse Corner resolver.
- The resolver never mutates semantic input. It derives a visual vertex grid.
- Exact authored Wang signatures are kept exact.
- The only automatic repair permitted is an unassigned two-material diagonal checkerboard.
- Any other missing Wang topology fails closed.
- Fences remain outside this promotion because their configured variants use probability weighting.
- Autumn remains partial/fail-closed.
- Jaidyn collision metadata remains provisional evidence.

## Dependency correction

Removed:

```toml
bevy_map_autotile = "=0.6.0"
```

Retained:

```toml
tiled = { version = "=0.16.0", default-features = false }
```

The normal Havenwild Full Quality Gate must recalculate `Cargo.lock`. After that gate, the Bevy 0.18 dependency family introduced by M2D095A should no longer be reachable from Havenwild.

## Certification

Normal `cargo test --all-targets` now exercises:

1. the synthetic 14/16 Corner grammar;
2. deterministic checkerboard repair;
3. fail-closed unknown sparse topology;
4. the real hydrated Summer TSX when the locally governed Jaidyn hydration is present;
5. all six core terrain pairs previously identified by M2D094B.

`tools/m2d095b_native_wang_selftest.py` additionally checks the dependency/source contract and runs a deterministic mirror stress suite without Bevy.

## Not yet promoted live

This pass does **not** replace worldgen/Paint/runtime rendering. It removes the duplicate/external architecture and establishes a clean project-native authority. The next live integration should be made only from this green baseline so Paint, worldgen and runtime can be switched to the same resolver together.
