# M2D131A native Cargo test recovery: 0.10.51 → 0.10.52

## Origin and diagnosis
Windows PCC Full Gate on source 0.10.51 passed phase 83 `cargo check` and phase 79–80 rustfmt normalization; phase 84 `cargo test --all-targets` yielded 157 passed and one failed: `editor_layout::scene_first_layout_tests::studio_convergence_layout_migrates_once_and_persists_resize` at `src/editor_layout.rs:495`. The one-time `studio_convergence_v1_initialized` false migration still set `layout.timeline_open = true` although M2D131 canvas-first design and the Rust test require a collapsed default.

## Source changes
- Make the one-time legacy studio-convergence migration default to collapsed Animation.
- Preserve the saved timeline setting once the convergence marker is set, including deliberate user re-open.
- Extend native Rust unit test to demonstrate an explicitly reopened timeline persists on subsequent startup.
- Normalize the relevant layout test statement to match `cargo fmt` run by PCC.
- Update the M2D131A source-only contract to verify the migration and user-reopen tests.
- Advance exact source metadata and Cargo lockstep to 0.10.52.

No world or source art modifications, no Bevy engine or dependency upgrade, no changes to scene/PIE input, no removal of old tests, no weakening of animation preference persistence.

## Gate expectation
PCC patch must be applied to source 0.10.51. Run Full Quality Gate. `cargo check` and the other 157 Rust tests passed previously, but 0.10.52 native compilation and runtime editor interaction still require a new Windows run. `cargo fmt -- --check` should not need to write `src/editor_layout.rs` again.
