# M2D05-A — World Authoring Foundation / Recovered ElizaWy Evidence

Source baseline: user-supplied `Havenwild_Bevy_Standalone_SOURCE_ONLY_20260929.zip`, source 0.5.6, PCC 1.2.6, ForgePY 0.4.6. Target source 0.5.7. This is a cumulative incremental PCC root-drop update **from that exact source revision**, not a replacement of the hydrated asset tree or local scene drafts.

## Live changes in this pass

* Resolve project root from explicit `HAVENWILD_BEVY_ROOT`, active working directory, or executable ancestor before a valid compile-time fallback. Source Evidence no longer assumes a previous `Havenwild Latest` directory; optional workstation M2C4 registry absence is a warning, not a hard blocker.
* Ship B48R9 recovery as project-versioned **read-only evidence**: 40 original source-sheet identities, 1,781 exact historical source addresses, 2,549 season/target comparisons, 284 grouped review cases (787 unresolved comparisons), historical approvals scoped in a separate ledger; none silently certified or promoted. Search all regions and compare candidate target atlas addresses in Source Evidence. M2C4 derivative registry remains optional/independent when present.
* Add lightweight visible chunk boundaries and coordinates to the canvas and status. Default 32x32 source cells; display selections 16/32/64. Negative global coordinates use Euclidean floor division. The original Summer map is still the **single 40x28 scene fixture**; this overlay does not generate/load/save multiple chunks or stream the world.
* Preserve the scene's independent sparse structural channel: expose selected-cell traversal Unknown/Walkable/Blocked; optional H0..H30; semantic hint and explicit connector label; overlay H/collision borders; scene-v2 atomic save and transaction undo/redo. PIE freezes and consults explicit traversal override before provisional source hints. The structural editor does not auto-infer collision from pixels or certify slope navigation, collision footprints or water physics.
* Explain the current `Terrain Rules` tab in the app. Its 16 four-corner Grass/Void recipes are distinct from 305/305 Summer source-region recovery and distinct from actual game collision/height. No automatically certified recipes were added.
* Group the narrow left rail into compact EDIT and DATA sections; preserve all existing buttons/workflows, the native window, on-demand panels and original-source canvas.
* PCC static and Full quality gates verify consolidated history without promoting it; all existing gates continue to run.

## Preserved source of truth

Protected original terrain, original ElizaWy pixels, 36 individually placed Summer instances, previous local River drafts and `.forgepy` mappings/drafts. Never claim M2C4 was reconstructed solely from the B48R9 crosswalk. No source PNGs included in handoff. No `assets/`, `.forgepy/`, `.pcc/` or output folders in the patch.

## Manual acceptance after Windows PCC Full Gate

1. Full Gate completes with receipts; if failed send automatically generated debug bundle. Runtime root resolves the installed project, not an earlier checkout.
2. Open Studio. The Summer map, house and trees persist, existing save/undo/play controls remain. View > Canvas grids toggles visible subtle chunk lines and optional per-tile grid. Changing 32 to 16/64 changes overlay only. Selection status reports chunk and local cell.
3. SRC > Source Evidence displays bundled 1,781 source-region browser, with search + review group tab, even when optional `.forgepy/recovered_mapping/elizawy_tiled_review_registry.v1.json` is absent. No new recipe certification is represented.
4. SEL a ground cell, COL, author an explicit blocked flag and height; enable overlay, Save Scene, close/reopen, verify persisted. Undo/Redo changes are reversible; original PNG unchanged. PIE freezes and honors the explicit flag for movement. Check with empty and existing local drafts, do not overwrite earlier River drafts.
5. Terrain Rules provides its in-panel explanation, retains existing 16-mask interaction.

## Deliberately deferred (separate later certified passes)

Actual global chunk descriptor, topology baking/streaming, procedural world editor scene loading; semantic multi-cell Tiled-style brushes and editable generated autotile layers; complete collision object footprints/water rules/ramp corridors and connector locomotion; true dockable Pixel Studio / Animation Studio with authored PNG creation/export; exhaustive donor-editor capability port. See the architecture document for contracts and port order. Offline checks are NOT Windows compile, PNG SHA runtime validation, or interactive visual acceptance.
