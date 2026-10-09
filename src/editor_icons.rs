//! Unified 24-unit monochrome vector icons for the editor tool and layer rails.
//! Icons are runtime geometry (no icon font, generated art, or external assets).
use bevy_egui::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RailIcon {
    Select,
    Pixel,
    Tile,
    Terrain,
    Erase,
    Sample,
    Bridge,
    Panel,
    Library,
    Layers,
    Apps,
    Eye,
    EyeOff,
    Foreground,
    Objects,
    Structures,
    Elevation,
    Water,
    Details,
    Ground,
    Collision,
}

fn paint_icon(
    painter: &egui::Painter,
    icon: RailIcon,
    center: egui::Pos2,
    scale: f32,
    ink: egui::Color32,
) {
    let pt = |x: f32, y: f32| center + egui::vec2(x * scale, y * scale);
    let stroke = egui::Stroke::new(1.8 * scale, ink);
    let thin = egui::Stroke::new(1.45 * scale, ink);
    let mut path = |xy: &[(f32, f32)]| {
        painter.add(egui::Shape::line(
            xy.iter().map(|&(x, y)| pt(x, y)).collect(),
            stroke,
        ));
    };
    let mut rectangle = |x: f32, y: f32, w: f32, h: f32, radius: f32| {
        painter.rect_stroke(
            egui::Rect::from_min_size(pt(x, y), egui::vec2(w * scale, h * scale)),
            radius * scale,
            thin,
            egui::StrokeKind::Inside,
        );
    };
    match icon {
        RailIcon::Select => {
            path(&[
                (-8.0, -9.0),
                (-8.0, 8.0),
                (-3.5, 3.0),
                (0.5, 10.0),
                (3.5, 8.6),
                (-0.5, 2.0),
                (7.0, 1.5),
                (-8.0, -9.0),
            ]);
        }
        RailIcon::Pixel => {
            // Pixel pencil, with clearly distinguishable square tip rather than a brush.
            path(&[
                (-8.0, 6.0),
                (-7.0, 9.0),
                (-4.0, 8.0),
                (7.5, -3.5),
                (3.5, -7.5),
                (-8.0, 3.5),
                (-8.0, 6.0),
            ]);
            path(&[(1.8, -5.8), (5.8, -1.8)]);
            rectangle(5.0, 5.0, 3.5, 3.5, 0.0);
        }
        RailIcon::Tile => {
            // Explicit 2x2 source-tile stamp. This is not the Ground layer glyph.
            for &(x, y) in &[(-8.5, -8.5), (1.0, -8.5), (-8.5, 1.0), (1.0, 1.0)] {
                rectangle(x, y, 7.5, 7.5, 1.0);
            }
            painter.rect_filled(
                egui::Rect::from_min_size(pt(2.8, 2.8), egui::vec2(3.9 * scale, 3.9 * scale)),
                0.5,
                ink,
            );
        }
        RailIcon::Terrain => {
            // One readable contour/mountain icon; not the Pixel pencil.
            path(&[
                (-9.5, 6.5),
                (-3.3, -5.5),
                (0.4, 0.0),
                (3.7, -8.0),
                (10.0, 6.5),
            ]);
            path(&[(-9.0, 9.0), (-2.0, 9.0), (1.0, 6.5), (4.0, 9.0), (9.0, 9.0)]);
        }
        RailIcon::Erase => {
            path(&[
                (-8.0, 1.0),
                (0.0, -8.0),
                (9.0, 0.0),
                (1.5, 8.0),
                (-8.0, 1.0),
            ]);
            path(&[(-4.0, 5.0), (4.5, -4.0)]);
            path(&[(-8.0, 10.0), (9.0, 10.0)]);
        }
        RailIcon::Sample => {
            painter.circle_stroke(pt(6.3, -6.3), 2.8 * scale, thin);
            path(&[
                (4.6, -4.5),
                (-5.0, 5.1),
                (-7.8, 6.4),
                (-6.4, 9.0),
                (-3.5, 7.6),
                (6.2, -2.4),
            ]);
        }
        RailIcon::Bridge => {
            // Purpose-specific bridge rather than a duplicated Structures house.
            path(&[(-10.0, -4.0), (10.0, -4.0)]);
            path(&[(-10.0, 0.0), (10.0, 0.0)]);
            path(&[
                (-7.0, 0.0),
                (-7.0, 8.0),
                (-1.0, 3.0),
                (1.0, 3.0),
                (7.0, 8.0),
                (7.0, 0.0),
            ]);
            path(&[(-9.0, 9.0), (-4.0, 9.0)]);
            path(&[(4.0, 9.0), (9.0, 9.0)]);
        }
        RailIcon::Panel => {
            // Dedicated contextual-panel identity instead of reusing Foreground.
            rectangle(-9.0, -8.0, 18.0, 16.0, 2.0);
            path(&[(-2.5, -8.0), (-2.5, 8.0)]);
            path(&[(-7.0, -4.5), (-4.0, -4.5)]);
            path(&[(-7.0, 0.0), (-4.0, 0.0)]);
        }
        RailIcon::Layers => {
            path(&[
                (-9.0, -3.5),
                (0.0, -8.0),
                (9.0, -3.5),
                (0.0, 1.0),
                (-9.0, -3.5),
            ]);
            path(&[(-9.0, 1.0), (0.0, 5.5), (9.0, 1.0)]);
            path(&[(-9.0, 5.0), (0.0, 9.5), (9.0, 5.0)]);
        }
        RailIcon::Library => {
            rectangle(-9.0, -7.0, 18.0, 15.0, 1.5);
            path(&[(-9.0, -2.0), (9.0, -2.0)]);
            path(&[(-5.0, 2.5), (-1.5, 2.5), (-1.5, 5.5), (-5.0, 5.5)]);
            path(&[(2.0, 2.5), (5.5, 2.5), (5.5, 5.5), (2.0, 5.5)]);
        }
        RailIcon::Apps => {
            for &(x, y) in &[(-7.5, -7.5), (1.5, -7.5), (-7.5, 1.5), (1.5, 1.5)] {
                rectangle(x, y, 6.0, 6.0, 1.4);
            }
        }
        RailIcon::Eye | RailIcon::EyeOff => {
            path(&[
                (-9.0, 0.0),
                (-5.0, -4.5),
                (0.0, -6.0),
                (5.0, -4.5),
                (9.0, 0.0),
                (5.0, 4.5),
                (0.0, 6.0),
                (-5.0, 4.5),
                (-9.0, 0.0),
            ]);
            painter.circle_stroke(pt(0.0, 0.0), 2.5 * scale, thin);
            if icon == RailIcon::EyeOff {
                path(&[(-9.5, -9.0), (9.5, 9.0)]);
            }
        }
        RailIcon::Foreground => {
            path(&[
                (-8.5, -3.0),
                (0.0, -8.0),
                (8.5, -3.0),
                (0.0, 2.0),
                (-8.5, -3.0),
            ]);
            path(&[(-8.5, 2.0), (0.0, 7.0), (8.5, 2.0)]);
            path(&[(-8.5, 6.0), (0.0, 11.0), (8.5, 6.0)]);
        }
        RailIcon::Objects => {
            painter.circle_stroke(pt(-2.0, -2.0), 5.0 * scale, thin);
            path(&[(-2.0, 3.0), (-2.0, 9.0)]);
            path(&[(-6.5, 9.0), (3.0, 9.0)]);
            path(&[(-2.0, 5.5), (3.0, 2.0)]);
        }
        RailIcon::Structures => {
            path(&[(-9.0, -1.5), (0.0, -9.0), (9.0, -1.5)]);
            path(&[(-7.0, -3.0), (-7.0, 9.0), (7.0, 9.0), (7.0, -3.0)]);
            rectangle(-2.0, 2.0, 4.0, 7.0, 0.0);
        }
        RailIcon::Elevation => {
            path(&[
                (-10.0, 8.0),
                (-2.7, -5.0),
                (0.0, -1.0),
                (3.0, -9.0),
                (10.0, 8.0),
                (-10.0, 8.0),
            ]);
        }
        RailIcon::Water => {
            for y in [-5.0, 1.0, 7.0] {
                path(&[
                    (-9.0, y + 1.0),
                    (-4.5, y - 1.5),
                    (0.0, y + 1.0),
                    (4.5, y - 1.5),
                    (9.0, y + 1.0),
                ]);
            }
        }
        RailIcon::Details => {
            for &(x, y, r) in &[(-6.0, -5.0, 1.8), (4.5, -2.0, 2.1), (-1.5, 6.0, 1.5)] {
                painter.circle_stroke(pt(x, y), r * scale, thin);
            }
        }
        RailIcon::Ground => {
            rectangle(-9.0, -9.0, 18.0, 18.0, 1.0);
            path(&[(0.0, -9.0), (0.0, 9.0)]);
            path(&[(-9.0, 0.0), (9.0, 0.0)]);
        }
        RailIcon::Collision => {
            path(&[
                (0.0, -9.0),
                (8.0, -6.0),
                (7.0, 3.0),
                (0.0, 10.0),
                (-7.0, 3.0),
                (-8.0, -6.0),
                (0.0, -9.0),
            ]);
        }
    }
}

/// The tool rail is one restrained icon-only surface: no permanent rows of outlined
/// boxes, no double-green visibility columns. Hover/selection remain explicit.
pub(crate) fn rail_icon_button(
    ui: &mut egui::Ui,
    icon: RailIcon,
    selected: bool,
    hover: &str,
    size: [f32; 2],
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(size[0], size[1]), egui::Sense::click());
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect.shrink(1.0), 6.0, egui::Color32::from_rgb(29, 76, 68));
        painter.rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height())),
            0.0,
            egui::Color32::from_rgb(97, 210, 177),
        );
    } else if response.hovered() {
        painter.rect_filled(rect.shrink(1.0), 6.0, egui::Color32::from_rgb(49, 58, 65));
    }
    let ink = if selected {
        egui::Color32::from_rgb(169, 246, 216)
    } else if response.hovered() {
        egui::Color32::from_rgb(242, 245, 246)
    } else {
        egui::Color32::from_rgb(184, 198, 203)
    };
    let scale = if size[0] <= 24.0 {
        0.66
    } else if size[0] <= 30.0 {
        0.82
    } else {
        0.98
    };
    paint_icon(painter, icon, rect.center(), scale, ink);
    response.on_hover_text(hover)
}
