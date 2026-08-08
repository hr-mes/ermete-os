pub mod physics;
pub mod stacks;
pub use crate::ui::snap_overlay;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Fixed};
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

    window.set_child(Some(&canvas));
    window.present();
}
