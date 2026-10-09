# M2D092A — Shared cliff topology + settlement semantic repair

Source: **0.9.5**  
PCC: **1.3.5**  
ForgePY: **0.4.10**

This checkpoint closes the 0.9.4 Full Gate failure and starts the real continuous-cliff implementation without inventing cliff artwork.

## Gate / settlement repair

- Settlement house labels now follow the generated settlement class (`town house`, `village house`, `hamlet house`, `camp house`, `outpost house`) instead of collapsing every major settlement to `village house`.
- Large houses use a deterministic nearby parcel search, preferring all-Grass footprints before other land, so purpose-first structures are not silently dropped when a nominal anchor overlaps a river or road transition.
- Generated World advances to generator **v5** so existing workstation-local generated chunks rebuild under the corrected placement semantics while authored cell/behavior overrides remain preserved; generator-owned tombstones/corrections are reset with the regenerated object set.

## Shared cliff topology foundation

- Every generated cell now persists a four-bit cardinal cliff mask derived exclusively from neighboring elevation drops: North=1, East=2, South=4, West=8.
- The mask is shared world authority. Studio rendering, built runtime rendering, collision/navigation, waterfalls and cave entrances are expected to consume the same topology; there is no editor-only cliff model.
- The cliff mask is source-art agnostic. `cliff_summer.png` role projection remains review/evidence gated, so this pass does **not** guess atlas roles or re-enable standalone waterfall/cave sprites.
- Generated World UI exposes the selected cell's cliff mask beside elevation for direct screenshot/debug review.

## Preserved visual rules

M2D091 remains active: compact coast bands, grass-first inland banks with sparse erosion, one-cell dirt roads, contextual bridge placement, separated roadside lighting, two-pixel large-tree atlas sampling, and cliff-gated waterfall/cave recipes.
