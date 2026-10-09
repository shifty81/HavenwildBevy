# M2D119 — World PIE house/cave transition vertical slice

Status: **source implemented, native Windows Cargo/visual validation pending**. Input baseline: clean Havenwild v0.10.36. Result: v0.10.37. No foreign project or extracted character/art files added.

## User walkthrough

1. Full Gate on Windows and hydrate ElizaWy/LPC sources. Load/materialize World, click a traversable cell near a town house, press F6.
2. Walk to a reachable exterior side of a generated/authored house; when **F: Enter** is shown, press F.
3. The authored 2×2 chunk document appears as neutral **GEOMETRY PREVIEW ONLY** with floor and walls. Walk with WASD. F by the gold exit returns to the exact same overworld position; Esc exits PIE without changing the editor document.
4. F8 from the overworld opens a **diagnostic** seed-driven 2×2 cave for movement and portal testing; this does not spawn a cave entrance. Cave runtime/worldgen visuals are still evidence-gated.

## Implementation

- `src/world_pie.rs`: selects a genuinely reachable house or cave object by footprint, enters isolated `RoomScene`, handles room movement/blocked walls and exit, seeds diagnostic caves deterministically.
- `src/room_scene.rs`: production-independent template loading and validation, starter authored house, cave exit, unique portal contracts, bounded sizes (2x2..3x3).
- `content/rooms/starter_house.room.json`: versioned 2×2 starter room with painted rectangular floor and derived four-direction wall masks, for authoring by future Studio room tooling. No faked/source-modified ElizaWy art.
- `src/main.rs`: visible in-canvas room preview and F/F8 controls, viewport-centered LPC display, no authoring mutation during PIE.
- `tools/m2d119_pie_room_transitions_selftest.py`: validates source wiring and authored room masks; registered twice in PCC (static and Full Gate).

## Not claimed

No final interior texture/wall appearance, no World Editor floor-paint workflow, no automatically generated house door sprite, no terrain cliff or waterfall art improvements, no map cave entrances, no runtime Windows certification. The diagnostic preview is intentionally labelled and is not an asset.
