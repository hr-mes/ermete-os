use drm::Device;
use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::os::unix::io::{AsFd, BorrowedFd, AsRawFd, RawFd};
use tracing::{info, warn};
use drm::control::Device as ControlDevice;

pub struct Card(std::fs::File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl drm::Device for Card {}
impl ControlDevice for Card {}

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
    drm_device: Option<Card>,
    gbm_device: Option<gbm::Device<std::fs::File>>,
}

impl DrmKmsBackend {
    pub fn new(config: DrmBackendConfig) -> Self {
        Self {
            config,
            active_cards: Vec::new(),
            is_headless: false,
            drm_device: None,
            gbm_device: None,
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
            
            let target_card = cards[0].clone();
            info!("Attempting to open KMS device: {}", target_card);
            
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&target_card)
                .with_context(|| format!("Failed to open {}", target_card))?;
                
            let card = Card(file.try_clone().context("Failed to clone file descriptor")?);
            
            info!("Acquiring DRM Master lock...");
            card.acquire_master_lock().context("Failed to acquire DRM Master privileges.")?;
            
            info!("Initializing GBM device from DRM node...");
            let gbm = gbm::Device::new(file).context("Failed to create GBM device")?;
            
            self.drm_device = Some(card);
            self.gbm_device = Some(gbm);
            
            self.active_cards = cards;
            self.is_headless = false;
            info!("DRM/KMS hardware backend successfully initialized with real ioctls.");
        } else if self.config.allow_headless_fallback {
            warn!("No DRM/KMS device nodes (/dev/dri/card*) detected or accessible. Falling back to headless virtual output backend.");
            self.is_headless = true;
        } else {
            anyhow::bail!("No DRM/KMS cards found and headless fallback is disabled");
        }

        Ok(())
    }
}


