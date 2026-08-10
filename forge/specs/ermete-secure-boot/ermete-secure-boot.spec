Name:           ermete-secure-boot
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Measured Secure Boot & TPM Sealing

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  systemd-rpm-macros
Requires:       systemd-ukify systemd-boot-unsigned sbsigntools tpm2-tools cryptsetup

%description
Ermete OS cryptographic scripts for Unified Kernel Image (UKI) generation using systemd-stub and ukify,
UEFI Secure Boot signing, and TPM 2.0 PCR 0, 2, 7, 11 measurement/sealing.

%prep
# Nothing to prep, just source files

%build
# Nothing to build

%install
mkdir -p %{buildroot}%{_libexecdir}/ermete
install -m 0755 %{_sourcedir}/usr/libexec/ermete-secure-boot-measure.sh %{buildroot}%{_libexecdir}/ermete-secure-boot-measure.sh
install -m 0755 %{_sourcedir}/usr/libexec/ermete-tpm-luks-seal.sh %{buildroot}%{_libexecdir}/ermete-tpm-luks-seal.sh
install -m 0755 %{_sourcedir}/usr/libexec/ermete/ermete-tpm-rollback-check.sh %{buildroot}%{_libexecdir}/ermete/ermete-tpm-rollback-check.sh
install -m 0755 %{_sourcedir}/usr/libexec/ermete/ermete-tpm-rollback-update.sh %{buildroot}%{_libexecdir}/ermete/ermete-tpm-rollback-update.sh

# Install systemd services
mkdir -p %{buildroot}%{_unitdir}
install -m 0644 %{_sourcedir}/usr/lib/systemd/system/ermete-tpm-rollback-check.service %{buildroot}%{_unitdir}/ermete-tpm-rollback-check.service
install -m 0644 %{_sourcedir}/usr/lib/systemd/system/ermete-tpm-rollback-update.service %{buildroot}%{_unitdir}/ermete-tpm-rollback-update.service
install -m 0644 %{_sourcedir}/usr/lib/systemd/system/ermete-tpm-luks-seal.service %{buildroot}%{_unitdir}/ermete-tpm-luks-seal.service

mkdir -p %{buildroot}%{_unitdir}/systemd-pcrphase-sysinit.service.d
install -m 0644 %{_sourcedir}/usr/lib/systemd/system/systemd-pcrphase-sysinit.service.d/10-rollback-check.conf %{buildroot}%{_unitdir}/systemd-pcrphase-sysinit.service.d/10-rollback-check.conf

cat <<EOF > %{buildroot}%{_unitdir}/ermete-secure-boot.service
[Unit]
Description=Ermete OS Measured Boot & UKI Signer
ConditionPathExists=/etc/keys/ermete-secure-boot.key

[Service]
MemoryDenyWriteExecute=true
CPUWeight=50
MemoryHigh=384M
MemoryMax=512M
Type=oneshot
ExecStart=%{_libexecdir}/ermete-secure-boot-measure.sh
RemainAfterExit=yes
NoNewPrivileges=yes
PrivateTmp=true
ProtectSystem=strict
ProtectHome=yes
RestrictAddressFamilies=AF_UNIX
SystemCallFilter=@system-service
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictSUIDSGID=true
LockPersonality=true
ReadWritePaths=/etc/keys /boot/efi /etc/systemd

[Install]
WantedBy=multi-user.target
EOF

%files
%{_libexecdir}/ermete-secure-boot-measure.sh
%{_libexecdir}/ermete-tpm-luks-seal.sh
%{_libexecdir}/ermete/ermete-tpm-rollback-check.sh
%{_libexecdir}/ermete/ermete-tpm-rollback-update.sh
%{_unitdir}/ermete-secure-boot.service
%{_unitdir}/ermete-tpm-luks-seal.service
%{_unitdir}/ermete-tpm-rollback-check.service
%{_unitdir}/ermete-tpm-rollback-update.service
%{_unitdir}/systemd-pcrphase-sysinit.service.d/10-rollback-check.conf

%changelog
* Fri Aug 07 2026 Ermete <ermete@ermete.os> - 1.0.0-2
- Add UKI assembly via systemd-stub and ukify, and TPM 2.0 PCR 0,2,7,11 LUKS sealing service
