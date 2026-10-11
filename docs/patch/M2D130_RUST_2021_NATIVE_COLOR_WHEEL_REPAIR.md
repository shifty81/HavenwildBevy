# M2D130 — Rust 2021 compatibility repair

## Windows evidence

User PCC v0.10.47 native Vulkan compilation failed at `src/main.rs:7527`:
`let chains are only allowed in Rust 2024 or later`. The later Full Gate
failed at Rustfmt on the same expression. The 349-file core-asset validation
and upstream PCC historical source checks passed.

## Fix

- Replace `if (response.dragged() || response.clicked()) && let Some(pointer) = ...`
  with nested conditionals. This preserves pointer drag, click, hue ring,
  saturation/value square, and save-on-click / save-on-drag-end behavior.
- Keep package `edition = "2021"` and pinned Rust 1.95; no edition migration.
- Add a cargo-free structural audit guard to reject `&& let` / `|| let`
  patterns in any Rust source file before future slow builds, with both
  positive and negative fixtures. Native compiler is still authoritative.
- Do not touch original art, mapping dictionaries, gameplay, world documents,
  animation data, inspector docking or PIE state.

## Acceptance

1. `python tools/rust_source_audit.py` passes.
2. `cargo fmt -- --check` passes on Windows.
3. `cargo check`, `cargo test` and `cargo build` pass in PCC Full Gate.
4. Vulkan Studio starts; Pixel Studio color wheel responds to dragging and
   clicking; world render, Layers, navigation and animation preview persist.
5. Source-only package and PCC patch hash/identity validations pass.

No native Rust toolchain is available in the packaging runtime, so only
source-side checks and PCC patch simulation can be performed here.
