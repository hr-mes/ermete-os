%global debug_package %{nil}
Name:           ermete-updater-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Over-The-Air Update Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo systemd-rpm-macros

%description
Ermete OS Over-The-Air Update Daemon for Dual-Layer OTA updates (bootc and rpm-ostree).

%prep
%autosetup

%build
%set_build_flags
%cargo_build --locked

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

# Install D-Bus system configuration
install -D -m 0644 os.ermete.Updater.conf %{buildroot}%{_datadir}/dbus-1/system.d/os.ermete.Updater.conf

# Install Polkit policy
install -D -m 0644 os.ermete.updater.policy %{buildroot}%{_datadir}/polkit-1/actions/os.ermete.updater.policy

# Create a systemd service file
mkdir -p %{buildroot}%{_unitdir}
cat <<EOF > %{buildroot}%{_unitdir}/%{name}.service
[Unit]
Description=Ermete OS Updater Daemon
After=network-online.target dbus.service
Requires=dbus.service

[Service]
Type=dbus
BusName=os.ermete.Updater
ExecStart=%{_bindir}/%{name}
Restart=always

[Install]
WantedBy=multi-user.target
EOF

%files
%{_bindir}/%{name}
%{_unitdir}/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Updater.conf
%{_datadir}/polkit-1/actions/os.ermete.updater.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
