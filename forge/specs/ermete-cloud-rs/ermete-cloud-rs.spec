%global debug_package %{nil}
Name:           ermete-cloud-rs
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS Continuity & Local P2P Sync Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge
Requires:       wl-clipboard


BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel

%description
Ermete OS Cloud Daemon for Universal Clipboard and Local P2P synchronization.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install D-Bus system configuration
install -D -m 0644 os.ermete.Cloud.conf %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Cloud.conf

# Install Polkit policy
install -D -m 0644 os.ermete.cloud.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.cloud.policy

# Create a systemd service file
mkdir -p %{buildroot}%{_unitdir}
cat <<EOF > %{buildroot}%{_unitdir}/%{name}.service
[Unit]
Description=Ermete OS Continuity Daemon
After=network-online.target dbus.service graphical.target
Requires=dbus.service

[Service]
MemoryDenyWriteExecute=true
CPUWeight=50
CPUQuota=100%
MemoryHigh=384M
MemoryMax=512M
OOMScoreAdjust=100
CapabilityBoundingSet=CAP_NET_ADMIN
AmbientCapabilities=CAP_NET_ADMIN
Type=dbus

BusName=os.ermete.Cloud
ExecStart=%{_bindir}/%{name}
Restart=always
RestartSec=5s
DynamicUser=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=true
NoNewPrivileges=yes
SystemCallFilter=@system-service
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictRealtime=true
RestrictSUIDSGID=true
LockPersonality=true

[Install]
WantedBy=graphical.target
EOF

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Cloud.conf
%{_datadir}/polkit-1/actions/os.ermete.cloud.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
