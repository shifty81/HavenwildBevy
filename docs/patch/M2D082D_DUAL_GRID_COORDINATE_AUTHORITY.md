# M2D082D — Dual-grid coordinate authority repair

Source 0.8.7 repairs the Generated World authoring/render coordinate mismatch exposed by one-cell Grass painting and shoreline square artifacts.

- Semantic cells remain integer-aligned authored 32×32 world squares.
- Dual-grid output tiles are centered on integer terrain vertices, using a -0.5/-0.5 cell render origin.
- An N×M semantic area renders N+1×M+1 terrain vertices so east/south boundaries are complete.
- Dual-grid rendering uses deterministic virtual generated neighbours outside the currently materialized rectangle, preserving outer shoreline continuity without generated artwork.
- Exact-source Direct Tile overrides remain locked to the selected semantic cell and render after the shared terrain pass.
- Collision, selection, object anchors, and semantic paint remain in semantic-cell coordinates.
