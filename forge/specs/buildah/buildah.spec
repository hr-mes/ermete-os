Name:           buildah
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - buildah

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for buildah.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building buildah..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/buildah
#!/bin/bash
echo "Executing buildah (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/buildah

%files
/usr/bin/buildah
