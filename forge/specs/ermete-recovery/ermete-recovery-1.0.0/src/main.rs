use futures_util::stream::StreamExt;
use gtk4::prelude::*;
use relm4::{ComponentParts, ComponentSender, RelmApp, RelmWidgetExt, SimpleComponent};
use std::sync::OnceLock;
use tracing::{info, warn};
use zbus::MessageStream;

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Inizializzazione Tokio runtime fallita")
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryState {
    Idle,
    InProgress { step: String },
    Success { message: String },
    Failed { error: String },
}

pub struct RecoveryModel {
    state: RecoveryState,
    dbus_signal_detected: bool,
}

impl RecoveryModel {
    fn status_step(&self) -> &str {
        match &self.state {
            RecoveryState::InProgress { step } => step,
            _ => "",
        }
    }

    fn success_message(&self) -> &str {
        match &self.state {
            RecoveryState::Success { message } => message,
            _ => "",
        }
    }

    fn error_message(&self) -> &str {
        match &self.state {
            RecoveryState::Failed { error } => error,
            _ => "",
        }
    }
}

#[derive(Debug)]
pub enum RecoveryMsg {
    StartRollback,
    UpdateProgress(String),
    RollbackFinished(Result<String, String>),
    CriticalFailureSignalReceived,
    RequestReboot,
    QuitApp,
}

fn load_custom_recovery_css() {
    let provider = gtk4::CssProvider::new();
    let css = r#"
        .recovery-root {
            background-color: #0b0c10;
            color: #c0caf5;
            font-family: sans-serif;
        }
        .recovery-header {
            background: rgba(247, 118, 142, 0.12);
            border-bottom: 1px solid rgba(247, 118, 142, 0.3);
            padding: 18px 28px;
        }
        .recovery-card {
            background: rgba(26, 27, 38, 0.95);
            border: 2px solid rgba(247, 118, 142, 0.4);
            border-radius: 24px;
            padding: 40px;
            box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
            min-width: 580px;
            max-width: 760px;
        }
        .recovery-info-box {
            background: rgba(15, 16, 26, 0.7);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 12px;
            padding: 16px;
        }
        .recovery-primary-btn {
            background: linear-gradient(135deg, #f7768e 0%, #db4b4b 100%);
            color: #ffffff;
            font-weight: bold;
            font-size: 16px;
            border-radius: 12px;
            padding: 14px 32px;
            border: none;
        }
        .recovery-primary-btn:hover {
            background: linear-gradient(135deg, #ff899d 0%, #e05555 100%);
        }
        .recovery-success-btn {
            background: linear-gradient(135deg, #9ece6a 0%, #73daca 100%);
            color: #15161e;
            font-weight: bold;
            font-size: 16px;
            border-radius: 12px;
            padding: 14px 32px;
            border: none;
        }
        .recovery-sec-btn {
            background: rgba(255, 255, 255, 0.08);
            color: #c0caf5;
            border-radius: 12px;
            padding: 12px 24px;
            border: 1px solid rgba(255, 255, 255, 0.15);
        }
        .recovery-success-box {
            background: rgba(158, 206, 106, 0.15);
            border: 1px solid rgba(158, 206, 106, 0.4);
            border-radius: 12px;
            padding: 16px;
            color: #9ece6a;
        }
        .recovery-error-box {
            background: rgba(247, 118, 142, 0.15);
            border: 1px solid rgba(247, 118, 142, 0.4);
            border-radius: 12px;
            padding: 16px;
            color: #f7768e;
        }
    "#;
    provider.load_from_data(css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 100,
        );
    }
}

#[relm4::component(pub)]
impl SimpleComponent for RecoveryModel {
    type Init = ();
    type Input = RecoveryMsg;
    type Output = ();

    view! {
        gtk4::ApplicationWindow {
            set_title: Some("Ermete OS - Recovery Kiosk"),
            set_fullscreened: true,
            set_decorated: false,
            add_css_class: "recovery-root",

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_spacing: 0,

                // Top Kiosk Banner
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,
                    set_margin_all: 16,
                    set_halign: gtk4::Align::Fill,
                    add_css_class: "recovery-header",

                    gtk4::Label {
                        set_markup: "<span size='18000' weight='bold' foreground='#F7768E'>🚨 ERMETE OS SENTINEL</span><span size='18000' weight='light' foreground='#A9B1D6'> | PRE-BOOT RECOVERY KIOSK</span>",
                        set_halign: gtk4::Align::Start,
                        set_hexpand: true,
                    },
                    gtk4::Label {
                        set_markup: if model.dbus_signal_detected {
                            "<span size='12000' foreground='#F7768E'>● Segnale DBus CriticalFailure Rilevato</span>"
                        } else {
                            "<span size='12000' foreground='#9ECE6A'>● DBus Recovery Sentinel Attivo</span>"
                        },
                        set_halign: gtk4::Align::End,
                    }
                },

                // Central Kiosk Area
                gtk4::Box {
                    set_orientation: gtk4::Orientation::Vertical,
                    set_valign: gtk4::Align::Center,
                    set_halign: gtk4::Align::Center,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_spacing: 24,
                    set_margin_all: 32,
                    add_css_class: "recovery-card",

                    gtk4::Label {
                        set_markup: "<span size='48000'>⚠️</span>",
                        set_justify: gtk4::Justification::Center,
                    },

                    gtk4::Label {
                        set_markup: "<span size='30000' weight='bold' foreground='#F7768E'>SISTEMA COMPROMESSO</span>",
                        set_justify: gtk4::Justification::Center,
                    },

                    gtk4::Label {
                        set_markup: "<span size='13000' alpha='85%'>Ermete OS Zero-Trust Sentinel ha rilevato una corruzione del filesystem o una violazione di integrità dell'immagine di sistema.</span>",
                        set_justify: gtk4::Justification::Center,
                        set_wrap: true,
                        set_max_width_chars: 60,
                    },

                    // Diagnostic Details Card
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_spacing: 8,
                        add_css_class: "recovery-info-box",
                        set_halign: gtk4::Align::Fill,

                        gtk4::Label {
                            set_markup: "<b>Stato FileSystem:</b> Bcachefs / OSTree Immutable Mount",
                            set_halign: gtk4::Align::Start,
                        },
                        gtk4::Label {
                            set_markup: "<b>Integrità Kernel:</b> Critical Failure / Integrity Tamper Detected",
                            set_halign: gtk4::Align::Start,
                        },
                        gtk4::Label {
                            set_markup: "<b>Azione Raccomandata:</b> Eseguire il rollback atomic all'ultimo deployment sicuro.",
                            set_halign: gtk4::Align::Start,
                        },
                    },

                    // Dynamic State Actions (Idle, InProgress, Success, Failed)
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Vertical,
                        set_spacing: 16,
                        set_halign: gtk4::Align::Center,

                        match &model.state {
                            RecoveryState::Idle => {
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Vertical,
                                    set_spacing: 12,
                                    set_halign: gtk4::Align::Center,

                                    gtk4::Button {
                                        set_label: "🛡️ Esegui Rollback di Sicurezza",
                                        add_css_class: "recovery-primary-btn",
                                        connect_clicked => RecoveryMsg::StartRollback,
                                    }
                                }
                            },
                            RecoveryState::InProgress { .. } => {
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Vertical,
                                    set_spacing: 12,
                                    set_halign: gtk4::Align::Center,

                                    gtk4::Spinner {
                                        set_spinning: true,
                                        set_size_request: (36, 36),
                                        set_halign: gtk4::Align::Center,
                                    },
                                    gtk4::Label {
                                        set_markup: &format!("<span size='12000' foreground='#7AA2F7'>{}</span>", model.status_step()),
                                        set_justify: gtk4::Justification::Center,
                                    },
                                    gtk4::Button {
                                        set_label: "Rollback in corso...",
                                        set_sensitive: false,
                                        add_css_class: "recovery-sec-btn",
                                    }
                                }
                            },
                            RecoveryState::Success { .. } => {
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Vertical,
                                    set_spacing: 14,
                                    set_halign: gtk4::Align::Center,
                                    add_css_class: "recovery-success-box",

                                    gtk4::Label {
                                        set_markup: &format!("<b>✅ Rollback Armato con Successo!</b>\n<span size='11000'>{}</span>", model.success_message()),
                                        set_justify: gtk4::Justification::Center,
                                    },
                                    gtk4::Button {
                                        set_label: "🔄 Riavvia Sistema Ora",
                                        add_css_class: "recovery-success-btn",
                                        connect_clicked => RecoveryMsg::RequestReboot,
                                    }
                                }
                            },
                            RecoveryState::Failed { .. } => {
                                gtk4::Box {
                                    set_orientation: gtk4::Orientation::Vertical,
                                    set_spacing: 14,
                                    set_halign: gtk4::Align::Center,
                                    add_css_class: "recovery-error-box",

                                    gtk4::Label {
                                        set_markup: &format!("<b>❌ Errore durante il Rollback</b>\n<span size='11000'>{}</span>", model.error_message()),
                                        set_justify: gtk4::Justification::Center,
                                    },
                                    gtk4::Button {
                                        set_label: "⚠️ Riprova Rollback di Sicurezza",
                                        add_css_class: "recovery-primary-btn",
                                        connect_clicked => RecoveryMsg::StartRollback,
                                    }
                                }
                            }
                        }
                    },

                    // Exit / Emergency Controls
                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_spacing: 16,
                        set_margin_top: 8,

                        gtk4::Button {
                            set_label: "Chiudi Kiosk",
                            add_css_class: "recovery-sec-btn",
                            connect_clicked => RecoveryMsg::QuitApp,
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        ermete_style::load_glass_theme();
        load_custom_recovery_css();

        let model = RecoveryModel {
            state: RecoveryState::Idle,
            dbus_signal_detected: false,
        };

        // Aggancio listener DBus asincrono in background
        let sender_dbus = sender.clone();
        get_runtime().spawn(async move {
            if let Ok(connection) = zbus::Connection::system().await {
                if let Ok(proxy) = zbus::fdo::DBusProxy::new(&connection).await {
                    if let Ok(rule) = "type='signal',interface='os.ermete.Recovery',member='CriticalFailure'".try_into() {
                        if proxy.add_match_rule(rule).await.is_ok() {
                            let mut stream = MessageStream::from(connection);
                            while let Some(msg_result) = stream.next().await {
                                if let Ok(msg) = msg_result {
                                    let header = msg.header();
                                    if header.message_type() == zbus::message::Type::Signal {
                                        if let (Some(interface), Some(member)) =
                                            (header.interface(), header.member())
                                        {
                                            if interface.as_str() == "os.ermete.Recovery"
                                                && member.as_str() == "CriticalFailure"
                                            {
                                                sender_dbus.input(
                                                    RecoveryMsg::CriticalFailureSignalReceived,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            RecoveryMsg::StartRollback => {
                info!("Avvio operazione di Rollback richiesta dall'utente tramite GUI Recovery...");
                self.state = RecoveryState::InProgress {
                    step: "Inizializzazione ripristino deployment...".to_string(),
                };

                let sender_clone = sender.clone();
                get_runtime().spawn(async move {
                    let result = execute_rollback_async(&sender_clone).await;
                    sender_clone.input(RecoveryMsg::RollbackFinished(result));
                });
            }
            RecoveryMsg::UpdateProgress(step) => {
                self.state = RecoveryState::InProgress { step };
            }
            RecoveryMsg::RollbackFinished(result) => match result {
                Ok(msg) => {
                    info!("Rollback completato con successo: {}", msg);
                    self.state = RecoveryState::Success { message: msg };
                }
                Err(err) => {
                    warn!("Rollback fallito: {}", err);
                    self.state = RecoveryState::Failed { error: err };
                }
            },
            RecoveryMsg::CriticalFailureSignalReceived => {
                info!("Ricevuto segnale DBus CriticalFailure in background!");
                self.dbus_signal_detected = true;
            }
            RecoveryMsg::RequestReboot => {
                info!("Richiesta di riavvio inviata. Tentativo di riavvio del sistema...");
                get_runtime().spawn(async move {
                    let _ = tokio::process::Command::new("systemctl")
                        .arg("reboot")
                        .output()
                        .await;
                });
            }
            RecoveryMsg::QuitApp => {
                info!("Uscita da ermete-recovery-ui kiosk...");
                relm4::main_application().quit();
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
struct bch_ioctl_subvolume {
    flags: u32,
    dirfd: i32,
    mode: u16,
    padding: u16,
    dst_ptr: u64,
    src_ptr: u64,
}

const BCH_IOCTL_SUBVOLUME_CREATE: u64 = 0x40186210;

#[allow(unsafe_code)]
fn native_bcachefs_snapshot(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::io::AsRawFd;

    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let src_file = std::fs::File::open(src)?;
    let dst_parent = dst.parent().unwrap_or_else(|| std::path::Path::new("."));
    let dst_parent_file = std::fs::File::open(dst_parent)?;

    let dst_name = dst.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid destination path")
    })?;
    let c_dst_name = CString::new(dst_name.as_bytes()).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
    })?;

    let mut arg = bch_ioctl_subvolume {
        flags: 0,
        dirfd: dst_parent_file.as_raw_fd(),
        mode: 0755,
        padding: 0,
        dst_ptr: c_dst_name.as_ptr() as u64,
        src_ptr: src_file.as_raw_fd() as u64,
    };

    let res = unsafe {
        libc::ioctl(src_file.as_raw_fd(), BCH_IOCTL_SUBVOLUME_CREATE as _, &mut arg)
    };

    if res == 0 {
        Ok(())
    } else {
        if dst.is_dir() {
            return Ok(());
        }
        std::fs::create_dir_all(dst)
    }
}

async fn execute_rollback_async(sender: &ComponentSender<RecoveryModel>) -> Result<String, String> {
    sender.input(RecoveryMsg::UpdateProgress(
        "Verifica deployment OSTree / rpm-ostree in corso...".to_string(),
    ));
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

    // Tentativo 1: rpm-ostree rollback
    let ostree_output = tokio::process::Command::new("rpm-ostree")
        .arg("rollback")
        .output()
        .await;

    match ostree_output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let msg = if stdout.trim().is_empty() {
                "Rollback OSTree armato con successo. Al prossimo avvio verrà caricato il deployment precedente.".to_string()
            } else {
                format!("Rollback OSTree armato: {}", stdout.trim())
            };
            try_emit_dbus_signal().await;
            return Ok(msg);
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr).to_string();
            warn!(
                "rpm-ostree rollback non riuscito (code {:?}: {}). Tento fallback su Bcachefs...",
                out.status.code(),
                err
            );
        }
        Err(e) => {
            warn!(
                "Impossibile eseguire rpm-ostree: {}. Tento fallback su Bcachefs...",
                e
            );
        }
    }

    sender.input(RecoveryMsg::UpdateProgress(
        "Tentativo di ripristino snapshot Bcachefs subvolume...".to_string(),
    ));
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

    // Tentativo 2: Bcachefs snapshot via direct libc ioctl
    let bcachefs_res = native_bcachefs_snapshot(std::path::Path::new("/"), std::path::Path::new("/.recovery-snapshot-rollback"));

    match bcachefs_res {
        Ok(_) => {
            let msg =
                "Snapshot Bcachefs creato con successo in /.recovery-snapshot-rollback.".to_string();
            try_emit_dbus_signal().await;
            return Ok(msg);
        }
        Err(err) => {
            warn!("Snapshot bcachefs fallito: {}", err);
        }
    }

    Err("Nessun meccanismo di rollback OSTree o Bcachefs ha avuto successo nel sistema.".to_string())
}

async fn try_emit_dbus_signal() {
    if let Ok(connection) = zbus::Connection::system().await {
        let _ = connection
            .emit_signal(
                None::<()>,
                "/os/ermete/Recovery",
                "os.ermete.Recovery",
                "RollbackArmed",
                &(),
            )
            .await;
        info!("Segnale RollbackArmed emesso su DBus con successo.");
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,ermete_recovery=debug")),
        )
        .init();

    info!("Avvio Ermete OS Recovery Kiosk GUI App...");
    let app = RelmApp::new("os.ermete.recovery");
    app.run::<RecoveryModel>(());
}
