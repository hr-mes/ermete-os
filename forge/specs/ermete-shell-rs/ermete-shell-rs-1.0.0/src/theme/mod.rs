
thread_local! {
    pub static CSS_PROVIDER: std::cell::RefCell<Option<gtk4::CssProvider>> = std::cell::RefCell::new(None);
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
}
