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
RUN dnf5 -y remove --no-autoremove kernel kernel-core kernel-modules kernel-modules-core kernel-modules-extra kernel-tools kernel-tools-libs zram-generator-defaults

# SECURITY: ermete-install.ks contains placeholder SSH keys that MUST be replaced before production

# Estrazione pacchetti RPM dai Micro-Container OCI di Ermete Forge (Isolamento Totale per Tier)
# I pacchetti vengono copiati chirurgicamente nei singoli Tier prima dell'installazione per evitare il cache-busting OCI.

# (dart-sass rimosso, se necessario andrà creato un micro-container spec dedicato)
# Nix is now fetched from ghcr.io/patapem/ermete-forge-nix-support
# FIX BEDROCK: I file per sysusers e la privacy sandbox (skel) non vengono più
# iniettati crudi, ma sono nativamente pacchettizzati negli RPM (ermete-system-config, ecc.)

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
# FIX BEDROCK (Universal Package Resolution & Zero-Conflict Ordering):
# 1) We install ONLY ermete-base-config first via rpm -Uvh so that repository definitions
#    (/etc/yum.repos.d/rpmfusion.repo, gpg keys, etc.) are injected into the rootfs.
# 2) We run dnf5 install to download and resolve all upstream packages and external dependencies.

# TIER 0: BEDROCK HARDWARE & KERNEL FOUNDATION (~3.3 GB - Static Cache - Reboot Required)
RUN --mount=type=bind,from=ghcr.io/patapem/ermete-forge-tier0-repo:latest,source=/,target=/mnt/tier0-repo \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    mkdir -p /tmp/tier0-repo && \
    cp -r /mnt/tier0-repo/*.rpm /tmp/tier0-repo/ && \
    rm -f /tmp/tier0-repo/libav*-free-*.rpm /tmp/tier0-repo/libsw*-free-*.rpm /tmp/tier0-repo/libpostproc-free-*.rpm /tmp/tier0-repo/ffmpeg-free-*.rpm /tmp/tier0-repo/nodejs20-devel-*.rpm /tmp/tier0-repo/v8-11.3-devel-*.rpm /tmp/tier0-repo/systemd-standalone-sysusers-*.rpm /tmp/tier0-repo/glibc32-*.rpm && \
    for name in $(rpm -qp --queryformat '%{NAME}\n' /tmp/tier0-repo/*.rpm | sort | uniq); do \
        ls -1v /tmp/tier0-repo/$name-[0-9]*.rpm 2>/dev/null | head -n -1 | xargs -r rm -f || true; \
    done && \
    if ls /tmp/tier0-repo/ermete-base-config*.rpm 1> /dev/null 2>&1; then \
        echo "Tier 0: Installing core Ermete Forge base configuration and repository definitions..." && \
        rpm -Uvh --replacefiles --replacepkgs --nodeps /tmp/tier0-repo/ermete-base-config*.rpm && \
        rm -f /tmp/tier0-repo/ermete-base-config*.rpm; \
    fi && \
    echo "Tier 0: Installing Bedrock hardware, kernel Chimera & NVIDIA dependencies..." && \
    dnf5 install -y --allowerasing --setopt=install_weak_deps=False --setopt=tsflags=nodocs /tmp/tier0-repo/*.rpm && \
    rm -rf /usr/lib/firmware/mellanox /usr/lib/firmware/qlogic /usr/lib/firmware/netronome /usr/lib/firmware/liquidio /usr/lib/firmware/cxgb4 /usr/lib/firmware/bnx2x /usr/lib/firmware/cavium /usr/lib/firmware/dpaa2 || true && \
    rm -rf /usr/share/doc/* /usr/share/man/* /usr/share/info/* /usr/share/gtk-doc/* /usr/share/help/* || true && \
    rm -rf /tmp/tier0-repo

### BEDROCK KERNEL & INITRAMFS GENERATION (Moved up for cache optimization)
# L'initramfs dinamico viene rigenerato per sincronizzare i moduli kernel e NVIDIA
# ed evitare API mismatch con l'ecosistema userspace.
RUN QUALIFIED_KERNEL="" && \
    for k in /lib/modules/*; do \
        if [ -e "$k/vmlinuz" ] || [ -L "$k/vmlinuz" ]; then \
            QUALIFIED_KERNEL=$(basename "$k") && break; \
        fi; \
    done && \
    if [ -z "$QUALIFIED_KERNEL" ]; then echo "ERRORE: Nessun vmlinuz trovato in /lib/modules! Build abortita." && exit 1; fi && \
    echo "Found Chimera Kernel: ${QUALIFIED_KERNEL}" && \
    depmod "${QUALIFIED_KERNEL}" && \
    echo "Generazione Dinamica Initramfs per ${QUALIFIED_KERNEL}..." && \
    dracut --no-hostonly --kver "${QUALIFIED_KERNEL}" --reproducible --zstd -v \
        --add ostree --add fido2 --force-drivers "nvidia nvidia_modeset nvidia_uvm nvidia_drm" \
        --install "/etc/group /etc/passwd" \
        -f /usr/lib/modules/${QUALIFIED_KERNEL}/initramfs.img && \
    chmod 0600 /usr/lib/modules/${QUALIFIED_KERNEL}/initramfs.img && \
    ldconfig

# TIER 1: DISPLAY SERVER & CORE USERSPACE SERVICES (~34 MB)
RUN --mount=type=bind,from=ghcr.io/patapem/ermete-forge-tier1-repo:latest,source=/,target=/mnt/tier1-repo \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    if ls /mnt/tier1-repo/*.rpm 1> /dev/null 2>&1; then \
        echo "Tier 1: Installing Display Server & Core Userspace Services..." && \
        dnf5 install -y --setopt=install_weak_deps=False --setopt=tsflags=nodocs /mnt/tier1-repo/*.rpm; \
    fi

# TIER 2: DESIGN SYSTEM & STATIC ASSETS (~18 MB)
RUN --mount=type=bind,from=ghcr.io/patapem/ermete-forge-tier2-repo:latest,source=/,target=/mnt/tier2-repo \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    if ls /mnt/tier2-repo/*.rpm 1> /dev/null 2>&1; then \
        echo "Tier 2: Installing Design System & Static Assets..." && \
        dnf5 install -y --setopt=install_weak_deps=False --setopt=tsflags=nodocs /mnt/tier2-repo/*.rpm; \
    fi

# TIER 3: AGILE RUST SHELL & APPS (~8 MB - Instant Live Swap Layer)
RUN --mount=type=bind,from=ghcr.io/patapem/ermete-forge-tier3-repo:latest,source=/,target=/mnt/tier3-repo \
    --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    if ls /mnt/tier3-repo/*.rpm 1> /dev/null 2>&1; then \
        echo "Tier 3: Installing Agile Rust Shell & Apps..." && \
        dnf5 install -y --setopt=install_weak_deps=False --setopt=tsflags=nodocs /mnt/tier3-repo/*.rpm; \
    fi

### BEDROCK SELINUX (Declarative Compilation)
# We compile the policies here in the Containerfile so they are atomic with the OCI image.
# We DO NOT use allow_execmem 1 to preserve enterprise security (W^X).
RUN semodule -i /usr/share/selinux/packages/bootupd_lsblk.pp && \
    semodule -i /usr/share/selinux/packages/ermete_scx.pp && \
    setsebool -P daemons_enable_cluster_mode 1 && \
    setsebool -P xserver_execmem 1

### DICHIARATIVITÀ ASSOLUTA (SYSTEMD PRESETS & SYSUSERS)
# Applichiamo nativamente tutti i file .preset e i gruppi utente in modo che i target
# (es. nix-daemon, udevd con utenti e gruppi) vengano registrati nell'immagine OCI.
RUN systemctl enable systemd-sysext.service && systemd-sysusers && systemctl preset-all && systemctl --global preset-all



### NIX STATE (Immutability Fix)
# Creiamo il symlink immutabile sul rootfs verso il mountpoint effimero in /var.
# Il restore del database Nix (amnesia-fix) è gestito in modo nativo e dichiarativo
# da tmpfiles.d (10-ermete-nix.conf) che copia lo stato iniziale al boot.
# Initial state for Nix is now managed by ermete-forge-nix-support RPM

### HARDENING & OSTREE LINTING FIXES
RUN authselect select sssd with-silent-lastlog without-nullok --force || authselect select local with-silent-lastlog without-nullok --force || true && \
    dnf5 remove -y --no-autoremove \
        gcc make kernel-devel llvm-static rust-std-static mesa-dxil-devel \
        edk2-aarch64 qemu-user qemu-user-static distribution-gpg-keys-copr \
        nodejs-docs glibc-all-langpacks python3-botocore || true && \
    rm -f /etc/machine-id && touch /etc/machine-id && \
    rm -rf /etc/NetworkManager/system-connections/* && \
    dnf5 clean all && \
    rm -rf /var/cache/dnf/* /var/lib/dnf/* /var/cache/libdnf5/* && \
    find /boot -mindepth 1 -delete || true && \
    find /run /tmp /var/log -mindepth 1 -delete || true && \
    find /var -type l -lname '/*' -delete || true

### LINTING
## Verify final image and contents are correct.
RUN bootc container lint --skip var-tmpfiles
