# M2D092C1 — Cargo-test compile repair

Source checkpoint: **0.9.8**  
Prerequisite: **0.9.7 / M2D092C**

The first Windows Full Gate for M2D092C passed asset verification, source audits, terrain/worldgen/compositor gates, Rustfmt normalization, and `cargo check`, then failed while compiling the Rust test target because the new topology-collision regression test referenced `COLLISION_MASK_SIDE` without importing it into `world_doc.rs`'s test module.

This hotfix:

- imports `crate::scene_v2::COLLISION_MASK_SIDE` in the `world_doc.rs` test module;
- preserves the exact Rustfmt normalization that the Windows Full Gate already applied locally;
- advances source identity to 0.9.8 so PCC can govern the byte-changing repair over 0.9.7;
- strengthens `m2d092c_unified_compositor_selftest.py` to require the test-module constant import.

No terrain, shoreline, bridge, layer, collision-runtime, worldgen, or source-art behavior is intentionally changed in this repair.
