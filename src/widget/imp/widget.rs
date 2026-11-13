use crate::AnimationKind;
use crate::widget::layout::RotatedLayout;
use crate::widget::widget::RotationWidget;
use glib::ControlFlow;
use glib::object::Cast;
use glib::subclass::prelude::ObjectImpl;
use glib::subclass::prelude::ObjectImplExt;
use glib::subclass::prelude::ObjectSubclass;
use glib::subclass::prelude::ObjectSubclassExt;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk4::PropagationPhase;
use gtk4::cairo::Region;
use gtk4::prelude::EventControllerExt;
use gtk4::prelude::GestureExt;
use gtk4::prelude::NativeExt;
use gtk4::prelude::SurfaceExt;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::WidgetClassExt;
use gtk4::subclass::prelude::WidgetImpl;
use gtk4::subclass::prelude::WidgetImplExt;
use smearor_commons_animation::EasingFunction;
use smearor_commons_animation::RotationZoomAnimation;
use smearor_commons_animation::Span;
use smearor_commons_animation::ThreePhaseSpan;
use smearor_commons_animation::drive;
use smearor_commons_geometry::Fraction;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Position;
use smearor_commons_geometry::Rect;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::ScaleFactor;
use smearor_commons_geometry::Vec2;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use tracing::debug;
use tracing::error;

#[derive(Default)]
pub struct RotationWidgetImpl {
    pub child: RefCell<Option<gtk4::Widget>>,
    pub animation_kind: Cell<AnimationKind>,
    pub animation_zoom: Cell<ScaleFactor>,
    pub animation_speed: Cell<u64>,
    pub animations_enabled: Cell<bool>,
    pub animation_overshoot: Cell<f64>,
    /// Whether rotation via gesture is active. Defaults to `true`.
    pub gesture_rotation_enabled: Cell<bool>,
    /// Whether `size_allocate` clamps the native surface's input region to the
    /// rotated rect. Defaults to `true` (layer-shell consumers); nested widgets
    /// must opt out - the clamp is surface-global.
    pub input_region_enabled: Cell<bool>,
    /// Reference to the rotate gesture controller for propagation phase toggling.
    pub rotate_gesture: RefCell<Option<gtk4::GestureRotate>>,
}

#[glib::object_subclass]
impl ObjectSubclass for RotationWidgetImpl {
    const NAME: &'static str = "RotationWidget";
    type Type = RotationWidget;
    type ParentType = gtk4::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<RotatedLayout>();
    }
}

impl ObjectImpl for RotationWidgetImpl {
    fn constructed(&self) {
        self.parent_constructed();

        // Gesture rotation is enabled by default
        self.gesture_rotation_enabled.set(true);
        // Input-region clamping is enabled by default - the layer-shell
        // fullscreen case; nested widgets opt out via
        // `set_input_region_enabled(false)`.
        self.input_region_enabled.set(true);

        // Penner's canonical overshoot strength - a `0.0` default
        // would silently degenerate the overshoot easing to a plain
        // quadratic ease-in-out.
        self.animation_overshoot.set(EasingFunction::DEFAULT_OVERSHOOT);

        // The scale at the dip of the three-phase zoom.
        self.animation_zoom.set(ScaleFactor::new(0.8));

        // Add gesture recognizers for rotation and pinch
        let widget = self.obj();

        // Rotate gesture
        let rotate_gesture = gtk4::GestureRotate::new();
        rotate_gesture.set_propagation_phase(PropagationPhase::Capture);
        widget.add_controller(rotate_gesture.clone().upcast::<gtk4::EventController>());
        *self.rotate_gesture.borrow_mut() = Some(rotate_gesture.clone());

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

            *initial_rotation_clone.borrow_mut() = layout.imp().rotation.get().to_degrees_f32();
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
        rotate_gesture.connect_angle_changed(move |_gesture, _angle, angle_delta| {
            debug!("angle_delta {angle_delta}");
            // Stop processing if rotation was already applied
            if *rotation_applied_clone.borrow() {
                return;
            }

            // 1. Convert to degrees (0.0 to 360.0)
            let current_angle = RotationRad::new(angle_delta).to_degrees_f32();

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

                let current_rotation = layout.imp().rotation.get().to_degrees_f32();
                let new_rotation = current_rotation + feedback_amount as f32;
                debug!("Applying feedback: current_rotation {} -> new_rotation {}", current_rotation, new_rotation);
                layout.imp().rotation.set(RotationRad::from_degrees(new_rotation));
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

            let current_rotation = layout.imp().rotation.get().to_degrees_f32();
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

                layout.imp().rotation.set(RotationRad::from_degrees(final_rotation));
                widget_clone.queue_allocate();
                *rotation_applied_clone.borrow_mut() = true;
                return;
            }

            // Start overshoot animation: rotate on the shortest arc
            // while zooming out, holding and zooming back in.
            let animation_speed_ms = widget_clone.imp().animation_speed.get();
            let overshoot_amount = widget_clone.imp().animation_overshoot.get();
            let mut animation = RotationZoomAnimation::new(
                (
                    Span::new(RotationRad::from_degrees(current_rotation), RotationRad::from_degrees(final_rotation)),
                    ThreePhaseSpan::new(
                        ScaleFactor::ONE,
                        widget_clone.imp().animation_zoom.get(),
                        ScaleFactor::ONE,
                        Fraction::new(1.0 / 3.0),
                        Fraction::new(2.0 / 3.0),
                    ),
                ),
                Duration::from_millis(animation_speed_ms),
                EasingFunction::Overshoot { overshoot_amount },
            );
            animation.start();

            drive(&widget_clone, animation, |widget, (rotation, scale)| {
                let Some(layout) = widget.layout_manager() else {
                    return ControlFlow::Continue;
                };
                let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                    return ControlFlow::Continue;
                };
                layout.imp().rotation.set(rotation);
                layout.imp().scale.set(scale);
                widget.queue_allocate();
                ControlFlow::Continue
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
                layout.imp().rotation.set(RotationRad::from_degrees(initial_rot));
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
        if !self.input_region_enabled.get() {
            return;
        }
        let widget = self.obj();
        if let Some(native) = widget.native()
            && let Some(surface) = native.surface()
        {
            let angle = widget
                .layout_manager()
                .and_then(|lm| lm.downcast::<RotatedLayout>().ok())
                .map(|rl| rl.imp().rotation.get())
                .unwrap_or(RotationRad::ZERO);
            let region = Rect::<Pixels>::new(Vec2::ZERO, Vec2::new(width, height).to_f64())
                .rotated_region(angle)
                .unwrap_or_else(|e| {
                    error!("Failed to build rotated input region: {}", e);
                    Region::create()
                });
            // `set_input_region` expects surface coordinates; the computed
            // region is widget-space (0..w x 0..h) - translate by the
            // widget's origin inside the native. Only exact when ancestor
            // transforms are pure translations.
            if let Some(origin) = widget.compute_point(&native, &gtk4::graphene::Point::new(0.0, 0.0)) {
                let offset = Position::<f32>::from(origin).to_i32_round();
                region.translate(offset.x, offset.y);
            }
            surface.set_input_region(Some(&region));
        }
    }

    fn unmap(&self) {
        self.parent_unmap();
        // The surface's input region outlives this widget - reset it so a
        // removed/hidden widget never leaves the surface clamped.
        if self.input_region_enabled.get()
            && let Some(surface) = self.obj().native().and_then(|n| n.surface())
        {
            surface.set_input_region(None);
        }
    }

    fn snapshot(&self, snapshot: &gtk4::Snapshot) {
        let widget = self.obj();
        if let Some(child) = widget.first_child() {
            widget.snapshot_child(&child, snapshot);
        }
    }
}
