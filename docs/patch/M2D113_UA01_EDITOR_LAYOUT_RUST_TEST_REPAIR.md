# Havenwild Bevy M2D113: UA-01 layout regression repair

**Source:** 0.10.31 → 0.10.32. Incremental, overwrite-capable PCC patch.

## Native failure

M2D112 intentionally shows the compact three-frame authoring context on a new workspace, but `fresh_studio_starts_unobstructed` retained the pre-UA-01 assertion that the entire tool-context was closed. The Full Gate stopped at Cargo tests: 122 passed, one failed, despite successfully compiling 123 tests.

## Resolution

- Keep the new layout's three-frame default visible, without launching floating Terrain Studio.
- Update and clarify the Rust default layout regression test.
- Add a native Rust persistence regression for user-driven collapse and reopen.
- Extend the existing M2D112 pre-Cargo source contract to assert the corrected default and the persistence test, preventing future contradictory changes.
- Change only version metadata, layout test source, guard and this note. No original assets, runtime terrain rules, Cargo dependencies, or gameplay behavior are modified.

## Test

After PCC applies the patch, Full Gate must pass cargo check, cargo fmt, `cargo test` (124 total tests if no further tests were added), and build before reporting certified PASS. The latest confirmed user's M2D112 gate failed; do not assume the updated native test has passed until the user runs it.
