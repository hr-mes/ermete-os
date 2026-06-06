# Allow build scripts to be referenced without being copied into the final image
FROM scratch AS ctx
COPY build_files /
COPY dot_config /dot_config

# Base Image
FROM ghcr.io/rakuos/rakuos-base-nvidia:latest
RUN sed -i 's/^ID=.*/ID=fedora/' /etc/os-release

# Copy Homebrew files from the brew image
# And enable
COPY --from=ghcr.io/ublue-os/brew:latest /system_files /
RUN --mount=type=cache,dst=/var/cache \
    --mount=type=cache,dst=/var/log \
    --mount=type=tmpfs,dst=/tmp \
    /usr/bin/systemctl preset brew-setup.service && \
    /usr/bin/systemctl preset brew-update.timer && \
    /usr/bin/systemctl preset brew-upgrade.timer

# Execute all modular scripts in the recipes directory
RUN --mount=type=bind,from=ctx,source=/,target=/ctx \
    --mount=type=cache,dst=/var/cache \
    --mount=type=cache,dst=/var/log \
    --mount=type=tmpfs,dst=/tmp \
    for script in /ctx/recipes/*.sh; do \
        echo "Executing $script..."; \
        bash "$script"; \
    done

### LINTING
## Verify final image and contents are correct.
RUN bootc container lint
