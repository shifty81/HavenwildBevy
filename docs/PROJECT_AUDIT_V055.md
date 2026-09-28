# Havenwild Bevy v0.5.5 pre-build audit

This checkpoint extends v0.5.4 without changing the source-authority boundary or inventing terrain mappings.

Verified before packaging:

- source/project/control metadata align at 0.5.5 / ForgePY 0.4.6 / PCC 1.2.5;
- Bevy is pinned at 0.19.0, bevy_egui at 0.42.0, Rust minimum at 1.95, ForgeGUI at the governed revision;
- Rust source module graph is closed (no missing/orphan modules), shared resolver remains UI-agnostic, and no merge markers/stale pre-DG03 save message remain;
- PCC hostile-patch regression covers 10 unsafe classes;
- DG seed and fixtures cover 16/16 masks, full shared output resolves 36/36 vertices, all 25 semantic cells obey four-vertex invalidation, and corner bit orientation is fixed at NW=1 / NE=2 / SW=4 / SE=8;
- Summer source recovery stays 305/305 and the authored 40x28 acceptance scene stays 1120/1120 source-valid;
- all 349 canonical ElizaWy core assets remain hash-valid;
- ForgePY FULL now fails on every Doctor failure; successful FULL must verify the generated Cargo.lock with `cargo metadata --locked`.

The remaining unverified boundary is the actual Windows Rust compile/test/runtime session. Recipe certification remains intentionally separate and 0/16 until visual review.
