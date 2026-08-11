use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DrawingArea, Grid, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutId {
    MacOs,
    Windows,
    Classic,
    Unity,
}

impl LayoutId {
    pub fn all() -> &'static [LayoutId] {
        &[LayoutId::MacOs, LayoutId::Windows, LayoutId::Classic, LayoutId::Unity]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LayoutId::MacOs => "macos",
            LayoutId::Windows => "windows",
            LayoutId::Classic => "classic",
            LayoutId::Unity => "unity",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "windows" => LayoutId::Windows,
            "classic" => LayoutId::Classic,
            "unity" => LayoutId::Unity,
            _ => LayoutId::MacOs,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            LayoutId::MacOs => "macOS Glass",
            LayoutId::Windows => "Windows 11 Fluent",
            LayoutId::Classic => "Classic Workstation",
            LayoutId::Unity => "Compact Unity",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            LayoutId::MacOs => "Dock di navigazione inferiore flottante, barra top-glass e sfocatura frosted.",
            LayoutId::Windows => "Taskbar inferiore con pulsanti centrati, menu start e barra notifiche Fluent.",
            LayoutId::Classic => "Doppio pannello tradizionale: top menu di sistema e bottom window list.",
            LayoutId::Unity => "Barra di stato superiore e launcher verticale sinistro per la massima area di lavoro.",
        }
    }

    pub fn badge_text(&self) -> &'static str {
        match self {
            LayoutId::MacOs => "Glass Dock & Top Bar",
            LayoutId::Windows => "Centered Fluent Taskbar",
            LayoutId::Classic => "Dual Workstation Panel",
            LayoutId::Unity => "Vertical Side Launcher",
        }
    }
}

pub struct CardWidgetRef {
    pub id: LayoutId,
    pub card_box: Box,
    pub select_btn: Button,
    pub status_badge: Label,
}

fn draw_rounded_rect(cr: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let degrees = std::f64::consts::PI / 180.0;
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -90.0 * degrees, 0.0 * degrees);
    cr.arc(x + w - r, y + h - r, r, 0.0 * degrees, 90.0 * degrees);
    cr.arc(x + r, y + h - r, r, 90.0 * degrees, 180.0 * degrees);
    cr.arc(x + r, y + r, r, 180.0 * degrees, 270.0 * degrees);
    cr.close_path();
}

fn create_topology_drawing_area(layout: LayoutId) -> DrawingArea {
    let area = DrawingArea::builder()
        .content_width(220)
        .content_height(130)
        .hexpand(true)
        .build();

    area.set_draw_func(move |_area, cr, width, height| {
        let w = width as f64;
        let h = height as f64;

        draw_rounded_rect(cr, 0.0, 0.0, w, h, 12.0);
        let _ = cr.clip();

        // Screen Background
        cr.set_source_rgb(0.11, 0.11, 0.16);
        let _ = cr.paint();

        // Soft background glowing accent graphics
        cr.set_source_rgba(0.53, 0.70, 0.98, 0.15);
        cr.arc(w * 0.7, h * 0.4, 45.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();

        cr.set_source_rgba(0.79, 0.65, 0.97, 0.12);
        cr.arc(w * 0.3, h * 0.7, 35.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();

        // Mini Window Representation
        let win_x = match layout {
            LayoutId::Unity => 24.0,
            _ => 20.0,
        };
        let win_y = match layout {
            LayoutId::MacOs | LayoutId::Classic | LayoutId::Unity => 20.0,
            LayoutId::Windows => 16.0,
        };
        let win_w = w - win_x - 20.0;
        let win_h = h - win_y - 28.0;

        draw_rounded_rect(cr, win_x, win_y, win_w, win_h, 6.0);
        cr.set_source_rgba(0.18, 0.18, 0.26, 0.85);
        let _ = cr.fill_preserve();
        cr.set_source_rgba(0.35, 0.36, 0.44, 0.5);
        cr.set_line_width(1.0);
        let _ = cr.stroke();

        draw_rounded_rect(cr, win_x, win_y, win_w, 10.0, 6.0);
        cr.set_source_rgba(0.24, 0.24, 0.34, 0.9);
        let _ = cr.fill();

        cr.set_source_rgb(0.95, 0.40, 0.40);
        cr.arc(win_x + 6.0, win_y + 5.0, 2.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
        cr.set_source_rgb(0.95, 0.80, 0.40);
        cr.arc(win_x + 12.0, win_y + 5.0, 2.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();
        cr.set_source_rgb(0.40, 0.85, 0.40);
        cr.arc(win_x + 18.0, win_y + 5.0, 2.0, 0.0, std::f64::consts::TAU);
        let _ = cr.fill();

        match layout {
            LayoutId::MacOs => {
                // Top status bar
                cr.set_source_rgba(0.20, 0.22, 0.32, 0.95);
                cr.rectangle(0.0, 0.0, w, 10.0);
                let _ = cr.fill();

                cr.set_source_rgba(0.8, 0.84, 0.95, 0.8);
                cr.rectangle(6.0, 3.0, 16.0, 4.0);
                let _ = cr.fill();
                cr.rectangle(w - 24.0, 3.0, 18.0, 4.0);
                let _ = cr.fill();

                // Floating dock
                let dock_w = 100.0;
                let dock_h = 14.0;
                let dock_x = (w - dock_w) / 2.0;
                let dock_y = h - dock_h - 4.0;
                draw_rounded_rect(cr, dock_x, dock_y, dock_w, dock_h, 7.0);
                cr.set_source_rgba(0.30, 0.32, 0.48, 0.85);
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.53, 0.70, 0.98, 0.6);
                cr.set_line_width(1.0);
                let _ = cr.stroke();

                let icons = [
                    (0.53, 0.70, 0.98),
                    (0.95, 0.54, 0.66),
                    (0.65, 0.89, 0.63),
                    (0.98, 0.70, 0.53),
                    (0.79, 0.65, 0.97),
                ];
                for (i, (r, g, b)) in icons.iter().enumerate() {
                    let cx = dock_x + 12.0 + (i as f64) * 19.0;
                    let cy = dock_y + 7.0;
                    cr.set_source_rgb(*r, *g, *b);
                    cr.arc(cx, cy, 4.0, 0.0, std::f64::consts::TAU);
                    let _ = cr.fill();
                }
            }
            LayoutId::Windows => {
                let tb_h = 14.0;
                let tb_y = h - tb_h;
                cr.set_source_rgba(0.15, 0.16, 0.24, 0.98);
                cr.rectangle(0.0, tb_y, w, tb_h);
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.30, 0.32, 0.45, 0.5);
                cr.set_line_width(1.0);
                let _ = cr.stroke();

                let center_x = w / 2.0;
                let icons = [
                    (0.34, 0.61, 0.95),
                    (0.95, 0.54, 0.66),
                    (0.65, 0.89, 0.63),
                    (0.98, 0.70, 0.53),
                ];
                let start_x = center_x - ((icons.len() as f64 * 14.0) / 2.0);
                for (i, (r, g, b)) in icons.iter().enumerate() {
                    let ix = start_x + (i as f64) * 14.0;
                    let iy = tb_y + 3.0;
                    draw_rounded_rect(cr, ix, iy, 9.0, 8.0, 2.0);
                    cr.set_source_rgb(*r, *g, *b);
                    let _ = cr.fill();
                }

                cr.set_source_rgba(0.8, 0.84, 0.95, 0.7);
                cr.rectangle(w - 28.0, tb_y + 5.0, 20.0, 4.0);
                let _ = cr.fill();
            }
            LayoutId::Classic => {
                cr.set_source_rgba(0.16, 0.17, 0.25, 0.98);
                cr.rectangle(0.0, 0.0, w, 10.0);
                let _ = cr.fill();

                cr.set_source_rgba(0.53, 0.70, 0.98, 0.9);
                cr.rectangle(6.0, 3.0, 22.0, 4.0);
                let _ = cr.fill();

                cr.set_source_rgba(0.8, 0.84, 0.95, 0.7);
                cr.rectangle(w - 30.0, 3.0, 24.0, 4.0);
                let _ = cr.fill();

                let tb_h = 10.0;
                let tb_y = h - tb_h;
                cr.set_source_rgba(0.16, 0.17, 0.25, 0.98);
                cr.rectangle(0.0, tb_y, w, tb_h);
                let _ = cr.fill();

                cr.set_source_rgba(0.30, 0.32, 0.46, 0.8);
                draw_rounded_rect(cr, 6.0, tb_y + 2.0, 35.0, 6.0, 2.0);
                let _ = cr.fill();
                draw_rounded_rect(cr, 45.0, tb_y + 2.0, 35.0, 6.0, 2.0);
                let _ = cr.fill();
            }
            LayoutId::Unity => {
                let dock_w = 16.0;
                cr.set_source_rgba(0.14, 0.15, 0.22, 0.98);
                cr.rectangle(0.0, 0.0, dock_w, h);
                let _ = cr.fill_preserve();
                cr.set_source_rgba(0.30, 0.32, 0.45, 0.5);
                cr.set_line_width(1.0);
                let _ = cr.stroke();

                let icons = [
                    (0.95, 0.40, 0.40),
                    (0.53, 0.70, 0.98),
                    (0.65, 0.89, 0.63),
                    (0.79, 0.65, 0.97),
                ];
                for (i, (r, g, b)) in icons.iter().enumerate() {
                    let ix = 3.0;
                    let iy = 6.0 + (i as f64) * 16.0;
                    draw_rounded_rect(cr, ix, iy, 10.0, 10.0, 3.0);
                    cr.set_source_rgb(*r, *g, *b);
                    let _ = cr.fill();
                }

                cr.set_source_rgba(0.20, 0.21, 0.30, 0.95);
                cr.rectangle(dock_w, 0.0, w - dock_w, 10.0);
                let _ = cr.fill();

                cr.set_source_rgba(0.8, 0.84, 0.95, 0.7);
                cr.rectangle(w - 28.0, 3.0, 22.0, 4.0);
                let _ = cr.fill();
            }
        }
    });

    area
}

pub fn build_switcher_section() -> Box {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .build();

    let section_title = Label::builder()
        .label("<b>Layout di Sistema (Zorin-style)</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    section_title.add_css_class("heading");
    section.append(&section_title);

    let grid = Grid::builder()
        .column_spacing(16)
        .row_spacing(16)
        .column_homogeneous(true)
        .build();

    let cards_store: Rc<RefCell<Vec<CardWidgetRef>>> = Rc::new(RefCell::new(Vec::new()));

    for (idx, &layout) in LayoutId::all().iter().enumerate() {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .css_classes(["liquid-surface", "liquid-surface"])
            .margin_start(4)
            .margin_end(4)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // Top Header inside card (Title + Badge)
        let header_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(Align::Center)
            .build();

        let title_label = Label::builder()
            .label(&format!("<b>{}</b>", layout.title()))
            .use_markup(true)
            .halign(Align::Start)
            .hexpand(true)
            .build();

        let badge = Label::builder()
            .label(layout.badge_text())
            .css_classes(["cc-label-sub"])
            .halign(Align::End)
            .build();

        header_box.append(&title_label);
        header_box.append(&badge);
        card.append(&header_box);

        // Drawing Area Topology
        let drawing_area = create_topology_drawing_area(layout);
        card.append(&drawing_area);

        // Subtitle / Description
        let desc_label = Label::builder()
            .label(layout.subtitle())
            .wrap(true)
            .max_width_chars(32)
            .halign(Align::Start)
            .css_classes(["cc-label-sub"])
            .build();
        card.append(&desc_label);

        // Action Button & Status Label
        let action_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .valign(Align::Center)
            .build();

        let status_badge = Label::builder()
            .label("Non attivo")
            .halign(Align::Start)
            .hexpand(true)
            .build();

        let select_btn = Button::builder()
            .label("Seleziona")
            .halign(Align::End)
            .css_classes(["cc-quick-btn"])
            .build();

        action_box.append(&status_badge);
        action_box.append(&select_btn);
        card.append(&action_box);

        let card_ref = CardWidgetRef {
            id: layout,
            card_box: card.clone(),
            select_btn: select_btn.clone(),
            status_badge: status_badge.clone(),
        };
        cards_store.borrow_mut().push(card_ref);

        // Card Click Handler via GestureClick on card_box
        let gesture = gtk4::GestureClick::new();
        let cards_store_clone = cards_store.clone();
        let target_id = layout;
        gesture.connect_pressed(move |_, _, _, _| {
            activate_layout(target_id, &cards_store_clone);
        });
        card.add_controller(gesture);

        // Select Button Click Handler
        let cards_store_clone2 = cards_store.clone();
        select_btn.connect_clicked(move |_| {
            activate_layout(target_id, &cards_store_clone2);
        });

        let col = (idx % 2) as i32;
        let row = (idx / 2) as i32;
        grid.attach(&card, col, row, 1, 1);
    }

    section.append(&grid);

    // Initial Load: check D-Bus / CRDT state asynchronously
    let cards_store_init = cards_store.clone();
    relm4::spawn_local(async move {
        let mut initial_layout = LayoutId::MacOs;
        if let Ok(conn) = crate::get_connection().await {
            if let Ok(proxy) = crate::settings_proxy::SettingsProxy::new(&conn).await {
                if let Ok(layout_str) = proxy.desktop_layout().await {
                    initial_layout = LayoutId::from_str(&layout_str);
                }
            } else if let Ok(proxy) = crate::settings_proxy::LayoutProxy::new(&conn).await {
                if let Ok(layout_str) = proxy.desktop_layout().await {
                    initial_layout = LayoutId::from_str(&layout_str);
                }
            }
        }
        activate_layout_ui_only(initial_layout, &cards_store_init);
    });

    section
}

fn activate_layout_ui_only(active_id: LayoutId, cards_store: &Rc<RefCell<Vec<CardWidgetRef>>>) {
    for card_ref in cards_store.borrow().iter() {
        if card_ref.id == active_id {
            card_ref.card_box.add_css_class("cc-btn-active");
            card_ref.select_btn.set_label("Attivo");
            card_ref.select_btn.add_css_class("suggested-action");
            card_ref.status_badge.set_markup("<b><span foreground='#89b4fa'>✓ ATTIVO</span></b>");
        } else {
            card_ref.card_box.remove_css_class("cc-btn-active");
            card_ref.select_btn.set_label("Seleziona");
            card_ref.select_btn.remove_css_class("suggested-action");
            card_ref.status_badge.set_markup("<span foreground='#a6adc8'>Non attivo</span>");
        }
    }
}

fn activate_layout(target_id: LayoutId, cards_store: &Rc<RefCell<Vec<CardWidgetRef>>>) {
    activate_layout_ui_only(target_id, cards_store);

    let layout_str = target_id.as_str().to_string();
    let l1 = layout_str.clone();
    let l2 = layout_str.clone();
    let l3 = layout_str.clone();
    let l4 = layout_str;

    relm4::spawn_local(async move {
        // 1. Zbus fittizia / reale call set_desktop_layout(layout_id) via LayoutProxy / SettingsProxy
        crate::settings_proxy::with_layout_proxy(move |proxy| async move {
            let _ = proxy.set_desktop_layout(&l1).await;
            let _ = proxy.apply_desktop_layout(&l2).await;
        }).await;

        crate::settings_proxy::with_settings_proxy(move |proxy| async move {
            let _ = proxy.set_desktop_layout(&l3).await;
        }).await;

        // 2. CRDT Sync
        crate::crdt_store::update_layout_crdt(&l4).await;
    });
}

pub fn build_page() -> Box {
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(24)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title = Label::builder()
        .label("<b>System Layout Switcher</b>")
        .use_markup(true)
        .halign(Align::Start)
        .build();
    title.add_css_class("title-1");

    let subtitle = Label::builder()
        .label("Personalizza la topologia dell'ambiente desktop Ermete OS in tempo reale.")
        .halign(Align::Start)
        .css_classes(["cc-label-sub"])
        .build();

    container.append(&title);
    container.append(&subtitle);

    let switcher_section = build_switcher_section();
    container.append(&switcher_section);

    container
}
