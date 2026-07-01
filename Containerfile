# I target delle versioni non sono più tracciati qui.
# Sono gestiti in completa autonomia dall'AI Bot di Ermete Forge.




# Allow build scripts to be referenced without being copied into the final image
FROM scratch AS ctx
# FIX: Applica permessi sicuri già nello stage rootless per evitare LPE nei mount bindati
COPY --chown=0:0 build_files /


# --- NUOVA FABBRICA: PARALLEL MULTI-STAGE BUILDERS ---
# Stage di base con tutte le dipendenze per velocizzare i successivi build
# Digest pinning guarantees 100% layer cache hit on GitHub Actions, reducing compilation time.
FROM registry.fedoraproject.org/fedora:43@sha256:adf1fd5fe1633c7553028ee91b4d0e29c814fbe91b813b21e87bcedeb6c4d915 AS build-base
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    dnf -y install --setopt=install_weak_deps=False rust cargo gcc gcc-c++ pkgconf-pkg-config make cmake \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel git mold

# Variabili d'ambiente per Overclocking Rust
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTFLAGS="-C target-cpu=x86-64-v3 -C link-arg=-fuse-ld=mold"

RUN mkdir -p /out/bin



# Gli stage build-starship, build-matugen e build-bibata sono stati amputati.
# =========================================================================
# FASE 2: NIX BUILDER (Per pacchetti specifici Nix se necessari)
# =========================================================================
FROM docker.io/nixos/nix:latest AS build-nix

# Fase C: Costruzione Link Simbolici (Dichiaratività Systemd)
FROM build-base AS build-symlinks
RUN mkdir -p /out/usr/lib/systemd/system

# --- IMMAGINE FINALE (PRODUZIONE) ---
# FIX: Renovate Bot sostituirà automaticamente il tag :latest con il vero digest SHA256 crittografico
FROM ghcr.io/patapem/ermete-base-nvidia:latest
# Estrazione pacchetti RPM puri dai Micro-Container OCI di Ermete Forge (Isolamento totale)
COPY --from=ghcr.io/patapem/ermete-forge-kernel:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-nvidia:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-selinux:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-starship:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-matugen:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-bibata:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-appmenu-glib-translator:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-astal-io:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-astal:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-astal-libs:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-astal-gjs:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-astal-gtk4:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-astal-lua:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-aylurs-gtk-shell:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-aylurs-gtk-shell2:latest / /tmp/forge-rpms/

COPY --from=ghcr.io/patapem/ermete-forge-hyprpanel:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-ananicy-cpp:latest / /tmp/forge-rpms/

COPY --from=ghcr.io/patapem/ermete-forge-ags-config:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-niri-session:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-ide-bootstrap:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-system-services:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-nix-support:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-system-config:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-system-tweaks:latest / /tmp/forge-rpms/
# --- INIZIO PACCHETTI ROLLING (Bedrock Auto-Generato) ---
COPY --from=ghcr.io/patapem/ermete-forge-rolling-eza:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-bat:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-fd-find:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-ripgrep:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-nushell:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libvirt:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-niri:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-pipewire:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-adw-gtk3-theme:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-bpftool:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-brightnessctl:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-btop:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-btrfs-progs:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-dbus-tools:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-dbus-x11:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-drm_info:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-ffmpeg:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-file-roller:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-firewalld:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-fontawesome-fonts-all:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-foot:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gnome-keyring:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gnome-keyring-pam:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-greenboot:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-greenboot-default-health-checks:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-greetd:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gtk4-layer-shell:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gtk-layer-shell:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-gvfs:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-imv:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-jetbrains-mono-fonts:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-just:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libnotify:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libva-nvidia-driver:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-libva-utils:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-mesa-dri-drivers:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-mesa-vulkan-drivers:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-mpv:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-nftables:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-nodejs:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-npm:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-papirus-icon-theme:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-parallel:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-playerctl:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-qemu-kvm:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-qt5-qtwayland:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-qt6-qtwayland:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-rsms-inter-fonts:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-swaybg:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-swaylock:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-sysstat:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-thunar:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-thunar-archive-plugin:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-thunar-volman:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-tuigreet:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-upower:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-virt-manager:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wayland-utils:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wireplumber:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wl-clipboard:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wl-mirror:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-wlr-randr:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-desktop-portal-gnome:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-desktop-portal-gtk:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-user-dirs:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xdg-user-dirs-gtk:latest / /tmp/forge-rpms/
COPY --from=ghcr.io/patapem/ermete-forge-rolling-xorg-x11-server-xwayland:latest / /tmp/forge-rpms/
# --- FINE PACCHETTI ROLLING ---

# (dart-sass rimosso, se necessario andrà creato un micro-container spec dedicato)

# Nix "Cucinato" fisicamente nell'immagine OCI (Zero-Execution)
COPY --from=build-nix --chown=0:0 /nix /nix

# Fissiamo i permessi di /etc/skel nativamente nell'immagine OCI (Zero-Boot-Delay)
# I permessi paranoici (0700 dir, 0600 file) sono applicati nel mutating RUN sottostante

# FIX BEDROCK: I file per sysusers e la privacy sandbox (skel) non vengono più
# iniettati crudi, ma sono nativamente pacchettizzati negli RPM (ermete-system-config, ecc.)
# Il RUN successivo li installerà atomicamente via dnf5 e applicherà i permessi.

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    mkdir -p /tmp/override-rpms && \
    mv /tmp/forge-rpms/aylurs-gtk-shell2*.rpm /tmp/override-rpms/ || true && \
    mv /tmp/forge-rpms/hyprpanel*.rpm /tmp/override-rpms/ || true && \
    mv /tmp/forge-rpms/ermete-system-config*.rpm /tmp/override-rpms/ || true && \
    mv /tmp/forge-rpms/ermete-niri-session*.rpm /tmp/override-rpms/ || true && \
    mv /tmp/forge-rpms/ermete-system-tweaks*.rpm /tmp/override-rpms/ || true && \
    dnf5 install -y --allowerasing /tmp/forge-rpms/*.x86_64.rpm /tmp/forge-rpms/*.noarch.rpm && \
    rpm -Uvh --replacefiles --force --nodeps /tmp/override-rpms/*.rpm && \
    rm -rf /tmp/forge-rpms /tmp/override-rpms && \
    rm -rf /usr/lib/modules/6.* && \
    mkdir -p /etc/systemd && rm -rf /etc/systemd/system.control && ln -s /dev/null /etc/systemd/system.control && \
    find /etc/skel -type d -exec chmod 0700 {} + && \
    find /etc/skel -type f -exec chmod 0600 {} + && \
    find /etc/skel -type f -name "*.sh" -exec chmod 0700 {} + && \
    find /usr/libexec -type f -name "*.sh" -exec chmod +x {} + && \
    find /usr/bin -type f -name "ermete-*" -exec chmod +x {} + && \
    ln -sf /dev/null /usr/lib/systemd/system/akmods-keygen@.service && \
    ln -sf /dev/null /usr/lib/systemd/system/akmods@.service && \
    for p in /nix/store/*/bin/nix; do if [ -e "$p" ]; then NIX_BIN_DIR=$(dirname "$p"); break; fi; done && \
    if [ -n "$NIX_BIN_DIR" ]; then cp -a $NIX_BIN_DIR/* /usr/bin/ || true; fi && \
    bash /ctx/recipes/01-system-setup.sh && \
    bash /ctx/recipes/02-repos-and-codecs.sh && \
    bash /ctx/recipes/03-desktop.sh && \
    chmod +x /usr/bin/niri-session && \
    if [ -f /usr/bin/firefox ]; then chmod +x /usr/bin/firefox; fi && \
    if [ -f /usr/bin/tuigreet ]; then chmod +x /usr/bin/tuigreet; fi

# (Nessun system_files iniettato. L'architettura Bedrock usa il 100% di astrazione via RPM OCI).
# I symlink precalcolati
COPY --from=build-symlinks --chown=0:0 /out/usr /usr

### STRUMENTI DIAGNOSTICI OMNI-VISION SUPREME
# Installazione pacchetti essenziali per il debugging a Raggi-X 
# (Consolidati nel layer 01-system-setup.sh per purezza OCI e riduzione layer bloat)

### ASSETS SICURI E PREPARAZIONE
# La directory assets/sfondi è creata nativamente via system_files
# per evitare LPE e layer bloat imperativo.
# NOTA: Per scompattare eventuali archivi futuri, usare SEMPRE:
# RUN tar -xzf /path/to/assets.tar.gz -C /usr/share/backgrounds/ermete --no-same-owner

### PRIVACY SANDBOXING (/etc/skel)
# Le policy UNIX paranoiche sono fissate nativamente nel Containerfile.

### NIX STATE (Immutability Fix)
# Creiamo il symlink immutabile sul rootfs verso il mountpoint effimero in /var.
# Il restore del database Nix (amnesia-fix) è gestito in modo nativo e dichiarativo
# da tmpfiles.d (10-ermete-nix.conf) che copia lo stato iniziale al boot.
RUN mkdir -p /usr/share/nix-initial-state/var/nix/profiles && \
    for p in /nix/store/*/bin/nix; do if [ -e "$p" ]; then NIX_BIN_DIR=$(dirname "$p"); break; fi; done && \
    if [ -n "$NIX_BIN_DIR" ]; then ln -sf $(dirname $NIX_BIN_DIR) /usr/share/nix-initial-state/var/nix/profiles/default; fi && \
    mkdir -p /nix/var && chmod 0755 /nix/var

### DICHIARATIVITÀ ASSOLUTA (SYSTEMD PRESETS)
# Applichiamo nativamente tutti i file .preset (es. 99-Ermete.preset) 
# in modo che nix-daemon.socket e gli altri target vengano registrati
# all'interno dell'immagine OCI, prima dell'avvio su baremetal.
RUN systemctl preset-all && systemctl --global preset-all

### FIX SELINUX NIX (Bedrock LPE Mitigation)
# SELinux non conosce la directory /nix, assegnandole default_t, bloccando l'esecuzione del demone.
# Mappiamo in modo equivalente /nix a /usr per ereditare correttamente le regole bin_t, lib_t, ecc.
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf \
    dnf install -y policycoreutils-python-utils || true && \
    mv /etc/selinux /etc/selinux.bak && cp -a /etc/selinux.bak /etc/selinux && \
    mv /var/lib/selinux /var/lib/selinux.bak && cp -a /var/lib/selinux.bak /var/lib/selinux && \
    semanage fcontext -a -e /usr /nix && \
    semanage permissive -a greetd_t || true && \
    rm -rf /etc/selinux.bak /var/lib/selinux.bak && \
    dnf remove -y policycoreutils-python-utils && \
    authselect select local with-silent-lastlog with-mdns4 without-nullok --force && \
    rm -f /etc/machine-id && touch /etc/machine-id && chmod 0444 /etc/machine-id && \
    rm -rf /etc/NetworkManager/system-connections/* && \
    dnf clean all

### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
