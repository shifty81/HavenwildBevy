# M2D095F — Shared Studio title/version regression contract / source 0.10.15

Windows evidence from source 0.10.14 showed M2D080 passing and M2D081 immediately failing on the same obsolete exact one-argument Studio `format!` source string. This pass removes that class of failure rather than repairing milestones one at a time.

- Adds `studio_title_uses_cargo_version()` to `tools/source_version_contract.py`.
- M2D07W, M2D080, and M2D081 all consume the shared semantic contract.
- The contract requires the visible `Havenwild — Bevy Studio v...` title to derive from `CARGO_PKG_VERSION`.
- Optional diagnostic/backend suffixes are allowed without weakening package-version identity.
- No runtime terrain, renderer, worldgen, asset, Wang, or backend behavior changes.
