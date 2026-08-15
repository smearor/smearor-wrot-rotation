//! smearor-wrot-rotation nested interactive demo
//!
//! This example showcases two nested `RotationWidget` instances, demonstrating
//! that gesture rotation, coordinate translation, and animations work correctly
//! in a nested layout. The outer widget has gesture rotation enabled, while the
//! inner widget has gesture rotation disabled by default to avoid gesture conflicts.

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
    let application = Application::builder().application_id("io.smearor.wrot.rotation.nested_demo").build();

    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &Application) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("smearor-wrot-rotation - Nested Demo")
        .default_width(700)
        .default_height(750)
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
        .label("RotationWidget Nested Demo")
        .css_classes(["title-1"])
        .halign(Align::Center)
        .build();
    main_box.append(&title_label);

    // Grid for controlling outer widget properties (stays outside the rotation)
    let outer_grid = gtk4::Grid::builder().row_spacing(8).column_spacing(12).margin_bottom(12).build();

    // Control: Outer Animation Switch
    let outer_anim_label = Label::new(Some("Outer Animations:"));
    outer_anim_label.set_halign(Align::Start);
    let outer_anim_switch = Switch::builder().active(true).halign(Align::Start).build();
    outer_grid.attach(&outer_anim_label, 0, 0, 1, 1);
    outer_grid.attach(&outer_anim_switch, 1, 0, 1, 1);

    // Control: Outer Gesture Rotation Switch
    let outer_gesture_label = Label::new(Some("Outer Gesture Rotation:"));
    outer_gesture_label.set_halign(Align::Start);
    let outer_gesture_switch = Switch::builder().active(true).halign(Align::Start).build();
    outer_grid.attach(&outer_gesture_label, 0, 1, 1, 1);
    outer_grid.attach(&outer_gesture_switch, 1, 1, 1, 1);

    main_box.append(&outer_grid);

    // Quick Snap Buttons for outer widget
    let outer_button_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Center)
        .build();

    let outer_button_0 = Button::with_label("Outer 0°");
    let outer_button_90 = Button::with_label("Outer 90°");
    let outer_button_180 = Button::with_label("Outer 180°");
    let outer_button_270 = Button::with_label("Outer 270°");

    outer_button_box.append(&outer_button_0);
    outer_button_box.append(&outer_button_90);
    outer_button_box.append(&outer_button_180);
    outer_button_box.append(&outer_button_270);
    main_box.append(&outer_button_box);

    // Quick Snap Buttons for inner widget
    let inner_button_box = gtk4::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Center)
        .build();

    let inner_button_0 = Button::with_label("Inner 0°");
    let inner_button_90 = Button::with_label("Inner 90°");
    let inner_button_180 = Button::with_label("Inner 180°");
    let inner_button_270 = Button::with_label("Inner 270°");

    inner_button_box.append(&inner_button_0);
    inner_button_box.append(&inner_button_90);
    inner_button_box.append(&inner_button_180);
    inner_button_box.append(&inner_button_270);

    // Inner controls grid — placed inside the outer RotationWidget so they rotate with it
    let inner_grid = gtk4::Grid::builder().row_spacing(8).column_spacing(12).margin_bottom(8).build();

    let inner_anim_label = Label::new(Some("Inner Animations:"));
    inner_anim_label.set_halign(Align::Start);
    let inner_anim_switch = Switch::builder().active(true).halign(Align::Start).build();
    inner_grid.attach(&inner_anim_label, 0, 0, 1, 1);
    inner_grid.attach(&inner_anim_switch, 1, 0, 1, 1);

    let inner_gesture_label = Label::new(Some("Inner Gesture Rotation:"));
    inner_gesture_label.set_halign(Align::Start);
    let inner_gesture_switch = Switch::builder().active(false).halign(Align::Start).build();
    inner_grid.attach(&inner_gesture_label, 0, 1, 1, 1);
    inner_grid.attach(&inner_gesture_switch, 1, 1, 1, 1);

    // Inner angle label and manual slider (also inside the outer rotation)
    let inner_angle_label = Label::builder().label("Inner Angle: 0.00°").margin_top(4).build();

    let inner_manual_label = Label::new(Some("Inner Manual Angle:"));
    let inner_manual_scale = Scale::with_range(Orientation::Horizontal, 0.0, 360.0, 1.0);
    inner_manual_scale.set_value(0.0);
    inner_manual_scale.set_size_request(200, -1);

    // Container for all inner controls — this goes inside the outer RotationWidget
    let inner_controls_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    inner_controls_box.append(&inner_grid);
    inner_controls_box.append(&inner_button_box);
    inner_controls_box.append(&inner_angle_label);
    inner_controls_box.append(&inner_manual_label);
    inner_controls_box.append(&inner_manual_scale);

    // Create the inner RotationWidget with gesture rotation disabled by default
    let inner_rotation_widget = RotationWidget::new(SmearorRotation::Deg0)
        .with_gesture_rotation_enabled(false)
        .with_animations_enabled(true);
    inner_rotation_widget.set_animation_speed(500);
    inner_rotation_widget.set_animation_overshoot(1.7);
    inner_rotation_widget.set_hexpand(true);
    inner_rotation_widget.set_vexpand(true);
    inner_rotation_widget.set_halign(Align::Center);
    inner_rotation_widget.set_valign(Align::Center);

    // Child widget inside the inner RotationWidget
    let inner_child_frame = Frame::builder().label("Inner Rotated Content").width_request(200).height_request(200).build();

    let inner_child_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let inner_child_label = Label::new(Some("I am doubly rotated!"));

    let inner_click_button = Button::with_label("Click Me!");
    let inner_status_label = Label::new(Some("Status: Ready"));

    inner_click_button.connect_clicked(glib::clone!(
        #[weak]
        inner_status_label,
        move |_| {
            inner_status_label.set_label("Status: Clicked! Nested input transform works!");
        }
    ));

    let inner_text_entry = Entry::builder().placeholder_text("Type nested rotated text...").build();

    inner_child_box.append(&inner_child_label);
    inner_child_box.append(&inner_click_button);
    inner_child_box.append(&inner_status_label);
    inner_child_box.append(&inner_text_entry);
    inner_child_frame.set_child(Some(&inner_child_box));

    inner_rotation_widget.set_child(Some(&inner_child_frame));

    // Add the inner RotationWidget to the inner controls box
    inner_controls_box.append(&inner_rotation_widget);

    // Wrap inner controls in a frame so the outer rotation content is visually framed
    let outer_child_frame = Frame::builder()
        .label("Outer Rotated Content (contains inner controls)")
        .width_request(400)
        .height_request(400)
        .build();

    let outer_child_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    outer_child_box.append(&inner_controls_box);
    outer_child_frame.set_child(Some(&outer_child_box));

    // Create the outer RotationWidget with gesture rotation enabled
    let outer_rotation_widget = RotationWidget::new(SmearorRotation::Deg0)
        .with_gesture_rotation_enabled(true)
        .with_animations_enabled(true);
    outer_rotation_widget.set_animation_speed(500);
    outer_rotation_widget.set_animation_overshoot(1.7);
    outer_rotation_widget.set_hexpand(true);
    outer_rotation_widget.set_vexpand(true);
    outer_rotation_widget.set_halign(Align::Center);
    outer_rotation_widget.set_valign(Align::Center);

    // Place the outer child frame (with inner controls + inner RotationWidget) inside the outer RotationWidget
    outer_rotation_widget.set_child(Some(&outer_child_frame));

    // Frame for the outer rotated widget to visualize bounds
    let viewport_frame = Frame::builder()
        .label("Outer Rotation Viewport Bounds")
        .hexpand(true)
        .vexpand(true)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    viewport_frame.set_child(Some(&outer_rotation_widget));
    main_box.append(&viewport_frame);

    // Outer status label and manual slider (outside the rotation)
    let outer_angle_label = Label::builder().label("Outer Angle: 0.00°").margin_bottom(6).build();
    main_box.append(&outer_angle_label);

    let outer_manual_label = Label::new(Some("Outer Manual Angle:"));
    let outer_manual_scale = Scale::with_range(Orientation::Horizontal, 0.0, 360.0, 1.0);
    outer_manual_scale.set_value(0.0);
    main_box.append(&outer_manual_label);
    main_box.append(&outer_manual_scale);

    // Connect outer control switches
    outer_anim_switch.connect_active_notify(glib::clone!(
        #[weak]
        outer_rotation_widget,
        move |switch| {
            outer_rotation_widget.set_animations_enabled(switch.is_active());
        }
    ));

    outer_gesture_switch.connect_active_notify(glib::clone!(
        #[weak]
        outer_rotation_widget,
        move |switch| {
            outer_rotation_widget.set_gesture_rotation_enabled(switch.is_active());
        }
    ));

    // Connect inner control switches
    inner_anim_switch.connect_active_notify(glib::clone!(
        #[weak]
        inner_rotation_widget,
        move |switch| {
            inner_rotation_widget.set_animations_enabled(switch.is_active());
        }
    ));

    inner_gesture_switch.connect_active_notify(glib::clone!(
        #[weak]
        inner_rotation_widget,
        move |switch| {
            inner_rotation_widget.set_gesture_rotation_enabled(switch.is_active());
        }
    ));

    // Outer quick snap buttons
    outer_button_0.connect_clicked(glib::clone!(
        #[weak]
        outer_rotation_widget,
        #[weak]
        outer_manual_scale,
        #[weak]
        outer_angle_label,
        move |_| {
            outer_rotation_widget.set_rotation_with_animation(0.0);
            outer_manual_scale.set_value(0.0);
            outer_angle_label.set_label("Outer Angle: 0.00°");
        }
    ));

    outer_button_90.connect_clicked(glib::clone!(
        #[weak]
        outer_rotation_widget,
        #[weak]
        outer_manual_scale,
        #[weak]
        outer_angle_label,
        move |_| {
            outer_rotation_widget.set_rotation_with_animation(90.0);
            outer_manual_scale.set_value(90.0);
            outer_angle_label.set_label("Outer Angle: 90.00°");
        }
    ));

    outer_button_180.connect_clicked(glib::clone!(
        #[weak]
        outer_rotation_widget,
        #[weak]
        outer_manual_scale,
        #[weak]
        outer_angle_label,
        move |_| {
            outer_rotation_widget.set_rotation_with_animation(180.0);
            outer_manual_scale.set_value(180.0);
            outer_angle_label.set_label("Outer Angle: 180.00°");
        }
    ));

    outer_button_270.connect_clicked(glib::clone!(
        #[weak]
        outer_rotation_widget,
        #[weak]
        outer_manual_scale,
        #[weak]
        outer_angle_label,
        move |_| {
            outer_rotation_widget.set_rotation_with_animation(270.0);
            outer_manual_scale.set_value(270.0);
            outer_angle_label.set_label("Outer Angle: 270.00°");
        }
    ));

    // Inner quick snap buttons
    inner_button_0.connect_clicked(glib::clone!(
        #[weak]
        inner_rotation_widget,
        #[weak]
        inner_manual_scale,
        #[weak]
        inner_angle_label,
        move |_| {
            inner_rotation_widget.set_rotation_with_animation(0.0);
            inner_manual_scale.set_value(0.0);
            inner_angle_label.set_label("Inner Angle: 0.00°");
        }
    ));

    inner_button_90.connect_clicked(glib::clone!(
        #[weak]
        inner_rotation_widget,
        #[weak]
        inner_manual_scale,
        #[weak]
        inner_angle_label,
        move |_| {
            inner_rotation_widget.set_rotation_with_animation(90.0);
            inner_manual_scale.set_value(90.0);
            inner_angle_label.set_label("Inner Angle: 90.00°");
        }
    ));

    inner_button_180.connect_clicked(glib::clone!(
        #[weak]
        inner_rotation_widget,
        #[weak]
        inner_manual_scale,
        #[weak]
        inner_angle_label,
        move |_| {
            inner_rotation_widget.set_rotation_with_animation(180.0);
            inner_manual_scale.set_value(180.0);
            inner_angle_label.set_label("Inner Angle: 180.00°");
        }
    ));

    inner_button_270.connect_clicked(glib::clone!(
        #[weak]
        inner_rotation_widget,
        #[weak]
        inner_manual_scale,
        #[weak]
        inner_angle_label,
        move |_| {
            inner_rotation_widget.set_rotation_with_animation(270.0);
            inner_manual_scale.set_value(270.0);
            inner_angle_label.set_label("Inner Angle: 270.00°");
        }
    ));

    // Manual scale connections
    outer_manual_scale.connect_value_changed(glib::clone!(
        #[weak]
        outer_rotation_widget,
        #[weak]
        outer_angle_label,
        move |scale| {
            let angle = scale.value();
            outer_rotation_widget.set_rotation(SmearorRotation::Deg(angle as f32));
            outer_angle_label.set_label(&format!("Outer Angle: {:.2}°", angle));
        }
    ));

    inner_manual_scale.connect_value_changed(glib::clone!(
        #[weak]
        inner_rotation_widget,
        #[weak]
        inner_angle_label,
        move |scale| {
            let angle = scale.value();
            inner_rotation_widget.set_rotation(SmearorRotation::Deg(angle as f32));
            inner_angle_label.set_label(&format!("Inner Angle: {:.2}°", angle));
        }
    ));

    window.set_child(Some(&main_box));
    window.present();
}
