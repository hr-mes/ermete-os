%global debug_package %{nil}
Name:           ermete-dock
Version:        1.0.0
Release:        1%{?dist}
Summary:        Visual Dock and taskbar application logic for Ermete OS

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config
BuildRequires:  gtk4-devel
BuildRequires:  gtk4-layer-shell-devel
BuildRequires:  glib2-devel

%description
Visual Dock and taskbar application library component for Ermete OS built with GTK4 and gtk4-layer-shell.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}/usr/lib64/ermete
if [ -f target/release/libermete_dock.rlib ]; then
    install -m 0644 target/release/libermete_dock.rlib %{buildroot}/usr/lib64/ermete/
fi

%files
/usr/lib64/ermete/

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-dock spec
