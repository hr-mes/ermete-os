use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button,
    EventControllerKey, Label, Orientation, ScrolledWindow,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use std::process::Stdio;



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

    let list_box_clone = list_box.clone();
    let window_clone = window.clone();
    glib::MainContext::default().spawn_local(async move {
        if let Ok(output) = tokio::process::Command::new("cliphist").arg("list").output().await {
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

                    let line_capture = line_str.clone();
                    btn.connect_clicked(glib::clone!(@weak window_clone => move |_| {
                        let line_cap = line_capture.clone();
                        let win = window_clone.clone();
                        glib::MainContext::default().spawn_local(async move {
                            if let Ok(mut decode_proc) = tokio::process::Command::new("cliphist")
                                .arg("decode")
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                if let Some(mut stdin) = decode_proc.stdin.take() {
                                    use tokio::io::AsyncWriteExt;
                                    let _ = stdin.write_all(line_cap.as_bytes()).await;
                                }
                                if let Ok(dec_out) = decode_proc.wait_with_output().await {
                                    let text = String::from_utf8_lossy(&dec_out.stdout).to_string();
                                    if let Some(display) = gtk4::gdk::Display::default() {
                                        display.clipboard().set_text(&text);
                                    }
                                }
                            }
                            win.close();
                        });
                    }));

                    list_box_clone.append(&btn);
                }
            }
        }
    });

    scroll.set_child(Some(&list_box));
    container.append(&scroll);
    window.set_child(Some(&container));

    crate::ui::popup_manager::setup_popup_autoclose(&window, "clipboard");

    window.present();
}

