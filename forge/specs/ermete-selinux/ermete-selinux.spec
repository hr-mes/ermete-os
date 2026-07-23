%global debug_package %{nil}
Name:           ermete-selinux
Version:        1.0
Release:        1%{?dist}
Summary:        Custom SELinux policies for Ermete OS
License:        MIT
URL:            https://github.com/patapem/ermete-forge
Source0:        bootupd_lsblk.te
Source1:        ermete_scx.te

BuildArch:      noarch
BuildRequires:  checkpolicy
BuildRequires:  policycoreutils

%description
Custom SELinux Type Enforcement policies for Ermete OS.
Includes mitigations for bootupd and scx eBPF schedulers.

%prep
cp %{SOURCE0} .
cp %{SOURCE1} .

%build
checkmodule -M -m -o bootupd_lsblk.mod bootupd_lsblk.te
semodule_package -o bootupd_lsblk.pp -m bootupd_lsblk.mod

checkmodule -M -m -o ermete_scx.mod ermete_scx.te
semodule_package -o ermete_scx.pp -m ermete_scx.mod

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/share/selinux/packages
install -m 644 bootupd_lsblk.pp %{buildroot}/usr/share/selinux/packages/
install -m 644 ermete_scx.pp %{buildroot}/usr/share/selinux/packages/

%files
/usr/share/selinux/packages/bootupd_lsblk.pp
/usr/share/selinux/packages/ermete_scx.pp

%changelog
* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0-2
- Purged dangerous %post scriptlet for OSTree compatibility
- Removed global allow_execmem 1 security risk

* Sun Jun 28 2026 Ermete Forge <forge@ermete.os> - 1.0-1
- Initial release migrating SELinux policies from Containerfile to RPM
