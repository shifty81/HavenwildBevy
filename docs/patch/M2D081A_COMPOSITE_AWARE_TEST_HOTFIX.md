# M2D081A — Composite-aware Summer regression hotfix

Source remains **0.8.1**.

The 0.8.1 Summer completion intentionally allows a resolved world tile to consist of multiple exact source-backed visual parts for checkerboard and three-material junction states. The existing Rust regression `summer_flatworld_cells_never_resolve_from_cliff_sheets` still required every checked cell to collapse to a single `SourceBinding`, so it failed when a legitimate composite was returned.

This hotfix changes only the regression contract: it now resolves the full visual-parts list and verifies that every part is canonical and comes from `Terrain/terrain_summer.png`. Runtime resolution behavior is unchanged. The M2D081 Python self-test also rejects reintroduction of the stale single-binding assertion.
