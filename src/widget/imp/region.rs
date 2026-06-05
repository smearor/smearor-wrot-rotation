use crate::widget::imp::widget::RotationWidgetImpl;
use gtk4::cairo::RectangleInt;
use gtk4::cairo::Region;
use std::cmp::Ordering;
use std::f32::consts::PI;
use tracing::error;

pub trait RotatedRegionCalculator {
    /// Calculate the rotated region with scale
    fn calculate_rotated_region_with_scale(&self, win_w: f32, win_h: f32, c_w: f32, c_h: f32, deg: f32) -> Region;
}

impl RotatedRegionCalculator for RotationWidgetImpl {
    fn calculate_rotated_region_with_scale(&self, win_w: f32, win_h: f32, c_w: f32, c_h: f32, deg: f32) -> Region {
        let rad = deg * PI / 180.0;
        let (sin, cos) = rad.sin_cos();
        let _scale_x = win_w / c_w;
        let _scale_y = win_h / c_h;
        let pts = [
            (-win_w / 2.0, -win_h / 2.0),
            (win_w / 2.0, -win_h / 2.0),
            (win_w / 2.0, win_h / 2.0),
            (-win_w / 2.0, win_h / 2.0),
        ];
        let mut transformed = Vec::new();
        for (x, y) in pts {
            let rx = x * cos - y * sin + win_w / 2.0;
            let ry = x * sin + y * cos + win_h / 2.0;
            transformed.push((rx, ry));
        }
        let min_y = transformed.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min).floor() as i32;
        let max_y = transformed.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max).ceil() as i32;
        let region = gtk4::cairo::Region::create();
        let _step = 1;
        for y in min_y..max_y {
            let current_y = y as f32 + 0.5; // Middle of the line
            let mut intersections = Vec::new();
            for i in 0..4 {
                let p1 = transformed[i];
                let p2 = transformed[(i + 1) % 4];
                if (p1.1 <= current_y && p2.1 > current_y) || (p2.1 <= current_y && p1.1 > current_y) {
                    let x = p1.0 + (current_y - p1.1) * (p2.0 - p1.0) / (p2.1 - p1.1);
                    intersections.push(x);
                }
            }

            if intersections.len() >= 2 {
                intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
                let start_x = intersections[0];
                let end_x = intersections[intersections.len() - 1];

                let rect = RectangleInt::new(start_x.round() as i32, y, (end_x - start_x).round() as i32, 1);
                if let Err(e) = region.union_rectangle(&rect) {
                    error!("Failed to union rectangle region: {}", e);
                }
            }
        }

        region
    }
}
