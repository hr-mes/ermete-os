%global debug_package %{nil}
Name:           ermete-cloud-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Continuity & Local P2P Sync Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel

%description
Ermete OS Cloud Daemon for Universal Clipboard and Local P2P synchronization.

%prep
%autosetup

%build
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
Type=dbus
BusName=os.ermete.Cloud
ExecStart=%{_bindir}/%{name}
Restart=always
RestartSec=5s
DynamicUser=yes
ProtectSystem=strict
ProtectHome=read-only
NoNewPrivileges=true

[Install]
WantedBy=graphical.target
EOF

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Cloud.conf
%{_datadir}/polkit-1/actions/os.ermete.cloud.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
