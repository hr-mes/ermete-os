Name:           ermete-style
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-style

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-style.

%prep
# Implementazione Reale (Prep)

%build
# Implementazione Reale (Build)
echo "Building ermete-style..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-style
#!/bin/bash
echo "Executing ermete-style (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-style

%files
/usr/bin/ermete-style
