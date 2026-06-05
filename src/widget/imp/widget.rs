use crate::animation::functions::easing::EasingFunction;
use crate::animation::rotation::RotationAnimation;
use crate::animation::zoom::RotationZoomAnimation;
use crate::widget::imp::region::RotatedRegionCalculator;
use crate::widget::layout::RotatedLayout;
use crate::widget::widget::RotationWidget;
use glib::ControlFlow;
use glib::object::Cast;
use glib::subclass::prelude::ObjectImpl;
use glib::subclass::prelude::ObjectImplExt;
use glib::subclass::prelude::ObjectSubclass;
use glib::subclass::prelude::ObjectSubclassExt;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk4::Orientation;
use gtk4::PropagationPhase;
use gtk4::prelude::EventControllerExt;
use gtk4::prelude::GestureExt;
use gtk4::prelude::NativeExt;
use gtk4::prelude::SurfaceExt;
use gtk4::prelude::WidgetExt;
use gtk4::prelude::WidgetExtManual;
use gtk4::subclass::prelude::WidgetClassExt;
use gtk4::subclass::prelude::WidgetImpl;
use gtk4::subclass::prelude::WidgetImplExt;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::debug;
use tracing::error;

#[derive(Default)]
pub struct RotationWidgetImpl {
    pub child: RefCell<Option<gtk4::Widget>>,
    pub animation: RefCell<Option<RotationAnimation>>,
    pub animation_speed: Cell<u64>,
    pub rotation_zoom_animation: RefCell<Option<RotationZoomAnimation>>,
    pub animations_enabled: Cell<bool>,
    pub animation_overshoot: Cell<f64>,
}

#[glib::object_subclass]
impl ObjectSubclass for RotationWidgetImpl {
    const NAME: &'static str = "RotatedBox";
    type Type = RotationWidget;
    type ParentType = gtk4::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<RotatedLayout>();
    }
}

impl ObjectImpl for RotationWidgetImpl {
    fn constructed(&self) {
        self.parent_constructed();

        // Add gesture recognizers for rotation and pinch
        let widget = self.obj();

        // Rotate gesture
        let rotate_gesture = gtk4::GestureRotate::new();
        rotate_gesture.set_propagation_phase(PropagationPhase::Bubble);
        widget.add_controller(rotate_gesture.clone().upcast::<gtk4::EventController>());

        let widget_clone = widget.clone();
        let initial_rotation = Rc::new(RefCell::new(0.0f32));
        let total_angle_delta = Rc::new(RefCell::new(0.0f32));
        let last_angle = Rc::new(RefCell::new(0.0f32));
        let min_angle_threshold = 45.0f32; // Minimum angle threshold in degrees
        let feedback_rotation = Rc::new(RefCell::new(0.0f32));
        let direction_determined = Rc::new(RefCell::new(false));
        let rotation_applied = Rc::new(RefCell::new(false));
        let initial_rotation_clone = initial_rotation.clone();
        let total_angle_delta_clone = total_angle_delta.clone();
        let last_angle_clone = last_angle.clone();
        let feedback_rotation_clone = feedback_rotation.clone();
        let direction_determined_clone = direction_determined.clone();
        let rotation_applied_clone = rotation_applied.clone();

        rotate_gesture.connect_begin(move |_gesture, _sequence| {
            let layout = match widget_clone.layout_manager() {
                Some(lm) => lm,
                None => {
                    error!("Failed to get layout manager in rotate gesture begin");
                    return;
                }
            };

            let layout = match layout.downcast::<RotatedLayout>() {
                Ok(rl) => rl,
                Err(_) => {
                    error!("Layout manager is not RotatedLayout in rotate gesture begin");
                    return;
                }
            };

            *initial_rotation_clone.borrow_mut() = layout.imp().rotation.get();
            *total_angle_delta_clone.borrow_mut() = 0.0;
            *last_angle_clone.borrow_mut() = 0.0;
            *feedback_rotation_clone.borrow_mut() = 0.0;
            *direction_determined_clone.borrow_mut() = false;
            *rotation_applied_clone.borrow_mut() = false;
        });

        let widget_clone = widget.clone();
        let initial_rotation_clone = initial_rotation.clone();
        let total_angle_delta_clone = total_angle_delta.clone();
        let last_angle_clone = last_angle.clone();
        let feedback_rotation_clone = feedback_rotation.clone();
        let direction_determined_clone = direction_determined.clone();
        let rotation_applied_clone = rotation_applied.clone();
        // Prevents gesture conflicts with pointer events
        rotate_gesture.set_propagation_phase(PropagationPhase::Capture);
        rotate_gesture.connect_angle_changed(move |_gesture, _angle, angle_delta| {
            debug!("angle_delta {angle_delta}");
            // Stop processing if rotation was already applied
            if *rotation_applied_clone.borrow() {
                return;
            }

            // 1. Convert to degrees (0.0 to 360.0)
            let current_angle = angle_delta.to_degrees() as f32;

            // 2. Calculate the real delta
            // We need the difference between the current angle and the last angle
            let mut last_angle_ref = last_angle_clone.borrow_mut();

            // Smooth the jump at 0/360 degrees:
            let mut delta = current_angle - *last_angle_ref;

            // If the jump is greater than 180 degrees, we have crossed the 0-degree line
            if delta > 180.0 {
                delta -= 360.0; // Left rotation across the zero boundary
            } else if delta < -180.0 {
                delta += 360.0; // Right rotation across the zero boundary
            }

            *last_angle_ref = current_angle;

            // 3. Now 'delta' is reliably positive (CW) or negative (CCW)
            *total_angle_delta_clone.borrow_mut() += delta;

            let current_total = *total_angle_delta_clone.borrow();

            // let mut current_angle = angle_delta.to_degrees();
            //
            // let delta_degrees = angle_delta.to_degrees();
            // *total_angle_delta_clone.borrow_mut() += delta_degrees as f32;
            //
            // let current_total = *total_angle_delta_clone.borrow();

            debug!("current_total {} min_angle_threshold {}", current_total, min_angle_threshold);

            // Determine direction and apply feedback rotation on first significant movement
            if !*direction_determined_clone.borrow() && current_total.abs() > 10.0 {
                *direction_determined_clone.borrow_mut() = true;
                let feedback_amount = if current_total > 0.0 { 2.0 } else { -2.0 };
                *feedback_rotation_clone.borrow_mut() = feedback_amount as f32;

                debug!("Direction determined: {}, feedback_amount: {}", current_total, feedback_amount);

                let layout = match widget_clone.layout_manager() {
                    Some(lm) => lm,
                    None => {
                        error!("Failed to get layout manager in rotate gesture feedback");
                        return;
                    }
                };

                let layout = match layout.downcast::<RotatedLayout>() {
                    Ok(rl) => rl,
                    Err(_) => {
                        error!("Layout manager is not RotatedLayout in rotate gesture feedback");
                        return;
                    }
                };

                let current_rotation = layout.imp().rotation.get();
                let new_rotation = current_rotation + feedback_amount as f32;
                debug!("Applying feedback: current_rotation {} -> new_rotation {}", current_rotation, new_rotation);
                layout.imp().rotation.set(new_rotation);
                widget_clone.queue_allocate();
                debug!("Applied feedback rotation: {} degrees", feedback_amount);
            }

            if current_total.abs() < min_angle_threshold {
                // Don't apply rotation until minimum threshold is reached
                return;
            }

            let layout = match widget_clone.layout_manager() {
                Some(lm) => lm,
                None => {
                    error!("Failed to get layout manager in rotate gesture");
                    return;
                }
            };

            let layout = match layout.downcast::<RotatedLayout>() {
                Ok(rl) => rl,
                Err(_) => {
                    error!("Layout manager is not RotatedLayout in rotate gesture");
                    return;
                }
            };

            let current_rotation = layout.imp().rotation.get();
            let initial_rot = *initial_rotation_clone.borrow();

            // Find next standard rotation based on direction
            let normalized_rot = initial_rot.rem_euclid(360.0);
            debug!("current_rotation: {current_rotation} initial_rot: {initial_rot} normalized_rot: {normalized_rot} current_total: {current_total}");
            let new_rotation = if current_total > 0.0 {
                // Clockwise direction
                if normalized_rot < 45.0 {
                    90.0f32
                } else if normalized_rot < 135.0 {
                    180.0
                } else if normalized_rot < 225.0 {
                    270.0
                } else {
                    0.0
                }
            } else {
                // Counter-clockwise direction
                if normalized_rot < 45.0 {
                    270.0
                } else if normalized_rot < 135.0 {
                    0.0
                } else if normalized_rot < 225.0 {
                    90.0
                } else {
                    180.0
                }
            };
            let final_rotation = new_rotation.rem_euclid(360.0);

            debug!(
                "Animation: initial_rot={} final_rotation={} current_rotation={} normalized_rot={}",
                initial_rot, final_rotation, current_rotation, normalized_rot
            );

            // Check if animations are enabled
            let animations_enabled = widget_clone.imp().animations_enabled.get();

            if !animations_enabled {
                // If animations are disabled, set rotation immediately
                let layout = match widget_clone.layout_manager() {
                    Some(lm) => lm,
                    None => return,
                };

                let layout = match layout.downcast::<RotatedLayout>() {
                    Ok(rl) => rl,
                    Err(_) => return,
                };

                layout.imp().rotation.set(final_rotation);
                widget_clone.queue_allocate();
                *rotation_applied_clone.borrow_mut() = true;
                return;
            }

            // Start overshoot animation using RotationZoomAnimation
            let animation_speed_ms = widget_clone.imp().animation_speed.get();
            let overshoot_amount = widget_clone.imp().animation_overshoot.get();
            let mut animation = RotationZoomAnimation::new(
                current_rotation as f64,
                final_rotation as f64,
                1.0,
                0.8,
                1.0,
                Duration::from_millis(animation_speed_ms),
                EasingFunction::Overshoot { overshoot_amount },
            );
            animation.start();
            *widget_clone.imp().rotation_zoom_animation.borrow_mut() = Some(animation);

            // Start animation tick callback
            widget_clone.add_tick_callback(move |widget, _frame_clock| {
                let imp = widget.imp();
                if let Some(ref mut anim) = *imp.rotation_zoom_animation.borrow_mut() {
                    let Some((current_rotation, current_scale)) = anim.get_current_values_with_phases() else {
                        return ControlFlow::Break;
                    };
                    let Some(layout) = widget.layout_manager() else {
                        return ControlFlow::Continue;
                    };
                    let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                        return ControlFlow::Continue;
                    };

                    // Apply rotation
                    layout.imp().rotation.set(current_rotation as f32);

                    // Apply scale using transform matrix instead of CSS
                    layout.imp().scale.set(current_scale as f32);

                    widget.queue_allocate();

                    if anim.is_complete() {
                        // Reset scale to 1.0 after animation completes
                        layout.imp().scale.set(1.0);
                        ControlFlow::Break
                    } else {
                        ControlFlow::Continue
                    }
                } else {
                    ControlFlow::Break
                }
            });

            // Mark rotation as applied to prevent further rotations
            *rotation_applied_clone.borrow_mut() = true;
        });

        let widget_clone = widget.clone();
        let initial_rotation_clone = initial_rotation.clone();
        let total_angle_delta_clone = total_angle_delta.clone();
        rotate_gesture.connect_end(move |_gesture, _sequence| {
            let current_total = *total_angle_delta_clone.borrow();

            if current_total.abs() < min_angle_threshold {
                // Reset to initial rotation if minimum threshold was not reached
                let Some(layout) = widget_clone.layout_manager() else {
                    error!("Failed to get layout manager in rotate gesture end");
                    return;
                };

                let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                    error!("Layout manager is not RotatedLayout in rotate gesture end");
                    return;
                };

                let initial_rot = *initial_rotation_clone.borrow();
                layout.imp().rotation.set(initial_rot);
                widget_clone.queue_allocate();
                debug!("Reset to initial rotation {initial_rot} because threshold was not reached");
            }
        });
    }

    fn dispose(&self) {
        if let Some(child) = self.child.borrow_mut().take() {
            child.unparent();
        }
    }
}

impl WidgetImpl for RotationWidgetImpl {
    fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
        self.parent_size_allocate(width, height, baseline);
        let widget = self.obj();
        if let Some(surface) = widget.native().and_then(|n| n.surface()) {
            if let Some(child) = widget.first_child() {
                let angle = widget
                    .layout_manager()
                    .and_then(|lm| lm.downcast::<RotatedLayout>().ok())
                    .map(|rl| rl.imp().rotation.get())
                    .unwrap_or(0.0);
                let (_, child_w, _, _) = child.measure(Orientation::Horizontal, -1);
                let (_, child_h, _, _) = child.measure(Orientation::Vertical, -1);
                let region = self.calculate_rotated_region_with_scale(width as f32, height as f32, child_w as f32, child_h as f32, angle);
                surface.set_input_region(Some(&region));
            }
        }
    }

    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
        let widget = self.obj();
        if let Some(child) = widget.first_child() {
            widget.snapshot_child(&child, snapshot);
        }
    }
}
