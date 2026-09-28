# Havenwild Bevy — M2C2+M2C3 cumulative ElizaWy evidence handoff

**2026-09-27 · additive evidence tooling only · no Bevy build or runtime changes · zero mappings promoted**

## What the supplied package actually contains

The user supplied the actual OpenGameArt derivative `lpc-revised-exterior-tilesets.zip` and five separate preview PNGs. This is a **rearranged third-party Tiled compilation**, not the original 16×26 ElizaWy atlas, and must be kept as a reference input rather than a replacement. In this upload the ZIP has 10 root members: 5 PNGs and 5 TSX files. Four terrain sheets plus one building sheet, each 2048×2048, 32px cells, 64 columns, 4096 address positions. There are no TMX/TMJ scene/map files or original source PNGs in this ZIP. The four individually supplied terrain PNGs are byte-for-byte identical to their corresponding ZIP members.

Original upstream fixed commit: `f07f7f5892e67c932c68f70bb04472f2c64e46bc` (ElizaWy/LPC). Third-party release: https://opengameart.org/content/lpc-revised-fully-configured-4-seasons-tilesets-for-tiled-map-editor . Full input ZIP SHA256: `f0236c16b272a7328dad66772e4f971513bc508cb69a56c60d736264a3a5b43a`. Per-asset SHA256 and precise reports: `docs/patch/evidence/M2C3_JAIDYNREIMAN_TILED_CONCORDANCE.v1.json`.

| TSX | Wang sets | Assignments | Animated tile definitions | Tile collision objectgroups |
|---|---:|---:|---:|---:|
| Summer | Terrain 331, Walls 24, Fences 60 | 415 | 325 | 1461 |
| Spring | Terrain 331, Walls 24, Fences 60 | 415 | 325 | 1461 |
| Autumn (filename **`-old.tsx`**) | Terrain 281, Walls 24; **no Fences set** | 305 | 214 | 1365 |
| Winter | Terrain 331, Walls 24, Fences 60 | 415 | 325 | 1461 |
| Buildings | Brick Walls 216, Flat Roofs 417, Angled Roof 54, Adobe Walls 45 | 732 | 0 | 1117 |

Spring and Summer numeric tile signatures match. Winter numeric signatures match too, but the third and eighth terrain color names change (`Dirt` to `Snow`, `Melted Ice` to `Ice`); therefore **same integer signature is not the same semantics**. Relative to Summer, the Autumn Terrain set has 58 Summer tile IDs absent, eight extra IDs and ten differing numeric Wang signatures on IDs present in both. It also lacks all 60 Summer fence assignments. The actual Autumn PNG exists; an incomplete/older TSX does not demonstrate missing artwork. Do not generate missing mappings automatically or label Autumn certified.

## Files in the cumulative patch

Includes the **four M2C2 files unchanged**: `tools/audit_elizawy_concordance.py`, `tools/inspect_elizawy_tiled.py`, `tools/elizawy_m2c2_selftest.py`, and `docs/patch/M2C2_ELIZAWY_CONCORDANCE.md`. Additionally includes:

- `tools/reconcile_elizawy_tiled.py`: read-only external ZIP analysis, per-season Wang signatures, missing/extra/changed assignments, image SHA and PNG dimensions, archival Summer map SHA check and optional original 32px pixel concordance. Strict mode checks the exact uploaded ZIP hash. Every external assignment remains evidence only, including matching pixels. No V7 tuples are imported.
- `tools/elizawy_m2c3_selftest.py`: synthetic metadata and optional original-archive tests, no network/build.
- `docs/patch/evidence/M2C3_JAIDYNREIMAN_TILED_CONCORDANCE.v1.json`: metadata-only proof from the actual supplied archive, without embedding third-party artwork or rehosting the TSX source.
- This document.

## Apply as one root-drop PCC ZIP

**Use only the new M2C2+M2C3 cumulative ZIP in the root/inbox.** Do not queue its earlier M2C2 ZIP at the same time: PCC correctly blocks multiple compatible patches against the same installed version. If M2C2 was already applied, the cumulative ZIP reuses the exact same four M2C2 file bytes, making the reapplication additive. PCC versions remain source `0.5.6`, ForgePY `0.4.6`, PCC `1.2.5`. No manual extraction or installer/repair script is required.

After applying, source-only tests:

```powershell
python tools\elizawy_m2c2_selftest.py
python tools\elizawy_m2c3_selftest.py
python tools\audit_elizawy_concordance.py
```

Keep the **original external ZIP** on the development PC wherever preferred; pass the path directly to avoid copying its licensed artwork into a root-drop source patch:

```powershell
python tools\inspect_elizawy_tiled.py "D:\Downloads\lpc-revised-exterior-tilesets.zip"
python tools\reconcile_elizawy_tiled.py "D:\Downloads\lpc-revised-exterior-tilesets.zip" --strict-archive-sha
```

Optional *original upstream* image-level concordance uses the already governed ForgePY hydration path. Only a hash-verified original LPC source tree is accepted, not the derivative ZIP:

```powershell
python tools\reconcile_elizawy_tiled.py "D:\Downloads\lpc-revised-exterior-tilesets.zip" --original-root "D:\Vault\Source\ElizaWy\LPC" --strict-archive-sha
```

Original PNG comparison requires optional Pillow (`PIL`). Without Pillow or verified original art, metadata inspection works and the report explicitly states why pixel proof was not run. This patch does not install Pillow, fetch art, or alter managed source libraries. Result: `.forgepy/recovered_mapping/external_tiled_concordance.v1.json`.

## Authority and future lane rules

1. Original ElizaWy and recovered 16×26 Summer / seasonal topology are authority candidates. The derivative 64×64 layout has **independent tile IDs**. Compare SHA-verified original 32px cells against rearranged derivative RGBA cells; record one-to-many and unmatched rather than assume atlas coordinates agree. A pixel match is address evidence, not semantic proof.
2. Review and explicitly approve role, animation, collisions and optional external Tiled metadata separately. Distinguish walkable natural rock floor from structural cliff wall. Keep genuine one-tier cliffs, height tiers 0–30, downhill drainage, required waterfall chains and traversal connectivity; never restore retired 2+ cliff-only rule from old crosswalk prose.
3. Preserve separate water / ice / climb / seasonal / buildings / roof/wall/fence registries. Autumn is flagged as an incomplete historical derivative metadata candidate until verified.
4. Source art and third-party derivative art/TSX stay outside patch; retain per-contributor credits and OGA-BY 3.0 lineage and review derivative terms before distribution.
5. Do **not** integrate V7 tuple IDs, generated V7 atlases, or V7 collision into ElizaWy's active resolver. V7 stays a quarantined donor/reference.
6. Do not claim Bevy runtime, GUI, Windows Full Gate or authoritative mapping certification from this tool-only patch. The next lane (M2C4/M2D) is reviewed mapping approval and Mapping Studio GUI integration after actual source-art proof.
