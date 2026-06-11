# Allow build scripts to be referenced without being copied into the final image
FROM scratch AS ctx
# FIX: Applica permessi sicuri già nello stage rootless per evitare LPE nei mount bindati
COPY --chown=0:0 build_files /

# Base Image
FROM ghcr.io/patapem/ermete-base-nvidia:latest
RUN sed -i 's/^ID=.*/ID=fedora/' /etc/os-release

# Copy Homebrew files from the brew image
# FIX: Aggiunto --chown=0:0 per coerenza di sicurezza sui binari iniettati
COPY --from=ghcr.io/ublue-os/brew:latest --chown=0:0 /system_files /

# Copy Nix environment (binaries, sysusers, systemd services, SELinux policies)
# Questa operazione bypassa la rottura di RPM in presenza del symlink immutabile /nix.
COPY --from=ghcr.io/ublue-os/nix-init:latest --chown=0:0 / /

RUN --mount=type=cache,dst=/var/cache \
    --mount=type=cache,dst=/var/log \
    --mount=type=tmpfs,dst=/tmp \
    /usr/bin/systemctl preset brew-setup.service && \
    /usr/bin/systemctl preset brew-update.timer && \
    /usr/bin/systemctl preset brew-upgrade.timer && \
    /usr/bin/systemctl preset nix-daemon.socket && \
    /usr/bin/systemctl preset nix-daemon.service

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

### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
