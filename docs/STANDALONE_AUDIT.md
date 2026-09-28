# Standalone source audit — 2026-09-22

## Executive result

The supplied base is suitable to become an independent Bevy project. Its runtime code is already small and isolated, but the original archive mixed a tiny source scaffold with ~300 MB of locally useful asset payload and did not contain the internal ForgePY project-control path requested for the new lane. This split keeps the source authority and mapping decisions while making assets a validated ForgePY-managed local dependency.

## Audited incoming source

- ZIP integrity: clean; all multipart archive data reconstructed without compressed-data errors.
- Top-level footprint: ~299 MB `assets`, ~28 MB `content`, ~3.2 MB `reference`, with only two Rust source files.
- Canonical active source inventory: 64,340 entries total: 349 Terrain/Structure/Objects/FX files plus 63,991 character entries.
- Canonical non-character source payload: ~4.33 MB.
- Character archive: ~299.25 MB and therefore the overwhelming majority of the handoff size.
- Alternative source family: 2,630 entries, including 1,959 same-path/different-byte collisions. It must remain isolated.
- Initial authored scene: 40x28 / 1,120 cells, all still marked `draft`; all current placements reference `Terrain/terrain_summer.png`.
- Runtime-bound source sheets: only four (`terrain_summer`, `cliff_summer`, `Waterfall`, `plants_summer`). The rest of the collection is present for future mapping but not yet surfaced by the UI.

## What was structurally good

- No live dependency on the old Havenwild repo.
- One Bevy app and one egui/Bevy camera path rather than competing renderer lanes.
- Original source art is read-only by design; edits persist to authored JSON scene data.
- ForgeGUI_Core is revision-pinned rather than floating.
- Bevy 0.19 and bevy_egui 0.42 are the intended matching generation.
- Asset provenance and alternate-source collision rules are documented.
- The mapping loop already has source-rect selection, drag/click placement, basic undo, and save.

## Gaps found in the incoming base

| Area | Finding | Action in standalone split |
|---|---|---|
| Project control | `Studio.cmd` directly called Cargo/Python; no internal ForgePY | Added root ForgePY command system + Windows launcher |
| Asset ownership | Large art payload was bundled into every source handoff | Art is now hydrated locally from the authoritative source and git-ignored |
| Asset verification | Runtime only checked PNG headers/dimensions | ForgePY verifies canonical hashes before copy/extract |
| Catalog | 27 MB full source index was bundled even though it is derived | Compact 349-file core manifest stays tracked; full local index is regenerated |
| Dependencies | First build relied on manual Cargo behavior | ForgePY owns `cargo fetch`, build/check/run and live logs |
| ForgeGUI visuals | Creator visuals/icon font were never applied | Bevy host now calls `apply_creator_visuals` once |
| Borderless window | Title controls existed, but resize/snap helpers were unused | Added ForgeGUI resize handles + snap state integration |
| Editing history | Undo existed but redo did not | Added bounded redo stack |
| Missing assets | Runtime could silently load a partial four-sheet set | Required initial sheets now fail with a direct ForgePY hydration instruction |
| Character payload | 64k entries were carried in the source package | Character archive/hydration is optional and local |
| Build proof | Incoming archive had no Cargo.lock and had not been compiled in the audit environment | Remains a Windows acceptance item; ForgePY makes the first native gate explicit |
| Tests | No Rust unit/integration tests | Still open; first tests should target scene round-trip, asset manifest checks, and mapping commands |
| Runtime/game | Mapping studio only; no actual game loop/world simulation | Still open and intentionally separated from baseline stabilization |

## Recommended next implementation sequence

1. Native Windows `ForgePY.cmd full` and DX12 launch; commit `Cargo.lock` only after a successful build.
2. Verify exact Atlas -> world drag -> Undo -> Redo -> Save -> relaunch persistence.
3. Convert the hard-coded four-sheet Atlas list into a lazy source browser backed by the ForgePY-generated local catalog.
4. Introduce a typed source pointer/assembly document shared by editor and future client; do not encode gameplay meaning directly into PNG coordinates.
5. Map Summer terrain families and transitions source-exactly before worldgen consumes them.
6. Add a shared renderer/data crate before introducing a separate game executable, so Studio and client resolve the same scene/recipe data.
7. Add collision/elevation/navigation only after source assemblies are explicit.
8. Build finite seeded archipelago/world descriptor and authored Willowmere/Estate semantics on top of the shared contracts.

## Do not carry forward

Do not re-import the old PCC gate stack, historical Bevy validation probes, old semantic fake-grid render paths, arbitrary GREEN receipts, stale source-lock rules, or unrelated Havenwild editor dependencies. Port useful donor behavior deliberately and file-by-file only after this standalone baseline is buildable.
