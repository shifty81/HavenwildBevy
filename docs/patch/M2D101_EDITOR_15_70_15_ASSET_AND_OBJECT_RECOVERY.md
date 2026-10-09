# Havenwild M2D101 — asset readiness and editor interaction recovery

**Input authority:** M2D100 / source 0.10.21, with ElizaWy/LPC core assets and pinned Characters.zip. **Target:** 0.10.22 source integration; Windows Cargo build and interactive acceptances still required.

## Inventory versus production readiness

The M2D098 inventory (before M2D099 credits repair and M2D100 Characters.zip intake) counted 349 core assets: 320 PNGs, 28 credits text files, 1 GIF. Runtime catalog covered all 320 PNGs, but authored region coverage was only 30 PNGs: terrain 11/15, vegetation 12/16, structures 4/98, objects 3/186, FX 0/5; 2,892 regions total, 28 reviewed object templates. Source coverage is **not** visual, navigation, semantic, animation, collision or gameplay completeness.

M2D100 restored byte identity of all 28 credits, added the pinned Characters.zip, and tested archive hydration. Character art is available on demand but is not automatically rigged, animated, or gameplay-ready. Do not infer that missing professions, skills, icons, gear, NPC portraits, seasonal structures, wildlife, or interaction effects are complete.

Asset-production priority: (1) approve terrain adjacency and shore animation/FX context; (2) source-exact cliff and waterfall recipes + traversal; (3) annotate/catalog trees, houses, props and furniture with source rectangles, anchors and depth; (4) certify character sprites/animations; (5) create original missing game-specific assets with license/provenance review. Preserve the ElizaWy source PNGs and credits.

## UI ownership — one authoritative 15/70/15 editor shell

- **Left 15%**: permanent Tools + contextual paint material choices, independent of layers.
- **Center approx. 70%**: rendered world/scene, upper-left embedded layer strip with eye/edit/isolate actions, upper-right Fit / zoom / FPS; the strip is UI chrome, not world geometry or a second dock.
- **Right 15%**: permanent Inspector, selection details, source-backed object coordinates, removal actions; no duplicate floating properties window.
- The layer strip has its own hit exclusion so clicking it cannot paint, select, or drag the world below. Status/action bars remain across the top/bottom.
- One tool, one layered-world model. Hidden layers are neither rendered nor selectable; Shift-isolate remains available.

## Generated World movement and history

The legacy Summer Scene already had `SceneCommand::MoveObject`. Generated World did not: it only selected or suppressed generator-owned objects. M2D101 adds a **manual instance placement correction** keyed by stable generator object ID. On primary press Select captures the picked object and pointer-to-anchor cell offset. Movement previews a snapped, nonmutating candidate; release validates the destination and commits one world checkpoint. Position overrides are JSON-serialized, undoable/redoble via the world history, remain separate from immutable generated chunks, and are honored by rendering/hit-testing including after panning across chunk boundaries.

Destination positions are currently restricted to materialized world cells, and only Objects/Structures/Foreground instances may be repositioned. Terrain, water and gameplay connectors must not be moved until traversal semantics are independently certified. Moving an object's sprite does not certify its physics or update every object-specific collision/interaction rule. Any generator-ID version migration must explicitly review compatibility; on generator-ID mismatch, old corrections are cleared alongside old tombstones rather than silently rebound.

## Windows acceptance required before signoff

1. Start in Generated World. Select a tree and a building (if spawned) from **any visible layer** with Select (Q). Drag each across multiple snapped cells. The grabbed part should remain under the cursor while moving.
2. Release to commit. Verify the right Inspector changes to the new coordinates, the original source PNG and ground remain unmodified, and Ctrl+Z/Ctrl+Y undo and redo each move once.
3. Pan across the original and destination chunk edges, including object overhang. The moved object should remain visible and selectable at its new position; it should not render twice.
4. Save the World, close and reopen Studio, then reload the world. Confirm the same object placement and unchanged generator-owned source anchors after regeneration.
5. Hide Objects/Structures from the left embedded canvas strip: neither their rendering nor selection should remain available. Check Shift-isolate and full re-visibility.
6. Select an instance in legacy Summer Scene and repeat click/drag/undo, verifying no terrain overwrite or mixed-layer selection.
7. Test a small and a large window: tool/inspector docks reserve 15% each, canvas remains interactive, layer strip and upper-right controls never paint through to terrain. Test Focus and PIE modes.
8. Run `PCC.cmd full`, native Rust compilation, and Vulkan launch. No green Windows build is claimed by this source-only pass.

## Known follow-up

The current world-object renderer checks visibility across the materialized object set to handle authored cross-chunk moves. Profile large-world iteration and optimize with a spatial index if necessary. Object collision/physics, alpha-precise picking, multi-selection, arbitrary drag of completely authored new world objects, and thorough asset readiness G2/G3 remain separate tasks.
