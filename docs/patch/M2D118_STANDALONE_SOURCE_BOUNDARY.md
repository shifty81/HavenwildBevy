# M2D118 — Standalone Havenwild source boundary (v0.10.36)

The previous full rollup accidentally carried an unrelated game’s applications, launchers, architecture documents, donor C++ source, and source manifest. None are Havenwild runtime/build dependencies. M2D118 explicitly removes these materials from the distributed source and from already-updated installs through a PCC deletion manifest.

The active Bevy game and editor, PCC, asset-catalog/terrain contracts, existing mapping evidence, original license/credit requirements, World PIE, and room/cave document engine are not removed.

Prevention: `ForgePY._source_package_include` excludes unrelated rooted workspaces and project-identity artifacts; `tools/havenwild_identity_boundary_selftest.py` inspects the entire deployable source tree and filename/content identities and is included in Full Gate and static audit. `tools/package_source_policy_selftest.py` checks the boundary against synthetic foreign paths.

The source-only package does **not** certify native Rust compilation or visual gameplay acceptance.
