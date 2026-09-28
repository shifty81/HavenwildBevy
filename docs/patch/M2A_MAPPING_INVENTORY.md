# M2A — Universal mapping inventory foundation

This is a non-destructive source-only inventory, not a semantic mapping import or GUI replacement.

- `content/catalog/mapping_inventory.v1.json`: deterministic identities for 349 canonical non-character assets, 416 Summer source cells and seven historical evidence records.
- `tools/build_mapping_inventory.py`: rebuild or check the inventory against the canonical source manifest and Summer recovery record.
- Run `python tools/build_mapping_inventory.py --check` to detect stale or conflicting inventory; run without `--check` to regenerate.
- All semantic roles are deliberately empty and certification is `not_certified`. The historical records are evidence, not promoted rules.
- The character library is not yet counted by the core manifest and is not silently represented as indexed.
- No existing files, scene data, recipes, UI code or runtime code are overwritten.
