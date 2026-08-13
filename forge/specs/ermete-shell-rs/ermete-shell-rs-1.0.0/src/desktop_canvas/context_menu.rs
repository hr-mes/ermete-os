use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, Orientation,
    Separator,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

fn init_context_menu_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(r#"
        window.macos-context-menu-window {
            background: transparent;
            background-color: transparent;
            border: none;
            box-shadow: none;
        }

        .macos-context-menu-card {
            background-color: rgba(24, 24, 34, 0.80);
            backdrop-filter: blur(28px) saturate(190%);
            border: 1px solid rgba(255, 255, 255, 0.16);
            border-radius: 14px;
            padding: 6px;
            box-shadow: 0 16px 44px rgba(0, 0, 0, 0.55), inset 0 1px 1px rgba(255, 255, 255, 0.2);
            min-width: 210px;
        }

        .macos-menu-item-btn {
            background: transparent;
            background-color: transparent;
            border: none;
            border-radius: 8px;
            padding: 6px 10px;
            transition: all 150ms cubic-bezier(0.16, 1, 0.3, 1);
        }

        .macos-menu-item-btn:hover {
            background-color: #007aff;
            box-shadow: 0 4px 14px rgba(0, 122, 255, 0.45);
        }

        .macos-menu-item-btn:hover label {
            color: #ffffff !important;
        }

        .macos-menu-item-btn:hover image {
            color: #ffffff !important;
        }

        .macos-menu-icon {
            color: rgba(255, 255, 255, 0.85);
            margin-right: 10px;
        }

        .macos-menu-text {
            font-size: 13px;
            font-weight: 500;
            color: rgba(255, 255, 255, 0.95);
            font-family: system-ui, -apple-system, sans-serif;
        }

        .macos-menu-shortcut {
            font-size: 11px;
            font-weight: 600;
            color: rgba(255, 255, 255, 0.5);
            font-family: system-ui, -apple-system, sans-serif;
            margin-left: 16px;
        }

        .macos-menu-separator {
            background-color: rgba(255, 255, 255, 0.12);
            min-height: 1px;
            margin: 4px 6px;
            border: none;
        }
    "#);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 100,
        );
    }
}

/// Helper to construct a single macOS-style menu item row with icon, title, optional shortcut, and action handler.
fn build_menu_item(
    icon_name: &str,
    title: &str,
    shortcut: Option<&str>,
    on_click: impl Fn() + 'static,
) -> Button {
    let btn = Button::builder()
        .css_classes(vec!["macos-menu-item-btn".to_string()])
        .build();

    let content_box = GtkBox::new(Orientation::Horizontal, 0);
    content_box.set_valign(Align::Center);

    let icon = Image::builder()
        .icon_name(icon_name)
        .pixel_size(16)
        .css_classes(vec!["macos-menu-icon".to_string()])
        .build();

    let label = Label::builder()
        .label(title)
        .css_classes(vec!["macos-menu-text".to_string()])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    content_box.append(&icon);
    content_box.append(&label);

    if let Some(sc) = shortcut {
        let sc_label = Label::builder()
            .label(sc)
            .css_classes(vec!["macos-menu-shortcut".to_string()])
            .halign(Align::End)
            .build();
        content_box.append(&sc_label);
    }

    btn.set_child(Some(&content_box));
    btn.connect_clicked(move |_| {
        on_click();
    });

    btn
}

/// Helper to build a thin menu separator
fn build_menu_separator() -> Separator {
    let sep = Separator::new(Orientation::Horizontal);
    sep.add_css_class("macos-menu-separator");
    sep
}

/// Shows the macOS-style Glassmorphism Desktop Context Menu at coordinates (x, y).
pub fn show_desktop_context_menu(app: &Application, x: f64, y: f64) {
    init_context_menu_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Desktop Context Menu")
        .css_classes(vec!["macos-context-menu-window"])
        .build();

    window.init_layer_shell();
    window.set_namespace("desktop-context-menu");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    // Position window relative to top-left at mouse right-click (x, y)
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Bottom, false);

    window.set_margin(Edge::Left, x.round() as i32);
    window.set_margin(Edge::Top, y.round() as i32);

    let menu_card = GtkBox::new(Orientation::Vertical, 2);
    menu_card.add_css_class("macos-context-menu-card");

    let win_close1 = window.clone();
    menu_card.append(&build_menu_item(
        "folder-new-symbolic",
        "New Folder",
        Some("⇧⌘N"),
        move || {
            tracing::info!("Context Menu: New Folder clicked");
            win_close1.close();
        },
    ));

    let win_close2 = window.clone();
    menu_card.append(&build_menu_item(
        "info-symbolic",
        "Get Info",
        Some("⌘I"),
        move || {
            tracing::info!("Context Menu: Get Info clicked");
            win_close2.close();
        },
    ));

    menu_card.append(&build_menu_separator());

    let win_close3 = window.clone();
    menu_card.append(&build_menu_item(
        "preferences-desktop-wallpaper-symbolic",
        "Change Wallpaper...",
        None,
        move || {
            tracing::info!("Context Menu: Change Wallpaper clicked");
            win_close3.close();
        },
    ));

    let win_close4 = window.clone();
    menu_card.append(&build_menu_item(
        "view-sort-ascending-symbolic",
        "Sort By Name",
        None,
        move || {
            tracing::info!("Context Menu: Sort By Name clicked");
            win_close4.close();
        },
    ));

    let win_close5 = window.clone();
    menu_card.append(&build_menu_item(
        "view-refresh-symbolic",
        "Clean Up Desktop",
        None,
        move || {
            tracing::info!("Context Menu: Clean Up Desktop clicked");
            win_close5.close();
        },
    ));

    menu_card.append(&build_menu_separator());

    let win_close6 = window.clone();
    let app_widgets = app.clone();
    menu_card.append(&build_menu_item(
        "office-calendar-symbolic",
        "Sidebar Widgets",
        None,
        move || {
            tracing::info!("Context Menu: Sidebar Widgets clicked");
            win_close6.close();
            crate::ui::widgets_board::toggle_widgets_board(&app_widgets);
        },
    ));

    let win_close7 = window.clone();
    menu_card.append(&build_menu_item(
        "utilities-terminal-symbolic",
        "Open in Terminal",
        Some("⌥⌘T"),
        move || {
            tracing::info!("Context Menu: Open in Terminal clicked");
            win_close7.close();
            let _ = std::process::Command::new("ptyxis").spawn();
        },
    ));

    menu_card.append(&build_menu_separator());

    let win_close8 = window.clone();
    menu_card.append(&build_menu_item(
        "preferences-desktop-display-symbolic",
        "Display Settings...",
        None,
        move || {
            tracing::info!("Context Menu: Display Settings clicked");
            win_close8.close();
        },
    ));

    let win_close9 = window.clone();
    menu_card.append(&build_menu_item(
        "emblem-system-symbolic",
        "System Settings...",
        None,
        move || {
            tracing::info!("Context Menu: System Settings clicked");
            win_close9.close();
        },
    ));

    window.set_child(Some(&menu_card));

    // Register popup autoclose so clicking anywhere outside closes menu
    crate::wayland::popup::setup_popup_autoclose(&window, "desktop-context-menu");

    window.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_instantiation() {
        if gtk4::init().is_ok() {
            let app = Application::new(Some("com.test.contextmenu"), Default::default());
            app.connect_startup(|a| {
                show_desktop_context_menu(a, 100.0, 100.0);
            });
        }
    }
}
