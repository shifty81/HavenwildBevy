# Havenwild Bevy v0.4.1 build repair

This checkpoint responds to the first real Rust compiler result from PCC v1.1.0.

## Observed compiler failure

`cargo check` reached the Havenwild crate and failed with Rust E0506 in `src/terrain_mapper.rs` inside `TerrainMapper::add_composite_part`: the selected recipe state remained mutably borrowed while `self.dirty` was assigned, and `state.parts.len()` was then used after that assignment.

## Repair

- All DG-01 mapper mutation methods now confine the selected-state mutable borrow to a short inner expression.
- Mapper status fields (`dirty`, `message`) are changed only after the recipe-state borrow has ended.
- Added mapper regression tests covering composite editing and state transitions.
- ForgePY v0.4.1 adds `cargo test --all-targets` as a first-class command.
- PCC v1.1.1 adds Cargo tests to the FULL gate between `cargo check` and `cargo build`, and exposes tests as menu/CLI operation.

## Asset boundary

No hydrated ElizaWy assets, character files, local catalogs, or generated PCC/ForgePY state belong in this repair. The patch is source-only.
