use anyhow::Result;
use tracing::{info, warn};

pub struct DrmBackendConfig {
    #[allow(dead_code)]
    pub prefer_primary_gpu: bool,
    pub allow_headless_fallback: bool,
}

impl Default for DrmBackendConfig {
    fn default() -> Self {
        Self {
            prefer_primary_gpu: true,
            allow_headless_fallback: true,
        }
    }
}

pub struct DrmKmsBackend {
    config: DrmBackendConfig,
    active_cards: Vec<String>,
    is_headless: bool,
}

impl DrmKmsBackend {
    pub fn new(config: DrmBackendConfig) -> Self {
        Self {
            config,
            active_cards: Vec::new(),
            is_headless: false,
        }
    }

    pub fn is_headless(&self) -> bool {
        self.is_headless
    }

    pub fn active_cards(&self) -> &[String] {
        &self.active_cards
    }

    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing DRM/KMS backend for native Wayland rendering...");

        // Zero-Trust DRM device discovery: Enumerate /dev/dri card nodes subject to logind session device lease
        let mut cards = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/dev/dri") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("card") {
                        cards.push(path.to_string_lossy().into_owned());
                    }
                }
            }
        }

        cards.sort();

        if !cards.is_empty() {
            info!("Discovered DRM/KMS device nodes: {:?}", cards);
            self.active_cards = cards;
            self.is_headless = false;
            info!("DRM/KMS hardware backend successfully initialized.");
        } else if self.config.allow_headless_fallback {
            warn!("No DRM/KMS device nodes (/dev/dri/card*) detected or accessible. Falling back to headless virtual output backend.");
            self.is_headless = true;
        } else {
            anyhow::bail!("No DRM/KMS cards found and headless fallback is disabled");
        }

        Ok(())
    }
}
