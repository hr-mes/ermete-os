use gtk4::prelude::*;
use gtk4::{CssProvider, gdk::Display};

pub fn load_global_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        "
        /* Glassmorphism tokens & Premium OS Standards */
        window {
            background-color: rgba(28, 28, 30, 0.85); /* macOS-like dark mode glass */
            backdrop-filter: blur(30px);
            border: 1px solid rgba(255, 255, 255, 0.15);
            border-radius: 16px;
            box-shadow: 0 12px 36px rgba(0, 0, 0, 0.4);
        }

        .sidebar-container {
            background-color: rgba(40, 40, 40, 0.6);
            border-right: 1px solid rgba(255, 255, 255, 0.1);
            padding: 8px;
        }

        .sidebar-list row {
            padding: 10px 16px;
            border-radius: 10px;
            margin: 4px 8px;
            transition: all 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
        }

        .sidebar-list row:hover {
            background-color: rgba(255, 255, 255, 0.1);
        }

        .sidebar-list row:selected {
            background-color: rgba(10, 132, 255, 0.85); /* macOS blue accent */
            color: white;
            box-shadow: 0 2px 6px rgba(10, 132, 255, 0.3);
        }

        .profile-name {
            font-size: 16pt;
            font-weight: 700;
            color: #ffffff;
            letter-spacing: -0.2px;
        }

        .profile-role {
            font-size: 11pt;
            color: rgba(255, 255, 255, 0.6);
        }

        .sidebar-search {
            border-radius: 8px;
            background-color: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            color: white;
            padding: 6px 12px;
            transition: all 0.2s ease;
        }

        .sidebar-search:focus {
            background-color: rgba(255, 255, 255, 0.1);
            border: 1px solid rgba(10, 132, 255, 0.6);
            box-shadow: 0 0 0 2px rgba(10, 132, 255, 0.2);
        }

        .stack-container {
            background-color: transparent;
            padding: 24px;
        }

        /* Typography globally for premium feel */
        label {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
        }
        "
    );

    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        
        // Dynamically load Matugen colors if available
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let mat_path = format!("{}/.config/gtk-4.0/colors.css", home);
        if std::path::Path::new(&mat_path).exists() {
            let mat_provider = CssProvider::new();
            mat_provider.load_from_path(&mat_path);
            gtk4::style_context_add_provider_for_display(
                &display,
                &mat_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_THEME,
            );
        }
    }
}
