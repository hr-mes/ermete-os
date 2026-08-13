use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Grid, Image, Label,
    Orientation, ScrolledWindow,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use chrono::Local;
use std::cell::RefCell;

thread_local! {
    static WIDGETS_BOARD_WIN: RefCell<Option<glib::WeakRef<ApplicationWindow>>> = const { RefCell::new(None) };
}

fn init_widgets_board_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(r#"
        window.widgets-board-window {
            background: transparent;
            background-color: transparent;
            border: none;
            box-shadow: none;
        }

        .widgets-board-panel {
            background-color: rgba(22, 22, 32, 0.84);
            backdrop-filter: blur(32px) saturate(180%);
            border: 1px solid rgba(255, 255, 255, 0.16);
            border-radius: 28px;
            padding: 20px;
            box-shadow: -10px 16px 48px rgba(0, 0, 0, 0.55);
            min-width: 360px;
            max-width: 380px;
        }

        .widgets-board-title {
            font-size: 20px;
            font-weight: 800;
            color: #ffffff;
            letter-spacing: -0.4px;
            font-family: system-ui, -apple-system, sans-serif;
        }

        .widgets-board-subtitle {
            font-size: 12px;
            font-weight: 600;
            color: rgba(255, 255, 255, 0.6);
        }

        .widget-card {
            background: linear-gradient(135deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.03));
            backdrop-filter: blur(20px);
            border: 1px solid rgba(255, 255, 255, 0.12);
            border-radius: 20px;
            padding: 16px;
            margin-bottom: 14px;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
            transition: all 250ms cubic-bezier(0.16, 1, 0.3, 1);
        }

        .widget-card:hover {
            border-color: rgba(137, 180, 250, 0.4);
            box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4), 0 0 16px rgba(137, 180, 250, 0.2);
            transform: translateY(-2px);
        }

        .widget-header-title {
            font-size: 14px;
            font-weight: 700;
            color: #cdd6f4;
            font-family: system-ui, -apple-system, sans-serif;
        }

        .widget-header-icon {
            color: #89b4fa;
            margin-right: 8px;
        }

        /* Calendar Widget Styling */
        .calendar-grid {
            margin-top: 10px;
        }

        .calendar-day-label {
            font-size: 11px;
            font-weight: 700;
            color: rgba(255, 255, 255, 0.5);
            margin-bottom: 6px;
        }

        .calendar-day-btn {
            background: transparent;
            border: none;
            border-radius: 50%;
            min-width: 32px;
            min-height: 32px;
            padding: 0px;
            font-size: 12px;
            font-weight: 600;
            color: rgba(255, 255, 255, 0.85);
            transition: all 200ms ease;
        }

        .calendar-day-btn:hover {
            background-color: rgba(255, 255, 255, 0.15);
            color: #ffffff;
        }

        .calendar-day-btn.today {
            background-color: #89b4fa;
            color: #11111b;
            font-weight: 800;
            box-shadow: 0 4px 14px rgba(137, 180, 250, 0.5);
        }

        .calendar-event-item {
            background-color: rgba(137, 180, 250, 0.12);
            border-left: 3px solid #89b4fa;
            border-radius: 6px;
            padding: 6px 10px;
            margin-top: 10px;
        }

        .calendar-event-time {
            font-size: 11px;
            font-weight: 700;
            color: #89b4fa;
        }

        .calendar-event-text {
            font-size: 12px;
            font-weight: 600;
            color: #ffffff;
        }

        /* Weather Widget Styling */
        .weather-temp-main {
            font-size: 36px;
            font-weight: 800;
            color: #ffffff;
            letter-spacing: -1px;
        }

        .weather-city {
            font-size: 15px;
            font-weight: 700;
            color: #cdd6f4;
        }

        .weather-condition {
            font-size: 12px;
            font-weight: 600;
            color: rgba(255, 255, 255, 0.7);
        }

        .weather-detail-badge {
            background-color: rgba(255, 255, 255, 0.08);
            border-radius: 10px;
            padding: 4px 10px;
            font-size: 11px;
            font-weight: 600;
            color: rgba(255, 255, 255, 0.8);
        }

        .weather-forecast-col {
            background-color: rgba(255, 255, 255, 0.05);
            border-radius: 12px;
            padding: 8px 6px;
            min-width: 58px;
        }

        .weather-forecast-day {
            font-size: 11px;
            font-weight: 700;
            color: rgba(255, 255, 255, 0.6);
        }

        .weather-forecast-temp {
            font-size: 12px;
            font-weight: 700;
            color: #ffffff;
        }

        /* Stocks Widget Styling */
        .stock-row {
            background-color: rgba(255, 255, 255, 0.04);
            border-radius: 12px;
            padding: 8px 12px;
            margin-bottom: 6px;
            transition: all 200ms ease;
        }

        .stock-row:hover {
            background-color: rgba(255, 255, 255, 0.1);
        }

        .stock-symbol {
            font-size: 13px;
            font-weight: 800;
            color: #ffffff;
        }

        .stock-name {
            font-size: 11px;
            font-weight: 500;
            color: rgba(255, 255, 255, 0.5);
        }

        .stock-price {
            font-size: 13px;
            font-weight: 700;
            color: #ffffff;
        }

        .stock-pill-positive {
            background-color: rgba(166, 227, 161, 0.2);
            color: #a6e3a1;
            border: 1px solid rgba(166, 227, 161, 0.4);
            border-radius: 8px;
            padding: 2px 8px;
            font-size: 11px;
            font-weight: 700;
        }

        .stock-pill-negative {
            background-color: rgba(243, 139, 168, 0.2);
            color: #f38ba8;
            border: 1px solid rgba(243, 139, 168, 0.4);
            border-radius: 8px;
            padding: 2px 8px;
            font-size: 11px;
            font-weight: 700;
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

/// Builds the Calendar Widget section
fn build_calendar_widget() -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 10);
    card.add_css_class("widget-card");

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 8);
    let icon = Image::builder()
        .icon_name("office-calendar-symbolic")
        .pixel_size(18)
        .css_classes(vec!["widget-header-icon".to_string()])
        .build();

    let now = Local::now();
    let month_year = now.format("%B %Y").to_string();
    let header_title = Label::builder()
        .label(&month_year)
        .css_classes(vec!["widget-header-title".to_string()])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let today_date_str = now.format("%a, %b %e").to_string();
    let sub_date = Label::builder()
        .label(&today_date_str)
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::End)
        .build();

    header.append(&icon);
    header.append(&header_title);
    header.append(&sub_date);
    card.append(&header);

    let err_msg = Label::builder()
        .label("Dati non disponibili (IPC calendar offline)")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::Center)
        .margin_top(20)
        .margin_bottom(20)
        .build();
    
    card.append(&err_msg);


    // Calendar Events (No mock events: empty state if no IPC backend)
    let no_event_lbl = Label::builder()
        .label("Nessun evento in programma (IPC calendar offline)")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::Center)
        .margin_top(8)
        .build();

    card.append(&no_event_lbl);
    card
}

/// Builds the Weather Widget section (Disconnected state without IPC backend)
fn build_weather_widget() -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 12);
    card.add_css_class("widget-card");

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 8);
    let icon = Image::builder()
        .icon_name("weather-severe-alert-symbolic")
        .pixel_size(18)
        .css_classes(vec!["widget-header-icon".to_string()])
        .build();

    let title = Label::builder()
        .label("Meteo")
        .css_classes(vec!["widget-header-title".to_string()])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let status = Label::builder()
        .label("Errore Connessione")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::End)
        .build();

    header.append(&icon);
    header.append(&title);
    header.append(&status);
    card.append(&header);

    // Empty / Error State (No fake temperatures or mock forecasts)
    let err_box = GtkBox::new(Orientation::Vertical, 6);
    err_box.set_halign(Align::Center);
    err_box.set_margin_top(8);
    err_box.set_margin_bottom(8);

    let err_msg = Label::builder()
        .label("Nessun dato meteo IPC disponibile. Demone Meteo offline.")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::Center)
        .build();

    err_box.append(&err_msg);
    card.append(&err_box);

    card
}

/// Builds the Stocks Watchlist Widget section (Disconnected state without IPC backend)
fn build_stocks_widget() -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 10);
    card.add_css_class("widget-card");

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 8);
    let icon = Image::builder()
        .icon_name("emblem-favorite-symbolic")
        .pixel_size(18)
        .css_classes(vec!["widget-header-icon".to_string()])
        .build();

    let title = Label::builder()
        .label("Mercati & Azioni")
        .css_classes(vec!["widget-header-title".to_string()])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let offline_badge = Label::builder()
        .label("OFFLINE")
        .css_classes(vec!["stock-pill-negative".to_string()])
        .halign(Align::End)
        .build();

    header.append(&icon);
    header.append(&title);
    header.append(&offline_badge);
    card.append(&header);

    // Empty / Error State (No fake stock tickers)
    let err_box = GtkBox::new(Orientation::Vertical, 6);
    err_box.set_halign(Align::Center);
    err_box.set_margin_top(8);
    err_box.set_margin_bottom(8);

    let err_msg = Label::builder()
        .label("Nessun provider IPC per dati finanziari connesso.")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::Center)
        .build();

    err_box.append(&err_msg);
    card.append(&err_box);

    card
}

/// Spawns or toggles the Sidebar Widgets Board window.
pub fn toggle_widgets_board(app: &Application) {
    let mut close_existing = false;
    WIDGETS_BOARD_WIN.with(|w| {
        if let Some(weak_ref) = w.borrow().as_ref() {
            if let Some(win) = weak_ref.upgrade() {
                if win.is_visible() {
                    win.close();
                    close_existing = true;
                }
            }
        }
    });

    if close_existing {
        return;
    }

    show_widgets_board(app);
}

/// Displays the Sidebar Widgets Board HUD anchored to the right side of the screen.
pub fn show_widgets_board(app: &Application) {
    init_widgets_board_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Sidebar Widgets Board")
        .css_classes(vec!["widgets-board-window"])
        .build();

    window.init_layer_shell();
    window.set_namespace("widgets-board");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    // Anchor to top-right-bottom for full height sidebar panel
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, false);

    window.set_margin(Edge::Top, 12);
    window.set_margin(Edge::Right, 12);
    window.set_margin(Edge::Bottom, 12);

    let panel = GtkBox::new(Orientation::Vertical, 14);
    panel.add_css_class("widgets-board-panel");

    // Title Row with Close Button
    let top_bar = GtkBox::new(Orientation::Horizontal, 10);

    let title_box = GtkBox::new(Orientation::Vertical, 2);
    title_box.set_hexpand(true);

    let title_lbl = Label::builder()
        .label("Widgets")
        .css_classes(vec!["widgets-board-title".to_string()])
        .halign(Align::Start)
        .build();

    let subtitle_lbl = Label::builder()
        .label("Ermete Desktop Dashboard")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::Start)
        .build();

    title_box.append(&title_lbl);
    title_box.append(&subtitle_lbl);

    let close_btn = Button::builder()
        .icon_name("window-close-symbolic")
        .css_classes(vec!["morphic-pill-btn".to_string()])
        .halign(Align::End)
        .valign(Align::Center)
        .build();

    let win_close = window.clone();
    close_btn.connect_clicked(move |_| {
        win_close.close();
    });

    top_bar.append(&title_box);
    top_bar.append(&close_btn);
    panel.append(&top_bar);

    // Scrollable Widgets Container
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();

    let content = GtkBox::new(Orientation::Vertical, 0);

    content.append(&build_calendar_widget());
    content.append(&build_weather_widget());
    content.append(&build_stocks_widget());

    scroll.set_child(Some(&content));
    panel.append(&scroll);

    window.set_child(Some(&panel));

    // Register popup autoclose behavior so clicking outside closes the sidebar board
    crate::wayland::popup::setup_popup_autoclose(&window, "widgets-board");

    WIDGETS_BOARD_WIN.with(|w| {
        *w.borrow_mut() = Some(window.downgrade());
    });

    window.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widgets_board_instantiation() {
        if gtk4::init().is_ok() {
            let app = Application::new(Some("com.test.widgets"), Default::default());
            app.connect_startup(|a| {
                show_widgets_board(a);
            });
        }
    }
}

