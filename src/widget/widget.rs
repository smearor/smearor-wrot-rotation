use crate::AnimationKind;
use crate::RotationControlHandler;
use crate::rotation::SmearorRotation;
use crate::widget::fill_strategy::FillStrategy;
use crate::widget::imp::widget::RotationWidgetImpl;
use crate::widget::layout::RotatedLayout;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Overflow;
use gtk4::PropagationPhase;
use gtk4::Widget;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use smearor_commons_animation::EasingFunction;
use smearor_commons_animation::RotationAnimation;
use smearor_commons_animation::RotationZoomAnimation;
use smearor_commons_animation::Span;
use smearor_commons_animation::ThreePhaseSpan;
use smearor_commons_animation::drive;
use smearor_commons_geometry::Fraction;
use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::ScaleFactor;
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

    /// Applies a rotation value to the layout, emitting `notify::rotation`
    /// and invalidating the measure chain. Rotation changes the rotated
    /// bounding box the widget requests, so a plain `queue_allocate` would
    /// leave parents allocating at the stale unrotated extent while the
    /// transform already renders the new rotation off-center.
    pub(crate) fn set_layout_rotation(&self, layout: &RotatedLayout, rotation: RotationRad) {
        if layout.imp().rotation.get() == rotation {
            return;
        }
        // `set_property` goes through the registered property and emits
        // `notify::rotation`, which lets consumers like the floating
        // board re-center and re-clamp before the next measure pass.
        layout.set_property("rotation", rotation.as_f64());
        layout.layout_changed();
        self.queue_resize();
    }

    /// Applies a scale value to the layout, emitting `notify::scale`.
    /// Scale does not affect the measured size, so a re-allocation of the
    /// child with the updated transform is sufficient.
    pub(crate) fn set_layout_scale(&self, layout: &RotatedLayout, scale: ScaleFactor) {
        if layout.imp().scale.get() == scale {
            return;
        }
        layout.set_property("scale", scale.as_f64());
        self.queue_allocate();
    }

    /// Enables or disables natural child size allocation. Defaults to
    /// `false`.
    ///
    /// With the default (`false`) the child is resized to the largest
    /// rectangle with its natural aspect ratio whose rotated bounding box
    /// fits inside the allocation. Enabled, the child keeps its measured
    /// natural size; `measure` already reports the rotated bounding
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
        if rotated_layout.imp().natural_child_size.get() != enabled {
            rotated_layout.set_property("natural-child-size", enabled);
            self.queue_allocate();
        }
    }

    /// Builder-style method to configure fill allocation. Defaults to
    /// `false`.
    pub fn with_fill_enabled(self, enabled: bool) -> Self {
        self.set_fill_enabled(enabled);
        self
    }

    /// Enables or disables fill allocation. Defaults to `false`.
    ///
    /// Enabled, the child is stretched to fill the whole allocation: it
    /// is allocated the size whose rotated bounding box exactly equals
    /// the allocation, so a surface-filling child covers the viewport
    /// without letterboxing at any rotation. Takes precedence over
    /// `natural_child_size`. Where the exact solve is infeasible the
    /// [`FillStrategy`] selected by [`Self::set_fill_strategy`]
    /// resolves the allocation.
    ///
    /// Enabling fill also sets `overflow: hidden`: cover strategies let
    /// the child overhang the allocation on one axis, and the clip
    /// keeps that overhang inside the widget bounds. Disabling
    /// restores `overflow: visible`.
    pub fn set_fill_enabled(&self, enabled: bool) {
        let Some(layout) = self.layout_manager() else {
            error!("Failed to get layout manager in set_fill_enabled");
            return;
        };
        let Ok(rotated_layout) = layout.downcast::<RotatedLayout>() else {
            error!("Layout manager is not RotatedLayout in set_fill_enabled");
            return;
        };
        if rotated_layout.imp().fill.get() != enabled {
            rotated_layout.set_property("fill", enabled);
            self.set_overflow(if enabled { Overflow::Hidden } else { Overflow::Visible });
            self.queue_allocate();
        }
    }

    /// Whether the child is stretched to fill the whole allocation.
    pub fn fill_enabled(&self) -> bool {
        self.layout_manager()
            .and_then(|layout| layout.downcast::<RotatedLayout>().ok())
            .map(|rotated_layout| rotated_layout.imp().fill.get())
            .unwrap_or(false)
    }

    /// Builder-style method to configure the fill fallback strategy.
    /// Defaults to [`FillStrategy::Fit`].
    pub fn with_fill_strategy(self, strategy: FillStrategy) -> Self {
        self.set_fill_strategy(strategy);
        self
    }

    /// Selects how `fill` resolves rotation angles where the exact
    /// `unrotated_bounds` solve is infeasible - see [`FillStrategy`]
    /// for the variants. Defaults to [`FillStrategy::Fit`].
    pub fn set_fill_strategy(&self, strategy: FillStrategy) {
        let Some(layout) = self.layout_manager() else {
            error!("Failed to get layout manager in set_fill_strategy");
            return;
        };
        let Ok(rotated_layout) = layout.downcast::<RotatedLayout>() else {
            error!("Layout manager is not RotatedLayout in set_fill_strategy");
            return;
        };
        if rotated_layout.imp().fill_strategy.get() != strategy {
            rotated_layout.set_property("fill-strategy", strategy);
            self.queue_allocate();
        }
    }

    /// The fallback strategy `fill` uses where no exact solve exists.
    pub fn fill_strategy(&self) -> FillStrategy {
        self.layout_manager()
            .and_then(|layout| layout.downcast::<RotatedLayout>().ok())
            .map(|rotated_layout| rotated_layout.imp().fill_strategy.get())
            .unwrap_or_default()
    }

    /// Builder-style method to configure rotated-bounds measurement.
    /// Defaults to `true` (the viewport follows the rotated bounding box).
    pub fn with_measure_rotated_bounds_enabled(self, enabled: bool) -> Self {
        self.set_measure_rotated_bounds_enabled(enabled);
        self
    }

    /// Enables or disables rotated-bounds measurement. Defaults to `true`.
    ///
    /// Enabled, `measure` reports the rotated bounding box: the
    /// allocation always contains the rotated child and the viewport
    /// grows with rotation. Disabled, the widget requests its unrotated
    /// size - the rotated child overflows the allocation symmetrically.
    /// Combine with [`Self::set_natural_child_size_enabled`]: enabled
    /// keeps the child at its natural size, disabled shrinks it via
    /// `unrotated_fit` so it still fits rotated inside the fixed
    /// viewport.
    ///
    /// Note: content outside the allocation is not hit-testable by
    /// default - consumers that need picking in the overflow area must
    /// override `contains` to map through the transform.
    pub fn set_measure_rotated_bounds_enabled(&self, enabled: bool) {
        let Some(layout) = self.layout_manager() else {
            error!("Failed to get layout manager in set_measure_rotated_bounds_enabled");
            return;
        };
        let Ok(rotated_layout) = layout.downcast::<RotatedLayout>() else {
            error!("Layout manager is not RotatedLayout in set_measure_rotated_bounds_enabled");
            return;
        };
        if rotated_layout.imp().measure_rotated_bounds.get() != enabled {
            rotated_layout.set_property("measure-rotated-bounds", enabled);
            rotated_layout.layout_changed();
            self.queue_resize();
        }
    }

    /// Whether the widget requests its rotated bounding box as size.
    pub fn measure_rotated_bounds_enabled(&self) -> bool {
        self.layout_manager()
            .and_then(|layout| layout.downcast::<RotatedLayout>().ok())
            .map(|rotated_layout| rotated_layout.imp().measure_rotated_bounds.get())
            .unwrap_or(true)
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

    /// Builder-style method to select the animation applied by
    /// [`Self::set_rotation_with_animation`]. Defaults to
    /// [`AnimationKind::RotationZoom`].
    pub fn with_animation_kind(self, kind: AnimationKind) -> Self {
        self.set_animation_kind(kind);
        self
    }

    /// Selects the animation applied by
    /// [`Self::set_rotation_with_animation`].
    pub fn set_animation_kind(&self, kind: AnimationKind) {
        self.imp().animation_kind.set(kind);
    }

    /// Builder-style method to configure the scale at the dip of
    /// the three-phase zoom. Defaults to `0.8`; `1.0` means no zoom.
    pub fn with_animation_zoom(self, zoom: ScaleFactor) -> Self {
        self.set_animation_zoom(zoom);
        self
    }

    /// Sets the scale at the dip of the three-phase zoom - how far
    /// the child shrinks mid-animation (`1.0` = no zoom, `0.5` =
    /// half size).
    pub fn set_animation_zoom(&self, zoom: ScaleFactor) {
        self.imp().animation_zoom.set(zoom);
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

            self.set_layout_rotation(&layout, new_rotation);
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

        // Rotation on the shortest arc plus a three-phase zoom
        // (out, hold, back in) - the same overshoot easing the
        // rotate gesture uses.
        let overshoot_amount = imp.animation_overshoot.get();
        let duration = Duration::from_millis(animation_speed_ms);
        match imp.animation_kind.get() {
            // Rotation on the shortest arc plus a three-phase zoom
            // (out, hold, back in).
            AnimationKind::RotationZoom => {
                let mut animation = RotationZoomAnimation::new(
                    (
                        Span::new(current_rotation, new_rotation),
                        ThreePhaseSpan::new(
                            ScaleFactor::ONE,
                            imp.animation_zoom.get(),
                            ScaleFactor::ONE,
                            Fraction::new(1.0 / 3.0),
                            Fraction::new(2.0 / 3.0),
                        ),
                    ),
                    duration,
                    EasingFunction::Overshoot { overshoot_amount },
                );
                animation.start();
                drive(self, animation, |widget, (rotation, scale)| {
                    let Some(layout) = widget.layout_manager() else {
                        return glib::ControlFlow::Continue;
                    };
                    let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                        return glib::ControlFlow::Continue;
                    };
                    widget.set_layout_rotation(&layout, rotation);
                    widget.set_layout_scale(&layout, scale);
                    glib::ControlFlow::Continue
                });
            }
            // Rotation on the shortest arc, no zoom.
            AnimationKind::Rotation => {
                let mut animation = RotationAnimation::new(Span::new(current_rotation, new_rotation), duration, EasingFunction::Overshoot { overshoot_amount });
                animation.start();
                drive(self, animation, |widget, rotation| {
                    let Some(layout) = widget.layout_manager() else {
                        return glib::ControlFlow::Continue;
                    };
                    let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                        return glib::ControlFlow::Continue;
                    };
                    widget.set_layout_rotation(&layout, rotation);
                    glib::ControlFlow::Continue
                });
            }
        }
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
                widget_clone_width.queue_resize();
            });
            new_child.connect_notify_local(Some("height-request"), move |_child, _param| {
                widget_clone_height.queue_resize();
            });
            new_child.set_parent(self);
            *self_child = Some(new_child.clone().upcast());
        }
    }
}
