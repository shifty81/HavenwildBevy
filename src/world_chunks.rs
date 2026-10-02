//! Editor-only world-grid overlay and global-coordinate conversion. The current
//! 40x28 Summer document remains a single fixture: surrounding chunk lines do
//! not materialize neighboring chunks or claim a streaming world implementation.
use bevy_egui::egui;

pub const DEFAULT_CHUNK_TILES: usize = 32;
pub const ALLOWED_CHUNK_TILES: [usize; 3] = [16, 32, 64];

pub fn chunk_and_local(world: [i64; 2], side: usize) -> ([i64; 2], [usize; 2]) {
    assert!(ALLOWED_CHUNK_TILES.contains(&side));
    let n = side as i64;
    (
        [world[0].div_euclid(n), world[1].div_euclid(n)],
        [
            world[0].rem_euclid(n) as usize,
            world[1].rem_euclid(n) as usize,
        ],
    )
}

pub fn draw(
    painter: &egui::Painter,
    canvas: egui::Rect,
    scene: egui::Rect,
    px: f32,
    scene_size: [usize; 2],
    side: usize,
    chunks: bool,
    tiles: bool,
) {
    if !px.is_finite() || px <= 0.0 || !ALLOWED_CHUNK_TILES.contains(&side) {
        return;
    }
    let clipped = painter.with_clip_rect(canvas);
    if tiles && px >= 8.0 {
        let inside = canvas.intersect(scene);
        if inside.width() > 0.0 && inside.height() > 0.0 {
            let p = clipped.with_clip_rect(inside);
            let stroke = egui::Stroke::new(
                0.55,
                egui::Color32::from_rgba_unmultiplied(203, 227, 215, 46),
            );
            for x in 0..=scene_size[0] {
                let sx = scene.left() + x as f32 * px;
                if sx >= inside.left() && sx <= inside.right() {
                    p.line_segment(
                        [
                            egui::pos2(sx, inside.top()),
                            egui::pos2(sx, inside.bottom()),
                        ],
                        stroke,
                    );
                }
            }
            for y in 0..=scene_size[1] {
                let sy = scene.top() + y as f32 * px;
                if sy >= inside.top() && sy <= inside.bottom() {
                    p.line_segment(
                        [
                            egui::pos2(inside.left(), sy),
                            egui::pos2(inside.right(), sy),
                        ],
                        stroke,
                    );
                }
            }
        }
    }
    if !chunks {
        return;
    }
    let unit = side as f32 * px;
    let first_x = ((canvas.left() - scene.left()) / unit).floor() as i64;
    let last_x = ((canvas.right() - scene.left()) / unit).ceil() as i64;
    let first_y = ((canvas.top() - scene.top()) / unit).floor() as i64;
    let last_y = ((canvas.bottom() - scene.top()) / unit).ceil() as i64;
    let stroke = egui::Stroke::new(
        1.15,
        egui::Color32::from_rgba_unmultiplied(99, 209, 179, 170),
    );
    for cx in first_x..=last_x {
        let x = scene.left() + cx as f32 * unit;
        clipped.line_segment(
            [egui::pos2(x, canvas.top()), egui::pos2(x, canvas.bottom())],
            stroke,
        );
    }
    for cy in first_y..=last_y {
        let y = scene.top() + cy as f32 * unit;
        clipped.line_segment(
            [egui::pos2(canvas.left(), y), egui::pos2(canvas.right(), y)],
            stroke,
        );
    }
    if unit >= 90.0 {
        for cy in first_y..last_y {
            for cx in first_x..last_x {
                let pos = egui::pos2(
                    scene.left() + cx as f32 * unit + 5.0,
                    scene.top() + cy as f32 * unit + 5.0,
                );
                if canvas.contains(pos) {
                    clipped.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        format!("CH {cx},{cy}"),
                        egui::FontId::monospace(10.0),
                        egui::Color32::from_rgb(163, 230, 210),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_and_positive_coordinates_use_euclidean_chunks() {
        assert_eq!(chunk_and_local([0, 0], 32), ([0, 0], [0, 0]));
        assert_eq!(chunk_and_local([31, 31], 32), ([0, 0], [31, 31]));
        assert_eq!(chunk_and_local([32, 0], 32), ([1, 0], [0, 0]));
        assert_eq!(chunk_and_local([-1, -32], 32), ([-1, -1], [31, 0]));
        assert_eq!(chunk_and_local([-33, 63], 32), ([-2, 1], [31, 31]));
    }
}

/// Editor-only grid overlay for an arbitrary signed-coordinate materialized region.
pub fn draw_materialized(
    painter: &egui::Painter,
    canvas: egui::Rect,
    region: egui::Rect,
    px: f32,
    origin_world: [i64; 2],
    region_size: [usize; 2],
    side: usize,
    chunks: bool,
    tiles: bool,
) {
    if !px.is_finite() || px <= 0.0 || !ALLOWED_CHUNK_TILES.contains(&side) {
        return;
    }
    let clipped = painter.with_clip_rect(canvas);
    let inside = canvas.intersect(region);
    if tiles && px >= 8.0 && inside.width() > 0.0 && inside.height() > 0.0 {
        let p = clipped.with_clip_rect(inside);
        let stroke = egui::Stroke::new(
            0.55,
            egui::Color32::from_rgba_unmultiplied(203, 227, 215, 46),
        );
        for x in 0..=region_size[0] {
            let sx = region.left() + x as f32 * px;
            if sx >= inside.left() && sx <= inside.right() {
                p.line_segment(
                    [
                        egui::pos2(sx, inside.top()),
                        egui::pos2(sx, inside.bottom()),
                    ],
                    stroke,
                );
            }
        }
        for y in 0..=region_size[1] {
            let sy = region.top() + y as f32 * px;
            if sy >= inside.top() && sy <= inside.bottom() {
                p.line_segment(
                    [
                        egui::pos2(inside.left(), sy),
                        egui::pos2(inside.right(), sy),
                    ],
                    stroke,
                );
            }
        }
    }
    if !chunks {
        return;
    }
    let unit = side as f32 * px;
    let origin_chunk = [
        origin_world[0].div_euclid(side as i64),
        origin_world[1].div_euclid(side as i64),
    ];
    let chunk_width = region_size[0].div_ceil(side);
    let chunk_height = region_size[1].div_ceil(side);
    let stroke = egui::Stroke::new(
        1.15,
        egui::Color32::from_rgba_unmultiplied(99, 209, 179, 170),
    );
    for x in 0..=chunk_width {
        let sx = region.left() + x as f32 * unit;
        clipped.line_segment(
            [
                egui::pos2(sx, region.top()),
                egui::pos2(sx, region.bottom()),
            ],
            stroke,
        );
    }
    for y in 0..=chunk_height {
        let sy = region.top() + y as f32 * unit;
        clipped.line_segment(
            [
                egui::pos2(region.left(), sy),
                egui::pos2(region.right(), sy),
            ],
            stroke,
        );
    }
    if unit >= 90.0 {
        for y in 0..chunk_height {
            for x in 0..chunk_width {
                let pos = egui::pos2(
                    region.left() + x as f32 * unit + 5.0,
                    region.top() + y as f32 * unit + 5.0,
                );
                if canvas.contains(pos) {
                    clipped.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        format!(
                            "CH {},{}",
                            origin_chunk[0] + x as i64,
                            origin_chunk[1] + y as i64
                        ),
                        egui::FontId::monospace(10.0),
                        egui::Color32::from_rgb(163, 230, 210),
                    );
                }
            }
        }
    }
}
