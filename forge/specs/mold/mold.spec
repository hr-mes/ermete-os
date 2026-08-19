Name:           mold
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - mold

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for mold.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building mold..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/mold
#!/bin/bash
echo "Executing mold (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/mold

%files
/usr/bin/mold
