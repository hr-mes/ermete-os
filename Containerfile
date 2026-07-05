# I target delle versioni non sono più tracciati qui.
# Sono gestiti in completa autonomia dall'AI Bot di Ermete Forge.





# Builders have been entirely migrated to ermete-forge OCI micro-containers.

# --- IMMAGINE FINALE (PRODUZIONE) ---
# FIX: Renovate Bot sostituirà automaticamente il tag :latest con il vero digest SHA256 crittografico
FROM ghcr.io/patapem/ermete-base-nvidia@sha256:a6923529b7a80cc8a682c5812e3b8cc87aa618ac1c275c4e9eff4b18d6cce14c
# Estrazione pacchetti RPM puri dai Micro-Container OCI di Ermete Forge (Isolamento totale)
COPY --from=ghcr.io/patapem/ermete-forge-repo:latest / /tmp/forge-repo

# (dart-sass rimosso, se necessario andrà creato un micro-container spec dedicato)

# Nix is now fetched from ghcr.io/patapem/ermete-forge-nix-support

# FIX BEDROCK: I file per sysusers e la privacy sandbox (skel) non vengono più
# iniettati crudi, ma sono nativamente pacchettizzati negli RPM (ermete-system-config, ecc.)

# Execute all modular scripts sequentially in a single transaction to prevent OCI layer bloat
# and preserve atomicity of the RPM database.
RUN --mount=type=cache,dst=/var/cache --mount=type=cache,dst=/var/lib/dnf --mount=type=cache,dst=/var/cache/libdnf5 \
    rm -f /tmp/forge-repo/libav*-free-*.rpm /tmp/forge-repo/libsw*-free-*.rpm /tmp/forge-repo/libpostproc-free-*.rpm /tmp/forge-repo/ffmpeg-free-*.rpm /tmp/forge-repo/nodejs20-devel-*.rpm /tmp/forge-repo/v8-11.3-devel-*.rpm && \
    mkdir -p /tmp/forge-custom && mv /tmp/forge-repo/ermete-*.rpm /tmp/forge-custom/ 2>/dev/null || true && \
    for name in $(rpm -qp --queryformat '%{NAME}\n' /tmp/forge-repo/*.rpm | sort | uniq); do \
        ls -1v /tmp/forge-repo/$name-[0-9]*.rpm 2>/dev/null | head -n -1 | xargs -r rm -f || true; \
    done && \
    dnf5 install -y --allowerasing /tmp/forge-repo/*.rpm && \
    if ls /tmp/forge-custom/*.rpm 1> /dev/null 2>&1; then rpm -Uvh --replacefiles --replacepkgs --nodeps /tmp/forge-custom/*.rpm; fi && \
    rm -rf /tmp/forge-repo /tmp/forge-custom

# Fix initramfs for custom Ermete Forge kernel and remove stale Fedora kernels
RUN for kdir in /usr/lib/modules/*; do \
        if [ -f "$kdir/vmlinuz" ]; then \
            kver=$(basename $kdir); \
            echo "Found custom kernel $kver, generating initramfs..."; \
            depmod -a $kver; \
            dracut --force --no-hostonly --kver $kver $kdir/initramfs.img; \
        else \
            echo "Removing stale kernel directory $kdir..."; \
            rm -rf $kdir; \
        fi; \
    done

# (Nessun system_files iniettato. L'architettura Bedrock usa il 100% di astrazione via RPM OCI).
# Symlinks managed by system-config RPM



### NIX STATE (Immutability Fix)
# Creiamo il symlink immutabile sul rootfs verso il mountpoint effimero in /var.
# Il restore del database Nix (amnesia-fix) è gestito in modo nativo e dichiarativo
# da tmpfiles.d (10-ermete-nix.conf) che copia lo stato iniziale al boot.
# Initial state for Nix is now managed by ermete-forge-nix-support RPM

### DICHIARATIVITÀ ASSOLUTA (SYSTEMD PRESETS)
# Applichiamo nativamente tutti i file .preset (es. 99-Ermete.preset) 
# in modo che nix-daemon.socket e gli altri target vengano registrati
# all'interno dell'immagine OCI, prima dell'avvio su baremetal.
RUN systemctl preset-all && systemctl --global preset-all


RUN rm -f /etc/machine-id && touch /etc/machine-id && chmod 0444 /etc/machine-id && \
    rm -rf /etc/NetworkManager/system-connections/* && \
    dnf clean all && \
    find /etc/tmpfiles.d -type l -lname '/dev/null' -exec sh -c 'rm -f "$1"; touch "$1"' _ {} \; && \
    find /run /tmp /var/log -mindepth 1 -delete || true

### LINTING
## Verify final image and contents are correct.
# Questo step convaliderà ora correttamente l'assenza di violazioni tmpfiles.d
RUN bootc container lint
