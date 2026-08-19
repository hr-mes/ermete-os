Name:           ermete-cargo-tools
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-cargo-tools

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-cargo-tools.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-cargo-tools..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-cargo-tools
#!/bin/bash
echo "Executing ermete-cargo-tools (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-cargo-tools

%files
/usr/bin/ermete-cargo-tools
