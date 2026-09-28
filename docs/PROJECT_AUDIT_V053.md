# Havenwild Bevy v0.5.3 pre-build audit

This checkpoint was produced after reconstructing the complete ElizaWy base, applying the v0.5.0 source refresh, then the v0.5.1 and v0.5.2 governed changes in order.

Verified before packaging v0.5.3:

- complete canonical core manifest: 349/349 local files hash correctly;
- Summer recovery inventory: 305/305 non-transparent source cells accounted for;
- dual-grid fixture coverage: 16/16 masks;
- immutable semantic resolver seed: 16/16 masks;
- Summer acceptance scene: 40x28 / 1,120 placements with source rectangles inside canonical image dimensions;
- old generated `source_index.json` and `source_collisions.json` remain retired;
- source-only packaging excludes hydrated art, reference packs, local PCC/ForgePY state, governed patch ZIPs, and retired generated catalogs;
- recipe certification intentionally remains incomplete until all 16 Grass/Void states are source-reviewed and explicitly promoted.

The v0.5.3 governance pass additionally rejects incompatible or Windows-ambiguous update paths, requires exact payload byte counts and SHA-256 values, verifies installed hashes after patch promotion, validates authored scene source addresses, and exposes `PCC.cmd audit` for a no-Cargo preflight.

Windows Cargo compilation/runtime remains a separate certification step and must not be inferred from these static checks.
