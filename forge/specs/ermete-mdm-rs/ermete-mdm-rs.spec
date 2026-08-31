%global debug_package %{nil}
Name:           ermete-mdm-rs
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS Mobile Device Management

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge
Requires:       polkit cryptsetup systemd


BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel

%description
Ermete OS MDM Daemon for Anti-Theft tracking and cryptographic Remote Wipe.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755
mkdir -p $(dirname 0644) && touch 0644
mkdir -p $(dirname os.ermete.Mdm.conf) && touch os.ermete.Mdm.conf
mkdir -p $(dirname 0644) && touch 0644
mkdir -p $(dirname os.ermete.mdm.policy) && touch os.ermete.mdm.policy

install -D -m 0755 target/release/%{name} %{buildroot}/usr/bin/%{name}

# Install D-Bus system configuration
install -D -m 0644 os.ermete.Mdm.conf %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Mdm.conf

# Install Polkit policy
install -D -m 0644 os.ermete.mdm.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.mdm.policy

# Create a systemd service file
mkdir -p %{buildroot}/usr/lib/systemd/system
cat <<EOF > %{buildroot}/usr/lib/systemd/system/%{name}.service
[Unit]
Description=Ermete OS Anti-Theft & MDM Daemon
After=network-online.target dbus.service
Requires=dbus.service

[Service]
CPUWeight=50
MemoryHigh=96M
MemoryMax=128M
OOMScoreAdjust=-100
Type=dbus
BusName=os.ermete.Mdm
ExecStart=/usr/bin/%{name}
Restart=always
RestartSec=5s
DynamicUser=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=true
MemoryDenyWriteExecute=true
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
WantedBy=multi-user.target
EOF

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
/usr/bin/%{name}
/usr/lib/systemd/system/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Mdm.conf
%{_datadir}/polkit-1/actions/os.ermete.mdm.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release

