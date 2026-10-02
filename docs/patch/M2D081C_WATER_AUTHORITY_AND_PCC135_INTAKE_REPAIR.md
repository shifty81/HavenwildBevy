# M2D081C — Water Authority Repair + PCC 1.3.5 Intake Reliability

Baseline: source `0.8.1`, ForgePY `0.4.10`, PCC `1.3.4`.

Target: source `0.8.2`, ForgePY `0.4.10`, PCC `1.3.5`.

## Water correction

The M2D081 implementation incorrectly promoted the eight recovered `summer_water_fill` cells from a `RepeatableFill` group into a global temporal animation. The recovered source authority never established frame order, timing, or that the cells form one animation sequence. Visual review confirmed the result behaved like detail/variant tiles being shuffled through water.

M2D081C therefore:

- keeps all eight cells cataloged in `summer_water_fill`;
- keeps their classification as static `RepeatableFill` source regions;
- restricts automatic homogeneous RiverWater to the conservative source cell `[384,512,32,32]`;
- removes the synthetic 220 ms / eight-frame RiverWater animation declaration;
- removes the runtime water-animation clock/repaint path;
- rejects re-promotion of `summer_water_fill_source_variants` into animation authority in the v5 runtime profile;
- preserves all 81/81 Grass/Dirt/Water corner states and all 305/305 recovered Summer source cells.

A future water animation may be enabled only after an explicitly ordered authored frame set is proven from source metadata or equivalent source evidence.

## PCC 1.3.5 intake correction

PCC previously discovered root patches only by the literal `*.pccpatch.zip` filename suffix. Browser-renamed downloads could therefore contain a valid governed manifest and still be invisible.

PCC 1.3.5:

- discovers root ZIPs by the presence of root `pcc_patch.json`;
- still surfaces malformed literal `*.pccpatch.zip` archives as invalid;
- ignores ordinary root source/debug ZIPs without a patch manifest;
- verifies and reconciles already-manually-installed governed patches;
- archives superseded older handoffs so they do not poison Full Gate;
- keeps manual extract/overwrite as a supported fallback followed by Full Gate certification.

## M2D082 cliff source foundation included

This checkpoint also adds a deterministic source-only cliff inventory before runtime cliff semantics are enabled:

- `Terrain/cliff_summer.png`: 512×448, 205 canonical 32×32 source regions;
- 212 retained historical reference entries across `mountain_base`, `mountain_vines`, `mountain_features`, `mountain_animated_water`, and `mountain_waterfall_transitions`;
- historical family labels remain non-promoting evidence;
- the Havenwild elevation contract is recorded as sea level 0 with +1 through +30 valid;
- one-level cliffs are explicitly legal;
- raised-region access must be route-aware rather than requiring ramps on every face;
- runtime cliff topology/collision/traversal/occlusion certification remains 0 until the next structural classification pass.

The next active terrain task is to classify cliff top/face/corner/cap/connector source geometry and bind it to elevation through one shared visual/collision/navigation/occlusion recipe. Waterfalls follow after that structural cliff system is complete.
