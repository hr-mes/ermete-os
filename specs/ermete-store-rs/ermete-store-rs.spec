%global debug_package %{nil}
Name:           ermete-store-rs
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS Universal App Store Daemon


License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel

%description
Ermete OS Universal App Store Daemon for Flatpak and OCI container management.

%prep
%autosetup

%build
%set_build_flags
cargo build --release --locked

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install D-Bus system configuration
install -D -m 0644 os.ermete.Store.conf %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Store.conf

# Install Polkit policy
install -D -m 0644 os.ermete.store.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.store.policy

# Create a systemd service file
mkdir -p %{buildroot}%{_unitdir}
cat <<EOF > %{buildroot}%{_unitdir}/%{name}.service
[Unit]
Description=Ermete OS Universal App Store Daemon
After=network-online.target dbus.service
Requires=dbus.service

[Service]
Type=dbus
BusName=os.ermete.Store
ExecStart=%{_bindir}/%{name}
Restart=always

[Install]
WantedBy=multi-user.target
EOF

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Store.conf
%{_datadir}/polkit-1/actions/os.ermete.store.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
