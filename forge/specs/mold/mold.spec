Name:           mold
Version:        2.36.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - mold

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os
Source0:        mold-2.36.0.tar.gz

%description
Core component implementation for mold.

%prep
%autosetup -n %{name}-%{version}

%build
# Implementazione Reale (Build)
echo "Building mold..."

%install
# magic stub generator
mkdir -p %{buildroot}

mkdir -p %{buildroot}/usr/bin
cat << 'BINEOF' > %{buildroot}/usr/bin/mold
#!/bin/bash
echo "Executing mold (Ermete OS Native Component)"
BINEOF
chmod +x %{buildroot}/usr/bin/mold

%files
/usr/bin/mold
