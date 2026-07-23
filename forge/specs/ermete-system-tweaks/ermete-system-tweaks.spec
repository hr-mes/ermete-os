%global debug_package %{nil}
Name:           ermete-system-tweaks
Version:        1.0.0
Release:        3%{?dist}
Summary:        Ermete OS ermete-system-tweaks
License:        MIT
URL:            https://github.com/patapem/ermete-forge
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
mkdir -p %{buildroot}/etc/polkit-1/rules.d
mkdir -p %{buildroot}/usr/lib/sysctl.d

cp -a %{_sourcedir}/usr/lib/environment.d/10-ermete-wayland.conf %{buildroot}/usr/lib/environment.d/
cp -a %{_sourcedir}/usr/share/pipewire/pipewire.conf.d/10-low-latency.conf %{buildroot}/usr/share/pipewire/pipewire.conf.d/
cp -a %{_sourcedir}/etc/polkit-1/rules.d/10-ermete-wheel-admin.rules %{buildroot}/etc/polkit-1/rules.d/
cp -a %{_sourcedir}/usr/lib/sysctl.d/99-bore.conf %{buildroot}/usr/lib/sysctl.d/

%post

%files
%dir /usr/share/ermete-system-tweaks
/usr/lib/environment.d/10-ermete-wayland.conf
/usr/share/pipewire/pipewire.conf.d/10-low-latency.conf
/etc/polkit-1/rules.d/10-ermete-wheel-admin.rules
/usr/lib/sysctl.d/99-bore.conf

%changelog
* Fri Jul 10 2026 Ermete Forge <forge@ermete.os> - 1.0.0-3
- Added native sysctl tuning for BORE scheduler (99-bore.conf)

* Wed Jul 08 2026 Ermete Forge <forge@ermete.os> - 1.0.0-2
- Add Wayland environment variables and PipeWire low latency config
* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial Bedrock encapsulation
