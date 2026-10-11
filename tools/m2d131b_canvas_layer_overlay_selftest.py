#!/usr/bin/env python3
"""M2D131B source contracts. UI and GPU acceptance still require a native run."""
from pathlib import Path
import json
import re

root = Path(__file__).resolve().parents[1]
main = (root / "src/main.rs").read_text(encoding="utf-8")
layout = (root / "src/editor_layout.rs").read_text(encoding="utf-8")
layer = (root / "src/layer_overlay.rs").read_text(encoding="utf-8")
metadata = json.loads((root / "project/forgepy.project.json").read_text("utf-8"))
assert metadata["sourceVersion"] == "0.10.57"
assert 'version = "0.10.57"' in (root / "Cargo.toml").read_text()

# Both tiny buttons are in the same canvas-top-left group, visible collapsed or open.
overlay = main.split("fn draw_layer_rail(", 1)[1].split("fn docked_overlay_window<", 1)[0]
assert re.search(r"ui\.horizontal\(\|ui\|\s*\{[\s\S]*?RailIcon::Layers,[\s\S]*?RailIcon::Collision", overlay)
assert overlay.index("RailIcon::Collision") < overlay.index("if !expanded")
assert overlay.count("RailIcon::Collision") == 1
assert "state.editor_layout.collision_overlay = !state.editor_layout.collision_overlay" in overlay
assert "state.editor_layout.structural_open" not in overlay, "collision visibility may not open property windows"
assert "state.collision_world_pixel_edit" not in overlay, "collision view may not enter paint mode"
assert "state.active_layer = id" in overlay
assert "state.editor_layout.world_layer_visibility[slot] = !visible" in overlay
assert "state.layer_isolate.toggle" in overlay and "state.layer_isolate.cancel()" in overlay
assert "Shift-click to isolate" in overlay and "Shift-click again to restore" in overlay
assert "canvas_chrome::panel_fill(&state.editor_layout.editor_theme)" in overlay, "opaque shared theme-owned UI backing required"

# Exclusivity: World and Scene use one shared hit-test exclusion zone.
for token in ("state.canvas_layers_rect.contains(pointer)", "state.editor_layout.layers_dock_open,"):
    assert main.count(token) == 2, token
assert "let width = 84.0;" in main
assert re.search(r"if\s+expanded\s*\{\s*282\.0\s*\}\s*else\s*\{\s*40\.0\s*\}", main)
# Native state-only unit tests ensure isolation restores pre-existing hidden flags.
for token in ("fn shift_isolate_restores_exact_hidden_layer_preferences", "fn manual_eye_toggle_prevents_restoring_stale_snapshot", "fn invalid_slot_cannot_change_visibility"):
    assert token in layer

# Existing 7-layer renderer/picking authority stays intact, not duplicated.
for token in ("world_layer_visible(state, object.layer)", "world_layer_visible(state, cached.layer)",
              "world_layer_visible(state, LayerId::Ground)", "default_world_layer_visibility"):
    assert token in (main + layout), token

# Right dock must reserve both halves, even for an empty/short Inspector.
inspector = main.split("fn draw_inspector_rail(", 1)[1].split("fn canvas_layer_rect(", 1)[0]
assert "state.editor_layout.inspector_split_ratio" in inspector
assert "split_available * (1.0 - state.editor_layout.inspector_split_ratio)" in inspector
assert "auto_shrink([false, false])" in inspector
assert "draw_authoring_library(ui, state)" in inspector
assert "grip.drag_delta().y / split_available" in inspector
assert "inspector_split_ratio.clamp(0.20, 0.80)" in layout
assert "if !layout.canvas_overlay_v1_initialized" in layout
assert "layout.library_in_inspector = true" in layout
assert "if !layout.library_floating" in layout
for token in ("fn m2d131b_right_browser_migration_defaults_to_half_and_runs_once", "fn m2d131b_explicit_floating_library_is_not_silently_redocked"):
    assert token in layout
# Only one inspector-side asset browser, no duplicate new Panel.
assert inspector.count("draw_authoring_library(ui, state)") == 1
print("M2D131B CANVAS/LAYER: PASS / adjacent view-only collision, reversible Shift-isolate, opaque click-safe HUD, right 50/50 split, one-time migration")
