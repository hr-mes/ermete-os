use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider,
    EventControllerKey, Label, Orientation, ScrolledWindow,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::io::Write;
use std::process::{Command, Stdio};

const CLIPBOARD_CSS: &str = r#"
.clipboard-card {
    background: radial-gradient(circle, alpha(@surface_darker, 0.9), alpha(@surface_dim, 0.9));
    border-radius: 12px;
    box-shadow: inset 1px 2px 2px rgba(255, 255, 255, 0.2), 0 4px 12px rgba(0,0,0,0.5);
    margin: 10px;
    padding: 16px;
}

.clipboard-item-btn {
    background: rgba(255, 255, 255, 0.04);
    border: none;
    border-radius: 8px;
    padding: 10px 14px;
    color: #e0def4;
}

.clipboard-item-btn:hover {
    background: rgba(156, 207, 216, 0.2);
    color: #9ccfd8;
}
"#;

pub fn show_clipboard_modal(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Clipboard History")
        .default_width(450)
        .default_height(400)
        .build();

    window.init_layer_shell();
    window.set_namespace("clipboard");
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_margin(gtk4_layer_shell::Edge::Top, 45);

    let provider = CssProvider::new();
    provider.load_from_data(CLIPBOARD_CSS);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Display default"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let key_controller = EventControllerKey::new();
    let win_weak = window.downgrade();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            if let Some(w) = win_weak.upgrade() {
                w.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    let container = GtkBox::new(Orientation::Vertical, 12);
    container.add_css_class("clipboard-card");

    let title = Label::new(Some("󰅍  Cronologia Appunti"));
    title.set_halign(Align::Start);
    container.append(&title);

    let scroll = ScrolledWindow::builder()
        .min_content_height(320)
        .build();
    let list_box = GtkBox::new(Orientation::Vertical, 6);

    // Read cliphist list
    if let Ok(output) = Command::new("cliphist").arg("list").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().take(30) {
            let line_str = line.to_string();
            let mut parts = line.splitn(2, '\t');
            if let (Some(_id), Some(content)) = (parts.next(), parts.next()) {
                let display_text = if content.len() > 60 {
                    format!("{}...", &content[..60])
                } else {
                    content.to_string()
                };

                let btn = Button::with_label(&display_text);
                btn.add_css_class("clipboard-item-btn");
                btn.set_halign(Align::Fill);

                let win_close = window.downgrade();
                let line_capture = line_str.clone();
                btn.connect_clicked(move |_| {
                    // cliphist decode | wl-copy
                    if let Ok(mut decode_proc) = Command::new("cliphist")
                        .arg("decode")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                    {
                        if let Some(mut stdin) = decode_proc.stdin.take() {
                            let _ = stdin.write_all(line_capture.as_bytes());
                        }
                        if let Ok(dec_out) = decode_proc.wait_with_output() {
                            if let Ok(mut wl_proc) = Command::new("wl-copy")
                                .stdin(Stdio::piped())
                                .spawn()
                            {
                                if let Some(mut wl_in) = wl_proc.stdin.take() {
                                    let _ = wl_in.write_all(&dec_out.stdout);
                                }
                            }
                        }
                    }

                    if let Some(w) = win_close.upgrade() {
                        w.close();
                    }
                });

                list_box.append(&btn);
            }
        }
    }

    scroll.set_child(Some(&list_box));
    container.append(&scroll);
    window.set_child(Some(&container));

    crate::ui::topbar::setup_popup_autoclose(&window, "clipboard");

    window.present();
}
