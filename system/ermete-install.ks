# Kickstart for Ermete OS Bare-Metal (LUKS2 + TPM2 ready)
lang en_US.UTF-8
keyboard it
timezone Europe/Rome --isUtc
bootloader --append="quiet splash fastboot iommu=pt intel_iommu=on amd_iommu=on efi=disable_early_pci_dma zswap.enabled=1 zswap.compressor=zstd rootflags=noatime slab_nomerge pti=on randomize_kstack_offset=on vsyscall=none debugfs=off oops=panic module.sig_enforce=1 lockdown=integrity init_on_free=1"

# OCI Image Provisioning
ostreecontainer --url=ghcr.io/hr-mes/ermete-os-system:latest --transport=registry

# User & Security Provisioning
rootpw --lock

firewall --enabled --default=drop --service=ssh
services --enabled=sshd,systemd-homed

reboot

%post --erroronfail
# Abilita il modulo pam_systemd_home e la risoluzione NSS via authselect
authselect enable-feature with-systemd-homed

# Avvia temporaneamente dbus e systemd-homed per consentire l'esecuzione di homectl
mkdir -p /run/dbus
dbus-daemon --system --fork --nopidfile
/usr/lib/systemd/systemd-homed &
HOMED_PID=$!

# TPM2 Monotonic Counter Initialization (NV Index 0x01800001)
if tpm2_getcap properties-fixed | grep -q "TPM2_PT_TOTAL_COMMANDS"; then
    echo "Inizializzazione NV Monotonic Counter TPM2 a 0x01800001..."
    tpm2_nvundefine 0x01800001 -C o 2>/dev/null
    tpm2_nvdefine 0x01800001 -C o -s 8 -a "ownerread|ownerwrite|authread|authwrite|nt=counter"
    tpm2_nvincrement 0x01800001 -C o
fi

# Creazione dell'utente hermes con Home cifrata LUKS2 loopback, TPM2/FIDO2 e chiave SSH
# SECRETS_PROVIDED_AT_RUNTIME: --password and --ssh-authorized-keys configured at provision time
homectl create hermes \
    --storage=luks \
    --fs-type=ext4 \
    --member-of=wheel \
    --tpm2-device=auto \
    --tpm2-pcrs=7+11 \
    --fido2-device=auto

# TPM 2.0 PCR Sealing (PCRs 0, 2, 7, 11) for LUKS partition /var/home
if command -v systemd-cryptenroll &>/dev/null; then
    echo "Esecuzione systemd-cryptenroll per sigillare la partizione LUKS /var/home a PCRs 0,2,7,11..."
    TARGET_LUKS=$(blkid -t TYPE=crypto_LUKS -o device 2>/dev/null | grep -E '/dev/vda|/dev/nvme|/dev/sda' | head -n 1 || echo "/dev/vda3")
    systemd-cryptenroll --tpm2-device=auto --tpm2-pcrs=0,2,7,11 "$TARGET_LUKS"
fi

kill $HOMED_PID || true
%end
