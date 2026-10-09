# M2D120 — Room authoring integrated in Studio (v0.10.38)

## Scope

- The World Editor command bar gains **Edit House Room**. It loads `content/rooms/starter_house.room.json` and displays an independent room canvas in the existing Studio central view, not a floating duplicate editor.
- Room capacity can be selected from **2×2, 3×2, 2×3 or 3×3** chunks; existing floor and portal locations are preserved, and shrinking refuses to drop an occupied portal.
- The shared bottom Library opens on **Rooms**. From that tab choose **Paint floor**, **Erase floor** or **Move exit**. Painting floors recalculates wall-adjacency masks in the RoomScene authority.
- Wall color is an authored per-room tint and updates both canvas preview and geometry-only house PIE. It does **not** certify or manufacture an ElizaWy floor/wall sprite. Source art is still unbound; the preview explicitly states this.
- Mouse wheel zoom anchors at pointer; middle-mouse drag pans. Each continuous stroke is one undo transaction, with Ctrl+Z/Y and command-bar controls.
- Portals stay on walkable floor and cannot be accidentally erased. A room is validated before a transactional, atomic save. Save & Return safely returns to World Editor; pressing Ctrl+S saves the same authored document.
- World PIE house entry consumes that document on its next `F` interaction. This is not a hot-reload of an already-entered room. World PIE remains frozen relative to authored world edits.
- Inspector includes a **Diagnose visible stones** action: count placed stone/rock instances, layer visibility, LOD state. Does not claim the missing-stone problem is visually fixed.

## Known limitations

- The final source-backed interior floor/wall sprite bindings, procedural cave entrance placement on certified cliff hosts, cliff and waterfall autotiling, door sprite placement, and composed outfit/character renderer are **pending**.
- Native Rust compilation and Vulkan/PIE testing require the Windows machine's PCC Full Quality Gate.

## Acceptance on Windows

1. `PCC.cmd` -> Full Quality Gate on 0.10.38. Do not publish on a failed gate.
2. Open Studio and click **Edit House Room**. With Library / Rooms visible, paint adjacent floors and confirm walls rebuild. Choose the exit tool and move it to an existing floor.
3. Change wall color, Ctrl+S, back to World. Reenter room editor and confirm the persisted geometry/color/exit.
4. Undo/redo a multi-cell drag; save and validate that the room's portal still occupies floor.
5. Generate a world, press F6, approach a reachable house, press F, inspect geometry preview and wall color; return using F at the saved exit.
6. In World Inspector click **Diagnose visible stones**, record generated count, Objects visibility and zoom/LOD state. If zero, regenerate on same seed with current generation code; if nonzero but invisible, inspect atlas hydration/visibility rather than further increasing density.
7. Check no references to foreign game/source identity are packaged and the complete rollup SHA report matches the bytes.
