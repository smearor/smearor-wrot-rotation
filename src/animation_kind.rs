/// The animation applied by
/// [`RotationWidget::set_rotation_with_animation`](crate::RotationWidget::set_rotation_with_animation).
#[derive(Debug, Clone, Copy, Default)]
pub enum AnimationKind {
    /// Rotate on the shortest arc while zooming out, holding and
    /// zooming back in - a `RotationZoomAnimation`.
    #[default]
    RotationZoom,
    /// Rotate on the shortest arc without zoom - a
    /// `RotationAnimation`.
    Rotation,
}
