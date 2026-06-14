# Kickstart for Ermete OS Bare-Metal (LUKS2 + TPM2 ready)
lang en_US.UTF-8
keyboard it
timezone Europe/Rome --isUtc

# Zero-Touch Partitioning (BTRFS + LUKS2)
# [ATTENZIONE]: Direttive distruttive come "zerombr" e "clearpart --all" sono state rimosse
# per preservare dati esistenti e configurazioni dual-boot avanzate.
# L'installazione si fermerà sulla UI di partizionamento (o in via testuale) chiedendoti
# di definire manualmente la Root BTRFS e il volume LUKS2 come richiesto dai Power User.

# OCI Image Provisioning
ostreecontainer --url=ghcr.io/patapem/ermete-os:latest --transport=registry

# Security Hardening: No cleartext root password, strictly SSH Keys
rootpw --lock
user --name=wheel --groups=wheel --password=$6$dummyhash --iscrypted
sshkey --username wheel "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI_DUMMY_KEY_HERE_"
sshkey --username root "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI_DUMMY_KEY_HERE_"

firewall --enabled --default=drop
services --enabled=sshd

reboot
