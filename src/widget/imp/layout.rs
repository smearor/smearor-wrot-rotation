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
use smearor_commons_geometry::AspectRatio;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::ScaleFactor;
use smearor_commons_geometry::Size;
use smearor_commons_geometry::WidgetSizeExt;
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
    /// When `true`, the child is allocated its measured natural size.
    /// When `false`, it is resized to the largest rectangle with its
    /// natural aspect ratio whose rotated bounding box fits inside the
    /// allocation (`unrotated_fit` - a one-sided condition that is never
    /// singular).
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
        let allocation = Size::new(width, height);
        let center = allocation.to_f32() / 2.0;
        let (_, child_nat) = c.measure_size();
        let target_size: Size<i32> = if self.natural_child_size.get() {
            // The child keeps its measured natural size: `measure`
            // reports the rotated bounding box, so the allocation
            // always contains the rotated child.
            child_nat
        } else {
            // The child keeps its natural aspect ratio and is resized
            // to the largest rectangle whose rotated bounding box fits
            // inside the allocation. `unrotated_fit` is a one-sided
            // condition, so it is never singular - the previous exact
            // `unrotated_bounds` solve forced a bounds-fill aspect
            // onto the child, letterboxing aspect-fixed content such
            // as a wide image down to an invisible sliver.
            let nat_aspect = child_nat.to_f64().aspect().unwrap_or(AspectRatio::ONE);
            allocation.to_f64().unrotated_fit(nat_aspect, rotation).map(|v| v as i32)
        };

        let angle_deg = rotation.to_degrees_f32();
        let current_scale = self.scale.get().as_f32();
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
            let (child_min, child_nat) = child.measure_size();
            trace!("child_min {child_min} child_nat {child_nat}");

            // Rotated bounding box of the child's natural and minimum size.
            let min = child_min.to_f64().rotated_bounds(rotation);
            let nat = child_nat.to_f64().rotated_bounds(rotation);
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
