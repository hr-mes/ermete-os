#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxTier {
    Flatpak,
    Native,
}

impl SandboxTier {
    pub fn label(&self) -> &'static str {
        match self {
            SandboxTier::Flatpak => "📦 Flatpak Sandbox",
            SandboxTier::Native => "⚙️ Applicazione Nativa",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            SandboxTier::Flatpak => "Flatpak Sandbox",
            SandboxTier::Native => "Native",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            SandboxTier::Flatpak => "badge-flatpak",
            SandboxTier::Native => "badge-native",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            SandboxTier::Flatpak => "package-x-generic-symbolic",
            SandboxTier::Native => "system-run-symbolic",
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
    let mut catalog = Vec::new();

    // Query dynamic real catalog from system Flatpak daemon
    let output = std::process::Command::new("flatpak")
        .args(["list", "--app", "--columns=application,name,description"])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.is_empty() || parts[0].trim().is_empty() {
                    continue;
                }
                let app_id = parts[0].trim().to_string();
                let name = if parts.len() > 1 && !parts[1].trim().is_empty() {
                    parts[1].trim().to_string()
                } else {
                    app_id.clone()
                };
                let summary = if parts.len() > 2 {
                    parts[2].trim().to_string()
                } else {
                    format!("Flatpak package {}", app_id)
                };

                catalog.push(AppItem {
                    id: app_id.clone(),
                    name,
                    summary,
                    category: "Flatpak".to_string(),
                    icon: "package-x-generic".to_string(),
                    rating: 5.0,
                    installed: true,
                    sandbox: SandboxTier::Flatpak,
                    video_preview_url: None,
                    banner_image_url: None,
                    developer: "Flathub / System".to_string(),
                    suggested_donation: 0,
                });
            }
        }
    }

    catalog
}


