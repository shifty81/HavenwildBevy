# M2D094B - Jaidyn Tiled Test Readiness

M2D094B supersedes M2D094A. It remains a forward-port kit for Havenwild Bevy **0.10.11+** and does not overwrite the M2D093B editor/grid/cache work.

## Gaps found in M2D094A

1. `Cargo.toml` declared `tiled = "=0.16.0"` but the handoff did not update or certify `Cargo.lock`. M2D094B also disables `tiled` default features because standalone TSX parsing does not require the crate's default Zstandard feature. Havenwild uses locked Cargo operations, so native testing could fail before executing any terrain test.
2. The offline self-test only searched for source tokens. It did not compile the real `tiled` API or parse a TSX through `rs-tiled`.
3. No end-to-end verifier checked the pinned hydrated TSX/PNG bytes against prior M2C3 evidence.
4. No gate proved Wang tile IDs resolved to rectangles inside the atlas.
5. The first exact resolver chooses uniformly between exact candidates. Tiled terrain painting honors per-tile probability, including probability zero.
6. The first exact resolver does not synthesize horizontally/vertically flipped or rotated Wang candidates. Tiled can do so when the tileset enables transformations.
7. The first handoff did not explicitly prove Summer/Spring/Winter numeric Wang topology identity or Winter's semantic rename (`Dirt -> Snow`, `Melted Ice -> Ice`).
8. Autumn is an old/partial TSX. Its missing Fences set and reduced Terrain assignments need a hard fail-closed test.
9. Animation and collision metadata were only counted by the Rust bridge; there was no independent real-source audit of frame IDs/durations or collision-count parity.
10. Hydration replacement removed the previous directory before final installation and could lose a known-good hydration if the final rename failed.
11. The downloader had no explicit user agent or archive/member size guards.
12. Attribution relied partly on other Havenwild records. The hydrated lane now carries the release's named attribution notice directly as well.
13. The current Jaidyn Terrain Wang set does **not** certify a `Pebble / Stone Path` material. Any such legacy palette entry must remain quarantined until another authored source certifies it.

## Test contract

`Test-M2D094B.ps1` performs:

1. Offline payload and synthetic parser checks.
2. Pinned Jaidyn archive hydration with exact SHA-256 verification.
3. Independent XML/PNG acceptance verification against the prior M2C3 audited profile.
4. Real Rust `tiled_terrain` unit tests using `rs-tiled`.
5. Locked Cargo compile check.
6. Havenwild PCC Full Quality Gate.
7. Post-gate re-verification of hydrated bytes.

The acceptance report is written to:

`artifacts/M2D094B_JaidynTiledAcceptance.json`

## Runtime-promotion blockers

The verifier distinguishes source/integration validity from live-renderer readiness. A valid source can return `PASS_FOUNDATION_LIVE_PROMOTION_BLOCKED` while native compile tests and the Full Gate continue.

Live promotion is blocked when the real TSX needs a Tiled behavior not yet represented by the exact resolver:

- horizontal/vertical flip or 90-degree rotation synthesis;
- exact Wang signatures with multiple candidate tiles whose probabilities are non-uniform or zero;
- incomplete exact 16-pattern corner coverage for Havenwild's required core transition pairs (Grass/Sand, Grass/Dirt, Grass/Shallow Water, Sand/Shallow Water, Dirt/Shallow Water, and Shallow Water/Deep Water).

The third case is especially important: missing exact signatures do **not** mean Jaidyn's TSX is wrong. Tiled's terrain algorithm can match/propagate beyond a naive exact lookup. A coverage blocker therefore directs the next pass toward a Tiled-compatible `WangFiller`, rather than another hand-authored atlas table.

These are not warnings to ignore. Runtime promotion must implement or delegate the missing semantics first.

## Core paint acceptance

Summer must contain authored pure-fill Wang signatures for:

- Grass
- Sand
- Dirt
- Shallow Water
- Deep Water

This directly covers the material classes that were visually failing in the previous hand-built mapping lane. `PebblePath`, `Pebble / Stone Path`, `Dry Sand`, and `Wet/Tidal Sand` are **not** automatically promoted just because older Havenwild metadata used those names.

## Result meanings

`FAIL_FOUNDATION` means source identity, parsing, atlas geometry, audited metadata, native compilation, or a project gate failed and must be repaired before proceeding.

`PASS_FOUNDATION_LIVE_PROMOTION_BLOCKED` means the Jaidyn/rs-tiled foundation is coherent and the native Havenwild gates can complete, but exact-only tile selection is not sufficient to claim Tiled behavior. The report names the transform, probability, or exact-coverage gaps that must be delegated/implemented next.

`PASS_EXACT_RUNTIME_READY` means the exact resolver also happened to need none of those additional Tiled behaviors for the audited core transition contract.

None of these statuses switch the renderer. M2D094B certifies the authority foundation and tells the following visual-integration pass which topology engine is actually required.
