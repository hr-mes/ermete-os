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
# No prep needed for local workspace build, sources are mounted directly

%build
%set_build_flags
cd /forge/system/ermete-net-unikernel
cargo build --release --offline

%install
mkdir -p %{buildroot}/usr/bin
install -m 755 /forge/system/ermete-net-unikernel/target/release/ermete-net-unikernel %{buildroot}/usr/bin/ermete-net-unikernel

%post
%systemd_post ermete-net-unikernel.service

%preun
%systemd_preun ermete-net-unikernel.service

%postun
%systemd_postun_with_restart ermete-net-unikernel.service

%files
/usr/bin/ermete-net-unikernel
/usr/lib/systemd/system/ermete-net-unikernel.service

%changelog
* Sat Aug 08 2026 Ermete Network Architect <network@ermete.os> - 1.0.0-1
- Initial release of isolated userspace Rust TCP/IP/IPv6 stack daemon

