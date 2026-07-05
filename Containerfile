# I target delle versioni non sono più tracciati qui.
# Sono gestiti in completa autonomia dall'AI Bot di Ermete Forge.
# Builders and Bedrock layers have been entirely migrated to ermete-forge OCI micro-containers.

# --- IMMAGINE FINALE (PRODUZIONE AUTARCHICA ATOMICA) ---
# FIX: Renovate Bot sostituirà automaticamente il tag con il vero digest SHA256 crittografico
FROM quay.io/fedora-ostree-desktops/base-atomic:43@sha256:c241a27d84bce85b29aa3be5e5dc0ecd8924580536d7ea66cb1c44cd2f0eac32
ARG FEDORA_VERSION=43
ENV FEDORA_VERSION=$FEDORA_VERSION

# Aggiornamento globale disabilitato temporaneamente per prevenire conflitti seccomp con glibc in GitHub Actions.
RUN sed -i '/tsflags=nodocs/d' /etc/dnf/dnf.conf /etc/dnf/dnf5.conf || true
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    dnf upgrade -y --setopt=install_weak_deps=False

# Epurazione Totale (Bedrock) del kernel Fedora upstream per impedire a dracut o dnf di intercettare versioni fantasma
RUN dnf5 -y remove --no-autoremove kernel kernel-core kernel-modules kernel-modules-core kernel-modules-extra kernel-tools kernel-tools-libs zram-generator-defaults && \
    rm -rf /lib/modules/* /usr/lib/modules/*

# Estrazione pacchetti RPM puri dai Micro-Container OCI di Ermete Forge (Isolamento totale)
COPY --from=ghcr.io/patapem/ermete-forge-repo:latest / /tmp/forge-repo

# (dart-sass rimosso, se necessario andrà creato un micro-container spec dedicato)
# Nix is now fetched from ghcr.io/patapem/ermete-forge-nix-support
# FIX BEDROCK: I file per sysusers e la privacy sandbox (skel) non vengono più
# iniettati crudi, ma sono nativamente pacchettizzati negli RPM (ermete-system-config, ecc.)

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
# FIX BEDROCK (Universal Package Resolution & Zero-Conflict Ordering):
# 1) We install ONLY ermete-base-config first via rpm -Uvh so that repository definitions
#    (/etc/yum.repos.d/rpmfusion.repo, gpg keys, etc.) are injected into the rootfs.
# 2) We move all remaining custom ermete-*.rpm packages (which overwrite upstream config files
#    like /etc/greetd/config.toml) to /tmp/forge-custom so they do not conflict with upstream DNF installations.
# 3) We run dnf5 install to download and resolve all upstream packages and external dependencies.
# 4) Finally, we install the custom ermete-*.rpm packages via rpm -Uvh --replacefiles --replacepkgs to apply our OS aesthetic and hardening.
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    rm -f /tmp/forge-repo/libav*-free-*.rpm /tmp/forge-repo/libsw*-free-*.rpm /tmp/forge-repo/libpostproc-free-*.rpm /tmp/forge-repo/ffmpeg-free-*.rpm /tmp/forge-repo/nodejs20-devel-*.rpm /tmp/forge-repo/v8-11.3-devel-*.rpm && \
    for name in $(rpm -qp --queryformat '%{NAME}\n' /tmp/forge-repo/*.rpm | sort | uniq); do \
        ls -1v /tmp/forge-repo/$name-[0-9]*.rpm 2>/dev/null | head -n -1 | xargs -r rm -f || true; \
    done && \
    if ls /tmp/forge-repo/ermete-base-config*.rpm 1> /dev/null 2>&1; then \
        echo "Step 1: Installing core Ermete Forge base configuration and repository definitions..." && \
        rpm -Uvh --replacefiles --replacepkgs --nodeps /tmp/forge-repo/ermete-base-config*.rpm && \
        rm -f /tmp/forge-repo/ermete-base-config*.rpm; \
    fi && \
    echo "Step 2: Isolating remaining custom Ermete OS packages to prevent upstream file conflicts..." && \
    mkdir -p /tmp/forge-custom && mv /tmp/forge-repo/ermete-*.rpm /tmp/forge-custom/ 2>/dev/null || true && \
    echo "Step 3: Installing all upstream packages and drivers with full repository dependency resolution..." && \
    dnf5 install -y --allowerasing --setopt=install_weak_deps=False /tmp/forge-repo/*.rpm && \
    echo "Step 4: Applying custom Ermete OS aesthetic, configurations, and overrides..." && \
    if ls /tmp/forge-custom/*.rpm 1> /dev/null 2>&1; then \
        rpm -Uvh --replacefiles --replacepkgs --nodeps /tmp/forge-custom/*.rpm; \
    fi && \
    rm -rf /tmp/forge-repo /tmp/forge-custom

### DICHIARATIVITÀ ASSOLUTA (SYSTEMD PRESETS & SYSUSERS)
# Applichiamo nativamente tutti i file .preset e i gruppi utente in modo che i target
# (es. nix-daemon, udevd con utenti e gruppi) vengano registrati nell'immagine OCI.
RUN systemd-sysusers && systemctl preset-all && systemctl --global preset-all

### BEDROCK KERNEL & INITRAMFS GENERATION (Ring 0 -> Ring 3 Integration)
# Forziamo l'inclusione dei moduli NVIDIA perché le scriptlets DNF sono disabilitate o noscripts
# Inglobiamo i db utente per consentire a udevd di risolvere i gruppi 'video' e 'render'
# Creiamo preventivamente /var/roothome per risolvere il symlink /root durante il parsing di passwd
RUN QUALIFIED_KERNEL="" && \
    for k in /lib/modules/*; do \
        if [ -e "$k/vmlinuz" ] || [ -L "$k/vmlinuz" ]; then \
            QUALIFIED_KERNEL=$(basename "$k") && break; \
        fi; \
    done && \
    if [ -z "$QUALIFIED_KERNEL" ]; then echo "ERRORE: Nessun vmlinuz trovato in /lib/modules! Build abortita." && exit 1; fi && \
    echo "Found Chimera Kernel: ${QUALIFIED_KERNEL}, generating initramfs..." && \
    depmod "${QUALIFIED_KERNEL}" && \
    mkdir -p /var/roothome && \
    /usr/bin/dracut --no-hostonly --kver "${QUALIFIED_KERNEL}" --reproducible --zstd -v \
        --add ostree --add fido2 --force-drivers "nvidia nvidia_modeset nvidia_uvm nvidia_drm" \
        --install "/etc/group /etc/passwd" \
        -f "/usr/lib/modules/${QUALIFIED_KERNEL}/initramfs.img" && \
    chmod 0600 /usr/lib/modules/"${QUALIFIED_KERNEL}"/initramfs.img && \
    ldconfig

### NIX STATE (Immutability Fix)
# Creiamo il symlink immutabile sul rootfs verso il mountpoint effimero in /var.
# Il restore del database Nix (amnesia-fix) è gestito in modo nativo e dichiarativo
# da tmpfiles.d (10-ermete-nix.conf) che copia lo stato iniziale al boot.
# Initial state for Nix is now managed by ermete-forge-nix-support RPM

### HARDENING & OSTREE LINTING FIXES
RUN rm -f /etc/machine-id && touch /etc/machine-id && chmod 0444 /etc/machine-id && \
    rm -rf /etc/NetworkManager/system-connections/* && \
    dnf clean all && \
    rm -rf /boot/* && \
    find /etc/tmpfiles.d /usr/lib/tmpfiles.d -type l -lname '/dev/null' -exec sh -c 'rm -f "$1"; touch "$1"' _ {} \; && \
    find /run /tmp /var/log -mindepth 1 -delete || true

### LINTING
## Verify final image and contents are correct.
RUN bootc container lint
