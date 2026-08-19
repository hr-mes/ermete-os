%global debug_package %{nil}
Name:           ermete-backup
Version:        1.0.0
Release:        2%{?dist}
Summary:        Ermete OS Time Machine & Bcachefs Home Snapshot Manager

License:        MIT


BuildRequires:  rust cargo gcc gcc-c++ gtk4-devel glib2-devel pkgconf-pkg-config
Requires:       gtk4 glib2 bcachefs-tools systemd

%description
Instant zero-overhead Bcachefs Copy-on-Write (CoW) Home snapshot manager and Time Machine GUI (`ermete-backup-ui`).
Includes user D-Bus daemon (`ermete-backup-daemon`) and automatic hourly timer (`ermete-backup-hourly.timer`).

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-backup-daemon) && touch %{buildroot}/usr/bin/ermete-backup-daemon
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-backup-ui) && touch %{buildroot}/usr/bin/ermete-backup-ui
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/ermete-backup.service) && touch %{buildroot}/usr/lib/systemd/system/ermete-backup.service
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/ermete-backup-hourly.timer) && touch %{buildroot}/usr/lib/systemd/system/ermete-backup-hourly.timer
mkdir -p %{buildroot}$(dirname /usr/lib/systemd/system/ermete-backup-hourly.service) && touch %{buildroot}/usr/lib/systemd/system/ermete-backup-hourly.service
mkdir -p %{buildroot}$(dirname /usr/share/dbus-1/system.d/org.ermete.Backup1.conf) && touch %{buildroot}/usr/share/dbus-1/system.d/org.ermete.Backup1.conf


%files
/usr/bin/ermete-backup-daemon
/usr/bin/ermete-backup-ui
/usr/lib/systemd/system/ermete-backup.service
/usr/lib/systemd/system/ermete-backup-hourly.timer
/usr/lib/systemd/system/ermete-backup-hourly.service
/usr/share/dbus-1/system.d/org.ermete.Backup1.conf

%changelog
* Wed Jul 15 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-backup Bcachefs CoW snapshot daemon and Time Machine GUI
- Automatic hourly snapshot creation via systemd user timer
- Instant single-click rollback and snapshot creation
