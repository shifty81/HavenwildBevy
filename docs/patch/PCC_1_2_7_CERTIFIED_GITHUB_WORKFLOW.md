# PCC 1.2.7 — Certified GitHub workflow

Baseline: Havenwild Bevy source `0.8.1`, ForgePY `0.4.6`, PCC `1.2.6`.
Target: Havenwild Bevy source `0.8.1`, ForgePY `0.4.6`, PCC `1.2.7`.

## Root operator workflow

The PCC root menu is now centered on the normal repository lifecycle:

1. Full Quality Gate / certify current source.
2. Commit + push the exact certified source state to GitHub.
3. Scan, review, and apply governed patches.

Secondary operations are grouped into Studio/Build/Run, Project Health, Assets, Terrain/Mapping Authority, Source Control/GitHub, Diagnostics, and Developer/Maintenance menus.

## Certified Git publish contract

A successful Full Gate stores Git branch, HEAD, origin, changed paths, and a deterministic worktree fingerprint in the gate receipt. PCC publish is blocked unless that exact state is still current.

Publish performs a fetch before mutation and refuses detached HEAD, missing origin, forbidden local/runtime payloads, stale gate fingerprints, remote-behind state, or branch divergence. It never automatically stashes, resets, merges, or rebases.

Only source-safe changed paths are staged. Hydrated assets, `.pcc`, `.forgepy`, build output, reference trees, consumed patch state, generated local catalogs, and root handoff/archive files are excluded from publication authority.

If commit succeeds but push fails, PCC records a `COMMITTED_PENDING_PUSH` state so the exact certified commit can be pushed again without rerunning Full Gate. Once the commit is confirmed at origin, the publish state is recorded as `PASS`.

## Regression coverage

`tools/pcc_git_workflow_selftest.py` creates an isolated temporary repository and local bare remote. It validates certified commit/push, post-gate edit invalidation, root handoff exclusion, and fail-closed behavior when origin advances. The test runs in both Static Audit and Full Gate.
