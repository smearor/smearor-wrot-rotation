/// How the `fill` allocation resolves rotation angles where the exact
/// `unrotated_bounds` solve is infeasible: the bounding-box aspect a
/// rotated rectangle can produce is bounded by `tan θ`/`cot θ`, so a
/// viewport aspect outside that range admits no rect whose rotated
/// bounds exactly equal the allocation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, glib::Enum)]
#[enum_type(name = "SmearorWrotFillStrategy")]
pub enum FillStrategy {
    /// Aspect-preserving inscribed fit: the child keeps its natural
    /// aspect and shrinks to the largest rect whose rotated bounds fit
    /// inside the allocation (`unrotated_fit`). Letterboxes inside the
    /// infeasible band - the original `fill` fallback behavior.
    #[default]
    Fit,
    /// Aspect-preserving cover: the child keeps its natural aspect and
    /// grows to the smallest rect whose rotated bounds contain the
    /// allocation - the `max`-sided counterpart of `unrotated_fit`.
    /// Covers without collapsing, but snaps at the band edge and
    /// overhangs the allocation on one axis.
    Cover,
    /// Aspect-clamped cover: the child aspect follows the exact solve's
    /// aspect clamped to a bounded range around its natural aspect.
    /// Covering with the exact aspect reproduces the exact solve
    /// wherever it is feasible, so the allocation stays continuous
    /// across the whole rotation - the child morphs bar-to-square-to-bar
    /// instead of jumping. Always covers the allocation.
    Clamp,
}
