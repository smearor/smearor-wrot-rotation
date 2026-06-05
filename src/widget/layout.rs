use crate::widget::imp::layout::RotatedLayoutImpl;
use gtk4::LayoutManager;

glib::wrapper! {
    pub struct RotatedLayout(ObjectSubclass<RotatedLayoutImpl>)
        @extends LayoutManager;
}
