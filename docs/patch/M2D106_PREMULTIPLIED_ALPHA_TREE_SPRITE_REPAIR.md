# M2D106 — Bevy/egui original PNG alpha compositing repair

**Source**: 0.10.26 → 0.10.27. **Scope**: rendered texture conversion only; no source sprite/art data is altered.

## Evidence

The Summer tree atlas (`assets/elizawy/Terrain/trees_summer.png`) contains source-transparent texels with nonzero RGB. Two pinned examples at original pixel coordinates:

- `(130,16)` = `(0,48,35,0)` (green rectangular padding)
- `(129,16)` = `(255,255,255,0)` (white atlas guide)

In the observed generated-world screenshot, the land was RGB `(70,130,50)`, but the rectangular area beneath the tree was `(70,138,63)`. Adding the transparent green RGB to the background under *premultiplied-alpha* blending reproduces the change to approximately `(70,138,62–63)`. Thus the renderer was treating a straight-alpha PNG as an egui premultiplied-alpha texture. Bounding-box corrections cannot fix this defect.

## Correction

- `prepare_elizawy_egui_textures` runs between the existing lazy PNG hydration system and `draw_ui` on `EguiPrimaryContextPass`.
- The converted image is the Bevy-decoded **in-memory** texture; original PNG bytes and canonical source rectangles remain unchanged.
- Transparent RGB is premultiplied to zero and partially transparent RGB is converted using Bevy's official linear-color premultiply/set_color_at pattern; opaque source texels are skipped to preserve their exact visual color.
- Source images are not made visible to UI until converted. Conversion is limited to two source sheets per frame to avoid one large initialization stall. Conversion can repeat on an AssetServer hot reload and covers all original PNG sheets (not only trees).
- A Rust regression protects transparent tree-green and atlas-white and semi-transparent color; a Python source contract verifies the exact original pinned pixels and load-to-draw ordering.

## Windows acceptance

1. Apply this governed patch to source `0.10.26`, run PCC Full Gate, and launch Vulkan Studio.
2. Zoom into Summer trees. Verify no opaque green square around the canopy and no white vertical source guide lines. Repeat with mirrored and unmirrored trees and across chunk boundaries.
3. Use Source Browser on the original `Terrain/trees_summer.png` and inspect alpha at the same original coordinates. Verify source image unchanged and other transparent objects still compose correctly.
4. Validate against previous coastline screenshot; this is an alpha fix, **not** certification of Wang transition coverage.

## Boundaries

No generated-art switch, mask-based art repaint, guessed source crop, collision alteration, extra assets, or arbitrary coast transition rules. A live Windows Rust compile and visual Studio run remain necessary to certify the repair.
