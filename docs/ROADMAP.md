- **M2D082D — Dual-grid coordinate authority:** semantic-cell authoring is now distinct from vertex-centered render output; fixes down/right paint offset and square seam leakage before the kitchen-sink world population pass.
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

**Immediate focus (M2D093A+):** visually certify the restored source-backed WetSand coast, explicit river mouths, purpose-object ecology reservations, and exact immutable-alpha tree/rock/stone render bounds on Windows. Finish Terrain Studio recipe editing/live-preview/pixel-correction workflow and retained chunk rendering so far zoom preserves detail without slideshow panning. Restore Generated World PIE with an animated development character, then classify the 205 cliff cells into elevation roles and continue into continuous cliffs, waterfalls and caves. The complete 320-image ElizaWy lane remains discoverable to worldgen, but automatic placement remains source/context-authority gated.

## M2D082C1 — Terrain authority version-skew repair
- Align Summer profile v6 across M2D080, runtime AssetAuthority, and Native convergence.
- Preserve M2D082C Generated World behavior; no worldgen rollback.


## M2D090 — ElizaWy kitchen-sink Generated World
- Promote exact coordinate-equivalent Spring/Summer/Autumn/Winter terrain, tree, plant, wildflower and cliff regions into the unified runtime authority.
- Regenerate the primary four-island world under generator v3 with 0/+1/+2 elevation metadata, hamlet/town anchors, road/cave connectivity, bridges, waterfall drops and cave approaches.
- Populate visible biomes with source-exact trees, shrubs, foliage, wildflowers, rocks, water details and bank plants; populate settlements with reviewed houses, storage clutter and outdoor lighting.
- Use source-exact reviewed waterfall and cliff/cave showcase regions now; finish continuous contour-cliff topology and ordered waterfall animation from screenshots/source evidence rather than guessed atlas semantics.
- Preserve the 0.8.7 dual-grid coordinate fix and 0.8.9 land-connectivity-first shoreline behavior.
- Treat the upstream ElizaWy Summer demo as a density/composition target, not a layout to copy 1:1.

### M2D090A — Seasonal ground gate repair — COMPLETE IN SOURCE 0.9.1
- Correct the stale pre-kitchen-sink Rust guard that required every generated ground cell to come from `terrain_summer.png`.
- Require coordinate-equivalent Spring/Summer/Autumn/Winter terrain sheets by generated season while continuing to reject cliff sheets as ground authority.
- Runtime/worldgen behavior is unchanged from 0.9.0; this is certification alignment only.
### M2D091 — Visual authority / seasonal-island differentiation — COMPLETE IN SOURCE 0.9.2
- Replace same-layout seasonal islands with distinct Spring/Summer/Autumn/Winter silhouettes, hydrology, settlement classes, elevation patterns and vegetation density.
- Tighten coast Sand/WetSand bands; use one-cell dirt roads; keep most river/lake banks grass-backed with deterministic clustered erosion.
- Place bridges only on real road/water crossings; use the reviewed south-facing waterfall body/splash crop without the north cap; use one complete wide cave-entrance source section.
- Move generated lamps/clutter away from house footprints and bump Generated World to v4 so prior generated chunks are rebuilt.
- Harden seasonal tree source sampling to a two-source-pixel inset while preserving the world footprint. Visual screenshot confirmation remains required before the persistent guide-bar defect is considered closed.

### M2D091A — Cliff-gate acceptance alignment — COMPLETE IN SOURCE 0.9.3
- Keep the reviewed waterfall and cave source recipes available while automatic placement remains gated until continuous cliff faces exist.
- Remove the stale radius-two Rust regression requirement that forced `Worldgen waterfall` / `Worldgen cliff cave entrance` objects onto flat generated terrain.
- Require bridge/town/tree population to remain present, require gated waterfall/cave objects to remain absent, and verify their reviewed recipes are still available in the asset authority.
- Runtime/worldgen behavior is unchanged from 0.9.2; this is certification alignment only.

### M2D091B — Gate-stable visual-authority checkpoint — COMPLETE IN SOURCE 0.9.4
- Fold the M2D091A waterfall/cave cliff-gate acceptance correction into a direct 0.9.2 -> 0.9.4 governed transition.
- Make M2D091 Rust-source semantic assertions whitespace-insensitive so `cargo fmt` cannot invalidate the Python pre-Cargo gate.
- Carry the formatter-normalized M2D091 Rust files in the checkpoint so Full Gate should be idempotent rather than rewriting source on first run.
- Keep runtime/worldgen behavior unchanged while continuous cliffs remain the next gameplay implementation milestone.

### M2D092B — Shoreline owner-authority recovery — COMPLETE IN SOURCE 0.9.6
- Recover October 6 shoreline work onto the authoritative 0.9.5 local baseline rather than depending on inaccessible chats.
- Promote Summer runtime profile v8 and derive all eight active material-boundary families from Native `shared_8_neighbor_owner_side` owner/neighbour metadata.
- Correct the reversed WetSand/Sand, ShallowWater/Sand, and DeepWater/ShallowWater outer/inner source roles.
- Certify the exact 12-role pale-shore ring as addressable source authority without automatically stamping it into every beach.
- Smooth deterministic island-edge noise across coarse lattice boundaries so outer coast/sand generation cannot expose 5x5 square hash cells.
- Advance Generated World to generator v6 to invalidate bad local generated-chunk caches while preserving M2D092A shared cliff topology, settlement semantics, and waterfall/cave gates.
- Add M2D092B source-rectangle/owner-side regression coverage to Static Audit and Full Gate.

### M2D092C — Unified Generated World compositor / topology behavior / modular bridges — COMPLETE IN SOURCE 0.9.7

### M2D092C1 — Cargo-test compile repair / rustfmt-normalized handoff — COMPLETE IN SOURCE 0.9.8

The Windows Full Gate exposed one test-only compile defect after Cargo check passed: `world_doc.rs` referenced the public `COLLISION_MASK_SIDE` constant from its test module without importing it. Source 0.9.8 imports the constant explicitly, preserves the Rustfmt normalization already applied by the Full Gate, and strengthens the M2D092C static gate so this exact regression is detected before Cargo tests. No gameplay/worldgen semantics are changed.

### M2D092C2 — Cargo-test collision assertion type repair — COMPLETE IN SOURCE 0.9.9

The next Windows Full Gate proved the import repair and Cargo check, then exposed a second test-only compile defect: `CollisionMask32::blocked_count()` returns `u32` while `COLLISION_MASK_SIDE * COLLISION_MASK_SIDE` is `usize`. Source 0.9.9 casts the constant-area expression to `u32`, preserves the Rustfmt-normalized source written by the Windows gate, and extends the M2D092C static regression gate to catch this exact type mismatch before Cargo tests. Runtime behavior is unchanged.

### M2D093B — Editor surface convergence / authoritative layer rail / viewport cache — COMPLETE IN SOURCE 0.10.11
- Retire duplicate Scene/Layers, Chunk Manager and standalone Asset Authority editor surfaces; keep one canonical app/control surface per responsibility.
- Make the permanent right rail authoritative for layer visibility and edit destination; hidden authored/generated content does not render, and hidden authored objects are not picked. Shift-click isolates a layer.
- Restore the Generated World semantic material palette directly beside the Paint tool and replace ambiguous Paint/Eraser/Eyedropper rail glyphs with clear vector icons.
- Keep Terrain Studio as source/pattern/evidence authority, Generated World as seed/materialization/chunk diagnostics, and World Properties as detailed collision/elevation authoring.
- Cache source-sheet lookup and resolved dual-grid vertex visuals by world revision/pending edits to remove repeated per-frame resolver/hash-map work while preserving exact-source rendering.
- Add M2D093B source regression coverage to Static Audit and Full Gate. Windows visual/performance acceptance remains required.

### M2D093A — Flat terrain authority convergence / Terrain Studio foundation — COMPLETE IN SOURCE 0.10.10 (M2D093A9 canonical-template quarantine-fixture repair over M2D093A8/M2D093A7/M2D093A6/M2D093A5 source-slice authority)
- Restore `DeepWater -> RiverWater -> WetSand -> Sand -> Grass` generated coast without inventing a direct RiverWater/WetSand atlas family: use the exact 12-role pale-shore source family as the live visual bridge.
- Extend Spring/Summer/Autumn primary channels through the irregular island mask so intended rivers open into shallow ocean; coastal shoulders resolve through WetSand/Sand rather than inland bank materials.
- Reserve existing purpose-object footprints before natural population so ecology cannot blindly overlap houses, bridges and other generated structures.
- Replace tree-only UV inset hacks with Full Gate v2 immutable-alpha render-bound certification for 7 seasonal trees plus 2 promoted Grasslands rocks/stones.
- Promote Terrain Studio / Production Pattern Matrix as the primary live Summer terrain-authority surface; keep the old 16-mask Grass/Void recipe as a legacy diagnostic lab only.
- Replace the PCC dashboard's misleading `0/16` primary status with production recipe/family/shore/fill counts.
- Advance Generated World to v9; keep water animation evidence-gated/off and preserve source pixels.

### M2D092C3 — Coastline / visible layers / vegetation-culling recovery — COMPLETE IN SOURCE 0.10.0

- Remove the unsupported automatic `RiverWater -> WetSand` coastline adjacency that forced whole-cell fallback art around every generated island.
- Generate ocean coast only through certified exact-source `DeepWater -> RiverWater -> Sand -> Grass` handoffs; keep WetSand available for authored/detail use.
- Replace coarse per-bank-cell dirt selection with smooth clustered erosion and reject isolated one-cell MudBank shoreline hooks.
- Partition Generated World semantic terrain into visible Ground, Terrain Details and Water composition lanes; route lily/water details to Water, water-edge plants/wildflowers to Terrain Details and vegetation to Objects.
- Expand generated-object chunk queries by eight cells before full visual-bounds culling so overhanging vegetation/details do not flash at chunk boundaries during pan/zoom.
- Restrict minor-detail LOD to the absolute far overview and preserve the existing far-terrain/collision-overlay performance lanes.
- Advance Generated World to generator v8 so stale v7 chunks rebuild under the corrected coast, bank and layer-routing rules.
- Preserve source pixels, modular bridge logic, topology-keyed terrain behavior and cliff/waterfall/cave gates.


- Promote the seven existing Scene-v2 layer IDs into real Generated World placement visibility and render ordering rather than informational UI only.
- Sort generated peers by ground/depth foot inside their layer and cull/select against complete visual source-part bounds, allowing behind/in-front overlap such as forageables beneath tree canopies.
- Persist Generated World layer visibility in editor layout and expose functional layer toggles in the Layers panel.
- Replace raw source-rectangle reusable terrain collision/traversal with semantic role + cardinal topology profiles; preserve local coordinate overrides and keep legacy source profiles serialized but inactive for generated terrain.
- Reduce collision-overlay cost with far suppression, medium coarse cells and horizontal run merging for exact 32x32 masks.
- Add stable minor-detail LOD hysteresis and a far-zoom exact-source cell rendering path so Fit World does not invoke the full dual-grid resolver across every visible vertex every frame.
- Replace the fixed 3x2 Wood Bridge A stamp with exact 32x64 left/middle/right source slices; measure land-to-land road/water span, repeat the middle and mark the generated bridge footprint walkable over water.
- Advance Generated World to v7 and runtime exact-region authority to 2,892; add M2D092C regression coverage to Static Audit and Full Gate.
- Keep bridge rail/deck sub-layer splitting, exact large-tree render-bound repair, continuous cliff art, waterfalls and caves evidence-gated rather than inventing pixels or claiming screenshot acceptance early.

### M2D092A — Shared cliff topology + settlement semantic repair — COMPLETE IN SOURCE 0.9.5
- Repair the 0.9.4 radius-two acceptance failure by preserving class-correct generated house labels, including `Worldgen town house`.
- Search deterministic nearby buildable parcels for large settlement structures instead of silently dropping houses when their nominal center intersects water/road terrain.
- Advance Generated World to generator v5 so existing generated chunks rebuild under the corrected settlement semantics.
- Persist one shared four-bit N/E/S/W cliff mask per generated cell, derived only from cardinal elevation drops.
- Expose the shared cliff mask in Generated World inspection while keeping exact cliff source-art roles, waterfall placement and cave entrances gated until the continuous cliff renderer consumes this topology.

### M2D119 — World PIE house entrance / room scene / cave preview (source staged; native verification outstanding)
- F near a reachable house entrance opens a separate 2×2 chunk room scene, with floor/derived-wall collision, LPC player display, and return via an exit portal to the original world position.
- A source-controlled `content/rooms/starter_house.room.json` acts as the initial **authored room data** and is validated/loaded fail-closed before use.
- F8 in World PIE opens an explicitly diagnostic deterministic cave scene; real world cave entrances still depend on source-backed cliff scene/rules and are not claimed complete.
- Room visuals use diagnostic geometry only until a reviewed ElizaWy floor/wall material binding is authored and accepted.
- Still pending: Studio room floor/wall painting, true house door visual/placement, certified cliffs and waterfalls, live cave entrances, multiplayer scene instances, complete character composition and runtime GUI acceptance.

## M2D120 Room Canvas (0.10.38, Windows certification pending)
- [x] In-Studio house room canvas, direct floor painting, bounded 2x2-3x3 resize and automatic wall-neighbor masks, selectable exit tool and wall tint
- [x] Room stroke undo/redo, atomic save and World PIE file reuse
- [x] Stone visibility count + layer/LOD diagnostics without inventing sprite identity
- [ ] Windows Cargo/Full Gate and Vulkan UI validation
- [ ] Certify original ElizaWy floor/wall/door art and bind it to the live room renderer
- [ ] Author source-exact cliff routes, waterfall rules and valid generated cave entrances
- [ ] Verify rocks display correctly after regeneration with full hydrated assets
