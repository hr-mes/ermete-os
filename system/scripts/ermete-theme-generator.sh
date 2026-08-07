#!/usr/bin/env bash
# 🦅 Ermete OS - Material 3 Dynamic Theme Generator Pipeline via Matugen
# Extract Material 3 color palettes from wallpapers and generate GTK4 @define-color CSS.

set -euo pipefail

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/ermete"
THEME_CSS="$CONFIG_DIR/theme.css"
mkdir -p "$CONFIG_DIR"

WALLPAPER_PATH="${1:-}"
MODE="${2:-dark}"

# Fallback to reading wallpaper path from appearance.json if not provided
if [[ -z "$WALLPAPER_PATH" && -f "$CONFIG_DIR/appearance.json" ]]; then
    WALLPAPER_PATH=$(jq -r '.wallpaper // empty' "$CONFIG_DIR/appearance.json" 2>/dev/null || true)
    SCHEME=$(jq -r '.color_scheme // empty' "$CONFIG_DIR/appearance.json" 2>/dev/null || true)
    if [[ "$SCHEME" == "default" ]]; then
        MODE="light"
    elif [[ "$SCHEME" == "prefer-dark" ]]; then
        MODE="dark"
    fi
fi

if [[ -z "$WALLPAPER_PATH" ]]; then
    WALLPAPER_PATH="/usr/share/backgrounds/ermete-default.png"
fi

if [[ "$MODE" != "light" && "$MODE" != "dark" ]]; then
    if [[ "$MODE" == "prefer-dark" ]]; then
        MODE="dark"
    else
        MODE="dark"
    fi
fi

# Locate Matugen template
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_PATH="$SCRIPT_DIR/matugen_theme.template"

if [[ ! -f "$TEMPLATE_PATH" ]]; then
    TEMPLATE_PATH="$CONFIG_DIR/matugen_theme.template"
fi

# Apply wallpaper via swww if swww is running or available
if command -v swww >/dev/null 2>&1; summit_swww=1; then
    if swww query >/dev/null 2>&1; then
        if [[ -f "$WALLPAPER_PATH" ]]; then
            swww img "$WALLPAPER_PATH" --transition-type outer --transition-step 90 || true
        fi
    fi
fi

# Generate theme.css via Matugen if installed and wallpaper exists
if command -v matugen >/dev/null 2>&1 && [[ -f "$WALLPAPER_PATH" && -f "$TEMPLATE_PATH" ]]; then
    TMP_CFG=$(mktemp /tmp/matugen_ermete_XXXXXX.toml)
    cat << EOF > "$TMP_CFG"
[config]
reload_apps = false

[templates.gtk4_theme]
input_path = "$TEMPLATE_PATH"
output_path = "$THEME_CSS"
EOF

    if matugen image "$WALLPAPER_PATH" --source-color-index 0 --mode "$MODE" -c "$TMP_CFG" >/dev/null 2>&1; then
        echo "Successfully generated dynamic Material 3 theme at $THEME_CSS"
        rm -f "$TMP_CFG"
        exit 0
    fi
    rm -f "$TMP_CFG"
fi

# Fallback: Write default Material 3 Dark theme if matugen fails or wallpaper is missing
cat << 'EOF' > "$THEME_CSS"
/* Material 3 Fallback Dynamic Theme for Ermete OS */
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
EOF

echo "Wrote fallback Material 3 theme to $THEME_CSS"
