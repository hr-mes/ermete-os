%global debug_package %{nil}
Name:           ermete-system-tweaks
Version:        1.0.0
Release:        4%{?dist}
Summary:        Ermete OS ermete-system-tweaks
License:        MIT
URL:            https://github.com/hr-mes/ermete-forge
BuildArch:      noarch

%description
Provides ermete-system-tweaks for Ermete OS.

%prep
# Nothing to prep

%build
# Nothing to build

%install
mkdir -p %{buildroot}/usr/share/ermete-system-tweaks
mkdir -p %{buildroot}/usr/lib/environment.d
mkdir -p %{buildroot}/usr/share/pipewire/pipewire.conf.d
mkdir -p %{buildroot}/etc/NetworkManager/conf.d
mkdir -p %{buildroot}/etc/systemd/resolved.conf.d
mkdir -p %{buildroot}/usr/share/polkit-1/rules.d
mkdir -p %{buildroot}/usr/lib/sysctl.d
mkdir -p %{buildroot}/usr/lib/tmpfiles.d
cp -a %{_sourcedir}/usr/lib/environment.d/10-ermete-wayland.conf %{buildroot}/usr/lib/environment.d/
cp -a %{_sourcedir}/usr/share/pipewire/pipewire.conf.d/10-low-latency.conf %{buildroot}/usr/share/pipewire/pipewire.conf.d/
cp -a %{_sourcedir}/etc/polkit-1/rules.d/10-ermete-wheel-admin.rules %{buildroot}/usr/share/polkit-1/rules.d/
cp -a %{_sourcedir}/usr/lib/sysctl.d/99-bore.conf %{buildroot}/usr/lib/sysctl.d/
cp -a %{_sourcedir}/usr/lib/sysctl.d/99-network-security.conf %{buildroot}/usr/lib/sysctl.d/
cp -a %{_sourcedir}/usr/lib/tmpfiles.d/99-ermete-kernel-sysfs.conf %{buildroot}/usr/lib/tmpfiles.d/
cp -a %{_sourcedir}/etc/NetworkManager/conf.d/99-mac-randomization.conf %{buildroot}/etc/NetworkManager/conf.d/
cp -a %{_sourcedir}/etc/systemd/resolved.conf.d/99-dns-tls.conf %{buildroot}/etc/systemd/resolved.conf.d/

%post

%files
%dir /usr/share/ermete-system-tweaks
%config(noreplace) /etc/NetworkManager/conf.d/99-mac-randomization.conf
%config(noreplace) /etc/systemd/resolved.conf.d/99-dns-tls.conf
/usr/lib/environment.d/10-ermete-wayland.conf
/usr/share/pipewire/pipewire.conf.d/10-low-latency.conf
/usr/share/polkit-1/rules.d/10-ermete-wheel-admin.rules
/usr/lib/sysctl.d/99-bore.conf
/usr/lib/sysctl.d/99-network-security.conf
/usr/lib/tmpfiles.d/99-ermete-kernel-sysfs.conf
%changelog
* Fri Jul 10 2026 Ermete Forge <forge@ermete.os> - 1.0.0-3
- Added native sysctl tuning for BORE scheduler (99-bore.conf)

* Wed Jul 08 2026 Ermete Forge <forge@ermete.os> - 1.0.0-2
- Add Wayland environment variables and PipeWire low latency config
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
