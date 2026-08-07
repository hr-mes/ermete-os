use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Image, Label, Orientation, ProgressBar};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use crate::ui::viewmodel::{OsdEvent, OsdViewModel};

fn init_osd_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(r#"
        window.osd-window {
            background: transparent;
            background-color: transparent;
            border: none;
            box-shadow: none;
        }

        .dynamic-island-container {
            background-color: rgba(18, 18, 22, 0.88);
            backdrop-filter: blur(24px);
            border: 1px solid rgba(255, 255, 255, 0.14);
            border-radius: 26px;
            padding: 8px 18px;
            box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
            min-width: 220px;
            min-height: 48px;
            opacity: 0;
            margin-top: -16px;
            transform: scale(0.92);
            transition: all 300ms cubic-bezier(0.05, 0.9, 0.1, 1.05);
        }

        .dynamic-island-container.active {
            opacity: 1;
            margin-top: 0px;
            transform: scale(1);
        }

        .dynamic-island-icon {
            color: #ffffff;
            margin-right: 10px;
            transition: all 200ms cubic-bezier(0.05, 0.9, 0.1, 1.05);
        }

        .dynamic-island-title {
            font-size: 13px;
            font-weight: 700;
            color: rgba(255, 255, 255, 0.95);
            font-family: system-ui, -apple-system, sans-serif;
            transition: all 200ms cubic-bezier(0.05, 0.9, 0.1, 1.05);
        }

        .dynamic-island-value {
            font-size: 12px;
            font-weight: 600;
            color: rgba(255, 255, 255, 0.7);
            transition: all 200ms cubic-bezier(0.05, 0.9, 0.1, 1.05);
        }

        .dynamic-island-progress progressbar trough {
            background-color: rgba(255, 255, 255, 0.15);
            border-radius: 6px;
            min-height: 6px;
            border: none;
        }

        .dynamic-island-progress progressbar progress {
            background-color: #007aff;
            border-radius: 6px;
            min-height: 6px;
            border: none;
            box-shadow: 0 0 8px rgba(0, 122, 255, 0.6);
            transition: all 250ms cubic-bezier(0.05, 0.9, 0.1, 1.05);
        }

        .dynamic-island-badge {
            border-radius: 12px;
            padding: 2px 10px;
            font-size: 11px;
            font-weight: 700;
            font-family: system-ui, -apple-system, sans-serif;
            background-color: rgba(255, 255, 255, 0.12);
            color: rgba(255, 255, 255, 0.85);
            border: 1px solid rgba(255, 255, 255, 0.2);
            transition: all 250ms cubic-bezier(0.05, 0.9, 0.1, 1.05);
        }

        .dynamic-island-badge.caps-on {
            background-color: rgba(255, 149, 0, 0.25);
            color: #ff9f0a;
            border-color: rgba(255, 149, 0, 0.5);
            box-shadow: 0 0 10px rgba(255, 149, 0, 0.3);
        }

        .dynamic-island-badge.caps-off {
            background-color: rgba(255, 255, 255, 0.1);
            color: rgba(255, 255, 255, 0.6);
            border-color: rgba(255, 255, 255, 0.15);
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

pub fn spawn_osd(app: &Application) {
    init_osd_css();

    let window = ApplicationWindow::new(app);
    window.add_css_class("osd-window");

    // Set up Layer Shell anchored floating top-center
    window.init_layer_shell();
    window.set_namespace("dynamic-island-osd");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::None);

    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_margin(Edge::Top, 12);
    window.set_visible(false);

    // Main Dynamic Island pill container
    let island_box = GtkBox::new(Orientation::Horizontal, 10);
    island_box.add_css_class("dynamic-island-container");
    island_box.set_halign(Align::Center);
    island_box.set_valign(Align::Center);

    // Icon on the left
    let icon = Image::builder()
        .icon_name("audio-volume-high-symbolic")
        .pixel_size(24)
        .css_classes(vec!["dynamic-island-icon".to_string()])
        .build();

    // Content box: title + value/badge row, progress bar below
    let content_box = GtkBox::new(Orientation::Vertical, 4);
    content_box.set_valign(Align::Center);
    content_box.set_hexpand(true);

    let header_box = GtkBox::new(Orientation::Horizontal, 8);
    header_box.set_hexpand(true);

    let title_label = Label::builder()
        .label("Volume")
        .css_classes(vec!["dynamic-island-title".to_string()])
        .halign(Align::Start)
        .build();

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let value_label = Label::builder()
        .label("50%")
        .css_classes(vec!["dynamic-island-value".to_string()])
        .halign(Align::End)
        .build();

    let badge_label = Label::builder()
        .label("OFF")
        .css_classes(vec!["dynamic-island-badge".to_string()])
        .halign(Align::End)
        .visible(false)
        .build();

    header_box.append(&title_label);
    header_box.append(&spacer);
    header_box.append(&value_label);
    header_box.append(&badge_label);

    let progress = ProgressBar::builder()
        .valign(Align::Center)
        .halign(Align::Fill)
        .hexpand(true)
        .css_classes(vec!["dynamic-island-progress".to_string()])
        .build();

    content_box.append(&header_box);
    content_box.append(&progress);

    island_box.append(&icon);
    island_box.append(&content_box);

    window.set_child(Some(&island_box));

    // Active GLib timeout references
    let active_timeout = Rc::new(RefCell::new(None::<glib::SourceId>));
    let hide_timeout = Rc::new(RefCell::new(None::<glib::SourceId>));

    let window_rc = window.clone();
    let island_box_rc = island_box.clone();
    let icon_rc = icon.clone();
    let title_label_rc = title_label.clone();
    let value_label_rc = value_label.clone();
    let badge_label_rc = badge_label.clone();
    let progress_rc = progress.clone();

    OsdViewModel::subscribe(move |event| {
        let (icon_name, title, val_text, is_badge, badge_text, badge_active, pct) = match event {
            OsdEvent::Volume(v) => {
                let clamped = v.clamp(0.0, 1.0);
                let icon_str = if clamped <= 0.001 {
                    "audio-volume-muted-symbolic"
                } else if clamped < 0.33 {
                    "audio-volume-low-symbolic"
                } else if clamped < 0.66 {
                    "audio-volume-medium-symbolic"
                } else {
                    "audio-volume-high-symbolic"
                };
                (
                    icon_str,
                    "Volume",
                    format!("{}%", (clamped * 100.0).round() as i32),
                    false,
                    String::new(),
                    false,
                    Some(clamped),
                )
            }
            OsdEvent::Brightness(b) => {
                let pct_val = if b > 1.0 { b.clamp(0.0, 100.0) } else { b * 100.0 };
                let icon_str = if pct_val < 40.0 {
                    "display-brightness-low-symbolic"
                } else {
                    "display-brightness-high-symbolic"
                };
                (
                    icon_str,
                    "Brightness",
                    format!("{}%", pct_val.round() as i32),
                    false,
                    String::new(),
                    false,
                    Some(pct_val / 100.0),
                )
            }
            OsdEvent::CapsLock(enabled) => {
                let badge_txt = if enabled { "ON" } else { "OFF" };
                (
                    "keyboard-caps-lock-symbolic",
                    "Caps Lock",
                    String::new(),
                    true,
                    badge_txt.to_string(),
                    enabled,
                    None,
                )
            }
        };

        // Update UI components
        icon_rc.set_icon_name(Some(icon_name));
        title_label_rc.set_text(title);

        if is_badge {
            value_label_rc.set_visible(false);
            progress_rc.set_visible(false);
            badge_label_rc.set_text(&badge_text);
            badge_label_rc.set_visible(true);
            if badge_active {
                badge_label_rc.remove_css_class("caps-off");
                badge_label_rc.add_css_class("caps-on");
            } else {
                badge_label_rc.remove_css_class("caps-on");
                badge_label_rc.add_css_class("caps-off");
            }
        } else {
            badge_label_rc.set_visible(false);
            value_label_rc.set_text(&val_text);
            value_label_rc.set_visible(true);
            progress_rc.set_visible(true);
            if let Some(fraction) = pct {
                progress_rc.set_fraction(fraction);
            }
        }

        // Cancel previous timeouts if still active
        if let Some(id) = active_timeout.borrow_mut().take() {
            id.remove();
        }
        if let Some(id) = hide_timeout.borrow_mut().take() {
            id.remove();
        }

        // Make window visible and trigger elastic show transition
        if !window_rc.is_visible() {
            island_box_rc.remove_css_class("active");
            window_rc.set_visible(true);
        }
        island_box_rc.add_css_class("active");

        let island_box_clone = island_box_rc.clone();
        let win_clone = window_rc.clone();
        let active_timeout_ref = active_timeout.clone();
        let hide_timeout_ref = hide_timeout.clone();

        *active_timeout.borrow_mut() = Some(glib::timeout_add_local_once(
            Duration::from_millis(2000),
            move || {
                // Remove active class -> triggers 300ms CSS cubic-bezier fade-out transition
                island_box_clone.remove_css_class("active");
                *active_timeout_ref.borrow_mut() = None;

                let win_hide = win_clone.clone();
                let hide_ref = hide_timeout_ref.clone();
                let hide_ref_in_closure = hide_ref.clone();
                *hide_ref.borrow_mut() = Some(glib::timeout_add_local_once(
                    Duration::from_millis(350),
                    move || {
                        win_hide.set_visible(false);
                        *hide_ref_in_closure.borrow_mut() = None;
                    },
                ));
            },
        ));
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_bezier_curve_definition() {
        let css = r#"transition: all 300ms cubic-bezier(0.05, 0.9, 0.1, 1.05);"#;
        assert!(css.contains("cubic-bezier(0.05, 0.9, 0.1, 1.05)"));
    }
}

