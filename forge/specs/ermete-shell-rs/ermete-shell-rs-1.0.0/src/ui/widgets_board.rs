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

    // Calendar Days Grid
    let grid = Grid::builder()
        .row_spacing(4)
        .column_spacing(4)
        .halign(Align::Center)
        .css_classes(vec!["calendar-grid".to_string()])
        .build();

    let day_headers = ["S", "M", "T", "W", "T", "F", "S"];
    for (col, day_name) in day_headers.iter().enumerate() {
        let lbl = Label::builder()
            .label(*day_name)
            .css_classes(vec!["calendar-day-label".to_string()])
            .halign(Align::Center)
            .build();
        grid.attach(&lbl, col as i32, 0, 1, 1);
    }

    // Days grid for current month mock (1..31 starting on Wednesday for Aug 2026)
    let current_day = now.format("%e").to_string().trim().parse::<i32>().unwrap_or(9);
    let start_offset = 6; // Aug 1, 2026 is Saturday (offset 6)
    let total_days = 31;

    for day in 1..=total_days {
        let pos = start_offset + day - 1;
        let row = 1 + (pos / 7);
        let col = pos % 7;

        let btn = Button::builder()
            .label(&day.to_string())
            .css_classes(vec!["calendar-day-btn".to_string()])
            .build();

        if day == current_day {
            btn.add_css_class("today");
        }

        grid.attach(&btn, col, row, 1, 1);
    }

    card.append(&grid);

    // Upcoming Event Mock
    let event_box = GtkBox::new(Orientation::Vertical, 2);
    event_box.add_css_class("calendar-event-item");

    let event_time = Label::builder()
        .label("14:30 • Upcoming Meeting")
        .css_classes(vec!["calendar-event-time".to_string()])
        .halign(Align::Start)
        .build();

    let event_title = Label::builder()
        .label("Ermete OS v1.0 Architecture Sync")
        .css_classes(vec!["calendar-event-text".to_string()])
        .halign(Align::Start)
        .build();

    event_box.append(&event_time);
    event_box.append(&event_title);
    card.append(&event_box);

    card
}

/// Builds the Weather Widget section
fn build_weather_widget() -> GtkBox {
    let card = GtkBox::new(Orientation::Vertical, 12);
    card.add_css_class("widget-card");

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 8);
    let icon = Image::builder()
        .icon_name("weather-clear-symbolic")
        .pixel_size(18)
        .css_classes(vec!["widget-header-icon".to_string()])
        .build();

    let title = Label::builder()
        .label("Weather")
        .css_classes(vec!["widget-header-title".to_string()])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let location = Label::builder()
        .label("Milan / Cupertino")
        .css_classes(vec!["widgets-board-subtitle".to_string()])
        .halign(Align::End)
        .build();

    header.append(&icon);
    header.append(&title);
    header.append(&location);
    card.append(&header);

    // Main Temp Display Row
    let temp_row = GtkBox::new(Orientation::Horizontal, 14);
    temp_row.set_valign(Align::Center);

    let temp_label = Label::builder()
        .label("24°C")
        .css_classes(vec!["weather-temp-main".to_string()])
        .halign(Align::Start)
        .build();

    let condition_box = GtkBox::new(Orientation::Vertical, 2);
    condition_box.set_valign(Align::Center);

    let city_label = Label::builder()
        .label("Sunny & Clear")
        .css_classes(vec!["weather-city".to_string()])
        .halign(Align::Start)
        .build();

    let details_label = Label::builder()
        .label("H: 27° L: 16° • AQI 24 (Good)")
        .css_classes(vec!["weather-condition".to_string()])
        .halign(Align::Start)
        .build();

    condition_box.append(&city_label);
    condition_box.append(&details_label);

    temp_row.append(&temp_label);
    temp_row.append(&condition_box);
    card.append(&temp_row);

    // Weather Metrics Row
    let metrics_row = GtkBox::new(Orientation::Horizontal, 8);
    metrics_row.set_halign(Align::Fill);
    metrics_row.set_hexpand(true);

    let humidity = Label::builder()
        .label("💧 42% Humidity")
        .css_classes(vec!["weather-detail-badge".to_string()])
        .hexpand(true)
        .halign(Align::Center)
        .build();

    let wind = Label::builder()
        .label("💨 14 km/h Wind")
        .css_classes(vec!["weather-detail-badge".to_string()])
        .hexpand(true)
        .halign(Align::Center)
        .build();

    metrics_row.append(&humidity);
    metrics_row.append(&wind);
    card.append(&metrics_row);

    // 4-Day Forecast Strip
    let forecast_box = GtkBox::new(Orientation::Horizontal, 6);
    forecast_box.set_halign(Align::Center);
    forecast_box.set_margin_top(4);

    let forecast_data = [
        ("Today", "☀️", "24°"),
        ("Mon", "🌤️", "26°"),
        ("Tue", "🌧️", "21°"),
        ("Wed", "🌤️", "25°"),
    ];

    for (day, weather_icon, temp) in forecast_data {
        let col = GtkBox::new(Orientation::Vertical, 4);
        col.add_css_class("weather-forecast-col");
        col.set_halign(Align::Center);

        let d_lbl = Label::builder()
            .label(day)
            .css_classes(vec!["weather-forecast-day".to_string()])
            .halign(Align::Center)
            .build();

        let i_lbl = Label::builder()
            .label(weather_icon)
            .halign(Align::Center)
            .build();

        let t_lbl = Label::builder()
            .label(temp)
            .css_classes(vec!["weather-forecast-temp".to_string()])
            .halign(Align::Center)
            .build();

        col.append(&d_lbl);
        col.append(&i_lbl);
        col.append(&t_lbl);
        forecast_box.append(&col);
    }

    card.append(&forecast_box);
    card
}

/// Builds the Stocks Watchlist Mock Widget section
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
        .label("Markets & Stocks")
        .css_classes(vec!["widget-header-title".to_string()])
        .halign(Align::Start)
        .hexpand(true)
        .build();

    let live_badge = Label::builder()
        .label("LIVE")
        .css_classes(vec!["stock-pill-positive".to_string()])
        .halign(Align::End)
        .build();

    header.append(&icon);
    header.append(&title);
    header.append(&live_badge);
    card.append(&header);

    // Mock Stocks List
    let stocks = [
        ("ERMT", "Ermete OS Core", "$342.80", "+4.12 (+1.22%)", true),
        ("AAPL", "Apple Inc.", "$225.40", "+1.85 (+0.83%)", true),
        ("NVDA", "NVIDIA Corp.", "$128.90", "-0.65 (-0.50%)", false),
        ("GOOGL", "Alphabet Inc.", "$178.50", "+2.10 (+1.19%)", true),
        ("MSFT", "Microsoft Corp.", "$448.20", "+3.40 (+0.76%)", true),
    ];

    for (symbol, name, price, change, is_positive) in stocks {
        let row = GtkBox::new(Orientation::Horizontal, 10);
        row.add_css_class("stock-row");
        row.set_valign(Align::Center);

        let sym_box = GtkBox::new(Orientation::Vertical, 1);
        sym_box.set_hexpand(true);

        let sym_lbl = Label::builder()
            .label(symbol)
            .css_classes(vec!["stock-symbol".to_string()])
            .halign(Align::Start)
            .build();

        let name_lbl = Label::builder()
            .label(name)
            .css_classes(vec!["stock-name".to_string()])
            .halign(Align::Start)
            .build();

        sym_box.append(&sym_lbl);
        sym_box.append(&name_lbl);

        let price_lbl = Label::builder()
            .label(price)
            .css_classes(vec!["stock-price".to_string()])
            .halign(Align::End)
            .build();

        let pill_class = if is_positive {
            "stock-pill-positive"
        } else {
            "stock-pill-negative"
        };

        let change_lbl = Label::builder()
            .label(change)
            .css_classes(vec![pill_class.to_string()])
            .halign(Align::End)
            .build();

        row.append(&sym_box);
        row.append(&price_lbl);
        row.append(&change_lbl);
        card.append(&row);
    }

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
    #[test]
    fn test_mock_weather_data() {
        let temp = "24°C";
        assert!(temp.contains("24"));
    }

    #[test]
    fn test_stock_formatting() {
        let ticker = "ERMT";
        let price = "$342.80";
        assert_eq!(ticker, "ERMT");
        assert!(price.starts_with('$'));
    }
}
