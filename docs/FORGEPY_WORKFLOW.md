# ForgePY workflow

ForgePY is the local project-control entry point for this standalone Bevy lane. `Studio.cmd` launches the DX12 Studio path through PCC. `ForgePY.cmd` remains the lower-level backend launcher; `PCC.cmd` is the normal persistent operator surface.

## Core commands

```bat
ForgePY.cmd doctor
ForgePY.cmd setup --source "D:\Assets\ElizaWy"
ForgePY.cmd assets status
ForgePY.cmd assets status --full
ForgePY.cmd assets sync --source "D:\Assets\ElizaWy"
ForgePY.cmd assets hydrate-characters
ForgePY.cmd assets hydrate-characters --verify
ForgePY.cmd catalog
ForgePY.cmd fetch
ForgePY.cmd build
ForgePY.cmd run --backend dx12
ForgePY.cmd run --backend vulkan
ForgePY.cmd run --backend dx12 --native-frame
ForgePY.cmd full
ForgePY.cmd package
```

`full` performs doctor -> required asset hydration when needed -> Cargo fetch -> fmt check -> Cargo check -> Cargo tests -> build. PCC adds the stricter 349-file core-manifest hash gate before Cargo work. Subprocess output is streamed live and mirrored to `.forgepy/logs/`.

## Asset source discovery

The explicit `--source` path wins. ForgePY then remembers it in `.forgepy/local.json`. It can also use `HAVENWILD_ASSET_SOURCE` or a small set of sibling/intake locations, but only accepts a candidate after a known active source sheet validates.

Accepted layouts:

- complete prior source/project root containing `assets/elizawy/`;
- extracted canonical tree containing `Terrain/`, `Structure/`, `Objects/`, and `FX/`;
- a source pack folder containing `Terrain.zip`, `Structure.zip`, `Objects.zip`, and `FX.zip`.

The tracked `core_source_manifest.json` contains hashes for all 349 non-character canonical source files. Core sync copies/extracts and verifies those files. `Characters.zip` is retained as a local optional archive and hydrated only when requested. Character hydration reports progress and writes a separate metadata-only `characters_index.local.json`; normal PCC operations do not SHA-256 hash all 63,991 character files. Once hydration state is certified, repeat hydration commands return immediately; use `--verify` when an explicit full file-size recheck is wanted. `FourSeasonAlternative.zip` may be retained locally but is never merged automatically.

## Repository policy

Hydrated art, optional asset archives, reference material, generated local catalogs, and one-time seed-overlay markers are git-ignored. Source code, authored scene/mapping documents, the compact canonical manifests, PCC aliases, and ForgePY stay tracked. `ForgePY.cmd package` is permanently source-only for this lane; future handoffs do not include the asset payload.

## PCC / GitHub source refresh

`PCC.cmd` is the normal persistent Project Control Center launcher. `ProjectControlCenter.cmd` is a compatibility alias to the same PCC. `ForgePY.cmd` exposes the lower-level backend directly when needed.

When the standalone project is a real Git checkout with an `origin` remote, both `PCC.cmd build` and `PCC.cmd full` perform a safe source refresh before compiling:

1. inspect the worktree;
2. if source is modified, **do not** stash/reset/rebase or overwrite anything — refresh is skipped and the local tree builds as-is;
3. if clean, `git fetch --prune origin`;
4. compare the current branch to `origin/<current branch>`;
5. fast-forward only when the local branch is behind and not divergent;
6. refuse automatic merge/rebase when histories diverge.

Useful commands:

```bat
PCC.cmd source status
PCC.cmd source refresh
PCC.cmd build
PCC.cmd full
```

A source ZIP extracted into a normal folder is not automatically turned into a Git repository. GitHub refresh activates once that project folder is cloned/checked out from its eventual GitHub repository and has an `origin` remote. This avoids ForgePY inventing or overwriting repository history.

## Character hydration migration

If a pre-v0.2 ForgePY run already printed `CHARACTERS: hydrated 63991 files`, v0.2 adopts that completed local tree by file count, writes local hydration state, and builds the metadata catalog without re-hashing the files. Use `PCC.cmd assets hydrate-characters --verify` whenever an explicit full file-size verification pass is desired.
