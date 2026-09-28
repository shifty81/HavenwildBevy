# Local asset workspace

This source-only repository does not vendor the ElizaWy/LPC Revised art.

Run:

```bat
ForgePY.cmd assets sync --source "D:\path\to\your\authoritative\asset source"
```

ForgePY validates known source hashes, copies immutable working files into
`assets/elizawy/`, preserves optional archives under `assets/optional/`, and
builds `content/catalog/source_index.local.json`.

The current mapper requires four source sheets immediately:

- `Terrain/terrain_summer.png`
- `Terrain/cliff_summer.png`
- `Terrain/Waterfall.png`
- `Terrain/plants_summer.png`

The full Terrain / Structure / Objects / FX family is synchronized when the
source is available. Characters remain an on-demand hydration step because the
collection is tens of thousands of files. The alternate four-season archive is
never automatically merged into the canonical source family.
