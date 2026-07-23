%global debug_package %{nil}
Name:           ermete-system-services
Version:        1.0.1
Release:        6%{?dist}
Summary:        Ermete OS ermete-system-services
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch
Requires:       systemd
Requires:       ermete-shell-rs

%description
Provides core systemd user targets, desktop panel lifecycle services, and skeleton synchronization for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-system-services
mkdir -p %{buildroot}/usr/lib/systemd/user
mkdir -p %{buildroot}/usr/lib/systemd/user-preset
cp -a %{_sourcedir}/usr/lib/systemd/user/* %{buildroot}/usr/lib/systemd/user/ || true
cp -a %{_sourcedir}/usr/lib/systemd/user-preset/* %{buildroot}/usr/lib/systemd/user-preset/ || true
ln -s ermete-shell.service %{buildroot}/usr/lib/systemd/user/ermete-ags.service

%files
%dir /usr/share/ermete-system-services
/usr/lib/systemd/user/niri-session.target
/usr/lib/systemd/user/ermete-skel-sync.service
/usr/lib/systemd/user/ermete-shell.service
/usr/lib/systemd/user/ermete-dock.service
/usr/lib/systemd/user/ermete-ags.service
/usr/lib/systemd/user-preset/99-ermete-desktop.preset

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.1-6
- Add ermete-dock.service as dedicated user systemd service for interactive Glassmorphic Dock

* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.1-5
- Rename ermete-ags.service to ermete-shell.service with backward compatibility alias symlink

* Sat Jul 11 2026 Ermete Forge <forge@ermete.os> - 1.0.1-4
- Switch ermete-ags.service to run pure Rust ermete-shell-rs native binary instead of GJS/JS

* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0.1-3
- Refactored ermete-skel-sync to copy all missing dotfiles (Niri, Matugen, etc) securely without overwriting

* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0.1-2
- Fix Wayland socket race condition by changing After to graphical-session.target for ags and wallpaper

* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0.1-1
- Implement systemd user target niri-session.target
- Implement Astal AGS desktop panel lifecycle service
- Implement skeleton sync for seamless user upgrade migration
