# M2D093B — Editor convergence + viewport cache

Source: 0.10.11  
Prerequisite: exact Havenwild Bevy source 0.10.10 / M2D093A9

## Purpose

This pass responds to the live Studio review where the permanent right layer rail appeared inert, the editor also exposed a second Scene/Layers window, Paint no longer exposed semantic terrain choices, the Paint/Eraser/Eyedropper icons were ambiguous, and Fit World/panning remained unnecessarily expensive.

The pass does **not** declare the coastline or opaque-green tree backdrop visually solved. Those remain visual-recovery work after this editor/runtime convergence checkpoint. It does, however, repair the separately identified white tree guide-line failure by upgrading object render-bound certification to v3 edge-guide-safe source analysis. Original ElizaWy source pixels remain immutable.

## Surface convergence

- The permanent right rail is the one layer-control surface. Each canonical Scene-v2 layer has an immediate eye toggle and an edit-destination icon; Shift-click isolates that layer.
- Hidden authored layers, generated terrain lanes and generated objects are skipped by rendering; hidden authored objects are not picked by Select.
- The old floating/dockable Scene/Layers application and its persisted layout state are retired.
- Chunk Manager is folded into Generated World as a collapsed materialized-chunk diagnostic section rather than a competing world-control app.
- Standalone ElizaWy Asset Authority is retired; source/pattern/evidence authority remains in Terrain Studio.
- The top toolbar no longer repeats the active layer already communicated by the right rail.
- Object Properties remains on-demand for selected-instance coordinates and destructive instance operations.

## Direct-tool repair

- Select remains the conventional arrow.
- Paint is redrawn as a recognizable brush, Erase as a slanted rectangular eraser, and Sample as a pipette/eyedropper using native egui vector drawing.
- Selecting Paint in Generated World immediately exposes a contextual material palette beside the left rail: Grass, Dirt / river bank, Dry sand, Wet / tidal sand, Shallow / river water, Deep water and Pebble / stone path.
- Material selection updates the semantic brush and the matching terrain composition layer. World Generator no longer owns basic brush/material selection.

## Viewport cost reduction

- Source sheet path -> loaded sheet index is now a persistent `HashMap<String, usize>` instead of being rebuilt during every Generated World frame.
- Exact dual-grid vertex visuals are cached per world vertex. Cache invalidation is keyed to world revision, pending semantic edit count and loaded-sheet count.
- The removed Scene/Layers window no longer performs full generated-cell/object layer-count scans every egui frame.
- Existing visible bounds, eight-cell overhang query, far-detail LOD and collision overlay tiers remain intact.
- Studio startup now compares the compiled `CARGO_PKG_VERSION` against `project/forgepy.project.json`. A stale executable fails visibly instead of silently rendering an old editor against newer source. Launch through `Studio.cmd` / PCC so Cargo rebuilds current source.

## Object-bound authority convergence

- Full Gate object certification is upgraded to v3. Long disconnected 1–2 px alpha strokes on declared atlas-slot edges are treated as source-sheet guides rather than object artwork; ordinary edge-touching art is retained.
- A source-byte-independent regression test locks that geometry rule, while Full Gate still hash-verifies the real hydrated ElizaWy PNGs before certification.
- `build_summer_flatworld_mapping.py` no longer owns a second connected-component tree/rock crop algorithm; it delegates to `certify_worldgen_object_bounds.py`, leaving one render-bound authority.
- This specifically targets the returned white vertical tree line. The opaque green backing visible in the latest screenshot remains an explicit interactive acceptance blocker rather than being declared fixed.

## Acceptance

Windows Full Quality Gate must pass, including M2D093B. The rebuilt title bar must read **v0.10.11** before visual acceptance. Interactive Studio verification must confirm: eye toggles visibly affect the canvas immediately; Shift-click isolates; Paint opens the material palette; the three redrawn icons are unambiguous; duplicate layer/chunk/asset-authority windows are absent; and Fit World/panning is materially smoother without losing exact source detail.
