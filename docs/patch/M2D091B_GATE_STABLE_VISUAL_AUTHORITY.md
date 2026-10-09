# M2D091B — Gate-stable visual-authority checkpoint

Source: **0.9.4**  
ForgePY: **0.4.10**  
PCC: **1.3.5**

## Why

The Windows 0.9.2 Full Gate correctly ran `cargo fmt`, which reformatted the M2D091 erosion return branch. On the next run the pre-Cargo Python self-test failed because it compared one exact Rust whitespace layout (`return if erosion < dirt_percent { "MudBank" } else { "Grass" };`) instead of the semantic source contract. This was a certification defect, not a worldgen defect.

The preceding 0.9.3 repair also corrected a stale Rust acceptance test that still required waterfall/cave objects even though M2D091 intentionally gates those features until continuous cliff faces exist.

## Repair

- Whitespace-collapse Rust source before M2D091 semantic token checks.
- Preserve the erosion/shoreline/road/bridge/tree/cliff-gate rules without depending on rustfmt layout.
- Pre-normalize the M2D091 Rust files to the formatter layout observed in the Windows gate.
- Keep reviewed waterfall/cave recipes available while automatic placement remains gated.
- Runtime/worldgen output is unchanged from M2D091/0.9.2.

## Next

After a green Windows Full Gate, proceed to continuous source-backed cliff topology, then re-enable context-valid waterfalls and embedded cave entrances.
