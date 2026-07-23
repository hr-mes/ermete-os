#[zbus::dbus_proxy(
    interface = "org.ermete.Settings",
    default_service = "org.ermete.Settings",
    default_path = "/org/ermete/Settings"
)]
pub trait Settings {
    #[dbus_proxy(property, name = "ColorScheme")]
    fn color_scheme(&self) -> zbus::Result<String>;
    #[dbus_proxy(property, name = "ColorScheme")]
    fn set_color_scheme(&self, value: &str) -> zbus::Result<()>;

    #[dbus_proxy(property, name = "AccentColor")]
    fn accent_color(&self) -> zbus::Result<String>;
    #[dbus_proxy(property, name = "AccentColor")]
    fn set_accent_color(&self, value: &str) -> zbus::Result<()>;

    #[dbus_proxy(property, name = "Wallpaper")]
    fn wallpaper(&self) -> zbus::Result<String>;
    #[dbus_proxy(property, name = "Wallpaper")]
    fn set_wallpaper(&self, value: &str) -> zbus::Result<()>;
}
