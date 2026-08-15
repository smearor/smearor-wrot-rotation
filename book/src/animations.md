# Animations and Snapping

`RotationWidget` features smooth, timing-based animations for orientation snapping transitions.

## Configurable Animations

You can customize rotation transition parameters:

```rust
// Enable or disable animations (default: true)
rotation_widget.set_animations_enabled(true);

// Set speed of transition in milliseconds (default: 500)
rotation_widget.set_animation_speed(500);

// Adjust overshoot/bounce-back intensity (default: 1.7)
rotation_widget.set_animation_overshoot(1.7);
```

## The Three-Phase Animation Zoom

When triggered with `set_rotation_with_animation(new_rotation)`, the widget initiates a high-fidelity visual transition with a **Three-Phase Zoom Effect**. The parameter is a `f64` representing the target angle in degrees (not `SmearorRotation`):

```rust
rotation_widget.set_rotation_with_animation(90.0);
```

1. **Phase 1 (0–33%)**: Visual zoom-out (scaling from `1.0` to `0.8`) and simultaneous rotation start, ensuring a comfortable screen clearance.
2. **Phase 2 (33–66%)**: Main rotation sweep to the target angle.
3. **Phase 3 (66–100%)**: Snapping to the target angle and zooming back in (scaling from `0.8` to `1.0`).

Available easing functions include:
- `Linear`: Constant speed throughout. Best for mechanical, predictable transitions.
- `EaseInOut`: Slow start and end, fast in the middle. Default for zoom animations. Best for natural-feeling transitions.
- `Overshoot`: Spring physics with configurable `overshoot_amount`. The widget slightly overshoots the target before settling. Best for playful, responsive feedback.

For a detailed visual explanation of the three-phase zoom and the internal animation architecture, see the [Architecture](architecture.md) page.
