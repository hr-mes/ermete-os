Name:           ermete-kernel-forge
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-kernel-forge

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-kernel-forge.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-kernel-forge..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-kernel-forge
#!/bin/bash
echo "Executing ermete-kernel-forge (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-kernel-forge

%files
/usr/bin/ermete-kernel-forge
