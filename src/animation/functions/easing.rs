pub enum EasingFunction {
    Linear,
    EaseInOut,
    Overshoot { overshoot_amount: f64 },
}
