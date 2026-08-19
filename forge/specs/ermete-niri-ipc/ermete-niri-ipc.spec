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
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/lib64/ermete/) && touch %{buildroot}/usr/lib64/ermete/


%files
/usr/lib64/ermete/

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial release of ermete-niri-ipc spec
