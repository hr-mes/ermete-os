%global debug_package %{nil}
%global __requires_exclude ^kernel-rt$
Name:           ermete-system-config
Version:        1.0.0
Release:        15%{?dist}
Summary:        Ermete OS ermete-system-config
License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

Requires:       cage greetd greenboot systemd-ukify usbguard
# Core UI and Daemons
Requires:       ermete-shell-rs ermete-settings-rs ermete-daemon-rs
Requires:       ermete-updater-rs ermete-store-rs ermete-telemetry-rs ermete-cloud-rs xdg-desktop-portal-ermete
# Provide the usbguard daemon conf without RPM file conflict
Requires:       usbguard
%description
Provides ermete-system-config for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}
cp -a %{_sourcedir}/usr %{buildroot}/ 2>/dev/null || true
cp -a %{_sourcedir}/etc %{buildroot}/ 2>/dev/null || true

mkdir -p %{buildroot}/usr/share/ermete-system-config
mv %{buildroot}/etc/usbguard/usbguard-daemon.conf %{buildroot}/usr/share/ermete-system-config/usbguard-daemon.conf

%post
mkdir -p /etc/greetd
ln -sf /usr/share/ermete-system-config/greetd.toml /etc/greetd/config.toml 2>/dev/null || true

# Overwrite usbguard-daemon.conf without claiming RPM file ownership
cp -f /usr/share/ermete-system-config/usbguard-daemon.conf /etc/usbguard/usbguard-daemon.conf 2>/dev/null || true

%files
%dir /usr/share/ermete-system-config
%attr(0755,root,root) /usr/bin/ermete-session
%attr(0755,root,root) /usr/bin/ermete-uki-enroll
%attr(0755,root,root) /usr/libexec/ermete-snapshot-trigger.sh
/usr/lib/systemd/system/ermete-timewarp.service
/usr/lib/systemd/system/ermete-timewarp.timer
/usr/lib/systemd/system-preset/99-Ermete.preset
/usr/lib/tmpfiles.d/10-ermete-greetd.conf
/usr/share/ermete-system-config/greetd.toml
/usr/share/ermete-system-config/usbguard-daemon.conf
%config(noreplace) /etc/yum.repos.d/ermete-forge.repo
%attr(0755,root,root) /etc/greenboot/check/required.d/10-greetd-running.sh
%config(noreplace) /etc/security/limits.d/99-ermete-realtime.conf

%changelog
* Fri Jul 17 2026 Ermete Forge <forge@ermete.os> - 1.0.0-15
- Fix DNF file conflicts: removed %dir ownerships for /etc/yum.repos.d and /etc/security/limits.d
- Fix usbguard-daemon.conf RPM file conflict by copying it in %post instead of packaging it in /etc

* Thu Jul 16 2026 Ermete Forge <forge@ermete.os> - 1.0.0-14
- Enforce PREEMPT_RT scheduling limits for sub-5ms latency and add kernel-rt requirement

* Thu Jul 16 2026 Ermete Forge <forge@ermete.os> - 1.0.0-12
- Add systemd-ukify dependency and ermete-uki-enroll script for UKI generation and TPM2 LUKS enrollment

* Thu Jul 16 2026 Ermete Forge <forge@ermete.os> - 1.0.0-11
- Add /etc/yum.repos.d/ermete-forge.repo for live DNF rolling release updates

* Tue Jul 14 2026 Ermete Forge <forge@ermete.os> - 1.0.0-10
- Encapsulate /usr/bin/ermete-session native script and add %post symlink for /etc/greetd/config.toml

* Tue Jul 14 2026 Ermete Forge <forge@ermete.os> - 1.0.0-9
- Add Requires: cage greetd ermete-shell-rs and remove obsolete niri-greeter.kdl and greeter-bundle.js

* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 1.0.0-8
- Remove direct /etc/greetd/config.toml to eliminate RPM transaction file conflict with greetd package (using tmpfiles L+ symlink override)

* Mon Jul 13 2026 Ermete Forge <forge@ermete.os> - 1.0.0-7
- Configure default_session command for cage Wayland kiosk executing ermete-shell-rs --greeter

* Sat Jul 11 2026 Ermete Forge <forge@ermete.os> - 1.0.0-6
- Fix %install source path expansion to copy directly from %{_sourcedir}/usr.

* Sat Jul 11 2026 Ermete Forge <forge@ermete.os> - 1.0.0-5
- Package updated greeter-bundle.js and shadow tmpfiles overrides for instant greeter transitions.

* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
