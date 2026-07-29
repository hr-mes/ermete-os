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
}

pub fn get_featured_catalog() -> Vec<AppItem> {
    vec![
        AppItem {
            id: "com.vivaldi.Vivaldi".to_string(),
            name: "Vivaldi".to_string(),
            summary: "Browser web potente, altamente personalizzabile e orientato alla privacy.".to_string(),
            category: "Browser".to_string(),
            icon: "web-browser".to_string(),
            rating: 4.9,
            installed: true,
        },
        AppItem {
            id: "com.valvesoftware.Steam".to_string(),
            name: "Steam".to_string(),
            summary: "La piattaforma di distribuzione digitale e gaming di riferimento su Linux.".to_string(),
            category: "Gaming".to_string(),
            icon: "input-gaming".to_string(),
            rating: 4.8,
            installed: false,
        },
        AppItem {
            id: "org.mozilla.firefox".to_string(),
            name: "Firefox".to_string(),
            summary: "Navigazione web veloce, sicura e indipendente.".to_string(),
            category: "Browser".to_string(),
            icon: "firefox".to_string(),
            rating: 4.7,
            installed: true,
        },
        AppItem {
            id: "com.visualstudio.code".to_string(),
            name: "VS Code".to_string(),
            summary: "Ambiente di sviluppo leggero, potente ed estendibile.".to_string(),
            category: "Development".to_string(),
            icon: "text-editor".to_string(),
            rating: 4.9,
            installed: false,
        },
        AppItem {
            id: "com.discordapp.Discord".to_string(),
            name: "Discord".to_string(),
            summary: "Piattaforma di comunicazione vocale e testuale per community.".to_string(),
            category: "Social".to_string(),
            icon: "call-start".to_string(),
            rating: 4.6,
            installed: false,
        },
        AppItem {
            id: "com.spotify.Client".to_string(),
            name: "Spotify".to_string(),
            summary: "Streaming musicale in alta qualità con milioni di brani e podcast.".to_string(),
            category: "Media".to_string(),
            icon: "audio-x-generic".to_string(),
            rating: 4.7,
            installed: false,
        },
        AppItem {
            id: "md.obsidian.Obsidian".to_string(),
            name: "Obsidian".to_string(),
            summary: "Base di conoscenza personale basata su file Markdown locali.".to_string(),
            category: "Productivity".to_string(),
            icon: "accessories-text-editor".to_string(),
            rating: 4.9,
            installed: false,
        },
        AppItem {
            id: "org.gimp.GIMP".to_string(),
            name: "GIMP".to_string(),
            summary: "Editor di grafica raster avanzato e fotoritocco professionale.".to_string(),
            category: "Graphics".to_string(),
            icon: "image-x-generic".to_string(),
            rating: 4.5,
            installed: false,
        },
    ]
}
