# M2D082B Runtime Time-Wiring Repair

Target: source `0.8.4`, ForgePY `0.4.10`, PCC `1.3.5`.

The Windows 0.8.3 Full Gate reached Cargo compilation and exposed a stale `draw_ui` signature: PIE movement called `time.delta_secs()` without binding Bevy `Res<Time>`. This checkpoint restores the Time resource to the system signature, keeps the M2D081C static-water authority unchanged, retains the 205-cell M2D082 cliff-source authority, and adds a Cargo-free structural guard so this exact regression fails before Cargo in future handoffs.

No cliff semantics are promoted in this repair. The next functional pass remains M2D082 cliff classification/elevation.
