# M2D102 — 0.10.21 → 0.10.23 cumulative Havenwild Bevy repair

## Source authority and rollout

- Designed for **M2D100 source 0.10.21**, *without first applying M2D101*. The package embeds every M2D101 source change.
- M2D099 is already present in M2D100. Preserve local world saves, `.git`, history, `.pcc`, and project-specific overrides.
- Install via project-local `PCC.cmd` root-drop governed patch intake; apply from the internal PCC, then run its Full Gate.
- No original LPC/ElizaWy PNG, TSX, character image or license file changes in this update.
- In this packaging runtime no Rust compiler is available; Python source/contract tests and ZIP compatibility **do not certify** Bevy/Egui native behavior.

## Coast — source-backed fix, not another overlay

Audit `tools/audit_tiled_shoreline_recipes.py` reads the immutable summer TSX Wang-color IDs and exact Wang tile assignments. It documents:

| Transition | Supported corner masks |
| --- | --- |
| Grass/Dirt/Sand versus Shallow Water | 14/16 each |
| Shallow Water versus Deep Water | 14/16 |
| Grass/Dirt/Sand versus Deep Water | 2/16 each |
| Deep Sand versus Shallow Water | 2/16 |

The generator already tracks `Coast` separately from `Ocean` while recording sea terrain as `DeepWater`. The renderer previously tried to resolve land→DeepWater directly; almost all mixed Wang signatures are absent in that combination, causing the terrain fallback/projection path and hard-looking borders. `resolved_role_for_dual_grid` now renders **Coast DeepWater cells as the source's Shallow Water role**, preserving the original **DeepWater semantic/collision/navigation role** in `resolved_role`. Far offshore stays DeepWater. Shallow/deep transitions then use the authored TSX patterns. The legacy direct deep-to-land fallback is not promoted as a certified art recipe.

The 2 unsupported diagonal configurations (0110 and 1001) remain topology decisions. The existing projection handling is preserved; a docked Metrics > Audit shoreline topology command counts exact, projected, and unsupported boundary vertices in the materialized world. Metrics > Outline missing coast recipes highlights projected vertices yellow and unresolved vertices red. These overlays do not change the art.

The live dual-grid terrain mesh now rounds shared *edges* to display pixels, avoiding fractional-zoom hairline gaps; this does not replace Wang art correctness.

## Permanent 15/70/15 editor and canvas-integrated layers

- 15% Tool Rail on the left, 70% central canvas, 15% Inspector dock on the right, as per M2D101 (included).
- Canvas-independent layer state/visibility buttons anchored at left edge **inside** canvas. Not a second right-hand layer panel.
- The scene/world zoom, Fit and FPS controls appear as canvas chrome at the **upper-right**; world mode now has +/- zoom buttons.
- Right inspector contains selected object metadata and its authored position controls. Click **Metrics ▴** in the bottom status bar to slide up diagnostics **inside the bottom of the Inspector dock**; close using the ▾ control there. Persisted per local editor layout.
- Normal generated-object selection and direct mouse drag from M2D101 is retained; undo and source-preserving persistent authored position override retained.

## Drag-to-span bridge authoring

- New left tool context **Bridge span · drag**, active only in Generated World. Source-backed *horizontal* span only.
- Press on land, drag across contiguous water to land on the far shore, release to preview/validate/commit one undoable authored `WorldObject` with exact endcap/deck/endcap source pieces.
- Source rects from the existing reviewed 32×64 template are repeated, **never stretched**. Water must continue across the two visual rows underneath each intermediate span segment. Minimum 3, maximum 64 cells; endpoints must be land; both rows must be materialized. Invalid/vertical paths fail without editing the document.
- Authored objects persist in `authoredObjects` and appear through the shared world object renderer, selection, collision/traversal connector test and undo/saving path. Existing generated bridge objects remain untouched. Authored bridges cannot be dragged independently without topology revalidation.
- Proper north–south bridges and automatic connection/path reservations remain unfinished pending exact source direction proof and runtime navigation tests.

## Vegetation variation

- Source-preserving, seed/instance-ID-deterministic horizontal mirror for supported world-generated trees, palms, shrubs, flowers and other foliage. Mirror transform applies to the **whole object** across its visual bounds; hit-test and collision bounds do not change.
- Quarter-turn sprite rotation restricted to **single-part 32×32, low-lying, square** decorative foliage. No 90-degree rotation for trees, houses, bridges or characters; no unsupported new atlas assets.
- Visual variant remains stable across camera pan and scene reload because it is hashed from the persistent object ID.

## Local Windows acceptance matrix (not yet performed)

1. Install M2D102 over **M2D100/0.10.21**, **without** M2D101; confirm version 0.10.23, Full Gate and Vulkan Studio build.
2. Inspect coast from screenshots at 50%, 75%, 100%, and 150% zoom. Ensure the coast uses shallow-water art, smooth deep-water fade, no hard square blocks, no 1px seams. Audit shoreline in Metrics and inspect yellow/red exceptions.
3. Confirm 15/70/15 chrome, Layers on left *inside* canvas, zoom/FPS top-right, Inspector right, Metrics from status bar bottom of right dock.
4. Click/drag a tree and house across materialized chunk boundaries; confirm one undo, redo, save and reopen position persistence. Test layer hide/show, selection, tools and painting.
5. With Bridge span active, drag land→water→land across a *two-cell-high* water corridor, release and inspect; undo/redo, save/reload, traversal parity. Invalid, reversed, too short, too long and vertical drags must fail without source changes.
6. Inspect mirrored trees and 32×32 foliage at multiple zoom levels; confirm proper ground-foot anchor, alpha bounds, source art quality, collision parity.
7. Run both Vulkan and DX12 if available and compare water animation and FPS. No claim of finished cliffs, waterfalls or source-region mapping without a separately passed acceptance gate.

## Follow-on asset closure

The project has enough original core ElizaWy/character source material to proceed, but many object/structure regions still lack reviewed canonical authoring recipes. The remaining work should be prioritized by exact runtime usage and TSX/Wang evidence, not arbitrary painting rules or manual guessed index mapping. In particular, Deep Sand shoreline transitions and north–south bridge art require verified source/demo proof.
