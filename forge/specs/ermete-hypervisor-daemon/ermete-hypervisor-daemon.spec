%global debug_package %{nil}
Name:           ermete-hypervisor-daemon
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Zero-Trust Hardware Micro-Hypervisor & Confidential Enclave Orchestrator

License:        GPLv3+
URL:            https://github.com/hr-mes/ermete-os
Requires:       qemu-kvm dbus


BuildRequires:  rust >= 1.83.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

%description
Ermete OS Zero-Trust Hardware Micro-Hypervisor daemon managing lightweight AMD SEV-SNP
and Intel TDX confidential micro-VM enclaves for isolating untrusted agents and applications.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release -p ermete-hypervisor-daemon

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/ermete-hypervisor-daemon %{buildroot}%{_bindir}/ermete-hypervisor-daemon

# systemd service
mkdir -p %{buildroot}%{_unitdir}
cat > %{buildroot}%{_unitdir}/ermete-hypervisor.service <<EOF
[Unit]
Description=Ermete OS Zero-Trust Hardware Micro-Hypervisor Daemon
After=network.target dbus.service

[Service]
LockPersonality=true
RestrictSUIDSGID=true
RestrictRealtime=true
MemoryDenyWriteExecute=true
ProtectControlGroups=true
ProtectKernelLogs=true
ProtectKernelModules=true
ProtectKernelTunables=true
CPUWeight=200
CPUQuota=200%
MemoryHigh=256M
MemoryMax=512M

OOMScoreAdjust=-300
Type=simple
ExecStart=%{_bindir}/ermete-hypervisor-daemon
Restart=on-failure
RestartSec=3s
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=true
NoNewPrivileges=yes
AmbientCapabilities=CAP_SYS_ADMIN CAP_NET_ADMIN CAP_BPF
CapabilityBoundingSet=CAP_SYS_ADMIN CAP_NET_ADMIN CAP_BPF
SystemCallFilter=@system-service

[Install]
WantedBy=multi-user.target
EOF

%post
%systemd_post ermete-hypervisor.service

%preun
%systemd_preun ermete-hypervisor.service

%postun
%systemd_postun_with_restart ermete-hypervisor.service

%files
%{_bindir}/ermete-hypervisor-daemon
%{_unitdir}/ermete-hypervisor.service

%changelog
* Fri Aug 07 2026 Ermete Security Architect <security@ermete.os> - 1.0.0-1
- Initial release of zero-trust hardware Micro-Hypervisor enclave orchestrator
