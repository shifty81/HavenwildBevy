# Havenwild Bevy — M2D131B1 Canvas Overlay / Layer Authority (v0.10.53)

## Scope and source

Upgrades the exact-version v0.10.52 project with a safe-first overlay/layer pass. It retains the seven existing scene/world layer IDs, generated terrain/render cache, source-accurate ElizaWy art, scene/world saving, and PIE. This is a deliberately bounded first implementation of M2D131B, not completed multi-mode docking.

## Implemented

1. Two adjacent canvas-native buttons: Layers expands/collapses the layer strip; Collision ONLY toggles collision visibility (does not open World Properties, switch tools, or write collision).
2. Dark **opaque** in-canvas button backing; canvas hit-test exclusion protects the entire buttons and expanded strip against editing click-through.
3. Shift-click layer isolation remembers the original seven visibility flags and restores them on Shift-clicking the isolated layer again. Moving isolation among layers preserves the original snapshot. A manual eye toggle cancels that snapshot; the user's new visibility choice takes precedence.
4. Right dock uses **Inspector above one shared Asset Browser**, 50/50 by default. Vertical drag adjusts a persisted, clamped split ratio (20–80%). The Inspector reserves its half even when selected content is short.
5. **One-time legacy layout migration** exposes the shared Asset Browser in the right dock if it was previously hidden or docked left/bottom; an explicitly floating browser remains floating. A user-chosen ratio persists after the migration.
6. Three native Rust model/layout tests and three isolation tests; source-level tests enforce collision/view separation, exact layer visibility and canvas hit-testing. The historical M2D093B, M2D124 and M2D128 source checks now assert the evolved behavior instead of searching for retired inline isolation/fixed-height code.

## Seven-layer behavior audit

| Contract | Status from source inspection | Acceptance / open issue |
|---|---|---|
| Seven stable visual layers | Implemented, preserved (`LayerId` + `EditorLayout::world_layer_visibility`) | No new duplicate layer authority |
| Select active layer | Implemented (canvas rail) | Verify pointer/highlight at 100%, 125% and 200% Windows DPI |
| Eye visibility | Implemented, locally persisted | Verify each ground/water/objects/structures/elevation/foreground/details class visibly responds |
| Shift-isolate | **Fixed** to restore original visibility flags | Run native `layer_overlay` tests and interactive Shift-click/repeat |
| Invisible layers not rendered or picked | Source paths gate both world/scene visuals and picking | Confirm no stale cached vertices during rapid toggles |
| Collision overlay | **Fixed** as independent view-only top-left toggle | Confirm no collision edits, no popup and no paint on click |
| Collision authoring | Existing property window and undo-backed masks remain | Migrate properties into Inspector > World in B2 (not yet complete) |
| Layout persistence | Implemented + one-time inspector-browser repair | Test save/restart and display resize |
| Lock/opacity/group/reorder | **Not implemented for canonical world layer model** | Requires command/API and document schema review; don't add cosmetic inert icons |
| Layer undo/redo | Visual toggles are local layout settings, not authored history | Define undo policy before adding commands |
| Pixel editor layers | Current world/scene layer behavior not a true editable-image layer stack | Separate project-owned image layer document needed |
| Multi-mode behavior | Shared shell and data model foundation only | World/Pixel/Autotile/Scene mode adapters still to implement |

## Native acceptance required

1. Drop governed ZIP in root alongside `PCC.cmd`; choose **3 — Patch Scan / Review / Apply**. The updater must show **0.10.52 → 0.10.53**.
2. Choose **1 — Full Quality Gate**; verify rustfmt, Cargo check, native tests, build, and version handshake. Native compile **not performed** in the package authoring environment.
3. Launch **Vulkan** Studio; Layers and Collision appear adjacent on the canvas even collapsed. Toggle collision without World Properties appearing or changing the active tool. Canvas below icons never paints through. Toggle isolation and restore preexisting hidden flags.
4. Confirm right inspector and Asset Browser occupy 50/50 on an existing local layout. Resize divider and restart: ratio persists. Explicit floating Asset Browser stays floating.
5. Confirm PIE, asset atlas, original asset SHA identities and shoreline/collision rendering unchanged by the shell patch.

## Next development passes

- **M2D131B2:** migrate World Properties to Inspector > World, contextual tabs, consolidate left 15% workspace as one three-section dock while retaining the vertical tool palette; finalize theme + color controls.
- **M2D131B3:** renderer-backed live nested tabs, drag docking, floating panel graph, mode registry + saved per-mode layouts, preview inset, remaining context gizmos.
- **M2D132:** ElizaWy terrain mapping/inner WetSand→Sand, all authored land/water transition masks and visual certification. Preserve no invented ElizaWy art; native placeholder lane remains separately attributed and provisional.

Implementation and tests are source backed; this document is not a certification of visual correctness.
