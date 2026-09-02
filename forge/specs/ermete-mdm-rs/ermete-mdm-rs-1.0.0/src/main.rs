//! Ermete OS MDM (Mobile Device Management) daemon.
//!
//! Exposes the `os.ermete.Mdm` D-Bus system interface (see [`MdmDBusInterface`] in
//! this file and [`dbus::MdmIface`]) so an administrator-controlled MDM server (or
//! local UI, for the wipe path) can push a small set of device policies:
//!
//! - `disable_usb`: writes a modprobe blacklist for `usb-storage` and unloads the
//!   `usb_storage` kernel module, to lock down removable-media exfiltration.
//! - `force_vpn`: enables and starts the `openvpn-client@ermete.service` systemd unit.
//!
//! Every D-Bus call is gated by a Polkit authorization check
//! (`ermete_bus_api::polkit::check_polkit_auth_zbus`) before anything is applied. There is currently
//! no code in this crate that actually polls a remote MDM server on a schedule —
//! [`wipe::WipeEngine::poll_server`] exists but nothing calls it, so policy
//! application is push-only (driven by whoever calls the D-Bus method), not a
//! background pull loop.
//!
//! `dbus.rs` additionally defines a second, unused `os.ermete.Mdm` interface type
//! ([`dbus::MdmIface`]) exposing a `trigger_local_wipe` method that triggers a
//! destructive LUKS-header wipe via [`wipe::WipeEngine`] — this daemon's `main()`
//! only ever builds and serves the [`MdmDBusInterface`] defined in this file, so
//! `MdmIface`/`trigger_local_wipe` is currently dead code, not a reachable action.

mod dbus;
mod wipe;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use serde::Deserialize;
use tokio::process::Command;
use tracing::{error, info, warn};
use zbus::{connection::Builder, interface, object_server::SignalEmitter};

#[derive(Deserialize, Debug)]
struct MdmPayload {
    action: String,
}

struct MdmDBusInterface;

impl MdmDBusInterface {
    /// Blacklists the `usb-storage` kernel module (writes
    /// `/etc/modprobe.d/disable-usb-storage.conf`) and unloads it via `rmmod`.
    /// Returns `false` if the modprobe config couldn't be written; the `rmmod`
    /// result is not checked (module may already be in use or absent).
    async fn disable_usb(&self) -> bool {
        info!("Disabling USB storage...");
        // Applying the policy directly to disk via non-blocking I/O
        let res = async {
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open("/etc/modprobe.d/disable-usb-storage.conf")
                .await?;
            tokio::io::AsyncWriteExt::write_all(&mut file, b"install usb-storage /bin/true\n").await
        }.await;
        if let Err(e) = res {
            error!("Failed to write modprobe config: {}", e);
            return false;
        }

        // Execute system command asynchronously
        let _ = Command::new("rmmod").arg("usb_storage").output().await;

        true
    }

    /// Enables and starts `openvpn-client@ermete.service` via `systemctl`.
    /// Returns `false` if the command couldn't run or exited non-zero.
    async fn force_vpn(&self) -> bool {
        info!("Forcing VPN...");
        let output = Command::new("systemctl")
            .args(["enable", "--now", "openvpn-client@ermete.service"])
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

#[interface(name = "os.ermete.Mdm")]
impl MdmDBusInterface {
    /// D-Bus method backing `os.ermete.Mdm.apply_policy`.
    ///
    /// Requires Polkit authorization for the `os.ermete.mdm.apply_policy` action
    /// (via `ermete_bus_api::polkit::check_polkit_auth_zbus`). `payload_json` is
    /// deserialized into
    /// [`MdmPayload`] and dispatched to [`Self::disable_usb`] or
    /// [`Self::force_vpn`] based on the `action` field; any other action name is
    /// rejected. Emits the `policy_applied` signal on success.
    ///
    /// # Errors
    /// Returns `AccessDenied` if the caller has no D-Bus sender or fails the
    /// Polkit check, `InvalidArgs` if `payload_json` isn't valid JSON for
    /// [`MdmPayload`], and `Failed` if the requested action's handler reports
    /// failure.
    async fn apply_policy(
        &self,
        payload_json: &str,
        #[zbus(signal_emitter)] ctxt: SignalEmitter<'_>,
        #[zbus(header)] hdr: zbus::message::Header<'_>,
        #[zbus(connection)] conn: &zbus::Connection,
    ) -> zbus::fdo::Result<String> {
        info!("Received policy payload: {}", payload_json);

        let sender = hdr.sender().ok_or(zbus::fdo::Error::AccessDenied("No sender".into()))?;
        let is_auth = ermete_bus_api::polkit::check_polkit_auth_zbus(conn, sender.as_str(), "os.ermete.mdm.apply_policy", true)
            .await
            .map_err(|e| zbus::fdo::Error::AccessDenied(format!("Polkit check failed: {}", e)))?;

        if !is_auth {
            return Err(zbus::fdo::Error::AccessDenied("Polkit authorization failed for apply_policy".into()));
        }

        let payload: Result<MdmPayload, serde_json::Error> = serde_json::from_str(payload_json);
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
                    let _ = Self::policy_applied(&ctxt, &p.action).await;
                    Ok(format!("Policy {} applied successfully.", p.action))
                } else {
                    error!("Action {} failed", p.action);
                    Err(zbus::fdo::Error::Failed(format!("Policy {} execution failed.", p.action)))
                }
            }
            Err(e) => {
                error!("Invalid payload: {}", e);
                Err(zbus::fdo::Error::InvalidArgs(format!("Invalid payload JSON: {}", e)))
            }
        }
    }

    #[zbus(signal)]
    async fn policy_applied(ctxt: &SignalEmitter<'_>, action: &str) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting ermete-mdm-rs via DBus...");

    let mdm_interface = MdmDBusInterface;

    // Set up DBus connection for the interface
    let _conn = Builder::system()?
        .name("os.ermete.Mdm")?
        .serve_at("/os/ermete/Mdm", mdm_interface)?
        .build()
        .await?;

    info!("DBus interface os.ermete.Mdm is ready.");

    // Keep the daemon alive and listen for exit signals
    let mut exit_sig =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut int_sig =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

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
