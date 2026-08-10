#[zbus::proxy(
    interface = "org.ermete.Settings",
    default_service = "org.ermete.Settings",
    default_path = "/org/ermete/Settings"
)]
pub trait Settings {
    #[zbus(property, name = "ColorScheme")]
    fn color_scheme(&self) -> zbus::Result<String>;
    #[zbus(property, name = "ColorScheme")]
    fn set_color_scheme(&self, value: &str) -> zbus::Result<()>;

    #[zbus(property, name = "AccentColor")]
    fn accent_color(&self) -> zbus::Result<String>;
    #[zbus(property, name = "AccentColor")]
    fn set_accent_color(&self, value: &str) -> zbus::Result<()>;

    #[zbus(property, name = "Wallpaper")]
    fn wallpaper(&self) -> zbus::Result<String>;
    #[zbus(property, name = "Wallpaper")]
    fn set_wallpaper(&self, value: &str) -> zbus::Result<()>;

    #[zbus(property, name = "DesktopLayout")]
    fn desktop_layout(&self) -> zbus::Result<String>;
    #[zbus(property, name = "DesktopLayout")]
    fn set_desktop_layout(&self, value: &str) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.ermete.Settings.Appearance",
    default_service = "org.ermete.Settings",
    default_path = "/org/ermete/Settings/Appearance"
)]
pub trait Appearance {
    #[zbus(property, name = "ColorScheme")]
    fn color_scheme(&self) -> zbus::Result<String>;
    #[zbus(property, name = "ColorScheme")]
    fn set_color_scheme(&self, value: &str) -> zbus::Result<()>;

    #[zbus(property, name = "AccentColor")]
    fn accent_color(&self) -> zbus::Result<String>;
    #[zbus(property, name = "AccentColor")]
    fn set_accent_color(&self, value: &str) -> zbus::Result<()>;

    #[zbus(property, name = "Wallpaper")]
    fn wallpaper(&self) -> zbus::Result<String>;
    #[zbus(property, name = "Wallpaper")]
    fn set_wallpaper(&self, value: &str) -> zbus::Result<()>;

    #[zbus(property, name = "DesktopLayout")]
    fn desktop_layout(&self) -> zbus::Result<String>;
    #[zbus(property, name = "DesktopLayout")]
    fn set_desktop_layout(&self, value: &str) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.ermete.Settings.Layout",
    default_service = "org.ermete.Settings",
    default_path = "/org/ermete/Settings/Layout"
)]
pub trait Layout {
    #[zbus(property, name = "DesktopLayout")]
    fn desktop_layout(&self) -> zbus::Result<String>;
    #[zbus(property, name = "DesktopLayout")]
    fn set_desktop_layout(&self, value: &str) -> zbus::Result<()>;

    fn apply_desktop_layout(&self, layout_id: &str) -> zbus::Result<()>;
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

/// Consolidated helper for executing async operations on LayoutProxy without boilerplate
pub async fn with_layout_proxy<F, Fut>(f: F)
where
    F: FnOnce(LayoutProxy<'static>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    if let Ok(conn) = crate::get_connection().await {
        if let Ok(proxy) = LayoutProxy::new(&conn).await {
            f(proxy).await;
        }
    }
}


