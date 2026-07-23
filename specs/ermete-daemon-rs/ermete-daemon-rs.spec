%global debug_package %{nil}
Name:           ermete-daemon-rs
Version:        0.2.1
Release:        1%{?dist}
Summary:        Ermete OS Native D-Bus Bedrock, ACID Settings & Multimedia Portal Daemon

License:        MIT
Source0:        ermete-daemon-rs-%{version}.tar.gz

BuildRequires:  rust cargo gcc gcc-c++ pkgconf-pkg-config
Requires:       pipewire wireplumber

%description
Pure Rust native D-Bus IPC service for Ermete OS audio, system bedrock management, ACID settings database, and XDG Desktop Portal backend (Settings, ScreenCast, RemoteDesktop).

%prep
%autosetup

%build
%set_build_flags
%cargo_build

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/ermete-daemon-rs %{buildroot}%{_bindir}/ermete-daemon-rs

mkdir -p %{buildroot}%{_datadir}/dbus-1/services
install -m 0644 org.ermete.Settings.service %{buildroot}%{_datadir}/dbus-1/services/org.ermete.Settings.service

%files
%{_bindir}/ermete-daemon-rs
%{_datadir}/dbus-1/services/org.ermete.Settings.service

%changelog
* Fri Jul 17 2026 Ermete Forge <forge@ermete.os> - 0.2.1-1
- Remove portal configuration files (migrated to dedicated xdg-desktop-portal-ermete package)

* Mon Jul 15 2026 Ermete Forge <forge@ermete.os> - 0.2.0-3
- Implemented native XDG Desktop Portal ScreenCast and RemoteDesktop backends (org.freedesktop.impl.portal.ScreenCast & RemoteDesktop)
- Added Niri output discovery via UNIX socket ($NIRI_SOCKET) and PipeWire stream negotiation

* Mon Jul 15 2026 Ermete Forge <forge@ermete.os> - 0.2.0-2
- Added ACID JSON Settings engine (org.ermete.Settings) and XDG Desktop Portal backend (org.freedesktop.impl.portal.Settings)
- Installed portal configuration files and D-Bus service activation units

* Mon Jul 15 2026 Ermete Forge <forge@ermete.os> - 0.2.0-1
- Migrated from CLI subprocess wrappers (nmcli/bluetoothctl) to native zbus 5.17.0 D-Bus proxies
- Modularized source into network.rs, bluetooth.rs, and bedrock.rs

* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial native Rust D-Bus daemon package
