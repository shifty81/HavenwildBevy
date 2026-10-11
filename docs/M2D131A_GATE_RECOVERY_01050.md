# Havenwild Bevy M2D131A: Full Gate source-contract repair (0.10.49 → 0.10.50)

## Evidence

Local Full Gate after successful M2D131A update to 0.10.49 stopped at M2D116 source contract. This is a **preexisting false negative in 0.10.48**, as its Rust PIE start call is line-broken and ends with a Rust trailing comma while the Python test demands one exact single-line string.

Auditing successive source contracts on the original 0.10.48 and installed 0.10.49 identified the following preexisting brittle assertions:

- M2D116 WorldPlaySession start and draw call: normalize whitespace and optional trailing argument comma, retaining ordered parameter checks.
- M2D117 shared Library/Rooms authoring: allow multiline `egui` method chaining, validate canvas-owned Layers rather than removed rail button.
- M2D120 room saving and workspace: recognize semantically identical multiline return and function calls.
- M2D121 mapped doorway call: preserve ordered arguments while accepting multiline syntax/trailing comma.
- M2D123 approved object placement: correctly construct its formatting-resilience mutation probe from multiline source syntax; its required canonical source binding, object layer, bridge rejection and persistent mutation checks remain active.
- M2D126 Tooling/Pixel Studio: accept rustfmt multiline splitter invocation, preserving layout and authoring workflow checks.
- M2D131A workspace version assertion: test the 0.10.49+ contract and exact Cargo/project version agreement instead of pinning one release forever.

## Boundaries

- No Rust gameplay/editor code, original source images, maps, world saves, asset mappings, PCC internals, or ForgePY have been modified in this repair.
- No source mutation guard is disabled and no test removed; the assertions still fail when the required operation is deleted or its required arguments change.
- All tests and byte identity checked in a temporary standalone source fixture; local hydrated ElizaWy and Jaidyn reference files cannot be checked from a source-only ZIP.
- Native Windows Cargo build, Vulkan UI interaction, PIE and in-game visuals **remain unverified** here. Run **Full Quality Gate** on Windows after PCC applies this governed patch.

## Installation

Place the **unextracted** `Havenwild_M2D131A_Gate_Recovery_v0.10.49_to_v0.10.50.pccpatch.zip` into the Havenwild project root next to `PCC.cmd`. Launch PCC, select patch scan/review/apply, approve, and rerun Full Quality Gate. The old 0.10.49 M2D131A patch need not be reapplied.
