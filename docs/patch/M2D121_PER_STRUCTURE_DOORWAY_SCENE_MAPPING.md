# M2D121: Structure context and doorway→room mapping

- Source: 0.10.38 → 0.10.39. Standalone Havenwild source only; no foreign project imports.
- World mode: right-click an **actual house**. `Edit Interior` opens a **per-world/per-house** room file in the Studio canvas; no unrelated house is altered. `Map Doorway → Scene` asks for an adjacent walkable exterior cell. Esc cancels.
- Authored world keeps `structureDoors` (object ID, exterior cell, project-local destination room ID, interior entrance cell) with backward-compatible serde default and undo snapshots. The map is validated for cardinal adjacency and rejects unsafe paths.
- Each editable room is project-owned under `content/rooms/houses/house_<world-seed>_<object-hash>.room.json`, initialized from the authored starter house layout, saved atomically. Original source PNGs remain immutable.
- PIE honors mapped exterior doors rather than inferring any side, loads only that house's room, and spawns at the authored room exit/entry. F at the room exit returns to the exact recorded overworld position. If the room is not yet saved, PIE reports a clear error instead of opening the global starter room.
- Moving a house shifts its authored doorway with it; suppressing/removing the building drops its mapping. Moving the interior exit updates the matching destination coordinate and marks the world dirty.
- Current door marker is a colored D diagnostic overlay, **not certified house-door art**. Cave door mapping and arbitrary scene-asset selection beyond house interiors remain later milestones. No claim of completed source-atlas door art or Windows runtime acceptance.

## Manual Windows tests
1. Full Gate on 0.10.39. Launch Vulkan Studio, materialize a world, select actual visible house, right click; confirm Edit Interior and Map Doorway. Right-click a rock and confirm no house actions.
2. Open two different houses, change and save one room; verify distinct room files and no leak.
3. Map exterior door to a walkable cell bordering the house; verify green D and `Ctrl+S` world save, `Esc` cancels map without mutation.
4. F6 World PIE. Walk to mapped doorway; F enters specific house room; walk to exit and press F. Test alternate inferred side does not enter mapped house.
5. Move a mapped building in editor; verify doorway moves with it, undo/redo behaves correctly, save and reload world. Test door mapping to diagonal/nonwalkable cells fails with explanation.
6. Move room exit, save room, return World, save world, restart, verify PIE spawns at updated point. Test unmapped house still follows legacy provisional route.
7. Confirm original ElizaWy pixel hashes, project identity, no foreign files, and no display/input regression.

Not native-built in this environment. Source-contract tests are not proof of Rust compilation or visual behavior.
