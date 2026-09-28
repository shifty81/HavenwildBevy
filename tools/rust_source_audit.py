#!/usr/bin/env python3
"""Cargo-free structural audit for the Havenwild Bevy Rust source lane.

This is not a Rust compiler. It catches source/control drift that can be proven
without Cargo so PCC audit can fail early on malformed handoffs, orphan modules,
version skew, merge markers, or a disconnected shared terrain resolver.
"""
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
CARGO = ROOT / "Cargo.toml"
PROJECT = ROOT / "project" / "forgepy.project.json"


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def main() -> int:
    errors: list[str] = []
    warnings: list[str] = []

    try:
        cargo = tomllib.loads(CARGO.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"RUST SOURCE AUDIT: FAIL: cannot parse Cargo.toml: {exc}")
        return 2
    try:
        project = json.loads(PROJECT.read_text(encoding="utf-8"))
    except Exception as exc:
        print(f"RUST SOURCE AUDIT: FAIL: cannot parse project metadata: {exc}")
        return 2

    package = cargo.get("package") or {}
    deps = cargo.get("dependencies") or {}
    engine = project.get("engine") or {}
    gui = project.get("gui") or {}

    source_version = str(project.get("sourceVersion", "")).strip()
    bevy_version = str(engine.get("version", "")).strip()
    bevy_egui_version = str(gui.get("bevyEguiVersion", "")).strip()
    rust_min = str(engine.get("rustMinimum", "")).strip()
    forge_rev = str(gui.get("forgeGuiRevision", "")).strip()

    if package.get("name") != "havenwild_bevy":
        fail(errors, f"Cargo package name must be havenwild_bevy, got {package.get('name')!r}")
    if str(package.get("version", "")) != source_version:
        fail(errors, f"Cargo package version {package.get('version')!r} != sourceVersion {source_version!r}")
    if str(package.get("rust-version", "")) != rust_min:
        fail(errors, f"Cargo rust-version {package.get('rust-version')!r} != metadata rustMinimum {rust_min!r}")
    if str(package.get("edition", "")) != "2021":
        fail(errors, f"Cargo edition must remain 2021 for this lane, got {package.get('edition')!r}")

    if deps.get("bevy") != f"={bevy_version}":
        fail(errors, f"Bevy dependency must be exact ={bevy_version}, got {deps.get('bevy')!r}")
    if not bevy_egui_version:
        fail(errors, "project metadata gui.bevyEguiVersion is required")
    elif deps.get("bevy_egui") != f"={bevy_egui_version}":
        fail(errors, f"bevy_egui dependency must be exact ={bevy_egui_version}, got {deps.get('bevy_egui')!r}")

    for dep_name in ("forge_gui_chrome", "forge_gui_theme"):
        dep = deps.get(dep_name)
        if not isinstance(dep, dict):
            fail(errors, f"{dep_name} must be a pinned git dependency")
            continue
        if dep.get("rev") != forge_rev:
            fail(errors, f"{dep_name} rev {dep.get('rev')!r} != project ForgeGUI rev {forge_rev!r}")

    main_path = SRC / "main.rs"
    try:
        main_text = main_path.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"RUST SOURCE AUDIT: FAIL: cannot read src/main.rs: {exc}")
        return 2

    declared = set(re.findall(r"(?m)^mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$", main_text))
    source_modules = {p.stem for p in SRC.glob("*.rs") if p.name != "main.rs"}
    missing = sorted(declared - source_modules)
    orphan = sorted(source_modules - declared)
    if missing:
        fail(errors, "declared Rust modules missing files: " + ", ".join(missing))
    if orphan:
        fail(errors, "orphan Rust source files not declared by main.rs: " + ", ".join(orphan))

    required_modules = {
        "atomic_file", "canvas_rulers", "document", "editor_layout", "semantic_lab", "source_catalog",
        "terrain", "terrain_mapper", "terrain_resolver", "elizawy_review",
    }
    absent = sorted(required_modules - declared)
    if absent:
        fail(errors, "required authoring modules not declared: " + ", ".join(absent))

    review_source = (SRC / "elizawy_review.rs").read_text(encoding="utf-8") if (SRC / "elizawy_review.rs").is_file() else ""
    for required in ("evidence_only_zero_promotions", "original_sha_verified_rgba32_exact_match", "thread::spawn", "pub fn validate(&self)"):
        if required not in review_source:
            fail(errors, f"M2D review authority/nonblocking contract missing: {required}")
    for required in ("fn draw_elizawy_review_browser(", "state.review_browser.poll()", "draw_elizawy_review_browser(ui, state)"):
        if required not in main_text:
            fail(errors, f"M2D review browser not wired into native mapper: {required}")
    # M2D02-A honest canvas-first controls. These are structural checks only;
    # a native Cargo build and interactive Windows test remain independent gates.
    for required in (
        "tool: CanvasTool::Select", "fn draw_tool_rail(", "fn draw_layer_guide(",
        "fn draw_alpha_checker(", "fn sample_canvas_source(",
        "fn save_scene(", "state.canvas_controls_rect.contains(pointer)",
        "ui.add_enabled(false, egui::Button::new(\"Regenerate selection / scene\"))",
    ):
        if required not in main_text:
            fail(errors, f"M2D02-A scene-first safe UI contract missing: {required}")
    for retired_ui in ("Stamp source tile on click", "Save canvas draft"):
        if retired_ui in main_text:
            fail(errors, f"M2D02-A obsolete default action remains: {retired_ui}")
    layout_text = (SRC / "editor_layout.rs").read_text(encoding="utf-8")
    for required in ("pub layers_open: bool", "pub inspect_alpha: bool", "terrain_mapper.visible = false"):
        if required not in layout_text:
            fail(errors, f"M2D02-A local layout default missing: {required}")
    if "resolve_semantic_lab(&state.semantic_lab, &state.terrain_mapper)" not in main_text:
        fail(errors, "Studio UI is not wired to the shared terrain resolver")
    if "EguiPrimaryContextPass" not in main_text or "PrimaryEguiContext" not in main_text:
        fail(errors, "manual primary egui context contract is missing from main.rs")
    if "Semantic Save & Resolve arrives with the DG live-world pass" in main_text:
        fail(errors, "stale pre-DG03 canvas-save message remains in main.rs")

    # M2D02-A1: the actual pinned egui context exposes the egui_-prefixed
    # keyboard focus API. The old method caused Windows E0599 on three callsites.
    if ".wants_keyboard_input(" in main_text:
        fail(errors, "obsolete egui Context::wants_keyboard_input call: use egui_wants_keyboard_input")
    if main_text.count(".egui_wants_keyboard_input()") != 3:
        fail(errors, "M2D02-A must retain three keyboard-focus guards (Delete, Undo, Redo)")

    # M2D02-B1: a separate source-fingerprinted v2 draft, never a hidden
    # rewrite/activation of the legacy scene or an invented procedural generator.
    v2_text = (SRC / "scene_v2.rs").read_text(encoding="utf-8") if (SRC / "scene_v2.rs").is_file() else ""
    for required in (
        "pub const SCHEMA:", "pub struct SceneV2", "legacy_base: source.tiles",
        "structural_cells: Vec::new()", "pub struct SceneHistory",
        "pub enum SceneCommand", "pub struct RemovalTombstone",
        "pub fn verify_legacy_source", "write_atomic_new(draft_path, &bytes)",
        "fn canonical_v1_fixture_import_roundtrip_preserves_all_tiles_and_roles",
    ):
        if required not in v2_text:
            fail(errors, f"M2D02-B v2 draft or history guard missing: {required}")
    for required in (
        "mod scene_v2;", "migration_preview: Option<SceneV2>",
        "SceneV2::preview_import(source)",
        "save_new_draft(&source, &destination)",
        "Preview separate v2 scene draft", "Create separate v2 draft",
    ):
        if required not in main_text:
            fail(errors, f"M2D02-B native migration UI missing: {required}")
    if "pub fn write_atomic_new" not in (SRC / "atomic_file.rs").read_text(encoding="utf-8"):
        fail(errors, "M2D02-B create-new draft publisher missing")

    # M2D02-B2: rulers are a display-only overlay in canvas coordinates, not a dock.
    rulers = (SRC / "canvas_rulers.rs").read_text(encoding="utf-8") if (SRC / "canvas_rulers.rs").is_file() else ""
    for required in ("pub fn draw(", "pub fn blocks_scene_edit(", "fn major_tile_step(", "painter.with_clip_rect", "adaptive_ticks_remain_legible_as_zoom_changes"):
        if required not in rulers:
            fail(errors, f"M2D02-B2 ruler overlay / no-paint guard missing: {required}")
    for required in ("mod canvas_rulers;", "canvas_rulers::draw(&painter, canvas, scene, px", "canvas_rulers::blocks_scene_edit(state.canvas_rect", "Show canvas rulers (tile coordinates)"):
        if required not in main_text:
            fail(errors, f"M2D02-B2 ruler integration missing: {required}")
    if "pub rulers_visible: bool" not in layout_text or "rulers_visible: true" not in layout_text:
        fail(errors, "M2D02-B2 persistent-on default rulers setting missing")


    # M2D02-E: revert guessed D composition; only a pinned authored fixture is active,
    # not 29 original atlases pasted onto an infinite scene. The previous derived
    # atlas-board draft is preserved under its old path and never overwritten.
    for required in (
        'content/scenes/elizawy_mapping_certification.scene.json',
        'content/scenes/derived/elizawy_mapping_certification.source_exact.draft.json',
        'fn draw_assembled_objects(', 'fn hide_selected_visual_sample(',
        'mapping_lab.sample_zone_contains(pos)', 'Show reviewed-source markers',
        'let initial_pan = egui::Vec2::ZERO;',
        'scene.hidden_visual_samples', 'fn focus_mapping_zone(',
        'fn snapped_scene_edge(', 'visual_samples.len() != 0',
    ):
        if required not in main_text:
            fail(errors, f"M2D02-E source-exact lab/source correction integration missing: {required}")
    if 'fn draw_mapping_lab_markers(' in main_text or 'mapping_lab.boards' in main_text:
        fail(errors, "Legacy atlas-board canvas unexpectedly still in live editor")
    from subprocess import run
    result = run([sys.executable, str(ROOT / 'tools/build_mapping_certification_scene.py'), '--check'],
                 cwd=ROOT, capture_output=True, text=True, timeout=35)
    if result.returncode:
        fail(errors, 'M2D02-E source-exact fixture generator check failed: ' + (result.stdout + result.stderr)[-900:])
    result = run([sys.executable, str(ROOT / 'tools/m2d02c_lab_selftest.py')],
                 cwd=ROOT, capture_output=True, text=True, timeout=35)
    if result.returncode:
        fail(errors, 'M2D02-E no-guessed-composition fixture test failed: ' + (result.stdout + result.stderr)[-900:])

    # M2D02-F1: PIE is a sandbox of the currently authored, source-addressed scene.
    # A flattened demo screenshot is not an editable map or runtime asset authority.
    playtest_source = (SRC / "playtest.rs").read_text(encoding="utf-8") if (SRC / "playtest.rs").is_file() else ""
    for required in ("mod playtest;", "playtest: Option<PlaySession>",
                     "PlaySession::start(&source_exact_snapshot", "fn stop_playtest(",
                     "playtest.step(direction, time.delta_secs())", "snapshot: scene.clone()"):
        if required not in main_text + playtest_source:
            fail(errors, f"PIE source-snapshot/play lifecycle missing: {required}")
    if "fn provisional_walkable(" not in playtest_source or '"RiverWater"' not in playtest_source:
        fail(errors, "PIE provisional v1 traversal policy must be explicit and testable")
    result = run([sys.executable, str(ROOT / 'tools/pie_source_contract_selftest.py')],
                 cwd=ROOT, capture_output=True, text=True, timeout=35)
    if result.returncode:
        fail(errors, 'PIE scene-reference integrity selftest failed: ' + (result.stdout + result.stderr)[-900:])

    # M2D03-A: all original ElizaWy source PNGs are searchable and only requested
    # lazily. Scene V2 visual layers composite atop the unchanged original base and
    # PIE freezes the same visual layers. No synthetic object categories are approved.
    for required in (
        "fn load_source_sheet_on_demand(", "fn hydrate_pending_source(",
        "state.source_catalog.entries.iter()", "source_family_filter",
        "fn draw_source_exact_visual_layers(", "playtest_layered: Option<SceneV2>",
        "state.layered_scene.resolved_legacy_tile(index)",
        "SceneCommand::PaintCells { layer: state.active_layer",
        "state.layered_scene.save_editable_draft(",
        'let terrain_selection = source_path.starts_with("Terrain/")',
        'ui.add_enabled(terrain_selection, egui::Button::new("Assign selected as sprite"))',
        'ui.add_enabled(terrain_selection, egui::Button::new("Add selected tile as part"))',
        'ui.add_enabled(terrain_selection, egui::Button::new("Replace from selection"))',
    ):
        if required not in main_text:
            fail(errors, f"M2D03-A complete-source/layered-authoring contract missing: {required}")
    result = run([sys.executable, str(ROOT / 'tools/audit_elizawy_asset_consumption.py'), '--check'],
                 cwd=ROOT, capture_output=True, text=True, timeout=35)
    if result.returncode:
        fail(errors, 'M2D03-A original source consumption audit failed: '+(result.stdout+result.stderr)[-1000:])

    # egui 0.36 unified SidePanel/TopBottomPanel into Panel and moved top-level
    # panel/central layout onto a parent Ui. Catch the exact API drift that only
    # a native Cargo check exposed in v0.5.5 before it can escape another handoff.
    for deprecated in ("egui::SidePanel::", "egui::TopBottomPanel::"):
        if deprecated in main_text:
            fail(errors, f"deprecated egui 0.36 panel API remains in main.rs: {deprecated}")
    if "egui::Ui::new(" not in main_text or "havenwild.viewport.root" not in main_text:
        fail(errors, "egui 0.36 root viewport Ui contract is missing from main.rs")
    # Native Windows decorations own the title bar. Do not resurrect the old
    # in-client product chrome merely to satisfy a stale migration assertion.
    if 'egui::Panel::top("havenwild.product.chrome")' in main_text:
        fail(errors, "obsolete in-client title bar conflicts with native Windows chrome")
    if not re.search(r"let native_frame\s*=\s*true\s*;", main_text):
        fail(errors, "native Windows frame must be enabled by default")
    if "decorations: native_frame" not in main_text:
        fail(errors, "Bevy window must use the native-frame setting")
    for required_panel in (
        'egui::Panel::top("havenwild.action.chrome")',
        'egui::Panel::bottom("havenwild.status.chrome")',
        'egui::Panel::left("havenwild.terrain.mapper.left")',
        'egui::Panel::right("havenwild.terrain.mapper.right")',
        'egui::Panel::bottom("havenwild.terrain.mapper.bottom")',
    ):
        if required_panel not in main_text:
            fail(errors, f"egui 0.36 Panel migration contract missing: {required_panel}")
    if re.search(r"egui::CentralPanel::default\(\)[\s\S]{0,180}\.show\(ctx,", main_text):
        fail(errors, "CentralPanel is still being shown directly on Context instead of the root viewport Ui")

    scan_paths = [CARGO, PROJECT, ROOT / "ForgePY.py", ROOT / "ProjectControlCenter.py"]
    scan_paths.extend(sorted(SRC.glob("*.rs")))
    merge_tokens = ("<<<<<<< ", "=======", ">>>>>>> ")
    for path in scan_paths:
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as exc:
            fail(errors, f"cannot read {path.relative_to(ROOT)}: {exc}")
            continue
        for token in merge_tokens:
            if token in text:
                fail(errors, f"merge-conflict marker {token.strip()!r} in {path.relative_to(ROOT)}")
                break

    # The runtime-facing resolver is expected to remain UI-agnostic: it may depend
    # on semantic/topology/recipe state, but never on egui/Bevy rendering APIs.
    resolver_path = SRC / "terrain_resolver.rs"
    resolver_text = resolver_path.read_text(encoding="utf-8") if resolver_path.is_file() else ""
    for forbidden in ("egui::", "bevy::", "EguiContexts", "Handle<Image>"):
        if forbidden in resolver_text:
            fail(errors, f"shared terrain_resolver.rs leaked UI/render dependency {forbidden!r}")

    print("HAVENWILD RUST SOURCE STRUCTURAL AUDIT")
    print(f"Source      : {source_version or '-'}")
    print(f"Modules     : {len(source_modules)} source modules / {len(declared)} declared")
    print(f"Bevy        : {bevy_version or '-'}")
    print(f"bevy_egui   : {bevy_egui_version or '-'}")
    print(f"Rust minimum: {rust_min or '-'}")
    print(f"ForgeGUI    : {forge_rev[:12] if forge_rev else '-'}")
    for warning in warnings:
        print("[WARN]", warning)
    for error in errors:
        print("[FAIL]", error)
    if errors:
        print(f"RUST SOURCE AUDIT: FAIL ({len(errors)} issue(s))")
        return 3
    print("RUST SOURCE AUDIT: PASS / Cargo-free structural checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
