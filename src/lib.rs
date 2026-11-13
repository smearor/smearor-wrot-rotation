//! smearor-wrot-rotation: GTK4 widget for rotating any GTK4 widget with input/output transformation

pub mod animation_kind;
#[cfg(feature = "layer-shell")]
pub mod layer;
pub mod rotation;
pub mod widget;

pub use animation_kind::AnimationKind;
pub use rotation::SmearorRotation;
pub use widget::rotation::RotationControlHandler;
pub use widget::widget::RotationWidget;
