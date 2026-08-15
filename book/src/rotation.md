# SmearorRotation Enum

The `SmearorRotation` enum represents standard and custom rotation angles. It supports serde serialization, string parsing, and provides helper methods for orientation detection and layer shell anchoring.

## Variants

```rust
pub enum SmearorRotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
    Deg(f32),
}
```

- **`Deg0`**: No rotation (0°). Default value.
- **`Deg90`**: Rotated 90° clockwise.
- **`Deg180`**: Rotated 180° (upside down).
- **`Deg270`**: Rotated 270° clockwise (or 90° counter-clockwise).
- **`Deg(f32)`**: Arbitrary angle in degrees. Use this for non-standard rotations like 45° or 12.5°.

## Construction

### Direct Variant Usage

```rust
use smearor_wrot_rotation::SmearorRotation;

let rotation = SmearorRotation::Deg90;
let custom = SmearorRotation::Deg(45.0);
```

### Constructor Function

The `new()` function normalizes standard angles to their corresponding enum variants:

```rust
let rotation = SmearorRotation::new(90.0);  // Returns SmearorRotation::Deg90
let rotation = SmearorRotation::new(0.0);   // Returns SmearorRotation::Deg0
let rotation = SmearorRotation::new(45.0);  // Returns SmearorRotation::Deg(45.0)
```

## Conversion

### To Degrees

```rust
let degrees: f32 = SmearorRotation::Deg90.to_degrees();  // 90.0
let degrees: f32 = SmearorRotation::Deg(45.0).to_degrees();  // 45.0
```

### From String

The `From<&str>` implementation accepts numeric strings and degree-prefixed strings (case-insensitive):

```rust
let r = SmearorRotation::from("0");       // Deg0
let r = SmearorRotation::from("90");      // Deg90
let r = SmearorRotation::from("deg180");  // Deg180
let r = SmearorRotation::from("Deg270");  // Deg270
let r = SmearorRotation::from("45");      // Deg(45.0)
```

Invalid input falls back to `Deg0`.

## Orientation Helpers

### `is_horizontal()`

Returns `true` for rotations that result in a horizontal orientation (0° or 180°):

```rust
SmearorRotation::Deg0.is_horizontal();   // true
SmearorRotation::Deg180.is_horizontal(); // true
SmearorRotation::Deg90.is_horizontal();  // false
```

### `is_vertical()`

Returns `true` for rotations that result in a vertical orientation (90° or 270°):

```rust
SmearorRotation::Deg90.is_vertical();   // true
SmearorRotation::Deg270.is_vertical();  // true
SmearorRotation::Deg0.is_vertical();    // false
```

## Layer Shell Anchor

> **Feature gate**: This method is only available when the `layer-shell` feature is enabled (default).

The `anchor()` method maps standard rotations to `gtk4_layer_shell::Edge` values, useful for positioning layer shell surfaces:

```rust
use gtk4_layer_shell::Edge;

SmearorRotation::Deg0.anchor();   // Some(Edge::Bottom)
SmearorRotation::Deg90.anchor();  // Some(Edge::Left)
SmearorRotation::Deg180.anchor(); // Some(Edge::Top)
SmearorRotation::Deg270.anchor(); // Some(Edge::Right)
SmearorRotation::Deg(45.0).anchor(); // None (no anchor for custom angles)
```

## Serde Serialization

`SmearorRotation` derives `Serialize` and `Deserialize` with `#[serde(untagged)]`. Standard variants serialize as numbers; custom angles serialize as floats. Multiple aliases are supported for deserialization:

```rust
// Serializes as: 90
let r = SmearorRotation::Deg90;

// Deserialization accepts:
// "0", "deg0", "Deg0"     -> Deg0
// "90", "deg90", "Deg90"  -> Deg90
// "180", "deg180", "Deg180" -> Deg180
// "270", "deg270", "Deg270" -> Deg270
// 45.0                     -> Deg(45.0)
```

This makes `SmearorRotation` suitable for configuration files where users can specify rotations as plain numbers or strings.
