# M2D132A — Opt-in Hybrid 3D Geometry Proof

## Scope and safety
- Additive stand-alone `examples/hybrid_terrain_proof.rs`. The existing 2D Studio, renderer, source image authority and PCC remain unchanged.
- Generated deterministic 40×40 tile geometry from a discrete 0–3 elevation function, including exposed cliff faces; simple translucent river patches and directional lighting/shadows. This demonstrates an actual 3D render pipeline, **not** conversion of ElizaWy artwork yet.
- Current image payload is intentionally absent from source-only checkpoint. Source-exact atlas UV integration, tile material references, source region certification, sprites, water flow, hydrology, collision, integrated canvas, and gameplay are **not implemented** by this proof.
- No generated artwork; proof materials are temporary solid colors.

## Windows test (run from repo root)
```powershell
$env:WGPU_BACKEND = 'vulkan'
cargo run --example hybrid_terrain_proof --release
```
Use W/A/S/D to translate proof camera. Close the proof window to exit. Existing `Studio.cmd` remains untouched.

## Revert
- Source root `.pccpatch.zip` installation is governed by existing PCC; retain your v0.10.57 checkpoint.
- If the example does not compile, report the Rust compiler diagnostics; do not treat this as certification.
- Reverting the patch or restoring the unchanged v0.10.57 snapshot returns to the original Studio.

## Next meaningful stages
1. Native Bevy build check on Windows, patch compiler/API issues if any.
2. Consume semantic world height/cell data instead of the proof synthetic function; match exact source atlas UVs.
3. Add image-backed materials and source-faithful 3D sprites only after source sheet region certification.
4. Implement shared camera and editor overlay once 2D and 3D proofs are validated in isolation.
5. Add collision/navigation parity and opt-in profile swap; separately gate gameplay.
