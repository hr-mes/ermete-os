use gtk::gdk;

pub fn load_glass_theme() {
    let css_provider = gtk::CssProvider::new();
    let css = include_str!("style.css");
    css_provider.load_from_data(css);

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
