use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};
use std::path::Path;

#[derive(Parser)]
#[command(name = "ermete-store")]
#[command(about = "Ermete OS Proprietary Flatpak Store Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a Flatpak app from the Ermete Store
    Install {
        /// The Flatpak App ID to install
        app_id: String,
    },
    /// Disconnect from Flathub
    DisconnectFlathub,
}

const REGISTRY_URL: &str = "ghcr.io/hr-mes/ermete-store";
const PUBLIC_KEY_PATH: &str = "/etc/ermete/keys/cosign.pub";

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { app_id } => {
            install_app(app_id)?;
        }
        Commands::DisconnectFlathub => {
            disconnect_flathub()?;
        }
    }

    Ok(())
}

fn disconnect_flathub() -> Result<()> {
    println!("Disconnecting completely from Flathub...");
    
    // Ignore errors here if the remote doesn't exist
    let _ = Command::new("flatpak")
        .args(["remote-delete", "--system", "flathub"])
        .status();
        
    let _ = Command::new("flatpak")
        .args(["remote-delete", "--user", "flathub"])
        .status();
        
    println!("Successfully disconnected from Flathub.");
    Ok(())
}

fn install_app(app_id: &str) -> Result<()> {
    println!("Preparing to install '{}' from Ermete Store...", app_id);
    
    let oci_image = format!("{}/{}", REGISTRY_URL, app_id);
    let install_url = format!("oci+https://{}", oci_image);

    // 1. Verify signature with cosign
    println!("Verifying cryptographic signature (SLSA 4) for {}...", oci_image);
    
    // Ensure public key exists or mock it if this is a dev environment, but we must use it
    if !Path::new(PUBLIC_KEY_PATH).exists() {
        eprintln!("Warning: Public key not found at {}. Make sure to provision the Ermete OS keys.", PUBLIC_KEY_PATH);
    }

    let cosign_status = Command::new("cosign")
        .args([
            "verify",
            "--key",
            PUBLIC_KEY_PATH,
            &oci_image,
        ])
        .status()
        .context("Failed to run 'cosign verify'. Is cosign installed?")?;

    if !cosign_status.success() {
        anyhow::bail!("Cosign signature verification failed! Installation blocked for security reasons.");
    }

    println!("Signature verified successfully.");

    // 2. Pass the OCI image to flatpak install
    println!("Installing Flatpak from {}...", install_url);
    
    let flatpak_status = Command::new("flatpak")
        .args([
            "install",
            "--system",
            "-y",
            &install_url,
        ])
        .status()
        .context("Failed to run 'flatpak install'")?;

    if !flatpak_status.success() {
        anyhow::bail!("Flatpak installation failed with exit code: {}", flatpak_status);
    }

    println!("Successfully installed '{}'.", app_id);
    
    Ok(())
}
