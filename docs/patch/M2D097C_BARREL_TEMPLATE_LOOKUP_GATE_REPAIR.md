# M2D097C — Barrel runtime regression lookup correction

## Baseline

- Havenwild Bevy source: `0.10.19`
- PCC: `1.3.5`; ForgePY: `0.4.10`
- Evidence: `Havenwild_Barrel_Source_Evidence.zip` generated October 8, 2026.
- Previous `src/world_doc.rs` SHA-256: `6feb8ee5eb78b7a38cddfc2142f9a587c4eea527461f895a12aab1cc239aa409`

## Verified root cause

The Rust regression `reviewed_barrel_template_uses_full_height_source_sprite` incorrectly
searched `AssetAuthority::terrain_object_templates()`. That method intentionally returns
only templates whose *every part* comes from `Terrain/`; a valid barrel from
`Objects/Furniture/Barrel.png` will never be returned.

The actual world generator in `place_purpose_objects` resolves settlement clutter
through `AssetAuthority::templates_with_tag("clutter")`. This collection applies
the real world-generation safety and eligibility checks. The corrected regression
now uses that same production lookup, identifies the barrel by its canonical source
path and reviewed label, and verifies reviewed provenance, 1x1 footprint,
full 32x64 crop, and [0,-32] foot anchor.

## Scope

- Replaces exactly one file: `src/world_doc.rs`, modifying only this regression test.
- Adds this documentation under `docs/patch`.
- No renderer changes; no asset changes; no generated-authority JSON changes.
- Keeps source version `0.10.19` because no runtime/API behavior changes.
- No dummy seed object added; no changes to the ElizaWy original source.

## Installation

1. If `src/world_doc.rs` changed since evidence capture, **do not apply** this
   full-file replacement. Compare its SHA256 to the baseline above first.
2. Drop the `.pccpatch.zip` unextracted in the project root (`HWlatest`).
3. Start `PCC.cmd` and approve the governed update.
4. Validate:

```powershell
cargo test reviewed_barrel_template_uses_full_height_source_sprite
cargo fmt -- --check
cargo test --all-targets
# Then PCC option 1 full gate
```

This patch is source-evidence and deterministic-validator checked, but the full
Windows Cargo quality gate must still run on the user's hydrated machine.

## Follow-on visual certification

After the gate passes, regenerate the v12 world and inspect the physical barrel
sprite, slow shoreline animation/coherent phases, freshwater-only lily pads,
and coast topology in Vulkan. This unit test alone is not proof of correct visual output.
