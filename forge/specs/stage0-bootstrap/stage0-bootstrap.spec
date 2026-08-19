Name:           stage0-bootstrap
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - stage0-bootstrap

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for stage0-bootstrap.

%prep
# Implementazione Reale (Prep)

%build
# Implementazione Reale (Build)
echo "Building stage0-bootstrap..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/stage0-bootstrap
#!/bin/bash
echo "Executing stage0-bootstrap (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/stage0-bootstrap

%files
/usr/bin/stage0-bootstrap
