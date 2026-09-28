# Havenwild Bevy v0.5.6 native compile correction audit

The first Windows FULL run established a clean boundary: Python/project/PCC/Rust-structure/authored-scene/DG/recovery/asset/toolchain phases all passed, including 349/349 full core asset hashes and Rust/Cargo 1.95.0. Cargo then isolated seven egui 0.36 panel API errors in `src/main.rs`.

v0.5.6 corrects that boundary without changing terrain authority or ElizaWy assets:

- replaces `SidePanel`/`TopBottomPanel` with unified `egui::Panel`;
- constructs one root viewport `egui::Ui` and shows docked panels + `CentralPanel` inside it;
- keeps floating `Window` and resize-handle `Area` context-hosted where appropriate;
- migrates panel width/height methods to `default_size`/`min_size`/`exact_size`;
- removes the reported unnecessary mutable closure;
- teaches the Cargo-free Rust audit to reject the obsolete panel API and require the viewport-root contract.

No DG recipe states are promoted by this pass.
