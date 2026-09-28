# DG-01 — ElizaWy Terrain Recipe Mapper

This checkpoint turns the dual-grid plan into an authoring workflow inside the standalone Bevy Studio.

## What is authoritative

The saved world remains semantic. Terrain recipes are metadata that map semantic topology to exact hydrated ElizaWy source regions. Source PNG bytes remain immutable and local-only.

The first recipe family is `Summer / Grass against Void`. It intentionally begins uncertified. The editor must classify every mask as one of:

- `sprite`: one exact source rectangle.
- `composite`: one or more source rectangles with pixel offsets.
- `unsupported`: the source family cannot represent that topology and the resolver must fail closed or use an authored higher-level rule.
- `unmapped`: not reviewed yet.

## Bevy authoring workflow

Open **Atlas + Terrain Recipe Mapper** in Studio.

1. Select the exact hydrated source sheet and a 32×32 source cell.
2. Select one of the 16 dual-grid masks.
3. Compare the corner occupancy preview to the source artwork.
4. Assign the source as a single sprite, add it as a composite part, or mark the topology unsupported.
5. Use `Next unresolved` until every mask has an explicit classification.
6. Save recipe metadata.
7. Run `PCC.cmd terrain fixtures` to verify topology fixtures.
8. Run `PCC.cmd terrain certify` only when all 16 states have been reviewed.

No source coordinates are promoted automatically from the old 40×28 scene.

## Fixture policy

Checked-in semantic fixtures now exercise all 16 masks. The original irregular reference shape remains a regression target, with focused diagonal and three-corner fixtures closing the mask-coverage gaps.

Fixture coverage proves the topology test set can exercise a state. It does **not** certify that an ElizaWy source mapping is visually correct.

## Next DG-01 work

After the native build is GREEN and the mapper is usable:

- add assigned-sprite preview at authored scale;
- add source-atlas zoom without changing source coordinates;
- add recipe undo/redo independent of scene-tile undo;
- add explicit review state (`draft`, `reviewed`, `approved`);
- add composite-part reorder/removal and per-part layer;
- add a deterministic 16-state rendered certification board using actual mapped source art.

## v0.5.2 debug-ready additions

DG-02 now sits directly beside DG-01 instead of replacing it. The mapper reads the canonical core manifest and exposes all hydrated `Terrain/` images, previews the actual resolved recipe source, and supports per-part composite locate/replace/offset/remove operations. A persisted semantic-only 5x5 lab covers all 16 topology masks and can drive the currently selected recipe mask.

Recipe classification and recipe certification are now separate. Any sprite/composite edit starts or returns the mask to `candidate`; the user can promote it through `reviewed` to `certified`. `PCC.cmd terrain certify` requires all 16 states to be explicitly classified **and** certified.

See `DG02_RESOLVER_LAB_V052.md`.
