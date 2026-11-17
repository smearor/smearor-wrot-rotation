use crate::widget::fill_strategy::FillStrategy;
use crate::widget::layout::RotatedLayout;
use glib::subclass::object::ObjectImpl;
use glib::subclass::prelude::ObjectSubclass;
use glib::value::ToValue;
use gtk4::LayoutManager;
use gtk4::Orientation;
use gtk4::gsk::Transform;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::layout_manager::LayoutManagerImpl;
use smearor_commons_geometry::AspectRatio;
use smearor_commons_geometry::CenteredTransform;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::ScaleFactor;
use smearor_commons_geometry::Size;
use smearor_commons_geometry::WidgetSizeExt;
use std::cell::Cell;
use std::sync::LazyLock;
use tracing::debug;
use tracing::trace;
use tracing::warn;

pub struct RotatedLayoutImpl {
    pub rotation: Cell<RotationRad>,
    pub scale: Cell<ScaleFactor>,
    /// When `true`, the child is allocated its measured natural size.
    /// When `false`, it is resized to the largest rectangle with its
    /// natural aspect ratio whose rotated bounding box fits inside the
    /// allocation (`unrotated_fit` - a one-sided condition that is never
    /// singular).
    pub natural_child_size: Cell<bool>,
    /// When `true` (default), `measure` requests the rotated bounding
    /// box, so the allocation always contains the rotated child and the
    /// viewport grows with rotation. When `false`, the request stays
    /// unrotated; `allocate` still centers the child, so it overflows
    /// the fixed viewport symmetrically. Combine with
    /// `natural_child_size` to keep the child at its natural size, or
    /// leave it disabled to shrink the child via `unrotated_fit` so it
    /// still fits rotated inside the fixed viewport.
    pub measure_rotated_bounds: Cell<bool>,
    /// When `true`, the child is stretched to fill the whole allocation:
    /// it is allocated the size whose rotated bounding box exactly equals
    /// the allocation (`unrotated_bounds`). Takes precedence over
    /// `natural_child_size`. Where the exact solve is infeasible the
    /// `fill_strategy` fallback resolves the allocation.
    pub fill: Cell<bool>,
    /// How `fill` resolves rotation angles where the exact
    /// `unrotated_bounds` solve is infeasible - see [`FillStrategy`].
    pub fill_strategy: Cell<FillStrategy>,
}

impl Default for RotatedLayoutImpl {
    fn default() -> Self {
        Self {
            rotation: Cell::new(RotationRad::ZERO),
            scale: Cell::new(ScaleFactor::ONE),
            natural_child_size: Cell::new(false),
            measure_rotated_bounds: Cell::new(true),
            fill: Cell::new(false),
            fill_strategy: Cell::new(FillStrategy::default()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RotatedLayoutImpl {
    const NAME: &'static str = "RotatedLayout";
    type Type = RotatedLayout;
    type ParentType = LayoutManager;
}

/// Property names shared by `properties`, `property` and `set_property`.
mod prop {
    pub const ROTATION: &str = "rotation";
    pub const SCALE: &str = "scale";
    pub const NATURAL_CHILD_SIZE: &str = "natural-child-size";
    pub const MEASURE_ROTATED_BOUNDS: &str = "measure-rotated-bounds";
    pub const FILL: &str = "fill";
    pub const FILL_STRATEGY: &str = "fill-strategy";
}

impl ObjectImpl for RotatedLayoutImpl {
    /// The layout's state is stored in `Cell` newtypes that do not
    /// implement `glib::property::Property`, so the properties are
    /// registered manually as plain `f64`/`bool` values mapped onto the
    /// cells. Registered properties are required for `notify::rotation`
    /// to reach consumers (e.g. the floating board's re-centering).
    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
            vec![
                glib::ParamSpecDouble::builder(prop::ROTATION).build(),
                glib::ParamSpecDouble::builder(prop::SCALE).build(),
                glib::ParamSpecBoolean::builder(prop::NATURAL_CHILD_SIZE).build(),
                glib::ParamSpecBoolean::builder(prop::MEASURE_ROTATED_BOUNDS).build(),
                glib::ParamSpecBoolean::builder(prop::FILL).build(),
                glib::ParamSpecEnum::builder::<FillStrategy>(prop::FILL_STRATEGY).build(),
            ]
        });
        PROPERTIES.as_slice()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            prop::ROTATION => self.rotation.get().as_f64().to_value(),
            prop::SCALE => self.scale.get().as_f64().to_value(),
            prop::NATURAL_CHILD_SIZE => self.natural_child_size.get().to_value(),
            prop::MEASURE_ROTATED_BOUNDS => self.measure_rotated_bounds.get().to_value(),
            prop::FILL => self.fill.get().to_value(),
            prop::FILL_STRATEGY => self.fill_strategy.get().to_value(),
            name => {
                warn!(%name, "unknown RotatedLayout property");
                pspec.default_value().to_owned()
            }
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            prop::ROTATION => {
                if let Ok(rotation) = value.get::<f64>() {
                    self.rotation.set(RotationRad::new(rotation));
                }
            }
            prop::SCALE => {
                if let Ok(scale) = value.get::<f64>() {
                    self.scale.set(ScaleFactor::new(scale));
                }
            }
            prop::NATURAL_CHILD_SIZE => {
                if let Ok(enabled) = value.get::<bool>() {
                    self.natural_child_size.set(enabled);
                }
            }
            prop::MEASURE_ROTATED_BOUNDS => {
                if let Ok(enabled) = value.get::<bool>() {
                    self.measure_rotated_bounds.set(enabled);
                }
            }
            prop::FILL => {
                if let Ok(enabled) = value.get::<bool>() {
                    self.fill.set(enabled);
                }
            }
            prop::FILL_STRATEGY => {
                if let Ok(strategy) = value.get::<FillStrategy>() {
                    self.fill_strategy.set(strategy);
                }
            }
            name => warn!(%name, "unknown RotatedLayout property"),
        }
    }
}

impl LayoutManagerImpl for RotatedLayoutImpl {
    fn allocate(&self, widget: &gtk4::Widget, width: i32, height: i32, baseline: i32) {
        let Some(c) = widget.first_child().filter(|c| c.should_layout()) else {
            return;
        };
        let rotation = self.rotation.get();
        let allocation = Size::new(width, height);
        let center = allocation.to_f32().half();
        let (_, child_nat) = c.measure_size();
        let target_size: Size<i32> = if self.fill.get() {
            // The child is stretched so its rendered (rotated and
            // scaled) bounding box covers the allocation. The exact
            // `unrotated_bounds` solve runs against the
            // scale-compensated allocation so the zoom of rotation
            // animations does not shrink the covered area. The solve
            // only exists while the viewport aspect stays inside the
            // `[tan θ, cot θ]` band a rotated rect can bound - outside
            // it the required child aspect degenerates (negative or
            // non-finite components near `45° + k·90°`) and
            // `fill_strategy` takes over.
            let cover_target = allocation.to_f64().map(|v| v / self.scale.get().as_f64());
            let solved = cover_target.unrotated_bounds(rotation);
            let exact_valid = solved.x.is_finite() && solved.y.is_finite() && solved.x > 0.0 && solved.y > 0.0;
            let nat_aspect = child_nat.to_f64().aspect().unwrap_or(AspectRatio::ONE);
            match self.fill_strategy.get() {
                // The aspect-clamped cover reproduces the exact solve
                // wherever its aspect is in bounds, so it stays
                // continuous across the whole rotation.
                FillStrategy::Clamp => cover_target.unrotated_cover(fill_aspect(solved, nat_aspect), rotation).map(|v| v.ceil() as i32),
                FillStrategy::Fit | FillStrategy::Cover if exact_valid => Size::new(solved.x.ceil() as i32, solved.y.ceil() as i32),
                // `unrotated_fit` on the unscaled allocation mirrors
                // the non-fill letterbox exactly, including through
                // the zoom dip.
                FillStrategy::Fit => allocation.to_f64().unrotated_fit(nat_aspect, rotation).map(|v| v as i32),
                FillStrategy::Cover => cover_target.unrotated_cover(nat_aspect, rotation).map(|v| v.ceil() as i32),
            }
        } else if self.natural_child_size.get() {
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

        let transform: Transform = CenteredTransform::new(center, target_size.to_f32(), rotation, self.scale.get()).into();

        trace!("size {target_size} baseline {baseline} transform {transform:?}");
        c.allocate(target_size.x, target_size.y, baseline, Some(transform));
    }

    fn measure(&self, widget: &gtk4::Widget, orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
        let rotation = self.rotation.get();

        let Some(child) = widget.first_child() else {
            debug!("No child");
            return (0, 0, -1, -1);
        };
        let (child_min, child_nat) = child.measure_size();
        trace!("child_min {child_min} child_nat {child_nat}");

        // `measure_rotated_bounds` requests the rotated bounding box, so
        // the allocation always contains the rotated child; disabled,
        // the request stays unrotated and the child overflows its
        // allocation symmetrically (`allocate` centers either way).
        let bounds = |size: Size<i32>| {
            if self.measure_rotated_bounds.get() {
                let rotated = size.to_f64().rotated_bounds(rotation).to_i32_ceil();
                Size::new(rotated.x, rotated.y)
            } else {
                size
            }
        };
        let (min, nat) = (bounds(child_min), bounds(child_nat));
        if orientation == Orientation::Horizontal {
            (min.x, nat.x, -1, -1)
        } else {
            (min.y, nat.y, -1, -1)
        }
    }
}

/// The child aspect for [`FillStrategy::Clamp`]: the exact solve's
/// aspect `solved.x / solved.y`, clamped to a bounded distortion range
/// around the child's natural aspect. Covering with the exact aspect
/// reproduces `unrotated_bounds` wherever the solve is feasible (both
/// axes tight at the same value), so the resulting allocation is
/// continuous across the whole rotation. Inside the infeasible band
/// the solve's aspect sweeps `∞ → 0` and the clamp morphs the child
/// bar-to-square-to-bar without jumps; a degenerate `NaN` (square
/// viewport at `45° + k·90°`) falls back to the natural aspect.
fn fill_aspect(solved: Size<f64>, natural: AspectRatio) -> AspectRatio {
    /// Bound on how far the fill solve may distort the child's natural
    /// aspect ratio - `1` keeps it exactly, `4` allows a `4:1` bar.
    const MAX_DISTORTION: f64 = 4.0;
    let natural_f64 = natural.as_f64();
    let raw = solved.signed_aspect();
    let clamped = if raw.is_nan() {
        natural_f64
    } else if !raw.is_finite() {
        natural_f64 * MAX_DISTORTION
    } else {
        raw.abs().clamp(natural_f64 / MAX_DISTORTION, natural_f64 * MAX_DISTORTION)
    };
    AspectRatio::new(clamped).unwrap_or(natural)
}
