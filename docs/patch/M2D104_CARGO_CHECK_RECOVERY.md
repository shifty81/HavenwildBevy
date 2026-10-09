# M2D104 — native compile recovery after M2D103 (0.10.24 → 0.10.25)

## Evidence

PCC Windows Full Gate `20261008-203943-669645_full` applied M2D103 (source 0.10.24),
verified 349/349 core assets, and hydrated 63,991 characters. `cargo fmt -- --check`
initially reported source formatting differences; PCC ran `cargo fmt` and passed recheck.
`cargo check` then failed with Rust E0308 at the Metrics status-bar toggle and four E0502
borrow-checker errors in the Generated World inspector actions.

## Source corrections

- In `src/main.rs`, resolve the selected generated object into an owned `Option<WorldObject>`
  before entering the `if let` branch that changes the undo stack/world document.
  This closes the immutable iterator lifetime before mutable borrow operations.
- The Metrics status toggle calls `save_editor_layout(&mut state)` (requires `&mut Studio`).
- Ensure the M2D101/M2D102/M2D103 source checks tolerate `rustfmt` line reflow;
  add PCC-wired M2D101 early warning assertions for these borrow-scope regressions.
- Preserve M2D103 shoreline, bridge, foliage, canvas, metrics, and inspector behavior.
- Synchronize Cargo, Cargo.lock, project metadata, and README to source 0.10.25.

## Acceptance (Windows machine required)

1. Place the M2D104 `.pccpatch.zip` in project root after M2D103 is installed;
   PCC will apply the exact `0.10.24 → 0.10.25` update transactionally.
2. Run PCC Full Gate. Confirm `cargo fmt -- --check`, `cargo check`, `cargo test`,
   and native Studio build pass; ensure there are no new Rust diagnostics.
3. Start Vulkan Studio, open Generated World, select a tree, change X/Y in the
   Inspector, drag it, undo/redo, save and reload. Toggle Metrics from the status bar.
4. Audit coast/paint topology separately. Source compile repair does **not** establish
   terrain visual correctness or missing Wang recipe certification.

## Limits

This source-only patch was inspected and Python/static tests were run without
an installed Rust compiler in the packaging runtime. Rust compilation and visual QA
must be certified on the user's Windows PCC, not asserted by a source-text selftest.
