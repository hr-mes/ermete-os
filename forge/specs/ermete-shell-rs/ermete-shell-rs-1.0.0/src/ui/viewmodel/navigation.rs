use gtk4::Application;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiPopoverTarget {
    StartMenu,
    ControlCenter,
    AudioMixer,
    Bluetooth,
    Wifi,
    WifiPassword(String),
    WifiDetails(String, bool),
    SystemMonitor,
    Calendar,
    Spotlight,
    Notifications,
    PowerMenu,
    Clipboard,
    Store,
}

pub struct NavigationViewModel;

impl NavigationViewModel {
    pub fn navigate_to(app: &Application, target: UiPopoverTarget) {
        use crate::ui::topbar::toggle_or_open_popup;
        match target {
            UiPopoverTarget::StartMenu => {
                toggle_or_open_popup("launcher", || crate::ui::control_center::show_start_menu_popover(app));
            }
            UiPopoverTarget::ControlCenter => {
                crate::control_center::show_control_center_panel(app);
            }
            UiPopoverTarget::AudioMixer => {
                toggle_or_open_popup("media-player", || crate::ui::control_center::show_audio_mixer_popover(app));
            }
            UiPopoverTarget::Bluetooth => {
                toggle_or_open_popup("bluetooth", || crate::ui::control_center::show_bluetooth_popover(app));
            }
            UiPopoverTarget::Wifi => {
                toggle_or_open_popup("wifi", || crate::ui::control_center::show_wifi_popover(app));
            }
            UiPopoverTarget::WifiPassword(ssid) => {
                crate::ui::control_center::wifi::show_wifi_password_modal(app, &ssid);
            }
            UiPopoverTarget::WifiDetails(ssid, active) => {
                crate::ui::control_center::wifi::show_wifi_details_modal(app, &ssid, active);
            }
            UiPopoverTarget::SystemMonitor => {
                toggle_or_open_popup("sys-monitor", || crate::ui::control_center::show_system_monitor_modal(app));
            }
            UiPopoverTarget::Calendar => {
                toggle_or_open_popup("calendar", || crate::ui::control_center::show_calendar_popover(app));
            }
            UiPopoverTarget::Spotlight => {
                toggle_or_open_popup("spotlight", || crate::ui::spotlight::show_spotlight_modal(app));
            }
            UiPopoverTarget::Notifications => {
                toggle_or_open_popup("notifications", || crate::ui::notifications::show_notification_center(app));
            }
            UiPopoverTarget::PowerMenu => {
                toggle_or_open_popup("powermenu", || crate::ui::powermenu::show_powermenu_modal(app));
            }
            UiPopoverTarget::Clipboard => {
                toggle_or_open_popup("clipboard", || crate::ui::clipboard::show_clipboard_modal(app));
            }
            UiPopoverTarget::Store => {
                toggle_or_open_popup("store", || crate::ui::store::show_store_modal(app));
            }
        }
    }
}
