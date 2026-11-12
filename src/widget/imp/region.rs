use crate::widget::imp::widget::RotationWidgetImpl;
use gtk4::cairo::RectangleInt;
use gtk4::cairo::Region;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::RotationRad;
use tracing::error;

pub trait RotatedRegionCalculator {
    /// Calculate the input region of `rect` rotated by `rotation`.
    fn calculate_rotated_region(&self, rect: Rect<Pixels>, rotation: RotationRad) -> Region;
}

impl RotatedRegionCalculator for RotationWidgetImpl {
    fn calculate_rotated_region(&self, rect: Rect<Pixels>, rotation: RotationRad) -> Region {
        let center = rect.center();
        let region = Region::create();
        // Scanline strips are relative to the rect's center; offset them
        // by the rect's center to get widget-space coordinates.
        for strip in rect.rotated_region_scanlines(rotation) {
            let rect = RectangleInt::new(
                (center.x + f64::from(strip.x_start)).round() as i32,
                (center.y + f64::from(strip.y)).round() as i32,
                strip.width(),
                1,
            );
            if let Err(e) = region.union_rectangle(&rect) {
                error!("Failed to union rectangle region: {}", e);
            }
        }
        region
    }
}
