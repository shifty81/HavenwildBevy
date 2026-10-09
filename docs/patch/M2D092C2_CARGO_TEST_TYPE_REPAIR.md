# M2D092C2 — Cargo-test collision assertion type repair

Source checkpoint: **0.9.9**  
Prerequisite: **0.9.8 / M2D092C1**

The Windows Full Quality Gate passed Cargo check and reached Cargo tests, where one regression assertion compared `CollisionMask32::blocked_count()` (`u32`) with `COLLISION_MASK_SIDE * COLLISION_MASK_SIDE` (`usize`).

This repair:
- casts the expected 32x32 mask area to `u32`;
- preserves the Windows `cargo fmt` normalization of the new terrain-topology editor code;
- strengthens `m2d092c_unified_compositor_selftest.py` to detect the import and type contract statically;
- advances source identity to 0.9.9;
- changes no gameplay, worldgen, source art, layer ordering, collision semantics or bridge behavior.

Windows Full Gate remains the runtime/compiler authority for final acceptance.
