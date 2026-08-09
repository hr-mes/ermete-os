#![allow(deprecated)]
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

#[zbus::dbus_proxy(
    interface = "org.ermete.Settings.Appearance",
    default_service = "org.ermete.Settings",
    default_path = "/org/ermete/Settings/Appearance"
)]
pub trait Appearance {
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

/// Consolidated helper for executing async operations on SettingsProxy without boilerplate
pub async fn with_settings_proxy<F, Fut>(f: F)
where
    F: FnOnce(SettingsProxy<'static>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    if let Ok(conn) = crate::get_connection().await {
        if let Ok(proxy) = SettingsProxy::new(&conn).await {
            f(proxy).await;
        }
    }
}

/// Consolidated helper for executing async operations on AppearanceProxy without boilerplate
pub async fn with_appearance_proxy<F, Fut>(f: F)
where
    F: FnOnce(AppearanceProxy<'static>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    if let Ok(conn) = crate::get_connection().await {
        if let Ok(proxy) = AppearanceProxy::new(&conn).await {
            f(proxy).await;
        }
    }
}

