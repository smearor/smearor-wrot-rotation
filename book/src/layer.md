# SmearorLayer

The `SmearorLayer` enum represents layer shell layers for `gtk4-layer-shell`. It provides a convenient, serializable wrapper around `gtk4_layer_shell::Layer` with string parsing support.

> **Feature gate**: `SmearorLayer` is only available when the `layer-shell` feature is enabled (default). To disable it, set `default-features = false` in your `Cargo.toml`.

## Variants

```rust
pub enum SmearorLayer {
    Background,
    Bottom,
    Top,
    Overlay,
}
```

- **`Background`**: The background layer, below all other windows.
- **`Bottom`**: The bottom layer, above background but below normal windows.
- **`Top`**: The top layer, above normal windows. This is the default.
- **`Overlay`**: The overlay layer, above everything including panels and notifications.

## Default

The default value is `Top`:

```rust
use smearor_wrot_rotation::SmearorLayer;

let default_layer = SmearorLayer::default();  // SmearorLayer::Top
```

## Conversion to gtk4_layer_shell::Layer

`SmearorLayer` converts into `gtk4_layer_shell::Layer` via the `From` trait:

```rust
use gtk4_layer_shell::Layer;
use smearor_wrot_rotation::SmearorLayer;

let layer: Layer = SmearorLayer::Overlay.into();
```

## String Parsing

### `From<&str>`

Accepts case-insensitive strings:

```rust
let layer = SmearorLayer::from("background");  // Background
let layer = SmearorLayer::from("BOTTOM");      // Bottom
let layer = SmearorLayer::from("Top");         // Top
let layer = SmearorLayer::from("overlay");     // Overlay
```

Invalid input falls back to `Top`.

### `FromStr`

The `FromStr` implementation returns `Result<Self, miette::Error>`:

```rust
use std::str::FromStr;
use smearor_wrot_rotation::SmearorLayer;

let layer = SmearorLayer::from_str("bottom").unwrap();  // Bottom
```

Invalid input also falls back to `Top` (returns `Ok(Top)` rather than an error).

## Serde Serialization

`SmearorLayer` derives `Serialize` and `Deserialize` with case-insensitive alias support:

```rust
// Deserialization accepts:
// "background", "Background", "BACKGROUND" -> Background
// "bottom", "Bottom", "BOTTOM"           -> Bottom
// "top", "Top", "TOP"                     -> Top
// "overlay", "Overlay", "OVERLAY"         -> Overlay
```

## Usage with SmearorRotation

`SmearorLayer` is typically used alongside `SmearorRotation` when configuring layer shell widgets:

```rust
use smearor_wrot_rotation::{SmearorLayer, SmearorRotation};
use gtk4_layer_shell::Layer;

let rotation = SmearorRotation::Deg90;
let layer: Layer = SmearorLayer::Top.into();

// Use rotation.anchor() to determine the layer shell edge
if let Some(edge) = rotation.anchor() {
    // Configure layer shell anchor based on rotation
}
```
