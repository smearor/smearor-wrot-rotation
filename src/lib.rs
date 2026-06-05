//! smearor-wrot-rotation: GTK4 widget for rotating any GTK4 widget with input/output transformation

pub mod animation;
pub mod layer;
pub mod rotation;
pub mod widget;

pub use rotation::SmearorRotation;
pub use widget::rotation::RotationControlHandler;
pub use widget::widget::RotationWidget;
