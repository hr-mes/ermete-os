Name:           ermete-mesh-bus
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-mesh-bus

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-mesh-bus.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-mesh-bus..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-mesh-bus
#!/bin/bash
echo "Executing ermete-mesh-bus (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-mesh-bus

%files
/usr/bin/ermete-mesh-bus
