# Havenwild Bevy — M2C4 full ElizaWy external review registry + PCC gate

Scope: focused root-drop overwrite-capable PCC patch, **built against the 0.5.6 / ForgePY 0.4.6 / PCC 1.2.5 source-only ZIP plus the already-installed M2AB and M2C2+M2C3 additions**. PCC becomes 1.2.6. This is not a whole-source rollup or Bevy runtime patch.

## Additions

- `tools/export_elizawy_review_registry.py`: strict pinned ZIP and original-art verification, complete per-occupied-cell record for all four 2048² derivative seasonal sheets, full original-coordinate candidate lists, ambiguity/unmatched status, TSX Wang associations, Tiled animation/collision flags *as evidence only*. Writes only `.forgepy/recovered_mapping/elizawy_tiled_review_registry.v1.json` by default.
- `tools/elizawy_m2c4_selftest.py`: adversarial tests on full review registry rows and zero-promotion invariants.
- `tools/verify_elizawy_derivative_evidence.py` + `tools/elizawy_derivative_gate_selftest.py`: checks existing local M2C3 report integrity/known source identity/counts; if the complete registry exists, validates all 10,104 occupied derivative records (2,526 per season) without requiring the external ZIP at normal PCC Full Gate time.
- PCC 1.2.6: M2C2, M2C3, M2C4 and derivative-gate self-tests included in **both static audit and Full Gate**; historical/source audit and derivative-report/registry verification included after full asset verification. Each self-test or check receives its own Full Gate phase and receipt.
- PCC skips optional workstation-local historical index or external report only with explicit `SKIP / NOT CERTIFIED` messaging. If present but invalid, evidence checks fail closed. Full Gate PASS is never a synonym for an approved or complete terrain recipe.

## Local generation command (PowerShell, after PCC applies this patch)

The ZIP is intentionally **not** embedded into the root-drop patch. To use your local download:

```powershell
$archive = Get-ChildItem "$env:USERPROFILE\Downloads" -Filter 'lpc-revised-exterior-tilesets*.zip' -File |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1 -ExpandProperty FullName
if (-not $archive) { throw 'Download the LPC Revised Tiled ZIP before exporting review evidence.' }
python tools\export_elizawy_review_registry.py "$archive" --original-root '.\assets\elizawy' --strict-archive-sha
python tools\export_elizawy_review_registry.py --verify '.forgepy\recovered_mapping\elizawy_tiled_review_registry.v1.json'
.\PCC.cmd full
```

The expected local derivative counts are Summer/Spring/Autumn unique 512, ambiguous 29, unmatched 1,985 each; Winter unique 306, ambiguous 3, unmatched 2,217. Each has 2,526 occupied derivative cells. Source matching is **coordinate evidence**, not proof of gameplay semantics, collision, certification, or an automatic recipe publication. The Autumn `-old.tsx` metadata gaps (58 missing, 8 extra and 10 changed terrain ID signatures; missing 60 fence assignments) and Winter material-name substitution remain unresolved review flags. The original source includes a separate winter-ice source variant not included in the derivative's 4-season TSX packages.

## Governance, deferrals and recovery

- No original art, derivative archive, V7 data, scene data, terrain recipe, or mapping promotion is in this patch. Zero production mappings certified. The external Buildings TSX remains architecture reference only.
- Recovered topology provenance `output_sha256` mismatch is not repaired or overwritten; historical records remain immutable.
- Read-only registry is a prerequisite for the upcoming visual M2D Mapping Studio browser. This patch does **not** claim that the GUI browser exists yet. A fresh Windows source-only snapshot is still needed before large native UI refactors.
- Old PCC `1.2.5` project metadata upgrades together with the actual PCC script, avoiding mismatched metadata/gate identity. The PCC transactional updater retains its rollback/receipt behavior.
- Locally tested: Python source compilation, M2C2/3/4 adversarial self-tests, uploaded real report schema/contents verifier, Rust source structural audit of unchanged 0.5.6 runtime, and generated patch manifest/hash validation. **Windows Cargo build and native GUI interaction remain for the user's PCC session**.
