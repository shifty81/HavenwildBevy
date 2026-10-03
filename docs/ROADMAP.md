# Implementation milestones — first usable game from ElizaWy

No arbitrary pass-count validators or donor PCC chains. Work is accepted with functioning user workflows and real visual/manual evidence. First Windows machine owns compile/runtime confirmation.

| Order | Milestone | Demonstrable acceptance |
|---|---|---|
| 0 | Standalone baseline | First build from this project with Cargo.lock pinned; DX12 source-pixel river, one visible title, OS fallback; original Havenwild untouched. Fix any compile/API mismatch; no unverified GREEN claim. |
| 1 | Source mapping loop | Floating Atlas -> choose exact original cell -> drag ghost -> replace one scene instance -> undo -> save -> reopen -> same pixels; no source-image mutation. Source identity survives multi-sheet selection. |
| 2 | Workspace normalization | Full-window canvas, no duplicated header/status paragraphs, compact tool rail and optional tools; no permanent Inspector or framing sidebars, native resize/move/snap parity, persisted tools, no input-through overlays. Upstream ForgeGUI CanvasDesktop feature if reusable; repin after verified integration. |
| 3 | DG-00/DG-01 terrain contract + mapper | Semantic world cells, explicit dual-grid coordinate/mask contract, stable recipe IDs/provenance, 16-state Grass/Void certification board, irregular-shape fixture, source browser with 32px snapping and recipe assignment. No atlas coordinates in saved world data. |
| 4 | DG-02..DG-06 surface terrain certification | Source-certify Grass/Void first (including diagonal sprite/composite/unsupported decisions), then semantic live painting, exact dirty-vertex updates, chunk-seam regression, river fixture migration, and Grass/Water shoreline recipes. |
| 5 | Structural elevation | Sea 0, valid +1..+30, all real cliff height tiers, source-exact faces/corners/ends/cave openings, ladders/vines/handholds and connected collision/navigation. |
| 6 | Water and waterfall certification | North/south/east/west where demonstrably present in SOURCE, modular widths, cliff contact, receiving pools, recesses, animated frames and water flow; no false one-cell stretching. |
| 7 | Summer assembly expansion | Bridges, fences, caves, structures, foliage, trees, props, FX, modular original character/animation; one accumulating scene, not disposable isolated demonstrations. |
| 8 | Shared world/game renderer | Unified scene/terrain source recipe -> resolved draw packets -> studio/game same output, collision/nav/animation/layer parity and play-in-editor from same documents. |
| 9 | Finite huge world | 3–15 massive seeded landmasses, ocean outer borders, entire descriptor navigation, lazy chunks, Willowmere authored core and persistent Estate near northeast main-city edge. |
| 10 | Game systems | Persistence, characters, inventory/equipment, interactions, farming/trading/professions, hunting/skinning, water/weather/lighting, online/LAN co-op and Estate portability as scoped in product plan. |

**Immediate focus (M2D082C+):** the user-facing editor now stays on one Generated World rather than a separate River demonstration. Finish visual certification of the four-island/ocean macro layout, mixed-bank shoreline continuity and tree atlas-guide repair; then classify the 205 cliff cells into elevation roles, wire waterfalls/hydrology, broaden seasonal source mapping, and build purpose-first roads/settlements/biomes followed by deterministic weekly-reset procedural caves. The complete 320-image ElizaWy lane is discoverable to worldgen, but automatic placement remains source/context-authority gated.

## M2D082C1 — Terrain authority version-skew repair
- Align Summer profile v6 across M2D080, runtime AssetAuthority, and Native convergence.
- Preserve M2D082C Generated World behavior; no worldgen rollback.
