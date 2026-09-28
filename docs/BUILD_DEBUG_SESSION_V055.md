# Havenwild Bevy v0.5.5 Build / Debug Session

Source v0.5.5 / ForgePY 0.4.6 / PCC 1.2.5 is the pre-build hardening checkpoint over DG-03. It does not manufacture recipe certification. Grass/Void remains intentionally 0/16 certified until the source-backed visual review is performed.

## First pass

From the project root:

```bat
PCC.cmd updates apply
PCC.cmd audit
PCC.cmd full
```

If an older PCC applies the cumulative patch during FULL and reports that its runtime generation is stale, start `PCC.cmd full` again. That fail-closed restart boundary is intentional.

## What audit must prove before Cargo

- project/Cargo metadata agree on source 0.5.5, Bevy 0.19.0, bevy_egui 0.42.0, Rust >=1.95 and the pinned ForgeGUI revision;
- every declared `src/*.rs` module exists and no source module is orphaned;
- the shared terrain resolver remains UI/render independent and is actually wired into Studio;
- governed patch adversarial tests reject all 10 unsafe fixture classes;
- Summer scene source rectangles are valid;
- fixture + semantic seed coverage remains 16/16;
- resolved pipeline remains 36/36 vertices and all 25 semantic cells dirty exactly four in-bounds vertices;
- corner bit order remains NW=0001, NE=0010, SW=0100, SE=1000;
- Summer recovery remains 305/305;
- all 349 canonical core assets hash correctly.

## FULL-specific acceptance

Doctor must find `cargo`, `rustc >= 1.95`, and `rustfmt`. FULL then performs fetch, rustfmt check/normalization, Cargo check, all-target tests, build, verifies that `Cargo.lock` exists, and runs `cargo metadata --locked --no-deps --format-version 1` against that lockfile.

A missing tool, old Rust compiler, non-reproducible lockfile, compile/test failure, or renderer launch failure is a real red state. The intentionally unmapped 0/16 Grass/Void recipe is not a build failure.

## Native Studio test after FULL GREEN

Run:

```bat
PCC.cmd run dx12
```

Exercise canvas locate/replace/erase, Save All + reload, mapper docking, recipe undo/redo, composite part locate/replace/offset/remove, semantic undo/redo, and the 6x6 resolved vertex grid. Toggle cells at the center and along each edge/corner; every edit should highlight exactly four vertices.

For renderer trouble:

```bat
PCC.cmd run dx12 --verbose-gpu
PCC.cmd run vulkan
PCC.cmd run vulkan --verbose-gpu
```

Keep the newest `.pcc/debug/Havenwild_Bevy_DebugBundle_*.zip`; it contains Cargo metadata/lock evidence when available, source/control files, gate receipts, logs, semantic/recipe state, and the Rust source-audit helper.
