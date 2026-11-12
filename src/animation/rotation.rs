use crate::animation::functions::easing::EasingFunction;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::SmoothingAlpha;
use std::time::Duration;
use std::time::Instant;

pub struct RotationAnimation {
    start_value: RotationRad,
    end_value: RotationRad,
    duration: Duration,
    easing_function: EasingFunction,
    start_time: Option<Instant>,
}

impl RotationAnimation {
    pub fn new(start_value: RotationRad, end_value: RotationRad, duration: Duration, easing_function: EasingFunction) -> Self {
        Self {
            start_value,
            end_value,
            duration,
            easing_function,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn get_current_value(&self) -> Option<RotationRad> {
        let start_time = self.start_time?;
        let elapsed = start_time.elapsed();
        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();

        if progress >= 1.0 {
            return Some(self.end_value);
        }

        // Shortest-path interpolation between the two angles.
        let eased_progress = self.apply_easing(progress);
        let current = RotationRad::ema(self.start_value, self.end_value, SmoothingAlpha::new(eased_progress));
        // Normalize to the 0-2π range
        Some(current.normalized())
    }

    pub fn is_complete(&self) -> bool {
        if let Some(start_time) = self.start_time {
            start_time.elapsed() >= self.duration
        } else {
            false
        }
    }

    fn apply_easing(&self, progress: f64) -> f64 {
        match &self.easing_function {
            EasingFunction::Linear => progress,
            EasingFunction::EaseInOut => {
                if progress < 0.5 {
                    2.0 * progress * progress
                } else {
                    -1.0 + (4.0 - 2.0 * progress) * progress
                }
            }
            EasingFunction::Overshoot { overshoot_amount } => {
                let s = overshoot_amount * 1.525;
                if progress < 0.5 {
                    let p = progress * 2.0;
                    0.5 * (p * p * ((s + 1.0) * p - s))
                } else {
                    let p = progress * 2.0 - 2.0;
                    0.5 * (p * p * ((s + 1.0) * p + s) + 2.0)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_linear() {
        let mut animation = RotationAnimation::new(
            RotationRad::from_degrees(0.0),
            RotationRad::from_degrees(100.0),
            Duration::from_millis(100),
            EasingFunction::Linear,
        );
        animation.start();
        std::thread::sleep(Duration::from_millis(50));
        let value = animation.get_current_value();
        assert!(value.is_some());
        let degrees = value.unwrap().to_degrees();
        assert!(degrees > 40.0 && degrees < 60.0);
    }

    #[test]
    fn test_animation_complete() {
        let mut animation = RotationAnimation::new(
            RotationRad::from_degrees(0.0),
            RotationRad::from_degrees(100.0),
            Duration::from_millis(10),
            EasingFunction::Linear,
        );
        animation.start();
        std::thread::sleep(Duration::from_millis(20));
        assert!(animation.is_complete());
        assert_eq!(animation.get_current_value(), Some(RotationRad::from_degrees(100.0)));
    }

    #[test]
    fn test_animation_overshoot() {
        let mut animation = RotationAnimation::new(
            RotationRad::from_degrees(0.0),
            RotationRad::from_degrees(100.0),
            Duration::from_millis(100),
            EasingFunction::Overshoot { overshoot_amount: 1.7 },
        );
        animation.start();
        std::thread::sleep(Duration::from_millis(50));
        let value = animation.get_current_value();
        assert!(value.is_some());
        // Overshoot should go beyond the target temporarily
        let mid_value = value.unwrap();
        assert!(mid_value.to_degrees() > 40.0);
    }
}
