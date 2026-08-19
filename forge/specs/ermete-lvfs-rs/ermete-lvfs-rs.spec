%global debug_package %{nil}
Name:           ermete-lvfs-rs
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Firmware Automation Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust cargo systemd-rpm-macros pkgconf-pkg-config openssl-devel
Requires:       fwupd

%description
Ermete OS LVFS Daemon for automated background UEFI/BIOS firmware updates via fwupdmgr.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/%{name}) && touch %{buildroot}/usr/bin/%{name}
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/%{name}.service) && touch %{buildroot}/usr/lib/systemd/system/%{name}.service


%files
/usr/bin/%{name}
/usr/lib/systemd/system/%{name}.service
%{_datadir}/dbus-1/system.d/os.ermete.Lvfs.conf
%{_datadir}/polkit-1/actions/os.ermete.lvfs.policy

%changelog
* Thu Jul 16 2026 Ermete <ermete@ermete.os> - 1.0.0-1
- Initial release
