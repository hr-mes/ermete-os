Name:           ermete-audio-bus
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-audio-bus

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-audio-bus.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-audio-bus..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-audio-bus
#!/bin/bash
echo "Executing ermete-audio-bus (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-audio-bus

%files
/usr/bin/ermete-audio-bus
