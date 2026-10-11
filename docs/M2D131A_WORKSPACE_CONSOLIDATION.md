# M2D131A — Workspace/Canvas foundation (Havenwild Bevy 0.10.49)

## Installation

This is a governed **incremental PCC update for exactly 0.10.48**, with PCC
1.3.5 and ForgePY 0.4.10. Put its `.pccpatch.zip` at the top-level of the
Havenwild project (same folder as `PCC.cmd`) **without extracting it**. Launch
`PCC.cmd`, review the offered patch and approve it, then run **Full Quality
Gate** and launch Studio with the Vulkan option. Original ElizaWy/LPC art,
authored world data, hydrated assets, caches and local editor-layout files are
not included in this patch.

## Included functionality

- Adds an editor-only serializable nested dock graph, stable panel identities,
  tab/split/floating validation, invalid-layout recovery, and unit tests. It is
  the future docker's state authority; *full interactive tab dragging and
  nesting are not yet connected to the GUI*.
- Makes right Inspector / Asset Browser width resizable and persisted in the
  workstation-local layout. The View menu can hide/reopen this right dock and
  the existing three-section left tooling dock independently.
- Removes redundant Apps launcher and Layers buttons from the left tool rail;
  Apps remains under Tools / Ctrl+Space and Layers remains canvas-owned.
- Replaces the permanently full-width animation strip with an optional
  center-only timeline, toggled at the bottom-left canvas. The world pointer
  and scene pointer hit tests both exclude its toggle rectangle.
- Adds a center-only bottom Console with Console, Problems, Jobs and Debug
  tabs. The Problems tab reflects actual runtime asset-sync failures; Jobs
  reflects the real pending texture queue and explicitly does not claim PCC
  build-stream integration. Console history is session-only.
- PIE collapses editor-only timeline/side panels but retains the Console strip;
  switching back to Edit uses the same persisted layout choices.
- Updates old layout-source regression tests to tolerate equivalent multiline
  Rust formatting instead of failing on spacing.

## Acceptance on the user's Windows machine

1. PCC reports **0.10.49**, installed update and a passing Full Quality Gate.
2. Studio launches with Vulkan, no visible flicker or broken layouts.
3. Resize right dock then restart; size persists. Hide/reopen it from View.
4. Layers top-left still selects/isolates layers and blocks painting under it.
5. Animation bottom-left toggles the timeline; clicking it must **not** paint
   or select the world cell beneath the button.
6. Console bottom retains editor messages and runtime asset sync failures;
   Problems/Jobs/Debug present truthful status, no duplicated browser.
7. Start World PIE, confirm editor timeline/panels are hidden and Console
   stays accessible; stop PIE and confirm selection, camera and panels restore.
8. Verify paint/select/erase, world save/reopen, original source hashes and
   scene/world PIE remain unchanged.

## Deliberately still pending (M2D131B/C)

- Real dock drag/drop, tab nesting, undocking for all authoring panels, and
  arbitrary user-customizable splits (the **state model** exists only).
- Borderless separately pannable/zoomable PiP canvas Preview.
- Grid/ruler relocation and expanded zoom gizmos/performance diagnostics.
- Event-based build/job/console stream, measured CPU/GPU timings and Vulkan
  performance work. Native acceptance and Bevy compilation cannot be certified
  from this source-only build environment.
- Source-exact contextual bridge placement replacing the existing Bridge mode;
  it is intentionally **not removed** until an equally functional flow exists.
