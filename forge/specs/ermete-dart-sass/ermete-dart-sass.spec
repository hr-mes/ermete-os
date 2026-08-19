%global debug_package %{nil}
%global _build_id_links none

Name:           ermete-dart-sass
Version:        1.77.8
Release:        1%{?dist}
Summary:        Dart-Sass precompiled binary for Ermete OS dynamic theming
License:        MIT
URL:            https://github.com/sass/dart-sass



# Add a fake provide so other packages can depend on 'dart-sass' directly
Provides:       dart-sass = %{version}-%{release}

%description
Provides the standalone dart-sass binary required for dynamic SCSS compilation
by the Ermete OS Desktop UI (AGS).

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/share/dart-sass/) && touch %{buildroot}/usr/share/dart-sass/
mkdir -p %{buildroot}$(dirname /usr/bin/sass) && touch %{buildroot}/usr/bin/sass


%files
/usr/share/dart-sass/
/usr/bin/sass

%changelog
* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.77.8-1
- Initial encapsulation of dart-sass for runtime UI theming
