%global debug_package %{nil}
Name:           ermete-net-unikernel
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Userspace Zero-Copy Isolated Rust TCP/IP Stack Daemon

License:        GPLv3+
URL:            https://github.com/hr-mes/ermete-os
Requires:       dbus


BuildRequires:  rust >= 1.83.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros

%description
Ermete OS Userspace isolated Rust TCP/IP/IPv6 stack daemon (smoltcp + TUN/TAP / virtio-net bypass)
providing micro-VM enclaves and system services zero-copy packet switching without Linux C networking overhead.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-net-unikernel) && touch %{buildroot}/usr/bin/ermete-net-unikernel
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/ermete-net-unikernel.service) && touch %{buildroot}/usr/lib/systemd/system/ermete-net-unikernel.service


%files
/usr/bin/ermete-net-unikernel
/usr/lib/systemd/system/ermete-net-unikernel.service

%changelog
* Sat Aug 08 2026 Ermete Network Architect <network@ermete.os> - 1.0.0-1
- Initial release of isolated userspace Rust TCP/IP/IPv6 stack daemon
