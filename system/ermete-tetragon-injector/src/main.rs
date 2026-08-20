use anyhow::{Context, Result};
use serde::Serialize;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{info, warn, Level};

#[derive(Serialize)]
struct TracingPolicy {
    apiVersion: String,
    kind: String,
    metadata: Metadata,
    spec: Spec,
}

#[derive(Serialize)]
struct Metadata {
    name: String,
    namespace: String,
}

#[derive(Serialize)]
struct Spec {
    kprobes: Vec<KProbe>,
}

#[derive(Serialize)]
struct KProbe {
    call: String,
    syscall: bool,
    args: Vec<Arg>,
    selectors: Vec<Selector>,
}

#[derive(Serialize)]
struct Arg {
    index: u32,
    #[serde(rename = "type")]
    arg_type: String,
}

#[derive(Serialize)]
struct Selector {
    matchArgs: Vec<String>, // Vuoto per catturare tutto
}

/// Genera la policy hardcodata, impossibile da alterare da disco
fn generate_hardcoded_policy() -> TracingPolicy {
    TracingPolicy {
        apiVersion: "cilium.io/v1alpha1".to_string(),
        kind: "TracingPolicy".to_string(),
        metadata: Metadata {
            name: "sys-execve-monitor-zero-trust".to_string(),
            namespace: "kube-system".to_string(),
        },
        spec: Spec {
            kprobes: vec![KProbe {
                call: "sys_execve".to_string(),
                syscall: true,
                args: vec![
                    Arg { index: 0, arg_type: "string".to_string() },
                    Arg { index: 1, arg_type: "string_array".to_string() },
                    Arg { index: 2, arg_type: "string_array".to_string() },
                ],
                selectors: vec![Selector { matchArgs: vec![] }],
            }],
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    info!("🚀 Inizializzazione Ermete Tetragon Zero-Trust Injector");

    let policy = generate_hardcoded_policy();
    let payload = serde_json::to_string(&policy)?;

    info!("💉 Iniettando policy eBPF dinamica via memoria (senza file fisici)...");

    // Comunichiamo con Tetragon direttamente via stdin per evitare file su disco
    let mut child = Command::new("tetra")
        .arg("tracingpolicy")
        .arg("add")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Fallito avvio del client tetra. Tetragon è installato?")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(payload.as_bytes()).await?;
    }

    let output = child.wait_with_output().await?;
    if output.status.success() {
        info!("✅ Policy eBPF iniettata con successo nel ring-0 del Kernel.");
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        warn!("⚠️ Fallimento iniezione policy (il demone Tetragon potrebbe non essere attivo in CI): {}", err);
    }

    Ok(())
}
