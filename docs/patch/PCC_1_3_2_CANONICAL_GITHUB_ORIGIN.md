# PCC 1.3.2 — canonical GitHub origin and self-healing publish

This pass closes the final fresh-deployment source-control gap.

## Canonical repository

- GitHub: `https://github.com/shifty81/HavenwildBevy.git`
- Branch: `main`
- Existing remote history is preserved.

## Primary workflow

The intended operator path is now truly `1 Full Gate -> 2 Commit + Push -> 3 Patch Scan/Review/Apply`.

Full Gate records a publishable-source SHA-256 fingerprint that is independent of `.git` history/configuration. Therefore option 2 may initialize local Git, configure the canonical origin, or adopt existing remote history after certification without invalidating the gate as long as no publishable source bytes changed.

When an extracted deployment already has a PCC-created local root commit but `origin/main` contains older unrelated history, PCC offers the safe adoption path: `git reset --mixed origin/main`. This moves HEAD/index onto canonical history while leaving all current working files untouched. The current source then becomes ordinary local changes which PCC stages, commits, and pushes only after certification.

PCC still refuses automatic merge/rebase/stash/hard reset/force-push, behind/diverged remotes, detached HEADs, local runtime state, hydrated assets, patch archives, and any source edit made after certification.

A root `.gitattributes` now normalizes repository text line endings to stop Windows `LF will be replaced by CRLF` warning floods while keeping `.cmd`/`.ps1` CRLF and binary assets binary.
