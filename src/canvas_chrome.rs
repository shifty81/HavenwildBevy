//! Shared opaque chrome for controls drawn *within* the authoring canvas.
//! World/Scene use the same geometry and theme tokens; game art is never tinted.
use bevy_egui::egui;

pub const THEME_NAMES: [&str; 3] = ["North Star", "Slate", "High Contrast"];

/// These rectangles must be registered before canvas input hit-testing; the
/// central Ui draws its widgets afterward without an independent floating Area.
#[derive(Clone, Copy, Debug)]
pub struct OverlayGeometry {
    pub grid: egui::Rect,
    pub zoom: egui::Rect,
}

pub fn overlay_geometry(canvas: egui::Rect, ruler_top: f32) -> OverlayGeometry {
    let top = canvas.top() + ruler_top + 8.0;
    let grid = if canvas.width() >= 660.0 {
        egui::Rect::from_min_size(
            egui::pos2(canvas.center().x - 105.0, top),
            egui::vec2(210.0, 37.0),
        )
    } else {
        egui::Rect::NOTHING
    };
    let zoom = if canvas.width() >= 330.0 {
        egui::Rect::from_min_size(
            egui::pos2(canvas.right() - 202.0, top),
            egui::vec2(194.0, 37.0),
        )
    } else {
        egui::Rect::NOTHING
    };
    OverlayGeometry { grid, zoom }
}

pub fn panel_fill(theme: &str) -> egui::Color32 {
    match theme {
        "Slate" => egui::Color32::from_rgb(38, 43, 51),
        "High Contrast" => egui::Color32::from_rgb(10, 12, 17),
        _ => egui::Color32::from_rgb(27, 34, 39),
    }
}

fn hover_fill(theme: &str) -> egui::Color32 {
    match theme {
        "Slate" => egui::Color32::from_rgb(66, 75, 88),
        "High Contrast" => egui::Color32::from_rgb(49, 64, 81),
        _ => egui::Color32::from_rgb(49, 73, 83),
    }
}

fn active_fill(theme: &str) -> egui::Color32 {
    match theme {
        "Slate" => egui::Color32::from_rgb(66, 97, 132),
        "High Contrast" => egui::Color32::from_rgb(18, 113, 178),
        _ => egui::Color32::from_rgb(27, 109, 117),
    }
}

fn border(theme: &str) -> egui::Color32 {
    match theme {
        "High Contrast" => egui::Color32::from_rgb(183, 205, 225),
        "Slate" => egui::Color32::from_rgb(91, 104, 120),
        _ => egui::Color32::from_rgb(62, 85, 95),
    }
}

/// All canvas groups have fully opaque backgrounds; the world never shows
/// through button backing. The exact visual grouping follows the editor theme.
pub fn tool_group(ui: &mut egui::Ui, theme: &str, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::NONE
        .fill(panel_fill(theme))
        .stroke(egui::Stroke::new(1.0, border(theme)))
        .corner_radius(5.0)
        .inner_margin(3)
        .show(ui, content);
}

pub fn action(ui: &mut egui::Ui, label: &str, hint: &str, theme: &str) -> bool {
    ui.add(
        egui::Button::new(label)
            .fill(panel_fill(theme))
            .stroke(egui::Stroke::new(1.0, border(theme))),
    )
    .on_hover_text(hint)
    .clicked()
}

pub fn toggle(ui: &mut egui::Ui, label: &str, active: bool, hint: &str, theme: &str) -> bool {
    ui.add(
        egui::Button::new(label)
            .fill(if active {
                active_fill(theme)
            } else {
                panel_fill(theme)
            })
            .stroke(egui::Stroke::new(1.0, border(theme))),
    )
    .on_hover_text(hint)
    .clicked()
}

/// Normalize all regular egui widgets (menus, docks, dialogs, Inspector) using
/// semantic colors after ForgeGUI installs the common baseline theme.
pub fn apply_theme(ctx: &egui::Context, theme: &str) {
    let mut visuals = ctx.style_of(egui::Theme::Dark).visuals.clone();
    visuals.panel_fill = panel_fill(theme);
    visuals.window_fill = panel_fill(theme);
    visuals.extreme_bg_color = match theme {
        "High Contrast" => egui::Color32::from_rgb(0, 0, 0),
        "Slate" => egui::Color32::from_rgb(27, 31, 37),
        _ => egui::Color32::from_rgb(19, 27, 34),
    };
    visuals.widgets.noninteractive.bg_fill = panel_fill(theme);
    visuals.widgets.inactive.bg_fill = panel_fill(theme);
    visuals.widgets.hovered.bg_fill = hover_fill(theme);
    visuals.widgets.active.bg_fill = active_fill(theme);
    visuals.selection.bg_fill = active_fill(theme);
    ctx.set_visuals(visuals);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlay_zones_do_not_overlap_or_escape_a_desktop_canvas() {
        let canvas = egui::Rect::from_min_size(egui::pos2(335.0, 56.0), egui::vec2(1250.0, 660.0));
        let z = overlay_geometry(canvas, 0.0);
        assert!(canvas.contains(z.grid.min));
        assert!(canvas.contains(z.zoom.max));
        assert!(!z.grid.intersects(z.zoom));
    }
    #[test]
    fn compact_canvas_hides_grid_before_groups_can_overlap() {
        let canvas = egui::Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(430.0, 500.0));
        let z = overlay_geometry(canvas, 0.0);
        assert_eq!(z.grid, egui::Rect::NOTHING);
        assert!(canvas.contains(z.zoom.min));
    }
    #[test]
    fn grid_hides_at_intermediate_width_without_hitting_zoom() {
        for width in [330.0, 430.0, 570.0, 639.0] {
            let canvas =
                egui::Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(width, 500.0));
            let zones = overlay_geometry(canvas, 0.0);
            assert_eq!(zones.grid, egui::Rect::NOTHING);
        }
        let canvas = egui::Rect::from_min_size(egui::pos2(50.0, 50.0), egui::vec2(640.0, 500.0));
        let zones = overlay_geometry(canvas, 0.0);
        assert!(!zones.grid.intersects(zones.zoom));
    }
    #[test]
    fn themes_and_overlay_chrome_are_fully_opaque() {
        for name in THEME_NAMES {
            assert_eq!(panel_fill(name).a(), 255);
            assert_eq!(active_fill(name).a(), 255);
        }
    }
}
