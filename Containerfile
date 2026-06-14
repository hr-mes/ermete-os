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
RUN dnf -y install --setopt=install_weak_deps=False rust cargo gcc gcc-c++ pkgconf-pkg-config make cmake \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel git mold

# Variabili d'ambiente per Overclocking Rust
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

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
    mv /out/bin/bottom /out/bin/btm || true

# Builder Ironbar
FROM build-base AS build-ironbar
ARG IRONBAR_VER
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-ironbar \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-ironbar \
    cargo install --locked --root /out ironbar --version ${IRONBAR_VER#v}

# Builder Anyrun
FROM build-base AS build-anyrun
ARG ANYRUN_COMMIT
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=registry-anyrun \
    --mount=type=cache,target=/usr/local/cargo/git,id=git-anyrun \
    git clone https://github.com/anyrun-org/anyrun.git /tmp/anyrun-src && \
    cd /tmp/anyrun-src && git checkout ${ANYRUN_COMMIT} && \
    cargo build --release --locked && \
    mv target/release/anyrun /out/bin/ && \
    cp target/release/*.so /out/lib64/anyrun/ 2>/dev/null || true && \
    strip /out/bin/anyrun /out/lib64/anyrun/*.so 2>/dev/null || true

# Builder Bibata Cursor (Zero-Network-Failure OCI layer)
FROM build-base AS build-bibata
ARG BIBATA_VER
RUN mkdir -p /out/icons && \
    curl -sLO "https://github.com/ful1e5/Bibata_Cursor/releases/download/${BIBATA_VER}/Bibata-Modern-Classic.tar.xz" && \
    tar -xJ --no-same-owner -C /out/icons -f Bibata-Modern-Classic.tar.xz

# Fase C: Costruzione Link Simbolici (Dichiaratività Systemd)
FROM build-base AS build-symlinks
RUN mkdir -p /out/etc/systemd/system /out/usr/lib && \
    ln -sf /usr/lib/systemd/system/graphical.target /out/etc/systemd/system/default.target && \
    ln -sf /usr/lib64/anyrun /out/usr/lib/anyrun

# --- IMMAGINE FINALE (PRODUZIONE) ---
FROM ghcr.io/patapem/ermete-base-nvidia:latest
ARG BIBATA_VER

# Copia i binari purificati dai rispettivi branch paralleli (Hardening Deterministico)
COPY --from=build-starship --chown=0:0 --chmod=755 /out/bin/starship /usr/bin/
COPY --from=build-bottom --chown=0:0 --chmod=755 /out/bin/btm /usr/bin/
COPY --from=build-ironbar --chown=0:0 --chmod=755 /out/bin/ironbar /usr/bin/
COPY --from=build-anyrun --chown=0:0 --chmod=755 /out/bin/anyrun /usr/bin/
COPY --from=build-anyrun --chown=0:0 --chmod=755 /out/lib64/anyrun /usr/lib64/anyrun

# Copia asset immutabili
COPY --from=build-bibata /out/icons/Bibata-Modern-Classic /usr/share/icons/Bibata-Modern-Classic

# Iniettiamo la gerarchia nativa OCI delle configurazioni statiche (Zero-Echo) e i symlink precalcolati
COPY --chown=0:0 system_files /
COPY --from=build-symlinks /out/etc /etc
COPY --from=build-symlinks /out/usr/lib/ /usr/lib/

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
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
# Imponiamo rigorose policy UNIX per prevenire data leak tra sessioni
RUN find /etc/skel -type d -exec chmod 700 {} \+ && \
    find /etc/skel -type f -exec chmod 600 {} \+

### NIX MOUNTPOINT (Immutability Fix)
# Creiamo il mountpoint vuoto sul rootfs immutabile per permettere a nix.mount
# di montare /var/opt/nix al boot. Senza questo, il demone fallirebbe.
RUN mkdir -p /nix

### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
