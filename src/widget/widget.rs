use crate::RotationControlHandler;
use crate::animation::functions::easing::EasingFunction;
use crate::animation::zoom::RotationZoomAnimation;
use crate::rotation::SmearorRotation;
use crate::widget::imp::widget::RotationWidgetImpl;
use crate::widget::layout::RotatedLayout;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Widget;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
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

    /// Transform input coordinates based on current rotation
    pub fn input_transform(&self, x: f64, y: f64) -> (f64, f64) {
        let rotation = self.rotation();
        if rotation == 0.0 {
            (x, y)
        } else {
            // Get widget dimensions
            let width = self.width() as f64;
            let height = self.height() as f64;

            // Translate to center
            let cx = width / 2.0;
            let cy = height / 2.0;

            let dx = x - cx;
            let dy = y - cy;

            // Rotate in reverse direction to compensate for widget rotation
            let radians = (-rotation).to_radians() as f64;
            let cos = radians.cos();
            let sin = radians.sin();

            let rotated_x = dx * cos - dy * sin;
            let rotated_y = dx * sin + dy * cos;

            // Translate back
            (rotated_x + cx, rotated_y + cy)
        }
    }

    pub fn set_animation_speed(&self, speed_ms: u64) {
        self.imp().animation_speed.set(speed_ms);
    }

    pub fn set_animations_enabled(&self, enabled: bool) {
        self.imp().animations_enabled.set(enabled);
    }

    pub fn set_animation_overshoot(&self, overshoot: f64) {
        self.imp().animation_overshoot.set(overshoot);
    }

    pub fn set_rotation_with_animation(&self, new_rotation: f64) {
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

            layout.imp().rotation.set(new_rotation as f32);
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
            new_rotation,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smearor_rotation_deg0() {
        let rotation = SmearorRotation::Deg0;
        assert_eq!(rotation.to_degrees(), 0.0);
    }

    #[test]
    fn test_smearor_rotation_deg90() {
        let rotation = SmearorRotation::Deg90;
        assert_eq!(rotation.to_degrees(), 90.0);
    }

    #[test]
    fn test_smearor_rotation_deg180() {
        let rotation = SmearorRotation::Deg180;
        assert_eq!(rotation.to_degrees(), 180.0);
    }

    #[test]
    fn test_smearor_rotation_deg270() {
        let rotation = SmearorRotation::Deg270;
        assert_eq!(rotation.to_degrees(), 270.0);
    }

    #[test]
    fn test_smearor_rotation_custom() {
        let rotation = SmearorRotation::Deg(45.0);
        assert_eq!(rotation.to_degrees(), 45.0);
    }

    #[test]
    fn test_smearor_rotation_from_str() {
        let rotation = SmearorRotation::from("90");
        assert_eq!(rotation.to_degrees(), 90.0);
    }

    #[test]
    fn test_smearor_rotation_from_str_deg() {
        let rotation = SmearorRotation::from("deg180");
        assert_eq!(rotation.to_degrees(), 180.0);
    }

    #[test]
    fn test_smearor_rotation_from_str_custom() {
        let rotation = SmearorRotation::from("45");
        assert_eq!(rotation.to_degrees(), 45.0);
    }
}
