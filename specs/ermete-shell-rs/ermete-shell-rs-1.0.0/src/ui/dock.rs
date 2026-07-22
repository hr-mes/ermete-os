use crate::core::dock_config::{add_pin, load_dock_config, remove_pin, DockConfig};
use crate::core::dock_data::{reconcile_dock_items, DockItem, NiriWindowInfo, NiriWorkspaceInfo};
use crate::core::dock_watcher::{fetch_current_niri_windows, fetch_current_workspaces, spawn_dock_watchers};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, EventControllerMotion,
    EventControllerScroll, EventControllerScrollFlags, GestureClick, Image, Orientation, Popover,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

const DOCK_CSS: &str = r#"
window.dock-window {
    background-color: transparent;
}

.dock-container {
    background: alpha(#1e1e20, 0.85);
    border: 1px solid alpha(white, 0.15);
    border-radius: 24px;
    padding: 6px 12px;
    box-shadow: 0px 16px 48px alpha(black, 0.60);
    transition: all 0.25s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.dock-hidden {
    transform: translateY(120px);
    opacity: 0;
}

.dock-item-btn {
    background: transparent;
    border: none;
    border-radius: 16px;
    padding: 6px 8px;
    transition: all 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.dock-item-btn:hover {
    background: alpha(white, 0.12);
    transform: scale(1.12) translateY(-4px);
}

.dock-item-btn:active {
    transform: scale(0.96);
}

.dock-indicator {
    min-height: 4px;
    min-width: 4px;
    border-radius: 99px;
    background-color: alpha(white, 0.5);
    margin-top: 2px;
}

.dock-indicator-focused {
    min-height: 4px;
    min-width: 16px;
    border-radius: 99px;
    background-color: @shell_primary;
    margin-top: 2px;
}

.dock-popover {
    background: alpha(#1a1b26, 0.95);
    border: 1px solid alpha(white, 0.12);
    border-radius: 12px;
    padding: 6px;
}

.dock-popover-btn {
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    color: #ffffff;
}

.dock-popover-btn:hover {
    background: alpha(white, 0.1);
}

.dock-trigger-area {
    background-color: alpha(black, 0.01);
    min-height: 6px;
}

.dock-instance-badge {
    background-color: @shell_primary;
    color: #ffffff;
    font-size: 11px;
    font-weight: bold;
    border-radius: 99px;
    padding: 0 5px;
    min-width: 16px;
    min-height: 16px;
    box-shadow: 0px 2px 4px alpha(black, 0.4);
}
"#;

struct DockState {
    pinned: Vec<String>,
    windows: Vec<NiriWindowInfo>,
    workspaces: Vec<NiriWorkspaceInfo>,
    is_hovered: bool,
}

struct DockMonitorInstance {
    monitor_connector: String,
    screen_height: i32,
    window: glib::WeakRef<ApplicationWindow>,
    container: GtkBox,
    trigger_win: glib::WeakRef<ApplicationWindow>,
    state: Rc<RefCell<DockState>>,
    spring: Rc<crate::core::spring::SpringAnimator>,
}

thread_local! {
    static DOCK_INSTANCES: RefCell<Vec<DockMonitorInstance>> = RefCell::new(Vec::new());
}

fn animate_dock_visibility(container: &GtkBox, _spring: &crate::core::spring::SpringAnimator, hide: bool) {
    if hide {
        if !container.has_css_class("dock-hidden") {
            container.add_css_class("dock-hidden");
        }
    } else {
        if container.has_css_class("dock-hidden") {
            container.remove_css_class("dock-hidden");
        }
    }
}

fn should_autohide_for_monitor(state: &DockState, monitor_connector: &str, screen_height: i32) -> bool {
    let target_ws_id = match state.workspaces.iter().find_map(|ws| {
        if ws.output.as_deref() == Some(monitor_connector) && (ws.is_active || ws.is_focused) {
            Some(ws.id)
        } else {
            None
        }
    }) {
        Some(id) => id,
        None => {
            match state.workspaces.iter().find(|ws| ws.is_focused || ws.is_active) {
                Some(ws) => ws.id,
                None => return false,
            }
        }
    };

    let overlap_threshold = (screen_height as f64) - 85.0;

    state.windows.iter().any(|w| {
        if w.workspace_id != Some(target_ws_id) {
            return false;
        }
        if let Some(layout) = &w.layout {
            let y = layout.tile_pos_in_workspace_view.map(|p| p.1).unwrap_or(0.0);
            let h = layout.window_size.map(|s| s.1 as f64).unwrap_or(0.0);
            if (y + h) >= overlap_threshold {
                return true;
            }
        }
        w.is_focused
    })
}

#[allow(dead_code)]
pub fn build_ui(app: &Application) -> ApplicationWindow {
    let display = gtk4::gdk::Display::default().expect("Display default");
    let provider = CssProvider::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let colors_path = format!("{}/.config/ermete-shell/colors.css", home);
    let colors_css = std::fs::read_to_string(&colors_path).unwrap_or_else(|_| "".to_string());
    let fallback = r#"
        @define-color shell_bg alpha(#1e1e20, 0.85);
        @define-color shell_fg #f8fafc;
        @define-color shell_primary #0a84ff;
        @define-color shell_border alpha(white, 0.1);
    "#;
    let full_css = format!("{}\n{}\n{}", colors_css, fallback, DOCK_CSS);
    provider.load_from_data(&full_css);
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let (tx_win, rx_win) = glib::MainContext::channel::<Vec<NiriWindowInfo>>(glib::Priority::DEFAULT);
    let (tx_cfg, rx_cfg) = glib::MainContext::channel::<DockConfig>(glib::Priority::DEFAULT);
    let (tx_ws, rx_ws) = glib::MainContext::channel::<Vec<NiriWorkspaceInfo>>(glib::Priority::DEFAULT);

    spawn_dock_watchers(tx_win, tx_cfg, tx_ws);

    let initial_config = load_dock_config();
    let initial_windows = fetch_current_niri_windows();
    let initial_workspaces = fetch_current_workspaces();

    let monitors = display.monitors();
    let mut first_window: Option<ApplicationWindow> = None;

    DOCK_INSTANCES.with(|instances| {
        instances.borrow_mut().clear();
    });

    for i in 0..monitors.n_items() {
        if let Some(monitor) = monitors.item(i).and_downcast::<gtk4::gdk::Monitor>() {
            let win = create_dock_for_monitor(
                app,
                Some(&monitor),
                &initial_config,
                &initial_windows,
                &initial_workspaces,
            );
            if first_window.is_none() {
                first_window = Some(win);
            }
        }
    }

    if first_window.is_none() {
        let win = create_dock_for_monitor(
            app,
            None,
            &initial_config,
            &initial_windows,
            &initial_workspaces,
        );
        first_window = Some(win);
    }

    rx_win.attach(None, move |windows| {
        DOCK_INSTANCES.with(|insts| {
            for inst in insts.borrow_mut().iter_mut() {
                if inst.state.borrow().windows != windows {
                    inst.state.borrow_mut().windows = windows.clone();
                    refresh_monitor_instance(inst);
                }
            }
        });
        glib::ControlFlow::Continue
    });

    rx_cfg.attach(None, move |cfg| {
        DOCK_INSTANCES.with(|insts| {
            for inst in insts.borrow_mut().iter_mut() {
                if inst.state.borrow().pinned != cfg.pinned {
                    inst.state.borrow_mut().pinned = cfg.pinned.clone();
                    refresh_monitor_instance(inst);
                }
            }
        });
        glib::ControlFlow::Continue
    });

    rx_ws.attach(None, move |workspaces| {
        DOCK_INSTANCES.with(|insts| {
            for inst in insts.borrow_mut().iter_mut() {
                if inst.state.borrow().workspaces != workspaces {
                    inst.state.borrow_mut().workspaces = workspaces.clone();
                    refresh_monitor_instance(inst);
                }
            }
        });
        glib::ControlFlow::Continue
    });

    first_window.expect("At least one dock window created")
}

fn create_dock_for_monitor(
    app: &Application,
    monitor: Option<&gtk4::gdk::Monitor>,
    initial_config: &DockConfig,
    initial_windows: &[NiriWindowInfo],
    initial_workspaces: &[NiriWorkspaceInfo],
) -> ApplicationWindow {
    let connector = monitor
        .and_then(|m| m.connector())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "DP-1".to_string());
    let screen_height = monitor.map(|m| m.geometry().height()).unwrap_or(1080);

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("Ermete Dock ({})", connector))
        .css_classes(["dock-window"])
        .build();

    window.init_layer_shell();
    if let Some(m) = monitor {
        window.set_monitor(m);
    }
    window.set_layer(Layer::Top);
    window.set_namespace("dock");
    window.set_anchor(Edge::Bottom, true);
    window.set_margin(Edge::Bottom, 12);

    let container = GtkBox::new(Orientation::Horizontal, 8);
    container.add_css_class("dock-container");
    container.set_halign(Align::Center);
    container.set_valign(Align::Center);
    container.set_size_request(64, 48);
    window.set_child(Some(&container));

    let trigger_win = ApplicationWindow::builder()
        .application(app)
        .title(format!("Ermete Dock Trigger ({})", connector))
        .css_classes(["dock-window"])
        .build();

    trigger_win.init_layer_shell();
    if let Some(m) = monitor {
        trigger_win.set_monitor(m);
    }
    trigger_win.set_namespace("dock-trigger");
    trigger_win.set_layer(Layer::Overlay);
    trigger_win.set_exclusive_zone(-1);
    trigger_win.set_anchor(Edge::Bottom, true);
    trigger_win.set_anchor(Edge::Left, true);
    trigger_win.set_anchor(Edge::Right, true);
    trigger_win.set_height_request(6);

    let trigger_box = GtkBox::new(Orientation::Horizontal, 0);
    trigger_box.set_hexpand(true);
    trigger_box.set_vexpand(true);
    trigger_box.add_css_class("dock-trigger-area");
    trigger_win.set_child(Some(&trigger_box));

    let state = Rc::new(RefCell::new(DockState {
        pinned: initial_config.pinned.clone(),
        windows: initial_windows.to_vec(),
        workspaces: initial_workspaces.to_vec(),
        is_hovered: false,
    }));

    let spring = Rc::new(crate::core::spring::SpringAnimator::new(0.0, crate::core::spring::SpringConfig::default()));

    let motion_trigger = EventControllerMotion::new();
    let container_weak = container.downgrade();
    let window_weak = window.downgrade();
    let state_trig = state.clone();
    let spring_trig = spring.clone();
    motion_trigger.connect_enter(move |_, _, _| {
        state_trig.borrow_mut().is_hovered = true;
        if let Some(cont) = container_weak.upgrade() {
            animate_dock_visibility(&cont, &spring_trig, false);
        }
        if let Some(win) = window_weak.upgrade() {
            win.present();
        }
    });
    trigger_box.add_controller(motion_trigger);

    let motion_trig_win = EventControllerMotion::new();
    let container_weak_win = container.downgrade();
    let window_weak_win = window.downgrade();
    let state_trig_win = state.clone();
    let spring_trig_win = spring.clone();
    motion_trig_win.connect_enter(move |_, _, _| {
        state_trig_win.borrow_mut().is_hovered = true;
        if let Some(cont) = container_weak_win.upgrade() {
            animate_dock_visibility(&cont, &spring_trig_win, false);
        }
        if let Some(win) = window_weak_win.upgrade() {
            win.present();
        }
    });
    trigger_win.add_controller(motion_trig_win);

    let motion_dock_enter = EventControllerMotion::new();
    let container_weak_enter = container.downgrade();
    let state_enter = state.clone();
    let spring_enter = spring.clone();
    motion_dock_enter.connect_enter(move |_, _, _| {
        state_enter.borrow_mut().is_hovered = true;
        if let Some(cont) = container_weak_enter.upgrade() {
            animate_dock_visibility(&cont, &spring_enter, false);
        }
    });
    container.add_controller(motion_dock_enter);

    let motion_dock_leave = EventControllerMotion::new();
    let container_weak_leave = container.downgrade();
    let state_leave = state.clone();
    let connector_clone = connector.clone();
    let spring_leave = spring.clone();
    motion_dock_leave.connect_leave(move |_| {
        state_leave.borrow_mut().is_hovered = false;
        let cont_weak = container_weak_leave.clone();
        let st = state_leave.clone();
        let conn = connector_clone.clone();
        let spr = spring_leave.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
            if let Some(cont) = cont_weak.upgrade() {
                if !st.borrow().is_hovered && should_autohide_for_monitor(&st.borrow(), &conn, screen_height) {
                    animate_dock_visibility(&cont, &spr, true);
                }
            }
            glib::ControlFlow::Break
        });
    });
    container.add_controller(motion_dock_leave);

    let motion_trig_leave = EventControllerMotion::new();
    let container_weak_trig_leave = container.downgrade();
    let state_trig_leave = state.clone();
    let connector_clone2 = connector.clone();
    let spring_trig_leave = spring.clone();
    motion_trig_leave.connect_leave(move |_| {
        state_trig_leave.borrow_mut().is_hovered = false;
        let cont_weak = container_weak_trig_leave.clone();
        let st = state_trig_leave.clone();
        let conn = connector_clone2.clone();
        let spr = spring_trig_leave.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
            if let Some(cont) = cont_weak.upgrade() {
                if !st.borrow().is_hovered && should_autohide_for_monitor(&st.borrow(), &conn, screen_height) {
                    animate_dock_visibility(&cont, &spr, true);
                }
            }
            glib::ControlFlow::Break
        });
    });
    trigger_box.add_controller(motion_trig_leave);

    trigger_win.present();

    let mut inst = DockMonitorInstance {
        monitor_connector: connector,
        screen_height,
        window: window.downgrade(),
        container: container.clone(),
        trigger_win: trigger_win.downgrade(),
        state,
        spring,
    };
    refresh_monitor_instance(&mut inst);
    window.present();

    DOCK_INSTANCES.with(|instances| {
        instances.borrow_mut().push(inst);
    });

    window
}

#[allow(dead_code)]
pub fn toggle_dock_visibility() {
    DOCK_INSTANCES.with(|instances| {
        for inst in instances.borrow().iter() {
            if let Some(win) = inst.window.upgrade() {
                win.set_visible(true);
                win.present();
                let is_hidden = inst.spring.value() > 0.5 || inst.container.has_css_class("dock-hidden");
                animate_dock_visibility(&inst.container, &inst.spring, !is_hidden);
            }
        }
    });
}

fn refresh_monitor_instance(inst: &mut DockMonitorInstance) {
    while let Some(child) = inst.container.first_child() {
        inst.container.remove(&child);
    }

    let state = inst.state.borrow();
    let items = reconcile_dock_items(&state.pinned, &state.windows);
    let mut added_unpinned_separator = false;

    for item in items {
        if !item.is_pinned && !added_unpinned_separator {
            let sep = gtk4::Separator::new(Orientation::Vertical);
            sep.set_margin_top(8);
            sep.set_margin_bottom(8);
            inst.container.append(&sep);
            added_unpinned_separator = true;
        }

        let btn = Button::builder().css_classes(["dock-item-btn"]).build();
        let box_inner = GtkBox::new(Orientation::Vertical, 2);
        box_inner.set_halign(Align::Center);

        let icon = Image::from_icon_name(&item.icon_name);
        icon.set_pixel_size(44);
        
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&icon));
        if item.window_ids.len() > 1 {
            let badge = gtk4::Label::builder()
                .label(item.window_ids.len().to_string())
                .css_classes(["dock-instance-badge"])
                .halign(Align::End)
                .valign(Align::Start)
                .margin_top(0)
                .margin_end(0)
                .build();
            overlay.add_overlay(&badge);
        }
        box_inner.append(&overlay);

        let indicator = GtkBox::new(Orientation::Horizontal, 0);
        indicator.set_halign(Align::Center);
        if item.is_focused {
            indicator.add_css_class("dock-indicator-focused");
        } else if !item.window_ids.is_empty() {
            indicator.add_css_class("dock-indicator");
        } else {
            indicator.set_opacity(0.0);
            indicator.set_size_request(4, 4);
        }
        box_inner.append(&indicator);
        btn.set_child(Some(&box_inner));

        let voice_text = if item.window_ids.is_empty() {
            format!("App, {}", item.key_id.replace(".desktop", "").replace("org.", "").replace("com.", "").replace("gnome.", ""))
        } else {
            format!("{}, {} finestre aperte", item.key_id.replace(".desktop", "").replace("org.", "").replace("com.", "").replace("gnome.", ""), item.window_ids.len())
        };
        crate::core::attach_voiceover_hover(&btn, &voice_text);

        let item_clone = item.clone();
        let btn_clone = btn.clone();
        btn.connect_clicked(move |_| {
            if item_clone.window_ids.len() == 1 {
                let win_id = item_clone.window_ids[0];
                crate::core::niri_client::focus_window(win_id);
            } else if item_clone.window_ids.len() > 1 {
                show_window_picker_popover(&btn_clone, &item_clone);
            } else {
                let _ = Command::new("gtk-launch").arg(&item_clone.key_id).spawn();
            }
        });

        let gesture_right = GestureClick::new();
        gesture_right.set_button(3);
        let item_clone2 = item.clone();
        let btn_clone2 = btn.clone();
        gesture_right.connect_released(move |_, _, _, _| {
            show_dock_context_menu(&btn_clone2, &item_clone2);
        });
        btn.add_controller(gesture_right);

        let gesture_middle = GestureClick::new();
        gesture_middle.set_button(2);
        let key_id_mid = item.key_id.clone();
        gesture_middle.connect_released(move |_, _, _, _| {
            let _ = Command::new("gtk-launch").arg(&key_id_mid).spawn();
        });
        btn.add_controller(gesture_middle);

        let scroll_ctrl = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        let win_ids = item.window_ids.clone();
        scroll_ctrl.connect_scroll(move |_, _, dy| {
            if !win_ids.is_empty() {
                let idx = if dy > 0.0 { 0 } else { win_ids.len() - 1 };
                let win_id = win_ids[idx];
                crate::core::niri_client::focus_window(win_id);
            }
            glib::Propagation::Stop
        });
        btn.add_controller(scroll_ctrl);

        inst.container.append(&btn);
    }

    let should_hide = !state.is_hovered && should_autohide_for_monitor(&state, &inst.monitor_connector, inst.screen_height);
    animate_dock_visibility(&inst.container, &inst.spring, should_hide);
}

fn show_window_picker_popover(anchor: &Button, item: &DockItem) {
    let popover = Popover::builder()
        .autohide(true)
        .css_classes(["dock-popover"])
        .build();
    popover.set_parent(anchor);
    popover.connect_closed(|p| {
        p.set_child(None::<&gtk4::Widget>);
        p.unparent();
    });

    let box_inner = GtkBox::new(Orientation::Vertical, 4);
    for (i, title) in item.window_titles.iter().enumerate() {
        let win_id = item.window_ids[i];
        let btn = Button::builder()
            .label(title)
            .css_classes(["dock-popover-btn"])
            .build();
        let pop_close = popover.clone();
        btn.connect_clicked(move |_| {
            crate::core::niri_client::focus_window(win_id);
            pop_close.popdown();
        });
        box_inner.append(&btn);
    }
    popover.set_child(Some(&box_inner));
    popover.popup();
}

fn show_dock_context_menu(anchor: &Button, item: &DockItem) {
    let popover = Popover::builder()
        .autohide(true)
        .css_classes(["dock-popover"])
        .build();
    popover.set_parent(anchor);
    popover.connect_closed(|p| {
        p.set_child(None::<&gtk4::Widget>);
        p.unparent();
    });

    let box_inner = GtkBox::new(Orientation::Vertical, 4);

    let pin_label = if item.is_pinned {
        "Rimuovi dalla Dock"
    } else {
        "Fissa nella Dock"
    };
    let btn_pin = Button::builder()
        .label(pin_label)
        .css_classes(["dock-popover-btn"])
        .build();
    let key_id = item.key_id.clone();
    let is_pinned = item.is_pinned;
    let pop_close = popover.clone();
    btn_pin.connect_clicked(move |_| {
        if is_pinned {
            let _ = remove_pin(&key_id);
        } else {
            let _ = add_pin(&key_id);
        }
        pop_close.popdown();
    });
    box_inner.append(&btn_pin);

    let btn_new = Button::builder()
        .label("Nuova Finestra")
        .css_classes(["dock-popover-btn"])
        .build();
    let key_id2 = item.key_id.clone();
    let pop_close2 = popover.clone();
    btn_new.connect_clicked(move |_| {
        let _ = Command::new("gtk-launch").arg(&key_id2).spawn();
        pop_close2.popdown();
    });
    box_inner.append(&btn_new);

    if !item.window_ids.is_empty() {
        let btn_close = Button::builder()
            .label("Chiudi finestre")
            .css_classes(["dock-popover-btn"])
            .build();
        let win_ids = item.window_ids.clone();
        let pop_close3 = popover.clone();
        btn_close.connect_clicked(move |_| {
            for id in &win_ids {
                crate::core::niri_client::close_window_by_id(*id);
            }
            pop_close3.popdown();
        });
        box_inner.append(&btn_close);
    }

    let sep1 = gtk4::Separator::new(Orientation::Horizontal);
    sep1.set_margin_top(4);
    sep1.set_margin_bottom(4);
    box_inner.append(&sep1);

    let btn_settings = Button::builder()
        .label("Impostazioni")
        .css_classes(["dock-popover-btn"])
        .build();
    let pop_close_s = popover.clone();
    btn_settings.connect_clicked(move |_| {
        let _ = Command::new("gtk-launch").arg("os.ermete.Settings.desktop").spawn();
        pop_close_s.popdown();
    });
    box_inner.append(&btn_settings);

    let btn_sysmon = Button::builder()
        .label("Monitor di Sistema")
        .css_classes(["dock-popover-btn"])
        .build();
    let pop_close_sm = popover.clone();
    btn_sysmon.connect_clicked(move |_| {
        let _ = Command::new("gtk-launch").arg("missioncenter.desktop").spawn();
        pop_close_sm.popdown();
    });
    box_inner.append(&btn_sysmon);

    popover.set_child(Some(&box_inner));
    popover.popup();
}
