Name:           osbuild
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - osbuild

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for osbuild.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/osbuild) && touch %{buildroot}/usr/bin/osbuild


%files
/usr/bin/osbuild
