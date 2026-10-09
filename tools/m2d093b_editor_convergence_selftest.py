#!/usr/bin/env python3
"""M2D093B regression gate for editor-surface convergence and viewport caching."""
from pathlib import Path
import json
import re
from source_version_contract import cargo_source_version, project_matches_cargo, source_at_least

ROOT = Path(__file__).resolve().parents[1]


def req(value, message):
    if not value:
        raise SystemExit(f"M2D093B SELFTEST: FAIL / {message}")


def main() -> int:
    cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    project = json.loads((ROOT / "project/forgepy.project.json").read_text(encoding="utf-8"))
    main_rs = (ROOT / "src/main.rs").read_text(encoding="utf-8")
    layout = (ROOT / "src/editor_layout.rs").read_text(encoding="utf-8")
    world = (ROOT / "src/world_doc.rs").read_text(encoding="utf-8")
    compact = re.sub(r"\s+", " ", main_rs)

    req(source_at_least(cargo, "0.10.13"), "Cargo source predates 0.10.13")
    req(project_matches_cargo(project, cargo), "project sourceVersion does not match Cargo")

    # One authoritative layer surface: permanent rail, no duplicate Scene/Layers app.
    req('fn draw_layer_rail(' in main_rs, "canvas-integrated layer controls missing")
    req('RailIcon::Eye' in main_rs and 'RailIcon::EyeOff' in main_rs, "layer visibility icons missing")
    req('state.editor_layout.world_layer_visibility[slot] = !visible' in main_rs,
        "layer eye does not toggle persisted visibility")
    req('state.editor_layout.world_layer_visibility = [false; 7]' in main_rs,
        "Shift-isolate layer behavior missing")
    req('if !world_layer_visible(state, layer.id)' in main_rs,
        "authored visual rendering ignores layer visibility")
    req('world_layer_visible(state, layer)' in main_rs,
        "generated object rendering ignores layer visibility")
    req('filter(|object| world_layer_visible(state, object.layer)' in compact,
        "hidden authored layers remain pickable")

    for retired in (
        'fn draw_layer_panel(', 'egui::Window::new("Scene / Layers")',
        'fn draw_chunk_manager_panel(', 'fn draw_asset_authority_panel(',
        'pub layers_open: bool', 'pub scene_layers_compact: bool',
        'pub asset_authority_open: bool', 'pub chunk_manager_open: bool',
    ):
        req(retired not in (main_rs + layout), f"retired duplicate surface/state returned: {retired}")

    # Direct tools are one rail; Paint restores its contextual semantic palette.
    req('fn draw_inspector_rail(' in main_rs, "contextual Paint palette missing")
    for label in ('Grass', 'Dirt / river bank', 'Sand', 'Shallow / river water', 'Deep water'):
        req(label in main_rs, f"Paint palette material missing: {label}")
    req('Evidence-gated manual materials' in main_rs and 'Legacy wet/tidal sand (manual)' in main_rs and 'Legacy pebble/stone path (manual)' in main_rs,
        "legacy evidence-gated Paint section missing")
    req('if state.world_mode && tool == CanvasTool::Paint' in main_rs,
        "Paint selection does not activate world material workflow")
    icon_rs = (ROOT / "src/editor_icons.rs").read_text(encoding="utf-8")
    req('mod editor_icons;' in main_rs and 'use editor_icons::{rail_icon_button, RailIcon};' in main_rs,
        "shared professional icon palette not imported")
    for glyph in ('Pixel', 'Tile', 'Terrain', 'Erase', 'Sample', 'Bridge', 'Panel', 'EyeOff'):
        req(f'RailIcon::{glyph}' in icon_rs, f"unified vector icon missing: {glyph}")
    req('RailIcon::Panel, state.editor_layout.tool_context_open' in main_rs,
        "context panel must not reuse a layer icon")

    # Terrain/asset authority and chunk diagnostics are folded into their canonical apps.
    req('egui::Window::new("Terrain Studio · ElizaWy Authority")' in main_rs,
        "Terrain Studio authority app missing")
    req('Materialized chunks · debug' in main_rs,
        "Generated World no longer owns chunk diagnostics")
    req('Open source browser' in main_rs and 'Open evidence review' in main_rs,
        "Terrain Studio entry points missing from launcher")

    # Avoid repeated source-path hash map construction and repeated dual-grid resolution while panning.
    req('sheet_lookup: HashMap<String, usize>' in main_rs, "persistent source-sheet lookup missing")
    req('world_visual_cache: WorldVisualCache' in main_rs, "world visual cache missing")
    req('fn prepare_world_visual_cache(' in main_rs and 'fn cached_world_vertex_visual(' in main_rs,
        "dual-grid vertex cache path missing")
    req('state.world_visual_cache.vertices.clear();' in main_rs,
        "world visual cache has no revision invalidation")
    req('pending_semantic_edit_count(&self)' in world,
        "pending semantic edit generation is not part of cache invalidation")
    req('let sheet_lookup: std::collections::HashMap<&str, usize>' not in main_rs,
        "per-frame source-sheet HashMap rebuild remains in world draw")
    req('fn assert_current_binary_matches_source(' in main_rs and 'STALE HAVENWILD STUDIO BINARY' in main_rs,
        "runtime has no stale-executable/source-version guard")
    req('assert_current_binary_matches_source(&root);' in main_rs,
        "startup does not enforce compiled/source version equality")

    # Object-bound tooling also converges on one authority; source-sheet guide strokes
    # must not return as visible tree/rock crop pixels.
    certifier = (ROOT / "tools/certify_worldgen_object_bounds.py").read_text(encoding="utf-8")
    mapper = (ROOT / "tools/build_summer_flatworld_mapping.py").read_text(encoding="utf-8")
    req('"version": 3' in certifier and 'guideLike' in certifier and 'discardedGuidePixels' in certifier,
        "v3 guide-safe object-bound certification missing")
    req('largest_connected_crop' not in mapper and 'build_certified_object_bounds' in mapper,
        "duplicate tree/rock crop authority remains in Summer mapper")
    req((ROOT / "tools/worldgen_object_bounds_algorithm_selftest.py").is_file(),
        "guide-safe object-bound algorithm selftest missing")

    # Inspector owns the selected object's layer; the top command bar does not.
    toolbar_region = main_rs.split('egui::Panel::top("havenwild.command.toolbar")', 1)[1].split('egui::Panel::bottom("havenwild.status.chrome")', 1)[0]
    req('ui.small(format!("Layer: {:?}", state.active_layer))' not in toolbar_region,
        "top toolbar duplicates inspector active layer")

    print("M2D093B EDITOR CONVERGENCE SELFTEST: PASS")
    print("  shell       : 62px tool rail + on-demand active-tool settings, canvas layer controls, 15% inspector")
    print("  layers      : persisted visibility + Shift isolate; hidden content not rendered/picked")
    print("  paint       : contextual semantic material palette restored")
    print("  apps        : duplicate Scene/Layers, Chunk Manager and standalone Asset Authority retired")
    print("  performance : persistent sheet lookup + revision-keyed dual-grid visual cache")
    print("  launch      : stale executable/source-version mismatch fails visibly")
    print("  objects     : single v3 guide-safe tree/rock render-bound authority")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
