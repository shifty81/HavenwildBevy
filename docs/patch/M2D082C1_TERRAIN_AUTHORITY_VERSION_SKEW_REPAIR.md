# M2D082C1 — Terrain Authority Version-Skew Repair

Source 0.8.6 is a gate-repair checkpoint on top of M2D082C. It does not change Generated World geography, shoreline topology, tree sampling, or ElizaWy placement policy.

The M2D082C Summer runtime profile is version 6, but three older consumers still capped support at version 5: the M2D080 Native Summer self-test, Rust `AssetAuthority::validate`, and the Native→Bevy convergence verifier. This checkpoint aligns all three with v6 and removes brittle terrain tests that hard-code a particular Studio 0.8.x patch version.

Water remains static/evidence-gated: the eight `summer_water_fill` cells remain RepeatableFill source/detail variants and are not temporal animation frames.
