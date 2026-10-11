# Havenwild Bevy — Standalone Game and Studio

**Source:** 0.10.48 / M2D130 Rust 2021 source-compatibility repair after M2D129 gate repair  
**Engine:** Bevy 0.19.0 (Rust >=1.95)  
**Project Control Center:** `PCC.cmd`  
**Backend:** `ForgePY.py` / **Studio:** `Studio.cmd`

Havenwild is an independent cozy life-sim / adventure game with a native Bevy world/scene editor. This is a **source-only** distribution with no unrelated project workspaces or imported donor code. The ElizaWy/LPC art remains externally hydrated from original pinned sources; original assets are not modified.

## Windows setup

1. Extract this repository-root ZIP into a **new, empty folder**; do not overlay a separate game project.
2. Start `PCC.cmd`; ensure the displayed source version is **0.10.48**.
3. Use PCC asset hydration / verification for the canonical ElizaWy content. Hydrate the optional LPC character collection when testing character PIE.
4. Run **Full Quality Gate** before publishing or claiming a certified build.
5. Launch Vulkan Studio from PCC and test world and authored-scene editing / PIE.

## Authoring state

- World PIE with an initial source-backed LPC body animation path is in place but still requires native Windows verification.
- Layer dock and collapsible Grid/List/Atlas asset library are source-implemented.
- A bounded 2x2 or 3x3 chunk room document, house-specific doorway-to-interior transitions and procedural cave preview exist; final sourced wall/floor rendering remains outstanding.
- Some shoreline Wang tuple assignments, cliff roles, waterfall connections, and visual object coverage still need source-specific review. Do not interpret source-inventory counts as completed rendering certification.

## Sources, build and provenance

Main entrypoint: `src/main.rs`; world/terrain: `src/world_doc.rs`, `src/tiled_terrain.rs`; editor state: `src/editor_layout.rs`; new room data: `src/room_scene.rs`. Project metadata: `project/forgepy.project.json`. Source distribution and identity are guarded by `ForgePY._source_package_include` and `tools/havenwild_identity_boundary_selftest.py`. Previous patch notes are preserved under `docs/patch/` as history, not as current installation instructions.

See `START_HERE_NEW_PROJECT.md` for the current setup and `docs/patch/M2D118_STANDALONE_SOURCE_BOUNDARY.md` for boundary requirements.

### M2D119 (native certification pending)

World PIE now supports `F` near a generated/authored house to enter a 2×2-chunk authored room, walk within its floor/wall collision and use `F` on the gold exit to return to the exact overworld position. `F8` opens an explicitly diagnostic seed-driven cave scene (no fictitious world entrance). The initial room artwork is a clearly labeled geometry preview until exact interior tiles have been assigned. See `docs/patch/M2D119_PLAYABLE_PORTALS.md`.

## v0.10.39 / M2D120 — In-canvas Room Authoring

Open **Edit House Room** from Studio's World command bar. The central canvas edits
`content/rooms/starter_house.room.json`; the bottom Library **Rooms** tab offers
floor/erase/exit tools, a wall-tint picker and 2×2 through 3×3 chunk-size choices.
Use Ctrl+S to save, Ctrl+Z/Y to undo/redo, mouse wheel to zoom, and middle drag to
pan. **Save & Return to World**, then F6 and F to test the edited house in World PIE.
Geometry preview and wall tint do not imply source-backed art certification.
See `docs/patch/M2D120_ROOM_CANVAS_EDITING.md`. Native Windows Full Gate is pending.

### M2D123 — Whole-object placement and original-alpha evidence

The bottom Library now distinguishes 28 pre-reviewed source-authored object templates (placeable in World) from 238 alpha-isolated candidate groups (inspection-only), and explicitly lists 138 dense atlas groups needing manual splitting. Original image SHA identity is checked for all 320 PNGs; source bytes are excluded from source ZIPs. See `docs/patch/M2D123_SOURCE_ALPHA_REVIEW_AND_REVIEWED_PLACEMENT.md`.

### M2D124 — Canvas-native Layers + active tool settings

The former 75px Layers dock is now a compact canvas-anchored icon control without a surrounding panel. Collapse/expand via the Layers icon; eye/active/Shift-isolate and collision overlay behavior remain. The default-open empty Tooling and Tool Options slab is removed; the optional contextual settings strip now shows only active-tool settings, while the Library remains the shared bottom/undocked browser. The Pixel tool remains disabled until a derived-pixel editor exists. Run native Windows Full Gate and DPI/interaction checks before adopting as a certified release.

### M2D125 — Reviewed template placement gate repair

M2D123 previously compared a literal Rust iterator expression, leading to an opaque failure on a divergent working tree. The new check inspects placement logic within the target method and is whitespace-independent, with explicit source/canonical/bridge/immutability checks. A native Rust regression exercises approved placement, saved authored object insertion, and rejection of unknown/bridge templates. No source PNGs or authored world saves are changed.

## M2D126 recovered GUI workspace

Tooling, active tool context, and their draggable vertical splitter have been restored as a unified left authoring workspace adjacent to the vector icon rail. The workspace opens once on migration from the previous reduced layout, then respects user collapse/reopen. The bottom Library is a single shared, source-backed, undockable tray; its visible top grip can be dragged upward to expand or downward to shrink, and its collapsed header remains available. Layers remain individual unboxed canvas controls. Original asset mappings, generated worlds, authored rooms, and PIE are unchanged. Pixel stroke backend, color wheel, and unfinished asset classifications have **not** been claimed as complete.

### M2D127 — Three-section Tooling / Context / Library restoration

The single left authoring panel now stacks Tooling, Tool Context and Library/Palette with independent drag splitters. The same Library can dock left, dock bottom or float. Floating resize uses a visible bottom-right corner handle with persisted width/height. Bottom drag-up resizing is preserved. Unboxed canvas Layers and all gameplay/asset source mapping code remain unchanged. Native Windows compilation and visual acceptance required.

## M2D128 Studio convergence (first working integration slice)

- Permanent bottom timeline controls the existing source-backed world water animation preview clock (play/pause/scrub/step). Selecting an animated terrain vertex displays original source frame thumbnails and durations. This is a viewer/preview authority, **not** a finished animation editor.
- Three independent left workspaces: Pixel Studio (color/palette, non-destructive authoring contract), Game Tools (existing semantic/source/object tools), Scene Navigator (real world chunk/object/door link selection) with stacked resize.
- One shared contextual Asset Browser defaults to the lower Inspector with resizable split and optional left/bottom/floating destinations; no duplicate source catalog.
- Unboxed canvas Layers strip, existing asset mappings, terrain rules, world saves and PIE remain unchanged.
- Direct world pixel painting, onion skinning, editable animation clips, full RGB color wheel, and production animation events are **not** implemented in this slice. Do not mistake new UI labels for those backends.

Run native Windows Full Quality Gate before considering this build certified.

## M2D129 — Historical GUI gate compatibility

- M2D114 and M2D124 static regression checks now validate the existing Asset Browser / Pixel Studio responsibilities, not retired strings from the bottom Library or old pixel-backend warning.
- M2D128 acceptance checks remain valid for future source versions; the test no longer requires `0.10.46` exactly.
- Original source-backed sprites, world data, render mappings, animation preview clock and PIE are unchanged. The repair does not implement derived-image pixel painting or animation clip authoring.
- Native Windows Cargo check, tests, build and visual acceptance remain mandatory after applying.

## M2D130 — Rust 2021 GUI color-wheel build repair

Fix the original-color-wheel pointer interaction using nested Rust 2021 conditionals
without changing its behavior or mutating ElizaWy art. The cargo-free Rust structural
audit now rejects the edition-2024-only `&& let` / `|| let` syntax in all source files
before PCC's long native compile. Full Windows Cargo formatting, check, tests and
visual Studio acceptance remain the final gate.
