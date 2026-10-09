# M2D093A3 — Seasonal certified render-crop repair

Source transition: **0.10.3 -> 0.10.4**. ForgePY remains **0.4.10** and PCC remains **1.3.5**.

The 0.10.3 Windows Full Gate reached Cargo check and 93/94 Rust tests. The final failure proved a real runtime issue introduced by exact tree render-bound certification: a certified Summer tree crop is a sub-slice of the canonical 96x128 atlas slot, so seasonalization rejected the same coordinate crop on the Winter sheet when it required exact canonical-region identity and silently fell back to Summer.

0.10.4 changes seasonalization to accept only a verified source slice contained inside the coordinate-equivalent seasonal canonical region. The regression test now explicitly exercises a cropped tree slice and requires the Winter source sheet plus the exact same rectangle. No terrain topology, WetSand, river-mouth, object placement, collision, or source artwork is changed.
