use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::process::Command;
use zbus::{connection::Builder, interface};
use tracing::{info, error, warn};

#[derive(Deserialize, Debug)]
struct MdmPayload {
    action: String,
}

#[derive(Debug)]
enum ActorMessage {
    ProcessPayload(String),
}

struct SystemActor {
    receiver: mpsc::Receiver<ActorMessage>,
    notify_sender: mpsc::Sender<String>,
}

impl SystemActor {
    fn new(receiver: mpsc::Receiver<ActorMessage>, notify_sender: mpsc::Sender<String>) -> Self {
        Self { receiver, notify_sender }
    }

    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                ActorMessage::ProcessPayload(payload_str) => {
                    self.handle_payload(&payload_str).await;
                }
            }
        }
    }

    async fn handle_payload(&self, payload_str: &str) {
        let payload: Result<MdmPayload, serde_json::Error> = serde_json::from_str(payload_str);
        match payload {
            Ok(p) => {
                let success = match p.action.as_str() {
                    "disable_usb" => self.disable_usb().await,
                    "force_vpn" => self.force_vpn().await,
                    _ => {
                        warn!("Unknown action: {}", p.action);
                        false
                    }
                };
                
                if success {
                    info!("Action {} applied successfully", p.action);
                    let _ = self.notify_sender.send(format!("SUCCESS: {}", p.action)).await;
                } else {
                    error!("Action {} failed", p.action);
                    let _ = self.notify_sender.send(format!("FAILURE: {}", p.action)).await;
                }
            }
            Err(e) => {
                error!("Invalid payload: {}", e);
            }
        }
    }

    async fn disable_usb(&self) -> bool {
        info!("Disabling USB storage...");
        // Applying the policy directly to disk via non-blocking I/O
        let res = tokio::fs::write("/etc/modprobe.d/disable-usb-storage.conf", "install usb-storage /bin/true\n").await;
        if res.is_err() {
            error!("Failed to write modprobe config");
            return false;
        }
        
        // Execute system command asynchronously
        let _ = Command::new("rmmod")
            .arg("usb_storage")
            .output()
            .await;
            
        true
    }

    async fn force_vpn(&self) -> bool {
        info!("Forcing VPN...");
        let output = Command::new("systemctl")
            .args(&["enable", "--now", "openvpn-client@ermete.service"])
            .output()
            .await;
            
        match output {
            Ok(out) => out.status.success(),
            Err(e) => {
                error!("Failed to execute systemctl: {}", e);
                false
            }
        }
    }
}

struct MdmDBusInterface {
    sender: mpsc::Sender<ActorMessage>,
}

#[interface(name = "os.ermete.Mdm")]
impl MdmDBusInterface {
    async fn apply_policy(&self, payload_json: &str) -> String {
        info!("Received policy payload: {}", payload_json);
        if self.sender.send(ActorMessage::ProcessPayload(payload_json.to_string())).await.is_ok() {
            "Policy received and queued for processing.".to_string()
        } else {
            "Failed to queue policy.".to_string()
        }
    }

    #[zbus(signal)]
    async fn policy_applied(ctxt: &zbus::SignalContext<'_>, action: &str) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting ermete-mdm-rs Actor Model via DBus...");

    let (tx, rx) = mpsc::channel(32);
    let (notify_tx, mut notify_rx) = mpsc::channel(32);

    let actor = SystemActor::new(rx, notify_tx);
    
    // Spawn the core actor task to process commands
    tokio::spawn(async move {
        actor.run().await;
    });

    let mdm_interface = MdmDBusInterface { sender: tx };

    // Set up DBus connection for the interface
    let _conn = Builder::system()?
        .name("os.ermete.Mdm")?
        .serve_at("/os/ermete/Mdm", mdm_interface)?
        .build()
        .await?;

    info!("DBus interface os.ermete.Mdm is ready.");

    // Retrieve the SignalContext to emit signals across DBus
    let iface_ref = _conn.object_server().interface::<_, MdmDBusInterface>("/os/ermete/Mdm").await?;
    let signal_context = iface_ref.signal_context().clone();

    // A separate async task to handle MPSC notifications and forward to DBus
    tokio::spawn(async move {
        while let Some(msg) = notify_rx.recv().await {
            info!("Notification on MPSC channel: {}", msg);
            if msg.starts_with("SUCCESS: ") {
                let action = msg.strip_prefix("SUCCESS: ").unwrap();
                if let Err(e) = MdmDBusInterface::policy_applied(&signal_context, action).await {
                    error!("Failed to emit DBus signal: {}", e);
                }
            }
        }
    });

    // Keep the daemon alive and listen for exit signals
    let mut exit_sig = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut int_sig = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

    tokio::select! {
        _ = exit_sig.recv() => {
            info!("Received SIGTERM, shutting down.");
        }
        _ = int_sig.recv() => {
            info!("Received SIGINT, shutting down.");
        }
    }

    Ok(())
}
