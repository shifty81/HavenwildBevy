# M2D03-C — One live scene, one PIE control, usable object workbench

## Target and baseline

Cumulative governed root-drop `.pccpatch.zip` **from the published M2D03-A commit, including M2D03-B and M2D03-C** (also safe on an installed B) on the user's active `C:\Users\Shifty\Desktop\Havenwild Latest`; published GitHub `main` currently remains initial M2D03-A commit `3d8a52a`. No GitHub push is performed by this package. The legacy `Havenwild Bevy` folder remains reference-only. This is not a reconstructed Summer demo and does not include original ElizaWy PNG bytes.

## Why the previous passes felt inert

The pinned original Summer River fixture contains **zero manually placed Scene V2 object instances**. A MOV tool on that initial scene cannot pick up and drag inherited legacy ground/cliff cells; it only understands a manually placed independent object. A second "Preview separate v2 scene draft" created unrelated migration output not used by the active canvas. The floating selection action overlay also occupied and blocked the upper-left canvas hit region.

## Consolidated active workflow

- One active layered document (`SceneV2`) over the pinned source-fingerprinted v1 River baseline. The live canvas, saving and PIE all use it. Historical migration and create-new safety code is retained internally for recovery; obsolete user-facing *Preview separate v2 scene draft* and *Create separate v2 draft* controls are removed. Existing local v1/v2 drafts and the original fixture are not silently changed or deleted.
- **SEL is select and drag**, no MOV button or alternate relocate mode. Press captures the original pointer cell before the egui drag threshold changes the cursor cell. While held, the selected exact original pixels and yellow selection bounds preview the valid prospective position. Release applies **one** `MoveObject` history transaction; leaving the canvas or releasing beyond valid scene bounds cancels the move safely. Clicking a plain legacy ground cell reports that it is not an independently placed object.
- The source/object/scene operations live in the dedicated left **Scene / Layers** panel outside the canvas. The panel opens on the first M2D03-C launch even if the prior version's empty layer guide had been hidden; later user closes are respected by persisted layout state. It visibly reports the actual number of placed objects; the empty state explains how to create one. It includes explicit layer selection, instance list, selected original path/rect/footprint, object X/Y position controls, find on canvas, source sampling, remove, placement at selected cell, undo/redo, and **Save Scene**. The upper-left floating selection actions are removed so they do not intercept canvas pointer hits.
- Original source painting remains honest: choosing a complete region in SRC and placing it on Objects/Structures/Elevation/Foreground produces one source-bound instance; Ground remains a separate original-art layer. The actual PNG alpha is not altered. Source selection is manual, not an automatically verified tree/reed/cliff composition. `can_move_object_to` is a pure bounds probe mirroring the transaction's object validation.
- The **existing single Play Scene** toolbar control owns PIE start/stop. There is no second Play From Here / test world / PIE launcher. It snapshots the same scene's current ground and layered visuals, and Stop/Esc returns to the editor; provisional collision is not production traversal authority.

## Actual-test walkthrough

1. Apply this **cumulative A→C patch** to the published A or locally installed B baseline with PCC Option 11 → 2, then run Option 1. It must reach Cargo fmt/check/tests/build on the user's Windows machine before describing the build as green.
2. Launch DX12, expect *Scene / Layers* visible. It must explicitly show **Placed objects (0)** in the untouched River fixture, instead of suggesting that inherited River cells are movable instances.
3. Click a grass location on the canvas in SEL. Open SRC, choose a **visually inspected exact original vegetation region**, select Objects, press the panel's *Place selected source at cell* or PNT+click on canvas. Confirm count becomes one.
4. In SEL, hold left click on the new source-bound instance, drag and release; confirm the original source pixels move in a live preview, the underlay persists, Undo and Redo work. Select the same object by the object list; modify X/Y, remove, undo and Ctrl+S. Close and relaunch: the derived draft persists. No extra authored v2 preview file is created.
5. Place manually inspected reed region over the existing river, inspect alpha via `tools/inspect_elizawy_source_alpha.py` if a dark rectangle remains. Do **not** assume any 32px cell represents a complete plant sprite until source review confirms it.
6. Press the **one existing Play Scene** button and check snapshot visuals, basic marker movement, Stop/Esc restoring editor and no source-art mutation. Verify core original assets with `python tools/audit_elizawy_asset_consumption.py --check --verify-local`.

## Limitations, not to disguise as completion

The 349 source files are catalogued but the source-only checkpoint here has **no original PNG bytes**; it cannot visually certify vegetation alpha, extract actual complete object regions, author a source-exact full Summer demonstration, or certify real cliff topology/connected traversal. The existing static 40×28 River fixture is unchanged. The original 305/305 occupied Summer source cells recovered is **not** 16-mask rule certification (still 0/16). The occasional non-reproduced DX12 ResizeBuffers issue remains logged, not assigned an invented fix. No Windows Rust toolchain is available in the packaging environment. Python/Cargo-free checks are not a native compile or interaction pass.

## Packaging and safeguards

- Original 349 core manifest and source hashes, external credits, both checked-in source fixture documents, Cargo.toml/Cargo.lock, the original reference demo evidence and PCC implementation must remain byte-identical to the published A baseline and B checkpoint.
- `.pccpatch.zip` must contain only explicit changed source/docs/tests and `pcc_patch.json` with matching SHA-256 and byte-count receipts. A complete source-only archive is delivered separately and must **not** be dropped in PCC.
- Do not push C to `main` until the real Windows Full Gate and hands-on behavior are verified.
