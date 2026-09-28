# Havenwild Bevy v0.5.6 Build / Debug Session

Source v0.5.6 / ForgePY 0.4.6 / PCC 1.2.5 is the egui-0.36 panel-API correction over the v0.5.5 pre-build hardening checkpoint. It preserves DG-03, the Summer recovery authority, and the governed update/audit lane.

## Why this pass exists

The first native Windows Cargo run proved all pre-Cargo gates GREEN, then `cargo check` failed because egui 0.36 removed the old `SidePanel`/`TopBottomPanel` aliases and changed panel/central layout to operate on a parent `Ui`. v0.5.6 migrates the Studio shell to one root viewport `Ui`, uses `egui::Panel::{left,right,top,bottom}`, and shows `CentralPanel` inside that same root surface.

The Cargo-free Rust audit now rejects the obsolete panel aliases and requires the root viewport-Ui contract so this exact regression cannot pass static audit again.

## Tonight

1. Apply the v0.5.5 -> v0.5.6 incremental patch on the already-tested Windows tree.
2. Run `PCC.cmd audit`.
3. Run `PCC.cmd full`.
4. If Cargo reaches a new compiler error, send the newest `.pcc\debug\Havenwild_Bevy_DebugBundle_*.zip` or the complete PCC log.
5. If FULL is GREEN, run Studio DX12 and test docked mapper left/right/bottom/floating, Save All/reload, semantic edit dirty-vertex highlighting, and recipe selection from the resolved grid.

Expected non-build status remains **0/16 classified / 0/16 certified** for the Grass/Void recipe until visual source-backed mapping review is performed.
