# M2D122 — Original Asset → Renderer Integration, measurable baseline

**Source:** 0.10.39 → 0.10.40. **Priority:** asset consumption, not more editor windows.

## Important boundary

Havenwild's `content/catalog/core_source_manifest.json` lists **320 original PNGs**. These are all discoverable from the Library, and original sheets may be loaded on demand. That is **not** a claim that they have authored pixel-region selections, object templates, gameplay meanings, animations, or certified render behavior.

The `content/assets/authority/elizawy_runtime_index.v1.json` has **2,892 canonical exact source-region records on 30 distinct PNGs**, and **28 reviewed object templates on 13 distinct PNGs**. Only 30/320 source PNGs have at least one enumerated runtime source rectangle. The remaining **290** need source-region authoring/review (or should be explicitly marked not intended for tile/object use). Count groups:

| Collection | Original PNG sheets | Sheets with canonical regions | Sheets with reviewed object templates | Without canonical regions |
| --- | ---: | ---: | ---: | ---: |
| Terrain | 29 | 23 | 6 | 6 |
| Objects | 188 | 3 | 3 | 185 |
| Structure | 98 | 4 | 4 | 94 |
| FX | 5 | 0 | 0 | 5 |
| **Total** | **320** | **30** | **13** | **290** |

Of the 2,892 regions, many come from historical/source evidence and **do not imply semantic approval**. A 32-pixel atlas slice is not automatically a full tree, a door, a stone, a ground material, or a waterfall. Actual PNG alpha, complete sprite dimensions, collision, feet/anchors, animated frames, and contextual use still need visual source review. LPC Characters are an optional, separately hydrated library and **are not part of the 320 core PNG denominator**.

## Implemented in M2D122

1. `src/render_asset_sync.rs` collects exact source dependencies of **all materialized generated ground cells, existing generated objects, authored objects/bridges, and explicit source overrides**. Only paths actually referenced in the world are requested; the entire 320-PNG catalog is not preloaded.
2. `hydrate_pending_source` invalidates the render-dependency plan on `WorldDocument.revision` change (including world regeneration). At most **four new PNG handles/frame** are handed to Bevy's asynchronous AssetServer. A failed/missing source is recorded rather than silently accepted. This closes the startup-only texture registration gap for newly regenerated stone/object sheets; the separate worldgen frequency and LOD filters remain in effect.
3. World Inspector now shows **registered, awaiting GPU/egui texture, queued and failed** source sheet counts, failed paths, and a user-triggered rescan. Loading a source is **not** evidence that all world objects become visible; collision, alpha crop, LOD and gameplay placement still require visual inspection.
4. Bottom Source/Assets Library shows **registered canonical region count for the selected original sheet**. Catalog-only sheets are flagged. Single-part, 32px-aligned reviewed templates can be selected using the **entire original source rectangle** (e.g. 32×64 barrel), rather than the top-left 32×32 cell. No guessed semantic role or compositing is introduced.
5. `tools/audit_asset_render_coverage.py --check` emits `artifacts/asset_lane/render_coverage.v1.json` (ignored by source packages) containing an actionable path-by-path ledger, family coverage, original image identity checks, canonical rectangles and template membership. `--verify-hydrated` optionally SHA-256 verifies all original PNGs on a hydrated workstation. Added PCC Full Gate phases and source-contract regressions to keep coverage claims honest.

## Remaining project-wide asset/render work, in implementation order

1. **Priority A: exact source region inspection for the 290 catalog-only PNGs.** Build a background original-alpha analyzer (immutable, provenance tagged) to propose *candidate* logical sprites and nonuniform grid/crop boundaries; native editor approval writes region records to a separately reviewed authority. Never mass-promote random 32×32 slices to complete assets.
2. **Priority B: object and structure authoring.** Author complete tree/stone/prop/door/house pieces, anchors, footprints, occlusion/layers, animations, collisions and traversal; enable normal World object placement for reviewed complete sprites, not just Terrain 32px overrides. Implement unit/collision/PIE tests and screenshot diffs for each family.
3. **Priority C: terrain topology.** Correct currently missing/mismapped live Jaidyn Tiled/native-Wang coastline tuples (including material transitions, corner masks and all-season counterpart validation), then review cliff faces, corners, elevation intersections, cave portals, water contacts and waterfall assets. `305/305` Summer source recovered **does not** mean fully certified coast/cliff rules.
4. **Priority D: indoor materials and features.** Map floor/wall/door sprites into per-house room documents. Replace the provisional colored collision/geometry room preview only after source-backed floor/wall auto-tiling and PIE render parity.
5. **Priority E: FX and character layers.** Review animation frame bounds, durations, compositing, directional sprites, feet, tool-use and clothing; ensure runtime, PIE and Studio all consume the *same* source identities.
6. **Priority F: comprehensive certification.** Each family needs reference atlases, complete source crop/animation identity, scene exercise fixture, render-debug screenshot, editor/PIE parity, performance, licensing/attribution and deterministic regression checks. A catalog count alone cannot satisfy this.

## Windows acceptance

Run PCC Full Gate from **0.10.40**. Rebuild/re-generate an area with natural stone placements. Inspector → **Rescan world asset bindings**, verify queued/failed counts return to zero after Bevy loads the images, and verify stones actually appear at expected world/LOD layers. Inspect the generated `artifacts/asset_lane/render_coverage.v1.json` to see which source PNGs are still catalog-only.

**Verification scope:** source contract + PCC patch simulation + Python audit; native Cargo compilation, hydrated original-PNG alpha, and runtime visual acceptance are required on the Windows workstation.
