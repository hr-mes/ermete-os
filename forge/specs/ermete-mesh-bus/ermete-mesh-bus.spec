Name:           ermete-mesh-bus
Version:        1.0.0
Release:        1%{?dist}
Summary:        Ermete OS Core Component - ermete-mesh-bus

License:        GPLv3
URL:            https://github.com/hr-mes/ermete-os

%description
Core component implementation for ermete-mesh-bus.

%prep
# Stub prep

%build
# Stubbed

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}$(dirname /usr/bin/ermete-mesh-bus) && touch %{buildroot}/usr/bin/ermete-mesh-bus


%files
/usr/bin/ermete-mesh-bus
