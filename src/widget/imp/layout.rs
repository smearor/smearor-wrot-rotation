use crate::widget::layout::RotatedLayout;
use glib::subclass::object::ObjectImpl;
use glib::subclass::prelude::ObjectSubclass;
use gtk4::LayoutManager;
use gtk4::Orientation;
use gtk4::glib::Properties;
use gtk4::gsk::Transform;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::layout_manager::LayoutManagerImpl;
use gtk4::subclass::prelude::DerivedObjectProperties;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Position;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::ScaleFactor;
use smearor_commons_geometry::Size;
use smearor_commons_geometry::Vec2;
use std::cell::Cell;
use tracing::debug;
use tracing::trace;

#[derive(Properties)]
#[properties(wrapper_type = RotatedLayout)]
pub struct RotatedLayoutImpl {
    #[properties(get, set, notify)]
    pub rotation: Cell<RotationRad>,
    #[properties(get, set, notify)]
    pub scale: Cell<ScaleFactor>,
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
            rotation: Cell::new(RotationRad::ZERO),
            scale: Cell::new(ScaleFactor::ONE),
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
        let Some(c) = widget.first_child().filter(|c| c.should_layout()) else {
            return;
        };
        let rotation = self.rotation.get();
        let angle_deg = rotation.to_degrees_f32();
        let current_scale = self.scale.get().as_f32();
        let (sin, cos) = rotation.sin_cos();
        let abs_cos = cos.abs();
        let abs_sin = sin.abs();
        // `denom` is `cos(2θ)` and vanishes at 45° + k·90° - the
        // matrix singularity where no rotated child size can
        // reproduce a non-square allocation and the inverse
        // solve diverges. Blend quadratically from the solved
        // size at the band edge (`|denom| = 0.9`, continuous
        // with the pure solve outside) down to the square fit
        // at the singularity; a hard switch jumped the child
        // size whenever `|denom|` crossed the threshold.
        let denom = abs_cos * abs_cos - abs_sin * abs_sin;
        let square_fit = width as f64 / (abs_cos + abs_sin);
        let blend = (denom.abs() / 0.9).min(1.0);
        let blend = blend * blend;
        // The inverse-solve child size. At the singularity `denom` is 0
        // and `solved` holds +-inf/NaN - `blend == 0.0` takes the
        // square-fit branch and never reads it.
        let solved = Vec2::new(
            (width as f64 * abs_cos - height as f64 * abs_sin) / denom,
            (height as f64 * abs_cos - width as f64 * abs_sin) / denom,
        );
        let center = Position::<f32>::new(width as f32 / 2.0, height as f32 / 2.0);
        let target_size: Size<i32> = if self.natural_child_size.get() {
            // The child keeps its measured natural size: `measure`
            // reports the rotated bounding box, so the allocation
            // always contains the rotated child.
            let (_, child_nat_w, _, _) = c.measure(Orientation::Horizontal, -1);
            let (_, child_nat_h, _, _) = c.measure(Orientation::Vertical, -1);
            Size::new(child_nat_w, child_nat_h)
        } else if blend == 0.0 {
            let s = square_fit as i32;
            Size::new(s, s)
        } else {
            solved.map(|v| (square_fit + (v.max(0.0) - square_fit) * blend) as i32)
        };

        let child_center = target_size.map(|v| v as f32 / -2.0);
        let transform = Transform::new()
            .translate(&center.into())
            .rotate(angle_deg)
            .scale(current_scale, current_scale)
            .translate(&child_center.into());

        trace!("size {target_size} baseline {baseline} transform {transform:?}");
        c.allocate(target_size.x, target_size.y, baseline, Some(transform));
    }

    fn measure(&self, widget: &gtk4::Widget, orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let rotation = self.rotation.get();

        if let Some(child) = widget.first_child() {
            let (child_min_w, child_nat_w, _, _) = child.measure(Orientation::Horizontal, -1);
            let (child_min_h, child_nat_h, _, _) = child.measure(Orientation::Vertical, -1);
            trace!("child_min_w {child_min_w} child_nat_w {child_nat_w} child_min_h {child_min_h} child_nat_h {child_nat_h}");

            // Rotated bounding box of the child's natural and minimum size.
            let min = Rect::<Pixels>::new(Vec2::ZERO, Vec2::new(child_min_w, child_min_h).to_f64()).rotated_bounds(rotation);
            let nat = Rect::<Pixels>::new(Vec2::ZERO, Vec2::new(child_nat_w, child_nat_h).to_f64()).rotated_bounds(rotation);
            if orientation == Orientation::Horizontal {
                (min.to_i32_ceil().x, nat.to_i32_ceil().x, -1, -1)
            } else {
                (min.to_i32_ceil().y, nat.to_i32_ceil().y, -1, -1)
            }
        } else {
            debug!("No child");
            (0, 0, -1, -1)
        }
    }
}
