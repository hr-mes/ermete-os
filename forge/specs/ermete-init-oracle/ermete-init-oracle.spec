Name:           ermete-init-oracle
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-init-oracle

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-init-oracle.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-init-oracle) && touch %{buildroot}/usr/bin/ermete-init-oracle


%files
/usr/bin/ermete-init-oracle
