# Havenwild local asset hydration

**Default authority order:** when core assets are missing, Havenwild first mirrors the exact pinned `ElizaWy/LPC` GitHub revision (`f07f7f5892e67c932c68f70bb04472f2c64e46bc`) and verifies every installed file against the checked-in byte count and SHA-256 manifest. Existing files that already match the manifest are not downloaded again. Remembered/local source trees and pack staging folders are fallback-only unless the operator explicitly selects local-only recovery.

# Local asset workspace

This source-only repository does not vendor the ElizaWy/LPC Revised art.

Normal fresh-source hydration requires no local path:

```bat
ForgePY.cmd assets sync
```

ForgePY first restores only missing/hash-invalid core files from the pinned
GitHub revision, validates every byte against the checked-in manifest, and
builds `content/catalog/source_index.local.json`. If GitHub is unavailable, a
local recovery source can be supplied explicitly:

```bat
ForgePY.cmd assets sync --source "D:\path\to\your\authoritative\asset source" --local-only
```

Optional archives such as Characters remain local/on-demand and are preserved
under `assets/optional/`.

The current mapper requires four source sheets immediately:

- `Terrain/terrain_summer.png`
- `Terrain/cliff_summer.png`
- `Terrain/Waterfall.png`
- `Terrain/plants_summer.png`

The full Terrain / Structure / Objects / FX family is synchronized when the
source is available. Characters remain an on-demand hydration step because the
collection is tens of thousands of files. The alternate four-season archive is
never automatically merged into the canonical source family.
