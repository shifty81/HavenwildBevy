# PCC 1.3.1 — complete asset preflight

This correction closes a first-deployment gate hole exposed by a partially hydrated ElizaWy tree.

## Problem

PCC 1.3.0 used normal `assets status` for its early hydration preflight. That command proves the active runtime sheets but intentionally does not require every core-manifest path. A tree with the four active Summer sheets and 321/349 core files therefore passed the early preflight and only failed at the late full-manifest verification.

## Correction

- `ForgePY.py assets status --complete` now requires every checked-in core-manifest path to exist without hashing all files.
- Full Gate runs this complete-manifest preflight immediately after governed update intake.
- Any incomplete tree automatically enters the existing pinned-GitHub-first hydration/repair lane.
- Whether repair ran or not, Full Gate immediately performs `assets status --full` and requires full identity verification before source/Cargo certification continues.
- The old late duplicate full-asset phase is removed.
- The hydration policy self-test now reproduces the partial-manifest condition and proves it fails closed until the missing files are restored and verified.

The GitHub-first authority, pinned ElizaWy commit, exact binary hashes, and LF/CRLF-only compatibility for historical `Credits.txt` manifest entries remain unchanged.
