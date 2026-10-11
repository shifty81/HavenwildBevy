# M2D132D — M2D093B Windows rustfmt gate correction

The M2D093B source gate relied on a single-line textual substring for `state.layer_isolate.toggle(`. Windows rustfmt split this call across lines in `src/main.rs`. The active Shift-isolate code remains present and this fix checks its syntactic shape with whitespace-tolerant regex while still asserting the isolation-mask code. No runtime code or art files are modified.

Source version: 0.10.60 -> 0.10.61. Run PCC Full Quality Gate on Windows before testing the opt-in hybrid terrain example.
