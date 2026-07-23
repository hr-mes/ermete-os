Name:           ermete-niri
Version:        0.1.8
Release:        1.fc43
Summary:        Ermete-patched version of the Niri scrollable tiling compositor
License:        GPL-3.0-or-later

# Usa i sorgenti originali (Vanilla)
Source0:        https://github.com/YaLTeR/niri/archive/refs/tags/v%{version}.tar.gz



BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  systemd-devel
BuildRequires:  wayland-devel
BuildRequires:  mesa-libgbm-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  cairo-devel
BuildRequires:  pango-devel

# Evita conflitti con il niri vanilla di Fedora
Conflicts:      niri
Provides:       niri = %{version}-%{release}

%description
Ermete OS custom Wayland compositor based on Niri.
Features the Ermete 'Floating-First' UX injected at build time via spec patching,
retaining full upstream compatibility.

%prep
%autosetup -n niri-%{version}

%build
%set_build_flags
cargo build --release --locked

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/niri %{buildroot}%{_bindir}/niri

mkdir -p %{buildroot}%{_datadir}/wayland-sessions
install -m 0644 resources/niri.desktop %{buildroot}%{_datadir}/wayland-sessions/niri.desktop

%files
%{_bindir}/niri
%{_datadir}/wayland-sessions/niri.desktop

%changelog
* Sat Jul 18 2026 Ermete Forge <forge@ermete.os> - 0.1.8-1
- Initial Ermete-patched Niri spec with soft-forking architecture.
