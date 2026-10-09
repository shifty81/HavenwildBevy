# Havenwild Bevy M2D109 — interactive Git publish verification fix

Source: 0.10.29 -> 0.10.30. PCC and ForgePY versions unchanged.

## Problem

Once Git was initialized/adopted, pressing primary PCC menu option 2 appeared to hang without output. This happened because `source_publish_interactive()` called `git_publish_readiness()` before printing its first status line. The readiness path enumerates untracked Git files. The existing `.gitignore` was effectively from another project, leaving Havenwild's generated `target/`, `.pcc/`, `.forgepy/`, archived ZIPs, and hydrated asset library exposed to the expensive untracked scan (potentially over 63,991 characters plus build files).

## Change

- Add Havenwild-specific root Git ignore patterns. Preserve `assets/README.md` as source.
- Keep the existing fail-closed `git_publish_path_error` restrictions even for files already tracked by Git. `.gitignore` does not override these restrictions or remove tracked files.
- Print *before* the first Git publish readiness call; emit stages for source hashing, branch/remote, tracked diff, and untracked discovery, with an eight-second heartbeat and completion time.
- Keep unchanged source-byte fingerprint, Git binary-diff semantics, remote-ahead prevention, explicit user confirmation, and Full Gate requirements.
- Add automated ignored-path and progress regression selftest and run it as part of Full Gate and audit gate.

## Installing

Drop `.pccpatch.zip` at Havenwild root and approve it in PCC. If an older PCC is currently stalled before a commit confirmation, Ctrl+C interrupts that query without committing or pushing; return to menu with Enter, or close/restart after confirming no Git child operation is still active.

The new `.gitignore` is source code and therefore changes the source fingerprint. **Run the Full Quality Gate after installing before attempting Commit + Push.** Then choose menu 2 again, observe progress stages, and approve an explicit commit message and push only when ready. Do not run `git reset --hard`, `git clean`, or delete hydrated characters.

If readiness still takes excessive time, note the last `[PUBLISH PREFLIGHT]` phase and upload `.pcc/logs/<session>.log`, plus `git status --short -uno` (quick tracked-only status). Avoid making another huge recursive directory scan merely to debug the stall.

## Verification boundary

Source-level regression and simulated PCC compatibility can run outside Windows. A live Windows runtime timing and actual GitHub push are **not** verified by this patch.
