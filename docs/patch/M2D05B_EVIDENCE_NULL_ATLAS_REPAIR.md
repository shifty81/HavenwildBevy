# Havenwild Bevy M2D05-B: exact historical null-atlas repair

## Observed Windows failure

M2D05-A v0.5.7 passed cargo check but failed 1/51 tests in `historical_evidence::tests::embedded_historical_review_ledger_is_valid_and_remains_unpromoted` because `review_triage.v3.json` contains `"canonicalAtlas": null` at line 8426. Review triage has 21 such null atlases (of 284 clusters); region_candidates has 21 such null seasonal targets (of 2,549). Those cases have `canonicalMatchStatus: "none"`, indicating that an original source address exists but no canonical atlas is assigned. This is not evidence of missing original artwork.

## Code change

Both Rust `canonical_atlas` fields become `Option<String>`. Keep the historical JSON bytes unchanged. A strict validator accepts None only with `canonicalMatchStatus == "none"`; known atlas paths must be nonempty. No record is promoted. Browser shows an explicit no-atlas label and adds winter_ice to season filtering. Tests check the real null cases and reject invalid absent/blank targets.

## Safety / scope

PCC patch is exact-version 0.5.7 -> 0.5.8; it targets current M2D05-A installs, including a PCC run that performed cargo fmt automatically. It updates Rust reader/display and version metadata only. It does not overwrite `content/mapping/recovered` JSON, `content/scenes`, local `.forgepy` evidence, asset PNGs, character hydration, or your authored scene drafts. Existing PCC transactional rollback and full gate remain authoritative.

Do not claim GREEN until `PCC.cmd full` passes locally and Studio opens with Source Evidence review including the no-atlas cases.
