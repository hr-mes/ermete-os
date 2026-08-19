%global debug_package %{nil}
Name:           ermete-net-unikernel
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Userspace Zero-Copy Isolated Rust TCP/IP Stack Daemon

License:        GPLv3+
URL:            https://github.com/hr-mes/ermete-os
Requires:       dbus
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.83.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

%description
Ermete OS Userspace isolated Rust TCP/IP/IPv6 stack daemon (smoltcp + TUN/TAP / virtio-net bypass)
providing micro-VM enclaves and system services zero-copy packet switching without Linux C networking overhead.

%prep
%setup -q

%build
%set_build_flags
cargo generate-lockfile
cargo build --release -p ermete-net-unikernel

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/ermete-net-unikernel %{buildroot}%{_bindir}/ermete-net-unikernel

# systemd service
mkdir -p %{buildroot}%{_unitdir}
cat > %{buildroot}%{_unitdir}/ermete-net-unikernel.service <<EOF
[Unit]
Description=Ermete OS Isolated Rust Userspace TCP/IP Network Unikernel Daemon
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
MemoryHigh=256M
MemoryMax=512M
OOMScoreAdjust=-300
Type=simple
ExecStart=%{_bindir}/ermete-net-unikernel
Restart=on-failure
RestartSec=3s
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=true
NoNewPrivileges=yes
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_RAW CAP_BPF
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_RAW CAP_BPF

[Install]
WantedBy=multi-user.target
EOF

%post
%systemd_post ermete-net-unikernel.service

%preun
%systemd_preun ermete-net-unikernel.service

%postun
%systemd_postun_with_restart ermete-net-unikernel.service

%files
%{_bindir}/ermete-net-unikernel
%{_unitdir}/ermete-net-unikernel.service

%changelog
* Sat Aug 08 2026 Ermete Network Architect <network@ermete.os> - 1.0.0-1
- Initial release of isolated userspace Rust TCP/IP/IPv6 stack daemon
