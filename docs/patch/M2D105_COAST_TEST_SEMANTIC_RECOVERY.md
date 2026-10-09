# M2D105 — Generated coastline semantic regression recovery (0.10.25 → 0.10.26)

## Windows gate evidence

Gate `20261008-205542-863418_full` passed `cargo check` for 0.10.25, then failed
one of 120 native tests: `ocean_coast_uses_shallow_wang_visual_without_mutating_water_semantics`.
The failed assertion was `materialized seed should include a coastal ocean cell`.
119 other tests passed. All 349 core asset identities and all 63,991 character
files were present, so this is not an asset hydration failure.

## Root cause

The previous test materialized the 3x3 chunk cluster around [0,0] and
searched for a generated water cell with context `coast` **and** role
`DeepWater`. Both the search location and material-role expectation are invalid:
`worldgen_plan::terrain_role` maps the ocean coastline score band
(10000, 11100] to `RiverWater` (shallow), and maps the more distant open
ocean to `DeepWater`. Worldgen's `water_context` returns `Coast` for that
same shoreline band. Thus the original predicate could never match any
normally generated coastal water cell.

## Fix

- Find a genuine generated `Coast` cell deterministically near the configured
  summer island using the same proven world-hydrology test seed.
- Materialize its actual 32x32 chunk; verify that the saved semantics are
  `RiverWater` and the dual-grid Wang visual role is `RiverWater`.
- Paint `DeepWater` explicitly at that coastline location; verify that the
  saved/gameplay role becomes `DeepWater` while the visual shallow-water
  transition remains `RiverWater`, including after serialization.
- Add a PCC-wired source guard against the old impossible coastal test
  assumption, so the mistake is caught before the native Cargo stage.

## Boundaries

Only the Rust coastal unit test, the early test contract, and version/docs
change. No generated-world topology, rendering, runtime roles, TSX mappings,
artwork, player saves, character assets, PCC gates, or bridge/editor logic is
modified. This is **not** a visual certification of all coast and mixed-material
junctions. Unsupported three-material Wang junctions and runtime shoreline
visual defects require separate evidence and corrections.

## Windows acceptance

Run PCC Full Gate from the installed 0.10.25 folder after applying M2D105.
Check `cargo fmt`, `cargo check`, `cargo test --all-targets`, and Studio build.
Then check a manual painted sand island, shoreline audit overlays, object drag,
and dock/metrics behavior in Vulkan Studio. The package assembly environment
cannot certify a Windows Rust build.
