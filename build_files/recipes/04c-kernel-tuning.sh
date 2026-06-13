#!/bin/bash
set -ouex pipefail

echo "--- Applying Advanced Kernel & Boot Optimizations ---"

# 1. Sysctl Tuning (Network & Virtual Memory)
mkdir -p /etc/sysctl.d/
cat > /etc/sysctl.d/99-ermete-performance.conf << 'EOF'
# Abilita TCP BBR per latenza minima e throughput massimo (Ottimo per power-user e streaming)
net.core.default_qdisc = fq_pie
net.ipv4.tcp_congestion_control = bbr

# Ottimizzazione Virtual Memory per ZRAM
# Vogliamo usare la ZRAM il più possibile prima di andare incontro a OOM
vm.swappiness = 150
vm.watermark_boost_factor = 0
vm.watermark_scale_factor = 125
vm.page-cluster = 0
EOF

# Hardening Sicurezza Kernel
cat > /etc/sysctl.d/99-ermete-security.conf << 'EOF'
# Nasconde i puntatori kernel ai log dmesg per utenti normali
kernel.kptr_restrict = 2
kernel.dmesg_restrict = 1
# Previene l'abuso di BPF da parte di utenti non privilegiati
kernel.unprivileged_bpf_disabled = 1
EOF

# 2. ZRAM estremo (ZSTD invece di LZO)
mkdir -p /etc/systemd/zram-generator.conf.d/
cat > /etc/systemd/zram-generator.conf.d/ermete-zram.conf << 'EOF'
[zram0]
# Alloca dinamicamente fino al 100% della RAM come ZRAM (spazio virtuale, non occupa RAM finché non serve)
zram-fraction = 1.0
# Usa ZSTD che ha un ratio di compressione 3x migliore rispetto ai default
compression-algorithm = zstd
EOF



# Limita la crescita esponenziale dei log di systemd nel lungo periodo (max 100MB)
mkdir -p /etc/systemd/journald.conf.d/
cat > /etc/systemd/journald.conf.d/00-journal-size.conf << 'EOF'
[Journal]
SystemMaxUse=100M
SystemMaxFiles=5
EOF

# Abilitazione BPF Scheduler (scx_rustland) per latenza zero
mkdir -p /etc/default/
cat > /etc/default/scx << 'EOF'
SCX_SCHEDULER=scx_rustland
SCX_FLAGS=""
EOF
echo "enable scx.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

# Abilita auto-renicing intelligente
echo "enable ananicy-cpp.service" >> /usr/lib/systemd/system-preset/99-Ermete.preset

echo "--- Kernel Optimizations Applied ---"
