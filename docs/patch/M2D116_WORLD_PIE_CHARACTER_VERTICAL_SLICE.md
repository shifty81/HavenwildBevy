# M2D116 — World PIE and source-backed LPC walking, phase 1

**Incremental source:** Havenwild Bevy `0.10.33 → 0.10.34` · **PCC 1.3.5** · **Bevy 0.19** · **No original artwork changes**.

This is the first *functional* editor/game vertical slice after the unified-authoring UI passes. It is deliberately not the full houses, caves, character creator or asset-authoring pass.

## Implemented in this pass

- Enable the `Play in Editor` toolbar command from both the authored scene and the materialized world; `F6` toggles play and `Esc` ends it.
- Freeze a clone of the current world document so editor/world source assets and authored changes are not mutated by play.
- Choose an initially traversable, unoccupied world spawn near the current selection; reject entirely unsupported worlds.
- Read existing world traversal modes, explicit 32×32 collision masks, and solid tree/rock/house/building footprints; prevent stepping out of materialized terrain. `W/A/S/D` / arrow keys move with diagonal normalization, axis sliding and short collision substeps.
- Use the user-hydrated original LPC base-body `Walk.png` (`512×256`; 8 columns × 4 directions of 64×64 frames) as a *temporary base body only*, sourced from `assets/elizawy/Characters/Body/Body 01 - Feminine, Thin/Peach/Walk.png`. No character PNG is packed into this patch. The previous debug marker remains in authored-scene PIE only if the locally hydrated art is absent. Missing art is visibly reported in World PIE rather than replaced by synthetic art.
- Separate direction/animation state for authored-scene PIE and World PIE. World PIE hides editing rails and exits the world-canvas gesture handler before paint, object moves or deletion can run.
- Add a dedicated cargo-free contract test to both PCC maintenance and Full Gate, while retaining native `cargo check`, tests, and build as the actual Rust authority.

## Known limitations, explicitly **NOT** implemented

- World PIE has no streaming beyond the materialized chunk set and no house/cave portal interactions; the view is camera-follow, not a full gameplay camera pipeline.
- Character currently uses the LPC body layer only. Clothing, hair, equipment, skin-tone/body selectors, rigorous row/frame visual certification, character definition data, diagonals, movement states and character creation remain pending.
- Static obstacle blocking is a conservative material/footprint policy, not complete authored object physics. Character sprite foot anchoring and direction mapping need screenshot verification in the actual Windows game.
- Swimming, boats, interactions, combat, needs, NPCs, doors, cliffs and cave traversal are not implemented here.
- No actual stone/ore visibility, atlas Wang mappings or coastline visual recipes were changed by this pass.
- No native Rust build was available in the handoff environment. **PCC Full Gate on Windows is mandatory**; this is an implementation candidate, not certified gameplay.

## Integrated next milestone: playable house / cave trip

1. A `havenwild.scene_link.v1` component resolves stable `WorldId + world coordinate + entry socket → SceneId + scene spawn socket`, with reverse exit and validation.
2. Place portal sockets on reviewed house-door and cave-mouth source-backed objects. Press `F` within reach to enter; never use a roof sprite as a doorway automatically.
3. Load an instanced authored interior using the **same scene/document/render/collision contracts**; default size **2×2 chunks = 64×64 cells** at the 32px grid. If hand-authored composition needs larger, allow **3×3 chunks = 96×96 cells**; no silent expansions.
4. Interior floor painter authors semantic floor cells. Wall style/material selection automatically resolves perimeter, internal wall edges, corners, openings and doorway gaps. The *wall artwork and collision belong to separate authoring layers*. Paint directly with contextual wall/floor previews.
5. Caves use the same scene and transition contract. Authorable cave rooms receive cliff/wall/ground autotiling; generated caves use deterministic seeded topology and explicit reachability/portal validation before decoration.
6. Cliff authoring is discrete `+1…+30` structural topology. Waterfalls appear only when drainage crosses an actual downward elevation step, using reviewed source recipes and preserving collision/navigation parity.
7. Terrain Brush exposes terrain and source-driven autotiling; Cliff/Structure Brush handles discrete levels and explicit connector placement. All paint strokes share editor history and validation but write to distinct semantic/structural records.
8. Stone visibility requires a source-backed worldgen audit (`rock` template selection, `rocky_upland` biome gating, footprint eligibility, atlas handle availability and occlusion/depth/culling), then an in-PIE acceptance scene. Do not invent stones or declare fixed merely because asset files are present.

## Windows acceptance

- Apply through project-root governed PCC intake on installed `0.10.33`, then verify source `0.10.34`.
- Run **Full Gate**, confirm Rust compilation and tests pass. If not, **stop** and provide first error + debug bundle.
- In Studio World Generator, generate/materialize the existing world, select grass near a tree/house, click **Play in Editor** (`F6`). Confirm actual body animation (if the hydrated character source exists), blocked water, blocked objects, camera follow, inability to paint during PIE, `Esc` return to same editor view.
- Open Summer Scene and verify existing PIE and scene editing still work. Save/reopen the world and verify PIE left source/document bytes untouched.
