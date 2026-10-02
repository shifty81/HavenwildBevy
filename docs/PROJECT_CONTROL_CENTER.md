# Havenwild Bevy Project Control Center

PCC is the persistent operator for this standalone source. `ProjectControlCenter.py` owns the interactive host and calls `ForgePY.py` as a child backend.

## Guarantees

- A failing ForgePY/Cargo/Git/validator child does not terminate the interactive PCC host.
- Child stdout/stderr is streamed live and copied to `.pcc/logs/`.
- A 15-second heartbeat is emitted when a child remains alive but silent.
- Every child job writes a JSON receipt under `.pcc/receipts/` and updates `.pcc/last_job.json`.
- FULL writes a gate receipt plus a unique JSON receipt for every phase under `.pcc/gates/`, and updates `.pcc/last_gate.json`.
- `.pcc/active_job.json` records an in-flight child. A later PCC launch records a recovery receipt if the prior host ended while that job was active.
- Failed gates/jobs create source-only debug ZIPs under `.pcc/debug/`.
- Hydrated art, local source archives, generated asset catalogs, `.forgepy/`, `.pcc/`, Cargo output, and update payloads are excluded from normal source packages.
- PCC never stashes/resets/rebases local Git work.

## Entry points

```bat
PCC.cmd
PCC.cmd full
PCC.cmd build
PCC.cmd run dx12
PCC.cmd run vulkan
PCC.cmd doctor
PCC.cmd status
PCC.cmd source status
PCC.cmd source refresh
PCC.cmd assets status
PCC.cmd assets verify-core
PCC.cmd assets hydrate-characters
PCC.cmd assets verify-characters
PCC.cmd terrain status
PCC.cmd terrain fixtures
PCC.cmd terrain certify
PCC.cmd updates status
PCC.cmd updates apply
PCC.cmd diagnostics bundle
PCC.cmd diagnostics last
PCC.cmd package
```

`Studio.cmd` launches `PCC.cmd run dx12`.

## FULL gate

PCC FULL is phase-oriented and stops at the first unrecovered failure:

1. governed update inbox processing;
2. Python controller syntax validation;
3. DG contract + full fixture topology coverage;
4. Summer mapping-recovery boundary validation;
5. safe GitHub source refresh when this folder is a clean checkout;
6. doctor;
7. asset status and remembered-source repair/sync when required;
8. full 349-file core-manifest size/SHA-256 verification;
9. Cargo fetch;
10. rustfmt check; if needed, governed `cargo fmt` normalization followed by a recheck;
11. Cargo check;
12. Cargo tests;
13. Cargo build without a second source refresh.

Strict terrain certification remains separate (`PCC.cmd terrain certify`) until all 16 Grass/Void topology states have explicit source-reviewed sprite/composite/unsupported decisions.

## Governed patch inbox

PCC detects:

- `updates/inbox/*.zip`
- root `*.pccpatch.zip`

A patch must contain `pcc_patch.json` with schema `havenwild.bevy.pcc.patch.v1`, a filesystem-safe stable `id`, and a SHA-256 declaration for every source file. Duplicate names/paths, oversized payloads, write/delete collisions, and reserved Windows device names are rejected. Extra undeclared files reject the patch. `assets/`, `reference/`, `.git/`, `.pcc/`, `.forgepy/`, `target/`, `artifacts/`, and generated local asset catalogs are forbidden destinations.

PCC stages and hashes the patch, backs up every destination to `.pcc/rollback/`, applies replacements atomically, archives the consumed ZIP to `updates/archive/`, and records a patch receipt. If the controller itself changes, subsequent governed update runs restart immediately into the updated PCC before continuing the gate/build. A failed application restores the backup.

Example manifest:

```json
{
  "schema": "havenwild.bevy.pcc.patch.v1",
  "id": "DG01-0001",
  "files": [
    {"path": "src/main.rs", "sha256": "..."}
  ],
  "delete": []
}
```

## DG-02 resolver diagnostics

`PCC.cmd terrain resolver` validates the persisted semantic-only 5x5 resolver lab and requires 16/16 topology-mask coverage. FULL runs fixtures, resolver-lab validation, and Summer recovery as separate receipted phases before Cargo work. Debug bundles include the resolver lab plus `src/semantic_lab.rs` and `src/source_catalog.rs`. Runtime launches also set `RUST_BACKTRACE=1` so Bevy/Rust panics captured by PCC contain actionable stack traces.

### v1.2.2 runtime diagnostic profiles

`PCC.cmd run dx12 --verbose-gpu` and `PCC.cmd run vulkan --verbose-gpu` keep normal runs quiet but provide an opt-in renderer/window trace with a full Rust backtrace for backend-specific failures. Runtime failures are still streamed, receipted, and bundled automatically.

FULL also checks the declared Rust minimum (1.95.0), requires rustfmt, compiles every source-controlled Python helper, and ends with a `Cargo lock snapshot` phase after Cargo build.


### v1.2.3 governed-update and static-audit hardening

- `PCC.cmd audit` runs the high-value source checks that do not require Cargo: Python syntax, metadata/dependency agreement, authored-scene source-address integrity, 16-mask fixtures/resolver seed, Summer recovery, and full 349-file core asset hashes.
- Governed patches must exactly match installed `source` / `forgepy` / `pcc` versions declared in `from`; later chain links remain blocked until their predecessor is applied.
- Only one compatible patch may target the current state. A PCC-changing patch is followed by an immediate controller restart before another chain link is considered.
- Patch paths are validated using Windows destination identity even when PCC runs elsewhere: case collisions, reserved device names, trailing-dot/space names, backslashes, invalid characters/ADS syntax, symlink escapes, and write/delete collisions are rejected.
- Every payload entry requires an exact byte count and full SHA-256. Installed files are re-hashed after promotion before a success receipt is written.
- The initial authored Summer scene is validated against canonical manifest dimensions so out-of-bounds or unknown source rectangles cannot hide behind otherwise-valid asset hashes.

- PCC child-job state now carries the running controller version. ForgePY Doctor compares that runtime generation with project metadata and fails closed if an older in-flight PCC process tries to continue after replacing its own source.

### v1.2.4 DG-03 resolved-pipeline audit

PCC v1.2.4 adds `terrain pipeline` as a first-class static/runtime-preflight surface. `audit` and FULL now verify the semantic seed through the complete semantic-cell -> corner-mask -> recipe-state path before Cargo. The check confirms 36/36 resolved vertices for the 5×5 seed, 16/16 mask coverage, foreground/background family agreement, and the exact four-vertex invalidation contract. Debug bundles now include `src/terrain_resolver.rs` so failures in the shared resolver can be reviewed with the rest of the mapping evidence.

### v1.2.5 pre-build structural / lockfile hardening

PCC v1.2.5 adds a Cargo-free Rust source audit before authored-content checks. It verifies the Rust module graph, exact Bevy/bevy_egui pins, Rust minimum, pinned ForgeGUI revision, shared-resolver wiring, merge-marker absence, and the UI-agnostic resolver boundary. DG pipeline validation now checks all semantic cells including edge/corner invalidation and independently locks the NW=1, NE=2, SW=4, SE=8 bit orientation. ForgePY FULL now fails on any Doctor error instead of continuing when only rustfmt/MSRV is wrong. After a successful Cargo build, FULL also runs `cargo metadata --locked --no-deps` so the generated `Cargo.lock` is not merely present but actually reproducible.

## PCC 1.2.7 operator workflow

The root menu is intentionally workflow-first:

1. **Full Quality Gate / Certify Current Source** — runs the complete gate and records a Git worktree fingerprint in the gate receipt.
2. **Commit + Push Certified Source to GitHub** — available only when the current worktree exactly matches the latest successful Full Gate. It fetches `origin`, refuses detached/missing/diverged/behind states, stages only source-safe changed paths, commits, and pushes the current branch. PCC never auto-stashes, rebases, resets, or merges during publish.
3. **Patch Scan / Review / Apply** — shows governed root/inbox patches and applies only the one exact-version-compatible patch selected by the existing transactional patch authority.

Build/run choices are grouped under **Studio / Build / Run**. Doctor/status/static audit are under **Project Health**. Asset hydration, terrain/mapping authority, source control, diagnostics, and maintenance each retain a dedicated submenu.

A successful Full Gate does not itself push to GitHub. The explicit publish action is the source-control boundary. Full Gate still retains the existing governed patch-intake phase before certification so a build can never certify stale pre-update source.

## Git filename-stream safety

PCC 1.3.3 keeps stderr separate from machine-readable NUL-delimited Git filename queries. Windows core.autocrlf/line-ending warnings therefore remain diagnostics and can never be interpreted as source paths during certified staging.

### PCC 1.3.4 retry-safe certified staging

Certified publish staging is now safe to retry after a previous publish stopped part-way through staging. PCC stages only paths whose index still differs from the working tree (plus untracked files), leaves already-staged deletions alone, then proves the final staged path set exactly equals the Full-Gate-certified change set before commit. This prevents deleted source paths that are already absent from the index from being re-submitted as invalid pathspecs.
