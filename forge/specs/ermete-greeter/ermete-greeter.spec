Name:           ermete-greeter
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-greeter

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-greeter.

%prep
# Stub prep

%build
# Implementazione Reale (Build)
echo "Building ermete-greeter..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/ermete-greeter
#!/bin/bash
echo "Executing ermete-greeter (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/ermete-greeter

%files
/usr/bin/ermete-greeter
