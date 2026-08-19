%global debug_package %{nil}
Name:           xdg-desktop-portal-ermete
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS Desktop Portal (Privacy & ScreenShare)

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust cargo pkgconf-pkg-config openssl-devel
Requires:       ermete-shell-rs

%description
Ermete OS implementation of the XDG Desktop Portal for native Wayland/Niri integration, privacy prompts, and hardware indicators.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}


%files
%{_libexecdir}/%{name}
%{_datadir}/dbus-1/services/org.freedesktop.impl.portal.desktop.ermete.service
%{_datadir}/xdg-desktop-portal/portals/ermete.portal

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
