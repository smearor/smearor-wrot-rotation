/// Progress curve of an animation: maps linear time progress
/// (`0.0..=1.0`) to eased progress.
pub enum EasingFunction {
    /// Constant speed - eased progress equals time progress.
    Linear,
    /// Slow start and end, fast middle (quadratic ease-in-out).
    EaseInOut,
    /// Goes beyond the target and settles back.
    Overshoot {
        /// Overshoot strength - higher values overshoot further.
        overshoot_amount: f64,
    },
}
