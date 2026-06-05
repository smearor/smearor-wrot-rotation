use crate::animation::functions::easing::EasingFunction;
use crate::animation::rotation::RotationAnimation;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;

pub struct RotationZoomAnimation {
    start_rotation: f64,
    end_rotation: f64,
    start_scale: f64,
    mid_scale: f64,
    end_scale: f64,
    duration: Duration,
    easing_function: EasingFunction,
    start_time: Option<Instant>,
}

impl RotationZoomAnimation {
    pub fn new(
        start_rotation: f64,
        end_rotation: f64,
        start_scale: f64,
        mid_scale: f64,
        end_scale: f64,
        duration: Duration,
        easing_function: EasingFunction,
    ) -> Self {
        Self {
            start_rotation,
            end_rotation,
            start_scale,
            mid_scale,
            end_scale,
            duration,
            easing_function,
            start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn get_current_values(&self) -> Option<(f64, f64)> {
        let start_time = self.start_time?;
        let elapsed = start_time.elapsed();
        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();

        if progress >= 1.0 {
            return Some((self.end_rotation, self.end_scale));
        }

        let eased_progress = self.apply_easing(progress);
        let delta = RotationAnimation::calculate_shortest_delta(self.start_rotation, self.end_rotation);
        let current_rotation = self.start_rotation + delta * eased_progress;
        // Normalize to 0-360 range
        let current_rotation = current_rotation.rem_euclid(360.0);
        let current_scale = self.start_scale + (self.end_scale - self.start_scale) * eased_progress;
        Some((current_rotation, current_scale))
    }

    pub fn get_current_values_with_phases(&self) -> Option<(f64, f64)> {
        let start_time = self.start_time?;
        let elapsed = start_time.elapsed();
        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();

        if progress >= 1.0 {
            return Some((self.end_rotation, self.end_scale));
        }

        let eased_progress = self.apply_easing(progress);
        let delta = RotationAnimation::calculate_shortest_delta(self.start_rotation, self.end_rotation);
        let current_rotation = self.start_rotation + delta * eased_progress;
        // Normalize to 0-360 range
        let current_rotation = current_rotation.rem_euclid(360.0);
        debug!(
            "start_rotation {} current_rotation {current_rotation} end_rotation {} delta {delta} eased_progress {eased_progress}",
            self.start_rotation, self.end_rotation
        );

        // Three-phase zoom:
        // 0-33%: Zoom out (1.0 -> 0.9)
        // 33-66%: No zoom (0.9 -> 0.9)
        // 66-100%: Zoom in (0.9 -> 1.0)
        let mid_scale_rev = 0.1 * self.mid_scale;
        let current_scale = if progress < 0.33 {
            // Phase 1: Zoom out
            let phase_progress = progress / 0.33;
            1.0 - (mid_scale_rev * phase_progress)
        } else if progress < 0.66 {
            // Phase 2: No zoom
            self.mid_scale
        } else {
            // Phase 3: Zoom in
            let phase_progress = (progress - 0.66) / 0.34;
            self.mid_scale + (mid_scale_rev * phase_progress)
        };

        Some((current_rotation, current_scale))
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
    fn test_rotation_zoom_animation() {
        let mut animation = RotationZoomAnimation::new(0.0, 90.0, 1.0, 0.8, 1.2, Duration::from_millis(100), EasingFunction::Linear);
        animation.start();
        std::thread::sleep(Duration::from_millis(50));
        let (rotation, scale) = animation.get_current_values().unwrap();
        assert!(rotation > 40.0 && rotation < 50.0);
        assert!(scale > 1.08 && scale < 1.12);
    }
}
