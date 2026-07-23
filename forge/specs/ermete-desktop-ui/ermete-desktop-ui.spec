%global debug_package %{nil}
Name:           ermete-desktop-ui
Version:        1.0.0
Release:        6%{?dist}
Summary:        Ermete OS Desktop UI configurations
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

Provides:       ermete-ags-config = 1.0.1-3
Obsoletes:      ermete-ags-config < 1.0.1-3
Provides:       ermete-niri-session = 1.0.0-3
Obsoletes:      ermete-niri-session < 1.0.0-3

Requires:       lxpolkit
Requires:       cliphist
Requires:       swayidle
Requires:       ddcutil

%description
Provides the unified Desktop UI (Niri) configuration for Ermete OS.
Includes dependencies for Wayland (lxpolkit, swayidle, ddcutil)
and configures UDEV for i2c access.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/etc/skel/.config/niri
mkdir -p %{buildroot}/etc/udev/rules.d
mkdir -p %{buildroot}/etc/skel/.config/systemd/user

# Copy systemd user services
cp -p %{_sourcedir}/etc/skel/.config/systemd/user/* %{buildroot}/etc/skel/.config/systemd/user/ || true

# Copy only relevant Niri files
cp -p %{_sourcedir}/etc/skel/.config/niri/config.kdl %{buildroot}/etc/skel/.config/niri/

# Copy UDEV rules
cp -p %{_sourcedir}/etc/udev/rules.d/99-ddcutil-i2c.rules %{buildroot}/etc/udev/rules.d/

%files
/etc/skel/.config/systemd/user/*
/etc/skel/.config/niri/config.kdl
/etc/udev/rules.d/99-ddcutil-i2c.rules

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-5
- Map Mod+D keyboard bind to ermete-shell-rs --dock single-instance toggle

* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 1.0.0-4
- Shift Niri keyboard shortcuts from ags toggle to native pure Rust ermete-shell-rs and ermete-settings-rs.

* Sat Jul 11 2026 Ermete Forge <forge@ermete.os> - 1.0.0-2
- Implement instant greeter termination on login success (killall -9 greeter session) and PAM CancelSession retry.

* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Unified AGS and Niri configs into ermete-desktop-ui.
- Integrated smembrated AGS app.ts into state, modals, notifications.
- Added essential Wayland deps: lxpolkit, swayidle, ddcutil.
- Added UDEV rules for ddcutil i2c.
