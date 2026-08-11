use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use tracing::{debug, info, warn};

const GHCR_BASE_URL: &str = "https://ghcr.io/v2/hr-mes/ermete-os-kernel";
const GITHUB_API_BASE: &str = "https://api.github.com/repos/hr-mes/ermete-os/actions/workflows";

#[derive(Serialize)]
struct WorkflowDispatchPayload {
    ref_name: String,
    inputs: serde_json::Value,
}

pub struct FalClient {
    http_client: Client,
    github_token: Option<String>,
}

impl FalClient {
    pub fn new(github_token: Option<String>) -> Self {
        Self {
            http_client: Client::builder()
                .user_agent("Ermete-FAL-Client/1.0")
                .build()
                .unwrap_or_default(),
            github_token,
        }
    }

    /// Controlla se esiste già un Kernel iper-ottimizzato per questo specifico Hardware Hash
    /// nella Global Cache pubblica (GHCR).
    pub async fn check_global_cache(&self, hardware_hash: &str) -> Result<bool> {
        info!("Interrogazione Global Cache (GHCR) per Hash: {}", hardware_hash);
        
        // Costruisce l'URL della manifest OCI
        let url = format!("{}/manifests/{}", GHCR_BASE_URL, hardware_hash);
        
        let response = self.http_client
            .head(&url)
            .send()
            .await
            .context("Impossibile contattare la Global Cache (GHCR)")?;

        match response.status() {
            StatusCode::OK => {
                info!("✅ CACHE HIT: Trovato Kernel pre-compilato e ottimizzato per questo hardware!");
                Ok(true)
            }
            StatusCode::NOT_FOUND => {
                info!("❌ CACHE MISS: Nessun Kernel trovato per questo hardware specifico.");
                Ok(false)
            }
            _ => {
                warn!("Risposta inaspettata dalla Global Cache: {}", response.status());
                Ok(false)
            }
        }
    }

    /// Triggera la compilazione remota del Kernel tramite le GitHub Actions dell'azienda
    /// (o del BYOC se configurato), passando l'hardware hash come input.
    pub async fn trigger_remote_build(&self, hardware_hash: &str) -> Result<()> {
        let token = self.github_token.as_ref()
            .ok_or_else(|| anyhow!("Nessun token OIDC/GitHub configurato per innescare la Remote Forge"))?;

        info!("Innesco compilazione remota (Cloud Forge) per Hash: {}", hardware_hash);

        // workflow_id può essere il nome del file yml
        let url = format!("{}/kernel-build.yml/dispatches", GITHUB_API_BASE);
        
        let payload = WorkflowDispatchPayload {
            ref_name: "main".to_string(),
            inputs: serde_json::json!({
                "hardware_hash": hardware_hash
            }),
        };

        let response = self.http_client
            .post(&url)
            .bearer_auth(token)
            .header("Accept", "application/vnd.github.v3+json")
            .json(&payload)
            .send()
            .await
            .context("Fallimento della richiesta di compilazione remota (Network Error)")?;

        if response.status().is_success() {
            info!("✅ Remote Forge Innescata con successo. I server Microsoft stanno compilando il Kernel.");
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(anyhow!("Errore nell'innesco della Remote Forge: {} - {}", status, body))
        }
    }

    /// Simula il fetch crittografico tramite ostree/bootc del container kernel 
    pub async fn pull_and_deploy(&self, hardware_hash: &str) -> Result<()> {
        info!("📥 Avvio Download e Deploy del Kernel ottimizato (Hash: {})", hardware_hash);
        info!("Verifica firme Sigstore/Cosign... (Vitreol Phase)");

        let container_ref = format!("{}:{}", GHCR_BASE_URL, hardware_hash);

        // 1. Verifica crittografica (Sigstore/Cosign)
        let cosign_status = tokio::process::Command::new("cosign")
            .arg("verify")
            .arg("--key")
            .arg("/etc/ermete/pki/cosign.pub")
            .arg(&container_ref)
            .status()
            .await
            .context("Comando cosign non trovato o esecuzione fallita")?;

        if !cosign_status.success() {
            return Err(anyhow!("Verifica Sigstore FALLITA per il container {}. Aggiornamento annullato per motivi di sicurezza (ToFU).", container_ref));
        }
        info!("✅ Verifica crittografica superata.");

        // 2. Deploy tramite bootc
        info!("Esecuzione bootc switch al container {} ...", container_ref);
        let bootc_status = tokio::process::Command::new("bootc")
            .arg("switch")
            .arg(&container_ref)
            .status()
            .await
            .context("Comando bootc non trovato o esecuzione fallita")?;

        if !bootc_status.success() {
            return Err(anyhow!("Fallimento durante l'operazione di bootc switch."));
        }

        info!("✅ Bootc switch completato. Il nuovo Kernel sarà attivo al prossimo riavvio.");
        Ok(())
    }
}
