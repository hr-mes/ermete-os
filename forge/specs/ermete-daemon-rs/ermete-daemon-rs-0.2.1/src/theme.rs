
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};
use anyhow::{Context, Result};

const DEFAULT_TEMPLATE: &str = include_str!("../assets/matugen_theme.template.default");

pub fn config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("ermete")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("ermete")
    } else {
        PathBuf::from("/var/lib/ermete")
    }
}

/// Dynamic Theme Pipeline: extracts Material 3 color palette via Matugen / script
/// and updates swww wallpaper + ~/.config/ermete/theme.css GTK4 directives.
pub async fn apply_dynamic_theme(wallpaper_path: &str, color_scheme: &str) -> Result<()> {
    info!(
        wallpaper = %wallpaper_path,
        scheme = %color_scheme,
        "Applying dynamic Material 3 theme extraction..."
    );

    let wallpaper = wallpaper_path.to_string();
    let scheme = color_scheme.to_string();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let mode = if scheme == "default" { "light" } else { "dark" };

        let script_paths = [
            "/usr/bin/ermete-theme-generator",
            "/usr/libexec/ermete/ermete-theme-generator.sh",
            "system/scripts/ermete-theme-generator.sh",
        ];

        let mut executed = false;
        for path in script_paths {
            if std::path::Path::new(path).exists() {
                if let Ok(status) = Command::new(path).args([&wallpaper, mode]).status() {
                    if status.success() {
                        info!(script = %path, "Dynamic theme generated successfully via script.");
                        executed = true;
                        break;
                    }
                }
            }
        }

        if !executed {
            let cfg_dir = config_dir();
            std::fs::create_dir_all(&cfg_dir)
                .with_context(|| format!("Failed to create config directory {:?}", cfg_dir))?;
            let theme_css = cfg_dir.join("theme.css");

            // Apply wallpaper via swww if running
            let _ = Command::new("swww")
                .args(["img", &wallpaper, "--transition-type", "outer", "--transition-step", "90"])
                .status();

            let template_path = cfg_dir.join("matugen_theme.template");
            if !template_path.exists() {
                let default_tpl = std::fs::read_to_string("/usr/share/ermete/matugen_theme.template")
                    .unwrap_or_else(|_| DEFAULT_TEMPLATE.to_string());
                std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&template_path)
                .and_then(|mut f| std::io::Write::write_all(&mut f, default_tpl.as_bytes()))
                    .with_context(|| format!("Failed to write template file {:?}", template_path))?;
            }

            let tmp_cfg = cfg_dir.join("matugen_tmp.toml");
            let cfg_content = format!(
                "[config]\nreload_apps = false\n\n[templates.gtk4_theme]\ninput_path = \"{}\"\noutput_path = \"{}\"\n",
                template_path.display(),
                theme_css.display()
            );

            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&tmp_cfg)
                .and_then(|mut f| std::io::Write::write_all(&mut f, cfg_content.as_bytes()))
                .with_context(|| format!("Failed to write temp config file {:?}", tmp_cfg))?;

            let tmp_cfg_str = tmp_cfg.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp config path"))?;
            let mat_res = Command::new("matugen")
                .args(["image", &wallpaper, "--source-color-index", "0", "--mode", mode, "-c", tmp_cfg_str])
                .status();
            if let Err(e) = std::fs::remove_file(&tmp_cfg) {
            tracing::error!("Failed to remove tmp_cfg {:?}: {:?}", tmp_cfg, e);
        }

            if mat_res.map(|s| s.success()).unwrap_or(false) {
                info!("Dynamic theme generated cleanly via matugen direct CLI.");
                return Ok(());
            }

            // Fallback GTK4 CSS
            let fallback_css = r#"/* Material 3 Fallback Dynamic Theme for Ermete OS */
@define-color primary #89b4fa;
@define-color on_primary #11111b;
@define-color primary_container #313244;
@define-color on_primary_container #cdd6f4;
@define-color secondary #cba6f7;
@define-color on_secondary #11111b;
@define-color secondary_container #45475a;
@define-color on_secondary_container #cdd6f4;
@define-color tertiary #f5e0dc;
@define-color on_tertiary #11111b;
@define-color tertiary_container #585b70;
@define-color on_tertiary_container #cdd6f4;
@define-color error #f38ba8;
@define-color on_error #11111b;
@define-color error_container #45475a;
@define-color on_error_container #f38ba8;
@define-color surface #1e1e2e;
@define-color on_surface #cdd6f4;
@define-color surface_variant #313244;
@define-color on_surface_variant #a6adc8;
@define-color surface_container #181825;
@define-color surface_container_high #313244;
@define-color surface_container_highest #45475a;
@define-color surface_container_low #181825;
@define-color surface_container_lowest #11111b;
@define-color background #1e1e2e;
@define-color on_background #cdd6f4;
@define-color outline #585b70;
@define-color outline_variant #45475a;

/* Ermete UI Compatibility Tokens */
@define-color accent_color #89b4fa;
@define-color window_bg #1e1e2e;
@define-color window_fg #cdd6f4;
"#;
            std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&theme_css)
            .and_then(|mut f| std::io::Write::write_all(&mut f, fallback_css.as_bytes()))
                .with_context(|| format!("Failed to write fallback theme CSS: {:?}", theme_css))?;
            warn!("Matugen not available. Wrote fallback theme to theme.css.");
        }
        Ok(())
    })
    .await
    .map_err(|e| anyhow::anyhow!("Spawn blocking task failed: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_apply_dynamic_theme_pipeline() {
        let tmp_dir = std::env::temp_dir().join("ermete_test_theme_pipeline");
        if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
            tracing::error!("Failed to create tmp_dir {:?}: {:?}", tmp_dir, e);
        }
        let wallpaper = tmp_dir.join("test_bg.png");
        if let Err(e) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&wallpaper)
            .and_then(|mut f| std::io::Write::write_all(&mut f, b"fake_png_data"))
        {
            tracing::error!("Failed to write wallpaper {:?}: {:?}", wallpaper, e);
        }

        let wallpaper_str = wallpaper.to_str().expect("wallpaper path UTF-8");
        apply_dynamic_theme(wallpaper_str, "prefer-dark").await.unwrap();

        let cfg_dir = config_dir();
        let theme_css = cfg_dir.join("theme.css");
        assert!(theme_css.exists());
        let content = std::fs::read_to_string(&theme_css).expect("Read theme.css");
        assert!(content.contains("@define-color primary"));
        assert!(content.contains("@define-color surface"));
    }
}

