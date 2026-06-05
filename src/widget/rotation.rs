use crate::RotationWidget;
use crate::SmearorRotation;
use crate::widget::layout::RotatedLayout;
use glib::object::Cast;
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk4::prelude::WidgetExt;
use tracing::error;

pub trait RotationControlHandler {
    /// Sets the rotation in degrees
    fn set_rotation(&self, rotation: SmearorRotation);

    /// Returns the current rotation in degrees
    fn rotation(&self) -> f32;
}

impl RotationControlHandler for RotationWidget {
    fn set_rotation(&self, rotation: SmearorRotation) {
        let Some(layout) = self.layout_manager() else {
            error!("Failed to get layout manager in set_rotation");
            return;
        };
        let Ok(layout) = layout.downcast::<RotatedLayout>() else {
            error!("Layout manager is not RotatedLayout in set_rotation");
            return;
        };
        layout.imp().rotation.set(rotation.to_degrees());
        self.queue_allocate();
    }

    fn rotation(&self) -> f32 {
        let Some(layout_manager) = self.layout_manager() else {
            error!("Failed to get layout manager in rotation");
            return 0.0;
        };
        let Ok(rotated_layout) = layout_manager.downcast::<RotatedLayout>() else {
            error!("Layout manager is not RotatedLayout in rotation");
            return 0.0;
        };
        rotated_layout.imp().rotation.get()
    }
}
