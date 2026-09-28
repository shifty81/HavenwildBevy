# Havenwild Bevy v0.5 — Summer Mapping Authority Recovery

The standalone Bevy lane must not equate `DG recipe 0/16` with `Summer mapping 0`. Those are different layers.

## Recovered now

`content/terrain/recovery/summer_source_cells.v1.json` replays the canonical `Terrain/terrain_summer.png` at 32×32 cell granularity and restores the complete source-cell inventory:

- sheet geometry: 512×832;
- grid: 16×26 / 416 cells;
- 305 non-transparent cells, all represented as `source_mapped` evidence;
- 111 transparent cells;
- canonical source SHA-256 retained in metadata.

Historical build evidence says 14 of those 111 transparent cells were structural and 97 unused. The standalone source does not contain the old identity list, so v0.5 deliberately does **not** guess which transparent cells belong to either group.

## Historical evidence retained separately

`content/terrain/recovery/historical_authority.v1.json` records prior verified project milestones without silently treating them as current recipe certification:

- 305/305 Summer non-transparent cells mapped;
- 20 topology families across five seasonal sheets;
- 176 ordered-pair transition variants;
- 144 outer variants + 32 inner corners seam-conformant;
- historical mapped-terrain snapshot with 5,617 exact tuples + 57 pure fills;
- B48R15 Summer mapper evidence with 641 Summer crosswalk hints and 1,781 seasonal source addresses;
- later separate terrain-v7 authority with 34 materials and 15,562 exact tuple signatures.

The B48R15 crosswalk was explicitly source-address evidence rather than blanket semantic cliff/water approval. That distinction remains locked.

## Recovering raw historical metadata locally

Use:

```bat
PCC.cmd terrain recover
```

or:

```bat
PCC.cmd terrain recover --source "C:\path\to\older\Havenwild"
```

`tools/recover_legacy_mapping.py` searches only source/metadata lanes for known Summer/topology/tuple artifacts, hashes them, and copies local evidence under `.forgepy/recovered_mapping/`. `.forgepy` is ignored and source-only packaging excludes it.

Recovered donor metadata is evidence only until normalized/promoted into the current recipe schema.

## Confidence vocabulary

Current terrain mapping work uses these distinct states:

`certified` → `reviewed` → `source_mapped` → `candidate` → `ambiguous` → `unsupported` → `unmapped`.

No historical count or filename automatically promotes a new DG recipe to `certified`.
