use gtk4::prelude::*;
use gtk4::{Align, Application, ApplicationWindow, Box, EventControllerMotion, GestureClick, Label, Orientation};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;

// --- ext_ermete_snap_v1 Protocol Constants & Definitions ---

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// --- Snap Layout Visual Popover UI ---

thread_local! {
    static ACTIVE_SNAP_OVERLAY: RefCell<Option<ApplicationWindow>> = const { RefCell::new(None) };
}

/// Builds an interactive tile slot in the Snap Layout Popover
fn build_snap_tile_slot(
    app: &Application,
    label_text: &str,
    zone: SnapZone,
    preview_bounds: (i32, i32, u32, u32),
    popover_win: ApplicationWindow,
) -> Box {
    let slot = Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Fill)
        .valign(Align::Fill)
        .hexpand(true)
        .vexpand(true)
        .css_classes(vec!["snap-tile-slot"])
        .build();

    let lbl = Label::builder()
        .label(label_text)
        .halign(Align::Center)
        .valign(Align::Center)
        .css_classes(vec!["cc-label-sub"])
        .build();
    slot.append(&lbl);

    // Motion controller for live zone preview on hover
    let app_clone = app.clone();
    let motion = EventControllerMotion::new();
    motion.connect_enter(move |_, _, _| {
        let (px, py, pw, ph) = preview_bounds;
        show_snap_preview(&app_clone, px, py, pw, ph);
    });

    motion.connect_leave(move |_| {
        hide_snap_preview();
    });
    slot.add_controller(motion);

    // Click handler to commit snap request
    let click = GestureClick::new();
    let popover_close = popover_win;
    click.connect_pressed(move |_, _, _, _| {
        hide_snap_preview();
        SnapProtocolClient::set_snap_zone(
            zone,
            SnapFlag::ANIMATE | SnapFlag::AUTO_REFLOW,
            None,
        );
        SnapProtocolClient::commit_snap();
        popover_close.close();
    });
    slot.add_controller(click);

    slot
}

/// Creates the 50/50 Split template card
fn build_split_50_50_card(app: &Application, pop: ApplicationWindow) -> Box {
    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(vec!["snap-layout-card"])
        .build();

    let title = Label::builder()
        .label("50 / 50 Split")
        .css_classes(vec!["cc-label-main"])
        .halign(Align::Start)
        .build();
    card.append(&title);

    let grid = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .height_request(80)
        .width_request(160)
        .build();

    let left = build_snap_tile_slot(app, "Left", SnapZone::LeftHalf, (0, 0, 960, 1080), pop.clone());
    let right = build_snap_tile_slot(app, "Right", SnapZone::RightHalf, (960, 0, 960, 1080), pop);

    grid.append(&left);
    grid.append(&right);
    card.append(&grid);

    card
}

/// Creates the 4 Quadrants template card
fn build_quadrants_card(app: &Application, pop: ApplicationWindow) -> Box {
    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(vec!["snap-layout-card"])
        .build();

    let title = Label::builder()
        .label("4 Quadrants")
        .css_classes(vec!["cc-label-main"])
        .halign(Align::Start)
        .build();
    card.append(&title);

    let main_vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .homogeneous(true)
        .height_request(80)
        .width_request(160)
        .build();

    let top_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let bot_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let tl = build_snap_tile_slot(app, "TL", SnapZone::TopLeftQuadrant, (0, 0, 960, 540), pop.clone());
    let tr = build_snap_tile_slot(app, "TR", SnapZone::TopRightQuadrant, (960, 0, 960, 540), pop.clone());
    let bl = build_snap_tile_slot(app, "BL", SnapZone::BottomLeftQuadrant, (0, 540, 960, 540), pop.clone());
    let br = build_snap_tile_slot(app, "BR", SnapZone::BottomRightQuadrant, (960, 540, 960, 540), pop);

    top_row.append(&tl);
    top_row.append(&tr);
    bot_row.append(&bl);
    bot_row.append(&br);

    main_vbox.append(&top_row);
    main_vbox.append(&bot_row);
    card.append(&main_vbox);

    card
}

/// Creates the 3 Columns template card
fn build_three_columns_card(app: &Application, pop: ApplicationWindow) -> Box {
    let card = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .css_classes(vec!["snap-layout-card"])
        .build();

    let title = Label::builder()
        .label("3 Columns")
        .css_classes(vec!["cc-label-main"])
        .halign(Align::Start)
        .build();
    card.append(&title);

    let grid = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .height_request(80)
        .width_request(180)
        .build();

    let col1 = build_snap_tile_slot(app, "1/3", SnapZone::CustomRegion, (0, 0, 640, 1080), pop.clone());
    let col2 = build_snap_tile_slot(app, "Center", SnapZone::CenterStage, (640, 0, 640, 1080), pop.clone());
    let col3 = build_snap_tile_slot(app, "3/3", SnapZone::CustomRegion, (1280, 0, 640, 1080), pop);

    grid.append(&col1);
    grid.append(&col2);
    grid.append(&col3);
    card.append(&grid);

    card
}

/// Displays the Snap Layouts visual popover HUD
pub fn show_snap_overlay(app: &Application, _parent: Option<&ApplicationWindow>) {
    ACTIVE_SNAP_OVERLAY.with(|cell| {
        if let Some(old_win) = cell.borrow_mut().take() {
            old_win.close();
            return;
        }

        let win = ApplicationWindow::builder()
            .application(app)
            .title("Snap Layouts")
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
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(16)
            .build();

        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .build();

        let header_icon = Label::builder()
            .label("📐")
            .css_classes(vec!["cc-circle-blue"])
            .build();

        let header_title = Label::builder()
            .label("Ermete Snap Layouts")
            .css_classes(vec!["cc-label-title"])
            .build();

        header.append(&header_icon);
        header.append(&header_title);
        main_box.append(&header);

        let layouts_hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(14)
            .build();

        let card1 = build_split_50_50_card(app, win.clone());
        let card2 = build_quadrants_card(app, win.clone());
        let card3 = build_three_columns_card(app, win.clone());

        layouts_hbox.append(&card1);
        layouts_hbox.append(&card2);
        layouts_hbox.append(&card3);

        main_box.append(&layouts_hbox);

        win.set_child(Some(&main_box));
        win.present();

        *cell.borrow_mut() = Some(win);
    });
}

/// Attach hover trigger to window maximize button to open Snap Layouts visual popover
pub fn attach_maximize_hover_trigger(widget: &gtk4::Widget, app: &Application) {
    let app_clone = app.clone();
    let motion = EventControllerMotion::new();
    motion.connect_enter(move |_, _, _| {
        show_snap_overlay(&app_clone, None);
    });
    widget.add_controller(motion);
}
