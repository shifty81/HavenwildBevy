# M2D093A1 — icon-rail compile repair

Source transition: **0.10.1 -> 0.10.2**. ForgePY remains **0.4.10** and PCC remains **1.3.5**.

The Windows Full Gate for 0.10.1 passed asset identity, object-bound certification, terrain/worldgen/compositor regressions, and Rust structural checks, then failed at Cargo check because two `rail_icon_button` calls passed owned `String` values from `command_tooltip(...)` where `&str` was required. This repair borrows those two temporary strings for the duration of the call.

No terrain, shoreline, river-mouth, WetSand, object-bound, ecology reservation, worldgen, collision, layer, or source-art behavior changes in this hotfix.
