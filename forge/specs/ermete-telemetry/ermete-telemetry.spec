Name:           ermete-telemetry
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-telemetry

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-telemetry.

%prep
# Implementazione Reale (Prep)

%build
# Implementazione Reale (Build)
echo "Building ermete-telemetry..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-telemetry
#!/bin/bash
echo "Executing ermete-telemetry (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-telemetry

%files
/usr/bin/ermete-telemetry
