# I target delle versioni non sono più tracciati qui.
# Sono gestiti in completa autonomia dall'AI Bot di Ermete Forge.





# Builders have been entirely migrated to ermete-forge OCI micro-containers.

# --- IMMAGINE FINALE (PRODUZIONE) ---
# FIX: Renovate Bot sostituirà automaticamente il tag :latest con il vero digest SHA256 crittografico
FROM ghcr.io/patapem/ermete-base-nvidia:latest
# Estrazione pacchetti RPM puri dai Micro-Container OCI di Ermete Forge (Isolamento totale)
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-kernel
ARG ERMETE_FORGE_KERNEL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-kernel:${ERMETE_FORGE_KERNEL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-nvidia
ARG ERMETE_FORGE_NVIDIA_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-nvidia:${ERMETE_FORGE_NVIDIA_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-selinux
ARG ERMETE_FORGE_SELINUX_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-selinux:${ERMETE_FORGE_SELINUX_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-starship
ARG ERMETE_FORGE_STARSHIP_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-starship:${ERMETE_FORGE_STARSHIP_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-matugen
ARG ERMETE_FORGE_MATUGEN_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-matugen:${ERMETE_FORGE_MATUGEN_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-bibata
ARG ERMETE_FORGE_BIBATA_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-bibata:${ERMETE_FORGE_BIBATA_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-appmenu-glib-translator
ARG ERMETE_FORGE_APPMENU_GLIB_TRANSLATOR_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-appmenu-glib-translator:${ERMETE_FORGE_APPMENU_GLIB_TRANSLATOR_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-astal-io
ARG ERMETE_FORGE_ASTAL_IO_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-astal-io:${ERMETE_FORGE_ASTAL_IO_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-astal
ARG ERMETE_FORGE_ASTAL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-astal:${ERMETE_FORGE_ASTAL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-astal-libs
ARG ERMETE_FORGE_ASTAL_LIBS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-astal-libs:${ERMETE_FORGE_ASTAL_LIBS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-astal-gjs
ARG ERMETE_FORGE_ASTAL_GJS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-astal-gjs:${ERMETE_FORGE_ASTAL_GJS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-astal-gtk4
ARG ERMETE_FORGE_ASTAL_GTK4_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-astal-gtk4:${ERMETE_FORGE_ASTAL_GTK4_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-astal-lua
ARG ERMETE_FORGE_ASTAL_LUA_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-astal-lua:${ERMETE_FORGE_ASTAL_LUA_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-aylurs-gtk-shell
ARG ERMETE_FORGE_AYLURS_GTK_SHELL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-aylurs-gtk-shell:${ERMETE_FORGE_AYLURS_GTK_SHELL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-aylurs-gtk-shell2
ARG ERMETE_FORGE_AYLURS_GTK_SHELL2_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-aylurs-gtk-shell2:${ERMETE_FORGE_AYLURS_GTK_SHELL2_COMMIT} / /tmp/forge-rpms/

# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-hyprpanel
ARG ERMETE_FORGE_HYPRPANEL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-hyprpanel:${ERMETE_FORGE_HYPRPANEL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-ananicy-cpp
ARG ERMETE_FORGE_ANANICY_CPP_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-ananicy-cpp:${ERMETE_FORGE_ANANICY_CPP_COMMIT} / /tmp/forge-rpms/

# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-ags-config
ARG ERMETE_FORGE_AGS_CONFIG_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-ags-config:${ERMETE_FORGE_AGS_CONFIG_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-niri-session
ARG ERMETE_FORGE_NIRI_SESSION_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-niri-session:${ERMETE_FORGE_NIRI_SESSION_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-ide-bootstrap
ARG ERMETE_FORGE_IDE_BOOTSTRAP_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-ide-bootstrap:${ERMETE_FORGE_IDE_BOOTSTRAP_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-system-services
ARG ERMETE_FORGE_SYSTEM_SERVICES_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-system-services:${ERMETE_FORGE_SYSTEM_SERVICES_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-nix-support
ARG ERMETE_FORGE_NIX_SUPPORT_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-nix-support:${ERMETE_FORGE_NIX_SUPPORT_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-system-config
ARG ERMETE_FORGE_SYSTEM_CONFIG_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-system-config:${ERMETE_FORGE_SYSTEM_CONFIG_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-system-tweaks
ARG ERMETE_FORGE_SYSTEM_TWEAKS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-system-tweaks:${ERMETE_FORGE_SYSTEM_TWEAKS_COMMIT} / /tmp/forge-rpms/
# --- INIZIO PACCHETTI ROLLING (Bedrock Auto-Generato) ---
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-eza
ARG ERMETE_FORGE_ROLLING_EZA_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-eza:${ERMETE_FORGE_ROLLING_EZA_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-bat
ARG ERMETE_FORGE_ROLLING_BAT_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-bat:${ERMETE_FORGE_ROLLING_BAT_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-fd-find
ARG ERMETE_FORGE_ROLLING_FD_FIND_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-fd-find:${ERMETE_FORGE_ROLLING_FD_FIND_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-ripgrep
ARG ERMETE_FORGE_ROLLING_RIPGREP_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-ripgrep:${ERMETE_FORGE_ROLLING_RIPGREP_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-nushell
ARG ERMETE_FORGE_ROLLING_NUSHELL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-nushell:${ERMETE_FORGE_ROLLING_NUSHELL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-libvirt
ARG ERMETE_FORGE_ROLLING_LIBVIRT_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libvirt:${ERMETE_FORGE_ROLLING_LIBVIRT_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-niri
ARG ERMETE_FORGE_ROLLING_NIRI_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-niri:${ERMETE_FORGE_ROLLING_NIRI_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-pipewire
ARG ERMETE_FORGE_ROLLING_PIPEWIRE_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-pipewire:${ERMETE_FORGE_ROLLING_PIPEWIRE_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-adw-gtk3-theme
ARG ERMETE_FORGE_ROLLING_ADW_GTK3_THEME_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-adw-gtk3-theme:${ERMETE_FORGE_ROLLING_ADW_GTK3_THEME_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-bpftool
ARG ERMETE_FORGE_ROLLING_BPFTOOL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-bpftool:${ERMETE_FORGE_ROLLING_BPFTOOL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-brightnessctl
ARG ERMETE_FORGE_ROLLING_BRIGHTNESSCTL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-brightnessctl:${ERMETE_FORGE_ROLLING_BRIGHTNESSCTL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-btop
ARG ERMETE_FORGE_ROLLING_BTOP_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-btop:${ERMETE_FORGE_ROLLING_BTOP_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-btrfs-progs
ARG ERMETE_FORGE_ROLLING_BTRFS_PROGS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-btrfs-progs:${ERMETE_FORGE_ROLLING_BTRFS_PROGS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-dbus-tools
ARG ERMETE_FORGE_ROLLING_DBUS_TOOLS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-dbus-tools:${ERMETE_FORGE_ROLLING_DBUS_TOOLS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-dbus-x11
ARG ERMETE_FORGE_ROLLING_DBUS_X11_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-dbus-x11:${ERMETE_FORGE_ROLLING_DBUS_X11_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-drm_info
ARG ERMETE_FORGE_ROLLING_DRM_INFO_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-drm_info:${ERMETE_FORGE_ROLLING_DRM_INFO_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-ffmpeg
ARG ERMETE_FORGE_ROLLING_FFMPEG_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-ffmpeg:${ERMETE_FORGE_ROLLING_FFMPEG_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-file-roller
ARG ERMETE_FORGE_ROLLING_FILE_ROLLER_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-file-roller:${ERMETE_FORGE_ROLLING_FILE_ROLLER_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-firewalld
ARG ERMETE_FORGE_ROLLING_FIREWALLD_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-firewalld:${ERMETE_FORGE_ROLLING_FIREWALLD_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-fontawesome-fonts-all
ARG ERMETE_FORGE_ROLLING_FONTAWESOME_FONTS_ALL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-fontawesome-fonts-all:${ERMETE_FORGE_ROLLING_FONTAWESOME_FONTS_ALL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-foot
ARG ERMETE_FORGE_ROLLING_FOOT_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-foot:${ERMETE_FORGE_ROLLING_FOOT_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-gnome-keyring
ARG ERMETE_FORGE_ROLLING_GNOME_KEYRING_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gnome-keyring:${ERMETE_FORGE_ROLLING_GNOME_KEYRING_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-gnome-keyring-pam
ARG ERMETE_FORGE_ROLLING_GNOME_KEYRING_PAM_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gnome-keyring-pam:${ERMETE_FORGE_ROLLING_GNOME_KEYRING_PAM_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-greenboot
ARG ERMETE_FORGE_ROLLING_GREENBOOT_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-greenboot:${ERMETE_FORGE_ROLLING_GREENBOOT_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-greenboot-default-health-checks
ARG ERMETE_FORGE_ROLLING_GREENBOOT_DEFAULT_HEALTH_CHECKS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-greenboot-default-health-checks:${ERMETE_FORGE_ROLLING_GREENBOOT_DEFAULT_HEALTH_CHECKS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-greetd
ARG ERMETE_FORGE_ROLLING_GREETD_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-greetd:${ERMETE_FORGE_ROLLING_GREETD_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-gtk4-layer-shell
ARG ERMETE_FORGE_ROLLING_GTK4_LAYER_SHELL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gtk4-layer-shell:${ERMETE_FORGE_ROLLING_GTK4_LAYER_SHELL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-gtk-layer-shell
ARG ERMETE_FORGE_ROLLING_GTK_LAYER_SHELL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gtk-layer-shell:${ERMETE_FORGE_ROLLING_GTK_LAYER_SHELL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-gvfs
ARG ERMETE_FORGE_ROLLING_GVFS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gvfs:${ERMETE_FORGE_ROLLING_GVFS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-imv
ARG ERMETE_FORGE_ROLLING_IMV_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-imv:${ERMETE_FORGE_ROLLING_IMV_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-jetbrains-mono-fonts
ARG ERMETE_FORGE_ROLLING_JETBRAINS_MONO_FONTS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-jetbrains-mono-fonts:${ERMETE_FORGE_ROLLING_JETBRAINS_MONO_FONTS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-just
ARG ERMETE_FORGE_ROLLING_JUST_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-just:${ERMETE_FORGE_ROLLING_JUST_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-libnotify
ARG ERMETE_FORGE_ROLLING_LIBNOTIFY_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libnotify:${ERMETE_FORGE_ROLLING_LIBNOTIFY_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-libva-nvidia-driver
ARG ERMETE_FORGE_ROLLING_LIBVA_NVIDIA_DRIVER_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libva-nvidia-driver:${ERMETE_FORGE_ROLLING_LIBVA_NVIDIA_DRIVER_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-libva-utils
ARG ERMETE_FORGE_ROLLING_LIBVA_UTILS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libva-utils:${ERMETE_FORGE_ROLLING_LIBVA_UTILS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-mesa-dri-drivers
ARG ERMETE_FORGE_ROLLING_MESA_DRI_DRIVERS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-mesa-dri-drivers:${ERMETE_FORGE_ROLLING_MESA_DRI_DRIVERS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-mesa-vulkan-drivers
ARG ERMETE_FORGE_ROLLING_MESA_VULKAN_DRIVERS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-mesa-vulkan-drivers:${ERMETE_FORGE_ROLLING_MESA_VULKAN_DRIVERS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-mpv
ARG ERMETE_FORGE_ROLLING_MPV_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-mpv:${ERMETE_FORGE_ROLLING_MPV_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-nftables
ARG ERMETE_FORGE_ROLLING_NFTABLES_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-nftables:${ERMETE_FORGE_ROLLING_NFTABLES_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-nodejs
ARG ERMETE_FORGE_ROLLING_NODEJS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-nodejs:${ERMETE_FORGE_ROLLING_NODEJS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-npm
ARG ERMETE_FORGE_ROLLING_NPM_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-npm:${ERMETE_FORGE_ROLLING_NPM_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-papirus-icon-theme
ARG ERMETE_FORGE_ROLLING_PAPIRUS_ICON_THEME_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-papirus-icon-theme:${ERMETE_FORGE_ROLLING_PAPIRUS_ICON_THEME_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-parallel
ARG ERMETE_FORGE_ROLLING_PARALLEL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-parallel:${ERMETE_FORGE_ROLLING_PARALLEL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-playerctl
ARG ERMETE_FORGE_ROLLING_PLAYERCTL_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-playerctl:${ERMETE_FORGE_ROLLING_PLAYERCTL_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-qemu-kvm
ARG ERMETE_FORGE_ROLLING_QEMU_KVM_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-qemu-kvm:${ERMETE_FORGE_ROLLING_QEMU_KVM_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-qt5-qtwayland
ARG ERMETE_FORGE_ROLLING_QT5_QTWAYLAND_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-qt5-qtwayland:${ERMETE_FORGE_ROLLING_QT5_QTWAYLAND_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-qt6-qtwayland
ARG ERMETE_FORGE_ROLLING_QT6_QTWAYLAND_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-qt6-qtwayland:${ERMETE_FORGE_ROLLING_QT6_QTWAYLAND_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-rsms-inter-fonts
ARG ERMETE_FORGE_ROLLING_RSMS_INTER_FONTS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-rsms-inter-fonts:${ERMETE_FORGE_ROLLING_RSMS_INTER_FONTS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-swaybg
ARG ERMETE_FORGE_ROLLING_SWAYBG_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-swaybg:${ERMETE_FORGE_ROLLING_SWAYBG_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-swaylock
ARG ERMETE_FORGE_ROLLING_SWAYLOCK_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-swaylock:${ERMETE_FORGE_ROLLING_SWAYLOCK_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-sysstat
ARG ERMETE_FORGE_ROLLING_SYSSTAT_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-sysstat:${ERMETE_FORGE_ROLLING_SYSSTAT_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-thunar
ARG ERMETE_FORGE_ROLLING_THUNAR_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-thunar:${ERMETE_FORGE_ROLLING_THUNAR_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-thunar-archive-plugin
ARG ERMETE_FORGE_ROLLING_THUNAR_ARCHIVE_PLUGIN_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-thunar-archive-plugin:${ERMETE_FORGE_ROLLING_THUNAR_ARCHIVE_PLUGIN_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-thunar-volman
ARG ERMETE_FORGE_ROLLING_THUNAR_VOLMAN_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-thunar-volman:${ERMETE_FORGE_ROLLING_THUNAR_VOLMAN_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-tuigreet
ARG ERMETE_FORGE_ROLLING_TUIGREET_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-tuigreet:${ERMETE_FORGE_ROLLING_TUIGREET_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-upower
ARG ERMETE_FORGE_ROLLING_UPOWER_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-upower:${ERMETE_FORGE_ROLLING_UPOWER_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-virt-manager
ARG ERMETE_FORGE_ROLLING_VIRT_MANAGER_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-virt-manager:${ERMETE_FORGE_ROLLING_VIRT_MANAGER_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-wayland-utils
ARG ERMETE_FORGE_ROLLING_WAYLAND_UTILS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wayland-utils:${ERMETE_FORGE_ROLLING_WAYLAND_UTILS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-wireplumber
ARG ERMETE_FORGE_ROLLING_WIREPLUMBER_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wireplumber:${ERMETE_FORGE_ROLLING_WIREPLUMBER_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-wl-clipboard
ARG ERMETE_FORGE_ROLLING_WL_CLIPBOARD_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wl-clipboard:${ERMETE_FORGE_ROLLING_WL_CLIPBOARD_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-wl-mirror
ARG ERMETE_FORGE_ROLLING_WL_MIRROR_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wl-mirror:${ERMETE_FORGE_ROLLING_WL_MIRROR_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-wlr-randr
ARG ERMETE_FORGE_ROLLING_WLR_RANDR_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wlr-randr:${ERMETE_FORGE_ROLLING_WLR_RANDR_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-xdg-desktop-portal-gnome
ARG ERMETE_FORGE_ROLLING_XDG_DESKTOP_PORTAL_GNOME_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-desktop-portal-gnome:${ERMETE_FORGE_ROLLING_XDG_DESKTOP_PORTAL_GNOME_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-xdg-desktop-portal-gtk
ARG ERMETE_FORGE_ROLLING_XDG_DESKTOP_PORTAL_GTK_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-desktop-portal-gtk:${ERMETE_FORGE_ROLLING_XDG_DESKTOP_PORTAL_GTK_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-xdg-user-dirs
ARG ERMETE_FORGE_ROLLING_XDG_USER_DIRS_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-user-dirs:${ERMETE_FORGE_ROLLING_XDG_USER_DIRS_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-xdg-user-dirs-gtk
ARG ERMETE_FORGE_ROLLING_XDG_USER_DIRS_GTK_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-user-dirs-gtk:${ERMETE_FORGE_ROLLING_XDG_USER_DIRS_GTK_COMMIT} / /tmp/forge-rpms/
# renovate: datasource=docker depName=ghcr.io/patapem/ermete-forge-rolling-xorg-x11-server-xwayland
ARG ERMETE_FORGE_ROLLING_XORG_X11_SERVER_XWAYLAND_COMMIT="latest"
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xorg-x11-server-xwayland:${ERMETE_FORGE_ROLLING_XORG_X11_SERVER_XWAYLAND_COMMIT} / /tmp/forge-rpms/
# --- FINE PACCHETTI ROLLING ---

# (dart-sass rimosso, se necessario andrà creato un micro-container spec dedicato)

# Nix is now fetched from ghcr.io/patapem/ermete-forge-nix-support

# FIX BEDROCK: I file per sysusers e la privacy sandbox (skel) non vengono più
# iniettati crudi, ma sono nativamente pacchettizzati negli RPM (ermete-system-config, ecc.)

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    dnf5 install -y --allowerasing /tmp/forge-rpms/*.rpm && \
    rm -rf /tmp/forge-rpms

# (Nessun system_files iniettato. L'architettura Bedrock usa il 100% di astrazione via RPM OCI).
# Symlinks managed by system-config RPM



### NIX STATE (Immutability Fix)
# Creiamo il symlink immutabile sul rootfs verso il mountpoint effimero in /var.
# Il restore del database Nix (amnesia-fix) è gestito in modo nativo e dichiarativo
# da tmpfiles.d (10-ermete-nix.conf) che copia lo stato iniziale al boot.
# Initial state for Nix is now managed by ermete-forge-nix-support RPM

### DICHIARATIVITÀ ASSOLUTA (SYSTEMD PRESETS)
# Applichiamo nativamente tutti i file .preset (es. 99-Ermete.preset) 
# in modo che nix-daemon.socket e gli altri target vengano registrati
# all'interno dell'immagine OCI, prima dell'avvio su baremetal.
RUN systemctl preset-all && systemctl --global preset-all


RUN rm -f /etc/machine-id && touch /etc/machine-id && chmod 0444 /etc/machine-id && \
    rm -rf /etc/NetworkManager/system-connections/* && \
    dnf clean all

### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
