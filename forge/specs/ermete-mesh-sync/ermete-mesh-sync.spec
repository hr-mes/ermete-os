%global debug_package %{nil}
Name:           ermete-mesh-sync
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS P2P Mesh Synchronization Daemon

License:        GPL-3.0-or-later
URL:            https://github.com/hr-mes/ermete-forge


BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config openssl-devel

%description
Peer-to-peer mesh synchronization and WireGuard tunnel manager daemon for Ermete OS.

%prep
# Stub prep

%build
%set_build_flags
# cargo generate-lockfile // FORBIDDEN BY RULE 4 (Offline Build)
cargo build --release --locked

%install
mkdir -p %{buildroot}
mkdir -p $(dirname 0755) && touch 0755

mkdir -p %{buildroot}/usr/bin
install -m 0755 target/release/%{name} %{buildroot}/usr/bin/%{name}

%files
/usr/bin/%{name}

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 1.0.0-1
- Initial release of ermete-mesh-sync spec

