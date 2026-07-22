# Kickstart for Ermete OS Bare-Metal (LUKS2 + TPM2 ready)
lang en_US.UTF-8
keyboard it
timezone Europe/Rome --isUtc
bootloader --append="quiet splash fastboot"

# Zero-Touch Partitioning (BTRFS + LUKS2)
# [ATTENZIONE]: Direttive distruttive come "zerombr" e "clearpart --all" sono state rimosse
# per preservare dati esistenti e configurazioni dual-boot avanzate.
# L'installazione si fermerà sulla UI di partizionamento (o in via testuale) chiedendoti
# di definire manualmente la Root BTRFS e il volume LUKS2 come richiesto dai Power User.

# OCI Image Provisioning
ostreecontainer --url=ghcr.io/patapem/ermete-os:latest --transport=registry

# Security Hardening: No cleartext root password, strictly SSH Keys
# WARNING: Replace ALL placeholder values below before production deployment
rootpw --lock
user --name=hermes --groups=wheel --password=$6$REPLACE_WITH_SECURE_HASH --iscrypted
# TODO: Replace with actual SSH public keys
sshkey --username hermes "ssh-ed25519 REPLACE_WITH_ACTUAL_SSH_PUBLIC_KEY"
sshkey --username root "ssh-ed25519 REPLACE_WITH_ACTUAL_SSH_PUBLIC_KEY"

firewall --enabled --default=drop
services --enabled=sshd

reboot
