use crate::ui::popup_manager::setup_popup_autoclose;
use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, ProgressBar};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

pub fn show_system_monitor_modal(app: &Application) {
    let pop = ApplicationWindow::builder()
        .application(app)
        .title("Monitor Risorse")
        .css_classes(["popup-window"])
        .default_width(360)
        .build();

    pop.init_layer_shell();
    pop.set_layer(Layer::Overlay);
    setup_popup_autoclose(&pop, "sys-monitor");
    pop.set_anchor(Edge::Top, true);
    pop.set_anchor(Edge::Right, true);
    pop.set_margin(Edge::Top, 34);
    pop.set_margin(Edge::Right, 50);

    let card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .css_classes(["cc-card"])
        .build();

    let header = Label::builder()
        .label("MONITOR DI SISTEMA — ERMETE OS")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    // CPU Metric Card (Passive initial UI)
    let cpu_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["metric-card"])
        .build();
    let cpu_top = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let cpu_val_lbl = Label::builder()
        .label("0%")
        .css_classes(["metric-value"])
        .halign(Align::Start)
        .build();
    let cpu_desc = Label::builder()
        .label("Processore\nCarico: In attesa...")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .build();
    cpu_top.append(&cpu_val_lbl);
    cpu_top.append(&cpu_desc);
    let cpu_bar = ProgressBar::builder()
        .fraction(0.0)
        .css_classes(["cc-progress-blue"])
        .build();
    cpu_card.append(&cpu_top);
    cpu_card.append(&cpu_bar);

    // RAM Metric Card (Passive initial UI)
    let ram_card = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .css_classes(["metric-card"])
        .build();
    let ram_top = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    let ram_val_lbl = Label::builder()
        .label("0%")
        .css_classes(["metric-value"])
        .halign(Align::Start)
        .build();
    let ram_desc = Label::builder()
        .label("Memoria RAM\nIn attesa...")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .hexpand(true)
        .build();
    ram_top.append(&ram_val_lbl);
    ram_top.append(&ram_desc);
    let ram_bar = ProgressBar::builder()
        .fraction(0.0)
        .css_classes(["cc-progress-indigo"])
        .build();
    ram_card.append(&ram_top);
    ram_card.append(&ram_bar);

    let sys_info = Label::builder()
        .label("Wayland / Niri Compositor — Forgia Atomica RPM")
        .css_classes(["cc-label-sub"])
        .halign(Align::Start)
        .build();

    let close_btn = Button::builder()
        .label("Chiudi")
        .css_classes(["cc-quick-btn"])
        .build();
    let pop_clone = pop.clone();
    close_btn.connect_clicked(move |_| {
        pop_clone.close();
    });

    card.append(&header);
    card.append(&cpu_card);
    card.append(&ram_card);
    card.append(&sys_info);
    card.append(&close_btn);

    pop.set_child(Some(&card));
    pop.present();
}
