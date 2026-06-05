//! smearor-wrot-rotation interactive demo
//!
//! This example showcases the `RotationWidget` layout, coordinates translation,
//! and snappy animations inside a modern GTK4 application.

use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Button;
use gtk4::Entry;
use gtk4::Frame;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Scale;
use gtk4::Switch;
use gtk4::prelude::*;
use smearor_wrot_rotation::RotationControlHandler;
use smearor_wrot_rotation::RotationWidget;
use smearor_wrot_rotation::SmearorRotation;

fn main() -> glib::ExitCode {
    let application = Application::builder().application_id("io.smearor.wrot.rotation.demo").build();

    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &Application) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("smearor-wrot-rotation - Interactive Demo")
        .default_width(600)
        .default_height(650)
        .build();

    // Main vertical layout container
    let main_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Title label
    let title_label = Label::builder()
        .label("RotationWidget Interactive Demo")
        .css_classes(["title-1"])
        .halign(Align::Center)
        .build();
    main_box.append(&title_label);

    // Grid for controlling properties
    let grid = gtk4::Grid::builder().row_spacing(8).column_spacing(12).margin_bottom(12).build();

    // Control: Animation Switch
    let switch_label = Label::new(Some("Enable Animations:"));
    switch_label.set_halign(Align::Start);
    let animation_switch = Switch::builder().active(true).halign(Align::Start).build();
    grid.attach(&switch_label, 0, 0, 1, 1);
    grid.attach(&animation_switch, 1, 0, 1, 1);

    // Control: Speed Slider
    let speed_label = Label::new(Some("Animation Speed (ms):"));
    speed_label.set_halign(Align::Start);
    let speed_scale = Scale::with_range(Orientation::Horizontal, 100.0, 3000.0, 50.0);
    speed_scale.set_value(500.0);
    speed_scale.set_hexpand(true);
    grid.attach(&speed_label, 0, 1, 1, 1);
    grid.attach(&speed_scale, 1, 1, 1, 1);

    // Control: Overshoot Slider
    let overshoot_label = Label::new(Some("Overshoot Amount:"));
    overshoot_label.set_halign(Align::Start);
    let overshoot_scale = Scale::with_range(Orientation::Horizontal, 0.1, 5.0, 0.1);
    overshoot_scale.set_value(1.7);
    overshoot_scale.set_hexpand(true);
    grid.attach(&overshoot_label, 0, 2, 1, 1);
    grid.attach(&overshoot_scale, 1, 2, 1, 1);

    main_box.append(&grid);

    // Quick Snap Buttons
    let button_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Center)
        .build();

    let button_0 = Button::with_label("Snap to 0°");
    let button_90 = Button::with_label("Snap to 90°");
    let button_180 = Button::with_label("Snap to 180°");
    let button_270 = Button::with_label("Snap to 270°");

    button_box.append(&button_0);
    button_box.append(&button_90);
    button_box.append(&button_180);
    button_box.append(&button_270);
    main_box.append(&button_box);

    // Create the RotationWidget initially at 0 degrees
    let rotation_widget = RotationWidget::new(SmearorRotation::Deg0);
    rotation_widget.set_animations_enabled(true);
    rotation_widget.set_animation_speed(500);
    rotation_widget.set_animation_overshoot(1.7);

    // Ensure it expands to fill remaining space nicely
    rotation_widget.set_hexpand(true);
    rotation_widget.set_vexpand(true);
    rotation_widget.set_halign(Align::Center);
    rotation_widget.set_valign(Align::Center);

    // Child widget inside the RotationWidget (an interactive card)
    let child_frame = Frame::builder().label("Rotated Content").width_request(240).height_request(240).build();

    let child_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let child_label = Label::new(Some("I am rotated!"));

    // Add interactive elements inside the rotated child to test coordinates mapping
    let click_me_button = Button::with_label("Click Me!");
    let status_label = Label::new(Some("Status: Ready"));

    // Connection for interactive testing
    click_me_button.connect_clicked(glib::clone!(
        #[weak]
        status_label,
        move |_| {
            status_label.set_label("Status: Button Clicked! Input transform works!");
        }
    ));

    let text_entry = Entry::builder().placeholder_text("Type rotated text...").build();

    child_box.append(&child_label);
    child_box.append(&click_me_button);
    child_box.append(&status_label);
    child_box.append(&text_entry);
    child_frame.set_child(Some(&child_box));

    // Place child inside the rotation container
    rotation_widget.set_child(Some(&child_frame));

    // Frame for the rotated widget to clearly visualize the bounds
    let viewport_frame = Frame::builder()
        .label("Rotation Viewport Bounds")
        .hexpand(true)
        .vexpand(true)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    viewport_frame.set_child(Some(&rotation_widget));
    main_box.append(&viewport_frame);

    // Status / Degree display at the bottom
    let current_angle_label = Label::builder().label("Current Angle: 0.00°").margin_bottom(6).build();
    main_box.append(&current_angle_label);

    // Custom slider for manual arbitrary rotation
    let manual_rotation_label = Label::new(Some("Manual Angle (Arbitrary Rotation):"));
    let manual_scale = Scale::with_range(Orientation::Horizontal, 0.0, 360.0, 1.0);
    manual_scale.set_value(0.0);
    main_box.append(&manual_rotation_label);
    main_box.append(&manual_scale);

    // Connect control switches/sliders to RotationWidget
    animation_switch.connect_active_notify(glib::clone!(
        #[weak]
        rotation_widget,
        move |switch| {
            rotation_widget.set_animations_enabled(switch.is_active());
        }
    ));

    speed_scale.connect_value_changed(glib::clone!(
        #[weak]
        rotation_widget,
        move |scale| {
            let speed_ms = scale.value() as u64;
            rotation_widget.set_animation_speed(speed_ms);
        }
    ));

    overshoot_scale.connect_value_changed(glib::clone!(
        #[weak]
        rotation_widget,
        move |scale| {
            let overshoot = scale.value();
            rotation_widget.set_animation_overshoot(overshoot);
        }
    ));

    // Quick snap transitions with three-phase animations
    button_0.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        move |_| {
            rotation_widget.set_rotation_with_animation(0.0);
            manual_scale.set_value(0.0);
            current_angle_label.set_label("Current Angle: 0.00°");
        }
    ));

    button_90.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        move |_| {
            rotation_widget.set_rotation_with_animation(90.0);
            manual_scale.set_value(90.0);
            current_angle_label.set_label("Current Angle: 90.00°");
        }
    ));

    button_180.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        move |_| {
            rotation_widget.set_rotation_with_animation(180.0);
            manual_scale.set_value(180.0);
            current_angle_label.set_label("Current Angle: 180.00°");
        }
    ));

    button_270.connect_clicked(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        manual_scale,
        #[weak]
        current_angle_label,
        move |_| {
            rotation_widget.set_rotation_with_animation(270.0);
            manual_scale.set_value(270.0);
            current_angle_label.set_label("Current Angle: 270.00°");
        }
    ));

    // Dragging manual scale changes the angle in real-time
    manual_scale.connect_value_changed(glib::clone!(
        #[weak]
        rotation_widget,
        #[weak]
        current_angle_label,
        move |scale| {
            let angle = scale.value();
            rotation_widget.set_rotation(SmearorRotation::Deg(angle as f32));
            current_angle_label.set_label(&format!("Current Angle: {:.2}°", angle));
        }
    ));

    window.set_child(Some(&main_box));
    window.present();
}
