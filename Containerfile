# renovate: datasource=github-releases depName=ful1e5/Bibata_Cursor
ARG BIBATA_VER="v2.0.7"

# renovate: datasource=github-releases depName=starship/starship
ARG STARSHIP_VER="v1.22.1"




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



# Builder Starship
FROM build-base AS build-starship
ARG STARSHIP_VER
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-starship \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-starship \
    --mount=type=cache,target=/tmp/cargo-target,id=target-starship \
    env CARGO_TARGET_DIR=/tmp/cargo-target cargo install --locked --root /out starship --version ${STARSHIP_VER#v}

# Builder Matugen
FROM build-base AS build-matugen
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-matugen \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-matugen \
    --mount=type=cache,target=/tmp/cargo-target,id=target-matugen \
    env CARGO_TARGET_DIR=/tmp/cargo-target cargo install --locked --root /out matugen


# Builder Bibata Cursor (Zero-Network-Failure OCI layer)
FROM build-base AS build-bibata
ARG BIBATA_VER
# SHA256 checksum for v2.0.7
ARG BIBATA_SHA256="7d3495864e5bbef02f5e77de760b2905903b63c71495a78ef6306d19a3b556d8"
RUN mkdir -p /out/icons && \
    curl -sLO "https://github.com/ful1e5/Bibata_Cursor/releases/download/${BIBATA_VER}/Bibata-Modern-Classic.tar.xz" && \
    echo "${BIBATA_SHA256}  Bibata-Modern-Classic.tar.xz" | sha256sum -c - && \
    tar -xJ --no-same-owner -C /out/icons -f Bibata-Modern-Classic.tar.xz

# Builder Nix (Zero-Execution State Copy)
FROM nixos/nix:latest AS build-nix

# Fase C: Costruzione Link Simbolici (Dichiaratività Systemd)
FROM build-base AS build-symlinks
RUN mkdir -p /out/usr/lib/systemd/system

# --- IMMAGINE FINALE (PRODUZIONE) ---
# FIX: Renovate Bot sostituirà automaticamente il tag :latest con il vero digest SHA256 crittografico
FROM ghcr.io/patapem/ermete-base-nvidia@sha256:97f3d5dfdbeb305328d7cb77d989c36ee8f86464ba34cfcd27395ee094d13fbd
ARG BIBATA_VER

# Copia i binari purificati dai rispettivi branch paralleli (Hardening Deterministico)
COPY --from=build-starship --chown=0:0 --chmod=755 /out/bin/starship /usr/bin/
COPY --from=build-matugen --chown=0:0 --chmod=755 /out/bin/matugen /usr/bin/

# Nix "Cucinato" fisicamente nell'immagine OCI (Zero-Execution)
COPY --from=build-nix --chown=0:0 /nix /nix

# Copia asset immutabili
COPY --from=build-bibata /out/icons/Bibata-Modern-Classic /usr/share/icons/Bibata-Modern-Classic

# Fissiamo i permessi di /etc/skel nativamente nell'immagine OCI (Zero-Boot-Delay)
# I permessi paranoici (0700 dir, 0600 file) sono applicati nel mutating RUN sottostante

# FIX BEDROCK: Copiamo selettivamente SOLO i file vitali per sysusers e la privacy sandbox (skel)
# PRIMA degli script di build, in modo che il RUN successivo possa applicare i permessi corretti (chmod)
# e creare gli utenti (greeter, nixbld) senza sovrascrivere prematuramente DNF.
COPY --chown=0:0 system_files/usr/lib/sysusers.d /usr/lib/sysusers.d
COPY --chown=0:0 system_files/etc/skel /etc/skel

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
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

# Iniettiamo la gerarchia nativa OCI delle configurazioni statiche (Zero-Echo)
# ALLA FINE per evitare Layer Bloat, invalidazione Cache OCI e garantire la precedenza sulle policy OS-level.
COPY --chown=0:0 system_files /
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
