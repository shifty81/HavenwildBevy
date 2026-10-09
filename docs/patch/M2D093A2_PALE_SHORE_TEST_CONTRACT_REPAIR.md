# M2D093A2 — pale-shore/test-contract repair

Source transition: **0.10.2 -> 0.10.3**. ForgePY remains **0.4.10** and PCC remains **1.3.5**.

Repairs the Windows Cargo-test failures exposed after the 0.10.2 icon-rail compile repair:

- Rust asset authority now requires the current `certified_source_visual_bridge_for_wetsand` pale-shore runtime contract instead of the retired non-automatic-fill label.
- River-mouth shoulder regression coverage excludes river-water cells before asserting Sand/WetSand shoulders.
- Clustered inland MudBank erosion may only validate against another actual inland-bank candidate below the coast-band threshold, eliminating the isolated `[33, -5]` bank cell seen on the certified seed.
- M2D093A source-side checks now guard these exact contracts.

No source PNG mutation, generated artwork, terrain-family remapping, bridge changes, or object-bound changes are introduced.
