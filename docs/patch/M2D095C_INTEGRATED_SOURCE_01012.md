# M2D095C — Integrated 0.10.12 source checkpoint

Source: **0.10.12**
Baseline: exact cumulative M2D093B / 0.10.11 source checkpoint.

## Integrated

- `tiled = 0.16.0` with default features disabled.
- Jaidyn configured-TSX authority bridge (`src/tiled_terrain.rs`).
- Havenwild-native sparse Corner Wang resolver (`src/tiled_terrain/native_wang.rs`).
- Governed Jaidyn hydration and independent TSX verification tools.
- Cargo.lock pinned to `tiled 0.16.0` + `quick-xml 0.31.0` using the existing base64/flate2 graph.
- Existing M2D093B editor convergence, right-rail layer authority, viewport cache, source-bound object fixes, terrain/worldgen work and PCC/ForgePY workflow are retained.

## Removed / not carried forward

- `bevy_map_autotile 0.6.0`.
- The accidental Bevy 0.18 parallel dependency stack.
- Standalone Jaidyn PowerShell/CMD test-kit workflow.

## Authority boundary

Havenwild owns semantic/world state. Tiled/Jaidyn owns configured tile topology evidence. The native Wang lane derives visual topology only; exact signatures are preferred and unknown sparse cases fail closed. Live Paint/worldgen/runtime promotion is a later visual-acceptance pass.
