use gtk4::prelude::*;
use gtk4::{CssProvider, gdk::Display};

pub fn load_global_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        "
        /* Glassmorphism tokens */
        window {
            background-color: rgba(30, 30, 30, 0.85);
            backdrop-filter: blur(20px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
        }
        "
    );

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
