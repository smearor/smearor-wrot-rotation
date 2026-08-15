# The RotationWidget

The primary component of this library is the `RotationWidget` (internally subclassed in GObject as `RotationWidget`).

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
rotation_widget.set_child(Some( & button));
```

## Gesture Rotation

By default, the `RotationWidget` responds to `GestureRotate` touch gestures, allowing users to rotate the child widget with a two-finger twist motion. This behavior can be disabled at construction time or at runtime.

### Construction Time

Use the builder-style method `with_gesture_rotation_enabled`:

```rust
// Create a widget with gesture rotation disabled
let rotation_widget = RotationWidget::new(SmearorRotation::Deg0)
    .with_gesture_rotation_enabled(false);

// Explicitly enabled (default)
let rotation_widget = RotationWidget::new(SmearorRotation::Deg0)
    .with_gesture_rotation_enabled(true);
```

### Runtime

Use `set_gesture_rotation_enabled` to toggle at any time:

```rust
// Disable gesture-based rotation
rotation_widget.set_gesture_rotation_enabled(false);

// Re-enable gesture-based rotation (default)
rotation_widget.set_gesture_rotation_enabled(true);
```

When gesture rotation is disabled, the gesture controller's propagation phase is set to `None`, meaning it receives no events at all. This ensures child widgets retain full access to two-finger gestures (e.g., pinch-to-zoom). Programmatic rotation via `set_rotation` and `set_rotation_with_animation` remains fully functional regardless of the gesture setting.

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
