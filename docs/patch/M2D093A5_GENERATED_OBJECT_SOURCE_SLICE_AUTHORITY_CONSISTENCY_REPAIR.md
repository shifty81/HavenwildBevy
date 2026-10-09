# M2D093A5 — Generated object source-slice authority consistency repair

Source transition: **0.10.5 -> 0.10.6**. ForgePY remains **0.4.10** and PCC remains **1.3.5**.

The 0.10.5 Windows Full Gate reached **93/94 Rust tests**. The only failing assertion still required every Generated World object part to be an exact canonical binding even though M2D093A3/A4 intentionally certify tree/rock render crops as immutable sub-slices inside canonical ElizaWy regions. Startup quarantine used the same obsolete exact-binding rule and could remove valid cropped objects after validation accepted them.

0.10.6 normalizes all generated-object authority consumers to the existing fail-closed `contains_source_slice()` contract: world validation, radius-two acceptance, and startup quarantine agree. A regression test now proves quarantine preserves a valid contained crop while removing an out-of-bounds object part. Terrain cells, authored overrides, behavior profiles, learned corrections and source artwork remain exact/immutable.
