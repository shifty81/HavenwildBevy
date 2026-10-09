# Havenwild M2D108 — Fast PCC startup (0.10.28 → 0.10.29)

## Root cause and fix

The persistent PCC menu previously ran `git_publish_readiness` on every redraw. This recursively inspected the working tree, computed a full publishable-source SHA-256 snapshot, ran binary Git diffs and examined untracked paths merely to label option 2 READY/BLOCKED. The dashboard also ran `git status --porcelain` on each menu redraw. On a machine with 63,991 hydrated character assets, large build outputs and Git state, a menu could appear to hang before displaying a prompt.

The normal menu now uses last-gate receipt status only (`VERIFY ON SELECTION` / `FULL GATE REQUIRED`) and a cheap branch lookup. Expensive source certification and Git publish checks run **only** for the actual gate/publish operations; the label deliberately does not claim READY without checking.

`source_snapshot_fingerprint` now prunes excluded root directories (`assets` except README, `.git`, `.pcc`, `.forgepy`, `target`, `artifacts`, etc.) before traversing, rather than traversing each excluded tree and discarding all the entries. Its eligible file order, hashes, and v1 digest remain unchanged, verified by a baseline-algorithm regression.

## Safety boundaries

- No deletion or relocation of assets, hydrated character library, Git objects, build cache, saves or world documents.
- Full Gate continues to use `git_worktree_state()` and a source byte fingerprint.
- Commit/Push continues to use `git_publish_readiness()` and its existing fail-closed certificate policy.
- The dashboard now says *changes checked on publish*, never falsely `Clean` when it has not checked.
- `tools/m2d108_pcc_fast_start_selftest.py` is included in Full Gate and static audit.
- The apparent post-Cargo-build delay in older logs requires separate child-process investigation; M2D108 targets menu startup and redraw, not every possible build hang.

## Local acceptance

1. Apply the patch matching your installed source version (or use cumulative 0.10.26 → 0.10.29 package if M2D107 was not installed).
2. Reopen `PCC.cmd`; measure time until `>` prompt, and after returning from a submenu.
3. Use Full Gate before Git publication. Option 2 computes readiness fully at selection time.
4. Send exact timings and the PCC session log if boot still stalls before the initial session banner, because that is a different stage (Python launch / logs pruning / OS I/O).
