use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, EventControllerKey, Image, Label,
    Orientation, Picture, ScrolledWindow, TextBuffer, TextView,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::path::{Path, PathBuf};
use std::sync::Once;

static CSS_INIT: Once = Once::new();

pub fn ensure_quicklook_css() {
    CSS_INIT.call_once(|| {
        if let Some(display) = gtk4::gdk::Display::default() {
            let provider = gtk4::CssProvider::new();
            let css = r#"
                .quicklook-window {
                    background-color: rgba(0, 0, 0, 0.45);
                }
                .quicklook-card {
                    background-color: rgba(30, 30, 46, 0.82);
                    backdrop-filter: blur(32px);
                    border: 1px solid rgba(255, 255, 255, 0.15);
                    border-radius: 24px;
                    padding: 24px;
                    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6);
                    min-width: 680px;
                    min-height: 500px;
                }
                .quicklook-header-title {
                    font-size: 18px;
                    font-weight: 800;
                    color: #cdd6f4;
                }
                .quicklook-close-btn {
                    background-color: rgba(255, 255, 255, 0.08);
                    border: 1px solid rgba(255, 255, 255, 0.12);
                    border-radius: 50%;
                    min-width: 32px;
                    min-height: 32px;
                    padding: 0;
                    color: #cdd6f4;
                }
                .quicklook-close-btn:hover {
                    background-color: rgba(243, 139, 168, 0.3);
                    color: #f38ba8;
                }
                .quicklook-preview-container {
                    background-color: rgba(17, 17, 27, 0.6);
                    border: 1px solid rgba(255, 255, 255, 0.08);
                    border-radius: 16px;
                    padding: 12px;
                    min-height: 340px;
                }
                .quicklook-picture {
                    border-radius: 12px;
                }
                .quicklook-scrolled-text {
                    background-color: transparent;
                }
                .quicklook-textview {
                    background-color: transparent;
                    color: #cdd6f4;
                    font-family: monospace;
                    font-size: 13px;
                }
                .quicklook-fallback-icon {
                    color: #89b4fa;
                    margin-bottom: 8px;
                }
                .quicklook-fallback-label {
                    font-size: 14px;
                    color: #a6adc8;
                }
                .quicklook-footer {
                    padding-top: 8px;
                }
                .quicklook-filename {
                    font-size: 16px;
                    font-weight: 700;
                    color: #cdd6f4;
                }
                .quicklook-filesize {
                    font-size: 12px;
                    font-weight: 500;
                    color: #a6adc8;
                }
                .quicklook-openwith-btn {
                    font-size: 14px;
                    font-weight: 700;
                    padding: 10px 20px;
                    border-radius: 14px;
                }
            "#;
            provider.load_from_data(css);
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 10,
            );
        }
    });
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuickLookContentType {
    Image,
    Text(String),
    BinaryOrOther,
}

#[derive(Debug, Clone)]
pub struct QuickLookFileData {
    pub path: PathBuf,
    pub file_name: String,
    pub size_formatted: String,
    pub content_type: QuickLookContentType,
}

impl QuickLookFileData {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Sconosciuto".to_string());

        let size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);
        let size_formatted = format_file_size(size);

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let content_type = match ext.as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "svg" | "gif" | "bmp" | "ico" | "tiff" => {
                QuickLookContentType::Image
            }
            "rs" | "toml" | "json" | "md" | "txt" | "py" | "c" | "cpp" | "h" | "hpp" | "sh"
            | "yaml" | "yml" | "css" | "html" | "js" | "ts" | "go" | "java" | "kt" | "nix"
            | "conf" | "ini" | "log" | "xml" | "patch" | "diff" | "lock" => {
                let text = std::fs::read_to_string(&path)
                    .map(|s| {
                        if s.len() > 65536 {
                            format!("{}...\n\n[Anteprima troncata a 64 KB]", &s[..65536])
                        } else {
                            s
                        }
                    })
                    .unwrap_or_else(|_| "[Impossibile leggere il file di testo]".to_string());
                QuickLookContentType::Text(text)
            }
            _ => QuickLookContentType::BinaryOrOther,
        };

        Self {
            path,
            file_name,
            size_formatted,
            content_type,
        }
    }
}

pub fn format_file_size(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.1} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.1} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} B", size_bytes)
    }
}

pub fn open_file_with_default(path: &Path) {
    let uri = format!("file://{}", path.to_string_lossy());
    if gtk4::gio::AppInfo::launch_default_for_uri(&uri, None::<&gtk4::gio::AppLaunchContext>).is_err()
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }
}

pub struct QuickLookModel {
    pub file_data: Option<QuickLookFileData>,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum QuickLookMsg {
    OpenFile(PathBuf),
    OpenWith,
    Close,
}

#[relm4::component(pub)]
impl SimpleComponent for QuickLookModel {
    type Input = QuickLookMsg;
    type Output = ();
    type Init = Option<PathBuf>;

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Quick Look"),
            add_css_class: "quicklook-window",
            add_css_class: "glassmorphism",
            set_default_width: 720,
            set_default_height: 540,
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 16,
                set_margin_top: 20,
                set_margin_bottom: 20,
                set_margin_start: 20,
                set_margin_end: 20,
                set_valign: gtk::Align::Center,
                set_halign: gtk::Align::Center,
                add_css_class: "quicklook-card",

                // Header Bar
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,

                    gtk::Label {
                        set_label: "👁 Quick Look",
                        add_css_class: "quicklook-header-title",
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                    },

                    gtk::Button {
                        set_label: "✕",
                        add_css_class: "quicklook-close-btn",
                        connect_clicked => QuickLookMsg::Close,
                    }
                },

                // Content Preview Container
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,
                    set_hexpand: true,
                    add_css_class: "quicklook-preview-container",
                },

                // Footer Bar (Title, Size, Open With button)
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 16,
                    set_valign: gtk::Align::Center,
                    add_css_class: "quicklook-footer",

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_hexpand: true,

                        gtk::Label {
                            #[watch]
                            set_label: model.file_data.as_ref().map(|d| d.file_name.as_str()).unwrap_or("Nessun file selezionato"),
                            add_css_class: "quicklook-filename",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Label {
                            #[watch]
                            set_label: model.file_data.as_ref().map(|d| d.size_formatted.as_str()).unwrap_or("0 B"),
                            add_css_class: "quicklook-filesize",
                            set_halign: gtk::Align::Start,
                        }
                    },

                    gtk::Button {
                        set_label: "Apri con...",
                        add_css_class: "suggested-action",
                        add_css_class: "quicklook-openwith-btn",
                        connect_clicked => QuickLookMsg::OpenWith,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        ensure_quicklook_css();

        root.init_layer_shell();
        root.set_layer(Layer::Overlay);
        root.set_namespace("quicklook");
        root.set_keyboard_mode(KeyboardMode::Exclusive);

        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Bottom, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);

        let file_data = init.map(QuickLookFileData::from_path);
        let visible = file_data.is_some();

        let model = QuickLookModel { file_data, visible };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            QuickLookMsg::OpenFile(path) => {
                self.file_data = Some(QuickLookFileData::from_path(path));
                self.visible = true;
            }
            QuickLookMsg::OpenWith => {
                if let Some(ref data) = self.file_data {
                    open_file_with_default(&data.path);
                }
                self.visible = false;
            }
            QuickLookMsg::Close => {
                self.visible = false;
            }
        }
    }
}

pub fn show_quicklook_modal(app: &Application, path: &Path) {
    ensure_quicklook_css();
    let data = QuickLookFileData::from_path(path);

    let window = ApplicationWindow::builder()
        .application(app)
        .title(format!("Quick Look - {}", data.file_name))
        .css_classes(["quicklook-window", "glassmorphism"])
        .default_width(720)
        .default_height(540)
        .build();

    window.init_layer_shell();
    window.set_namespace("quicklook");
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    let key_controller = EventControllerKey::new();
    let win_weak = window.downgrade();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk4::gdk::Key::Escape {
            if let Some(w) = win_weak.upgrade() {
                w.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    let container = GtkBox::new(Orientation::Vertical, 16);
    container.set_valign(Align::Center);
    container.set_halign(Align::Center);
    container.add_css_class("quicklook-card");

    // Header bar
    let header_box = GtkBox::new(Orientation::Horizontal, 12);
    let title_lbl = Label::new(Some("👁 Quick Look"));
    title_lbl.add_css_class("quicklook-header-title");
    title_lbl.set_halign(Align::Start);
    title_lbl.set_hexpand(true);

    let close_btn = Button::with_label("✕");
    close_btn.add_css_class("quicklook-close-btn");
    let win_close = window.downgrade();
    close_btn.connect_clicked(move |_| {
        if let Some(w) = win_close.upgrade() {
            w.close();
        }
    });

    header_box.append(&title_lbl);
    header_box.append(&close_btn);
    container.append(&header_box);

    // Preview Body (gtk4::Picture or ScrolledWindow text area or icon)
    let preview_box = GtkBox::new(Orientation::Vertical, 0);
    preview_box.set_vexpand(true);
    preview_box.set_hexpand(true);
    preview_box.add_css_class("quicklook-preview-container");

    match &data.content_type {
        QuickLookContentType::Image => {
            let picture = Picture::for_filename(&data.path);
            picture.set_can_shrink(true);
            picture.set_vexpand(true);
            picture.set_hexpand(true);
            picture.add_css_class("quicklook-picture");
            preview_box.append(&picture);
        }
        QuickLookContentType::Text(text) => {
            let scrolled = ScrolledWindow::builder()
                .hscrollbar_policy(gtk4::PolicyType::Automatic)
                .vscrollbar_policy(gtk4::PolicyType::Automatic)
                .min_content_height(320)
                .min_content_width(600)
                .vexpand(true)
                .hexpand(true)
                .build();
            scrolled.add_css_class("quicklook-scrolled-text");

            let buffer = TextBuffer::builder().text(text).build();
            let text_view = TextView::builder()
                .buffer(&buffer)
                .editable(false)
                .monospace(true)
                .cursor_visible(false)
                .wrap_mode(gtk4::WrapMode::WordChar)
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .build();
            text_view.add_css_class("quicklook-textview");
            scrolled.set_child(Some(&text_view));
            preview_box.append(&scrolled);
        }
        QuickLookContentType::BinaryOrOther => {
            let fallback_box = GtkBox::new(Orientation::Vertical, 12);
            fallback_box.set_valign(Align::Center);
            fallback_box.set_halign(Align::Center);
            fallback_box.set_vexpand(true);

            let icon = Image::from_icon_name("document-open-symbolic");
            icon.set_pixel_size(96);
            icon.add_css_class("quicklook-fallback-icon");

            let lbl = Label::new(Some("Nessuna anteprima disponibile per questo formato"));
            lbl.add_css_class("quicklook-fallback-label");

            fallback_box.append(&icon);
            fallback_box.append(&lbl);
            preview_box.append(&fallback_box);
        }
    }

    container.append(&preview_box);

    // Footer Bar (Title, Size, "Apri con..." button)
    let footer_box = GtkBox::new(Orientation::Horizontal, 16);
    footer_box.set_valign(Align::Center);
    footer_box.add_css_class("quicklook-footer");

    let info_box = GtkBox::new(Orientation::Vertical, 2);
    info_box.set_hexpand(true);

    let filename_lbl = Label::new(Some(&data.file_name));
    filename_lbl.add_css_class("quicklook-filename");
    filename_lbl.set_halign(Align::Start);

    let filesize_lbl = Label::new(Some(&data.size_formatted));
    filesize_lbl.add_css_class("quicklook-filesize");
    filesize_lbl.set_halign(Align::Start);

    info_box.append(&filename_lbl);
    info_box.append(&filesize_lbl);
    footer_box.append(&info_box);

    let open_btn = Button::with_label("Apri con...");
    open_btn.add_css_class("suggested-action");
    open_btn.add_css_class("quicklook-openwith-btn");

    let file_path_clone = data.path.clone();
    let win_open = window.downgrade();
    open_btn.connect_clicked(move |_| {
        if let Some(w) = win_open.upgrade() {
            w.close();
        }
        open_file_with_default(&file_path_clone);
    });

    footer_box.append(&open_btn);
    container.append(&footer_box);

    window.set_child(Some(&container));
    crate::ui::popup_manager::setup_popup_autoclose(&window, "quicklook");

    window.present();
}
