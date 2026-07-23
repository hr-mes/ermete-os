%global debug_package %{nil}
Name:           xdg-desktop-portal-ermete
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Desktop Portal (Privacy & ScreenShare)

License:        GPL-3.0-or-later
URL:            https://github.com/patapem/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust cargo pkgconf-pkg-config openssl-devel

%description
Ermete OS implementation of the XDG Desktop Portal for native Wayland/Niri integration, privacy prompts, and hardware indicators.

%prep
%autosetup

%build
%set_build_flags
%cargo_build --locked

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_libexecdir}/%{name}

# Install D-Bus session service
install -D -m 0644 org.freedesktop.impl.portal.desktop.ermete.service %{buildroot}%{_datadir}/dbus-1/services/org.freedesktop.impl.portal.desktop.ermete.service

# Install Portal definition
install -D -m 0644 ermete.portal %{buildroot}%{_datadir}/xdg-desktop-portal/portals/ermete.portal

%files
%{_libexecdir}/%{name}
%{_datadir}/dbus-1/services/org.freedesktop.impl.portal.desktop.ermete.service
%{_datadir}/xdg-desktop-portal/portals/ermete.portal

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
