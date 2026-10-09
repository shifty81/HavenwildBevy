# M2D093A9 — Canonical-template quarantine fixture repair

Source: 0.10.10

Windows Full Gate on 0.10.9 compiled the test target and executed all 94 Rust tests. 93 passed. The only failure was the generated-object quarantine regression fixture, which searched for the retired legacy template id `large_tree_01`. The current ElizaWy authority uses stable hashed canonical template ids, so that fixture aborted before exercising quarantine behavior.

This repair selects an actual worldgen-safe tree through `templates_with_tag("tree")`, captures its canonical hashed `template_id`, and uses that same id for both the certified contained source slice and the intentionally invalid source slice. The test therefore exercises the real source-slice quarantine contract instead of depending on a removed identifier.

The M2D093A source-side gate now rejects any return of the legacy `large_tree_01` fixture dependency and requires the canonical tag-based selector plus propagation of the selected template id.

Runtime behavior is unchanged. No terrain, shoreline, worldgen, object-bound, rendering, collision, placement, or ElizaWy source semantics change.
