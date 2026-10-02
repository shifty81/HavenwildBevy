# PCC 1.2.8 — First-deployment bootstrap repair

This pass closes two source-only deployment gaps exposed by a fresh Havenwild Bevy extraction.

## Asset bootstrap

Full Gate now performs asset readiness immediately after governed patch intake. If the ElizaWy runtime tree is absent, PCC first attempts the remembered/auto-discovered ForgePY source. If no authoritative source is available, the gate stops immediately with the exact hydration route instead of spending the full gate and failing near the end.

The Assets / hydration menu now prompts for an authoritative source path and passes it to `ForgePY.py assets sync --source ...`. ForgePY continues to validate and remember the source under local `.forgepy` state; hydrated art is never source-packaged.

## Git/GitHub bootstrap

Source control / GitHub now includes a first-time bootstrap for an extracted source-only package. For a new/empty GitHub remote it creates a local Git repository and one source-safe baseline commit. If the selected remote branch already has history, PCC fetches it and anchors the new local branch to `origin/<branch>` with `git reset --mixed`; this updates HEAD/index but deliberately does not replace the extracted working files, so the current source remains visible as normal local changes for certification.

The baseline is local only. Normal publication remains: **1 Full Gate -> 2 Commit + Push certified source**.

A separate origin configuration action is also available for an existing checkout or a baseline initialized without a remote.
