
use anyhow::{Context, Result};

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

pub mod crdt;
pub mod storage;
use storage::{DatabaseEngine, DEFAULT_CQ_DEPTH, DEFAULT_SQ_DEPTH};

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
    /// Asynchronously write a DB snapshot in background via io_uring
    Snapshot,
    /// Synchronize DB state with io_uring storage engine
    SyncDb,
}

const REGISTRY_URL: &str = "ghcr.io/hr-mes/ermete-store";
const PUBLIC_KEY_PATH: &str = "/etc/ermete/keys/cosign.pub";
const PQC_PUBLIC_KEY_PATH: &str = "/etc/ermete/keys/dilithium5.pub";
const SIGNATURES_DIR: &str = "/etc/ermete/keys/signatures";
const DEFAULT_DB_PATH: &str = "/var/lib/ermete/store_db.json";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();
    let db_engine = Arc::new(DatabaseEngine::new(
        DEFAULT_SQ_DEPTH,
        DEFAULT_CQ_DEPTH,
        PathBuf::from(DEFAULT_DB_PATH),
    )?);

    match &cli.command {
        Commands::Install { app_id } => {
            install_app(app_id, &db_engine).await?;
        }
        Commands::DisconnectFlathub => {
            disconnect_flathub()?;
        }
        Commands::Snapshot => {
            let mut snapshot = db_engine.load_snapshot().await?;
            snapshot.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let handle = db_engine.write_snapshot_background(snapshot);
            handle.await.context("Background DB snapshot task panicked or failed")??;
        }
        Commands::SyncDb => {
            let snapshot = db_engine.load_snapshot().await?;
            println!(
                "Loaded Ermete Store DB Snapshot ({} packages registered).",
                snapshot.installed_packages.len()
            );
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

/// Consolidated helper to asynchronously read key/signature files using io_uring.
async fn read_key_or_sig_io_uring(
    db_engine: &DatabaseEngine,
    path: &Path,
    missing_msg: &str,
    err_context: &str,
) -> Result<Vec<u8>> {
    if !path.exists() {
        anyhow::bail!(
            "CRITICAL ZERO-TRUST ERROR: {}. Verification blocked.",
            missing_msg
        );
    }

    db_engine
        .read_file_io_uring(path)
        .await
        .context(err_context.to_string())
}

async fn verify_pqc_package_signature(
    app_id: &str,
    oci_image: &str,
    db_engine: &DatabaseEngine,
) -> Result<()> {
    println!(
        "Verifying Level 13 Post-Quantum Cryptographic signature (Dilithium5 ML-DSA) for {}...",
        app_id
    );

    let pubkey_bytes = read_key_or_sig_io_uring(
        db_engine,
        Path::new(PQC_PUBLIC_KEY_PATH),
        &format!("Post-Quantum Dilithium5 public key missing at {}", PQC_PUBLIC_KEY_PATH),
        "Failed to read Dilithium5 public key via io_uring",
    )
    .await?;

    let sig_path = PathBuf::from(SIGNATURES_DIR).join(format!("{}.sig", app_id));
    let sig_bytes = read_key_or_sig_io_uring(
        db_engine,
        &sig_path,
        &format!("Package signature file missing at {}", sig_path.display()),
        "Failed to read signature file via io_uring",
    )
    .await?;

    let payload = format!("ERMETE_STORE_PACKAGE:{}:{}", app_id, oci_image);
    pqc_dilithium::verify(&sig_bytes, payload.as_bytes(), &pubkey_bytes)
        .map_err(|_| anyhow::anyhow!("Dilithium5 package signature verification failed"))?;

    println!("Dilithium5 ML-DSA signature verified successfully!");

    Ok(())
}

async fn install_app(app_id: &str, db_engine: &DatabaseEngine) -> Result<()> {
    println!(
        "Preparing to install '{}' from Ermete Store (Level 13 PQC Protected)...",
        app_id
    );

    let oci_image = format!("{}/{}", REGISTRY_URL, app_id);
    let install_url = format!("oci+https://{}", oci_image);

    // 1. Verify Post-Quantum Dilithium5 signature using io_uring file reads
    verify_pqc_package_signature(app_id, &oci_image, db_engine).await?;

    // 2. Verify classical signature with cosign
    println!("Verifying cryptographic signature for {}...", oci_image);

    if !Path::new(PUBLIC_KEY_PATH).exists() {
        anyhow::bail!(
            "CRITICAL ZERO-TRUST ERROR: Cosign public key missing at {}. Installation blocked.",
            PUBLIC_KEY_PATH
        );
    }

    let cosign_status = Command::new("cosign")
        .args(["verify", "--key", PUBLIC_KEY_PATH, &oci_image])
        .status()
        .context("Failed to run 'cosign verify'. Is cosign installed?")?;

    if !cosign_status.success() {
        anyhow::bail!(
            "Cosign signature verification failed! Installation blocked for security reasons."
        );
    }

    println!("Signature verified successfully.");

    // 3. Pass the OCI image to flatpak install
    println!("Installing Flatpak from {}...", install_url);

    let flatpak_status = Command::new("flatpak")
        .args(["install", "--system", "-y", &install_url])
        .status()
        .context("Failed to run 'flatpak install'")?;

    if !flatpak_status.success() {
        anyhow::bail!(
            "Flatpak installation failed with exit code: {}",
            flatpak_status
        );
    }

    println!(
        "Successfully installed '{}' with Level 13 PQC verification.",
        app_id
    );

    // 4. Update and asynchronously save DB snapshot in background via io_uring
    let mut snapshot = db_engine.load_snapshot().await?;
    if !snapshot.installed_packages.contains(&app_id.to_string()) {
        snapshot.installed_packages.push(app_id.to_string());
    }
    snapshot.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let bg_handle = db_engine.write_snapshot_background(snapshot);
    println!("Asynchronous DB snapshot write submitted in background via io_uring.");

    // Spawn task wait in background to handle any potential error logging cleanly
    tokio::spawn(async move {
        if let Err(e) = bg_handle.await {
            eprintln!("Error in background io_uring snapshot persistence: {:?}", e);
        }
    });

    Ok(())
}
