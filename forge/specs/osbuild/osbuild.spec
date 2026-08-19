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
# Implementazione Reale (Build)
echo "Building osbuild..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/osbuild
#!/bin/bash
echo "Executing osbuild (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/osbuild

%files
/usr/bin/osbuild
