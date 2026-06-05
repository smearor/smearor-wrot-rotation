# The RotationWidget

The primary component of this library is the `RotationWidget` (internally subclassed in GObject as `RotatedBox`).

## Construction

Creating a widget is straightforward:

```rust
use smearor_wrot_rotation::RotationWidget;
use smearor_wrot_rotation::SmearorRotation;

// Create a widget at a standard rotation (90 degrees)
let rotation_widget = RotationWidget::new(SmearorRotation::Deg90);
```

You can set any GTK4 widget as its child:

```rust
use gtk4::Button;

let button = Button::with_label("Interactive Button");
rotation_widget.set_child(Some(&button));
```

## Rotation Orientation Options

The `SmearorRotation` enum represents standard and custom rotation geometries:

```rust
pub enum SmearorRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
    Deg(f32), // Arbitrary angle in degrees
}
```
