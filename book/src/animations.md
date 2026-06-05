# Animations and Snapping

`RotationWidget` features smooth, timing-based animations for orientation snapping transitions.

## Configurable Animations

You can customize rotation transition parameters:

```rust
// Enable or disable animations
rotation_widget.set_animations_enabled(true);

// Set speed of transition in milliseconds
rotation_widget.set_animation_speed(500);

// Adjust overshoot/bounce-back intensity
rotation_widget.set_animation_overshoot(1.7);
```

## The Three-Phase Animation Zoom

When triggered with `set_rotation_with_animation(new_rotation)`, the widget initiates a high-fidelity visual transition with a **Three-Phase Zoom Effect**:

1. **Phase 1 (0–33%)**: Visual zoom-out (scaling from `1.0` to `0.8`) and simultaneous rotation start, ensuring a comfortable screen clearance.
2. **Phase 2 (33–66%)**: Main rotation sweep to the target angle.
3. **Phase 3 (66–100%)**: Snapping to the target angle and zooming back in (scaling from `0.8` to `1.0`).

Available easing functions include:
- `Linear`
- `EaseInOut`
- `Overshoot` (utilizing a spring physics equation)
