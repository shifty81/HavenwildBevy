# M2D07Q — Canonical object-template region repair

Source **0.7.1 → 0.7.2**.

The Windows Full Gate reached Cargo tests and exposed a real authority inconsistency: all 20 source-exact Summer object templates referenced exact ElizaWy source rectangles that were valid inside the canonical source images but had not been inserted into `canonicalRuntimeRegions`. Rust correctly rejected those placements because Havenwild policy requires every runtime placement to resolve through the single ElizaWy asset lane.

This repair:

- registers every exact object-template source rectangle as a canonical runtime region before object templates are emitted;
- increases the runtime region count from **2,183 to 2,203** while keeping the same **320 immutable ElizaWy source images**;
- records `objectTemplateUseCount` and `source_exact_visual_study_object_seed` provenance on those regions;
- strengthens the M2D07 Python self-test so every template part must resolve to the same canonical region ID, source path, and source rectangle;
- keeps generated artwork disabled.

No PNG bytes are added, generated, cropped, recolored, or modified.
