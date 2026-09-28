# M2D02-C — One ElizaWy Mapping & Certification Lab

## Why one scene
- Retire the proposal to create a second user-facing `ElizaWy Terrain Source Overview` scene.
- `content/scenes/elizawy_mapping_certification.scene.json` is the only default Studio workspace and PCC initialScene. It contains 29 original Terrain-sheet reference boards **and** a source-exact copy of the complete 40x28 Summer River visual fixture as an editable test region. This is not a gameplay world or an automatic mapping certification.
- `content/scenes/summer_river.scene.json` is retained unchanged as a protected input fixture for tests/regeneration, not displayed as a second editing scene.
- The board index `content/scenes/elizawy_mapping_certification.index.json` records source paths/hashes, footprints, fixture SHA and explicit zero-promotion/evidence-only status.

## Using the one canvas
- Studio opens centred on the original Summer terrain sheet. Canvas upper-right `Source` and `Test` controls focus the reference area and the editable River Test area within the **same scene**.
- The original sheet reference area is selectable and sampleable but never Paint/Erase enabled. Its green corner marks mean source-recovered, not DG certified. View menu toggles recovery marks and rulers.
- Use Sample on a reference cell, navigate to River Test, Paint/Erase there, then Save Scene. The result is a separate `content/scenes/derived/elizawy_mapping_certification.draft.json` document that can be reviewed and committed deliberately. The generated scene, old fixture, immutable artwork, historical SHA evidence, recipes and registry remain unchanged.
- Existing local derived drafts load preferentially on startup. `tools/build_mapping_certification_scene.py` regenerates *only* the checked-in canonical reference scene/index and has `--check` for source/drift verification; it does not erase user drafts.
- `Scene > Preview separate v2 scene draft` still creates a separate experimental v2 draft, not a fake live v2 runtime switch. Recipe certification, regeneration of project worlds and full-scene Pixel editing remain future stages.

## Accurate reporting
- The original Summer terrain sheet has 305/305 occupied cells recovered as exact source evidence; 111 transparent cell identities unresolved. 29 original Terrain sheets are browsable here, but their presence does not imply they are fully semantically mapped. The current sample Grass/Void recipe remains 0/16 classified/certified.
- The 40x28 River Test is an exact visual copy of the fixture, not validated worldgen, drainage, navigation or gameplay semantics. External Tiled/JaidynReiman art remains evidence only via the advanced ElizaWy Review tab.
- Native UI/full gate must be run on Windows. Python structural checks do not establish native compile or visual correctness.

## Safe GitHub onboarding to an empty repository
- The public `shifty81/HavenwildBevy` repository is presently empty. Your Windows project root is a local folder, not a Git checkout. Do not refresh from origin or replace the local folder.
- Apply/certify this cumulative patch first. Review `.gitignore` and `git status` for derived documents, secrets, hydrated art, debug bundles and ZIP payloads before the first publication.
- Initialize `main` in the existing project root and set `origin` to `https://github.com/shifty81/HavenwildBevy.git` only after verifying the local FULL gate. Publish source only; do not add original ElizaWy art, local registry, logs, archives or generated binary assets.
