# Havenwild M2D131B3 — Integrated Studio / Build Checkpoint (v0.10.55)

This is one governed PCC update from **source v0.10.54** to **v0.10.55**.
Standalone Project Control Center v1.3.5 and ForgePY 0.4.10 are unchanged.
Original ElizaWy art, world documents, terrain semantics, PIE and world-generation
source files are unchanged.

## Included integration passes

1. **Session-only Shift-isolate disk safety**: the shared layout-save path persists
   the pre-isolate seven-layer visibility array, not the temporary isolated view.
   Manual eye toggling still cancels isolation; user preferences survive restart.
   A native Rust unit test covers isolation -> unrelated save -> restart -> restore.
2. **Unified left authoring dock**: the former separate 62px rail is folded into
   the resizable three-section dock. Sections: Mode & Tools (existing Select,
   Pixel/Tile/Terrain Paint, Erase, Sample, Bridge and authoring settings), Pixel
   Studio, Scene Navigator. Section heights remain independently resizable.
   The *default* width becomes 288px; a one-time layout migration converts an
   untouched 350px legacy context width, preserving any customized width.
3. **Inspector and World Properties**: added persistent Inspect / World / Terrain
   tabs above the existing Asset Browser (default 50/50 split). World now renders
   the existing collision/elevation/traversal editing code inside Inspector,
   including the original undoable SceneV2 structural operations. The floating
   World Properties window is removed. Choosing the World tab authorizes exact
   pixel collision editing; the adjacent canvas Collision eye remains view-only.
4. **World right-click**: context-menu opening records the clicked world cell
   without changing the selected cell, selected object, active brush or layer.
   Choosing 'Select object at cell' explicitly selects. Terrain sample, painting,
   erasure and house/door actions retain the original guarded commands.

## Verification performed outside Windows

- 71/71 Python source regression scripts pass with existing hydrated audit art.
- Rust source structural audit PASS (30/30 modules declared and present).
- PCC adversarial patch-validator self-test PASS.
- Patch is exact-version guarded, SHA-256 verified and tested in a disposable
  installation; PCC creates rollback state before file replacement.

**Cannot certify from this environment**: Rustfmt, `cargo check`, `cargo test`,
actual native Windows GUI interactions, Vulkan FPS, click-through/DPI/resize,
PIE state restore. The next step is PCC Full Quality Gate on your Windows copy.

## Windows test checklist

1. Place the `.pccpatch.zip` beside `PCC.cmd`; do not extract; PCC option 3 to apply.
2. PCC option 1: Full Quality Gate. It runs formatting, Rust check and tests.
   **Do not publish to GitHub until gate PASS.**
3. Launch Studio Vulkan. Confirm one left three-section dock (not two rails),
   Mode & Tools actions work, resizers work, Inspector tabs switch, and Assets
   remain in the right lower half.
4. Open Inspector > World and edit/view collision and elevation; verify undo/save,
   scene vs world scope, and that the canvas Collision HUD button remains view-only.
5. Set nonuniform layer visibility, Shift-isolate, change theme/panel width,
   close and reopen; original visibility should survive. Repeat without restart.
6. Right-click a different world cell while an object remains selected; selection
   must not change until choosing 'Select object at cell'. Sample and paint menu
   commands must operate on the *clicked* cell, not the previously selected one.
7. Check Pixel Studio palette, Grid/Chunks/Zoom/Animation overlays, FPS bottom,
   and PIE startup/stop for unintended regressions. Upload gate output and a
   screenshot or debug bundle if anything behaves incorrectly.

## Explicitly not claimed as complete

- The current docking graph is not yet drag-to-nest/tab/floating UI.
- Pixel image write layers and full 360px expandable palette are not yet built.
- ElizaWy terrain/water transitions, wet-sand and missing art mapping remain
  separate priority work after the GUI-native checkpoint.
