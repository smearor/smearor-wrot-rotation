use crate::widget::layout::RotatedLayout;
use glib::subclass::object::ObjectImpl;
use glib::subclass::prelude::ObjectSubclass;
use gtk4::LayoutManager;
use gtk4::Orientation;
use gtk4::glib::Properties;
use gtk4::graphene::Point;
use gtk4::gsk::Transform;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::layout_manager::LayoutManagerImpl;
use gtk4::subclass::prelude::DerivedObjectProperties;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::RotationRad;
use std::cell::Cell;
use tracing::debug;
use tracing::trace;

#[derive(Properties)]
#[properties(wrapper_type = RotatedLayout)]
pub struct RotatedLayoutImpl {
    #[properties(get, set, notify)]
    pub rotation: Cell<f32>,
    #[properties(get, set, notify)]
    pub scale: Cell<f32>,
    /// When `true`, the child is allocated its measured natural size instead
    /// of solving which rotated child size reproduces the allocation. The
    /// inverse solve is ill-conditioned near 45 degrees (and symmetric bands),
    /// where it shrinks non-square children into a square. Defaults to
    /// `false` to preserve the legacy allocation behavior.
    #[properties(get, set, notify)]
    pub natural_child_size: Cell<bool>,
}

impl Default for RotatedLayoutImpl {
    fn default() -> Self {
        Self {
            rotation: Cell::new(0.0),
            scale: Cell::new(1.0),
            natural_child_size: Cell::new(false),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RotatedLayoutImpl {
    const NAME: &'static str = "RotatedLayout";
    type Type = RotatedLayout;
    type ParentType = LayoutManager;
}

#[glib::derived_properties]
impl ObjectImpl for RotatedLayoutImpl {}

impl LayoutManagerImpl for RotatedLayoutImpl {
    fn allocate(&self, widget: &gtk4::Widget, width: i32, height: i32, baseline: i32) {
        let mut child = widget.first_child();
        let angle_deg = self.rotation.get();
        let current_scale = self.scale.get();
        let angle_rad = angle_deg.to_radians();
        let abs_cos = angle_rad.cos().abs();
        let abs_sin = angle_rad.sin().abs();
        while let Some(ref c) = child {
            if c.should_layout() {
                let (target_child_w, target_child_h) = if self.natural_child_size.get() {
                    // The child keeps its measured natural size: `measure`
                    // reports the rotated bounding box, so the allocation
                    // always contains the rotated child.
                    let (_, child_nat_w, _, _) = c.measure(Orientation::Horizontal, -1);
                    let (_, child_nat_h, _, _) = c.measure(Orientation::Vertical, -1);
                    (child_nat_w, child_nat_h)
                } else {
                    // `denom` is `cos(2θ)` and vanishes at 45° + k·90° - the
                    // matrix singularity where no rotated child size can
                    // reproduce a non-square allocation and the inverse
                    // solve diverges. Blend quadratically from the solved
                    // size at the band edge (`|denom| = 0.9`, continuous
                    // with the pure solve outside) down to the square fit
                    // at the singularity; a hard switch jumped the child
                    // size whenever `|denom|` crossed the threshold.
                    let denom = abs_cos * abs_cos - abs_sin * abs_sin;
                    let square_fit = width as f32 / (abs_cos + abs_sin);
                    let blend = (denom.abs() / 0.9).min(1.0);
                    let blend = blend * blend;
                    if blend == 0.0 {
                        let s = square_fit as i32;
                        (s, s)
                    } else {
                        let solved_w = (width as f32 * abs_cos - height as f32 * abs_sin) / denom;
                        let solved_h = (height as f32 * abs_cos - width as f32 * abs_sin) / denom;
                        let blended = |solved: f32| (square_fit + (solved.max(0.0) - square_fit) * blend) as i32;
                        (blended(solved_w), blended(solved_h))
                    }
                };

                let transform = Transform::new()
                    .translate(&Point::new(width as f32 / 2.0, height as f32 / 2.0))
                    .rotate(angle_deg)
                    .scale(current_scale, current_scale)
                    .translate(&Point::new(target_child_w as f32 / -2.0, target_child_h as f32 / -2.0));

                trace!("width {target_child_w} height {target_child_h} baseline {baseline} transform {transform:?}");
                c.allocate(target_child_w, target_child_h, baseline, Some(transform));
            }
            child = c.next_sibling();
        }
    }

    fn measure(&self, widget: &gtk4::Widget, orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let rotation = RotationRad::from_degrees(self.rotation.get() as f64);

        if let Some(child) = widget.first_child() {
            let (child_min_w, child_nat_w, _, _) = child.measure(Orientation::Horizontal, -1);
            let (child_min_h, child_nat_h, _, _) = child.measure(Orientation::Vertical, -1);
            trace!("child_min_w {child_min_w} child_nat_w {child_nat_w} child_min_h {child_min_h} child_nat_h {child_nat_h}");

            // Rotated bounding box of the child's natural and minimum size.
            let min = Rect::<Pixels>::from_xywh(0.0, 0.0, child_min_w as f64, child_min_h as f64).rotated_bounds(rotation);
            let nat = Rect::<Pixels>::from_xywh(0.0, 0.0, child_nat_w as f64, child_nat_h as f64).rotated_bounds(rotation);
            if orientation == Orientation::Horizontal {
                (min.x.ceil() as i32, nat.x.ceil() as i32, -1, -1)
            } else {
                (min.y.ceil() as i32, nat.y.ceil() as i32, -1, -1)
            }
        } else {
            debug!("No child");
            (0, 0, -1, -1)
        }
    }
}
