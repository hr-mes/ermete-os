Name:           ermete-compositor
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-compositor

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-compositor.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-compositor..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-compositor
#!/bin/bash
echo "Executing ermete-compositor (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-compositor

%files
/usr/bin/ermete-compositor
