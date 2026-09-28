# Havenwild Bevy v0.5.3 Build / Debug Session

This is the Windows certification runbook for source v0.5.3 / ForgePY 0.4.4 / PCC 1.2.3. The key change from v0.5.2 is a fast static audit before Cargo plus stricter governed-update/source-address validation. Recipe certification is still intentionally separate from build certification.

## 0. Apply updates and run the static audit first

From the project root:

```bat
PCC.cmd updates status
PCC.cmd updates apply
PCC.cmd audit
```

If a v1.2.0 controller was already inside FULL when it applied this cumulative update, ForgePY Doctor deliberately stops that legacy process and tells you to restart PCC; this prevents the old phase list from producing a false GREEN.

`audit` must pass Python syntax, project metadata/dependency agreement, authored Summer scene source rectangles, DG fixture/resolver coverage, Summer source recovery, and all 349 core asset hashes. It does **not** require Cargo and it does **not** require the 16 Grass/Void recipe states to be certified.

Then continue with the FULL gate below.


1. Drop the governed `.pccpatch.zip` into the repository root or `updates/inbox/` without extracting it.
2. Run `PCC.cmd full`. FULL applies governed updates first and restarts PCC automatically if the controller itself changed. Doctor now requires `cargo`, `rustc >= 1.95.0`, and `rustfmt`; a green FULL also requires a generated `Cargo.lock`.
3. Do not run strict terrain certification as a prerequisite. The Grass/Void recipe intentionally begins at 0/16 certified and is manual authoring work, not a build blocker.

FULL must independently receipt: update apply, Python syntax, DG fixture coverage, DG semantic resolver lab, Summer recovery, source refresh, Doctor, asset status/repair when needed, full 349-file core verification, Cargo fetch, rustfmt, Cargo check, tests, and build.

## Expected terrain state before manual authoring

- Summer source recovery: 305/305 non-transparent cells.
- Fixture topology coverage: 16/16 masks.
- Semantic resolver lab coverage: 16/16 masks.
- Grass/Void recipe: 0/16 classified and 0/16 certified until authored.
- The checked-in 5x5 semantic seed contains no atlas coordinates and remains immutable during normal editor work. The editor saves experiments to `.forgepy/semantic_lab.local.json`, so exploratory toggles cannot invalidate the 16/16 gate fixture.

## Studio visual smoke test

After FULL is green, run DX12 from PCC. Confirm the Terrain Mapper can be docked left, right, bottom, and floated without making the controls unreadable. Confirm the source browser reports 29 hydrated Terrain sheets from the canonical manifest and that filtering can switch among them.

On the acceptance canvas, select a placed cell and test Locate source, Replace from current source, Erase, Undo, Redo, and Save all / Ctrl+S. Close and reopen the Studio and verify the saved correction and mapper layout persist.

In the DG recipe area, assign one topology as a sprite, change it to a composite, add at least two parts, edit their offsets, locate each part, replace one part, remove one part, then exercise Undo recipe / Redo recipe. Any source or composite edit must demote that mapping to candidate. Promote it to Reviewed and then Certified only after the resolved preview is visually correct.

In the Live semantic resolver lab, toggle one cell and verify the UI reports exactly four dirty surrounding vertices. Move the vertex inspector through several positions. The shown 4-bit mask must change from semantic topology alone, and the resolved source preview must track the recipe for that mask. Save writes only `.forgepy/semantic_lab.local.json`; use **Reset from 16-mask seed** to return the working copy to the checked-in diagnostic pattern.

## Runtime fallback matrix

If DX12 fails but FULL was green, rerun once with `PCC.cmd run dx12 --verbose-gpu` to capture renderer/window tracing and a full Rust backtrace, then try Vulkan without changing source. If Vulkan also fails, use `PCC.cmd run vulkan --verbose-gpu`. A Vulkan success with DX12 failure is renderer/backend evidence, not terrain-authoring evidence. Each failed run automatically creates a debug bundle.

If FULL fails at rustfmt, PCC will normalize formatting and recheck automatically. A later Cargo check/test/build failure should be investigated from the gate receipt and generated debug bundle rather than rerunning commands manually first.

If saving fails, do not delete the authored document. The transactional writer uses sibling temporary/backup files and attempts rollback. Preserve the project root and generate a diagnostics bundle before further edits.

## Evidence to return after the session

The most useful artifact is the newest `.pcc/debug/Havenwild_Bevy_DebugBundle_*.zip`. It includes environment/toolchain/GPU information, recent PCC logs and receipts, the last gate state, Summer scene, DG recipe and semantic lab, sanitized mapper layout state, and the relevant source/configuration files. If the run succeeds but a visual behavior is wrong, create a manual bundle from PCC Diagnostics immediately after reproducing the problem so its logs and saved authoring state line up with the visual report.
