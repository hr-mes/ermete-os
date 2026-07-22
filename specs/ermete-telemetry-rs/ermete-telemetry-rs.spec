%global debug_package %{nil}
Name:           ermete-telemetry-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Crash & Telemetry Reporter

License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel

%description
Ermete OS Telemetry Daemon for opt-in kernel panic and coredump reporting to GitHub.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install D-Bus system configuration
install -D -m 0644 os.ermete.Telemetry.conf %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Telemetry.conf

# Install Polkit policy
install -D -m 0644 os.ermete.telemetry.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.telemetry.policy

# Create a systemd service file
mkdir -p %{buildroot}%{_unitdir}
cat <<EOF > %{buildroot}%{_unitdir}/%{name}.service
[Unit]
Description=Ermete OS Telemetry Daemon
After=network-online.target dbus.service graphical.target
Requires=dbus.service

[Service]
Type=dbus
BusName=os.ermete.Telemetry
ExecStart=%{_bindir}/%{name}
Restart=always

[Install]
WantedBy=graphical.target
EOF

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Telemetry.conf
%{_datadir}/polkit-1/actions/os.ermete.telemetry.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
