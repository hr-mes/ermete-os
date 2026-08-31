use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta == 0.0 {
            (0.0, 0.0, l)
        } else {
            let s = if l < 0.5 {
                delta / (max + min)
            } else {
                delta / (2.0 - max - min)
            };

            let h = if max == r {
                (g - b) / delta + (if g < b { 6.0 } else { 0.0 })
            } else if max == g {
                (b - r) / delta + 2.0
            } else {
                (r - g) / delta + 4.0
            };

            (h * 60.0, s, l)
        }
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = ((h % 360.0) + 360.0) % 360.0;
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r_prime + m) * 255.0).round() as u8,
            g: ((g_prime + m) * 255.0).round() as u8,
            b: ((b_prime + m) * 255.0).round() as u8,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Material3Palette {
    pub primary: ColorRgb,
    pub on_primary: ColorRgb,
    pub primary_container: ColorRgb,
    pub on_primary_container: ColorRgb,
    pub secondary: ColorRgb,
    pub on_secondary: ColorRgb,
    pub secondary_container: ColorRgb,
    pub on_secondary_container: ColorRgb,
    pub tertiary: ColorRgb,
    pub on_tertiary: ColorRgb,
    pub tertiary_container: ColorRgb,
    pub on_tertiary_container: ColorRgb,
    pub error: ColorRgb,
    pub on_error: ColorRgb,
    pub error_container: ColorRgb,
    pub on_error_container: ColorRgb,
    pub surface: ColorRgb,
    pub on_surface: ColorRgb,
    pub surface_variant: ColorRgb,
    pub on_surface_variant: ColorRgb,
    pub surface_container: ColorRgb,
    pub surface_container_high: ColorRgb,
    pub surface_container_highest: ColorRgb,
    pub surface_container_low: ColorRgb,
    pub surface_container_lowest: ColorRgb,
    pub background: ColorRgb,
    pub on_background: ColorRgb,
    pub outline: ColorRgb,
    pub outline_variant: ColorRgb,
    pub is_dark: bool,
}

impl Material3Palette {
    pub fn default_dark() -> Self {
        Self::from_seed_color(ColorRgb::new(137, 180, 250), true)
    }

    pub fn default_light() -> Self {
        Self::from_seed_color(ColorRgb::new(30, 102, 245), false)
    }

    pub fn from_seed_color(seed: ColorRgb, is_dark: bool) -> Self {
        let (h, s, _l) = seed.to_hsl();
        let vibrant_s = s.max(0.65);

        if is_dark {
            let primary = ColorRgb::from_hsl(h, vibrant_s, 0.75);
            let on_primary = ColorRgb::from_hsl(h, 0.3, 0.10);
            let primary_container = ColorRgb::from_hsl(h, 0.45, 0.25);
            let on_primary_container = ColorRgb::from_hsl(h, vibrant_s, 0.90);

            let secondary = ColorRgb::from_hsl(h + 30.0, 0.45, 0.72);
            let on_secondary = ColorRgb::from_hsl(h + 30.0, 0.3, 0.10);
            let secondary_container = ColorRgb::from_hsl(h + 30.0, 0.35, 0.28);
            let on_secondary_container = ColorRgb::from_hsl(h + 30.0, 0.45, 0.88);

            let tertiary = ColorRgb::from_hsl(h + 120.0, 0.60, 0.78);
            let on_tertiary = ColorRgb::from_hsl(h + 120.0, 0.3, 0.10);
            let tertiary_container = ColorRgb::from_hsl(h + 120.0, 0.40, 0.28);
            let on_tertiary_container = ColorRgb::from_hsl(h + 120.0, 0.60, 0.90);

            let error = ColorRgb::new(243, 139, 168);
            let on_error = ColorRgb::new(17, 17, 27);
            let error_container = ColorRgb::new(69, 71, 90);
            let on_error_container = ColorRgb::new(243, 139, 168);

            let surface = ColorRgb::from_hsl(h, 0.15, 0.12);
            let on_surface = ColorRgb::from_hsl(h, 0.10, 0.90);
            let surface_variant = ColorRgb::from_hsl(h, 0.18, 0.22);
            let on_surface_variant = ColorRgb::from_hsl(h, 0.12, 0.75);

            let surface_container = ColorRgb::from_hsl(h, 0.15, 0.14);
            let surface_container_high = ColorRgb::from_hsl(h, 0.15, 0.18);
            let surface_container_highest = ColorRgb::from_hsl(h, 0.15, 0.23);
            let surface_container_low = ColorRgb::from_hsl(h, 0.15, 0.11);
            let surface_container_lowest = ColorRgb::from_hsl(h, 0.15, 0.08);

            let background = surface.clone();
            let on_background = on_surface.clone();

            let outline = ColorRgb::from_hsl(h, 0.12, 0.45);
            let outline_variant = ColorRgb::from_hsl(h, 0.12, 0.30);

            Self {
                primary,
                on_primary,
                primary_container,
                on_primary_container,
                secondary,
                on_secondary,
                secondary_container,
                on_secondary_container,
                tertiary,
                on_tertiary,
                tertiary_container,
                on_tertiary_container,
                error,
                on_error,
                error_container,
                on_error_container,
                surface,
                on_surface,
                surface_variant,
                on_surface_variant,
                surface_container,
                surface_container_high,
                surface_container_highest,
                surface_container_low,
                surface_container_lowest,
                background,
                on_background,
                outline,
                outline_variant,
                is_dark,
            }
        } else {
            let primary = ColorRgb::from_hsl(h, vibrant_s, 0.40);
            let on_primary = ColorRgb::from_hsl(h, 0.1, 0.98);
            let primary_container = ColorRgb::from_hsl(h, 0.50, 0.90);
            let on_primary_container = ColorRgb::from_hsl(h, vibrant_s, 0.15);

            let secondary = ColorRgb::from_hsl(h + 30.0, 0.40, 0.42);
            let on_secondary = ColorRgb::from_hsl(h + 30.0, 0.1, 0.98);
            let secondary_container = ColorRgb::from_hsl(h + 30.0, 0.40, 0.88);
            let on_secondary_container = ColorRgb::from_hsl(h + 30.0, 0.40, 0.15);

            let tertiary = ColorRgb::from_hsl(h + 120.0, 0.55, 0.38);
            let on_tertiary = ColorRgb::from_hsl(h + 120.0, 0.1, 0.98);
            let tertiary_container = ColorRgb::from_hsl(h + 120.0, 0.50, 0.88);
            let on_tertiary_container = ColorRgb::from_hsl(h + 120.0, 0.55, 0.15);

            let error = ColorRgb::new(210, 40, 80);
            let on_error = ColorRgb::new(255, 255, 255);
            let error_container = ColorRgb::new(255, 218, 220);
            let on_error_container = ColorRgb::new(140, 0, 30);

            let surface = ColorRgb::from_hsl(h, 0.10, 0.98);
            let on_surface = ColorRgb::from_hsl(h, 0.10, 0.12);
            let surface_variant = ColorRgb::from_hsl(h, 0.15, 0.90);
            let on_surface_variant = ColorRgb::from_hsl(h, 0.12, 0.30);

            let surface_container = ColorRgb::from_hsl(h, 0.10, 0.94);
            let surface_container_high = ColorRgb::from_hsl(h, 0.10, 0.91);
            let surface_container_highest = ColorRgb::from_hsl(h, 0.10, 0.88);
            let surface_container_low = ColorRgb::from_hsl(h, 0.10, 0.96);
            let surface_container_lowest = ColorRgb::from_hsl(h, 0.10, 1.00);

            let background = surface.clone();
            let on_background = on_surface.clone();

            let outline = ColorRgb::from_hsl(h, 0.12, 0.50);
            let outline_variant = ColorRgb::from_hsl(h, 0.12, 0.75);

            Self {
                primary,
                on_primary,
                primary_container,
                on_primary_container,
                secondary,
                on_secondary,
                secondary_container,
                on_secondary_container,
                tertiary,
                on_tertiary,
                tertiary_container,
                on_tertiary_container,
                error,
                on_error,
                error_container,
                on_error_container,
                surface,
                on_surface,
                surface_variant,
                on_surface_variant,
                surface_container,
                surface_container_high,
                surface_container_highest,
                surface_container_low,
                surface_container_lowest,
                background,
                on_background,
                outline,
                outline_variant,
                is_dark,
            }
        }
    }

    pub fn extract_from_wallpaper(wallpaper_path: Option<&Path>, is_dark: bool) -> Self {
        if let Some(path) = wallpaper_path {
            if path.exists() {
                let mode = if is_dark { "dark" } else { "light" };
                let script_paths = [
                    "/usr/bin/ermete-theme-generator",
                    "/usr/libexec/ermete/ermete-theme-generator.sh",
                ];
                for s in script_paths {
                    if Path::new(s).exists() {
                        if let Ok(st) = Command::new(s).args([path.to_str().unwrap_or(""), mode]).status() {
                            if st.success() {
                                // Script executed successfully
                            }
                        }
                    }
                }

                if let Ok(bytes) = std::fs::read(path) {
                    if !bytes.is_empty() {
                        let mut sum_r: u32 = 0;
                        let mut sum_g: u32 = 0;
                        let mut sum_b: u32 = 0;
                        let mut count: u32 = 0;

                        let step = (bytes.len() / 256).max(1);
                        for i in (0..bytes.len()).step_by(step) {
                            if i + 2 < bytes.len() {
                                sum_r += bytes[i] as u32;
                                sum_g += bytes[i + 1] as u32;
                                sum_b += bytes[i + 2] as u32;
                                count += 1;
                            }
                        }

                        if count > 0 {
                            let r = (sum_r / count) as u8;
                            let g = (sum_g / count) as u8;
                            let b = (sum_b / count) as u8;
                            return Self::from_seed_color(ColorRgb::new(r, g, b), is_dark);
                        }
                    }
                }
            }
        }

        if is_dark {
            Self::default_dark()
        } else {
            Self::default_light()
        }
    }

    pub fn to_gtk4_css(&self) -> String {
        format!(
            r#"/* Material 3 Dynamic Theme Generated for Ermete Shell GTK4 */
@define-color primary {};
@define-color on_primary {};
@define-color primary_container {};
@define-color on_primary_container {};
@define-color secondary {};
@define-color on_secondary {};
@define-color secondary_container {};
@define-color on_secondary_container {};
@define-color tertiary {};
@define-color on_tertiary {};
@define-color tertiary_container {};
@define-color on_tertiary_container {};
@define-color error {};
@define-color on_error {};
@define-color error_container {};
@define-color on_error_container {};
@define-color surface {};
@define-color on_surface {};
@define-color surface_variant {};
@define-color on_surface_variant {};
@define-color surface_container {};
@define-color surface_container_high {};
@define-color surface_container_highest {};
@define-color surface_container_low {};
@define-color surface_container_lowest {};
@define-color background {};
@define-color on_background {};
@define-color outline {};
@define-color outline_variant {};

/* Ermete UI Glassmorphism & Core Tokens */
@define-color accent_color {};
@define-color window_bg {};
@define-color window_fg {};
@define-color glass_bg {};
@define-color glass_border {};
@define-color hover_bg {};
"#,
            self.primary.to_hex(),
            self.on_primary.to_hex(),
            self.primary_container.to_hex(),
            self.on_primary_container.to_hex(),
            self.secondary.to_hex(),
            self.on_secondary.to_hex(),
            self.secondary_container.to_hex(),
            self.on_secondary_container.to_hex(),
            self.tertiary.to_hex(),
            self.on_tertiary.to_hex(),
            self.tertiary_container.to_hex(),
            self.on_tertiary_container.to_hex(),
            self.error.to_hex(),
            self.on_error.to_hex(),
            self.error_container.to_hex(),
            self.on_error_container.to_hex(),
            self.surface.to_hex(),
            self.on_surface.to_hex(),
            self.surface_variant.to_hex(),
            self.on_surface_variant.to_hex(),
            self.surface_container.to_hex(),
            self.surface_container_high.to_hex(),
            self.surface_container_highest.to_hex(),
            self.surface_container_low.to_hex(),
            self.surface_container_lowest.to_hex(),
            self.background.to_hex(),
            self.on_background.to_hex(),
            self.outline.to_hex(),
            self.outline_variant.to_hex(),
            self.primary.to_hex(),
            self.surface.to_hex(),
            self.on_surface.to_hex(),
            format!("rgba({}, {}, {}, 0.78)", self.surface_container.r, self.surface_container.g, self.surface_container.b),
            format!("rgba({}, {}, {}, 0.25)", self.outline.r, self.outline.g, self.outline.b),
            format!("rgba({}, {}, {}, 0.12)", self.on_surface.r, self.on_surface.g, self.on_surface.b),
        )
    }

    pub fn write_to_file(&self, target_path: &Path) -> std::io::Result<()> {
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(target_path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, self.to_gtk4_css().as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_palette_generation() {
        let seed = ColorRgb::new(100, 150, 200);
        let palette = Material3Palette::from_seed_color(seed, true);
        let css = palette.to_gtk4_css();

        assert!(css.contains("@define-color primary #"));
        assert!(css.contains("@define-color surface #"));
        assert!(css.contains("@define-color glass_bg rgba"));
        assert!(css.contains("@define-color accent_color #"));
    }

    #[test]
    fn test_wallpaper_extraction_and_write() {
        let tmp = std::env::temp_dir().join("ermete_palette_test");
        let wallpaper = tmp.join("bg.png");
        if let Err(e) = std::fs::create_dir_all(&tmp) {
                tracing::error!("Failed to create tmp dir {:?}: {:?}", tmp, e);
            }
        let wallpaper_data = vec![120, 80, 200, 255].repeat(250);
            if let Err(e) = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&wallpaper)
                .and_then(|mut f| std::io::Write::write_all(&mut f, &wallpaper_data))
            {
                tracing::error!("Failed to write wallpaper at {:?}: {:?}", wallpaper, e);
            }

        let palette = Material3Palette::extract_from_wallpaper(Some(&wallpaper), true);
        let out_css = tmp.join("theme.css");
        palette.write_to_file(&out_css).expect("Ermete OS: Fallimento critico di unwrapping. Zero-Trust Panic Invocato.");

        assert!(out_css.exists());
        let content = std::fs::read_to_string(&out_css).expect("Ermete OS: Fallimento critico di unwrapping. Zero-Trust Panic Invocato.");
        assert!(content.contains("@define-color primary"));
    }
}

