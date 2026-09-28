# Havenwild Bevy v0.5.1 hardening pass

This pass hardens the v0.5.0 Summer-recovery/editor checkpoint without expanding the lane into unimplemented Havenwild gameplay systems.

## Persistence

Scene drafts, editor layout state, and terrain-recipe metadata now use one shared transactional writer. New bytes are staged beside the destination, the previous file is protected as a same-directory backup, and a failed promotion attempts to restore the previous file. This avoids relying on platform-specific overwrite behavior from `std::fs::rename(temp, existing)`.

## PCC / ForgePY

- PCC 1.2.1 rejects unsafe/reserved patch IDs before staging or rollback paths are created.
- A patch that replaces `ProjectControlCenter.py` causes FULL/build/menu update flows to restart into the updated controller before more work is executed.
- Every FULL phase receives its own phase ID and phase receipt; a phase only links a child-job receipt when that job was actually created by that phase.
- FULL now performs a 349-file core-manifest size/SHA-256 verification after hydration/repair and before Cargo work.
- Debug bundles include the Summer acceptance scene plus a sanitized local Terrain Mapper layout snapshot when available.
- The old `tools/package_source.py` entry point delegates to ForgePY so there is only one source-only packaging policy.

## Catalog cleanup

`content/catalog/source_index.json` and `content/catalog/source_collisions.json` are retired generated artifacts from the older complete-source handoff. The governed patch deletes them, `.gitignore` prevents their return, and source packaging excludes them even if they remain in an older checkout before the patch is applied.

## Still intentionally unresolved

The recovered 305/305 non-transparent Summer cells are source-address evidence, not the missing historical 641-crosswalk / topology / tuple authority. DG recipe certification remains independent and must not auto-promote historical counts into mappings.
