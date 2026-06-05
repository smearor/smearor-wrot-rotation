# smearor-wrot-rotation

[![Version](https://img.shields.io/badge/version-0.1.0-f5b700.svg)](https://github.com/smearor/smearor-wrot-rotation)
[![Rust Edition](https://img.shields.io/badge/rust-2024-00a1e4.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![GTK4](https://img.shields.io/badge/GTK4-v4__20-dc0073.svg)](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
[![License](https://img.shields.io/badge/license-MIT-04e762.svg)](LICENSE.md)

`smearor-wrot-rotation` is a high-performance, robust, and idiomatically written Rust library providing a specialized GTK4 widget (`RotationWidget`) capable of smoothly rotating any child GTK4 widget. It integrates automated, pixel-perfect bidirectional input/output coordinate transformations and dynamic window/surface input region calculation.

---

## Abstract

In modern Linux desktop and compositing environments, rotating interactive visual elements (such as application windows, custom overlays, or touch interfaces) introduces significant challenges. Rendering rotated content while maintaining correct user input routing (clicks, drags, touch gestures) and ensuring click-through accuracy for transparent, non-overlapping geometric areas are traditionally complex and performance-intensive.

`smearor-wrot-rotation` provides a modern, production-grade GTK4 container widget—`RotationWidget`—built on Rust Edition 2024. It leverages high-performance hardware-accelerated GDK/GSK transforms to rotate arbitrary GTK4 widgets. It features complete input coordinate translation and pixel-exact Cairo region mask calculation, ensuring flawless interaction, touch-rotation gestures, and window transparency click-through.

---

## Description

The `RotationWidget` (internally registered in GObject as `RotatedBox` subclassing `gtk4::Widget`) acts as a wrapper around any single GTK4 widget. Instead of forcing expensive widget re-rendering, redrawing, or utilizing heavy CSS-based hacks, the library achieves lightweight, hardware-accelerated rotation through GTK4's modular layout and rendering pipeline:

1. **Custom Layout Manager (`RotatedLayout`)**:
   - Overrides the `measure` function to compute the proper rotated bounding box of the child widget. It maps the natural and minimum size of the child using trigonometric bounds tracking ($\cos \theta$, $\sin \theta$).
   - Overrides the `allocate` function to map coordinates, performing translation to the center, applying rotation and scaling, and translating back to correctly place and scale the child.

2. **Inverse Coordinate Mapping**:
   - The `input_transform` function translates coordinates in the reverse direction of the widget rotation around the widget's center. This maps outer boundary pointer events back to localized child-relative coordinates, allowing interactive inner widgets (buttons, text boxes, etc.) to receive precise cursor positions.

3. **Pixel-Perfect Surface Input Masking**:
   - When a widget is rotated (especially at angles not multiples of 90°), the enclosing rectangular widget boundary contains blank/transparent corners.
   - The library implements `RotatedRegionCalculator` which uses a custom rasterization line-scanning algorithm to calculate a precise, non-overlapping polygonal mask (`cairo::Region`) of the actual rotated widget geometry.
   - This region is dynamically applied to the native window surface (`set_input_region`), enabling transparent pixel-exact click-through so underlying windows receive clicks in empty spaces.

4. **Multi-Phase Animation System**:
   - Utilizes custom-built timing structures (`RotationZoomAnimation` and `RotationAnimation`) to execute high-fidelity, organic snapping transitions.
   - Employs a **Three-Phase Zoom Effect**:
     - *Phase 1 (0–33%)*: Visual zoom-out (scale from `1.0` to `0.8`) and simultaneous rotation start, providing comfortable screen clearance during transition.
     - *Phase 2 (33–66%)*: Main rotation sweep.
     - *Phase 3 (66–100%)*: Rotation termination and zoom-in snap back (scale from `0.8` back to `1.0`).
   - Supports linear, ease-in-out, and overshoot physics, providing responsive, fluid feedback.

---

## Usage

Add `smearor-wrot-rotation` to your `Cargo.toml`. Below is a minimal example showing how to wrap and rotate an interactive GTK4 button:

```rust
use smearor_wrot_rotation::{RotationControlHandler, RotationWidget, SmearorRotation};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button};

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("io.smearor.wrot.example")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Minimal Rotation Example")
            .default_width(400)
            .default_height(400)
            .build();

        // 1. Create your interactive child widget
        let button = Button::with_label("Click Me!");
        button.connect_clicked(|_| println!("Button clicked! Input transform works."));

        // 2. Wrap it inside a RotationWidget with 90° rotation
        let rotation_widget = RotationWidget::new(SmearorRotation::Deg90);
        rotation_widget.set_child(Some(&button));

        // 3. Add the rotation widget to your window
        window.set_child(Some(&rotation_widget));
        window.present();
    });

    application.run()
}
```

---

## Features

- **Flexible Rotations**: Standard snapped orientations ($0^\circ$, $90^\circ$, $180^\circ$, $270^\circ$) as well as arbitrary float angles ($deg \in [0.0, 360.0]$) via the `SmearorRotation` enum.
- **Bidirectional Coordinate Mapping**: Perfect coordinate translations using `input_transform` to make sure all mouse and touch events land exactly where they should.
- **Dynamic Cairo Input Regions**: Dynamically recalculates native input regions based on layout size, scale, and angle using a fast line-scanning solver. Transparent regions are fully click-through.
- **Interactive Gestures**: Built-in support for pinch and rotation touch gestures (`gtk4::GestureRotate`) that track user fingers, apply a transient feedback offset, and snap on release once a configurable angle threshold (e.g. $45^\circ$) is met.
- **Advanced Easing and Snapping**: Beautiful, fluid motion featuring configurable animation speeds and spring-based overshoot effects (`EasingFunction::Overshoot`).
- **Rust Edition 2024 & Idiomatic Subclassing**: No abbreviations, clean encapsulation, explicit error handling via `miette`/`thiserror`, and 100% panic-free operations (no `unwrap` or `expect` statements in critical paths).
- **Accessibility & Theme Integration**: Leverages accessibility (`Accessible`) and building integrations matching GTK4 specifications.

---

## Configurability

The `RotationWidget` offers extensive customization through programmatic controls, trait implementations, and layout variables.

### Traits and Interfaces
The `RotationWidget` implements the `RotationControlHandler` trait for consistent API routing:
- `set_rotation(SmearorRotation)`: Instantly sets or adjusts the current rotation angle.
- `rotation() -> f32`: Queries the current angle in degrees.

### Control APIs
- **`set_child(Option<&Widget>)`**: Dynamically binds or detaches any GTK4 child widget.
- **`set_animations_enabled(bool)`**: Enables or disables transitions.
- **`set_animation_speed(u64)`**: Adjusts the transition speed of the rotation/scaling sweep (in milliseconds).
- **`set_animation_overshoot(f64)`**: Configures the spring intensity for overshoot snap transitions.
- **`set_rotation_with_animation(f64)`**: Programmatically triggers the advanced three-phase visual rotation-zoom snap sequence.

---

## Dependencies

The library is built on modern, lightweight, and thoroughly audited Rust crates, matching the project standards outlined in `AGENTS.md`.

- **`gtk4`** (version `0.11`, featuring `v4_20`): Core GTK4 UI framework bindings.
- **`gtk4-layer-shell`** (version `0.8.0`): Wayland Layer Shell support to place rotated desktop overlays.
- **`glib`**, **`glib-sys`**, and **`glib-unix`** (version `0.22`): Fundamental GObject binding infrastructure and UNIX event handling integration.
- **`miette`** (version `7.6`) & **`thiserror`** (version `2.0`): User-friendly diagnostic formatting and robust internal error typing.
- **`serde`** (version `1.0`) & **`serde_json`** (version `1.0`): Serialization/deserialization profiles for seamless configuration parsing.
- **`tracing`** (version `0.1`): Structured diagnostic tracing and asynchronous logging integration.
