# M2D129 — Obsolete GUI acceptance repairs

Source: `0.10.46` → `0.10.47`; source-only and PCC governed.

## Observed native failure

Full Gate from `C:\Users\Shifty\Desktop\HW` stopped at M2D114 because the historical
assertion required `Material: {} · choose in bottom Library`. M2D128 moved the
single shared Asset Browser to the Inspector by default, without removing the
Terrain tab or changing the `world_terrain_brush` authority.

## Changes

- M2D114 confirms the live Terrain material label, terrain palette action, shared
  Inspector Asset Browser, and working Pixel Studio wheel/selection authority.
- M2D124 checks that derived-image editing remains disabled and separated from
  semantic terrain painting, rather than matching retired warning text.
- M2D128 source test becomes a minimum-version feature contract (`>= 0.10.46`).
- Existing source renderer, `src/main.rs`, `src/world_doc.rs`, and all source PNGs
  are unchanged; no art or gameplay mutation.

## Acceptance

1. Run M2D114, M2D124 and M2D128 Python regressions and their neighboring GUI tests.
2. Verify `Cargo.toml`, `Cargo.lock`, project metadata, and package source agree.
3. Simulate PCC installation from the byte-identical original `0.10.46` release.
4. Run Windows PCC Full Quality Gate; do not infer success from source-only checks.
