# Havenwild Bevy M2D099 — Verified source credit identity recovery

**Base:** `0.10.19` / PCC `1.3.5` / ForgePY `0.4.10`  
**Patch target:** `0.10.20` (diagnostics and controlled source-credit restoration only)

## Evidence from the uploaded October 8 HWlatest working tree

The M2D098 source inventory reports `SOURCE_IDENTITY_MISMATCH` for 28 files. A fresh local SHA-256 comparison proves **all 28** mismatches are `assets/elizawy/**/Credits.txt`. Every original file differs only by **LF in place of Windows CRLF**. On *each* file, converting only `\n` to `\r\n` reproduces both the exact pinned manifest SHA-256 and the exact pinned byte count. All 320 original PNGs and the one GIF already match their pinned identity. No manifest or artwork fix is warranted.

A separate gate blocker remains: `Characters.zip` is not provided/verified. **This repair is only expected to clear the credit-source identity issue, not certify the whole character library or production visuals.**

The barrel metadata already passes the current doctor: `Objects/Furniture/Barrel.png`, exact original 160×64 sheet, reviewed 32×64 column, [0,-32] foot anchor, runtime hash and template count. The M2D097C Rust regression lookup fix is present. The doctor incorrectly emitted an obsolete “legacy seed-only builder” warning, despite the builder consuming the reviewed `content/worldgen/elizawy_showcase_templates.v1.json`; this patch corrects that diagnostic. It does not alter the barrel runtime sprite or suggest visual certification.

## Applying safely

1. Copy `Havenwild_Bevy_M2D099_Credit_Identity_Recovery_From_0.10.19.pccpatch.zip` into the **Havenwild project root**. Do not extract the governed patch or drop it into a different project.
2. Run `PCC.cmd`; use the governed patch intake/apply menu and approve the pending update. The patch transitions the source package from `0.10.19` to `0.10.20` for the PCC state contract. It does **not** contain any hydrated asset bytes.
3. From PowerShell in that root, preview the manifest-backed plan:

   ```powershell
   python tools/repair_elizawy_credit_identity.py --root .
   ```

4. If the output is exactly `28 RESTORABLE_CRLF / 0 CONFLICT`, explicitly restore the exact pinned original bytes:

   ```powershell
   python tools/repair_elizawy_credit_identity.py --root . --apply --report artifacts/m2d099_credit_recovery.json
   ```

   The script checks *all* sources before applying changes, backs up the original LF copies under `.forgepy/asset_authority_recovery/credits_crlf/`, makes atomic replacements, verifies exact final byte identities, and rolls back partial writes on error. It never modifies PNGs, GIFs, project scripts or source manifests. Unexpected content changes cause it to refuse all modifications.
5. Re-run the external M2D098 asset audit to confirm that `SOURCE_IDENTITY_MISMATCH` disappears, and review the separate character dependency. Run `python -m unittest discover -s tests -p "test_*.py"`, `python tools/build_elizawy_asset_lane.py --root . --check`, and the PCC Full Gate. For visual acceptance, launch the updated Generated World in Vulkan and examine coast edges, water cadence, pond/river lily pads and full-height objects in an actual runtime frame.

## Out of scope / contamination warning

The original upload included unrelated project files that were excluded by the M2D118 identity-boundary cleanup. The Havenwild source and original art identities remain authoritative.

**Verification boundaries:** Python/JSON source tests and local deterministic image identity can be checked here. A new Windows Rust compilation/Full Gate and visual renderer acceptance must still be performed locally; no renderer acceptance is claimed by this source patch.
