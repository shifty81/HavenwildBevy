# Havenwild Bevy 0.10.34 — complete source-only rollup

This is a **reconstructed source handoff**, based on the standalone source-only snapshot at `0.10.26` and the seven enumerated governed PCC updates ending at `0.10.34` (M2D116). The patches were applied to the source file set only; this ZIP is not a git checkout or a PCC patch.

## Version and status

- Cargo source version: **0.10.34**
- Engine: **Bevy 0.19.0**
- PCC: **1.3.5**, ForgePY **0.4.10**
- Contains M2D114 professional tool-rail and terrain inspection work, and M2D116 experimental generated-world PIE with optional LPC player sprite.
- M2D115 is a **design document only**, not a code patch.
- **Native Windows compilation, Full Quality Gate, runtime graphics, doors, cave transitions, cave generator, and shoreline visual correctness have not been certified from this reconstructed rollup.**

## Source-only limitations

This archive intentionally excludes the hydrated ElizaWy image library (`assets/elizawy` PNGs), the 63,991-file LPC characters library, target/build caches, local `.pcc`/`.forgepy` state and generated local world saves. It includes the repository source, content/catalog metadata, historical legacy source, scripts, tests, planning docs, provenance and Cargo.lock. You must hydrate assets/character library through the project workflow for full runtime testing. Offline Rust dependency resolution requires already cached dependencies.

**Do not simply overwrite a different working checkout**, particularly `HavenwildFinal` version `0.10.11`. Extract into a separate clean folder, then use its `PCC.cmd` to verify and build. Back up any local project edits first.

## Reconstructed update sequence

| From | To | Patch ID |
|---|---|---|
| 0.10.26 | 0.10.28 | `HW-BEVY-M2D107-UI01-BR01-CUMULATIVE` |
| 0.10.28 | 0.10.29 | `HW-BEVY-M2D108-FAST-PCC-28` |
| 0.10.29 | 0.10.30 | `HW-BEVY-M2D110-COMBINED-PUBLISH-NATIVE-BUILD` |
| 0.10.30 | 0.10.31 | `HW-BEVY-M2D112-UA01-UNIFIED-AUTHORING-P1` |
| 0.10.31 | 0.10.32 | `HW-BEVY-M2D113-UA01-LAYOUT-TEST-REPAIR` |
| 0.10.32 | 0.10.33 | `HW-BEVY-M2D114-UA02-TOOL-RAIL-TERRAIN-WORKFLOW` |
| 0.10.33 | 0.10.34 | `HW-BEVY-M2D116-WORLD-PIE-LPC-CHARACTER-P1` |

## Integrity and validation

The source archive and every patch ZIP were CRC tested; every patch payload's claimed size and SHA-256 were independently verified; versions were checked after each application. The JSON source manifest `docs/rollup/M2D116_SOURCE_FILE_SHA256_20261009.json` gives the SHA-256 for every source payload. Historical M2D100 manifests remain unchanged historical records, not current-source manifests.

The source-only inspection and Python regression result are recorded in the delivery response. Windows native compilation is required to certify this source.
