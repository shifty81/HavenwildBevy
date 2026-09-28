# M2D02-F1 — In-editor source-addressed playtest proof

This pass starts playable **Summer River**, not a reconstructed original collection demo. Its single editable scene and original ElizaWy source-address references remain authoritative. No guessed tree, rock, waterfall or building crop; no copy of a flattened demo image is used as gameplay background. The existing original Summer reference PNG is comparison evidence only.

The supplied `HavenwildBevyLatest.zip` has four original seasonal demo PNGs and a landscape PNG, but no editable TMX/TMJ/ASE/ASEPRITE map for that demo. Accordingly, the original demo's placements, entity identities, collisions and navigation cannot be recovered from the screenshot alone. The eventual similar environment must be authored using individually verified original image compositions and promoted source metadata, not inferred as certified automatically.

## Windows acceptance

1. Apply this ZIP to the installed M2D02-E source through PCC 11 → 2, then run PCC 1 (full quality gate).
2. Launch Studio using PCC 3 (DX12). The current original Summer River authoring scene appears as usual.
3. Optionally select a grassy cell, then click **Play Scene** on the upper toolbar. Otherwise PIE selects the nearest supported land spawn deterministically.
4. The compact PIE banner appears, editor tools/panels are concealed and the scene is rendered from an immutable clone of the active edited scene. The yellow circle is a **debug player marker**, not new player artwork. Move with WASD or arrow keys; try crossing the river; use Esc or Stop PIE to return to exactly the prior editor viewport.
5. Verify save/paint/erase/undo controls are absent while playing; on exit, all prior unsaved scene edits and editor view return unchanged.
6. Check automatic Cargo tests, including snapshot isolation, collision exclusion/river block and normalized movement.

## Scope and honesty

- This is an in-process Bevy Studio PIE harness, sharing original source-image rendering with Scene mode. It is *not* a separate launched game executable or the full Havenwild runtime.
- V1 has only legacy `Grass`, `MudBank`, `RiverWater` role hints, not a certified structural navigation layer. To exercise walking safely, PIE provisionally treats original terrain-sheet Grass/MudBank as walkable and blocks water, cliff-sheet content, unknown roles, empty cells and bounds. It does not certify any worldgen/terrain rule or infer cliff connectors from image appearance.
- PIE session state is transient and cannot write original assets, mapping evidence, derived drafts or the scene fixture. No character sprite, source composition or procedural terrain has been generated.
- Actual source-referenced demo-style reconstruction comes *after* original compositions (tree canopies/trunks, cliff faces/corners, bridge, waterfall, building, props) are inspected and represented explicitly in the shared scene document. The screenshot is a visual acceptance target, not editable world data.

## Next acceptance gate

Render and interact with a scene whose structural and visual layers are explicitly authored from verified original source elements, then test camera, traversal, bridge/path entrances, scene persistence and the future game-client PIE adapter. Do not claim completion of that phase on the basis of this F1 proof.
