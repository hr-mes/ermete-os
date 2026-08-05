%global debug_package %{nil}
Name:           ermete-style
Version:        0.1.0
Release:        1%{?dist}
Summary:        Shared CSS and GTK4 design system styles for Ermete OS

License:        MIT
URL:            https://github.com/hr-mes/ermete-forge
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.80.0
BuildRequires:  cargo
BuildRequires:  gcc gcc-c++ pkgconf-pkg-config
BuildRequires:  gtk4-devel

%description
Shared GTK4 CSS theme system and Rust style integration library for Ermete OS UI components.

%prep
%autosetup -c

%build
%set_build_flags
cargo generate-lockfile
cargo build --release

%install
mkdir -p %{buildroot}%{_libdir}/ermete
if [ -f target/release/libermete_style.rlib ]; then
    install -m 0644 target/release/libermete_style.rlib %{buildroot}%{_libdir}/ermete/
fi

%files
%{_libdir}/ermete/

%changelog
* Wed Aug 05 2026 Ermete Forge <forge@ermete.os> - 0.1.0-1
- Initial release of ermete-style spec
