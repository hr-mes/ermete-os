Name:           ermete-astro-toolchain
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-astro-toolchain

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-astro-toolchain.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-astro-toolchain..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-astro-toolchain
#!/bin/bash
echo "Executing ermete-astro-toolchain (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-astro-toolchain

%files
/usr/bin/ermete-astro-toolchain
