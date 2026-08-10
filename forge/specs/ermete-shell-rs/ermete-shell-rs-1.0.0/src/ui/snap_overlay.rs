use gtk4::cairo;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box, Button, DrawingArea, EventControllerMotion,
    GestureClick, Grid, Label, Orientation,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

// --- ext_ermete_snap_v1 Protocol Constants & Definitions ---

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnapZone {
    None = 0,
    LeftHalf = 1,
    RightHalf = 2,
    TopHalf = 3,
    BottomHalf = 4,
    TopLeftQuadrant = 5,
    TopRightQuadrant = 6,
    BottomLeftQuadrant = 7,
    BottomRightQuadrant = 8,
    CenterStage = 9,
    CustomRegion = 10,
    LeftTwoThirds = 11,
    RightOneThird = 12,
    LeftOneThird = 13,
    RightTwoThirds = 14,
    CenterOneThird = 15,
    LeftQuarter = 16,
    CenterHalf = 17,
    RightQuarter = 18,
}

impl SnapZone {
    pub fn default_preview_bounds(&self) -> (i32, i32, u32, u32) {
        match self {
            SnapZone::None => (0, 0, 1920, 1080),
            SnapZone::LeftHalf => (0, 0, 960, 1080),
            SnapZone::RightHalf => (960, 0, 960, 1080),
            SnapZone::TopHalf => (0, 0, 1920, 540),
            SnapZone::BottomHalf => (0, 540, 1920, 540),
            SnapZone::TopLeftQuadrant => (0, 0, 960, 540),
            SnapZone::TopRightQuadrant => (960, 0, 960, 540),
            SnapZone::BottomLeftQuadrant => (0, 540, 960, 540),
            SnapZone::BottomRightQuadrant => (960, 540, 960, 540),
            SnapZone::CenterStage => (320, 180, 1280, 720),
            SnapZone::CustomRegion => (0, 0, 1920, 1080),
            SnapZone::LeftTwoThirds => (0, 0, 1280, 1080),
            SnapZone::RightOneThird => (1280, 0, 640, 1080),
            SnapZone::LeftOneThird => (0, 0, 640, 1080),
            SnapZone::RightTwoThirds => (640, 0, 1280, 1080),
            SnapZone::CenterOneThird => (640, 0, 640, 1080),
            SnapZone::LeftQuarter => (0, 0, 480, 1080),
            SnapZone::CenterHalf => (480, 0, 960, 1080),
            SnapZone::RightQuarter => (1440, 0, 480, 1080),
        }
    }
}

pub struct SnapFlag;
impl SnapFlag {
    pub const ANIMATE: u32 = 1;
    pub const AUTO_REFLOW: u32 = 2;
    pub const STICKY: u32 = 4;
}

#[derive(Debug, Clone)]
pub struct CustomRegionBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Helper struct for communicating with ext_ermete_snap_v1 compositor protocol / IPC bridge
pub struct SnapProtocolClient;

impl SnapProtocolClient {
    pub fn set_snap_zone(zone: SnapZone, flags: u32, custom: Option<CustomRegionBounds>) {
        tracing::info!(
            "ext_ermete_snap_v1: set_snap_zone requested zone={:?}, flags={:#x}, custom={:?}",
            zone,
            flags,
            custom
        );

        // Send protocol request over Wayland socket / DBus IPC bridge to compositor
        glib::MainContext::default().spawn_local(async move {
            let zone_id = zone as u32;
            let conn = zbus::Connection::session().await;
            if let Ok(connection) = conn {
                let _ = connection
                    .call_method(
                        Some("os.ermete.Compositor"),
                        "/os/ermete/Compositor/Tiling",
                        Some("os.ermete.Compositor.Tiling"),
                        "SetSnapZone",
                        &(zone_id, flags),
                    )
                    .await;
            }
        });
    }

    pub fn commit_snap() {
        tracing::info!("ext_ermete_snap_v1: commit_snap executed");
    }

    pub fn unset_snap() {
        tracing::info!("ext_ermete_snap_v1: unset_snap executed");
    }
}

// --- Live Screen Preview HUD ---

thread_local! {
    static PREVIEW_HUD: RefCell<Option<ApplicationWindow>> = const { RefCell::new(None) };
}

fn show_snap_preview(app: &Application, x: i32, y: i32, width: u32, height: u32) {
    PREVIEW_HUD.with(|hud| {
        let mut borrow = hud.borrow_mut();
        if borrow.is_none() {
            let win = ApplicationWindow::builder()
                .application(app)
                .title("Snap Preview")
                .css_classes(vec!["snap-preview-screen"])
                .build();

            win.init_layer_shell();
            win.set_layer(Layer::Overlay);
            win.set_keyboard_mode(KeyboardMode::None);

            let preview_box = Box::builder()
                .hexpand(true)
                .vexpand(true)
                .build();
            win.set_child(Some(&preview_box));

            *borrow = Some(win);
        }

        if let Some(win) = borrow.as_ref() {
            win.set_anchor(Edge::Left, true);
            win.set_anchor(Edge::Top, true);

            win.set_margin(Edge::Left, x);
            win.set_margin(Edge::Top, y);
            win.set_default_size(width as i32, height as i32);
            win.present();
        }
    });
}

fn hide_snap_preview() {
    PREVIEW_HUD.with(|hud| {
        if let Some(win) = hud.borrow_mut().take() {
            win.close();
        }
    });
}

// --- RAM State for Mock Running Apps & Snap Groups ---

#[derive(Debug, Clone)]
pub struct RunningApp {
    pub id: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon_glyph: &'static str,
    pub window_class: &'static str,
}

pub static MOCK_RUNNING_APPS: &[RunningApp] = &[
    RunningApp {
        id: "terminal",
        title: "Ermete Terminal",
        subtitle: "zsh - ermete-core-rs",
        icon_glyph: "💻",
        window_class: "org.ermete.Terminal",
    },
    RunningApp {
        id: "browser",
        title: "Firefox Web Browser",
        subtitle: "Ermete OS Architecture Docs",
        icon_glyph: "🌐",
        window_class: "org.mozilla.firefox",
    },
    RunningApp {
        id: "files",
        title: "Files Manager",
        subtitle: "/var/home/ermete/GEMINI",
        icon_glyph: "📁",
        window_class: "org.gnome.Nautilus",
    },
    RunningApp {
        id: "editor",
        title: "VS Code Editor",
        subtitle: "snap_overlay.rs - Ermete Shell",
        icon_glyph: "📝",
        window_class: "code",
    },
    RunningApp {
        id: "settings",
        title: "System Settings",
        subtitle: "Display & Window Tiling",
        icon_glyph: "⚙️",
        window_class: "org.ermete.Settings",
    },
    RunningApp {
        id: "music",
        title: "Lo-Fi Music Player",
        subtitle: "Synthwave Chill Beats - Playing",
        icon_glyph: "🎵",
        window_class: "org.ermete.Music",
    },
];

// --- Snap Layout Specifications & Visual Previews ---

#[derive(Debug, Clone)]
pub struct LayoutSlotInfo {
    pub label: &'static str,
    pub zone: SnapZone,
    pub bounds: (i32, i32, u32, u32),
    pub rel_rect: (f64, f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct LayoutTemplate {
    pub id: &'static str,
    pub title: &'static str,
    pub slots: Vec<LayoutSlotInfo>,
}

pub fn get_layout_templates() -> Vec<LayoutTemplate> {
    vec![
        LayoutTemplate {
            id: "split_50_50",
            title: "50 / 50 Split",
            slots: vec![
                LayoutSlotInfo {
                    label: "Left",
                    zone: SnapZone::LeftHalf,
                    bounds: (0, 0, 960, 1080),
                    rel_rect: (0.0, 0.0, 0.48, 1.0),
                },
                LayoutSlotInfo {
                    label: "Right",
                    zone: SnapZone::RightHalf,
                    bounds: (960, 0, 960, 1080),
                    rel_rect: (0.52, 0.0, 0.48, 1.0),
                },
            ],
        },
        LayoutTemplate {
            id: "split_67_33",
            title: "2/3 & 1/3 Split",
            slots: vec![
                LayoutSlotInfo {
                    label: "2/3 Main",
                    zone: SnapZone::LeftTwoThirds,
                    bounds: (0, 0, 1280, 1080),
                    rel_rect: (0.0, 0.0, 0.64, 1.0),
                },
                LayoutSlotInfo {
                    label: "1/3 Side",
                    zone: SnapZone::RightOneThird,
                    bounds: (1280, 0, 640, 1080),
                    rel_rect: (0.68, 0.0, 0.32, 1.0),
                },
            ],
        },
        LayoutTemplate {
            id: "three_columns",
            title: "3 Columns",
            slots: vec![
                LayoutSlotInfo {
                    label: "Left 1/3",
                    zone: SnapZone::LeftOneThird,
                    bounds: (0, 0, 640, 1080),
                    rel_rect: (0.0, 0.0, 0.31, 1.0),
                },
                LayoutSlotInfo {
                    label: "Center",
                    zone: SnapZone::CenterOneThird,
                    bounds: (640, 0, 640, 1080),
                    rel_rect: (0.345, 0.0, 0.31, 1.0),
                },
                LayoutSlotInfo {
                    label: "Right 1/3",
                    zone: SnapZone::RightOneThird,
                    bounds: (1280, 0, 640, 1080),
                    rel_rect: (0.69, 0.0, 0.31, 1.0),
                },
            ],
        },
        LayoutTemplate {
            id: "focus_column",
            title: "Focus Column",
            slots: vec![
                LayoutSlotInfo {
                    label: "1/4 Left",
                    zone: SnapZone::LeftQuarter,
                    bounds: (0, 0, 480, 1080),
                    rel_rect: (0.0, 0.0, 0.23, 1.0),
                },
                LayoutSlotInfo {
                    label: "1/2 Main",
                    zone: SnapZone::CenterHalf,
                    bounds: (480, 0, 960, 1080),
                    rel_rect: (0.26, 0.0, 0.48, 1.0),
                },
                LayoutSlotInfo {
                    label: "1/4 Right",
                    zone: SnapZone::RightQuarter,
                    bounds: (1440, 0, 480, 1080),
                    rel_rect: (0.77, 0.0, 0.23, 1.0),
                },
            ],
        },
        LayoutTemplate {
            id: "quadrants",
            title: "4 Quadrants",
            slots: vec![
                LayoutSlotInfo {
                    label: "TL",
                    zone: SnapZone::TopLeftQuadrant,
                    bounds: (0, 0, 960, 540),
                    rel_rect: (0.0, 0.0, 0.48, 0.48),
                },
                LayoutSlotInfo {
                    label: "TR",
                    zone: SnapZone::TopRightQuadrant,
                    bounds: (960, 0, 960, 540),
                    rel_rect: (0.52, 0.0, 0.48, 0.48),
                },
                LayoutSlotInfo {
                    label: "BL",
                    zone: SnapZone::BottomLeftQuadrant,
                    bounds: (0, 540, 960, 540),
                    rel_rect: (0.0, 0.52, 0.48, 0.48),
                },
                LayoutSlotInfo {
                    label: "BR",
                    zone: SnapZone::BottomRightQuadrant,
                    bounds: (960, 540, 960, 540),
                    rel_rect: (0.52, 0.52, 0.48, 0.48),
                },
            ],
        },
        LayoutTemplate {
            id: "primary_stack",
            title: "Main & Stack",
            slots: vec![
                LayoutSlotInfo {
                    label: "Main",
                    zone: SnapZone::LeftHalf,
                    bounds: (0, 0, 960, 1080),
                    rel_rect: (0.0, 0.0, 0.48, 1.0),
                },
                LayoutSlotInfo {
                    label: "Top R",
                    zone: SnapZone::TopRightQuadrant,
                    bounds: (960, 0, 960, 540),
                    rel_rect: (0.52, 0.0, 0.48, 0.48),
                },
                LayoutSlotInfo {
                    label: "Bot R",
                    zone: SnapZone::BottomRightQuadrant,
                    bounds: (960, 540, 960, 540),
                    rel_rect: (0.52, 0.52, 0.48, 0.48),
                },
            ],
        },
    ]
}

// --- Session & Snap Group Memory ---

#[derive(Debug, Clone)]
pub struct PendingAssistSession {
    pub template: LayoutTemplate,
    pub initial_slot_idx: usize,
    pub remaining_slot_indices: Vec<usize>,
    pub current_remaining_step: usize,
    pub slot_assignments: Vec<(LayoutSlotInfo, Option<RunningApp>)>,
}

#[derive(Debug, Clone)]
pub struct ActiveSnapGroup {
    pub group_id: String,
    pub layout_name: String,
    pub created_at: String,
    pub allocations: Vec<(&'static str, SnapZone, String)>,
}

thread_local! {
    static ACTIVE_SNAP_OVERLAY: RefCell<Option<ApplicationWindow>> = const { RefCell::new(None) };
    static CURRENT_ASSIST_SESSION: RefCell<Option<PendingAssistSession>> = const { RefCell::new(None) };
    static ACTIVE_SNAP_GROUPS: RefCell<Vec<ActiveSnapGroup>> = const { RefCell::new(Vec::new()) };
}

pub fn get_active_snap_groups() -> Vec<ActiveSnapGroup> {
    ACTIVE_SNAP_GROUPS.with(|groups| groups.borrow().clone())
}

pub fn clear_snap_groups() {
    ACTIVE_SNAP_GROUPS.with(|groups| groups.borrow_mut().clear());
}

fn ensure_snap_css_loaded() {
    thread_local! {
        static CSS_LOADED: RefCell<bool> = const { RefCell::new(false) };
    }
    CSS_LOADED.with(|loaded| {
        if !*loaded.borrow() {
            let provider = gtk4::CssProvider::new();
            let css = r#"
                .snap-overlay-window {
                    background-color: rgba(24, 24, 37, 0.88);
                    backdrop-filter: blur(28px);
                    border: 1px solid rgba(88, 91, 112, 0.35);
                    border-radius: 24px;
                    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.55);
                    color: #cdd6f4;
                }
                .snap-layout-card {
                    background-color: rgba(49, 50, 68, 0.45);
                    border: 1px solid rgba(88, 91, 112, 0.3);
                    border-radius: 18px;
                    padding: 12px;
                    transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
                }
                .snap-layout-card:hover {
                    background-color: rgba(49, 50, 68, 0.75);
                    border-color: rgba(137, 180, 250, 0.6);
                    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
                }
                .snap-vector-drawing {
                    border-radius: 10px;
                }
                .snap-assist-app-card {
                    background-color: rgba(49, 50, 68, 0.5);
                    border: 1px solid rgba(88, 91, 112, 0.3);
                    border-radius: 16px;
                    padding: 12px 14px;
                    transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
                }
                .snap-assist-app-card:hover {
                    background-color: rgba(137, 180, 250, 0.2);
                    border-color: #89b4fa;
                    transform: translateY(-2px);
                    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.35);
                }
                .snap-assist-badge {
                    background-color: #89b4fa;
                    color: #11111b;
                    border-radius: 10px;
                    padding: 3px 10px;
                    font-weight: 800;
                    font-size: 11px;
                }
                .snap-group-chip {
                    background-color: rgba(203, 166, 247, 0.2);
                    border: 1px solid #cba6f7;
                    color: #cba6f7;
                    border-radius: 14px;
                    padding: 4px 12px;
                    font-size: 12px;
                    font-weight: 600;
                }
                .snap-preview-screen {
                    background-color: rgba(137, 180, 250, 0.22);
                    border: 2px solid #89b4fa;
                    border-radius: 16px;
                    backdrop-filter: blur(12px);
                    box-shadow: 0 0 30px rgba(137, 180, 250, 0.3);
                }
                .snap-action-btn {
                    background-color: rgba(49, 50, 68, 0.6);
                    border: 1px solid rgba(88, 91, 112, 0.4);
                    border-radius: 14px;
                    padding: 8px 16px;
                    color: #cdd6f4;
                    font-weight: 600;
                }
                .snap-action-btn:hover {
                    background-color: rgba(137, 180, 250, 0.3);
                    border-color: #89b4fa;
                }
            "#;
            provider.load_from_data(css);
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 50,
                );
            }
            *loaded.borrow_mut() = true;
        }
    });
}

// --- Vector Cairo Drawing Engine for Visual Layout Thumbnails ---

fn detect_slot(template: &LayoutTemplate, area_w: f64, area_h: f64, mouse_x: f64, mouse_y: f64) -> Option<usize> {
    let margin = 4.0;
    let inner_w = area_w - (margin * 2.0);
    let inner_h = area_h - (margin * 2.0);

    if inner_w <= 0.0 || inner_h <= 0.0 {
        return None;
    }

    for (idx, slot) in template.slots.iter().enumerate() {
        let (rx, ry, rw, rh) = slot.rel_rect;
        let sx = margin + (rx * inner_w);
        let sy = margin + (ry * inner_h);
        let sw = rw * inner_w;
        let sh = rh * inner_h;

        if mouse_x >= sx && mouse_x <= (sx + sw) && mouse_y >= sy && mouse_y <= (sy + sh) {
            return Some(idx);
        }
    }
    None
}

fn draw_rounded_path(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = r.min(w / 2.0).min(h / 2.0);
    let degrees = std::f64::consts::PI / 180.0;

    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -90.0 * degrees, 0.0 * degrees);
    cr.arc(x + w - r, y + h - r, r, 0.0 * degrees, 90.0 * degrees);
    cr.arc(x + r, y + h - r, r, 90.0 * degrees, 180.0 * degrees);
    cr.arc(x + r, y + r, r, 180.0 * degrees, 270.0 * degrees);
    cr.close_path();
}

fn render_layout_vector_card(
    cr: &cairo::Context,
    w: f64,
    h: f64,
    template: &LayoutTemplate,
    hovered_slot: Option<usize>,
) {
    let margin = 4.0;
    let inner_w = w - (margin * 2.0);
    let inner_h = h - (margin * 2.0);

    // Outer monitor frame
    draw_rounded_path(cr, margin, margin, inner_w, inner_h, 8.0);
    cr.set_source_rgba(0.11, 0.11, 0.16, 0.75);
    let _ = cr.fill_preserve();
    cr.set_source_rgba(0.34, 0.35, 0.44, 0.4);
    cr.set_line_width(1.2);
    let _ = cr.stroke();

    for (idx, slot) in template.slots.iter().enumerate() {
        let is_hovered = hovered_slot == Some(idx);
        let (rx, ry, rw, rh) = slot.rel_rect;

        let sx = margin + (rx * inner_w) + 2.0;
        let sy = margin + (ry * inner_h) + 2.0;
        let sw = (rw * inner_w) - 4.0;
        let sh = (rh * inner_h) - 4.0;

        if sw <= 0.0 || sh <= 0.0 {
            continue;
        }

        draw_rounded_path(cr, sx, sy, sw, sh, 5.0);

        if is_hovered {
            // Glowing primary accent fill
            cr.set_source_rgba(0.53, 0.70, 0.98, 0.50);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(0.53, 0.70, 0.98, 0.95);
            cr.set_line_width(2.0);
            let _ = cr.stroke();
        } else {
            // Glass window fill
            cr.set_source_rgba(0.19, 0.20, 0.28, 0.7);
            let _ = cr.fill_preserve();
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.18);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
        }

        // Render label text inside miniature slot
        if sw > 22.0 && sh > 14.0 {
            cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(if is_hovered { 11.0 } else { 10.0 });

            if is_hovered {
                cr.set_source_rgba(0.96, 0.97, 1.0, 1.0);
            } else {
                cr.set_source_rgba(0.68, 0.71, 0.81, 0.85);
            }

            if let Ok(extents) = cr.text_extents(slot.label) {
                let tx = sx + (sw - extents.width()) / 2.0 - extents.x_bearing();
                let ty = sy + (sh - extents.height()) / 2.0 - extents.y_bearing();
                cr.move_to(tx, ty);
                let _ = cr.show_text(slot.label);
            }
        }
    }
}

// --- Layout Card Builder ---

fn build_layout_card_widget(
    app: &Application,
    popover_win: &ApplicationWindow,
    template: LayoutTemplate,
) -> Box {
    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_start(4)
        .margin_end(4)
        .margin_top(4)
        .margin_bottom(4)
        .css_classes(vec!["snap-layout-card"])
        .build();

    let title_lbl = Label::builder()
        .label(template.title)
        .halign(Align::Center)
        .css_classes(vec!["cc-label-main"])
        .build();
    card.append(&title_lbl);

    let drawing_area = DrawingArea::builder()
        .content_width(160)
        .content_height(100)
        .hexpand(true)
        .vexpand(true)
        .css_classes(vec!["snap-vector-drawing"])
        .build();

    let hovered_slot_cell = Rc::new(RefCell::new(None::<usize>));

    let template_draw = template.clone();
    let h_draw = hovered_slot_cell.clone();
    drawing_area.set_draw_func(move |_area, cr, w, h| {
        let hovered = *h_draw.borrow();
        render_layout_vector_card(cr, w as f64, h as f64, &template_draw, hovered);
    });

    let motion = EventControllerMotion::new();
    let area_motion = drawing_area.clone();
    let template_motion = template.clone();
    let app_motion = app.clone();
    let h_motion = hovered_slot_cell.clone();

    motion.connect_motion(move |_, x, y| {
        let w = area_motion.width() as f64;
        let h = area_motion.height() as f64;
        let new_slot = detect_slot(&template_motion, w, h, x, y);

        let prev = *h_motion.borrow();
        if prev != new_slot {
            *h_motion.borrow_mut() = new_slot;
            area_motion.queue_draw();

            if let Some(idx) = new_slot {
                let slot = &template_motion.slots[idx];
                let (px, py, pw, ph) = slot.bounds;
                show_snap_preview(&app_motion, px, py, pw, ph);
            } else {
                hide_snap_preview();
            }
        }
    });

    let h_leave = hovered_slot_cell.clone();
    let area_leave = drawing_area.clone();
    motion.connect_leave(move |_| {
        *h_leave.borrow_mut() = None;
        area_leave.queue_draw();
        hide_snap_preview();
    });
    drawing_area.add_controller(motion);

    let click = GestureClick::new();
    let template_click = template.clone();
    let h_click = hovered_slot_cell.clone();
    let popover_click = popover_win.clone();
    let app_click = app.clone();
    let area_click = drawing_area.clone();

    click.connect_pressed(move |_, _, x, y| {
        let w = area_click.width() as f64;
        let h = area_click.height() as f64;
        let clicked_slot = detect_slot(&template_click, w, h, x, y)
            .or_else(|| *h_click.borrow());

        if let Some(slot_idx) = clicked_slot {
            hide_snap_preview();
            let slot = &template_click.slots[slot_idx];

            SnapProtocolClient::set_snap_zone(
                slot.zone,
                SnapFlag::ANIMATE | SnapFlag::AUTO_REFLOW,
                None,
            );
            SnapProtocolClient::commit_snap();

            let remaining: Vec<usize> = (0..template_click.slots.len())
                .filter(|&i| i != slot_idx)
                .collect();

            if !remaining.is_empty() {
                start_snap_assist_session(
                    &app_click,
                    &popover_click,
                    template_click.clone(),
                    slot_idx,
                    remaining,
                );
            } else {
                popover_click.close();
            }
        }
    });
    drawing_area.add_controller(click);

    card.append(&drawing_area);
    card
}

// --- Snap Assist View & Interactive Workflow ---

fn start_snap_assist_session(
    app: &Application,
    win: &ApplicationWindow,
    template: LayoutTemplate,
    chosen_slot_idx: usize,
    remaining_indices: Vec<usize>,
) {
    let mut initial_assignments = Vec::new();
    let first_slot = template.slots[chosen_slot_idx].clone();
    initial_assignments.push((first_slot, None));

    let session = PendingAssistSession {
        template: template.clone(),
        initial_slot_idx: chosen_slot_idx,
        remaining_slot_indices: remaining_indices,
        current_remaining_step: 0,
        slot_assignments: initial_assignments,
    };

    CURRENT_ASSIST_SESSION.with(|s| {
        *s.borrow_mut() = Some(session);
    });

    render_snap_assist_view(app, win);
}

fn render_snap_assist_view(app: &Application, win: &ApplicationWindow) {
    ensure_snap_css_loaded();

    let session_opt = CURRENT_ASSIST_SESSION.with(|s| s.borrow().clone());
    let Some(session) = session_opt else {
        win.close();
        return;
    };

    if session.current_remaining_step >= session.remaining_slot_indices.len() {
        // All slots filled! Commit Snap Group
        let mut group_allocations = Vec::new();
        for (slot_info, app_opt) in &session.slot_assignments {
            let app_name = app_opt.as_ref().map(|a| a.title).unwrap_or("Active Window");
            group_allocations.push((slot_info.label, slot_info.zone, app_name.to_string()));
        }

        let completed = ActiveSnapGroup {
            group_id: format!("snap-group-{}", glib::monotonic_time()),
            layout_name: session.template.title.to_string(),
            created_at: chrono::Local::now().format("%H:%M:%S").to_string(),
            allocations: group_allocations,
        };

        ACTIVE_SNAP_GROUPS.with(|groups| groups.borrow_mut().push(completed));
        CURRENT_ASSIST_SESSION.with(|s| *s.borrow_mut() = None);
        win.close();
        return;
    }

    let target_remaining_idx = session.remaining_slot_indices[session.current_remaining_step];
    let target_slot = session.template.slots[target_remaining_idx].clone();

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(16)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .css_classes(vec!["snap-overlay-window"])
        .build();

    // Header
    let header = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let badge = Label::builder()
        .label("⚡")
        .css_classes(vec!["cc-circle-amber"])
        .build();

    let title_vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title = Label::builder()
        .label("Snap Assist")
        .css_classes(vec!["cc-label-title"])
        .halign(Align::Start)
        .build();

    let subtitle_text = format!(
        "Choose an app to fill: {} ({}/{})",
        target_slot.label,
        session.current_remaining_step + 1,
        session.remaining_slot_indices.len()
    );
    let subtitle = Label::builder()
        .label(&subtitle_text)
        .css_classes(vec!["cc-label-sub"])
        .halign(Align::Start)
        .build();

    title_vbox.append(&title);
    title_vbox.append(&subtitle);

    let skip_btn = Button::builder()
        .label("Done / Skip")
        .css_classes(vec!["snap-action-btn"])
        .build();

    let win_skip = win.clone();
    skip_btn.connect_clicked(move |_| {
        CURRENT_ASSIST_SESSION.with(|s| *s.borrow_mut() = None);
        win_skip.close();
    });

    header.append(&badge);
    header.append(&title_vbox);
    header.append(&skip_btn);
    main_box.append(&header);

    // Active Snap Group Chips Row
    let chips_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .margin_bottom(6)
        .build();

    for (slot_idx, slot_info) in session.template.slots.iter().enumerate() {
        let is_current = slot_idx == target_remaining_idx;
        let is_filled = session.slot_assignments.iter().any(|(s, _)| s.label == slot_info.label);

        let chip_text = if is_filled {
            format!("✓ {}", slot_info.label)
        } else if is_current {
            format!("👉 {}", slot_info.label)
        } else {
            format!("⏳ {}", slot_info.label)
        };

        let chip = Label::builder()
            .label(&chip_text)
            .css_classes(vec![if is_filled || is_current {
                "snap-group-chip"
            } else {
                "cc-label-sub"
            }])
            .build();
        chips_row.append(&chip);
    }
    main_box.append(&chips_row);

    // Grid of Running Apps for Snap Assist
    let grid = Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .row_homogeneous(true)
        .column_homogeneous(true)
        .build();

    for (idx, app_item) in MOCK_RUNNING_APPS.iter().enumerate() {
        let app_card = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .css_classes(vec!["snap-assist-app-card"])
            .build();

        let app_badge = Label::builder()
            .label(app_item.icon_glyph)
            .css_classes(vec!["cc-circle-blue"])
            .build();

        let app_info = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let app_title = Label::builder()
            .label(app_item.title)
            .css_classes(vec!["cc-label-main"])
            .halign(Align::Start)
            .build();

        let app_sub = Label::builder()
            .label(app_item.subtitle)
            .css_classes(vec!["cc-label-sub"])
            .halign(Align::Start)
            .build();

        app_info.append(&app_title);
        app_info.append(&app_sub);

        let snap_badge = Label::builder()
            .label("Snap")
            .css_classes(vec!["snap-assist-badge"])
            .valign(Align::Center)
            .build();

        app_card.append(&app_badge);
        app_card.append(&app_info);
        app_card.append(&snap_badge);

        // Motion hover controller for live preview
        let app_shell = app.clone();
        let target_bounds = target_slot.bounds;
        let motion = EventControllerMotion::new();
        motion.connect_enter(move |_, _, _| {
            let (px, py, pw, ph) = target_bounds;
            show_snap_preview(&app_shell, px, py, pw, ph);
        });
        motion.connect_leave(move |_| {
            hide_snap_preview();
        });
        app_card.add_controller(motion);

        // Click controller to snap selected app
        let click = GestureClick::new();
        let app_selected = app_item.clone();
        let target_slot_snap = target_slot.clone();
        let app_ctx = app.clone();
        let win_ctx = win.clone();

        click.connect_pressed(move |_, _, _, _| {
            hide_snap_preview();

            // Dispatch Snap Protocol for selected app
            SnapProtocolClient::set_snap_zone(
                target_slot_snap.zone,
                SnapFlag::ANIMATE | SnapFlag::AUTO_REFLOW,
                None,
            );
            SnapProtocolClient::commit_snap();

            // Advance Session state
            CURRENT_ASSIST_SESSION.with(|s| {
                if let Some(ref mut sess) = *s.borrow_mut() {
                    sess.slot_assignments.push((target_slot_snap.clone(), Some(app_selected.clone())));
                    sess.current_remaining_step += 1;
                }
            });

            render_snap_assist_view(&app_ctx, &win_ctx);
        });
        app_card.add_controller(click);

        let row = (idx / 2) as i32;
        let col = (idx % 2) as i32;
        grid.attach(&app_card, col, row, 1, 1);
    }

    main_box.append(&grid);
    win.set_child(Some(&main_box));
}

// --- Main Visual Snap Selector Entry Point ---

/// Displays the Visual Snap Selector overlay HUD (Windows 11 style)
pub fn show_snap_overlay(app: &Application, _parent: Option<&ApplicationWindow>) {
    ensure_snap_css_loaded();

    ACTIVE_SNAP_OVERLAY.with(|cell| {
        if let Some(old_win) = cell.borrow_mut().take() {
            old_win.close();
            return;
        }

        let win = ApplicationWindow::builder()
            .application(app)
            .title("Visual Snap Selector")
            .css_classes(vec!["snap-overlay-window"])
            .build();

        win.init_layer_shell();
        win.set_layer(Layer::Top);
        win.set_keyboard_mode(KeyboardMode::OnDemand);

        win.set_anchor(Edge::Top, true);
        win.set_margin(Edge::Top, 48);

        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .build();

        // Header
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .build();

        let header_icon = Label::builder()
            .label("📐")
            .css_classes(vec!["cc-circle-blue"])
            .build();

        let header_vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let header_title = Label::builder()
            .label("Ermete Snap Layouts & Groups")
            .css_classes(vec!["cc-label-title"])
            .halign(Align::Start)
            .build();

        let header_sub = Label::builder()
            .label("Windows 11-style Vector Grid & Smart Snap Assist")
            .css_classes(vec!["cc-label-sub"])
            .halign(Align::Start)
            .build();

        header_vbox.append(&header_title);
        header_vbox.append(&header_sub);

        header.append(&header_icon);
        header.append(&header_vbox);
        main_box.append(&header);

        // Layout Templates Grid (3 cols x 2 rows)
        let templates = get_layout_templates();
        let grid = Grid::builder()
            .row_spacing(14)
            .column_spacing(14)
            .row_homogeneous(true)
            .column_homogeneous(true)
            .build();

        for (idx, template) in templates.into_iter().enumerate() {
            let card_widget = build_layout_card_widget(app, &win, template);
            let row = (idx / 3) as i32;
            let col = (idx % 3) as i32;
            grid.attach(&card_widget, col, row, 1, 1);
        }

        main_box.append(&grid);

        win.set_child(Some(&main_box));
        win.present();

        *cell.borrow_mut() = Some(win);
    });
}

/// Attach hover trigger to window maximize button to open Visual Snap Selector
pub fn attach_maximize_hover_trigger(widget: &gtk4::Widget, app: &Application) {
    let app_clone = app.clone();
    let motion = EventControllerMotion::new();
    motion.connect_enter(move |_, _, _| {
        show_snap_overlay(&app_clone, None);
    });
    widget.add_controller(motion);
}

