# Havenwild Bevy — Unified World/Scene/Pixel/Animation Authoring Target

This document is an architectural target. M2D05-A implements only the scoped foundation described in its patch notes. Avoid placeholder panels claiming completed authoring capabilities.

## A. Unified editor, persistent authority

Keep the existing original-source Bevy canvas, native-window/ForgeGUI shell, transaction-based SceneV2 document, authored objects and semantic layers. One selected project, one asset registry, one scene command/undo bus, one PCC. The former native Havenwild editor is **read-only donor** for proven capabilities; compare before adapting and never overwrite current Bevy models on assumption of identical semantics. In-game Play From Here is a frozen or sandboxed scene state, not a separate competing scene editor. PCC owns gate, logging, patch intake and package generation.

## B. World chunks / levels

World descriptor has finite seed, configurable enormous archipelago with sea on all four world borders, 3–15 major islands; editor can open whole overview while lazily materializing detailed chunks or opening individual scene/chunk directly. Global source-tile coordinates, chunk coordinates, and local tile addresses are different typed concepts. Suggested initial 32x32 logical tiles/chunk, manifest-configurable, not inferred from the 40x28 River fixture. Chunk data stores stable seed/source provenance, logical materials, layer data, authored overrides, elevation and object references. Stable seam ownership and neighbor invalidation across chunk edges, separate streaming/materialization vs view grid. Protected authored override survives generator rebakes; regenerating a chunk never erases it without explicit review. Home Estate placement convention NE within one overworld chunk of main city is a worldgen rule, not a fixed test-fixture requirement.

## C. Limited logical verticality, cliff-source presentation

Elevation sea=0, land +1..+30, contours are the authoritative structural boundary, even when rendered by 2D ElizaWy cliff-face tiles. Source pixel art is visual skin, not collision or terrain identity. +1 elevation is a legitimate cliff (discard obsolete >=2 historical eligibility condition). Real ground elevation, directional traversability, swim/wade, jump/step/ladder/ramp/vines, camera occlusion and object depth should be represented in scene data before rendering. Cliff bases in swimmable water get water-appropriate vines/climb access; constructed ladders primarily inland. Ramps may be authored only in derived slope corridors bounded by the two adjacent contour/cliff edges, not across arbitrary interiors or cliff faces.

## D. Tiled-like on-canvas workflow

Modes: navigate/select; semantic terrain brush (family -> variant -> detail) with drag-to-paint, fill/erase, non-destructive preview; object stamping and multi-tile selection/transform; collision/height/channel view; source/art sheet inspection; recipe/certification lab as advanced contextual panel. Families include multiple Grass, Dirt, Sand, Water, Mud, Road, Cliff, Rock etc., plus terrain-specific detail decals. Brush paints semantic material IDs and recomputes derived neighboring cliff/edge/shoreline transitions, including cross-chunk invalidation. Source exact selection and manual override remain distinct from auto-resolved tiles, with undo/redo, provenance and staged bake validation. Every layer has semantic label, visibility and optional linked logic channel, structurally separate from visuals.

## E. Collision and traversal

Shared structural metadata is independent of sprite transparency. Tri-state explicit blocking, height, flow and connector metadata should extend to footprints/shapes of complete objects, collision types and movement modes. Show overlay and live PIE probes; errors not inferred from arbitrary source pixels. Terrain Rules currently controls only the Grass/Void 4-bit NW/NE/SW/SE recipe: 16 combinations of corner materials can select source exact sprite/composites after review. Neither historical address count nor recipe completeness alone certifies runtime navigation.

## F. One internal Pixel Studio workspace

Pixel Studio is an internal first-class **application panel** in the Havenwild shell; it never replaces, frames or becomes the central world canvas and it never requires a separate executable. The panel itself can switch Compact/Full presentation and may float or pin over the left/right/bottom edge without resizing the underlying infinite canvas. Full presentation may contain its own compact pixel-tool rail, editable pixel canvas, layers/palette/inspector and animation timeline inside the panel. Open/create editable project-owned RGBA PNGs, palettes, brushes, selection, transform, undo, and publish through the shared content browser; author source-vs-derived separately and **never overwrite pinned original ElizaWy sheets**. Include atlas alignment/offset preview + deliberate confirm, 32px tile binding and terrain metadata. Animation Studio is a sibling application panel/workflow sharing the same document and publishing pipeline (frames/cels/onion skin/tags, event/anchor/socket/hitbox channels). Adapt mature native authoring features through an audited compatibility map rather than spawning a second editor subsystem. Store provenance, safe output paths and validation. No functional Pixel/Animation Studio port is claimed in M2D06-A.

## G. Donor capability reconciliation and completion order

KEEP in Bevy: current canvas, 36 Summer instances, scene/asset documents, existing Source Browser, shared visual layers, command/undo bus, dock shell, PIE and PCC. ADAPT from native Havenwild after file-by-file audit: chunk/world navigation, semantic terrain family brushes, layer/logic controls, visual scene authoring, collision and height tools, editor asset registry, mature pixel/animation workflows. Avoid direct transplant of old renderer assumptions, worldgen-only data structures, obsolete cliff eligibility, alternate independent PCC, duplicate panels, or third-party art baked into source. Completion order: (1) root + evidence + grid/structural foundation (this pass); (2) real chunk document/neighbor seam and scene switching; (3) semantic brush + derived terrain/collision validation; (4) native Pixel/Animation authoring workspace; (5) full World Editor/PIE and multi-chunk runtime certification.

## H. Current visual defects to diagnose, not guess

Screenshot's tree crown contains a rectangular darker-green region. Compare pinned source crop, alpha channel, GPU blend stage and editor overlay separately; do not alter ground art or auto-erode source pixels before source-level evidence establishes the root cause. Check house/tree render layering with selection/overlay disabled and zoom 1x. M2D04R2 corrected tree/rock source rectangles must be preserved and checked against hydrated originals.


## M2D06 locked editor-shell rule

The infinite world canvas is permanent. Only the narrow left direct-tool rail and narrow right active-layer rail are structural side rails. Every feature-rich editor surface (Source Browser, Terrain Rules, Source Evidence, Scene/Layers, Collision/Elevation, future World Generator, Chunk Manager, Pixel Studio, Animation Studio, object/NPC/item tools, diagnostics) is an application panel layered over the canvas. Dock Left/Right/Bottom means pinning the floating panel to that edge; it must not reserve layout space or resize the canvas. Compact/Full modifies only that panel. The bottom-left Havenwild launcher expands right/up and is the primary application-panel launcher. The top toolbar is reserved for real editor commands; command shortcuts are centralized and displayed in hover tooltips.
