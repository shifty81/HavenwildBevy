# M2D125 – M2D123 reviewed-placement Full Gate repair

## User report

The 0.10.42 Windows PCC Full Gate failed in Python at the literal assertion `authority.object_templates.iter()` inside `tools/m2d123_asset_review_placement_selftest.py`. The verified 0.10.42 source has that expression; M2D124 did not modify `world_doc.rs`, so the observed local source may differ. Do not infer a broken runtime solely from a source-string assertion.

## Repair

- Replace the opaque literal-string assertion with a whitespace-tolerant check scoped to the `place_reviewed_template` method; fail with the name of a missing semantic guard, not a bare source spelling.
- Preserve 320 SHA-pinned original image identities, 238 evidence-only candidate groups, 138 dense groups and 28 reviewed templates.
- Require the real placement method, canonical region verification, bridge rejection, unknown-template guard, persisted world-object insertion and revision increment.
- Add a native Rust test that exercises approved placement and rejects both unreviewed IDs and modular bridges.
- Include the certified `world_doc.rs` in the patch to recover divergent local source, with PCC rollback. This changes source code and tests, **not** authored project worlds or ElizaWy source files.

## Acceptance

1. Windows Full Gate reaches and passes the M2D123 reviewed placement check.
2. Cargo test compiles and passes the new `reviewed_object_placement_uses_canonical_sources_and_persists` test.
3. Studio Reviewed Complete Objects placement still works; candidates remain inspection-only.
4. The project remains Havenwild-only with existing GUI, asset, terrain and PIE regressions intact.

Python source-contract checks and PCC simulation do not replace native Windows Cargo certification.
