# Havenwild Bevy M2D097C — barrel gate / authority convergence

**Status:** read-only investigation deliverable. This is **not** a PCC-governed patch and should not be dropped into the project update inbox. No Havenwild source files are overwritten.

## Evidence from user Windows run (2026-10-08)

- Project: Havenwild Bevy source `0.10.19`, PCC `1.3.5`, ForgePY `0.4.10`, Bevy `0.19.0`.
- Full gate phase `P65 cargo check` passed with warnings; phase `P66 cargo test` failed after 112 passed / 1 failed. Exact failing test: `world_doc::tests::reviewed_barrel_template_uses_full_height_source_sprite`; panic at `src/world_doc.rs:3111` with message `reviewed barrel template`.
- The Full Gate failed. The Windows Vulkan previous run passed, but this does not certify current screenshot composition.
- Jaidyn Tiled source metadata, native Wang tests, new water-context rules, and static alignment checks passed. These tests do **not** provide visual acceptance of shoreline animation, coastline blending or generated objects.
- `cargo fmt --check` initially failed; PCC ran `cargo fmt`, then the recheck passed.
- Historical Grass/Void 16-mask contract is `0/16` classified/certified. This is a separate historical scaffold, not proof that the live Tiled/native-Wang renderer has only zero tiles mapped.
- Cliff authority: 205 canonical source cells inventoried, **zero** runtime cliff roles certified. Automatic cliff waterfalls/caves remain gated.
- The local project is **not a Git checkout**. Passing Git publish tests ran on disposable fixture repositories, not actual project GitHub publishing.

## Reproducible donor-source finding

The only complete source ZIP in this conversation is dated 2026-10-01 and has package version **0.8.1**. Its builder `tools/build_elizawy_asset_lane.py` registers source images from the manifest but derives all `objectTemplates` exclusively from `content/scenes/summer_world.visual_seed.v1.json`. The visual seed has no Barrel source instance, although the canonical manifest includes `Objects/Furniture/Barrel.png` at **160×64**, with pin SHA-256 `d6416ffdb15762f572a9a850b155207d05d50e2fe6c66bcdbd9a2dfc79df20d2`.

The v0.8.1 runtime authority therefore contains neither a 32×64 canonical Barrel region nor a reviewed Barrel object template. This provides a **plausible root cause**, not proof of the precise v0.10.19 code path. The v0.10.19 source is needed to confirm its expected label, constructor and crop/anchor contract.

## Proposed production fix (pending current-source verification)

1. Add a **separate reviewed object-template input**, not a fake placed instance in the Summer seed and not an ad-hoc edit to generated JSON.
2. Register an **original, source-pixel-verified 32×64 barrel sprite** on the 160×64 sheet. Choose the exact x-column only after inspecting the hydrated PNG; do not guess between five possible 32px columns.
3. Keep the source sprite a single tall region with ground/foot anchor; collision uses the object's base/footprint rather than the entire 64px visual billboard.
4. Teach the current deterministic builder to incorporate the reviewed template and its canonical source region, update the counted templates and lane SHA, and regenerate both authority documents from inputs.
5. Use `world_doc.rs` current regression test to confirm the precise expected template lookup, source path/rect, footprint, layer, anchor and worldgen behavior. Do not modify a failing test to hide a real production mismatch.
6. Validate: `python tools/build_elizawy_asset_lane.py --root . --check`; `cargo test reviewed_barrel_template_uses_full_height_source_sprite -- --nocapture`; `cargo fmt -- --check`; `cargo test --all-targets`; `PCC.cmd` full gate.

## Running the non-mutating doctor against the real source

Unzip this investigation bundle **outside** the Havenwild project, e.g. `C:\Temp\HavenwildBarrelDoctor`. Then from PowerShell:

```powershell
python "C:\Temp\HavenwildBarrelDoctor\tools\hw_barrel_gate_doctor.py" `
  --root "C:\Users\Shifty\Desktop\HWlatest" `
  --output "C:\Temp\havenwild-barrel-report.json" `
  --bundle "C:\Temp\Havenwild_Barrel_Source_Evidence_20261008.zip"
```

The command inspects files and produces a **source-only evidence ZIP**. It excludes raw PNG bytes, executable output, `.git`, caches, tokens and local settings. It does **not** install a patch. Exit code `2` means a known defect was detected; exit code `0` means the *metadata looks healthy*, not that visual acceptance or the Rust gate has passed. Exit code `3` means the source was missing/unreadable.

If the full fresh source ZIP is available instead, it can be audited directly without the doctor handoff.

## Safeguards

- Do not replace v0.10.19 `src/` or generated authority JSONs with copies from 0.8.1.
- Do not add Barrel as a terrain decoration or spawn it in the ocean merely to create a template.
- Do not change original ElizaWy artwork.
- Do not mark the historical dual-grid 16 masks, WetSand, PebblePath, or cliff roles as certified without actual source evidence.
- Keep Vulkan normal; DX12's fixed-size safe path remains separate.
