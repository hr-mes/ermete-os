use std::path::PathBuf;

/// RGB Color representation with floating point and hex conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn to_rgba_string(&self, alpha: f64) -> String {
        format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, alpha)
    }

    pub fn parse_hex(hex: &str) -> Self {
        let clean = hex.trim().trim_start_matches('#');
        if clean.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&clean[0..2], 16),
                u8::from_str_radix(&clean[2..4], 16),
                u8::from_str_radix(&clean[4..6], 16),
            ) {
                return Self::new(r, g, b);
            }
        } else if clean.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&clean[0..1].repeat(2), 16),
                u8::from_str_radix(&clean[1..2].repeat(2), 16),
                u8::from_str_radix(&clean[2..3].repeat(2), 16),
            ) {
                return Self::new(r, g, b);
            }
        }
        // Default Ermete OS Accent (#89b4fa)
        Self::new(137, 180, 250)
    }

    /// WCAG relative luminance calculation for contrast determination
    pub fn luminance(&self) -> f64 {
        let rf = self.r as f64 / 255.0;
        let gf = self.g as f64 / 255.0;
        let bf = self.b as f64 / 255.0;
        0.2126 * rf + 0.7152 * gf + 0.0722 * bf
    }

    /// High contrast text foreground selection based on luminance
    pub fn contrasting_fg(&self) -> Self {
        if self.luminance() > 0.45 {
            Self::new(17, 17, 27) // Dark contrast text (#11111b)
        } else {
            Self::new(255, 255, 255) // Light contrast text (#ffffff)
        }
    }

    pub fn to_hsl(&self) -> ColorHsl {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        let s = if delta == 0.0 {
            0.0
        } else if l <= 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let mut h = if delta == 0.0 {
            0.0
        } else if max == r {
            ((g - b) / delta) % 6.0
        } else if max == g {
            ((b - r) / delta) + 2.0
        } else {
            ((r - g) / delta) + 4.0
        };

        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }

        ColorHsl { h, s, l }
    }
}

/// HSL Color representation for shade variations and lightness shifts
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorHsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

impl ColorHsl {
    pub fn adjust_lightness(&self, factor: f64) -> Self {
        let new_l = (self.l * factor).clamp(0.0, 1.0);
        Self {
            h: self.h,
            s: self.s,
            l: new_l,
        }
    }

    pub fn to_rgb(&self) -> ColorRgb {
        let c = (1.0 - (2.0 * self.l - 1.0).abs()) * self.s;
        let x = c * (1.0 - ((self.h / 60.0) % 2.0 - 1.0).abs());
        let m = self.l - c / 2.0;

        let (r_prime, g_prime, b_prime) = if self.h < 60.0 {
            (c, x, 0.0)
        } else if self.h < 120.0 {
            (x, c, 0.0)
        } else if self.h < 180.0 {
            (0.0, c, x)
        } else if self.h < 240.0 {
            (0.0, x, c)
        } else if self.h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        let g = ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        let b = ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8;

        ColorRgb::new(r, g, b)
    }
}

/// Full derived accent color palette (Feren OS / XeroLinux reverse-engineered engine)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AccentPalette {
    pub base: ColorRgb,
    pub hover: ColorRgb,
    pub active: ColorRgb,
    pub fg: ColorRgb,
    pub hex: String,
    pub subtle_alpha: String,
    pub glass_alpha: String,
    pub focus_alpha: String,
}

impl AccentPalette {
    pub fn from_hex(hex: &str) -> Self {
        let base = ColorRgb::parse_hex(hex);
        let hsl = base.to_hsl();

        let hover = hsl.adjust_lightness(1.15).to_rgb();
        let active = hsl.adjust_lightness(0.85).to_rgb();
        let fg = base.contrasting_fg();

        Self {
            base,
            hover,
            active,
            fg,
            hex: base.to_hex(),
            subtle_alpha: base.to_rgba_string(0.15),
            glass_alpha: base.to_rgba_string(0.25),
            focus_alpha: base.to_rgba_string(0.50),
        }
    }

    /// Generates dynamic GTK4, Libadwaita, Compositor, Shell, and Dock CSS
    pub fn generate_gtk_css(&self) -> String {
        format!(
            r#"/* Dynamic Global Accent Color Engine (Feren OS / XeroLinux style for Ermete OS) */
@define-color accent_color {hex};
@define-color accent_bg_color {hex};
@define-color accent_fg_color {fg_hex};
@define-color accent_hover {hover_hex};
@define-color accent_active {active_hex};
@define-color accent_bg_alpha {subtle_alpha};
@define-color accent_border {glass_alpha};
@define-color focus_border {focus_alpha};

:root, window, .glass-panel, .dock-container, .flat-canvas-container {{
    --accent-color: {hex};
    --accent-hover: {hover_hex};
    --accent-active: {active_hex};
    --accent-fg: {fg_hex};
    --accent-subtle: {subtle_alpha};
    --accent-glass: {glass_alpha};
    --accent-focus: {focus_alpha};
}}

/* GTK4 & Libadwaita Suggested Actions & Buttons */
.accent, .accent-bg, button.suggested-action {{
    background-color: {hex} !important;
    color: {fg_hex} !important;
    border-color: {hover_hex} !important;
}}

.accent:hover, .accent-bg:hover, button.suggested-action:hover {{
    background-color: {hover_hex} !important;
    color: {fg_hex} !important;
}}

.accent:active, .accent-bg:active, button.suggested-action:active {{
    background-color: {active_hex} !important;
    color: {fg_hex} !important;
}}

/* Selection & Entry Highlights */
selection, entry selection, label:selected {{
    background-color: {subtle_alpha} !important;
    color: {hex} !important;
}}

entry:focus, button:focus, .focus-ring:focus {{
    outline-color: {focus_alpha} !important;
    border-color: {hex} !important;
}}

/* Switches, Checks, Radios, Sliders */
switch:checked {{
    background-color: {hex} !important;
}}

check:checked, radio:checked {{
    background-color: {hex} !important;
    color: {fg_hex} !important;
}}

scale highlight, slider:checked, progressbar progress {{
    background-color: {hex} !important;
}}

/* Dock, Shell, Compositor Tinting */
.dock-indicator-focused, .dock-instance-badge {{
    background-color: {hex} !important;
    color: {fg_hex} !important;
}}

.dock-item-btn:hover {{
    background: {subtle_alpha} !important;
}}

.morphic-pill.active, .morphic-pill:hover {{
    border-color: {hex} !important;
    box-shadow: 0 0 12px {focus_alpha} !important;
}}

.window-active-border {{
    border-color: {hex} !important;
}}

.card.accent-tint {{
    border: 1px solid {glass_alpha} !important;
    background: {subtle_alpha} !important;
}}
"#,
            hex = self.hex,
            fg_hex = self.fg.to_hex(),
            hover_hex = self.hover.to_hex(),
            active_hex = self.active.to_hex(),
            subtle_alpha = self.subtle_alpha,
            glass_alpha = self.glass_alpha,
            focus_alpha = self.focus_alpha,
        )
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("ermete")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("ermete")
    } else {
        PathBuf::from("/var/lib/ermete")
    }
}

pub fn get_accent_css_path() -> PathBuf {
    get_config_dir().join("accent.css")
}

pub fn get_theme_css_path() -> PathBuf {
    get_config_dir().join("theme.css")
}

thread_local! {
    static ACCENT_CSS_PROVIDER: std::cell::RefCell<Option<gtk4::CssProvider>> = const { std::cell::RefCell::new(None) };
}

/// Applies accent color hex: generates GTK CSS, persists accent.css and theme.css, and injects into GTK4 display
pub fn apply_accent_color(hex_color: &str) -> Result<String, String> {
    let palette = AccentPalette::from_hex(hex_color);
    let css_content = palette.generate_gtk_css();

    let config_dir = get_config_dir();
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        return Err(format!("Failed to create config dir: {}", e));
    }

    let accent_path = get_accent_css_path();
    if let Err(e) = std::fs::write(&accent_path, &css_content) {
        return Err(format!("Failed to write accent.css: {}", e));
    }

    // Update theme.css so processes monitoring theme.css reload instantly
    let theme_path = get_theme_css_path();
    let updated_theme_css = if theme_path.exists() {
        if let Ok(existing) = std::fs::read_to_string(&theme_path) {
            if let Some(pos) = existing.find("/* Dynamic Global Accent Color Engine") {
                format!("{}\n{}", &existing[..pos], css_content)
            } else {
                format!("{}\n\n{}", existing, css_content)
            }
        } else {
            css_content.clone()
        }
    } else {
        css_content.clone()
    };

    let _ = std::fs::write(&theme_path, updated_theme_css);

    // Inject into current process GTK display live
    inject_gtk_css(&css_content);

    Ok(css_content)
}

/// Inject generated CSS string into the default GTK4 display
pub fn inject_gtk_css(css: &str) {
    ACCENT_CSS_PROVIDER.with(|provider_cell| {
        let mut cell = provider_cell.borrow_mut();
        let provider = cell.get_or_insert_with(gtk4::CssProvider::new);
        provider.load_from_data(css);

        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                provider,
                gtk4::STYLE_PROVIDER_PRIORITY_USER + 100,
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_parsing() {
        let c1 = ColorRgb::parse_hex("#89b4fa");
        assert_eq!(c1, ColorRgb::new(137, 180, 250));

        let c2 = ColorRgb::parse_hex("f38ba8");
        assert_eq!(c2, ColorRgb::new(243, 139, 168));

        let c3 = ColorRgb::parse_hex("#fff");
        assert_eq!(c3, ColorRgb::new(255, 255, 255));
    }

    #[test]
    fn test_luminance_and_fg() {
        let dark = ColorRgb::new(20, 20, 20);
        assert_eq!(dark.contrasting_fg(), ColorRgb::new(255, 255, 255));

        let light = ColorRgb::new(240, 240, 240);
        assert_eq!(light.contrasting_fg(), ColorRgb::new(17, 17, 27));
    }

    #[test]
    fn test_hsl_conversion_roundtrip() {
        let rgb = ColorRgb::new(100, 150, 200);
        let hsl = rgb.to_hsl();
        let rgb_back = hsl.to_rgb();
        assert!((rgb.r as i16 - rgb_back.r as i16).abs() <= 2);
        assert!((rgb.g as i16 - rgb_back.g as i16).abs() <= 2);
        assert!((rgb.b as i16 - rgb_back.b as i16).abs() <= 2);
    }

    #[test]
    fn test_generate_gtk_css() {
        let palette = AccentPalette::from_hex("#fab387");
        let css = palette.generate_gtk_css();
        assert!(css.contains("@define-color accent_color #fab387"));
        assert!(css.contains("--accent-color: #fab387"));
        assert!(css.contains(".dock-indicator-focused"));
    }

    #[test]
    fn test_apply_accent_color_files() {
        let tmp = std::env::temp_dir().join("ermete_accent_engine_test");
        std::env::set_var("XDG_CONFIG_HOME", &tmp);

        let res = apply_accent_color("#cba6f7");
        assert!(res.is_ok());

        let accent_file = get_accent_css_path();
        let theme_file = get_theme_css_path();

        assert!(accent_file.exists());
        assert!(theme_file.exists());

        let accent_content = std::fs::read_to_string(&accent_file).unwrap();
        assert!(accent_content.contains("#cba6f7"));
    }
}
