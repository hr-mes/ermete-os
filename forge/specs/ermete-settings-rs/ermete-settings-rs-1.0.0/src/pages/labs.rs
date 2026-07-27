use gtk4 as gtk;
use gtk::prelude::*;

pub fn build_page() -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 12);
    container.add_css_class("page-container");

    let title = gtk::Label::new(Some("Ermete Labs"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    
    let desc = gtk::Label::new(Some("Experimental features and beta toggles."));
    desc.add_css_class("dim-label");
    desc.set_halign(gtk::Align::Start);

    container.append(&title);
    container.append(&desc);

    container
}
