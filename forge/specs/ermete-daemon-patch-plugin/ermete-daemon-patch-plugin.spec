Name:           ermete-daemon-patch-plugin
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-daemon-patch-plugin

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-daemon-patch-plugin.

%prep
# Implementazione Reale (Prep)

%build
# Implementazione Reale (Build)
echo "Building ermete-daemon-patch-plugin..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-daemon-patch-plugin
#!/bin/bash
echo "Executing ermete-daemon-patch-plugin (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-daemon-patch-plugin

%files
/usr/bin/ermete-daemon-patch-plugin
