%global debug_package %{nil}
Name:           ermete-base-config
Version:        1.0.0
Release:        3%{?dist}
Summary:        Ermete OS Base Configuration (NVIDIA, Systemd, Branding, GPG)

License:        MIT
URL:            https://github.com/patapem/ermete-forge
BuildArch:      noarch

Requires:       glibc-langpack-it glibc-langpack-en

%description
This package provides the foundational configuration for Ermete Base.
It includes NVIDIA sleep scripts, Dracut configurations, modprobe rules,
Systemd presets, custom Plymouth/GDM branding, Polkit rules, and GPG keys.

%prep
# No extraction needed, files are injected in install phase.

%build
# Nothing to build

%install
mkdir -p %{buildroot}
find %{_sourcedir} -mindepth 1 -maxdepth 1 ! -name "*.spec" -exec cp -a {} %{buildroot}/ \;

%files
/etc/pki/rpm-gpg/*
/etc/selinux/config
/etc/yum.repos.d/*
/etc/systemd/system/*
/etc/tmpfiles.d/*
/usr/bin/nvidia-sleep.sh
/usr/lib/fedora-release
/usr/lib/os-release
/usr/lib/bootc/kargs.d/01-nvidia.toml
/usr/lib/dracut/dracut.conf.d/*
/usr/lib/modprobe.d/*
/usr/lib/modules-load.d/*
/usr/lib/systemd/system-preset/*
/usr/lib/systemd/system-sleep/nvidia
/usr/lib/systemd/system/nvidia-*
/usr/lib/systemd/system/scx_loader.service.d/*
/usr/lib/sysusers.d/*
/usr/lib/udev/rules.d/*
/usr/share/pixmaps/*
/usr/share/plymouth/themes/spinner/watermark.png
/usr/share/polkit-1/rules.d/*
/usr/lib/systemd/system/bootc-fetch-apply-updates.service.d/override.conf

%changelog
* Tue Jul 14 2026 Ermete Forge <forge@ermete.os> - 1.0.0-3
- Require glibc-langpack-it and glibc-langpack-en to guarantee Bedrock locale availability across all apps when glibc-all-langpacks is pruned

* Mon Jul 06 2026 Ermete Forge <forge@ermete.os> - 1.0.0-2
- Add enable nvidia-persistenced.service to systemd preset for deterministic GPU node creation

* Wed Jul 01 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial encapsulation of raw files into RPM for Bedrock logic
