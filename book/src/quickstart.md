# Quick Start

## Installation

Add `smearor-wrot-rotation` to your `Cargo.toml`:

```toml
[dependencies]
smearor-wrot-rotation = "0.1"
gtk4 = { version = "0.11", features = ["v4_20"] }
```

## Minimal Example

The following example creates a GTK4 application window containing a `RotationWidget` that rotates a button by 90 degrees:

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

        // Create an interactive child widget
        let button = Button::with_label("Click Me!");
        button.connect_clicked(|_| println!("Button clicked! Input transform works."));

        // Wrap it inside a RotationWidget with 90° rotation
        let rotation_widget = RotationWidget::new(SmearorRotation::Deg90);
        rotation_widget.set_child(Some(&button));

        window.set_child(Some(&rotation_widget));
        window.present();
    });

    application.run()
}
```

## Running

Build and run the example:

```sh
cargo run
```

## Next Steps

- See the [Examples](examples.md) page for interactive demos with live controls.
- Read about the full [API Reference](api.md) for all available methods.
- Learn about [Animations and Snapping](animations.md) for smooth transitions.
