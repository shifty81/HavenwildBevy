# M2D02-B1 — Scene v2 draft adapter / transaction foundation

Focused patch based on M2D02-A plus A1 keyboard repair. It **does not activate v2 rendering or replace the current v1 canvas**. The M2D02-A1 Windows Full Gate is not available to the author yet: run the user-side gate before treating B1 as a green baseline.

## What becomes possible

- Scene > Preview separate v2 scene draft reads the currently saved v1 file, fingerprints exact original bytes with SHA-256, validates 40×28 fixture and shows a summary without saving anything.
- Scene > Create separate v2 draft publishes `content/scenes/summer_river.scene.v2.draft.json` by stage/sync/hard-link create-new, without touching `summer_river.scene.json`. Existing draft means an explicit error, not overwrite. Unsaved v1 edits block the preview/commit. If source v1 bytes change after preview, publishing refuses until re-preview.
- The v2 document retains every original legacy tile and semantic `role`, deterministic cell identities, exact v1 source hash, seven logical render layer identities in three UI groups, sparse visual overrides, sparse *unpopulated* structural channels, stable complete-object primitives, protected regions, generation-removal tombstones, and monotonic object IDs.
- A typed v2 command/history core implements grouped multi-cell strokes, place/move/remove object, undo/redo, revision-aware savepoint dirtiness; transaction history stores per-touched deltas instead of copying all 1120 legacy tiles for every command.
- Source binding validation fails closed on absolute/traversal paths, missing source rects, duplicate cells/IDs and out-of-bounds object placement. Structural values are NOT guessed from visual role hints. The current 0/16 DG status, M2C4 review evidence and ElizaWy original assets are unchanged.

## User test after PCC Full Gate

1. Run PCC 11 > 2, then PCC 1. Rust tests should include M2D02-B1 tests for canonical v1 import, SHA-256 vectors, round-trip, failure rollback, sparse command undo/redo and generated-object tombstone behavior. PCC's formatter may normalize the newly authored Rust module first.
2. Open Studio via PCC 3; confirm existing Select-first full-canvas behavior still works.
3. Scene > Preview separate v2 scene draft. Confirm 1120 source cells and source SHA prefix. No disk file should appear until explicit Create.
4. Scene > Create separate v2 draft. Verify separate `content/scenes/summer_river.scene.v2.draft.json`. The original `summer_river.scene.json` must remain unmodified by this operation.
5. Repeat Create: it must refuse to replace the existing draft. There is intentionally no automatic v2 canvas activation yet.

## Honest scope / follow-up

This is a **foundation** and preview/draft export, not completed whole-object visual manipulation, regenerated scenes, runtime structural terrain editing, or full-scene Pixel Studio. Current Canvas Save/Undo remain the v1 draft route during this transitional pass. M2D02-C must switch to an authoritative v2 runtime/editor adapter, unify live selection/save history, complete-object visual placement and validation before enabling Regenerate. No third-party derivative art gets promoted. No new dependencies or Cargo.lock updates.
