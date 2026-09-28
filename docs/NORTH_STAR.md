# North star: one world, one studio, source-first game development

## Identity and ownership

This is a **new independent Bevy project** that begins with the LPC Revised / ElizaWy collection as sole canonical active visual family. Do not pull Havenwild branding, unrelated LPC packs, validators, runtime assumptions or arbitrary mappings into source authority. The name Havenwild is retained here as requested working identity. Original Havenwild is a donor only, not auto-synced, not modified and not retired.

## Full-screen Game Canvas desktop

The Game Canvas is the permanent editor workspace, not a closable dock tab. Infinite panning and zoom describe the editor's coordinate workspace; the GAME world is a huge but FINITE seeded archipelago, 3–15 major landmasses and ocean on all four boundaries. World overview opens all descriptor extents; detailed chunks load on demand. A single scene/map/chunk substrate serves authored scenes, shared overworld and permanent home estate, differentiated by persistence policies rather than independent world engines.

ForgeGUI title chrome plus system window behavior: exactly one visible title, real minimize/maximize/restore/close/drag; edge/corner resize and Windows native fallback. Use proper native non-client behavior and Snapping when proven. World is visible behind the tool windows; don't shrink world when Atlas/Pixel/Animation/Character/Logic/Sound opens. Tool rail, contextual HUD, diagnostics, Inspector, palette and TaskShelf overlay the canvas, with persistent state and correct pointer capture. No dead application buttons, duplicate menu bars, or paragraphs of GPU evidence on canvas.

## The first game milestone

Create one accumulating **Summer Acceptance Scene**. Drag actual source cells and multi-cell assemblies from original atlases into live preview. Correct grass/dirt/river/shore transitions first, then true tile-height elevation and ElizaWy cliffs, waterfall orientations/width variations and receiving pools, source-supported walkable cliff terminations, vines, handholds, ladders, caves, structures, bridges, foliage, characters, objects and FX. Distinguish source animation frames from modular components. Source sheets remain immutable; save placements/recipes separately with draft/review/approved state.

## Asset/scene contracts

One canonical source ID = active pack ID + unmodified path + hash where relevant, exact pixel rect, provenance, tile-size and animation/assembly info. Source variants are discovered/verified, not invented by role name. One resolver converts semantic terrain/elevation/neighbor topology and authored assemblies into a source-exact draw packet with collision/navigation. One renderer consumes this draw packet in studio and game. Vulkan/DX12 are GPU backends, not parallel mapping logic. Unsupported combinations surface as review tasks instead of substituted artwork.

## World and game design decisions to carry forward

Willowmere is permanent authored capital in each seed, generated first and refined in the editor. Its identity/districts/required buildings and core NPCs survive seed changes. Home Estate near northeast main-city boundary (within one overworld chunk) persists across worlds and single/multiplayer servers under one canonical EstateInstanceId and synchronized authoritative session policy. Players can rent city/inn housing too. Discrete source-exact terrain heights: sea level 0 and meaningful +1 through +30, including one-level cliffs, recessed ponds/terraces and multi-level mountains. Traversal only through genuinely connected source-supported vines, indentations, ladders and natural cliff-end openings; cave entrances use modular source geometry. Water is its own animated/flowing system, not an indiscriminate painted land tile. Weather, lighting and atmospheric fog belong to canvas and runtime; exploration fog-of-war stays separate. Core gameplay includes animal hunting and skinning progression; broader systems follow art/scene parity rather than preceding it.

These are approved PRODUCT REQUIREMENTS carried forward, not claims of current implementation.
