%global debug_package %{nil}
Name:           ermete-recovery
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Pre-Boot GUI Recovery Kiosk & Rollback Manager

License:        MIT
Source0:        ermete-recovery-%{version}.tar.gz

BuildRequires:  rust cargo gcc gcc-c++ gtk4-devel glib2-devel pkgconf-pkg-config
Requires:       gtk4 glib2 cage rpm-ostree systemd

%description
Pre-Boot GUI Wayland Kiosk recovery environment for Ermete OS (`ermete-recovery-ui`).
Provides 1-click OSTree/bootc visual rollback and automatic failover when `greetd` or the graphical session crashes.

%prep
%autosetup

%build
%set_build_flags
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/ermete-recovery-ui %{buildroot}%{_bindir}/ermete-recovery-ui

mkdir -p %{buildroot}/usr/lib/systemd/system
install -m 0644 systemd/ermete-recovery.service %{buildroot}/usr/lib/systemd/system/ermete-recovery.service
install -m 0644 systemd/ermete-recovery.target %{buildroot}/usr/lib/systemd/system/ermete-recovery.target

mkdir -p %{buildroot}/usr/lib/systemd/system/greetd.service.d
install -m 0644 systemd/greetd-recovery-fallback.conf %{buildroot}/usr/lib/systemd/system/greetd.service.d/recovery-fallback.conf

%files
%{_bindir}/ermete-recovery-ui
/usr/lib/systemd/system/ermete-recovery.service
/usr/lib/systemd/system/ermete-recovery.target
%dir /usr/lib/systemd/system/greetd.service.d
/usr/lib/systemd/system/greetd.service.d/recovery-fallback.conf

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-recovery Pre-Boot GUI Wayland Kiosk (`cage` + `ermete-recovery-ui`)
- Automatic isolation to ermete-recovery.target when greetd fails StartLimitBurst=3 times
- Visual 1-click rollback to Bedrock Stable Commit (`8aa3fd4`) and previous stable OSTree deployments
