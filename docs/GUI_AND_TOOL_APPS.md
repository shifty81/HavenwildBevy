# GameMaker-style canvas and tool applications

**Locked concept:** one native OS host window. The entire client is a persistent, infinitely pannable authoring workspace (finite game-world descriptor). One ForgeGUI product title, one menu, contextual in-canvas rail/HUD, optional bottom TaskShelf, no closable framed center canvas. The source project includes a MINIMUM custom canvas host and a responsive dockable Terrain Mapper surface using the pinned ForgeGUI surface-state contract. The mapper can live left/right/bottom or floating while the Game Canvas remains the permanent center authority. It does not pretend the pinned ForgeGUI shell already has a finished CanvasDesktop API.

| Tool | Trigger | Desired actual functionality | State |
|---|---|---|---|
| Terrain Mapper | source browser or world selection | click/drag exact original atlas cell or verified assembly, source provenance, replacement/recipe modes | initial source-cell instance replacement source authored; Windows test pending |
| Pixel Studio | Edit Pixels on selected resource | common raster document, palette, brushes, layers, derived art, immutable originals | planned donor extraction |
| Animation Studio | animation asset/event | same raster backend, frame/timeline, tags, anchors, sound sync | planned |
| Character Studio | selected character | modular wearable/paper doll, gear, sprite layers, animation | planned |
| Logic Studio | selected asset/entity | simple WHEN/IF/DO behavior cards and optional advanced nodes, one serialized behavior model | planned |
| Sound Studio | asset/event/emitter | wave/audio editing, sound binding, effects, node DSP where needed | planned |

All real tools float over canvas first, optionally dock/minimize/restore with one shared document command bus, undo history and resource selection. Only register launchers when a tool genuinely opens. Overlays capture clicks/drags when active; ensure source-to-world drag destinations aren't obscured by another surface. Title uses ForgeGUI chrome, Bevy's window decorations disabled by default, `STUDIO_NATIVE_FRAME=1` fallback. Full Windows snap/native hit-test/multi-DPI integration must be engineered and tested rather than claimed present.

A screenshot of original pixels is a render observation; without review it is not semantic approval, collision correctness, source exact assembly certification or runtime parity.
