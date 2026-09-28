//! Non-document canvas chrome: pan/zoom-aware tile rulers embedded over the full canvas.
//! Rulers are display-only. They do not modify scene pixels, tile IDs, metadata, or layout size.
use bevy_egui::egui;

pub const TOP: f32 = 24.0;
pub const LEFT: f32 = 32.0;

/// Focus Mode intentionally hides all ruler chrome, without changing the user's preference.
pub fn visible(preference: bool, focus_mode: bool) -> bool {
    preference && !focus_mode
}

/// Ruler bands are visual overlays, not source-art painting targets.
pub fn blocks_scene_edit(canvas: egui::Rect, pointer: egui::Pos2, shown: bool) -> bool {
    shown
        && canvas.contains(pointer)
        && (pointer.x < canvas.left() + LEFT || pointer.y < canvas.top() + TOP)
}

/// Adaptive major divisions; the labels remain separated even at very low zoom.
fn major_tile_step(tile_screen_px: f32) -> i64 {
    if !tile_screen_px.is_finite() || tile_screen_px <= 0.0 {
        return 1;
    }
    let mut magnitude = 1_i64;
    loop {
        for multiplier in [1_i64, 2, 5] {
            let step = magnitude.saturating_mul(multiplier);
            if (step as f32) * tile_screen_px >= 72.0 {
                return step;
            }
        }
        if magnitude > 1_000_000_000 {
            return magnitude;
        }
        magnitude *= 10;
    }
}

pub fn draw(
    painter: &egui::Painter,
    canvas: egui::Rect,
    scene: egui::Rect,
    tile_screen_px: f32,
    pointer: Option<egui::Pos2>,
    selected: Option<[usize; 2]>,
) {
    if !tile_screen_px.is_finite()
        || tile_screen_px <= 0.0
        || canvas.width() <= LEFT + 12.0
        || canvas.height() <= TOP + 12.0
    {
        return;
    }
    let top = egui::Rect::from_min_max(canvas.min, egui::pos2(canvas.right(), canvas.top() + TOP));
    let left = egui::Rect::from_min_max(
        egui::pos2(canvas.left(), canvas.top() + TOP),
        egui::pos2(canvas.left() + LEFT, canvas.bottom()),
    );
    let corner = egui::Rect::from_min_size(canvas.min, egui::vec2(LEFT, TOP));
    let background = egui::Color32::from_rgba_unmultiplied(26, 34, 36, 212);
    painter.rect_filled(top, 0.0, background);
    painter.rect_filled(left, 0.0, background);
    painter.rect_filled(
        corner,
        0.0,
        egui::Color32::from_rgba_unmultiplied(31, 43, 45, 231),
    );
    let border = egui::Stroke::new(0.8, egui::Color32::from_gray(95));
    painter.line_segment(
        [
            egui::pos2(canvas.left(), top.bottom()),
            egui::pos2(canvas.right(), top.bottom()),
        ],
        border,
    );
    painter.line_segment(
        [
            egui::pos2(left.right(), canvas.top()),
            egui::pos2(left.right(), canvas.bottom()),
        ],
        border,
    );
    painter.text(
        corner.center(),
        egui::Align2::CENTER_CENTER,
        "T",
        egui::FontId::monospace(11.0),
        egui::Color32::from_gray(220),
    );

    let step = major_tile_step(tile_screen_px);
    let ink = egui::Color32::from_gray(202);
    let tick_ink = egui::Color32::from_gray(145);
    let top_painter = painter.with_clip_rect(egui::Rect::from_min_max(
        egui::pos2(left.right(), top.top()),
        top.max,
    ));
    let left_painter = painter.with_clip_rect(left);

    // Coordinates are always scene tile boundaries. They do not reset when zooming,
    // and negative/off-scene coordinates continue to track a panned infinite workspace.
    let first_x = ((left.right() - scene.left()) / tile_screen_px).floor() as i64;
    let last_x = ((top.right() - scene.left()) / tile_screen_px).ceil() as i64;
    let first_y = ((left.top() - scene.top()) / tile_screen_px).floor() as i64;
    let last_y = ((left.bottom() - scene.top()) / tile_screen_px).ceil() as i64;
    if tile_screen_px >= 12.0 {
        for tile in first_x..=last_x {
            if tile.rem_euclid(step) == 0 {
                continue;
            }
            let x = scene.left() + tile as f32 * tile_screen_px;
            top_painter.line_segment(
                [
                    egui::pos2(x, top.bottom() - 4.0),
                    egui::pos2(x, top.bottom() - 1.0),
                ],
                egui::Stroke::new(0.8, tick_ink),
            );
        }
        for tile in first_y..=last_y {
            if tile.rem_euclid(step) == 0 {
                continue;
            }
            let y = scene.top() + tile as f32 * tile_screen_px;
            left_painter.line_segment(
                [
                    egui::pos2(left.right() - 4.0, y),
                    egui::pos2(left.right() - 1.0, y),
                ],
                egui::Stroke::new(0.8, tick_ink),
            );
        }
    }
    let mut x_tile = first_x.div_euclid(step) * step;
    while x_tile <= last_x {
        let x = scene.left() + x_tile as f32 * tile_screen_px;
        top_painter.line_segment(
            [
                egui::pos2(x, top.bottom() - 8.0),
                egui::pos2(x, top.bottom() - 1.0),
            ],
            egui::Stroke::new(1.0, ink),
        );
        top_painter.text(
            egui::pos2(x + 3.0, top.top() + 7.0),
            egui::Align2::LEFT_CENTER,
            x_tile.to_string(),
            egui::FontId::monospace(10.0),
            ink,
        );
        x_tile += step;
    }
    let mut y_tile = first_y.div_euclid(step) * step;
    while y_tile <= last_y {
        let y = scene.top() + y_tile as f32 * tile_screen_px;
        left_painter.line_segment(
            [
                egui::pos2(left.right() - 8.0, y),
                egui::pos2(left.right() - 1.0, y),
            ],
            egui::Stroke::new(1.0, ink),
        );
        left_painter.text(
            egui::pos2(left.right() - 9.0, y + 6.0),
            egui::Align2::RIGHT_CENTER,
            y_tile.to_string(),
            egui::FontId::monospace(10.0),
            ink,
        );
        y_tile += step;
    }
    // Selected-cell markers are independent of hover and remain stable across pan/zoom.
    if let Some([sx, sy]) = selected {
        let x = scene.left() + (sx as f32 + 0.5) * tile_screen_px;
        let y = scene.top() + (sy as f32 + 0.5) * tile_screen_px;
        let selected_ink = egui::Stroke::new(2.0, egui::Color32::from_rgb(235, 195, 95));
        top_painter.line_segment(
            [
                egui::pos2(x, top.top() + 1.0),
                egui::pos2(x, top.bottom() - 1.0),
            ],
            selected_ink,
        );
        left_painter.line_segment(
            [
                egui::pos2(left.left() + 1.0, y),
                egui::pos2(left.right() - 1.0, y),
            ],
            selected_ink,
        );
    }
    if let Some(cursor) = pointer.filter(|pos| canvas.contains(*pos)) {
        if !blocks_scene_edit(canvas, cursor, true) {
            let hover = egui::Stroke::new(1.0, egui::Color32::from_rgb(135, 210, 220));
            top_painter.line_segment(
                [
                    egui::pos2(cursor.x, top.top() + 1.0),
                    egui::pos2(cursor.x, top.bottom() - 1.0),
                ],
                hover,
            );
            left_painter.line_segment(
                [
                    egui::pos2(left.left() + 1.0, cursor.y),
                    egui::pos2(left.right() - 1.0, cursor.y),
                ],
                hover,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_ticks_remain_legible_as_zoom_changes() {
        for px in [4.0, 8.0, 12.0, 22.4, 32.0, 64.0, 192.0] {
            let step = major_tile_step(px);
            assert!((step as f32) * px >= 72.0);
            assert!(
                step == 1 || step == 2 || step == 5 || {
                    let mut test = step;
                    while test % 10 == 0 {
                        test /= 10;
                    }
                    [1, 2, 5].contains(&test)
                }
            );
        }
    }
    #[test]
    fn overlay_does_not_steal_editing_space_outside_its_bands() {
        let rect = egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(800.0, 500.0));
        assert!(blocks_scene_edit(rect, egui::pos2(110.0, 250.0), true));
        assert!(blocks_scene_edit(rect, egui::pos2(250.0, 110.0), true));
        assert!(!blocks_scene_edit(rect, egui::pos2(250.0, 250.0), true));
        assert!(!blocks_scene_edit(rect, egui::pos2(110.0, 110.0), false));
        assert!(visible(true, false));
        assert!(!visible(true, true));
    }
}
