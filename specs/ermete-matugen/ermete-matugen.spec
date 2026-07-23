Name:           ermete-matugen
%global debug_package %{nil}
Version:        4.1.0
Release:        1%{?dist}
Summary:        Material Design 3 color generation tool
License:        GPLv3
URL:            https://github.com/InioX/matugen
Source0:        https://github.com/InioX/matugen/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  mold

%description
Matugen is a tool to generate a colorscheme from an image or a color and export it to a file.
Compiled natively in Ermete Forge with extreme optimizations.

%prep
%autosetup -n matugen-%{version}

# Disable GCC LTO as it conflicts with Rust LLVM LTO and mold
%define _lto_cflags %{nil}

%build
%set_build_flags
export RUSTFLAGS="%{rustflags}"
export CARGO_PROFILE_RELEASE_LTO="thin"
export CFLAGS="$(echo $CFLAGS | sed 's/-flto=auto//g')"
export CXXFLAGS="$(echo $CXXFLAGS | sed 's/-flto=auto//g')"
export LDFLAGS="$(echo $LDFLAGS | sed 's/-flto=auto//g')"
%set_build_flags
%cargo_build

%install
rm -rf %{buildroot}
install -Dm755 target/release/matugen %{buildroot}/usr/bin/matugen

%files
/usr/bin/matugen

%changelog
* Sun Jun 28 2026 Ermete Forge <forge@ermete.os> - 2.4.0-1
- Native OCI build with x86-64-v3 optimizations
