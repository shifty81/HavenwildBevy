# PCC 1.3.4 — Retry-safe deletion staging

PCC 1.3.3 correctly isolated Git stderr from NUL-delimited filename streams, but a retry after a partially completed publish could still fail if a deleted file had already been staged. The deleted path was still part of the certified `HEAD`-to-worktree change set but no longer existed in either the worktree or index, so re-running `git add -A -- <deleted-path>` could return a fatal pathspec error.

PCC 1.3.4 stages only current unstaged/untracked certified paths, preserves already-staged deletions, and then verifies that the complete staged path set exactly equals the certified change set before commit. The Git workflow self-test now includes a partially staged deletion retry fixture.
