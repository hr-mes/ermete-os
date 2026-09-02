%global debug_package %{nil}
Name:           ermete-lvfs-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Firmware Automation Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel
Requires:       fwupd

%description
Ermete OS LVFS Daemon for automated background UEFI/BIOS firmware updates via fwupdmgr.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}

install -D -m 0755 target/release/%{name} %{buildroot}/usr/bin/%{name}

# Install D-Bus system configuration
SRC_OS_ERMETE_LVFS_CONF=forge/specs/ermete-lvfs-rs/ermete-lvfs-rs-1.0.0/os.ermete.Lvfs.conf
[ -f "$SRC_OS_ERMETE_LVFS_CONF" ] || SRC_OS_ERMETE_LVFS_CONF=os.ermete.Lvfs.conf
install -D -m 0644 "$SRC_OS_ERMETE_LVFS_CONF" %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Lvfs.conf

# Install Polkit policy
SRC_OS_ERMETE_LVFS_POLICY=forge/specs/ermete-lvfs-rs/ermete-lvfs-rs-1.0.0/os.ermete.lvfs.policy
[ -f "$SRC_OS_ERMETE_LVFS_POLICY" ] || SRC_OS_ERMETE_LVFS_POLICY=os.ermete.lvfs.policy
install -D -m 0644 "$SRC_OS_ERMETE_LVFS_POLICY" %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.lvfs.policy

# Create a systemd service file
mkdir -p %{buildroot}/usr/lib/systemd/system
cat <<EOF > %{buildroot}/usr/lib/systemd/system/%{name}.service
[Unit]
Description=Ermete OS Firmware Automation Daemon
After=network-online.target dbus.service fwupd.service
Requires=dbus.service fwupd.service

[Service]
MemoryDenyWriteExecute=true
CPUWeight=30
CPUQuota=50%
MemoryHigh=192M
MemoryMax=256M
OOMScoreAdjust=300
CapabilityBoundingSet=CAP_SYS_ADMIN
AmbientCapabilities=CAP_SYS_ADMIN
Type=dbus

BusName=os.ermete.Lvfs
ExecStart=/usr/bin/%{name}
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
%{_datadir}/dbus-1/system.d/os.ermete.Lvfs.conf
%{_datadir}/polkit-1/actions/os.ermete.lvfs.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release

