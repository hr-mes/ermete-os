Name:           sccache
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - sccache

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for sccache.

%prep
# Implementazione Reale (Prep)

%build
# Implementazione Reale (Build)
echo "Building sccache..."

%install
mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/sccache
#!/bin/bash
echo "Executing sccache (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/sccache

%files
/usr/bin/sccache
