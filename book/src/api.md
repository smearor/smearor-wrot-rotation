# API Reference

## RotationControlHandler Trait

The `RotationControlHandler` trait provides the primary interface for controlling rotation on a `RotationWidget`:

```rust
pub trait RotationControlHandler {
    fn set_rotation(&self, rotation: SmearorRotation);
    fn rotation(&self) -> f32;
}
```

### `set_rotation(SmearorRotation)`

Sets the rotation instantly without animation. The layout is updated immediately via `queue_allocate()`.

```rust
use smearor_wrot_rotation::{RotationControlHandler, RotationWidget, SmearorRotation};

rotation_widget.set_rotation(SmearorRotation::Deg90);
rotation_widget.set_rotation(SmearorRotation::Deg(45.0));
```

### `rotation() -> f32`

Returns the current rotation angle in degrees as a `f32`.

```rust
let current: f32 = rotation_widget.rotation();  // e.g. 90.0
```

## Builder Methods

Builder-style methods consume `self` and return `Self`, enabling chained construction:

### `with_gesture_rotation_enabled(bool)`

Enables or disables gesture-based rotation at construction time. Defaults to `true`.

```rust
let widget = RotationWidget::new(SmearorRotation::Deg0)
    .with_gesture_rotation_enabled(false);
```

### `with_animations_enabled(bool)`

Enables or disables animations at construction time. Defaults to `true`.

```rust
let widget = RotationWidget::new(SmearorRotation::Deg0)
    .with_animations_enabled(false);
```

### Chaining

Both builder methods can be chained:

```rust
let widget = RotationWidget::new(SmearorRotation::Deg0)
    .with_gesture_rotation_enabled(false)
    .with_animations_enabled(true);
```

## Setter Methods

### `set_child(Option<&impl IsA<gtk4::Widget>>)`

Sets or removes the child widget. Passing `None` detaches the current child.

```rust
let button = gtk4::Button::with_label("Click Me!");
rotation_widget.set_child(Some(&button));

// Later, detach:
rotation_widget.set_child(None::<&gtk4::Button>);
```

### `set_animations_enabled(bool)`

Enables or disables transition animations at runtime. Default: `true`.

```rust
rotation_widget.set_animations_enabled(false);
```

### `set_animation_speed(u64)`

Sets the animation duration in milliseconds. Default: `500`.

```rust
rotation_widget.set_animation_speed(1000);  // 1 second
```

### `set_animation_overshoot(f64)`

Sets the spring overshoot intensity for snap transitions. Higher values produce stronger bounce-back. Default: `1.7`.

```rust
rotation_widget.set_animation_overshoot(2.5);
```

### `set_rotation_with_animation(f64)`

Triggers an animated rotation to the target angle (in degrees) using the three-phase zoom effect. When animations are disabled, the rotation is applied instantly.

```rust
rotation_widget.set_rotation_with_animation(90.0);
rotation_widget.set_rotation_with_animation(270.0);
```

Note: This method accepts `f64` (raw degrees), not `SmearorRotation`.

### `set_gesture_rotation_enabled(bool)`

Enables or disables touch gesture rotation at runtime. When disabled, the gesture controller's propagation phase is set to `None`, meaning it receives no events. Default: `true`.

```rust
rotation_widget.set_gesture_rotation_enabled(false);
```

## Layer Shell (Optional Feature)

The `layer-shell` feature (enabled by default) provides:

- [`SmearorLayer`](layer.md) enum with `gtk4_layer_shell::Layer` conversion
- `SmearorRotation::anchor() -> Option<gtk4_layer_shell::Edge>`

To disable layer shell support and remove the `gtk4-layer-shell` dependency:

```toml
[dependencies]
smearor-wrot-rotation = { version = "0.1", default-features = false }
```

When the feature is disabled, `SmearorLayer` and `anchor()` are not available. All other functionality (`RotationWidget`, `SmearorRotation`, animations, input transform) works without the `layer-shell` feature.

## Input Transform

### `input_transform(f64, f64) -> (f64, f64)`

Maps screen-relative coordinates to child-relative coordinates by applying the inverse rotation around the widget center. When rotation is 0°, coordinates are returned unchanged.

```rust
let (child_x, child_y) = rotation_widget.input_transform(screen_x, screen_y);
```

## Method Reference Table

| Method | Parameters | Default | Description |
|--------|-----------|---------|-------------|
| `set_rotation` | `SmearorRotation` | `Deg0` | Instant rotation |
| `rotation` | — | — | Current angle in degrees |
| `set_rotation_with_animation` | `f64` (degrees) | — | Animated rotation with zoom |
| `set_child` | `Option<&Widget>` | `None` | Set or remove child widget |
| `set_animations_enabled` | `bool` | `true` | Toggle animations |
| `set_animation_speed` | `u64` (ms) | `500` | Animation duration |
| `set_animation_overshoot` | `f64` | `1.7` | Spring overshoot intensity |
| `set_gesture_rotation_enabled` | `bool` | `true` | Toggle gesture rotation |
| `with_gesture_rotation_enabled` | `bool` | `true` | Builder: gesture rotation |
| `with_animations_enabled` | `bool` | `true` | Builder: animations |
| `input_transform` | `f64, f64` | — | Screen to child coordinates |
