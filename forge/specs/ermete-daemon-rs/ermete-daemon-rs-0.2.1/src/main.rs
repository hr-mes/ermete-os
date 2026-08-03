
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod bedrock;
mod network;
mod bluetooth;
mod settings;
mod portal;
mod portal_screencast;
mod secret_enroller;
mod gatekeeper_listener;
mod voiceover;
mod qos;
mod power;

use std::error::Error;
use zbus::connection::Builder;
use bedrock::Bedrock;
use network::Network;
use bluetooth::Bluetooth;
use settings::SettingsService;
use portal::PortalSettingsService;
use portal_screencast::{PortalScreenCastService, PortalRemoteDesktopService};
use secret_enroller::SecretEnrollerService;
use voiceover::VoiceOverService;

use power::PowerManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to system D-Bus for NetworkManager & BlueZ integration...");
    let sys_conn = zbus::Connection::system().await?;

    println!("Starting PowerManager...");
    let power_manager = PowerManager::new();
    power_manager.start_monitoring(sys_conn.clone()).await;

    println!("Starting Gatekeeper Listener...");
    let _ = gatekeeper_listener::start_gatekeeper_listener(sys_conn.clone()).await;



    println!("Starting Spatial Audio Raytracing engine...");


    println!("Starting Continuity & Handoff daemon...");



    println!("Initializing ACID Settings Engine and XDG Desktop Portal backend...");
    let settings_srv = SettingsService::new();
    let portal_srv = PortalSettingsService::new(settings_srv.state.clone());
    let screencast_srv = PortalScreenCastService::new();
    let remotedesktop_srv = PortalRemoteDesktopService::new(screencast_srv.clone());
    let voiceover_srv = VoiceOverService::new(settings_srv.state.clone());

    println!("Starting Ermete Bedrock Session Daemon on /os/ermete/Bedrock & /org/ermete/Settings...");
    let _conn = Builder::session()?
        .name("os.ermete.Bedrock")?
        .name("org.ermete.Settings")?
        .name("os.ermete.VoiceOver")?
        .name("org.freedesktop.impl.portal.desktop.ermete")?
        .serve_at("/os/ermete/Bedrock", Bedrock::new())?
        .serve_at("/os/ermete/Bedrock/Network", Network::new(sys_conn.clone()))?
        .serve_at("/os/ermete/Bedrock/Bluetooth", Bluetooth::new(sys_conn.clone()))?
        .serve_at("/os/ermete/Bedrock/SecretEnroller", SecretEnrollerService::new())?
        .serve_at("/org/ermete/Settings", settings_srv.clone())?
        .serve_at("/os/ermete/Bedrock/Settings", settings_srv)?
        .serve_at("/os/ermete/VoiceOver", voiceover_srv)?
        .serve_at("/org/freedesktop/portal/desktop", portal_srv)?
        .serve_at("/org/freedesktop/portal/desktop", screencast_srv)?
        .serve_at("/org/freedesktop/portal/desktop", remotedesktop_srv)?
        .build()
        .await?;

    println!("Ermete Bedrock & Settings Daemon started and serving natively over zbus.");
    std::future::pending::<()>().await;
    Ok(())
}

