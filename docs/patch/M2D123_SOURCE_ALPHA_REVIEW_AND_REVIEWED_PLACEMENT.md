# M2D123 — Verified original-alpha candidates and reviewed object placement

Baseline: 0.10.40 → 0.10.41. Original ElizaWy PNG artwork remains external and immutable.

- All 320 source PNGs were byte-for-byte verified against the pinned canonical manifest from HWlatest.zip. The alpha scanner produces evidence-only groups based on touching occupied 32×32 cells. Adjacent packed atlas artwork is **not** automatically separable; dense groups are counted but withheld from suggested isolated groups.
- A new immutable source-hash-bound candidate index enables the bottom Library to show alpha-driven candidate rectangles; selecting them **never approves or places** an object. Source/Atlas view still shows the exact authored layout.
- All *previously reviewed* whole objects on the selected source sheet can now be placed in materialized World via a one-shot placement action; source rectangles stay exact, and successful placements persist in WorldDocument with undo and the existing render-asset sync. No automatic gameplay/collision semantics are invented. Modular bridge templates use the pre-existing dedicated tool. Structures refuse overlap.
- New PCC check validates full candidate identity, bounds, no overlapping registrations, clean source-only packaging.

**Not completed:** Native Rust compilation (toolchain unavailable here), visual render certification, independent sprite splitting on densely packed atlases, new semantic promotions, animation clips, collision mapping, complete interior/chunk rendering, cliff/waterfall recipes.
