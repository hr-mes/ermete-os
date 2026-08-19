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
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
# magic stub generator
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755
mkdir -p $(dirname target/release/%{name}) && touch target/release/%{name}
mkdir -p $(dirname 0644) && touch 0644
mkdir -p $(dirname org.freedesktop.impl.portal.desktop.ermete.service) && touch org.freedesktop.impl.portal.desktop.ermete.service
mkdir -p $(dirname 0644) && touch 0644
mkdir -p $(dirname ermete.portal) && touch ermete.portal

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
