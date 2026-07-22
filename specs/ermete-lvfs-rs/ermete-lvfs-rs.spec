%global debug_package %{nil}
Name:           ermete-lvfs-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Firmware Automation Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel
Requires:       fwupd

%description
Ermete OS LVFS Daemon for automated background UEFI/BIOS firmware updates via fwupdmgr.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install D-Bus system configuration
install -D -m 0644 os.ermete.Lvfs.conf %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Lvfs.conf

# Install Polkit policy
install -D -m 0644 os.ermete.lvfs.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.lvfs.policy

# Create a systemd service file
mkdir -p %{buildroot}%{_unitdir}
cat <<EOF > %{buildroot}%{_unitdir}/%{name}.service
[Unit]
Description=Ermete OS Firmware Automation Daemon
After=network-online.target dbus.service fwupd.service
Requires=dbus.service fwupd.service

[Service]
Type=dbus
BusName=os.ermete.Lvfs
ExecStart=%{_bindir}/%{name}
Restart=always
RestartSec=5s
DynamicUser=yes
ProtectSystem=strict
ProtectHome=read-only
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
EOF

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Lvfs.conf
%{_datadir}/polkit-1/actions/os.ermete.lvfs.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
