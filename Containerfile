# Allow build scripts to be referenced without being copied into the final image
FROM scratch AS ctx
# FIX: Applica permessi sicuri già nello stage rootless per evitare LPE nei mount bindati
COPY --chown=0:0 build_files /

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

# --- NUOVA FABBRICA: MULTI-STAGE BUILDER ---
FROM registry.fedoraproject.org/fedora:43 AS builder
ARG IRONBAR_VER
ARG STARSHIP_VER
ARG BOTTOM_VER
ARG ANYRUN_COMMIT

# Installazione dipendenze di build in un unico layer cachato
RUN dnf -y install --setopt=install_weak_deps=False rust cargo gcc gcc-c++ pkgconf-pkg-config make cmake \
    glib2-devel gtk3-devel gtk4-devel gtk-layer-shell-devel gtk4-layer-shell-devel \
    cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
    autoconf automake libtool libevdev-devel upower-devel pulseaudio-libs-devel \
    libxkbcommon-devel wayland-devel openssl-devel luajit-devel clang \
    libinput-devel wayland-protocols-devel dbus-devel git

# Creazione delle directory di output
RUN mkdir -p /out/bin /out/lib64/anyrun

# Compilazione indipendente per Caching Granulare
RUN cargo install --locked --root /out starship --version ${STARSHIP_VER#v}
RUN cargo install --locked --root /out bottom --version ${BOTTOM_VER#v} && \
    mv /out/bin/bottom /out/bin/btm || true
RUN cargo install --locked --root /out ironbar --version ${IRONBAR_VER#v}

# Compilazione Anyrun
RUN git clone https://github.com/anyrun-org/anyrun.git /tmp/anyrun-src && \
    cd /tmp/anyrun-src && git checkout ${ANYRUN_COMMIT} && \
    cargo build --release --locked && \
    mv target/release/anyrun /out/bin/ && \
    cp target/release/*.so /out/lib64/anyrun/ 2>/dev/null || true && \
    strip /out/bin/anyrun /out/lib64/anyrun/*.so 2>/dev/null || true

# --- IMMAGINE FINALE (PRODUZIONE) ---
FROM ghcr.io/patapem/ermete-base-nvidia:latest
ARG BIBATA_VER

# Copia i binari purificati dalla fabbrica
COPY --from=builder /out/bin/* /usr/bin/
COPY --from=builder /out/lib64/anyrun /usr/lib64/anyrun
RUN ln -s /usr/lib64/anyrun /usr/lib/anyrun




RUN sed -i 's/^ID=.*/ID=fedora/' /etc/os-release

# Copy Homebrew files from the brew image
# FIX: Aggiunto --chown=0:0 per coerenza di sicurezza sui binari iniettati
COPY --from=ghcr.io/ublue-os/brew@sha256:5228826790d13d5e265f1fdbb41b65e3fac20361ee0d31b2fd496d81e1db14f6 --chown=0:0 /system_files /

RUN --mount=type=cache,dst=/var/cache \
    --mount=type=cache,dst=/var/log \
    --mount=type=tmpfs,dst=/tmp \
    /usr/bin/systemctl preset brew-setup.service && \
    /usr/bin/systemctl preset brew-update.timer && \
    /usr/bin/systemctl preset brew-upgrade.timer

# Execute all modular scripts sequentially to preserve OCI caching per-layer
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/01-system-setup.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/02-repos-and-codecs.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/03-desktop.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/04-system-config.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/04b-private-optimizations.sh

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/04c-kernel-tuning.sh

### STRUMENTI DIAGNOSTICI OMNI-VISION SUPREME
# Installazione pacchetti essenziali per il debugging a Raggi-X (Zero-Trust Supply Chain ok via DNF repo locale)
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    dnf -y install bpftool drm_info nftables wayland-utils

# FASE B: Rettifica Wayland Lifecycle
RUN if [ -f /etc/greetd/config.toml ]; then sed -i 's/command = "niri"/command = "niri-session"/g' /etc/greetd/config.toml; fi

RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    bash /ctx/recipes/05-cleanup.sh

### ASSETS SICURI E PREPARAZIONE
# Creazione sicura della directory assets/sfondi per evitare LPE
RUN mkdir -p /usr/share/backgrounds/ermete \
    && chown -R 0:0 /usr/share/backgrounds/ermete \
    && chmod 755 /usr/share/backgrounds/ermete
# NOTA: Per scompattare eventuali archivi futuri, usare SEMPRE:
# RUN tar -xzf /path/to/assets.tar.gz -C /usr/share/backgrounds/ermete --no-same-owner

### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
