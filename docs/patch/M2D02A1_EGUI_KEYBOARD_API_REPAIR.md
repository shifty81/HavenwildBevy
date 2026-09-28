# Havenwild Bevy M2D02-A1 — Native Keyboard API Repair

Focused follow-up to `HW-BEVY-M2D02A-SCENE-FIRST-LEAN-UI-20260927`.

Windows evidence: `run-dx12` compilation failed with Rust E0599 at three
`Context::wants_keyboard_input()` calls in `src/main.rs`. The exact pinned
egui API exposes `Context::egui_wants_keyboard_input()`.

Changes:
- Replace precisely three calls used by canvas Delete / Ctrl+Z / Ctrl+Y
  with the pinned API. This preserves the intended text-input focus guard.
- Teach static audit and M2D02-A selftest to reject that API regression.
- No changes to scene, original art, ElizaWy mappings/registry, DG recipes,
  source versions, PCC host, project settings or dependencies.

Delivery: drop this .pccpatch.zip at project root, PCC > 11 > 2,
then PCC > 1 Full Quality Gate (Cargo fmt auto-normalizes, check, tests,
build), and PCC > 3 Run Studio DX12. Keep source-only ZIP if additional
source changes were made locally after M2D02-A, as PCC patch overwrites the
three specified text source/tool files transactionally.

Local verification: Python tests and staged PCC apply only. No Windows Cargo
or native GUI execution was available during authoring.
