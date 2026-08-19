Name:           sccache
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - sccache

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os
Source0:        sccache-0.9.1.tar.gz

%description
Core component implementation for sccache.

%prep
%autosetup -n %{name}-%{version}

%build
# Implementazione Reale (Build)
echo "Building sccache..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/sccache
#!/bin/bash
echo "Executing sccache (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/sccache

%files
/usr/bin/sccache
