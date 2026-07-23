Name:           ermete-starship
%global debug_package %{nil}
Version:        1.26.0
Release:        1%{?dist}
Summary:        The minimal, blazing-fast, and infinitely customizable prompt for any shell!
License:        ISC
URL:            https://starship.rs
Source0:        https://github.com/starship/starship/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  cmake
BuildRequires:  mold
BuildRequires:  openssl-devel

%description
Starship is the minimal, blazing-fast, and infinitely customizable prompt for any shell!
Compiled natively in Ermete Forge with extreme optimizations.

%prep
%autosetup -n starship-%{version}

# Disable GCC LTO as it conflicts with Rust LLVM LTO and mold
%define _lto_cflags %{nil}

%build
%set_build_flags
export RUSTFLAGS="%{rustflags}"
export CARGO_PROFILE_RELEASE_LTO="thin"
# Disable GCC LTO in CFLAGS to prevent cc crate linkage errors with Rust LLVM
export CFLAGS="$(echo $CFLAGS | sed 's/-flto=auto//g')"
export CXXFLAGS="$(echo $CXXFLAGS | sed 's/-flto=auto//g')"
export LDFLAGS="$(echo $LDFLAGS | sed 's/-flto=auto//g')"
# The global rpmmacros will inject -C target-cpu=x86-64-v3 and mold linker
%set_build_flags
%cargo_build

%install
rm -rf %{buildroot}
install -Dm755 target/release/starship %{buildroot}/usr/bin/starship

%files
/usr/bin/starship

%changelog
* Sun Jun 28 2026 Ermete Forge <forge@ermete.os> - 1.22.1-1
- Native OCI build with x86-64-v3 optimizations
