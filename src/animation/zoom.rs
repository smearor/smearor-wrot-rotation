use crate::animation::functions::easing::EasingFunction;
use smearor_commons_geometry::RotationRad;
use smearor_commons_geometry::ScaleFactor;
use smearor_commons_geometry::SmoothingAlpha;
use std::time::Duration;
use std::time::Instant;
use tracing::debug;

/// The animation's current output: the eased rotation on the shortest
/// path between start and end angle plus the eased zoom factor.
#[derive(Debug, Clone, Copy)]
pub struct AnimationValues {
    /// Current rotation.
    pub rotation: RotationRad,
    /// Current zoom factor.
    pub scale: ScaleFactor,
}

pub struct RotationZoomAnimation {
    start_rotation: RotationRad,
    end_rotation: RotationRad,
    start_scale: ScaleFactor,
    mid_scale: ScaleFactor,
    end_scale: ScaleFactor,
    duration: Duration,
    easing_function: EasingFunction,
    start_time: Option<Instant>,
}

impl RotationZoomAnimation {
    pub fn new(
        start_rotation: RotationRad,
        end_rotation: RotationRad,
        start_scale: ScaleFactor,
        mid_scale: ScaleFactor,
        end_scale: ScaleFactor,
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

    pub fn get_current_values(&self) -> Option<AnimationValues> {
        let start_time = self.start_time?;
        let elapsed = start_time.elapsed();
        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();

        if progress >= 1.0 {
            return Some(AnimationValues {
                rotation: self.end_rotation,
                scale: self.end_scale,
            });
        }

        let eased_progress = self.apply_easing(progress);
        let current = RotationRad::ema(self.start_rotation, self.end_rotation, SmoothingAlpha::new(eased_progress));
        // Normalize to the 0-2π range
        let current_rotation = current.normalized();
        let current_scale = ScaleFactor::lerp(self.start_scale, self.end_scale, eased_progress);
        Some(AnimationValues {
            rotation: current_rotation,
            scale: current_scale,
        })
    }

    pub fn get_current_values_with_phases(&self) -> Option<AnimationValues> {
        let start_time = self.start_time?;
        let elapsed = start_time.elapsed();
        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();

        if progress >= 1.0 {
            return Some(AnimationValues {
                rotation: self.end_rotation,
                scale: self.end_scale,
            });
        }

        let eased_progress = self.apply_easing(progress);
        let current = RotationRad::ema(self.start_rotation, self.end_rotation, SmoothingAlpha::new(eased_progress));
        // Normalize to the 0-2π range
        let current_rotation = current.normalized();
        debug!(
            "start_rotation {} current_rotation {:?} end_rotation {} eased_progress {eased_progress}",
            self.start_rotation.to_degrees(),
            current_rotation.to_degrees(),
            self.end_rotation.to_degrees()
        );

        // Three-phase zoom:
        // 0-33%: Zoom out (1.0 -> 0.9)
        // 33-66%: No zoom (0.9 -> 0.9)
        // 66-100%: Zoom in (0.9 -> 1.0)
        let mid_scale_rev = 0.1 * self.mid_scale.as_f64();
        let current_scale = if progress < 0.33 {
            // Phase 1: Zoom out
            let phase_progress = progress / 0.33;
            ScaleFactor::new(1.0 - (mid_scale_rev * phase_progress))
        } else if progress < 0.66 {
            // Phase 2: No zoom
            self.mid_scale
        } else {
            // Phase 3: Zoom in
            let phase_progress = (progress - 0.66) / 0.34;
            ScaleFactor::new(self.mid_scale.as_f64() + (mid_scale_rev * phase_progress))
        };

        Some(AnimationValues {
            rotation: current_rotation,
            scale: current_scale,
        })
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
        let mut animation = RotationZoomAnimation::new(
            RotationRad::from_degrees(0.0),
            RotationRad::from_degrees(90.0),
            ScaleFactor::ONE,
            ScaleFactor::new(0.8),
            ScaleFactor::new(1.2),
            Duration::from_millis(100),
            EasingFunction::Linear,
        );
        animation.start();
        std::thread::sleep(Duration::from_millis(50));
        let values = animation.get_current_values().unwrap();
        let degrees = values.rotation.to_degrees();
        assert!(degrees > 40.0 && degrees < 50.0);
        let scale = values.scale.as_f64();
        assert!(scale > 1.08 && scale < 1.12);
    }
}
