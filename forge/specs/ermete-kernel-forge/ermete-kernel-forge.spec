Name:           ermete-kernel-forge
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-kernel-forge

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-kernel-forge.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-kernel-forge) && touch %{buildroot}/usr/bin/ermete-kernel-forge


%files
/usr/bin/ermete-kernel-forge
