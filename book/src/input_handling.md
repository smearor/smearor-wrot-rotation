# Layout and Input Transformation

## Layout Manager

`RotationWidget` delegates its measurement and layout to a custom `RotatedLayout` manager.

- **`measure`**: Computes the size of the rotated bounding box containing the rotated child widget.
- **`allocate`**: Translates to the center, applies the GSK transformation matrix (rotate, scale), and translates back to draw the child on the screen.

## Coordinate Transformation

Because the child is physically rotated on the screen, GDK's native event location checks would normally deliver clicks to incorrect parts of the child widget.

`RotationWidget` implements a reverse coordinate mapping function:

```rust
pub fn input_transform(&self, x: f64, y: f64) -> (f64, f64)
```

This function maps screen-relative coordinates back to child-relative coordinates, ensuring full button clickability, slider adjustments, and focus changes work flawlessly inside the rotated box.

## Precise Input Region Masks

Transparent parts of the widget's bounding box are made click-through via:

```rust
pub trait RotatedRegionCalculator
```

This rasterization line-scanning routine calculates a pixel-precise polygon region representing the actual rotated visual frame. This region is then supplied to the window's GDK surface using `set_input_region()`.
