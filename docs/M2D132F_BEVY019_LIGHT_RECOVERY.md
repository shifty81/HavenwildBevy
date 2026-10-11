# M2D132F — Bevy 0.19 directional light compatibility recovery

Debug bundle: Havenwild_Bevy_DebugBundle_20261010-211713-260093.zip

Root cause: cargo test builds examples; Bevy 0.19 DirectionalLight field is `shadow_maps_enabled`, not `shadows_enabled`. Rustfmt and cargo check completed earlier in this gate.

Change: update isolated `examples/hybrid_terrain_proof.rs` light field. No 2D Studio, ElizaWy source, terrain recipes, or game runtime modifications.

Gate testing: run full PCC gate on Windows. The new example has not been compiled in this Linux environment. Upon green gate, launch `cargo run --example hybrid_terrain_proof --release` with Vulkan backend selected.
