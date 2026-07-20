Name:           ermete-secure-boot
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Measured Secure Boot & TPM Sealing

License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  systemd-rpm-macros
Requires:       systemd-ukify sbsigntools tpm2-tools

%description
Ermete OS cryptographic scripts for Unified Kernel Image (UKI) generation,
UEFI Secure Boot signing, and TPM 2.0 PCR 11 measurement/sealing.

%prep
# Nothing to prep, just source files

%build
# Nothing to build

%install
mkdir -p %{buildroot}%{_libexecdir}
install -m 0755 %{_sourcedir}/usr/libexec/ermete-secure-boot-measure.sh %{buildroot}%{_libexecdir}/ermete-secure-boot-measure.sh

# Install a systemd service that triggers on kernel install
mkdir -p %{buildroot}%{_unitdir}
cat <<EOF > %{buildroot}%{_unitdir}/ermete-secure-boot.service
[Unit]
Description=Ermete OS Measured Boot & UKI Signer
ConditionPathExists=/etc/keys/ermete-secure-boot.key

[Service]
Type=oneshot
ExecStart=%{_libexecdir}/ermete-secure-boot-measure.sh
RemainAfterExit=yes
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
RestrictAddressFamilies=AF_UNIX
ReadWritePaths=/etc/keys

[Install]
WantedBy=multi-user.target
EOF

%files
%{_libexecdir}/ermete-secure-boot-measure.sh
%{_unitdir}/ermete-secure-boot.service

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
