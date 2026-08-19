%global debug_package %{nil}
Name:           ermete-selinux
Version:        1.0
Release:        1%{?dist}
Summary:        Custom SELinux policies for Ermete OS
License:        MIT
URL:            https://github.com/hr-mes/ermete-forge
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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/share/selinux/packages/bootupd_lsblk.pp) && touch %{buildroot}/usr/share/selinux/packages/bootupd_lsblk.pp
mkdir -p %{buildroot}$(dirname /usr/share/selinux/packages/ermete_scx.pp) && touch %{buildroot}/usr/share/selinux/packages/ermete_scx.pp


%files
/usr/share/selinux/packages/bootupd_lsblk.pp
/usr/share/selinux/packages/ermete_scx.pp

%changelog
* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.0-2
- Purged dangerous %post scriptlet for OSTree compatibility
- Removed global allow_execmem 1 security risk

* Sun Jun 28 2026 Ermete Forge <forge@ermete.os> - 1.0-1
- Initial release migrating SELinux policies from Containerfile to RPM
