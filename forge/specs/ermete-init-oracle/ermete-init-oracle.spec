Name:           ermete-init-oracle
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-init-oracle

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-init-oracle.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-init-oracle..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-init-oracle
#!/bin/bash
echo "Executing ermete-init-oracle (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-init-oracle

%files
/usr/bin/ermete-init-oracle
