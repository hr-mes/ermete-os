#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxTier {
    MicroVM,
    Flatpak,
    OCIEnclave,
}

impl SandboxTier {
    pub fn label(&self) -> &'static str {
        match self {
            SandboxTier::MicroVM => "🔒 Isolato in MicroVM (crosvm)",
            SandboxTier::Flatpak => "📦 Flatpak Sandbox",
            SandboxTier::OCIEnclave => "🛡️ OCI Enclave (Rootless)",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            SandboxTier::MicroVM => "MicroVM Enclave",
            SandboxTier::Flatpak => "Flatpak Sandbox",
            SandboxTier::OCIEnclave => "OCI Enclave",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            SandboxTier::MicroVM => "badge-microvm",
            SandboxTier::Flatpak => "badge-flatpak",
            SandboxTier::OCIEnclave => "badge-oci",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            SandboxTier::MicroVM => "security-high-symbolic",
            SandboxTier::Flatpak => "package-x-generic-symbolic",
            SandboxTier::OCIEnclave => "system-run-symbolic",
        }
    }
}

/// Struttura dati per rappresentare un'applicazione nello Store
#[derive(Debug, Clone)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub icon: String,
    pub rating: f32,
    pub installed: bool,
    pub sandbox: SandboxTier,
    pub video_preview_url: Option<String>,
    pub banner_image_url: Option<String>,
    pub developer: String,
    pub suggested_donation: u32,
}

pub fn get_featured_catalog() -> Vec<AppItem> {
    vec![
        AppItem {
            id: "com.vivaldi.Vivaldi".to_string(),
            name: "Vivaldi Browser".to_string(),
            summary: "Browser web di prossima generazione ultra-personalizzabile, orientato alla privacy con sandboxing hardware.".to_string(),
            category: "Browser".to_string(),
            icon: "web-browser".to_string(),
            rating: 4.9,
            installed: true,
            sandbox: SandboxTier::MicroVM,
            video_preview_url: Some("file:///usr/share/videos/vivaldi_preview.mp4".to_string()),
            banner_image_url: Some("web-browser".to_string()),
            developer: "Vivaldi Technologies".to_string(),
            suggested_donation: 10,
        },
        AppItem {
            id: "com.valvesoftware.Steam".to_string(),
            name: "Steam".to_string(),
            summary: "Piattaforma di distribuzione gaming di riferimento con Proton compresso e MicroVM GPU Pass-through.".to_string(),
            category: "Gaming".to_string(),
            icon: "input-gaming".to_string(),
            rating: 4.8,
            installed: false,
            sandbox: SandboxTier::MicroVM,
            video_preview_url: Some("file:///usr/share/videos/steam_preview.mp4".to_string()),
            banner_image_url: Some("input-gaming".to_string()),
            developer: "Valve Corporation".to_string(),
            suggested_donation: 15,
        },
        AppItem {
            id: "md.obsidian.Obsidian".to_string(),
            name: "Obsidian".to_string(),
            summary: "Base di conoscenza personale con crittografia post-quantistica e note Markdown locali.".to_string(),
            category: "Productivity".to_string(),
            icon: "accessories-text-editor".to_string(),
            rating: 4.9,
            installed: false,
            sandbox: SandboxTier::OCIEnclave,
            video_preview_url: Some("file:///usr/share/videos/obsidian_preview.mp4".to_string()),
            banner_image_url: Some("accessories-text-editor".to_string()),
            developer: "Dynalist Inc.".to_string(),
            suggested_donation: 15,
        },
        AppItem {
            id: "com.visualstudio.code".to_string(),
            name: "VS Code".to_string(),
            summary: "Ambiente di sviluppo professionale integrato con Copilot ed eBPF telemetry isolation.".to_string(),
            category: "Development".to_string(),
            icon: "text-editor".to_string(),
            rating: 4.9,
            installed: false,
            sandbox: SandboxTier::MicroVM,
            video_preview_url: Some("file:///usr/share/videos/vscode_preview.mp4".to_string()),
            banner_image_url: Some("text-editor".to_string()),
            developer: "Microsoft Corp.".to_string(),
            suggested_donation: 10,
        },
        AppItem {
            id: "org.mozilla.firefox".to_string(),
            name: "Firefox".to_string(),
            summary: "Navigazione web veloce, open source e protetta da Flatpak strict sandbox.".to_string(),
            category: "Browser".to_string(),
            icon: "firefox".to_string(),
            rating: 4.7,
            installed: true,
            sandbox: SandboxTier::Flatpak,
            video_preview_url: Some("file:///usr/share/videos/firefox_preview.mp4".to_string()),
            banner_image_url: Some("firefox".to_string()),
            developer: "Mozilla Foundation".to_string(),
            suggested_donation: 5,
        },
        AppItem {
            id: "com.discordapp.Discord".to_string(),
            name: "Discord".to_string(),
            summary: "Piattaforma di comunicazione vocale, video e chat per community di sviluppatori e gamer.".to_string(),
            category: "Social".to_string(),
            icon: "call-start".to_string(),
            rating: 4.6,
            installed: false,
            sandbox: SandboxTier::Flatpak,
            video_preview_url: Some("file:///usr/share/videos/discord_preview.mp4".to_string()),
            banner_image_url: Some("call-start".to_string()),
            developer: "Discord Inc.".to_string(),
            suggested_donation: 5,
        },
        AppItem {
            id: "com.spotify.Client".to_string(),
            name: "Spotify".to_string(),
            summary: "Streaming musicale in alta fedeltà con supporto audio PipeWire nativo su Ermete OS.".to_string(),
            category: "Media".to_string(),
            icon: "audio-x-generic".to_string(),
            rating: 4.7,
            installed: false,
            sandbox: SandboxTier::Flatpak,
            video_preview_url: Some("file:///usr/share/videos/spotify_preview.mp4".to_string()),
            banner_image_url: Some("audio-x-generic".to_string()),
            developer: "Spotify AB".to_string(),
            suggested_donation: 10,
        },
        AppItem {
            id: "org.gimp.GIMP".to_string(),
            name: "GIMP 3.0".to_string(),
            summary: "Suite professionale per fotoritocco, grafica vettoriale e pittura digitale.".to_string(),
            category: "Graphics".to_string(),
            icon: "image-x-generic".to_string(),
            rating: 4.5,
            installed: false,
            sandbox: SandboxTier::OCIEnclave,
            video_preview_url: Some("file:///usr/share/videos/gimp_preview.mp4".to_string()),
            banner_image_url: Some("image-x-generic".to_string()),
            developer: "The GIMP Team".to_string(),
            suggested_donation: 20,
        },
    ]
}

