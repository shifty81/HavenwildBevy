# M2D127 — Unified authoring + floating Library resize

Havenwild Studio `0.10.45` corrects the M2D126 mismatch observed in the user's native screenshot.

## One authoritative editing workspace

The left of the canvas contains one panel adjacent to the compact tool rail, divided into three vertically stacked areas: **Tooling**, **Tool Context**, and **Library / Palette**. The two internal dividers are draggable and saved. The entire left authoring workspace can be collapsed independently. Tool contexts continue to change based on the active Select/Tile/Terrain/Pixel/Objects actions. Pixel art strokes are still intentionally disabled until project-owned derived image editing exists.

The Library has exactly one selection state and three *destinations*: inside the left authoring workspace (default), the canvas-bottom tray (still drag-up resizable), or a floating window. The library toolbar contains Dock left, Dock bottom and Undock for explicit relocation; no duplicate source atlas is created. Collapsing the left workspace temporarily exposes the Library as a bottom tray so it is not lost.

The floating window now has a **dedicated bottom-right corner grip** and persists `[width, height]` separately from the docked Library height, with clamped limits. The old ambiguous egui edge resize is disabled on this window so the explicit handle owns the input. The window can still be dragged by its title bar. Closing the floating Library hides it but does not delete its state.

Layers remains the unboxed canvas top-left icon strip. Original sprites, approved source region tuples, world generation, authored scenes, and PIE implementations are unchanged.

## Test on the native Windows build

1. Run Full Gate from source 0.10.45; verify all Rust tests compile and pass.
2. Confirm the left Tooling/Context/Library splitters resize vertically. Change tools and verify controls change in Tool Context.
3. Undock Library. Drag the lower-right corner diagonally and vertically. Close/reopen and relaunch; verify width/height persist.
4. Dock Library left; then dock bottom. Drag the bottom tray's top grip upward, collapse/reopen, and verify all three views use the same exact source selection.
5. Verify the Layers strip doesn't steal canvas clicks or cover floating windows. Verify moving from Tile to Terrain paint does not change original asset content.

These source checks do not replace the native Rust compile or the interactive GUI acceptance test.
