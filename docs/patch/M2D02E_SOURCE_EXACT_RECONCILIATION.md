# M2D02-E — Source-exact rollback and renderer correction

This explicitly retires the M2D02-D guessed 18-object tree/boulder placements and extra synthetic grass/path fill. The active editor now shows ONLY the original SHA-pinned 40×28 Summer River regression fixture, using its exact individual original source pointers. This is **not** a finished all-asset certification world, and 305 recovered Summer cells do not equate to certified compositions. The 16 DG masks remain unclassified/uncertified.

The M2D02-D generated 72×28 scene and 18-object index are preserved byte-for-byte under `docs/handoff/retired/`. An existing local `content/scenes/derived/elizawy_mapping_certification.assembled.draft.json` is NOT touched or auto-imported. New corrections use `content/scenes/derived/elizawy_mapping_certification.source_exact.draft.json` so old 72×28 edits cannot be accidentally loaded against 40×28 dimensions. The source-only packager continues to exclude workstation-local derived drafts.

Canvas ground tile destinations now quantize each shared scene-grid EDGE once to physical display pixels using the active pixels-per-point, preventing independent fractional tile origin/size rounding. Source UV addresses and artwork are unchanged; further Windows/DPI screenshots are required to verify no visual seams. A Rust unit test covers shared boundary equality for several zoom/DPI combinations.

The user-supplied older `HavenwildBevyLatest.zip` already contains the collection's original seasonal authored demo scene previews. Their file/hash reference catalog is saved at `docs/research/ELIZAWY_ORIGINAL_DEMO_REFERENCES.json`. Use them to examine intended art compositions, not as flattened editable output, provenance for guessed object rectangles, or automatic certification.

Future authoring must ingest exact grouping/animation/anchor metadata or create an explicit human-reviewed derived composition and validate its full original-pixel footprint before scene placement. No arbitrary crop should be auto-promoted by an environment builder. Next implement renderer UI source-vs-scene inspection and evidence-backed object composition review. The full authored world target is blocked on source-exact composition recovery—not replaced with fake scenes.

Windows test: apply E, run PCC Full Gate then DX12; confirm River-only canvas with no stray rock/tree samples, select/paint/undo/redo/save/reopen, verify older `.assembled.draft.json` still exists and does not load automatically, inspect at fractional zoom and 100%/125%/150% DPI for seams. Cargo/native visuals not tested locally.
