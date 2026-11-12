use crate::RotationControlHandler;
use crate::animation::functions::easing::EasingFunction;
use crate::animation::zoom::RotationZoomAnimation;
use crate::rotation::SmearorRotation;
use crate::widget::imp::widget::RotationWidgetImpl;
use crate::widget::layout::RotatedLayout;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::PropagationPhase;
use gtk4::Widget;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::Vec2;
use std::time::Duration;
use tracing::error;

glib::wrapper! {
    pub struct RotationWidget(ObjectSubclass<RotationWidgetImpl>)
        @extends Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl RotationWidget {
    pub fn new(rotation: SmearorRotation) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.set_rotation(rotation);
        obj
    }

    /// Builder-style method to configure gesture rotation. Defaults to `true`.
    pub fn with_gesture_rotation_enabled(self, enabled: bool) -> Self {
        self.set_gesture_rotation_enabled(enabled);
        self
    }

    /// Builder-style method to configure animations. Defaults to `true`.
    pub fn with_animations_enabled(self, enabled: bool) -> Self {
        self.set_animations_enabled(enabled);
        self
    }

    /// Transform input coordinates based on current rotation
    pub fn input_transform(&self, point: Vec2<Pixels>) -> Vec2<Pixels> {
        let rotation = self.rotation();
        if rotation.is_zero() {
            point
        } else {
            // Rotate around the center in reverse direction to
            // compensate for the widget rotation.
            let center = Vec2::<Pixels>::new(self.width() as f64 / 2.0, self.height() as f64 / 2.0);
            (point - center).rotated(-rotation) + center
        }
    }

    pub fn set_animation_speed(&self, speed_ms: u64) {
        self.imp().animation_speed.set(speed_ms);
    }

    pub fn set_animations_enabled(&self, enabled: bool) {
        self.imp().animations_enabled.set(enabled);
    }

    /// Builder-style method to configure the surface input-region clamp.
    /// Defaults to `true`.
    pub fn with_input_region_enabled(self, enabled: bool) -> Self {
        self.set_input_region_enabled(enabled);
        self
    }

    /// Enables or disables clamping the native surface's input region to the
    /// rotated rect in `size_allocate`. Defaults to `true`.
    ///
    /// The clamp is surface-global: everything outside the region becomes
    /// click-through. Keep it enabled for surface-filling widgets (the
    /// layer-shell case); disable it when this widget is nested inside a
    /// layout, or the rest of the window loses input.
    pub fn set_input_region_enabled(&self, enabled: bool) {
        self.imp().input_region_enabled.set(enabled);
        if !enabled && let Some(surface) = self.native().and_then(|n| n.surface()) {
            surface.set_input_region(None);
        }
        self.queue_allocate();
    }

    /// Whether the surface input-region clamp is active.
    pub fn input_region_enabled(&self) -> bool {
        self.imp().input_region_enabled.get()
    }

    /// Builder-style method to configure natural child size allocation.
    /// Defaults to `false` (legacy inverse-solve allocation).
    pub fn with_natural_child_size_enabled(self, enabled: bool) -> Self {
        self.set_natural_child_size_enabled(enabled);
        self
    }

    /// Enables or disables natural child size allocation. Defaults to
    /// `false`.
    ///
    /// With the legacy allocation (`false`) the layout solves which child
    /// size, rotated, reproduces the allocation - ill-conditioned near 45°,
    /// where the child shrinks into a square. Enabled, the child keeps its
    /// measured natural size; `measure` already reports the rotated bounding
    /// box, so the allocation always contains the rotated child.
    pub fn set_natural_child_size_enabled(&self, enabled: bool) {
        let Some(layout) = self.layout_manager() else {
            error!("Failed to get layout manager in set_natural_child_size_enabled");
            return;
        };
        let Ok(rotated_layout) = layout.downcast::<RotatedLayout>() else {
            error!("Layout manager is not RotatedLayout in set_natural_child_size_enabled");
            return;
        };
        rotated_layout.imp().natural_child_size.set(enabled);
        self.queue_allocate();
    }

    /// Whether the child is allocated its measured natural size.
    pub fn natural_child_size_enabled(&self) -> bool {
        let Some(layout) = self.layout_manager() else {
            return false;
        };
        layout
            .downcast::<RotatedLayout>()
            .map(|rotated_layout| rotated_layout.imp().natural_child_size.get())
            .unwrap_or(false)
    }

    /// Enables or disables rotation via gesture. Defaults to `true`.
    pub fn set_gesture_rotation_enabled(&self, enabled: bool) {
        self.imp().gesture_rotation_enabled.set(enabled);
        let phase = if enabled { PropagationPhase::Capture } else { PropagationPhase::None };
        if let Some(ref gesture) = *self.imp().rotate_gesture.borrow() {
            gesture.set_propagation_phase(phase);
        }
    }

    pub fn set_animation_overshoot(&self, overshoot: f64) {
        self.imp().animation_overshoot.set(overshoot);
    }

    pub fn set_rotation_with_animation(&self, new_rotation: RotationRad) {
        let imp = self.imp();

        // Check if animations are disabled
        if !imp.animations_enabled.get() {
            // If animations are disabled, set rotation immediately
            let layout = match self.layout_manager() {
                Some(lm) => lm,
                None => {
                    error!("Failed to get layout manager in set_rotation_with_animation");
                    return;
                }
            };

            let layout = match layout.downcast::<RotatedLayout>() {
                Ok(rl) => rl,
                Err(_) => {
                    error!("Layout manager is not RotatedLayout in set_rotation_with_animation");
                    return;
                }
            };

            layout.imp().rotation.set(new_rotation.to_degrees() as f32);
            self.queue_allocate();
            return;
        }

        let layout = match self.layout_manager() {
            Some(lm) => lm,
            None => {
                error!("Failed to get layout manager in set_rotation_with_animation");
                return;
            }
        };

        let layout = match layout.downcast::<RotatedLayout>() {
            Ok(rl) => rl,
            Err(_) => {
                error!("Layout manager is not RotatedLayout in set_rotation_with_animation");
                return;
            }
        };

        let current_rotation = layout.imp().rotation.get();
        let animation_speed_ms = imp.animation_speed.get();

        // Create rotation zoom animation with three phases
        // 0-33%: Rotation and zoom out (scale 1.0 -> 0.9)
        // 33-66%: Rotation without zoom (scale 0.9 -> 0.9)
        // 66-100%: Rotation and zoom in (scale 0.9 -> 1.0)
        let mut animation = RotationZoomAnimation::new(
            current_rotation as f64,
            new_rotation.to_degrees(),
            1.0,
            0.8,
            1.0,
            Duration::from_millis(animation_speed_ms),
            EasingFunction::EaseInOut,
        );
        animation.start();
        *imp.rotation_zoom_animation.borrow_mut() = Some(animation);

        self.add_tick_callback(move |widget, _frame_clock| {
            let imp = widget.imp();
            if let Some(ref mut anim) = *imp.rotation_zoom_animation.borrow_mut() {
                let (current_rotation, current_scale): (f64, f64) = match anim.get_current_values_with_phases() {
                    Some(values) => values,
                    None => return glib::ControlFlow::Break,
                };

                let layout = match widget.layout_manager() {
                    Some(lm) => lm,
                    None => return glib::ControlFlow::Continue,
                };

                let layout = match layout.downcast::<RotatedLayout>() {
                    Ok(rl) => rl,
                    Err(_) => return glib::ControlFlow::Continue,
                };

                // Apply rotation
                layout.imp().rotation.set(current_rotation as f32);

                // Apply scale using transform matrix instead of CSS
                layout.imp().scale.set(current_scale as f32);

                widget.queue_allocate();

                if anim.is_complete() {
                    // Reset scale to 1.0 after animation completes
                    layout.imp().scale.set(1.0);
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    pub fn set_child(&self, child: Option<&impl IsA<gtk4::Widget>>) {
        let mut self_child = self.imp().child.borrow_mut();
        if let Some(old_child) = self_child.take() {
            old_child.unparent();
        }
        if let Some(new_child) = child {
            let widget_clone_width = self.clone();
            let widget_clone_height = self.clone();
            new_child.connect_notify_local(Some("width-request"), move |_child, _param| {
                widget_clone_width.queue_allocate();
            });
            new_child.connect_notify_local(Some("height-request"), move |_child, _param| {
                widget_clone_height.queue_allocate();
            });
            new_child.set_parent(self);
            *self_child = Some(new_child.clone().upcast());
        }
    }
}
