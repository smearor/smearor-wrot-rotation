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
use smearor_commons_animation::RotationSnap;
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

        // Gesture lifecycle: `connect_begin` starts a fresh snap
        // session with the rotation at drag start, `angle_changed`
        // feeds cumulative angle reports into it, `connect_end`
        // restores the initial rotation when the drag never snapped.
        let snap_session = Rc::new(RefCell::new(None::<RotationSnap>));

        let widget_clone = widget.clone();
        let snap_session_clone = snap_session.clone();
        rotate_gesture.connect_begin(move |_gesture, _sequence| {
            let Some(layout) = widget_clone.layout_manager() else {
                error!("Failed to get layout manager in rotate gesture begin");
                return;
            };

            let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                error!("Layout manager is not RotatedLayout in rotate gesture begin");
                return;
            };

            *snap_session_clone.borrow_mut() = Some(RotationSnap::new(layout.imp().rotation.get()));
        });

        let widget_clone = widget.clone();
        let snap_session_clone = snap_session.clone();
        rotate_gesture.connect_angle_changed(move |_gesture, _angle, angle_delta| {
            debug!("angle_delta {angle_delta}");
            let update = snap_session_clone
                .borrow_mut()
                .as_mut()
                .map(|session| session.update(RotationRad::new(angle_delta)))
                .unwrap_or_default();

            // Engagement feedback: nudge a few degrees once the drag
            // direction locks in.
            if let Some(feedback) = update.feedback {
                let Some(layout) = widget_clone.layout_manager() else {
                    error!("Failed to get layout manager in rotate gesture feedback");
                    return;
                };

                let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                    error!("Layout manager is not RotatedLayout in rotate gesture feedback");
                    return;
                };

                let new_rotation = layout.imp().rotation.get() + feedback;
                debug!("Applying feedback rotation: {new_rotation:?}");
                widget_clone.set_layout_rotation(&layout, new_rotation);
            }

            let Some(target) = update.target else {
                return;
            };

            let Some(layout) = widget_clone.layout_manager() else {
                error!("Failed to get layout manager in rotate gesture");
                return;
            };

            let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                error!("Layout manager is not RotatedLayout in rotate gesture");
                return;
            };

            let current_rotation = layout.imp().rotation.get();
            let final_rotation = target.to_rotation_rad();
            debug!("Snap: current_rotation={current_rotation:?} target={target:?}");

            if !widget_clone.imp().animations_enabled.get() {
                // If animations are disabled, set rotation immediately.
                widget_clone.set_layout_rotation(&layout, final_rotation);
                return;
            }

            // Start overshoot animation: rotate on the shortest arc
            // while zooming out, holding and zooming back in.
            let animation_speed_ms = widget_clone.imp().animation_speed.get();
            let overshoot_amount = widget_clone.imp().animation_overshoot.get();
            let mut animation = RotationZoomAnimation::new(
                (
                    Span::new(current_rotation, final_rotation),
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
                widget.set_layout_rotation(&layout, rotation);
                widget.set_layout_scale(&layout, scale);
                ControlFlow::Continue
            });
        });

        let widget_clone = widget.clone();
        rotate_gesture.connect_end(move |_gesture, _sequence| {
            let Some(session) = snap_session.borrow_mut().take() else {
                return;
            };

            let Some(restore) = session.end() else {
                return;
            };

            let Some(layout) = widget_clone.layout_manager() else {
                error!("Failed to get layout manager in rotate gesture end");
                return;
            };

            let Ok(layout) = layout.downcast::<RotatedLayout>() else {
                error!("Layout manager is not RotatedLayout in rotate gesture end");
                return;
            };

            widget_clone.set_layout_rotation(&layout, restore);
            debug!("Reset to initial rotation {restore:?} because threshold was not reached");
        });
    }

    fn dispose(&self) {
        if let Some(child) = self.child.borrow_mut().take() {
            child.unparent();
        }
    }
}

impl WidgetImpl for RotationWidgetImpl {
    /// `gtk_widget_do_pick` consults `contains` only as a leaf check: a
    /// container without an override claims picks in every area its
    /// children reject - the invisible corners of a rotated child or a
    /// non-rectangular child's unused allocation - and swallows them
    /// instead of letting them fall through to the widgets below.
    /// Delegating to the child keeps the transform stack's hit shape
    /// identical to the child's own.
    fn contains(&self, x: f64, y: f64) -> bool {
        let widget = self.obj();
        let Some(child) = widget.first_child() else {
            return false;
        };
        widget
            .compute_point(&child, &gtk4::graphene::Point::new(x as f32, y as f32))
            .map(|point| child.contains(f64::from(point.x()), f64::from(point.y())))
            .unwrap_or_default()
    }

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
