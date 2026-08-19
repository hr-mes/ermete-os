Name:           ermete-style
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-style

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-style.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-style) && touch %{buildroot}/usr/bin/ermete-style


%files
/usr/bin/ermete-style
