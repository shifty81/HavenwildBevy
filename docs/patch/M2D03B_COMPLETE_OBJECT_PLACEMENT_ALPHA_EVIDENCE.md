# M2D03-B — Source-exact complete-object instances and alpha diagnosis

## Working baseline

- Active working folder: `C:\Users\Shifty\Desktop\Havenwild Latest`.
- Published **M2D03-A**, branch `main`, initial commit `3d8a52a46e134374117d16f6dad86e47e67722c4` (125 files).
- This is an **incremental governed PCC patch** against the published A files, **not** an automatic GitHub commit, not a replacement for `.gitignore`, and not a cumulative patch from F1.
- The original 349-file ElizaWy/LPC Revised core asset manifest, 320 PNGs, recovered source metadata, reference v1 River fixture, 0/16 dual-grid certification and `Cargo.lock` are not modified.
- Any workstation-local v1 and v2 derived draft stays untouched by the patch itself. Old source `Havenwild Bevy` remains a read-only reference.

## Root-cause correction

M2D03-A made the complete PNG catalog discoverable and composited v2 visual layers above inherited ground. However, placing vegetation/cliff-region artwork through the canvas still created sparse *cell paint* operations; `SceneCommand::PlaceObject`, `MoveObject` and `RemoveObject` existed but were not available through live scene controls. Users therefore could not independently move or remove complete, multi-tile instances.

M2D03-B wires these existing v2 operations rather than adding a new scene model:

- On **Elevation**, **Structures**, **Objects** and **Foreground**, placing any manually selected exact PNG region creates **one source-bound object with an ID**, not a replacement ground cell. The selected world location is the *bottom-center-ish placement cell* (for even-width art, bias to the left). The crop's actual pixel size determines its visual footprint in 32px scene cells. The code does **not** infer the object's actual alpha silhouette, collision, heightmap, game role or source composition; the user chooses the entire source region manually.
- **Ground**, **TerrainDetails** and **Water** remain editable sparse cell overlays. Ground continues to refuse non-Terrain source art. V2 layer drawing and PIE frozen snapshots are unchanged: original art RGBA composites over lower layers, with no modified PNGs.
- **SEL** selects the uppermost complete object by bounding footprint; dragging with the primary pointer moves the whole instance, preserving grab offset. **MOV** is enabled as a click-to-select-then-click-new-foot fallback. The selected-instance panel offers sample and remove actions. **ERS/Delete** removes the selected/active-layer object via a v2 history transaction; it does not erase grass/water underneath. Undo/redo, save, reopen and PIE use the existing v2 transaction, atomic-save and snapshot contracts. Off-map object moves are refused rather than silently clamped.
- Source region width/height controls now follow the selected PNG dimensions, rather than arbitrarily stopping at 12 cells. The browser paints a checker *behind* the original PNG to help identify transparent pixels. Neither preview nor compositor invents/retouches source alpha.

## Source alpha inspection (not image modification)

To identify the source of a visible black rectangle, manually select the region in SRC and run the independent stdlib-only report with its **actual** catalog path/coordinates:

```powershell
python .\tools\inspect_elizawy_source_alpha.py --asset "Terrain/plants_summer.png" --rect 0 0 32 32
```

Replace `--rect` with the exact selected source rectangle, not a guessed sprite crop. The tool verifies the full original PNG's byte count, SHA-256 and dimensions against the pinned canonical manifest before analyzing a 8-bit non-interlaced grayscale/RGB/indexed/gray-alpha/RGBA PNG. Unsupported PNG formats fail explicitly. It reports fully transparent, partially transparent, fully opaque, fully opaque exactly black and near-black pixel totals **for the selected region only**. The report never writes source art, crops, tile definitions, scene drafts, collision or certification. A dark rectangle due to original opaque pixels requires source/provenance review; one caused by missing terrain beneath alpha requires layer composition review. Neither is silently erased.

## Explicit limitations

No pre-approved vegetation/reed/cliff prefab is fabricated. Selecting a complete source region is still manual and marked as an authoring draft until reviewed. Water-root placement is a visual *foot anchor*; structural water/shoreline collision and placement-rule verification remain pending and are not inferred from color or name. The original demo PNG is a visual reference, not an editable game map. Cliffs must still be assembled from correct original faces/tops/ends/corners; this pass supplies manipulable instances but does **not** certify all source combinations or reconstruct the large Summer demo. Actual source PNG bytes are absent from source-only handoff archives and cannot be visually certified by the authoring environment here.

## Windows acceptance

1. Drop this `.pccpatch.zip` only into the root of `Havenwild Latest`. PCC **11 → 2**, then **1 Full Gate**, then launch **3 DX12** (or **4 Vulkan** for graphics isolation). Do not drop the complete source-only ZIP into PCC.
2. In SRC select an **actual visually inspected** original vegetation/reed region and its exact cell width/height; target Objects; place it on grass and water in the pinned River scene. Confirm alpha reveals ground/water beneath, no unintended background rectangle is introduced by the renderer, and original source sprite is unchanged. If a dark rectangle remains, run the alpha CLI on that exact source selection.
3. Select the instance with SEL, drag it, use MOV to click a new placement foot, remove it with Delete, undo/redo, Ctrl+S, close/reopen and verify persistence. Confirm inherited Ground/Water remain intact. Attempt an off-edge placement: it must refuse safely.
4. PIE must use the same visible layered scene snapshot, with the provisional v1 traversal explicitly distinguished from production collision. Esc returns to editing, without modifying the original.
5. Repeat with an inspected cliff multi-cell **source region** on Elevation. Correct cliff adjacency and connected routes are still separate source-authority review and **not** certified by this patch.
6. `python tools/audit_elizawy_asset_consumption.py --check --verify-local` confirms all 349 core original files remain intact. `python tools/inspect_elizawy_source_alpha.py --asset ... --rect ...` checks the exact suspect crop.
7. If native Cargo fails, return the first compile error and new automatic debug bundle. This environment can run only cargo-free structure/fixtures, **not** Windows Rust compilation or interactive alpha acceptance.

## Verification boundary

- Cargo-free checks: source catalog/manifest audit, new synthetic RGBA alpha fixture, existing PIE contract, Rust structural integration tests, legacy M2C2/3/4 tests.
- The Windows Full Gate, DX12/Vulkan runtime, original pixel inspection and mouse interaction are **pending**, not claimed passed.
