# M2D093A8 — Test fixture SourceBinding compile repair

Source: 0.10.9

Windows Full Gate on 0.10.8 passed source audits, Rustfmt normalization/recheck, and Cargo check, then failed compiling the new generated-object quarantine regression because `TemplatePart` does not expose a `source` field. Its public source identity is `source_path` plus `source_rect_px`.

This repair constructs `SourceBinding` explicitly from those fields and adds a source-side guard for the exact API contract. Runtime behavior is unchanged.

No terrain, shoreline, worldgen, object-bound, rendering, collision, placement, or ElizaWy source semantics change.
