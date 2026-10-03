# M2D082F1 — deterministic fill regression repair

Source **0.8.10** is a narrow certification repair over 0.8.9.

The 0.8.9 runtime correctly closes ambiguous diagonal water and then selects Grass through `summer_flatworld_fill_for_world`, which intentionally chooses one deterministic source-backed Grass variant from the six authored fills. The new Rust regression incorrectly compared that result to the single conservative `safeFill` binding. On Windows the runtime returned `[160,64,32,32]` while the test pinned `[96,32,32,32]`; both are valid source-backed Grass fills.

This checkpoint changes the regression to compare against the same deterministic Grass-variant authority used by the runtime. No terrain topology, water, worldgen, or asset-placement behavior is changed. The source also includes the rustfmt normalization performed automatically by the Windows Full Gate so the next gate does not dirty the working tree before certification.
