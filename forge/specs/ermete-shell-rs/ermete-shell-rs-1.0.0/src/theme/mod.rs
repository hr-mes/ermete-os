pub mod palette;

pub use palette::*;

use gtk4::gio;
use gtk4::prelude::*;

thread_local! {
    pub static CSS_PROVIDER: std::cell::RefCell<Option<gtk4::CssProvider>> = const { std::cell::RefCell::new(None) };
    pub static DYNAMIC_THEME_PROVIDER: std::cell::RefCell<Option<gtk4::CssProvider>> = const { std::cell::RefCell::new(None) };
}

pub fn init_css() {
    CSS_PROVIDER.with(|provider_ref| {
        let mut p = provider_ref.borrow_mut();
        if p.is_none() {
            if let Some(display) = gtk4::gdk::Display::default() {
                let provider = gtk4::CssProvider::new();
                let path = "/usr/share/ermete/style.css";
                if std::path::Path::new(path).exists() {
                    provider.load_from_path(path);
                }
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
                *p = Some(provider);
            }
            ermete_style::load_glass_theme();
        }
    });

    init_dynamic_theme_css();
}

pub fn init_dynamic_theme_css() {
    DYNAMIC_THEME_PROVIDER.with(|provider_ref| {
        let mut p = provider_ref.borrow_mut();
        if p.is_none() {
            if let Some(display) = gtk4::gdk::Display::default() {
                let theme_provider = gtk4::CssProvider::new();

                let theme_path = get_theme_css_path();
                if !theme_path.exists() {
                    let default_palette = Material3Palette::default_dark();
                    let _ = default_palette.write_to_file(&theme_path);
                }

                if theme_path.exists() {
                    theme_provider.load_from_path(&theme_path);
                }

                gtk4::style_context_add_provider_for_display(
                    &display,
                    &theme_provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_USER,
                );

                setup_theme_css_monitor(theme_provider.clone(), theme_path.clone());

                *p = Some(theme_provider);
            }
        }
    });
}

pub fn apply_dynamic_material3_theme(wallpaper_path: Option<&std::path::Path>, is_dark: bool) {
    let theme_path = get_theme_css_path();
    let palette = Material3Palette::extract_from_wallpaper(wallpaper_path, is_dark);
    if let Err(e) = palette.write_to_file(&theme_path) {
        tracing::error!(error = %e, "Failed writing extracted Material 3 GTK4 CSS theme.");
    } else {
        tracing::info!(path = %theme_path.display(), "Extracted and wrote Material 3 dynamic theme CSS.");
    }
}

pub fn get_theme_css_path() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(xdg).join("ermete").join("theme.css")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".config").join("ermete").join("theme.css")
    } else {
        std::path::PathBuf::from("/var/lib/ermete/theme.css")
    }
}

fn setup_theme_css_monitor(provider: gtk4::CssProvider, theme_path: std::path::PathBuf) {
    let file = gio::File::for_path(&theme_path);
    if let Ok(monitor) = file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        let path_clone = theme_path.clone();
        monitor.connect_changed(move |_, _, _, event_type| {
            if event_type == gio::FileMonitorEvent::ChangesDoneHint || event_type == gio::FileMonitorEvent::Changed {
                if path_clone.exists() {
                    provider.load_from_path(&path_clone);
                    tracing::info!(path = %path_clone.display(), "Dynamic GTK4 Material 3 theme hot-reloaded successfully.");
                }
            }
        });
        std::mem::forget(monitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_css_path() {
        let path = get_theme_css_path();
        assert!(path.to_str().expect("Ermete OS: Fallimento critico di unwrapping. Zero-Trust Panic Invocato.").contains("theme.css"));
    }

    #[test]
    fn test_apply_dynamic_material3_theme() {
        let tmp = std::env::temp_dir().join("ermete_mod_theme_test");
        std::env::set_var("XDG_CONFIG_HOME", &tmp);

        apply_dynamic_material3_theme(None, true);
        let theme_file = get_theme_css_path();
        assert!(theme_file.exists());
        let content = std::fs::read_to_string(&theme_file).expect("Ermete OS: Fallimento critico di unwrapping. Zero-Trust Panic Invocato.");
        assert!(content.contains("@define-color primary"));
        assert!(content.contains("@define-color surface"));
    }
}

