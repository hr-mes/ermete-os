%global debug_package %{nil}
Name:           ermete-niri-ipc
Version:        0.1.0
Release:        1%{?dist}
Summary:        Async IPC library for Niri window manager in Ermete OS

License:        MIT
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config

%description
Async IPC library crate for interacting with the Niri Wayland compositor via UNIX sockets.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release

%install
mkdir -p %{buildroot}%{_libdir}/ermete
if [ -f target/release/libermete_niri_ipc.rlib ]; then
    install -m 0644 target/release/libermete_niri_ipc.rlib %{buildroot}%{_libdir}/ermete/
fi

%files
%{_libdir}/ermete/

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial release of ermete-niri-ipc spec
