use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, Box as GtkBox, Button, Label,
    Orientation, Spinner, Frame,
};
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use std::sync::Once;
use tracing::info;

static FORGE_CSS_INIT: Once = Once::new();

pub fn ensure_forge_css() {
    FORGE_CSS_INIT.call_once(|| {
        if let Some(display) = gtk4::gdk::Display::default() {
            let provider = gtk4::CssProvider::new();
            let css = r#"
                .forge-window {
                    background-color: rgba(15, 15, 23, 0.75);
                }
                .forge-card {
                    background-color: rgba(24, 25, 38, 0.88);
                    backdrop-filter: blur(28px);
                    border: 1px solid rgba(255, 255, 255, 0.12);
                    border-radius: 20px;
                    padding: 24px;
                    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
                    min-width: 540px;
                }
                .forge-title {
                    font-size: 20px;
                    font-weight: 800;
                    color: #cdd6f4;
                }
                .forge-subtitle {
                    font-size: 13px;
                    color: #a6adc8;
                }
                .forge-hw-chip {
                    background-color: rgba(30, 32, 48, 0.9);
                    border: 1px solid rgba(137, 180, 250, 0.3);
                    border-radius: 12px;
                    padding: 10px 14px;
                }
                .forge-hw-label {
                    font-family: monospace;
                    font-size: 13px;
                    font-weight: bold;
                    color: #89b4fa;
                }
                .forge-status-box {
                    background-color: rgba(17, 17, 27, 0.6);
                    border: 1px solid rgba(255, 255, 255, 0.08);
                    border-radius: 14px;
                    padding: 16px;
                }
                .status-verifying {
                    color: #f9e2af;
                    font-weight: 700;
                }
                .status-downloading {
                    color: #89b4fa;
                    font-weight: 700;
                }
                .status-compiling {
                    color: #fab387;
                    font-weight: 700;
                }
                .status-idle {
                    color: #a6adc8;
                    font-weight: 600;
                }
                .status-ready {
                    color: #a6e3a1;
                    font-weight: 700;
                }
                .forge-mode-btn {
                    border-radius: 10px;
                    padding: 8px 14px;
                    font-weight: 600;
                    background-color: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    color: #cdd6f4;
                }
                .forge-mode-btn.active {
                    background-color: rgba(137, 180, 250, 0.25);
                    border-color: #89b4fa;
                    color: #89b4fa;
                }
                .forge-action-btn {
                    border-radius: 10px;
                    padding: 6px 12px;
                    font-size: 12px;
                    background-color: rgba(255, 255, 255, 0.08);
                    color: #cdd6f4;
                }
                .forge-action-btn:hover {
                    background-color: rgba(255, 255, 255, 0.15);
                }
            "#;
            provider.load_from_data(css);
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeMode {
    Cloud,
    Local,
    Auto,
}

impl std::fmt::Display for ForgeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForgeMode::Cloud => write!(f, "Cloud Forge (GHCR Cache)"),
            ForgeMode::Local => write!(f, "Local Forge (crosvm / BuildKit)"),
            ForgeMode::Auto => write!(f, "Auto (Cloud First -> Local Fallback)"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FalState {
    VerifyingGhcrCache,
    DownloadingCloudKernel { progress: u32 },
    LocalCompilationIdleMode { cpu_usage: u32 },
    Idle,
    SyncedReady,
}

impl FalState {
    pub fn status_text(&self) -> String {
        match self {
            FalState::VerifyingGhcrCache => "Verifica Cache GHCR...".to_string(),
            FalState::DownloadingCloudKernel { progress } => {
                format!("Kernel in download dal Cloud ({}%)", progress)
            }
            FalState::LocalCompilationIdleMode { cpu_usage } => {
                format!("Compilazione locale in corso (Idle Mode - CPU: {}%)", cpu_usage)
            }
            FalState::Idle => "Idle Mode - In attesa di nuovi job".to_string(),
            FalState::SyncedReady => "Kernel Synced & Verified (SHA-256 Valid)".to_string(),
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            FalState::VerifyingGhcrCache => "status-verifying",
            FalState::DownloadingCloudKernel { .. } => "status-downloading",
            FalState::LocalCompilationIdleMode { .. } => "status-compiling",
            FalState::Idle => "status-idle",
            FalState::SyncedReady => "status-ready",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FalState::VerifyingGhcrCache => "🔍",
            FalState::DownloadingCloudKernel { .. } => "☁️",
            FalState::LocalCompilationIdleMode { .. } => "⚙️",
            FalState::Idle => "💤",
            FalState::SyncedReady => "✅",
        }
    }
}

pub struct ForgeWidgetModel {
    pub visible: bool,
    pub hw_hash: String,
    pub mode: ForgeMode,
    pub fal_state: FalState,
    pub last_sync_time: String,
}

#[derive(Debug)]
pub enum ForgeWidgetInput {
    ToggleVisible,
    SetMode(ForgeMode),
    SetState(FalState),
    SimulateVerifyGhcr,
    SimulateDownloadCloud,
    SimulateLocalCompile,
    RefreshHwHash,
}

#[relm4::component(pub)]
impl SimpleComponent for ForgeWidgetModel {
    type Input = ForgeWidgetInput;
    type Output = ();
    type Init = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Ermete OS - Forge Abstraction Layer (FAL)"),
            add_css_class: "forge-window",
            set_default_width: 580,
            set_default_height: 480,
            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 16,
                set_margin_top: 20,
                set_margin_bottom: 20,
                set_margin_start: 20,
                set_margin_end: 20,
                add_css_class: "forge-card",

                // Header
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        set_hexpand: true,

                        gtk::Label {
                            set_label: "🛠️ Forge Abstraction Layer (FAL)",
                            add_css_class: "forge-title",
                            set_halign: gtk::Align::Start,
                        },
                        gtk::Label {
                            set_label: "Gestione ibrida Cloud Forge (GHCR) & Local Forge (MicroVM)",
                            add_css_class: "forge-subtitle",
                            set_halign: gtk::Align::Start,
                        },
                    },

                    gtk::Button {
                        set_label: "✕",
                        add_css_class: "quicklook-close-btn",
                        connect_clicked => ForgeWidgetInput::ToggleVisible,
                    }
                },

                // Hardware Hash Badge
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    add_css_class: "forge-hw-chip",

                    gtk::Label {
                        set_label: "🔑 Hardware Hash:",
                        set_halign: gtk::Align::Start,
                        add_css_class: "forge-subtitle",
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &model.hw_hash,
                        add_css_class: "forge-hw-label",
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Button {
                        set_label: "🔄 Aggiorna",
                        add_css_class: "forge-action-btn",
                        connect_clicked => ForgeWidgetInput::RefreshHwHash,
                    }
                },

                // Forge Mode Selection (Cloud vs Local vs Auto)
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,

                    gtk::Label {
                        set_label: "Modalità Forge Attiva",
                        add_css_class: "forge-subtitle",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        set_homogeneous: true,

                        gtk::Button {
                            set_label: "☁️ Cloud Forge",
                            add_css_class: "forge-mode-btn",
                            #[watch]
                            set_css_classes: if model.mode == ForgeMode::Cloud { &["forge-mode-btn", "active"] } else { &["forge-mode-btn"] },
                            connect_clicked => ForgeWidgetInput::SetMode(ForgeMode::Cloud),
                        },

                        gtk::Button {
                            set_label: "💻 Local Forge",
                            add_css_class: "forge-mode-btn",
                            #[watch]
                            set_css_classes: if model.mode == ForgeMode::Local { &["forge-mode-btn", "active"] } else { &["forge-mode-btn"] },
                            connect_clicked => ForgeWidgetInput::SetMode(ForgeMode::Local),
                        },

                        gtk::Button {
                            set_label: "⚡ Auto (Cloud/Local)",
                            add_css_class: "forge-mode-btn",
                            #[watch]
                            set_css_classes: if model.mode == ForgeMode::Auto { &["forge-mode-btn", "active"] } else { &["forge-mode-btn"] },
                            connect_clicked => ForgeWidgetInput::SetMode(ForgeMode::Auto),
                        },
                    }
                },

                // FAL Live Status Display Card
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                    add_css_class: "forge-status-box",

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,

                        gtk::Label {
                            #[watch]
                            set_label: model.fal_state.icon(),
                        },

                        gtk::Label {
                            #[watch]
                            set_label: &model.fal_state.status_text(),
                            #[watch]
                            set_css_classes: &[model.fal_state.css_class()],
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Spinner {
                            #[watch]
                            set_spinning: matches!(model.fal_state, FalState::VerifyingGhcrCache | FalState::DownloadingCloudKernel { .. } | FalState::LocalCompilationIdleMode { .. }),
                            #[watch]
                            set_visible: matches!(model.fal_state, FalState::VerifyingGhcrCache | FalState::DownloadingCloudKernel { .. } | FalState::LocalCompilationIdleMode { .. }),
                        }
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &format!("Ultimo aggiornamento FAL: {}", model.last_sync_time),
                        add_css_class: "forge-subtitle",
                        set_halign: gtk::Align::Start,
                    }
                },

                // Simulation Controls for FAL testing
                gtk::Frame {
                    set_label: Some("Simulazione Mock FAL (Forge Abstraction Layer)"),
                    
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        set_margin_top: 10,
                        set_margin_bottom: 10,
                        set_margin_start: 10,
                        set_margin_end: 10,
                        set_homogeneous: true,

                        gtk::Button {
                            set_label: "1. Cache GHCR",
                            add_css_class: "forge-action-btn",
                            connect_clicked => ForgeWidgetInput::SimulateVerifyGhcr,
                        },

                        gtk::Button {
                            set_label: "2. Cloud Download",
                            add_css_class: "forge-action-btn",
                            connect_clicked => ForgeWidgetInput::SimulateDownloadCloud,
                        },

                        gtk::Button {
                            set_label: "3. Local Compile",
                            add_css_class: "forge-action-btn",
                            connect_clicked => ForgeWidgetInput::SimulateLocalCompile,
                        },
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        ensure_forge_css();

        let model = ForgeWidgetModel {
            visible: true,
            hw_hash: detect_hardware_hash(),
            mode: ForgeMode::Auto,
            fal_state: FalState::VerifyingGhcrCache,
            last_sync_time: chrono::Local::now().format("%H:%M:%S").to_string(),
        };

        let widgets = view_output!();

        // Removed mock state transition timer
        sender.input(ForgeWidgetInput::SetState(FalState::Idle));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        self.last_sync_time = chrono::Local::now().format("%H:%M:%S").to_string();
        match msg {
            ForgeWidgetInput::ToggleVisible => {
                self.visible = !self.visible;
            }
            ForgeWidgetInput::SetMode(mode) => {
                info!("FAL Forge mode changed to: {:?}", mode);
                self.mode = mode;
            }
            ForgeWidgetInput::SetState(state) => {
                info!("FAL status state updated to: {:?}", state);
                self.fal_state = state;
            }
            ForgeWidgetInput::SimulateVerifyGhcr => {
                self.fal_state = FalState::VerifyingGhcrCache;
            }
            ForgeWidgetInput::SimulateDownloadCloud => {
                self.fal_state = FalState::DownloadingCloudKernel { progress: 72 };
            }
            ForgeWidgetInput::SimulateLocalCompile => {
                self.fal_state = FalState::LocalCompilationIdleMode { cpu_usage: 24 };
            }
            ForgeWidgetInput::RefreshHwHash => {
                self.hw_hash = detect_hardware_hash();
            }
        }
    }
}

/// Helper function to detect local hardware hash (or generate realistic mock if sys info unavailable)
pub fn detect_hardware_hash() -> String {
    use sha2::{Digest, Sha256};
    
    // Read machine-id or boot_id to generate realistic hardware-bound hash
    let raw_id = std::fs::read_to_string("/etc/machine-id")
        .or_else(|_| std::fs::read_to_string("/proc/sys/kernel/random/boot_id"))
        .unwrap_or_else(|_| "ermete-default-hw-node-01".to_string());

    let mut hasher = Sha256::new();
    hasher.update(raw_id.as_bytes());
    let result = hasher.finalize();
    format!("hw_hash_{}", &hex::encode(result)[..16])
}

/// Helper function to construct a GTK Box representation of Forge Status for embedding in Control Center
pub fn create_forge_cc_tile() -> GtkBox {
    ensure_forge_css();

    let box_ = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build();

    box_.add_css_class("forge-status-box");

    let title_lbl = Label::builder()
        .label("🛠️ Forge Abstraction Layer (FAL)")
        .css_classes(["forge-title"])
        .halign(Align::Start)
        .build();

    let hw_lbl = Label::builder()
        .label(&format!("Hash: {}", detect_hardware_hash()))
        .css_classes(["forge-hw-label"])
        .halign(Align::Start)
        .build();

    let status_lbl = Label::builder()
        .label("Verifica Cache GHCR...")
        .css_classes(["status-verifying"])
        .halign(Align::Start)
        .build();

    box_.append(&title_lbl);
    box_.append(&hw_lbl);
    box_.append(&status_lbl);

    box_
}
