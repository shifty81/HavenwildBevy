# M2D131A structural Full Gate recovery (v0.10.51)

## Origin
PCC on Windows v0.10.50 reached Rust structural audit after passing phases 1–30 and failed solely because the historical M2D06 check required `command_tooltip(StudioCommand::ToggleLauncher)` on a retired Apps rail button. The launcher remains available through Tools > Havenwild Tool Launcher and Ctrl+Space.

## Changes
- Retire the stale rail-tooltip requirement and verify both launcher access paths, the launcher render call, and absence of duplicate Apps rail actions.
- Reinforce the existing M2D131A source regression with menu/keyboard access assertions.
- Repair a separately discovered M2D106 alpha-preparation schedule assertion to tolerate Rustfmt multiline tuples while still enforcing order.
- Bump source metadata, Cargo.toml and Cargo.lock from 0.10.50 to 0.10.51.
- No changes to `src/`, source assets, world persistence, editor placement or PIE.

## Expected Windows verification
Apply using PCC Patch Scan / Review / Apply, run the Full Quality Gate, inspect any further failure, and launch Vulkan Studio only after compilation succeeds. Source-only checks are not native Cargo / runtime certification.
