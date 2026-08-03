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
sleep 2

# TPM2 Monotonic Counter Initialization (NV Index 0x01800001)
if tpm2_getcap properties-fixed | grep -q "TPM2_PT_TOTAL_COMMANDS"; then
    echo "Inizializzazione NV Monotonic Counter TPM2 a 0x01800001..."
    tpm2_nvundefine 0x01800001 -C o 2>/dev/null || true
    tpm2_nvdefine 0x01800001 -C o -s 8 -a "ownerread|ownerwrite|authread|authwrite|nt=counter"
    tpm2_nvincrement 0x01800001 -C o
fi

# Creazione dell'utente hermes con Home cifrata LUKS2 loopback, TPM2/FIDO2 e chiave SSH
homectl create hermes \
    --storage=luks \
    --fs-type=ext4 \
    --member-of=wheel \
    --password="@HERMES_PASSWORD@" \
    --tpm2-device=auto \
    --tpm2-pcrs=7+11 \
    --fido2-device=auto \
    --ssh-authorized-keys="@HERMES_SSH_KEY@"

kill $HOMED_PID || true
%end
