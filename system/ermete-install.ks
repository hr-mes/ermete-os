# Kickstart for Ermete OS Bare-Metal (LUKS2 + TPM2 ready)
lang en_US.UTF-8
keyboard it
timezone Europe/Rome --isUtc
bootloader --append="quiet splash fastboot"

# OCI Image Provisioning
ostreecontainer --url=ghcr.io/hr-mes/ermete-os-system:latest --transport=registry

# User & Security Provisioning
rootpw --lock
user --name=hermes --groups=wheel --password=@HERMES_PASSWORD_HASH@ --iscrypted
sshkey --username hermes "@HERMES_SSH_KEY@"
sshkey --username root "@ROOT_SSH_KEY@"

firewall --enabled --default=drop --service=ssh
services --enabled=sshd

reboot
