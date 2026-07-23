%global debug_package %{nil}
Name:           ermete-backup
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Time Machine & Btrfs Home Snapshot Manager

License:        MIT
Source0:        ermete-backup-%{version}.tar.gz

BuildRequires:  rust cargo gcc gcc-c++ gtk4-devel glib2-devel pkgconf-pkg-config
Requires:       gtk4 glib2 btrfs-progs systemd

%description
Instant zero-overhead Btrfs Copy-on-Write (CoW) Home snapshot manager and Time Machine GUI (`ermete-backup-ui`).
Includes user D-Bus daemon (`ermete-backup-daemon`) and automatic hourly timer (`ermete-backup-hourly.timer`).

%prep
%autosetup

%build
%set_build_flags
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/ermete-backup-daemon %{buildroot}%{_bindir}/ermete-backup-daemon
install -m 0755 target/release/ermete-backup-ui %{buildroot}%{_bindir}/ermete-backup-ui

mkdir -p %{buildroot}/usr/lib/systemd/system
install -m 0644 systemd/ermete-backup.service %{buildroot}/usr/lib/systemd/system/ermete-backup.service
install -m 0644 systemd/ermete-backup-hourly.timer %{buildroot}/usr/lib/systemd/system/ermete-backup-hourly.timer
install -m 0644 systemd/ermete-backup-hourly.service %{buildroot}/usr/lib/systemd/system/ermete-backup-hourly.service

mkdir -p %{buildroot}/usr/share/dbus-1/system.d
install -m 0644 systemd/org.ermete.Backup1.conf %{buildroot}/usr/share/dbus-1/system.d/org.ermete.Backup1.conf

%files
%{_bindir}/ermete-backup-daemon
%{_bindir}/ermete-backup-ui
/usr/lib/systemd/system/ermete-backup.service
/usr/lib/systemd/system/ermete-backup-hourly.timer
/usr/lib/systemd/system/ermete-backup-hourly.service
/usr/share/dbus-1/system.d/org.ermete.Backup1.conf

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-backup Btrfs CoW snapshot daemon and Time Machine GUI
- Automatic hourly snapshot creation via systemd user timer
- Instant single-click rollback and snapshot creation
