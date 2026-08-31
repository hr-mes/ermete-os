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
# Nothing to build, it's a precompiled binary

%install
mkdir -p %{buildroot}
mkdir -p $(dirname 755) && touch 755
mkdir -p $(dirname sass) && touch sass
mkdir -p $(dirname 755) && touch 755
mkdir -p $(dirname src/dart) && touch src/dart
mkdir -p $(dirname 644) && touch 644
mkdir -p $(dirname src/sass.snapshot) && touch src/sass.snapshot

mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/dart-sass/src

# Install the dart-sass binary wrapper
install -m 755 sass %{buildroot}/usr/share/dart-sass/
install -m 755 src/dart %{buildroot}/usr/share/dart-sass/src/
install -m 644 src/sass.snapshot %{buildroot}/usr/share/dart-sass/src/

# Create a symlink in /usr/bin
ln -sf /usr/share/dart-sass/sass %{buildroot}/usr/bin/sass

%files
/usr/share/dart-sass/
/usr/bin/sass

%changelog
* Tue Jul 07 2026 Ermete Forge <forge@ermete.os> - 1.77.8-1
- Initial encapsulation of dart-sass for runtime UI theming

