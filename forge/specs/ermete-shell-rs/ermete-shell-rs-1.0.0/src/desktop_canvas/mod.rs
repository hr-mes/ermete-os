pub mod physics;
pub mod stacks;
pub mod context_menu;
pub use crate::ui::snap_overlay;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Fixed, GestureClick};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

/// Builds and launches the primary Desktop Canvas surface hosting physics-driven
/// Desktop Stacks and interactive desktop widgets.
pub fn build_desktop_canvas(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desktop Canvas & Stacks")
        .css_classes(vec!["desktop-overlay"])
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Bottom);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

    let canvas = Fixed::new();

    // Attach Desktop Stacks at top-right or left canvas region
    stacks::attach_desktop_stacks_to_canvas(&canvas, 80.0, 420.0);

    // Attach right-click gesture for macOS-style desktop context menu
    let right_click = GestureClick::new();
    right_click.set_button(3); // Right mouse button
    let app_clone = app.clone();
    right_click.connect_pressed(move |_, _, x, y| {
        context_menu::show_desktop_context_menu(&app_clone, x, y);
    });
    canvas.add_controller(right_click);

    window.set_child(Some(&canvas));
    window.present();
}
