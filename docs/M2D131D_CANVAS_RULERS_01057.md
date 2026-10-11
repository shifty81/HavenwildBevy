# M2D131D — Native Opaque World and Scene Rulers (0.10.57)

- Built upon exact source 0.10.56 / ForgePY 0.4.10 / PCC 1.3.6.
- World and authored scenes share one canvas ruler renderer. Horizontal/vertical ruler bands are **fully opaque**, theme-aware, and never become terrain or scene layers.
- World coordinate labels use signed absolute world cell positions even across pan/zoom and negative chunk coordinates; scene coordinate labels start at (0,0).
- Top-center shared opaque canvas group now includes **Grid**, **Chunks**, and **Rulers**, independently persisted.
- Top-left Layers/Collision cluster and bottom-left Animation cluster shift inward when rulers are visible, avoiding ruler occlusion and matching hit-test reservations. Existing toggle semantics remain unchanged.
- Rulers are display-only and excluded from both authored scene and generated world pointer painting. Focus/PIE suppression retains the user's saved Rulers preference.
- Ruler pan/zoom tick density adapts; hover and selected-cell guides update with the shared camera.
- Exact source PNGs, sprite mappings, terrain generation, collision data, ForgePY and PCC authority remain unchanged.

## Tests

`python tools/m2d131d_canvas_rulers_convergence_selftest.py`
`python tools/m2d02b2_rulers_selftest.py`
`python tools/m2d131b_canvas_layer_overlay_selftest.py`
`python tools/m2d131b2_canvas_chrome_selftest.py`
`python tools/rust_source_audit.py`
`python tools/pcc_patch_selftest.py`

**Certification boundary:** Python source tests do not compile Bevy/egui Rust or verify screenshots. On Windows apply through PCC, run Full Gate, launch Vulkan Studio and exercise both world and scene rulers with zoom, pan, negative world coordinates, switching themes, selection guides and toggles; confirm clicks on ruler strips do not alter terrain and PIE hides the ruler.
