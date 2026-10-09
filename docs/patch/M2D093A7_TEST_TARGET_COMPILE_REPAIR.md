# M2D093A7 — generated-object quarantine regression compile repair

Source: 0.10.8
Parent: 0.10.7 / M2D093A6

The 0.10.7 Windows Full Gate passed Rustfmt and Cargo check, then failed while compiling the Rust test target because the new generated-object quarantine regression called a nonexistent `AssetAuthority::object_template()` helper.

This repair:

- changes only that regression lookup to `a.object_templates.iter().find(...)`;
- preserves the certified contained-source-slice quarantine behavior from M2D093A5;
- adds a source-side guard rejecting the nonexistent helper call;
- makes no runtime terrain, shoreline, worldgen, rendering, collision, placement, source-art, or object-bound semantic changes.

Acceptance target: Windows Full Quality Gate compiles and executes the complete Rust test suite.
