# Havenwild Bevy PCC v1.1

PCC is the persistent operational supervisor. ForgePY remains the local project backend.

## FULL gate phases

`PCC.cmd full` now runs visible, separately receipted phases:

1. governed patch intake;
2. Python controller syntax check;
3. dual-grid contract + fixture coverage;
4. safe GitHub refresh when this folder is a clean checkout;
5. doctor;
6. asset status, with local repair/sync when needed;
7. Cargo fetch;
8. rustfmt check; if needed, governed `cargo fmt` normalization + recheck;
9. Cargo check;
10. Cargo build without a second source refresh.

A failure stops the gate at the actual failing phase and writes `.pcc/gates/<gate>.json` plus the normal job receipt/log and a source-only debug bundle.

## Resilience

- child output streams live into the same PCC console;
- a heartbeat is emitted after 15 seconds without child output;
- `.pcc/active_job.json` records an in-flight child process;
- if a prior host was terminated while a job was active, the next PCC launch records a recovery receipt instead of silently forgetting it;
- Ctrl+C stops the child job while preserving the PCC host;
- logs, receipts, gates and debug bundles use retention limits;
- no PCC diagnostic bundle or patch may include hydrated assets.

## Diagnostics

The Diagnostics menu can create a bundle, show the last receipt, tail the last log, list recent receipts, print environment/Git state, and prune retained history.


### v1.1.1

- FULL now runs `cargo test --all-targets` after `cargo check` and before the final build.
- `PCC.cmd test` / menu option 15 runs the same test phase directly.
- This checkpoint accompanies the DG-01 borrow-scope repair in Havenwild Bevy v0.4.1.
