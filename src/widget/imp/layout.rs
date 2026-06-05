use crate::widget::layout::RotatedLayout;
use glib::subclass::object::ObjectImpl;
use glib::subclass::prelude::ObjectSubclass;
use gtk4::LayoutManager;
use gtk4::Orientation;
use gtk4::graphene::Point;
use gtk4::gsk::Transform;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::layout_manager::LayoutManagerImpl;
use std::cell::Cell;
use tracing::debug;
use tracing::trace;

pub struct RotatedLayoutImpl {
    pub rotation: Cell<f32>,
    pub scale: Cell<f32>,
}

impl Default for RotatedLayoutImpl {
    fn default() -> Self {
        Self {
            rotation: Cell::new(0.0),
            scale: Cell::new(1.0),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RotatedLayoutImpl {
    const NAME: &'static str = "RotatedLayout";
    type Type = RotatedLayout;
    type ParentType = LayoutManager;
}

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
                let denom = abs_cos * abs_cos - abs_sin * abs_sin;
                let (target_child_w, target_child_h) = if denom.abs() < 0.9 {
                    // Sonderfall 45°, 135° etc. (Matrix singulär)
                    // Hier vereinfacht: Kind quadratisch einpassen
                    let s = (width as f32 / (abs_cos + abs_sin)) as i32;
                    (s, s)
                } else {
                    let w_c = (width as f32 * abs_cos - height as f32 * abs_sin) / denom;
                    let h_c = (height as f32 * abs_cos - width as f32 * abs_sin) / denom;
                    (w_c as i32, h_c as i32)
                };

                // let (_, child_nat_w, _, _) = c.measure(Orientation::Horizontal, -1);
                // let (_, child_nat_h, _, _) = c.measure(Orientation::Vertical, -1);
                // trace!("child_nat_w {child_nat_w} child_nat_h {child_nat_h}");
                let transform = Transform::new()
                    .translate(&Point::new(width as f32 / 2.0, height as f32 / 2.0))
                    .rotate(angle_deg)
                    .scale(current_scale, current_scale)
                    // .translate(&Point::new(child_nat_w as f32 / -2.0, child_nat_h as f32 / -2.0));
                    .translate(&Point::new(target_child_w as f32 / -2.0, target_child_h as f32 / -2.0));

                trace!("width {target_child_w} height {target_child_h} baseline {baseline} transform {transform:?}");
                // c.allocate(width, height, baseline, Some(transform));
                c.allocate(target_child_w, target_child_h, baseline, Some(transform));
            }
            child = c.next_sibling();
        }
    }

    fn measure(&self, widget: &gtk4::Widget, orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let angle_deg = self.rotation.get();
        let angle_rad = angle_deg.to_radians();
        let abs_cos = angle_rad.cos().abs();
        let abs_sin = angle_rad.sin().abs();

        if let Some(child) = widget.first_child() {
            let (child_min_w, child_nat_w, _, _) = child.measure(Orientation::Horizontal, -1);
            let (child_min_h, child_nat_h, _, _) = child.measure(Orientation::Vertical, -1);
            trace!("child_min_w {child_min_w} child_nat_w {child_nat_w} child_min_h {child_min_h} child_nat_h {child_nat_h}");

            if orientation == Orientation::Horizontal {
                // Calculate bounding box width
                let min = (child_min_w as f32 * abs_cos + child_min_h as f32 * abs_sin).ceil() as i32;
                let nat = (child_nat_w as f32 * abs_cos + child_nat_h as f32 * abs_sin).ceil() as i32;
                (min, nat, -1, -1)
            } else {
                // Calculate bounding box height
                let min = (child_min_w as f32 * abs_sin + child_min_h as f32 * abs_cos).ceil() as i32;
                let nat = (child_nat_w as f32 * abs_sin + child_nat_h as f32 * abs_cos).ceil() as i32;
                (min, nat, -1, -1)
            }
        } else {
            debug!("No child");
            (0, 0, -1, -1)
        }
    }
}
