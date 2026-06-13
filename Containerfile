# Allow build scripts to be referenced without being copied into the final image
FROM scratch AS ctx
# FIX: Applica permessi sicuri già nello stage rootless per evitare LPE nei mount bindati
COPY --chown=0:0 build_files /

# Base Image
FROM ghcr.io/patapem/ermete-base-nvidia:latest

# renovate: datasource=github-releases depName=ful1e5/Bibata_Cursor
ARG BIBATA_VER="v2.0.7"
ARG BIBATA_HASH="7d3495864e5bbef02f5e77de760b2905903b63c71495a78ef6306d19a3b556d8"

# renovate: datasource=github-releases depName=JakeStanger/ironbar
ARG IRONBAR_VER="v0.19.0"
ARG IRONBAR_HASH="5efbbfd79c3f97364d254c12abef24618375a2759c25c883812d23c3581081e9"

# renovate: datasource=github-releases depName=starship/starship
ARG STARSHIP_VER="v1.22.1"
ARG STARSHIP_HASH="e57db6f6497ee8a426c5e77b4d6f5c50734d3e9cca7a18a8aef46730505a3ae7"

# renovate: datasource=github-releases depName=ClementTsang/bottom
ARG BOTTOM_VER="0.10.2"
ARG BOTTOM_HASH="f20211d398b9744545b93ac4af73f3a9f3e67179c385fb0c73d0dd4d84d28a8f"

# renovate: datasource=github-tags depName=anyrun-org/anyrun
ARG ANYRUN_COMMIT="f3b23bc5520f7673a5119da44b3570fbe060db37"
ARG ANYRUN_HASH="11ac878a0e67025b4f439f0c14b8d87125c00aa573625fae0a35383fe7c18b95"

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
## Sterilizzazione Permessi Configurazione (Policy UNIX)
RUN chmod 700 /etc/skel \
 && chmod 700 /etc/skel/.config \
 && find /etc/skel/.config -type f -exec chmod 600 {} + \
 && find /etc/skel/.config -type d -exec chmod 700 {} +

## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
