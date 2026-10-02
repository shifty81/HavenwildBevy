# M2D07S — Direct-source worldgen correction repair

Source 0.7.3 → 0.7.4.

## Why this repair exists

M2D07R made Paint teach the generator, but the first rule key was too permissive: a terrain region that shared a broad semantic role could be learned for a different N/E/S/W topology. One incorrect Grass edge choice could therefore replace thousands of Grass interior cells and make the whole generated world repeat the same source region.

This pass fixes that at the authority boundary rather than hiding the symptom.

## Changes

- Generator learning now accepts a replacement only when the exact canonical ElizaWy region is already observed in the checked-in topology palette for the same semantic role and N/E/S/W mask.
- Unsafe existing learned rules are removed on startup and materialized chunks are rebuilt from the remaining topology-safe source mappings. The repaired world stays dirty until explicitly saved.
- Legacy visual overrides are promoted into PCG knowledge only when they are topology-compatible; incompatible edits remain local authored overrides.
- Learned corrections refresh the materialized world immediately after Paint instead of appearing to do nothing until a later regeneration.
- Shift+Paint remains the explicit location-only escape hatch.
- The duplicated Select/Paint/Erase/Pick controls were removed from the top command bar. The permanent left tool rail is now the single direct-canvas tool selector.
- Generated object presentation keeps terrain batching, but trees/rocks/other larger object regions return to direct per-object source-region draws with physical-pixel destination snapping. This isolates exact source alpha edges from the terrain mesh batch path.
- PCC Asset Operations gains **Restore exact ElizaWy source from pinned upstream**. It downloads the non-character canonical source set from pinned ElizaWy/LPC commit `f07f7f5892e67c932c68f70bb04472f2c64e46bc`, verifies every file against the checked-in byte count and SHA-256, and only then replaces the local hydrated copy.

## Source authority finding

The previous 0.7.3 patch did not contain any `assets/elizawy` PNG payloads and therefore could not modify the hydrated source images. The checked-in manifest identities for `Terrain/terrain_summer.png`, `Terrain/trees_summer.png`, and `Terrain/Rocks, Grasslands.png` also match the pinned upstream repository byte-for-byte. The visible repeated terrain was a generator-learning rule error, not generated replacement artwork.

Runtime world composition may generate *placements and topology decisions*, but source art remains direct canonical ElizaWy repository pixels. No replacement PNG is synthesized by this pass.
