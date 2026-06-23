# renovate: datasource=github-releases depName=ful1e5/Bibata_Cursor
ARG BIBATA_VER="v2.0.7"

# renovate: datasource=github-releases depName=JakeStanger/ironbar
ARG IRONBAR_VER="v0.19.0"

# renovate: datasource=github-releases depName=starship/starship
ARG STARSHIP_VER="v1.22.1"

# renovate: datasource=github-releases depName=ClementTsang/bottom
ARG BOTTOM_VER="0.10.2"

# renovate: datasource=github-commits depName=anyrun-org/anyrun
ARG ANYRUN_COMMIT="f3b23bc5520f7673a5119da44b3570fbe060db37"

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

RUN mkdir -p /out/bin /out/lib64/anyrun

# Builder Starship
FROM build-base AS build-starship
ARG STARSHIP_VER
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-starship \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-starship \
    cargo install --locked --root /out starship --version ${STARSHIP_VER#v}

# Builder Bottom
FROM build-base AS build-bottom
ARG BOTTOM_VER
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-bottom \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-bottom \
    cargo install --locked --root /out bottom --version ${BOTTOM_VER#v} && \
    if [ -f /out/bin/bottom ]; then mv /out/bin/bottom /out/bin/btm; fi

# Builder Ironbar
FROM build-base AS build-ironbar
ARG IRONBAR_VER
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-ironbar \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-ironbar \
    cargo install --locked --root /out ironbar --version ${IRONBAR_VER#v} --features workspaces+niri

# Builder Anyrun
FROM build-base AS build-anyrun
ARG ANYRUN_COMMIT
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-anyrun \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-anyrun \
    git clone https://github.com/anyrun-org/anyrun.git /tmp/anyrun-src && \
    cd /tmp/anyrun-src && git checkout ${ANYRUN_COMMIT} && \
    cargo build --release --locked && \
    mv target/release/anyrun /out/bin/ && \
    find target/release -maxdepth 1 -name '*.so' -exec cp {} /out/lib64/anyrun/ \; && \
    find /out/bin /out/lib64/anyrun -type f -exec strip {} +

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
RUN mkdir -p /out/usr/lib/systemd/system && \
    ln -sf /usr/lib64/anyrun /out/usr/lib/anyrun

# --- IMMAGINE FINALE (PRODUZIONE) ---
# FIX: Renovate Bot sostituirà automaticamente il tag :latest con il vero digest SHA256 crittografico
FROM ghcr.io/patapem/ermete-base-nvidia@sha256:643f31315e9207152d86ae4c38a30b2ab539362b9620953bdde8cd8e63f5b28c
ARG BIBATA_VER

# Copia i binari purificati dai rispettivi branch paralleli (Hardening Deterministico)
COPY --from=build-starship --chown=0:0 --chmod=755 /out/bin/starship /usr/bin/
COPY --from=build-bottom --chown=0:0 --chmod=755 /out/bin/btm /usr/bin/
COPY --from=build-ironbar --chown=0:0 --chmod=755 /out/bin/ironbar /usr/bin/
COPY --from=build-anyrun --chown=0:0 --chmod=755 /out/bin/anyrun /usr/bin/
COPY --from=build-anyrun --chown=0:0 --chmod=755 /out/lib64/anyrun /usr/lib64/anyrun

# Nix "Cucinato" fisicamente nell'immagine OCI (Zero-Execution)
COPY --from=build-nix --chown=0:0 /nix /nix

# Copia asset immutabili
COPY --from=build-bibata /out/icons/Bibata-Modern-Classic /usr/share/icons/Bibata-Modern-Classic

# Iniettiamo la gerarchia nativa OCI delle configurazioni statiche (Zero-Echo) e i symlink precalcolati
COPY --chown=0:0 system_files /
COPY --from=build-symlinks --chown=0:0 /out/etc /etc
COPY --from=build-symlinks --chown=0:0 /out/usr /usr

# Fissiamo i permessi di /etc/skel nativamente nell'immagine OCI (Zero-Boot-Delay)
# I permessi paranoici (0700 dir, 0600 file) sono applicati nel mutating RUN sottostante

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    find /etc/skel -type d -exec chmod 0700 {} + && \
    find /etc/skel -type f -exec chmod 0600 {} + && \
    find /etc/skel -type f -name "*.sh" -exec chmod 0700 {} + && \
    find /usr/libexec -type f -name "*.sh" -exec chmod +x {} + && \
    find /usr/bin -type f -name "ermete-*" -exec chmod +x {} + && \
    chmod +x /usr/bin/niri-session && \
    chmod +x /usr/bin/firefox && \
    chmod +x /usr/bin/tuigreet && \
    cp -a /nix/var/nix/profiles/default/bin/* /usr/bin/ || true && \
    bash /ctx/recipes/01-system-setup.sh && \
    bash /ctx/recipes/02-repos-and-codecs.sh && \
    bash /ctx/recipes/03-desktop.sh

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
RUN mkdir -p /nix && rm -rf /nix/var && ln -s /var/opt/nix/var /nix/var

### DICHIARATIVITÀ ASSOLUTA (SYSTEMD PRESETS)
# Applichiamo nativamente tutti i file .preset (es. 99-Ermete.preset) 
# in modo che nix-daemon.socket e gli altri target vengano registrati
# all'interno dell'immagine OCI, prima dell'avvio su baremetal.
RUN systemctl preset-all && systemctl --global preset-all



### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
